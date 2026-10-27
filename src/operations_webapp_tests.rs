//! Byte-for-byte comparisons with requests the AnyList web client sent on
//! 2026-10-27 (`tests/fixtures/webapp_2026_10_27`, captured with
//! `tools/webapp_capture`). Each test replays the web client's IDs and the
//! state it had, runs a builder, and compares every request it returns with
//! the captured ones.

use std::path::Path;

use prost::Message;
use serde_json::Value;

use super::*;
use crate::protobuf::anylist::{
    PbCalendarOperationList, PbListCategoryGroupResponse, PbListFolderOperationList,
    PbRecipeOperationList,
};

pub(super) fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/webapp_2026_10_27")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

pub(super) fn s(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or_else(|| panic!("not a string: {value}"))
        .to_string()
}

pub(super) fn op_id(operation: &Value) -> String {
    s(&operation["metadata"]["operationId"])
}

pub(super) fn user_id(fixture: &Value) -> String {
    s(&fixture["operations"][0]["metadata"]["userId"])
}

/// The protobuf the web client sent, decoded with the crate's types.
pub(super) fn body<M: Message + Default>(fixture: &Value) -> M {
    M::decode(hex::decode(s(&fixture["hex"])).unwrap().as_slice()).unwrap()
}

/// Asserts that `requests` are the captured `fixtures`, in order.
///
/// The web client writes some fields in the order its schema declares them
/// rather than in field-number order (an ingredient's identifier first, for
/// one); prost always writes field-number order. Protobuf readers treat both
/// the same, so a request matches when its bytes are the capture's, or the
/// capture's re-encoded in field-number order, as long as re-encoding loses
/// nothing.
pub(super) fn assert_requests(requests: &[Request], fixtures: &[&str]) {
    assert_eq!(
        requests.iter().map(|r| r.endpoint).collect::<Vec<_>>(),
        fixtures
            .iter()
            .map(|name| s(&fixture(name)["endpoint"]))
            .map(|e| &*e.trim_start_matches('/').to_string().leak())
            .collect::<Vec<_>>(),
        "endpoints"
    );
    for (request, name) in requests.iter().zip(fixtures) {
        let expected = fixture(name);
        let captured = hex::decode(s(&expected["hex"])).unwrap();
        if request.body == captured {
            continue;
        }
        let canonical = reencode(&expected, &captured);
        assert_eq!(
            canonical.len(),
            captured.len(),
            "{name} has fields the crate's schema lacks"
        );
        if request.body != canonical {
            let ours = debug_body(&expected, &request.body);
            let theirs = debug_body(&expected, &captured);
            panic!("{name} differs\n--- ours\n{ours}\n--- web client\n{theirs}");
        }
    }
}

/// The capture decoded with the crate's types and encoded again.
fn reencode(fixture: &Value, bytes: &[u8]) -> Vec<u8> {
    match fixture["type"].as_str().unwrap() {
        "PBListOperationList" => PbListOperationList::decode(bytes).unwrap().encode_to_vec(),
        "PBStarterListOperationList" => PbStarterListOperationList::decode(bytes)
            .unwrap()
            .encode_to_vec(),
        "PBListSettingsOperationList" => PbListSettingsOperationList::decode(bytes)
            .unwrap()
            .encode_to_vec(),
        "PBListFolderOperationList" => PbListFolderOperationList::decode(bytes)
            .unwrap()
            .encode_to_vec(),
        "PBRecipeOperationList" => PbRecipeOperationList::decode(bytes)
            .unwrap()
            .encode_to_vec(),
        "PBCalendarOperationList" => PbCalendarOperationList::decode(bytes)
            .unwrap()
            .encode_to_vec(),
        other => panic!("unexpected type {other}"),
    }
}

fn debug_body(fixture: &Value, bytes: &[u8]) -> String {
    match fixture["type"].as_str().unwrap() {
        "PBListOperationList" => format!("{:#?}", PbListOperationList::decode(bytes).unwrap()),
        "PBStarterListOperationList" => {
            format!("{:#?}", PbStarterListOperationList::decode(bytes).unwrap())
        }
        "PBListSettingsOperationList" => {
            format!("{:#?}", PbListSettingsOperationList::decode(bytes).unwrap())
        }
        "PBListFolderOperationList" => {
            format!("{:#?}", PbListFolderOperationList::decode(bytes).unwrap())
        }
        "PBRecipeOperationList" => format!("{:#?}", PbRecipeOperationList::decode(bytes).unwrap()),
        "PBCalendarOperationList" => {
            format!("{:#?}", PbCalendarOperationList::decode(bytes).unwrap())
        }
        other => panic!("unexpected type {other}"),
    }
}

