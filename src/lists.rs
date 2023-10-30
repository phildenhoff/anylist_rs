use crate::protobuf::anylist::{
    ListItem as ApiListItem, PbUserDataResponse, ShoppingListsResponse,
};
use prost::Message;
use reqwest::header;

pub struct ListItem {
    pub id: String,
    pub list_id: String,
    pub name: String,
    pub details: String,
    pub is_checked: bool,
}

pub struct List {
    pub id: String,
    pub name: String,
    pub items: Vec<ListItem>,
}

async fn get_user_data(
    signed_user_id: &str,
) -> Result<PbUserDataResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://www.anylist.com/data/user-data/get")
        .header(header::CONTENT_TYPE, "application/x-protobuf")
        .header("X-AnyLeaf-Signed-User-ID", signed_user_id)
        .send()
        .await?;

    let bytes = res.bytes().await?;
    let data = PbUserDataResponse::decode(bytes.as_ref())?;
    Ok(data)
}

fn transform_api_list_item(items: Vec<ApiListItem>) -> Vec<ListItem> {
    let mut result: Vec<ListItem> = Vec::new();
    for item in items {
        if let (Some(name), Some(details), Some(is_checked), Some(list_id)) =
            (item.name, item.details, item.checked, item.list_id)
        {
            let item = ListItem {
                id: item.identifier,
                list_id: list_id.clone(),
                name,
                details,
                is_checked,
            };
            result.push(item);
        }
    }
    result
}

fn lists_from_response(response: ShoppingListsResponse) -> Vec<List> {
    let mut lists: Vec<List> = Vec::new();
    for list in response.new_lists {
        if let Some(name) = list.name {
            let list = List {
                id: list.identifier,
                name,
                items: transform_api_list_item(list.items),
            };
            lists.push(list);
        }
    }
    lists
}

pub async fn get_lists(
    signed_user_id: &str,
) -> Result<Vec<List>, Box<dyn std::error::Error>> {
    let data = get_user_data(signed_user_id).await?;
    let lists = match data.shopping_lists_response {
        Some(ref res) => lists_from_response(res.clone()),
        None => Vec::new(),
    };
    Ok(lists)
}
