//! Recipe collection operations against a fake AnyList HTTP API.
//!
//! The fake answers `data/user-data/get` with a canned recipe data response
//! and records every request, so tests can check how many round trips an
//! operation makes and decode the operation it sent.

use std::sync::{Arc, Mutex};

use anylist_rs::protobuf::anylist::{
    PbRecipe, PbRecipeCollection, PbRecipeDataResponse, PbRecipeOperation, PbRecipeOperationList,
    PbUserDataResponse,
};
use anylist_rs::{AnyListClient, AnyListError, SavedTokens};
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const USER_DATA: &str = "/data/user-data/get";
const RECIPE_UPDATE: &str = "/data/user-recipe-data/update";
const RECIPE_DATA_ID: &str = "recipe-data-1";

struct Request {
    path: String,
    /// The "operations" multipart field.
    operations: Vec<u8>,
}

struct FakeApi {
    url: String,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl FakeApi {
    async fn start(recipe_data: PbRecipeDataResponse) -> Self {
        let user_data = PbUserDataResponse {
            recipe_data_response: Some(recipe_data),
            ..Default::default()
        }
        .encode_to_vec();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);

        tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_request(&mut stream).await;
                let body = if request.path == USER_DATA {
                    user_data.clone()
                } else {
                    Vec::new()
                };
                recorded.lock().unwrap().push(request);

                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/x-protobuf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).await.unwrap();
                stream.write_all(&body).await.unwrap();
                stream.shutdown().await.unwrap();
            }
        });

        Self { url, requests }
    }

    fn client(&self) -> AnyListClient {
        AnyListClient::from_tokens(SavedTokens::new("access", "refresh", "user-1", false))
            .unwrap()
            .with_base_url(&self.url)
    }

    fn paths(&self) -> Vec<String> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.path.clone())
            .collect()
    }

    /// The single operation sent to the recipe update endpoint.
    fn sent_operation(&self) -> PbRecipeOperation {
        let mut operations = self.sent_operations();
        assert_eq!(operations.len(), 1, "expected exactly one operation");
        operations.remove(0)
    }

    /// The operations sent to the recipe update endpoint, one per request.
    fn sent_operations(&self) -> Vec<PbRecipeOperation> {
        let requests = self.requests.lock().unwrap();
        requests
            .iter()
            .filter(|r| r.path == RECIPE_UPDATE)
            .map(|r| {
                let mut list = PbRecipeOperationList::decode(r.operations.as_slice()).unwrap();
                assert_eq!(
                    list.operations.len(),
                    1,
                    "expected one operation per request"
                );
                list.operations.remove(0)
            })
            .collect()
    }
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Request {
    let mut buf = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 4096];
        let n = stream.read(&mut chunk).await.unwrap();
        assert!(n > 0, "connection closed before headers were complete");
        buf.extend_from_slice(&chunk[..n]);
        if let Some(i) = find(&buf, b"\r\n\r\n") {
            break i + 4;
        }
    };

    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let path = head.split_whitespace().nth(1).unwrap().to_string();
    let header = |name: &str| {
        head.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name)
                .then(|| value.trim().to_string())
        })
    };
    let content_length: usize = header("content-length")
        .expect("request without Content-Length")
        .parse()
        .unwrap();

    while buf.len() < header_end + content_length {
        let mut chunk = [0u8; 4096];
        let n = stream.read(&mut chunk).await.unwrap();
        assert!(n > 0, "connection closed before body was complete");
        buf.extend_from_slice(&chunk[..n]);
    }
    let body = &buf[header_end..header_end + content_length];

    let boundary = header("content-type")
        .and_then(|ct| ct.split_once("boundary=").map(|(_, b)| b.to_string()))
        .expect("request is not multipart");
    Request {
        path,
        operations: multipart_field(body, &boundary, "operations"),
    }
}