/// The list "zz capture list" as the web client knew it: its category group
/// (from the new-shopping-list request) and, optionally, rules.
struct CaptureList {
    list_id: String,
    response: PbListResponse,
}

impl CaptureList {
    fn new() -> Self {
        let created: PbListOperationList = body(&fixture("lists/0004-list-create.json"));
        let operation = &created.operations[0];
        let group = operation.updated_category_group.clone().unwrap();
        Self {
            list_id: operation.list_id.clone().unwrap(),
            response: PbListResponse {
                list_id: operation.list_id.clone(),
                category_group_responses: vec![PbListCategoryGroupResponse {
                    category_group: Some(group),
                    ..Default::default()
                }],
                ..Default::default()
            },
        }
    }

    fn group_id(&self) -> String {
        self.group().identifier.clone().unwrap()
    }

    fn group(&self) -> &PbListCategoryGroup {
        self.response.category_group_responses[0]
            .category_group
            .as_ref()
            .unwrap()
    }

    fn context(&self) -> (ListContext<'_>, String) {
        (
            ListContext {
                list_id: &self.list_id,
                response: Some(&self.response),
                category_group_id: None,
                new_items_on_top: false,
            },
            self.group_id(),
        )
    }
}

fn with_group<'a>(context: ListContext<'a>, group_id: &'a str) -> ListContext<'a> {
    ListContext {
        category_group_id: Some(group_id),
        ..context
    }
}

/// The item a captured request carried.
fn item_in(fixture_name: &str, operation: usize) -> PbListItem {
    let list: PbListOperationList = body(&fixture(fixture_name));
    let operation = &list.operations[operation];
    operation
        .list_item
        .clone()
        .or_else(|| operation.list.as_ref().map(|l| l.items[0].clone()))
        .unwrap()
}

fn starter_list(id: &str, items: Vec<PbListItem>) -> PbStarterList {
    PbStarterList {
        identifier: id.to_string(),
        items,
        ..Default::default()
    }
}

#[test]
fn create_list() {
    let shopping = fixture("lists/0004-list-create.json");
    let starter = fixture("lists/0005-list-create.json");
    let settings = fixture("lists/0006-list-create.json");
    let icon = fixture("lists/0010-list-create.json");

    let new_list = &shopping["operations"][0];
    let mut replay = vec![
        s(&new_list["listId"]),
        s(&new_list["updatedCategoryGroup"]["identifier"]),
    ];
    for category in new_list["updatedCategoryGroup"]["categories"]
        .as_array()
        .unwrap()
    {
        replay.push(s(&category["identifier"]));
    }
    replay.push(op_id(new_list));
    for operation in starter["operations"].as_array().unwrap() {
        replay.push(op_id(operation));
    }
    replay.push(s(
        &settings["operations"][0]["updatedSettings"]["identifier"]
    ));
    for operation in settings["operations"].as_array().unwrap() {
        replay.push(op_id(operation));
    }
    replay.push(op_id(&icon["operations"][0]));

    let mut ids = Ids::replay(user_id(&shopping), replay);
    let created = super::create_list(
        &mut ids,
        NewList {
            name: "zz capture list",
            email: &s(&new_list["list"]["sharedUsers"][0]["email"]),
            folder_id: &s(&new_list["listFolderId"]),
            theme_id: DEFAULT_LIST_THEME_ID,
        },
    );
    assert_requests(
        &created.requests,
        &[
            "lists/0004-list-create.json",
            "lists/0005-list-create.json",
            "lists/0006-list-create.json",
            "lists/0010-list-create.json",
        ],
    );
}

#[test]
fn rename_list() {
    let f = fixture("lists/0028-list-rename.json");
    let operation = &f["operations"][0];
    let mut ids = Ids::replay(user_id(&f), [op_id(operation)]);
    let requests = super::rename_list(&mut ids, &s(&operation["listId"]), "zz capture list 2");
    assert_requests(&requests, &["lists/0028-list-rename.json"]);
}

#[test]
fn delete_list() {
    let folders = fixture("lists/0072-list-delete.json");
    let settings = fixture("lists/0073-list-delete.json");
    let folder_operation = &folders["operations"][0];
    let settings_operation = &settings["operations"][0];
    let stored_settings = PbListSettings {
        identifier: s(&settings_operation["updatedSettings"]["identifier"]),
        user_id: Some(user_id(&settings)),
        list_id: Some(s(&settings_operation["updatedSettings"]["listId"])),
        should_hide_prices: Some(true),
        list_theme_id: Some(DEFAULT_LIST_THEME_ID.to_string()),
        ..Default::default()
    };
    let mut ids = Ids::replay(
        user_id(&folders),
        [op_id(folder_operation), op_id(settings_operation)],
    );
    let requests = super::delete_list(
        &mut ids,
        &s(&folder_operation["listDataId"]),
        &s(&folder_operation["originalParentFolderId"]),
        &s(&folder_operation["folderItems"][0]["identifier"]),
        &stored_settings,
    );
    assert_requests(
        &requests,
        &["lists/0072-list-delete.json", "lists/0073-list-delete.json"],
    );
}

#[test]
fn add_items() {
    let list = CaptureList::new();
    let (context, group_id) = list.context();
    let context = with_group(context, &group_id);
    for (name, category, fixture_name) in [
        ("zz milk", "dairy", "lists/0011-item-add.json"),
        ("zz bread", "bakery", "lists/0012-item-add.json"),
        ("zz eggs", "dairy", "lists/0013-item-add.json"),
    ] {
        let f = fixture(fixture_name);
        let operation = &f["operations"][0];
        let mut ids = Ids::replay(user_id(&f), [s(&operation["listItemId"]), op_id(operation)]);
        let mut item = new_list_item(&mut ids, context, name, Some(category));
        // The web client's item database also tags the item for price matching.
        item.price_matchup_tag = Some(name.trim_start_matches("zz ").to_string());
        assert_requests(&super::add_item(&mut ids, context, item), &[fixture_name]);
    }
}

#[test]
fn cross_off_and_uncross_item() {
    let bread = item_in("lists/0012-item-add.json", 0);
    let checked = fixture("lists/0014-item-cross-off.json");
    let recents = fixture("lists/0015-item-cross-off.json");
    let recents_operation = &recents["operations"][0];
    let mut ids = Ids::replay(
        user_id(&checked),
        [
            op_id(&checked["operations"][0]),
            s(&recents_operation["list"]["items"][0]["identifier"]),
            op_id(recents_operation),
        ],
    );
    let recent_items = starter_list(&s(&recents_operation["listId"]), vec![]);
    let requests = set_item_checked(&mut ids, &bread, true, Some(&recent_items));
    assert_requests(
        &requests,
        &[
            "lists/0014-item-cross-off.json",
            "lists/0015-item-cross-off.json",
        ],
    );

    let unchecked = fixture("lists/0016-item-uncross.json");
    let mut ids = Ids::replay(user_id(&unchecked), [op_id(&unchecked["operations"][0])]);
    let requests = set_item_checked(&mut ids, &bread, false, Some(&recent_items));
    assert_requests(&requests, &["lists/0016-item-uncross.json"]);
}

#[test]
fn edit_item_note_and_quantity() {
    let milk = item_in("lists/0011-item-add.json", 0);
    let details = fixture("lists/0017-item-edit-note-and-quantity.json");
    let mut ids = Ids::replay(user_id(&details), [op_id(&details["operations"][0])]);
    assert_requests(
        &set_item_details(&mut ids, &milk, "zz note"),
        &["lists/0017-item-edit-note-and-quantity.json"],
    );

    let quantity = fixture("lists/0018-item-edit-note-and-quantity.json");
    let mut ids = Ids::replay(user_id(&quantity), [op_id(&quantity["operations"][0])]);
    assert_requests(
        &set_item_quantity(&mut ids, &milk, parse_quantity("2 L")),
        &["lists/0018-item-edit-note-and-quantity.json"],
    );
}

#[test]
fn change_item_category() {
    let list = CaptureList::new();
    let (context, group_id) = list.context();
    let context = with_group(context, &group_id);
    let rule = fixture("lists/0019-item-change-category.json");
    let item_ops = fixture("lists/0020-item-change-category.json");
    // The milk item as stored after its note and quantity were set.
    let milk = PbListItem {
        details: Some("zz note".to_string()),
        quantity_pb: Some(parse_quantity("2 L")),
        ..item_in("lists/0011-item-add.json", 0)
    };
    let beverages = context.find_category("beverages").unwrap().clone();
    let mut ids = Ids::replay(
        user_id(&rule),
        [
            op_id(&item_ops["operations"][0]),
            op_id(&rule["operations"][0]),
            op_id(&item_ops["operations"][1]),
        ],
    );
    let requests = set_item_category(&mut ids, context, &milk, &beverages);
    assert_requests(
        &requests,
        &[
            "lists/0019-item-change-category.json",
            "lists/0020-item-change-category.json",
        ],
    );
}