/// The bytes of the named field in a multipart/form-data body.
fn multipart_field(body: &[u8], boundary: &str, name: &str) -> Vec<u8> {
    let delimiter = format!("--{boundary}");
    let mut rest = body;
    while let Some(start) = find(rest, delimiter.as_bytes()) {
        rest = &rest[start + delimiter.len()..];
        let Some(headers_end) = find(rest, b"\r\n\r\n") else {
            break;
        };
        let headers = String::from_utf8_lossy(&rest[..headers_end]);
        let content = &rest[headers_end + 4..];
        let end = find(content, format!("\r\n{delimiter}").as_bytes()).unwrap();
        if headers.contains(&format!("name=\"{name}\"")) {
            return content[..end].to_vec();
        }
        rest = &content[end..];
    }
    panic!("multipart field {name:?} not found");
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn recipe_data(collections: Vec<PbRecipeCollection>) -> PbRecipeDataResponse {
    PbRecipeDataResponse {
        recipe_data_id: Some(RECIPE_DATA_ID.to_string()),
        recipe_collections: collections,
        recipes: ["r1", "r2"]
            .iter()
            .map(|id| PbRecipe {
                identifier: id.to_string(),
                name: Some(format!("Recipe {id}")),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

fn collection(id: &str, name: &str, recipe_ids: &[&str]) -> PbRecipeCollection {
    PbRecipeCollection {
        identifier: id.to_string(),
        name: Some(name.to_string()),
        recipe_ids: recipe_ids.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

fn handler_id(op: &PbRecipeOperation) -> &str {
    op.metadata.as_ref().unwrap().handler_id.as_deref().unwrap()
}

#[tokio::test]
async fn get_recipe_collections_maps_named_collections() {
    let api = FakeApi::start(recipe_data(vec![
        collection("c1", "Dinners", &["r1", "r2"]),
        PbRecipeCollection {
            identifier: "unnamed".to_string(),
            ..Default::default()
        },
    ]))
    .await;

    let collections = api.client().get_recipe_collections().await.unwrap();

    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0].id(), "c1");
    assert_eq!(collections[0].name(), "Dinners");
    assert_eq!(collections[0].recipe_ids(), ["r1", "r2"]);
}

#[tokio::test]
async fn create_recipe_collection_sends_recipe_data_id_then_sets_icon() {
    let api = FakeApi::start(recipe_data(vec![])).await;

    let created = api
        .client()
        .create_recipe_collection("Dinners")
        .await
        .unwrap();

    assert_eq!(created.name(), "Dinners");
    assert!(created.recipe_ids().is_empty());
    assert_eq!(api.paths(), [USER_DATA, RECIPE_UPDATE, RECIPE_UPDATE]);

    let ops = api.sent_operations();
    assert_eq!(handler_id(&ops[0]), "new-recipe-collection");
    assert_eq!(ops[0].recipe_data_id.as_deref(), Some(RECIPE_DATA_ID));
    assert!(ops[0].recipe_collection_ids.is_empty());
    assert!(ops[0].recipe_ids.is_empty());
    let sent = ops[0].recipe_collection.clone().unwrap();
    assert_eq!(sent.identifier, created.id());
    assert_eq!(sent.name.as_deref(), Some("Dinners"));
    assert!(sent.recipe_ids.is_empty());
    assert!(sent.collection_settings.is_none());

    // Like the web client, the new collection then gets the default icon.
    assert_eq!(handler_id(&ops[1]), "set-recipe-collection-icon");
    let icon = ops[1]
        .recipe_collection
        .clone()
        .unwrap()
        .collection_settings
        .unwrap()
        .icon
        .unwrap();
    assert_eq!(
        icon.icon_name.as_deref(),
        Some("food_and_cooking/stack_of_recipe_cards")
    );
}

#[tokio::test]
async fn create_recipe_collection_with_recipe_ids_adds_them_after_creating() {
    let api = FakeApi::start(recipe_data(vec![])).await;
    let ids = vec!["r1".to_string(), "r2".to_string()];

    let created = api
        .client()
        .create_recipe_collection_with_recipe_ids("Dinners", ids.clone())
        .await
        .unwrap();

    assert_eq!(created.recipe_ids(), ids.as_slice());
    let ops = api.sent_operations();
    let handlers: Vec<&str> = ops.iter().map(handler_id).collect();
    assert_eq!(
        handlers,
        [
            "new-recipe-collection",
            "set-recipe-collection-icon",
            "add-recipes-to-collection"
        ]
    );
    assert!(ops[0]
        .recipe_collection
        .as_ref()
        .unwrap()
        .recipe_ids
        .is_empty());
    assert_eq!(ops[2].recipe_collection.as_ref().unwrap().recipe_ids, ids);
}

#[tokio::test]
async fn create_recipe_collection_with_unknown_recipe_sends_nothing() {
    let api = FakeApi::start(recipe_data(vec![])).await;

    let result = api
        .client()
        .create_recipe_collection_with_recipe_ids("Dinners", vec!["missing".to_string()])
        .await;

    assert!(matches!(result, Err(AnyListError::NotFound(_))));
    assert_eq!(api.paths(), [USER_DATA]);
}

#[tokio::test]
async fn delete_recipe_collection_sends_the_stored_collection() {
    let api = FakeApi::start(recipe_data(vec![collection("c1", "Dinners", &["r1"])])).await;

    api.client().delete_recipe_collection("c1").await.unwrap();

    assert_eq!(api.paths(), [USER_DATA, RECIPE_UPDATE]);
    let op = api.sent_operation();
    assert_eq!(handler_id(&op), "remove-recipe-collection");
    assert_eq!(op.recipe_data_id.as_deref(), Some(RECIPE_DATA_ID));
    assert!(op.recipe_collection_ids.is_empty());
    let sent = op.recipe_collection.unwrap();
    assert_eq!(sent.identifier, "c1");
    assert_eq!(sent.name.as_deref(), Some("Dinners"));
    assert_eq!(sent.recipe_ids, ["r1"]);
}

#[tokio::test]
async fn add_recipe_to_collection_sends_only_the_added_recipe() {
    let api = FakeApi::start(recipe_data(vec![collection("c1", "Dinners", &["r1"])])).await;

    api.client()
        .add_recipe_to_collection("c1", "r2")
        .await
        .unwrap();

    assert_eq!(api.paths(), [USER_DATA, RECIPE_UPDATE]);
    let op = api.sent_operation();
    assert_eq!(handler_id(&op), "add-recipes-to-collection");
    assert_eq!(op.recipe_data_id.as_deref(), Some(RECIPE_DATA_ID));
    let sent = op.recipe_collection.unwrap();
    assert_eq!(sent.identifier, "c1");
    assert_eq!(sent.name.as_deref(), Some("Dinners"));
    assert_eq!(sent.recipe_ids, ["r2"]);
}

#[tokio::test]
async fn add_recipe_already_in_collection_sends_nothing() {
    let api = FakeApi::start(recipe_data(vec![collection("c1", "Dinners", &["r1"])])).await;

    api.client()
        .add_recipe_to_collection("c1", "r1")
        .await
        .unwrap();

    assert_eq!(api.paths(), [USER_DATA]);
}

#[tokio::test]
async fn remove_recipe_from_collection_sends_only_the_removed_recipe() {
    let api = FakeApi::start(recipe_data(vec![collection(
        "c1",
        "Dinners",
        &["r1", "r2"],
    )]))
    .await;

    api.client()
        .remove_recipe_from_collection("c1", "r1")
        .await
        .unwrap();

    assert_eq!(api.paths(), [USER_DATA, RECIPE_UPDATE]);
    let op = api.sent_operation();
    assert_eq!(handler_id(&op), "remove-recipes-from-collection");
    assert_eq!(op.recipe_data_id.as_deref(), Some(RECIPE_DATA_ID));
    let sent = op.recipe_collection.unwrap();
    assert_eq!(sent.name.as_deref(), Some("Dinners"));
    assert_eq!(sent.recipe_ids, ["r1"]);
}

#[tokio::test]
async fn remove_recipe_not_in_collection_sends_nothing() {
    let api = FakeApi::start(recipe_data(vec![collection("c1", "Dinners", &["r1"])])).await;

    api.client()
        .remove_recipe_from_collection("c1", "r2")
        .await
        .unwrap();

    assert_eq!(api.paths(), [USER_DATA]);
}

#[tokio::test]
async fn changing_an_unknown_collection_is_not_found_and_sends_nothing() {
    let api = FakeApi::start(recipe_data(vec![collection("c1", "Dinners", &[])])).await;
    let client = api.client();

    let add = client.add_recipe_to_collection("missing", "r1").await;
    let remove = client.remove_recipe_from_collection("missing", "r1").await;
    let delete = client.delete_recipe_collection("missing").await;

    assert!(matches!(add, Err(AnyListError::NotFound(_))));
    assert!(matches!(remove, Err(AnyListError::NotFound(_))));
    assert!(matches!(delete, Err(AnyListError::NotFound(_))));
    assert_eq!(api.paths(), [USER_DATA, USER_DATA, USER_DATA]);
}