#[test]
fn delete_item_files_it_under_recent_items() {
    let removed = fixture("lists/0044-item-delete.json");
    let recents = fixture("lists/0045-item-delete.json");
    let item = item_in("lists/0044-item-delete.json", 0);
    let recents_operation = &recents["operations"][0];
    let mut ids = Ids::replay(
        user_id(&removed),
        [
            op_id(&removed["operations"][0]),
            s(&recents_operation["list"]["items"][0]["identifier"]),
            op_id(recents_operation),
        ],
    );
    let recent_items = starter_list(&s(&recents_operation["listId"]), vec![]);
    let requests = delete_items(
        &mut ids,
        list_id_of(&item),
        std::slice::from_ref(&item),
        Some(&recent_items),
    );
    assert_requests(
        &requests,
        &["lists/0044-item-delete.json", "lists/0045-item-delete.json"],
    );
}

#[test]
fn cross_off_replaces_equal_recent_item() {
    // Crossing off eggs again: the earlier copy in Recent Items is replaced.
    let checked = fixture("lists/0060-list-delete-crossed-off.json");
    let recents = fixture("lists/0061-list-delete-crossed-off.json");
    let eggs = item_in("lists/0013-item-add.json", 0);
    let old_copy = item_in_starter("lists/0058-list-uncross-all.json");
    let remove = &recents["operations"][0];
    let add = &recents["operations"][1];
    let mut ids = Ids::replay(
        user_id(&checked),
        [
            op_id(&checked["operations"][0]),
            s(&add["list"]["items"][0]["identifier"]),
            op_id(remove),
            op_id(add),
        ],
    );
    let recent_items = starter_list(&s(&remove["listId"]), vec![old_copy]);
    let requests = set_item_checked(&mut ids, &eggs, true, Some(&recent_items));
    assert_requests(
        &requests,
        &[
            "lists/0060-list-delete-crossed-off.json",
            "lists/0061-list-delete-crossed-off.json",
        ],
    );
}

#[test]
fn delete_crossed_off_items() {
    // Eggs were filed under Recent Items when crossed off, so only the
    // removal is sent.
    let removed = fixture("lists/0062-list-delete-crossed-off.json");
    let eggs = item_in("lists/0062-list-delete-crossed-off.json", 0);
    let filed = item_in_starter_at("lists/0061-list-delete-crossed-off.json", 1);
    let mut ids = Ids::replay(user_id(&removed), [op_id(&removed["operations"][0])]);
    let recent_items = starter_list(filed.list_id.as_deref().unwrap(), vec![filed.clone()]);
    let requests = super::delete_crossed_off_items(
        &mut ids,
        list_id_of(&eggs),
        std::slice::from_ref(&eggs),
        Some(&recent_items),
    );
    assert_requests(&requests, &["lists/0062-list-delete-crossed-off.json"]);
}

fn item_in_starter(fixture_name: &str) -> PbListItem {
    item_in_starter_at(fixture_name, 0)
}

fn item_in_starter_at(fixture_name: &str, operation: usize) -> PbListItem {
    let list: PbStarterListOperationList = body(&fixture(fixture_name));
    let operation = &list.operations[operation];
    operation
        .list_item
        .clone()
        .or_else(|| operation.list.as_ref().map(|l| l.items[0].clone()))
        .unwrap()
}

#[test]
fn parse_quantities() {
    let q = parse_quantity("2 L");
    assert_eq!(
        (q.amount.as_deref(), q.unit.as_deref()),
        (Some("2"), Some("L"))
    );
    let q = parse_quantity("1 1/2 cups");
    assert_eq!(
        (q.amount.as_deref(), q.unit.as_deref()),
        (Some("1 1/2"), Some("cups"))
    );
    let q = parse_quantity("2");
    assert_eq!((q.amount.as_deref(), q.unit.as_deref()), (Some("2"), None));
    let q = parse_quantity("a pinch");
    assert_eq!(
        (q.amount.as_deref(), q.raw_quantity.as_deref()),
        (None, Some("a pinch"))
    );
}

#[test]
fn category_slugs() {
    assert_eq!(slug("Kids & Toys"), "kids-and-toys");
    assert_eq!(slug("  zz Capture Cat 2 "), "zz-capture-cat-2");
}

/// Like [`assert_requests`], but first clears what the web client's item
/// database adds to items (price matchup tags) in the captured list
/// requests, since the crate has no item database.
fn assert_requests_untagged(requests: &[Request], fixtures: &[&str]) {
    for (request, name) in requests.iter().zip(fixtures) {
        let expected = fixture(name);
        if expected["type"] != "PBListOperationList" {
            continue;
        }
        let mut theirs: PbListOperationList = body(&expected);
        for operation in &mut theirs.operations {
            if let Some(item) = operation.list_item.as_mut() {
                item.price_matchup_tag = None;
            }
        }
        let ours = PbListOperationList::decode(request.body.as_slice()).unwrap();
        assert_eq!(ours, theirs, "{name} differs");
    }
    assert_eq!(requests.len(), fixtures.len(), "number of requests");
}

/// The group "zz capture list" had once its categories were reordered.
fn sorted_group() -> PbListCategoryGroup {
    PbListCategoryGroup {
        logical_timestamp: Some(1),
        categories_logical_timestamp: Some(1),
        ..CaptureList::new().group().clone()
    }
}

#[test]
fn create_category_before_other() {
    let created = fixture("lists/0021-category-create.json");
    let sorted = fixture("lists/0022-category-create.json");
    let create = &created["operations"][0];
    let mut ids = Ids::replay(
        user_id(&created),
        [
            s(&create["updatedCategory"]["identifier"]),
            op_id(create),
            op_id(&sorted["operations"][0]),
        ],
    );
    let result = super::create_category(&mut ids, &sorted_group(), "zz Capture Cat");
    assert_requests(
        &result.requests,
        &[
            "lists/0021-category-create.json",
            "lists/0022-category-create.json",
        ],
    );
}

#[test]
fn rename_category() {
    let f = fixture("lists/0023-category-rename.json");
    let renamed: PbListOperationList = body(&f);
    let stored = PbListCategory {
        name: Some("zz Capture Cat".to_string()),
        ..renamed.operations[0].updated_category.clone().unwrap()
    };
    let mut ids = Ids::replay(user_id(&f), [op_id(&f["operations"][0])]);
    assert_requests(
        &super::rename_category(&mut ids, &stored, "zz Capture Cat 2"),
        &["lists/0023-category-rename.json"],
    );
}

#[test]
fn delete_category_moves_its_items_to_the_default() {
    let removed = fixture("lists/0024-category-delete.json");
    let rules = fixture("lists/0025-category-delete.json");
    let rule = fixture("lists/0026-category-delete.json");
    let item_ops = fixture("lists/0027-category-delete.json");
    let removed_body: PbListOperationList = body(&removed);
    let category = removed_body.operations[0]
        .updated_category_group
        .as_ref()
        .unwrap()
        .categories[0]
        .clone();
    let category_id = category.identifier.clone().unwrap();

    let mut group = sorted_group();
    let other = group.categories.last_mut().unwrap();
    other.sort_index = Some(20);
    group.categories.push(category.clone());
    let list = CaptureList::new();
    let response = PbListResponse {
        category_group_responses: vec![PbListCategoryGroupResponse {
            category_group: Some(group.clone()),
            ..Default::default()
        }],
        categorization_rules: vec![PbListCategorizationRule {
            category_id: Some(category_id.clone()),
            ..body::<PbListOperationList>(&rule).operations[0]
                .updated_categorization_rule
                .clone()
                .unwrap()
        }],
        ..list.response.clone()
    };
    let group_id = group.identifier.clone().unwrap();
    let context = ListContext {
        list_id: &list.list_id,
        response: Some(&response),
        category_group_id: Some(&group_id),
        new_items_on_top: false,
    };
    // Milk, filed under the category about to be deleted.
    let mut milk = item_in("lists/0027-category-delete.json", 0);
    milk.category_assignments[0].category_id = Some(category_id);

    let mut ids = Ids::replay(
        user_id(&removed),
        [
            op_id(&removed["operations"][0]),
            op_id(&rules["operations"][0]),
            op_id(&rules["operations"][1]),
            op_id(&item_ops["operations"][0]),
            op_id(&rule["operations"][0]),
            op_id(&item_ops["operations"][1]),
        ],
    );
    let requests = super::delete_category(&mut ids, context, &category, &[milk]);
    assert_requests(
        &requests,
        &[
            "lists/0024-category-delete.json",
            "lists/0025-category-delete.json",
            "lists/0026-category-delete.json",
            "lists/0027-category-delete.json",
        ],
    );
}

#[test]
fn create_and_rename_store() {
    let store = fixture("lists/0030-store-create.json");
    let filter = fixture("lists/0031-store-create.json");
    let mut ids = Ids::replay(
        user_id(&store),
        [
            s(&store["operations"][0]["updatedStore"]["identifier"]),
            op_id(&store["operations"][0]),
            s(&filter["operations"][0]["updatedStoreFilter"]["identifier"]),
            op_id(&filter["operations"][0]),
        ],
    );
    let list_id = s(&store["operations"][0]["listId"]);
    let created = super::create_store(&mut ids, &list_id, "zz capture store", &[], &[]);
    assert_requests(
        &created.requests,
        &[
            "lists/0030-store-create.json",
            "lists/0031-store-create.json",
        ],
    );

    let renamed = fixture("lists/0032-store-rename.json");
    let mut ids = Ids::replay(user_id(&renamed), [op_id(&renamed["operations"][0])]);
    assert_requests(
        &super::rename_store(&mut ids, &created.store, "zz capture store 2"),
        &["lists/0032-store-rename.json"],
    );
}

#[test]
fn delete_store_with_an_item_assigned() {
    let names = [
        "lists/0035-store-delete-with-an-item-assigned.json",
        "lists/0036-store-delete-with-an-item-assigned.json",
        "lists/0037-store-delete-with-an-item-assigned.json",
        "lists/0038-store-delete-with-an-item-assigned.json",
        "lists/0039-store-delete-with-an-item-assigned.json",
    ];
    let fixtures: Vec<Value> = names.iter().map(|n| fixture(n)).collect();
    let store = body::<PbListOperationList>(&fixtures[4]).operations[0]
        .updated_store
        .clone()
        .unwrap();
    let filter = body::<PbListOperationList>(&fixtures[2]).operations[0]
        .updated_store_filter
        .clone()
        .unwrap();
    let favourites = starter_list(&s(&fixtures[1]["operations"][0]["listId"]), vec![]);
    let recents = starter_list(&s(&fixtures[3]["operations"][0]["listId"]), vec![]);
    let mut ids = Ids::replay(
        user_id(&fixtures[0]),
        fixtures.iter().map(|f| op_id(&f["operations"][0])),
    );
    let requests = super::delete_store(
        &mut ids,
        &store,
        &[filter],
        Some(&favourites),
        Some(&recents),
    );
    assert_requests(&requests, &names);
}

#[test]
fn favourites() {
    let list = CaptureList::new();
    let (context, group_id) = list.context();
    let context = with_group(context, &group_id);

    let added = fixture("lists/0041-favourites-add-item-in-favourites-panel.json");
    let operation = &added["operations"][0];
    let favourites_id = s(&operation["listId"]);
    let mut ids = Ids::replay(
        user_id(&added),
        [s(&operation["listItemId"]), op_id(operation)],
    );
    let mut item = new_list_item(&mut ids, context, "zz fav butter", Some("dairy"));
    item.price_matchup_tag = Some("butter".to_string());
    let (butter, requests) = super::add_favourite(&mut ids, &favourites_id, item);
    assert_requests(
        &requests,
        &["lists/0041-favourites-add-item-in-favourites-panel.json"],
    );

    let copied = fixture("lists/0042-favourites-add-favourite-to-list.json");
    let operation = &copied["operations"][0];
    let mut ids = Ids::replay(
        user_id(&copied),
        [
            s(&operation["list"]["items"][0]["identifier"]),
            op_id(operation),
        ],
    );
    let (_, requests) =
        add_starter_list_items_to_list(&mut ids, &list.list_id, std::slice::from_ref(&butter));
    assert_requests(
        &requests,
        &["lists/0042-favourites-add-favourite-to-list.json"],
    );

    let deleted = fixture("lists/0043-favourites-delete-favourite.json");
    let mut ids = Ids::replay(user_id(&deleted), [op_id(&deleted["operations"][0])]);
    assert_requests(
        &remove_starter_list_items(&mut ids, &favourites_id, vec![butter]),
        &["lists/0043-favourites-delete-favourite.json"],
    );
}

/// The list recipe ingredients went to, as created.
fn recipe_list() -> (String, PbListResponse) {
    let created: PbListOperationList = body(&fixture(
        "recipes/0072-list-create-for-recipe-ingredients.json",
    ));
    let operation = &created.operations[0];
    let list_id = operation.list_id.clone().unwrap();
    let response = PbListResponse {
        list_id: Some(list_id.clone()),
        category_group_responses: vec![PbListCategoryGroupResponse {
            category_group: operation.updated_category_group.clone(),
            ..Default::default()
        }],
        ..Default::default()
    };
    (list_id, response)
}

fn recipe_in(fixture_name: &str) -> PbRecipe {
    body::<PbRecipeOperationList>(&fixture(fixture_name)).operations[0]
        .recipe
        .clone()
        .unwrap()
}

/// The items a captured list request carried.
fn items_in(fixture_name: &str) -> Vec<PbListItem> {
    body::<PbListOperationList>(&fixture(fixture_name))
        .operations
        .into_iter()
        .filter_map(|o| o.list_item)
        .collect()
}

#[test]
fn save_recipe() {
    let f = fixture("recipes/0080-recipe-create.json");
    let operation = &f["operations"][0];
    let mut ids = Ids::replay(user_id(&f), [op_id(operation)]);
    let requests = super::save_recipe(
        &mut ids,
        &s(&operation["recipeDataId"]),
        recipe_in("recipes/0080-recipe-create.json"),
    );
    assert_requests(&requests, &["recipes/0080-recipe-create.json"]);
}

#[test]
fn add_recipe_to_list() {
    let (list_id, mut response) = recipe_list();
    let recipe = recipe_in("recipes/0080-recipe-create.json");
    let f = fixture("recipes/0083-recipe-add-ingredients-to-list.json");
    // The web client's item database files the ingredients; rules stand in.
    let group = response.category_group_responses[0]
        .category_group
        .clone()
        .unwrap();
    for item in items_in("recipes/0083-recipe-add-ingredients-to-list.json") {
        response
            .categorization_rules
            .push(PbListCategorizationRule {
                item_name: item.name.clone().map(|n| n.to_lowercase()),
                category_group_id: group.identifier.clone(),
                category_id: item.category_assignments[0].category_id.clone(),
                ..Default::default()
            });
    }
    let context = ListContext {
        list_id: &list_id,
        response: Some(&response),
        category_group_id: group.identifier.as_deref(),
        new_items_on_top: false,
    };
    let ingredients: Vec<PbItemIngredient> = recipe
        .ingredients
        .iter()
        .map(|i| item_ingredient(&recipe, i))
        .collect();
    let mut ids = Ids::replay(
        user_id(&f),
        f["operations"].as_array().unwrap().iter().map(op_id),
    );
    let requests = add_ingredients_to_list(&mut ids, context, &[], None, None, &ingredients);
    assert_requests_untagged(
        &requests,
        &["recipes/0083-recipe-add-ingredients-to-list.json"],
    );
}

#[test]
fn rename_recipe_updates_its_list_items() {
    let (list_id, response) = recipe_list();
    let old = recipe_in("recipes/0080-recipe-create.json");
    let new = recipe_in("recipes/0084-recipe-edit-rename.json");
    let items = items_in("recipes/0083-recipe-add-ingredients-to-list.json");
    let f = fixture("recipes/0085-recipe-edit-rename.json");
    let context = ListContext {
        list_id: &list_id,
        response: Some(&response),
        category_group_id: None,
        new_items_on_top: false,
    };
    let mut ids = Ids::replay(
        user_id(&f),
        f["operations"].as_array().unwrap().iter().map(op_id),
    );
    let requests =
        update_recipe_ingredients_on_list(&mut ids, context, &items, None, None, &old, &new);
    assert_requests(&requests, &["recipes/0085-recipe-edit-rename.json"]);
}

#[test]
fn delete_recipe() {
    let events = fixture("recipes/0350-recipe-delete.json");
    let removed = fixture("recipes/0351-recipe-delete.json");
    let list_ops = fixture("recipes/0352-recipe-delete.json");
    let (list_id, _) = recipe_list();
    // The items, with the ingredients as they were after the rename.
    let renamed = items_in("recipes/0085-recipe-edit-rename.json");
    let items: Vec<PbListItem> = items_in("recipes/0083-recipe-add-ingredients-to-list.json")
        .into_iter()
        .zip(renamed)
        .map(|(item, renamed)| PbListItem {
            ingredients: renamed.ingredients,
            ..item
        })
        .collect();
    let event_operation = &events["operations"][0];
    let mut replay = vec![
        op_id(event_operation),
        s(&event_operation["updatedEvent"]["identifier"]),
        op_id(&removed["operations"][0]),
    ];
    replay.extend(list_ops["operations"].as_array().unwrap().iter().map(op_id));
    let mut ids = Ids::replay(user_id(&events), replay);
    let requests = super::delete_recipe(
        &mut ids,
        &s(&removed["operations"][0]["recipeDataId"]),
        &s(&event_operation["calendarId"]),
        &recipe_in("recipes/0351-recipe-delete.json"),
        Vec::new(),
        Some((&list_id, &items)),
    );
    assert_requests(
        &requests,
        &[
            "recipes/0350-recipe-delete.json",
            "recipes/0351-recipe-delete.json",
            "recipes/0352-recipe-delete.json",
        ],
    );
}

#[test]
fn create_collection() {
    let created = fixture("recipes/0092-collection-create.json");
    let icon = fixture("recipes/0093-collection-create.json");
    let operation = &created["operations"][0];
    let mut ids = Ids::replay(
        user_id(&created),
        [
            s(&operation["recipeCollection"]["identifier"]),
            op_id(operation),
            op_id(&icon["operations"][0]),
        ],
    );
    let result = super::create_collection(
        &mut ids,
        &s(&operation["recipeDataId"]),
        "zz capture collection",
    );
    assert_requests(
        &result.requests,
        &[
            "recipes/0092-collection-create.json",
            "recipes/0093-collection-create.json",
        ],
    );
}

#[test]
fn add_recipe_to_calendar() {
    let f = fixture("recipes/0171-meal-plan-add-recipe-to-calendar.json");
    let operation = &f["operations"][0];
    let event = &operation["updatedEvents"][0];
    let mut ids = Ids::replay(user_id(&f), [s(&event["identifier"]), op_id(operation)]);
    let (_, requests) = create_event(
        &mut ids,
        &s(&operation["calendarId"]),
        PbCalendarEvent {
            date: Some(s(&event["date"])),
            recipe_id: Some(s(&event["recipeId"])),
            ..Default::default()
        },
        &[],
    );
    assert_requests(
        &requests,
        &["recipes/0171-meal-plan-add-recipe-to-calendar.json"],
    );
}

#[test]
fn create_note_event() {
    let f = fixture("meal-plan/0385-meal-plan-note-event-ingredient-then-save.json");
    let operation = &f["operations"][0];
    let captured = body::<PbCalendarOperationList>(&f).operations[0]
        .updated_event
        .clone()
        .unwrap();
    let mut ids = Ids::replay(user_id(&f), [captured.identifier.clone(), op_id(operation)]);
    let (_, requests) = create_event(
        &mut ids,
        &s(&operation["calendarId"]),
        PbCalendarEvent {
            identifier: String::new(),
            calendar_id: None,
            order_added_sort_index: None,
            ..captured
        },
        &[],
    );
    assert_requests(
        &requests,
        &["meal-plan/0385-meal-plan-note-event-ingredient-then-save.json"],
    );
}

#[test]
fn delete_event() {
    let f = fixture("meal-plan/0401-meal-plan-remove-event-from-calendar.json");
    let stored = body::<PbCalendarOperationList>(&f).operations[0]
        .updated_event
        .clone()
        .unwrap();
    let mut ids = Ids::replay(user_id(&f), [op_id(&f["operations"][0])]);
    assert_requests(
        &super::delete_event(&mut ids, &stored, None),
        &["meal-plan/0401-meal-plan-remove-event-from-calendar.json"],
    );
}

#[test]
fn normalised_units() {
    assert_eq!(normalized_unit("cups"), "cup");
    assert_eq!(normalized_unit("mL"), "ml");
    assert_eq!(normalized_unit("tsp"), "tsp");
    assert_eq!(normalized_unit("tablespoons"), "Tbsp");
    assert_eq!(normalized_unit("slices"), "slice");
    assert_eq!(normalized_unit("lbs."), "lb");
}

// Requests captured from the web client on 2025-10-28 with
// tools/curl_to_snapshot.py.

const USER_2025: &str = "cda21b0078644a01b640c84d3d74187e";
const LIST_2025: &str = "58ec2be417b247d7a9edc2d9d66889ab";

fn list_2025() -> PbListResponse {
    PbListResponse {
        list_id: Some(LIST_2025.to_string()),
        category_group_responses: vec![PbListCategoryGroupResponse {
            category_group: Some(PbListCategoryGroup {
                identifier: Some("65564675a0de5a5fa6a69272df260fcc".to_string()),
                list_id: Some(LIST_2025.to_string()),
                categories: vec![PbListCategory {
                    identifier: Some("f962e619ab90479894c9dcc291a7f103".to_string()),
                    system_category: Some("other".to_string()),
                    name: Some("Other".to_string()),
                    ..Default::default()
                }],
                default_category_id: Some("f962e619ab90479894c9dcc291a7f103".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn webapp_add_shopping_list_item_2025_10_28() {
    let response = list_2025();
    let context = ListContext {
        list_id: LIST_2025,
        response: Some(&response),
        category_group_id: Some("65564675a0de5a5fa6a69272df260fcc"),
        new_items_on_top: false,
    };
    let mut ids = Ids::replay(
        USER_2025,
        [
            "d83d31d5d48d4b7591b05c7efb725a46",
            "0da34b3d00f54ce1bd6fd501ddf62f99",
        ],
    );
    let item = new_list_item(&mut ids, context, "nice new things", None);
    let requests = super::add_item(&mut ids, context, item);
    insta::assert_snapshot!(hex::encode(&requests[0].body));
}

#[test]
fn webapp_bulk_remove_list_items_2025_10_28() {
    let response = list_2025();
    let context = ListContext {
        list_id: LIST_2025,
        response: Some(&response),
        category_group_id: Some("65564675a0de5a5fa6a69272df260fcc"),
        new_items_on_top: false,
    };
    let mut ids = Ids::replay(
        USER_2025,
        [
            "2eb63311646048568db569a50116cae8",
            "e26f61f101a94a81b395b4db8b870071",
        ],
    );
    let item = new_list_item(&mut ids, context, "deleted item", None);
    let requests = delete_items(&mut ids, LIST_2025, &[item], None);
    insta::assert_snapshot!(hex::encode(&requests[0].body));
}
