//! Pure operation builders for AnyList API protobuf messages.
//!
//! This module is the "functional core": each builder takes the state the
//! AnyList web client would have in memory (lists, items, settings, ...) and
//! returns the requests the web client sends for the same action, in the same
//! order, with the same fields. The client methods (the imperative shell)
//! fetch that state, call a builder and post its requests.
//!
//! Every builder is checked byte for byte against requests captured from the
//! official web client (`tests/fixtures/webapp_2026_10_27`, see
//! `docs/webapp-operations.md`). New identifiers come from [`Ids`], which
//! the tests replay from the capture.

use std::collections::VecDeque;

use prost::Message;
use uuid::Uuid;

use crate::protobuf::anylist::pb_operation_metadata::OperationClass;
use crate::protobuf::anylist::{
    pb_list_folder_item, pb_shopping_list, pb_starter_list, PbCalendarEvent, PbCalendarOperation,
    PbCalendarOperationList, PbEmailUserIdPair, PbIcon, PbIngredient, PbItemIngredient,
    PbItemQuantity, PbListCategorizationRule, PbListCategory, PbListCategoryGroup,
    PbListFolderItem, PbListFolderOperation, PbListFolderOperationList, PbListItem,
    PbListItemCategoryAssignment, PbListOperation, PbListOperationList, PbListResponse,
    PbListSettings, PbListSettingsOperation, PbListSettingsOperationList, PbOperationMetadata,
    PbRecipe, PbRecipeCollection, PbRecipeCollectionSettings, PbRecipeOperation,
    PbRecipeOperationList, PbShoppingList, PbStarterList, PbStarterListOperation,
    PbStarterListOperationList, PbStore, PbStoreFilter,
};
use crate::utils::generate_id;

#[cfg(test)]
#[path = "operations_webapp_tests.rs"]
mod webapp_tests;

/// The endpoints operations are posted to.
pub mod endpoint {
    pub const SHOPPING_LISTS: &str = "data/shopping-lists/update";
    pub const SHOPPING_LISTS_V2: &str = "data/shopping-lists/update-v2";
    pub const STARTER_LISTS: &str = "data/starter-lists/update";
    pub const LIST_SETTINGS: &str = "data/list-settings/update";
    pub const LIST_FOLDERS: &str = "data/list-folders/update";
    pub const RECIPES: &str = "data/user-recipe-data/update";
    pub const CALENDAR: &str = "data/meal-planning-calendar/update";
}

/// One POST: an encoded operation list for an endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub endpoint: &'static str,
    pub body: Vec<u8>,
}

impl Request {
    fn new(endpoint: &'static str, operation_list: &impl Message) -> Self {
        Self {
            endpoint,
            body: operation_list.encode_to_vec(),
        }
    }

    fn list(endpoint: &'static str, operations: Vec<PbListOperation>) -> Self {
        Self::new(endpoint, &PbListOperationList { operations })
    }

    fn starter_list(operations: Vec<PbStarterListOperation>) -> Self {
        Self::new(
            endpoint::STARTER_LISTS,
            &PbStarterListOperationList { operations },
        )
    }

    fn list_settings(operations: Vec<PbListSettingsOperation>) -> Self {
        Self::new(
            endpoint::LIST_SETTINGS,
            &PbListSettingsOperationList { operations },
        )
    }

    fn list_folders(operations: Vec<PbListFolderOperation>) -> Self {
        Self::new(
            endpoint::LIST_FOLDERS,
            &PbListFolderOperationList { operations },
        )
    }
}

/// The signed-in user's ID and a source of new identifiers.
///
/// The client generates random IDs. Tests replay the IDs the web client
/// generated, in the order a builder asks for them.
pub struct Ids {
    user_id: String,
    replay: Option<VecDeque<String>>,
}

impl Ids {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            replay: None,
        }
    }

    pub fn replay<S: Into<String>>(
        user_id: impl Into<String>,
        ids: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            replay: Some(ids.into_iter().map(Into::into).collect()),
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// A new identifier: 32 lowercase hex digits, like the web client's.
    pub fn new_id(&mut self) -> String {
        match &mut self.replay {
            Some(ids) => ids.pop_front().expect("ran out of replayed IDs"),
            None => generate_id(),
        }
    }

    /// Operation metadata as the web client builds it: a new operation ID, the
    /// handler and the user. The operation class is left out when undefined.
    fn metadata(&mut self, handler_id: &str) -> PbOperationMetadata {
        PbOperationMetadata {
            operation_id: Some(self.new_id()),
            handler_id: Some(handler_id.to_string()),
            user_id: Some(self.user_id.clone()),
            ..Default::default()
        }
    }

    fn metadata_of(&mut self, handler_id: &str, class: OperationClass) -> PbOperationMetadata {
        PbOperationMetadata {
            operation_class: Some(class as i32),
            ..self.metadata(handler_id)
        }
    }
}

/// The web client's name-based UUIDs (`_.uuid5`), as 32 hex digits.
fn uuid5(name: &str, namespace: &str) -> String {
    let namespace = Uuid::parse_str(namespace).expect("valid namespace");
    Uuid::new_v5(&namespace, name.as_bytes())
        .simple()
        .to_string()
}

// ============================================================================
// Lists
// ============================================================================

/// The list theme a new list gets unless the user changed the default.
pub const DEFAULT_LIST_THEME_ID: &str = "7b1dd303fb6a44fbbed44667f199aa63";

/// Categories of a new list's category group: name and system category
/// (which is also the icon), in order. The last one is the default.
const DEFAULT_CATEGORIES: [(&str, &str); 20] = [
    ("Baby", "baby"),
    ("Bakery", "bakery"),
    ("Beverages", "beverages"),
    ("Breakfast & Cereal", "breakfast-and-cereal"),
    (
        "Condiments & Dressings",
        "condiments-oils-and-salad-dressings",
    ),
    ("Cooking & Baking", "cooking-and-baking"),
    ("Dairy", "dairy"),
    ("Deli", "deli"),
    ("Frozen Foods", "frozen-foods"),
    ("Grains, Pasta & Sides", "grains-pasta-and-side-dishes"),
    ("Health & Personal Care", "health-and-personal-care"),
    ("Household & Cleaning", "household-and-cleaning"),
    ("Meat", "meat"),
    ("Pet Supplies", "pet-supplies"),
    ("Produce", "produce"),
    ("Seafood", "seafood"),
    ("Snacks", "snacks-cookies-and-candy"),
    ("Soups & Canned Goods", "soups-and-canned-goods"),
    ("Wine, Beer & Spirits", "wine-beer-spirits"),
    ("Other", "other"),
];

/// What creating a list needs besides its name.
pub struct NewList<'a> {
    pub name: &'a str,
    /// The user's email, shared with the new list.
    pub email: &'a str,
    /// The folder the list goes in (the root folder).
    pub folder_id: &'a str,
    /// The theme new lists get, normally [`DEFAULT_LIST_THEME_ID`].
    pub theme_id: &'a str,
}

/// The list and the requests that create it.
pub struct CreatedList {
    pub list: PbShoppingList,
    pub requests: Vec<Request>,
}

/// Create a list: the list with its default categories, its Recent Items and
/// Favorite Items starter lists, its settings, then its icon.
pub fn create_list(ids: &mut Ids, new: NewList) -> CreatedList {
    let list_id = ids.new_id();
    let user_id = ids.user_id().to_string();

    let group_id = ids.new_id();
    let categories = DEFAULT_CATEGORIES
        .iter()
        .enumerate()
        .map(|(index, (name, system_category))| PbListCategory {
            identifier: Some(ids.new_id()),
            category_group_id: Some(group_id.clone()),
            list_id: Some(list_id.clone()),
            name: Some(name.to_string()),
            icon: Some(system_category.to_string()),
            system_category: Some(system_category.to_string()),
            sort_index: Some(index as i32),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let default_category_id = categories.last().and_then(|c| c.identifier.clone());
    let group = PbListCategoryGroup {
        identifier: Some(group_id.clone()),
        list_id: Some(list_id.clone()),
        categories,
        default_category_id,
        ..Default::default()
    };

    let list = PbShoppingList {
        identifier: list_id.clone(),
        name: Some(new.name.to_string()),
        creator: Some(user_id.clone()),
        shared_users: vec![PbEmailUserIdPair {
            email: Some(new.email.to_string()),
            user_id: Some(user_id.clone()),
            ..Default::default()
        }],
        list_item_sort_order: Some(pb_shopping_list::ListItemSortOrder::Alphabetical as i32),
        ..Default::default()
    };
    let new_list = PbListOperation {
        metadata: Some(ids.metadata("new-shopping-list")),
        list_id: Some(list_id.clone()),
        list: Some(list.clone()),
        list_folder_id: Some(new.folder_id.to_string()),
        updated_category_group: Some(group),
        ..Default::default()
    };

    let starter_lists = [
        ("Recent Items", pb_starter_list::Type::RecentItems),
        ("Favorite Items", pb_starter_list::Type::FavoriteItems),
    ]
    .into_iter()
    .map(|(name, kind)| {
        let starter_list_id = starter_list_id(&list_id, kind);
        PbStarterListOperation {
            metadata: Some(ids.metadata("new-starter-list")),
            list_id: Some(starter_list_id.clone()),
            list: Some(PbStarterList {
                identifier: starter_list_id,
                name: Some(name.to_string()),
                list_id: Some(list_id.clone()),
                starter_list_type: Some(kind as i32),
                ..Default::default()
            }),
            ..Default::default()
        }
    })
    .collect();

    let settings_id = ids.new_id();
    let settings = |changes: PbListSettings| PbListSettings {
        identifier: settings_id.clone(),
        user_id: Some(user_id.clone()),
        list_id: Some(list_id.clone()),
        ..changes
    };
    let initial_settings = [
        (
            "set-list-theme-id",
            PbListSettings {
                list_theme_id: Some(new.theme_id.to_string()),
                ..Default::default()
            },
        ),
        (
            "set-should-hide-categories",
            PbListSettings {
                should_hide_categories: Some(false),
                ..Default::default()
            },
        ),
        (
            "set-generic-grocery-autocomplete-enabled",
            PbListSettings {
                generic_grocery_autocomplete_enabled: Some(true),
                ..Default::default()
            },
        ),
        (
            "set-list-item-sort-order",
            PbListSettings {
                list_item_sort_order: Some("ALListItemSortOrderAlphabetical".to_string()),
                ..Default::default()
            },
        ),
        (
            "set-list-category-group-id",
            PbListSettings {
                list_category_group_id: Some(group_id),
                ..Default::default()
            },
        ),
        (
            "set-should-remember-item-categories",
            PbListSettings {
                should_remember_item_categories: Some(true),
                ..Default::default()
            },
        ),
        (
            "set-favorites-autocomplete-enabled",
            PbListSettings {
                favorites_autocomplete_enabled: Some(true),
                ..Default::default()
            },
        ),
        (
            "set-recent-items-autocomplete-enabled",
            PbListSettings {
                recent_items_autocomplete_enabled: Some(true),
                ..Default::default()
            },
        ),
    ]
    .into_iter()
    .map(|(handler_id, changes)| PbListSettingsOperation {
        metadata: Some(ids.metadata(handler_id)),
        updated_settings: Some(settings(changes)),
    })
    .collect();

    let set_icon = PbListSettingsOperation {
        metadata: Some(ids.metadata("set-icon")),
        updated_settings: Some(settings(PbListSettings {
            icon: Some(PbIcon {
                icon_name: Some("default_list_icon".to_string()),
                tint_hex_color: Some("16a1e0".to_string()),
            }),
            ..Default::default()
        })),
    };

    CreatedList {
        list,
        requests: vec![
            Request::list(endpoint::SHOPPING_LISTS, vec![new_list]),
            Request::starter_list(starter_lists),
            Request::list_settings(initial_settings),
            Request::list_settings(vec![set_icon]),
        ],
    }
}

/// Namespace of the IDs of lists' Recent Items.
const RECENT_ITEMS_NAMESPACE: &str = "f915bc60200c4c61b4b0a17bd9978960";
/// Namespace of the IDs of lists' Favorite Items.
const FAVORITE_ITEMS_NAMESPACE: &str = "839503408980408581879d73a33ec4c1";

/// A list's Recent Items and Favorite Items always have the same IDs.
pub fn starter_list_id(list_id: &str, kind: pb_starter_list::Type) -> String {
    match kind {
        pb_starter_list::Type::FavoriteItems => uuid5(list_id, FAVORITE_ITEMS_NAMESPACE),
        _ => uuid5(list_id, RECENT_ITEMS_NAMESPACE),
    }
}

/// Rename a list.
pub fn rename_list(ids: &mut Ids, list_id: &str, name: &str) -> Vec<Request> {
    let operation = PbListOperation {
        metadata: Some(ids.metadata("rename-list")),
        list_id: Some(list_id.to_string()),
        updated_value: Some(name.to_string()),
        ..Default::default()
    };
    vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])]
}

/// Delete a list: take it out of its folder, then remove its settings.
///
/// `settings` is the list's stored settings; only their identifier, user and
/// list are sent.
pub fn delete_list(
    ids: &mut Ids,
    list_data_id: &str,
    folder_id: &str,
    list_id: &str,
    settings: &PbListSettings,
) -> Vec<Request> {
    let delete_folder_items = PbListFolderOperation {
        metadata: Some(ids.metadata("delete-folder-items")),
        list_data_id: Some(list_data_id.to_string()),
        folder_items: vec![PbListFolderItem {
            identifier: list_id.to_string(),
            item_type: Some(pb_list_folder_item::ItemType::ListType as i32),
        }],
        original_parent_folder_id: Some(folder_id.to_string()),
        ..Default::default()
    };
    let remove_list_settings = PbListSettingsOperation {
        metadata: Some(ids.metadata("remove-list-settings")),
        updated_settings: Some(PbListSettings {
            identifier: settings.identifier.clone(),
            user_id: settings.user_id.clone(),
            list_id: settings.list_id.clone(),
            ..Default::default()
        }),
    };
    vec![
        Request::list_folders(vec![delete_folder_items]),
        Request::list_settings(vec![remove_list_settings]),
    ]
}

// ============================================================================
// Categories of a list's items
// ============================================================================

/// Namespace of the web client's category assignment IDs.
const CATEGORY_ASSIGNMENT_NAMESPACE: &str = "08e5c5bdcd694454a1ffd611b6d9abc0";
/// Namespace of the web client's categorization rule IDs.
const CATEGORIZATION_RULE_NAMESPACE: &str = "f4338133428d4f0b94027c9b23243f14";

/// System categories an item's `category` field may hold; any other
/// category match ID is stored as "other".
const ITEM_CATEGORIES: [&str; 19] = [
    "baby",
    "bakery",
    "beverages",
    "breakfast-and-cereal",
    "condiments-oils-and-salad-dressings",
    "cooking-and-baking",
    "dairy",
    "frozen-foods",
    "grains-pasta-and-side-dishes",
    "health-and-personal-care",
    "household-and-cleaning",
    "meat",
    "pet-supplies",
    "produce",
    "seafood",
    "snacks-cookies-and-candy",
    "soups-and-canned-goods",
    "wine-beer-spirits",
    "other",
];

/// An item's assignment to one category group always has the same ID.
pub fn category_assignment_id(category_group_id: &str) -> String {
    uuid5(category_group_id, CATEGORY_ASSIGNMENT_NAMESPACE)
}

/// A list has one categorization rule per item name and category group.
pub fn categorization_rule_id(item_name: &str, category_group_id: &str, list_id: &str) -> String {
    let name = format!(
        "{}{}{}",
        item_name.to_lowercase(),
        category_group_id,
        list_id
    );
    uuid5(&name, CATEGORIZATION_RULE_NAMESPACE)
}

/// The category match ID of a category: its system category, or for a
/// category the user made, its name as a slug ("Kids & Toys" → "kids-and-toys").
pub fn category_match_id(category: &PbListCategory) -> String {
    match category.system_category.as_deref() {
        Some(system) if !system.is_empty() => system.to_string(),
        _ => slug(category.name.as_deref().unwrap_or_default()),
    }
}

fn slug(name: &str) -> String {
    // The web client replaces only the first "&".
    let name = name.to_lowercase().trim().replacen('&', " and ", 1);
    let kept: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect();
    let mut slug = String::new();
    let mut in_separator = false;
    for c in kept.chars() {
        if c.is_whitespace() || c == '-' {
            if !in_separator {
                slug.push('-');
            }
            in_separator = true;
        } else {
            slug.push(c);
            in_separator = false;
        }
    }
    slug
}

/// What the web client knows about a shopping list when it adds or edits
/// items: its category groups and categorization rules, which group the list
/// currently shows, and where new items go.
#[derive(Clone, Copy)]
pub struct ListContext<'a> {
    pub list_id: &'a str,
    /// The list's stored categories, rules, stores and filters.
    pub response: Option<&'a PbListResponse>,
    /// The category group the list's settings select.
    pub category_group_id: Option<&'a str>,
    /// Whether the list adds new items at the top.
    pub new_items_on_top: bool,
}

impl<'a> ListContext<'a> {
    fn groups(&self) -> impl Iterator<Item = &'a PbListCategoryGroup> {
        self.response
            .into_iter()
            .flat_map(|r| &r.category_group_responses)
            .filter_map(|g| g.category_group.as_ref())
    }

    pub fn category(&self, category_id: &str) -> Option<&'a PbListCategory> {
        self.groups()
            .flat_map(|g| &g.categories)
            .find(|c| c.identifier.as_deref() == Some(category_id))
    }

    fn rule(
        &self,
        item_name: &str,
        category_group_id: &str,
    ) -> Option<&'a PbListCategorizationRule> {
        let item_name = item_name.to_lowercase();
        self.response
            .into_iter()
            .flat_map(|r| &r.categorization_rules)
            .find(|rule| {
                rule.item_name.as_deref() == Some(item_name.as_str())
                    && rule.category_group_id.as_deref() == Some(category_group_id)
            })
    }

    /// The category in `group` that `hint` names: a system category
    /// ("dairy") or a category name ("Dairy").
    fn category_named(group: &'a PbListCategoryGroup, hint: &str) -> Option<&'a PbListCategory> {
        group
            .categories
            .iter()
            .find(|c| c.system_category.as_deref() == Some(hint))
            .or_else(|| {
                group.categories.iter().find(|c| {
                    c.name
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(hint))
                })
            })
    }

    /// The category `hint` names in the list's current category group.
    pub fn find_category(&self, hint: &str) -> Option<&'a PbListCategory> {
        let group = self
            .groups()
            .find(|g| g.identifier.as_deref() == self.category_group_id)?;
        Self::category_named(group, hint)
    }

    /// The web client's category assignments for a new item: per category
    /// group, the list's categorization rule for the name, else the category
    /// the hint names (the web client uses its own item database here), else
    /// the group's default category.
    fn category_assignments(
        &self,
        name: &str,
        hint: Option<&str>,
    ) -> Vec<PbListItemCategoryAssignment> {
        self.groups()
            .filter_map(|group| {
                let group_id = group.identifier.as_deref()?;
                let category_id = match self.rule(name, group_id) {
                    Some(rule) => rule.category_id.clone(),
                    None => match hint.and_then(|hint| Self::category_named(group, hint)) {
                        Some(category) => category.identifier.clone(),
                        None => group.default_category_id.clone(),
                    },
                }
                .filter(|id| !id.is_empty())?;
                Some(PbListItemCategoryAssignment {
                    identifier: Some(category_assignment_id(group_id)),
                    category_group_id: Some(group_id.to_string()),
                    category_id: Some(category_id),
                })
            })
            .collect()
    }

    /// The category match ID of the item's category in the current group.
    fn category_match_id(&self, item: &PbListItem) -> String {
        item.category_assignments
            .iter()
            .find(|a| a.category_group_id.as_deref() == self.category_group_id)
            .and_then(|a| self.category(a.category_id.as_deref()?))
            .map(category_match_id)
            .unwrap_or_else(|| "other".to_string())
    }
}

/// Set an item's category match ID and the `category` field derived from it.
fn set_category_match_id(item: &mut PbListItem, match_id: String) {
    item.category = Some(if ITEM_CATEGORIES.contains(&match_id.as_str()) {
        match_id.clone()
    } else {
        "other".to_string()
    });
    item.category_match_id = Some(match_id);
}

/// A new item as the web client makes it from a name, categorised for the
/// list. `category` stands in for the web client's item database: a system
/// category or category name used when no categorization rule applies.
pub fn new_list_item(
    ids: &mut Ids,
    list: ListContext,
    name: &str,
    category: Option<&str>,
) -> PbListItem {
    let identifier = ids.new_id();
    new_list_item_with_id(ids, list, identifier, name, category)
}

fn new_list_item_with_id(
    ids: &Ids,
    list: ListContext,
    identifier: String,
    name: &str,
    category: Option<&str>,
) -> PbListItem {
    let mut item = PbListItem {
        identifier,
        name: Some(if name.is_empty() { "-" } else { name }.to_string()),
        list_id: Some(list.list_id.to_string()),
        user_id: Some(ids.user_id().to_string()),
        category_assignments: list.category_assignments(name, category),
        ..Default::default()
    };
    let match_id = list.category_match_id(&item);
    set_category_match_id(&mut item, match_id);
    item
}

/// Parse a quantity the way the web client stores it: a leading amount
/// ("2", "1 1/2", "0.5", "½") and the unit after it ("2 L" → 2, L).
pub fn parse_quantity(raw: &str) -> PbItemQuantity {
    static AMOUNT: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let amount_re = AMOUNT.get_or_init(|| {
        regex::Regex::new(
            r"^((?:\d+\s+)?\d+/\d+|\d+(?:[.,]\d+)?(?:\s*[½⅓⅔¼¾⅕⅖⅗⅘⅙⅚⅛⅜⅝⅞])?|[½⅓⅔¼¾⅕⅖⅗⅘⅙⅚⅛⅜⅝⅞])\s*(.*)$",
        )
        .expect("valid regex")
    });
    let raw = raw.trim();
    let (amount, unit) = match amount_re.captures(raw) {
        Some(c) => (
            Some(c[1].to_string()),
            Some(c[2].trim().to_string()).filter(|u| !u.is_empty()),
        ),
        None => (None, None),
    };
    PbItemQuantity {
        amount,
        unit,
        raw_quantity: (!raw.is_empty()).then(|| raw.to_string()),
    }
}

/// Add an item to a list.
pub fn add_item(ids: &mut Ids, list: ListContext, item: PbListItem) -> Vec<Request> {
    let operation = PbListOperation {
        metadata: Some(ids.metadata("add-shopping-list-item")),
        list_id: Some(list.list_id.to_string()),
        list_item_id: Some(item.identifier.clone()),
        list_item: Some(item),
        list: list.new_items_on_top.then(|| PbShoppingList {
            identifier: list.list_id.to_string(),
            new_list_item_position: Some(pb_shopping_list::NewListItemPosition::Top as i32),
            ..Default::default()
        }),
        ..Default::default()
    };
    vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])]
}

fn item_operation(
    ids: &mut Ids,
    handler_id: &str,
    list_id: &str,
    item_id: &str,
) -> PbListOperation {
    PbListOperation {
        metadata: Some(ids.metadata(handler_id)),
        list_id: Some(list_id.to_string()),
        list_item_id: Some(item_id.to_string()),
        ..Default::default()
    }
}

/// Rename an item.
pub fn set_item_name(ids: &mut Ids, item: &PbListItem, name: &str) -> Vec<Request> {
    let operation = PbListOperation {
        updated_value: Some(name.to_string()),
        original_value: item.name.clone(),
        ..item_operation(
            ids,
            "set-list-item-name",
            list_id_of(item),
            &item.identifier,
        )
    };
    vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])]
}

/// Change an item's note.
pub fn set_item_details(ids: &mut Ids, item: &PbListItem, details: &str) -> Vec<Request> {
    let operation = PbListOperation {
        updated_value: Some(details.to_string()),
        original_value: item.details.clone(),
        ..item_operation(
            ids,
            "set-list-item-details",
            list_id_of(item),
            &item.identifier,
        )
    };
    vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])]
}

/// Change an item's quantity.
pub fn set_item_quantity(
    ids: &mut Ids,
    item: &PbListItem,
    quantity: PbItemQuantity,
) -> Vec<Request> {
    let operation = PbListOperation {
        list_item: Some(PbListItem {
            identifier: item.identifier.clone(),
            list_id: item.list_id.clone(),
            quantity_pb: Some(quantity),
            ..Default::default()
        }),
        ..item_operation(
            ids,
            "set-list-item-quantity-v2",
            list_id_of(item),
            &item.identifier,
        )
    };
    vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])]
}

/// Move an item to another category of the list's current category group,
/// and remember the choice for the item's name, as the web client does when
/// an item's category is changed.
pub fn set_item_category(
    ids: &mut Ids,
    list: ListContext,
    item: &PbListItem,
    category: &PbListCategory,
) -> Vec<Request> {
    let list_id = list.list_id;
    let group_id = category.category_group_id.clone().unwrap_or_default();
    let category_id = category.identifier.clone().unwrap_or_default();
    let name = item.name.as_deref().unwrap_or_default();

    // The item's assignment to the group, replaced or added.
    let mut item = item.clone();
    let assignment = PbListItemCategoryAssignment {
        identifier: Some(category_assignment_id(&group_id)),
        category_group_id: Some(group_id.clone()),
        category_id: Some(category_id.clone()),
    };
    match item
        .category_assignments
        .iter_mut()
        .find(|a| a.identifier == assignment.identifier)
    {
        Some(existing) => *existing = assignment,
        None => item.category_assignments.push(assignment),
    }
    let update_assignment = PbListOperation {
        list_item: Some(item.clone()),
        ..item_operation(
            ids,
            "update-list-item-category-assignment",
            list_id,
            &item.identifier,
        )
    };

    let rule_id = categorization_rule_id(name, &group_id, list_id);
    let rule = list
        .response
        .into_iter()
        .flat_map(|r| &r.categorization_rules)
        .find(|rule| rule.identifier.as_deref() == Some(rule_id.as_str()))
        .cloned()
        .unwrap_or_else(|| PbListCategorizationRule {
            identifier: Some(rule_id),
            item_name: Some(name.to_lowercase()),
            category_group_id: Some(group_id.clone()),
            list_id: Some(list_id.to_string()),
            ..Default::default()
        });
    let save_rule = PbListOperation {
        metadata: Some(ids.metadata_of(
            "save-categorization-rule",
            OperationClass::ListCategorizationRule,
        )),
        list_id: Some(list_id.to_string()),
        updated_categorization_rule: Some(PbListCategorizationRule {
            category_id: Some(category_id),
            ..rule
        }),
        ..Default::default()
    };

    set_category_match_id(&mut item, category_match_id(category));
    let set_match_id = PbListOperation {
        original_value: item.category_match_id.clone(),
        list_item: Some(item.clone()),
        ..item_operation(
            ids,
            "set-list-item-category-match-id",
            list_id,
            &item.identifier,
        )
    };

    vec![
        Request::list(endpoint::SHOPPING_LISTS_V2, vec![save_rule]),
        Request::list(
            endpoint::SHOPPING_LISTS,
            vec![update_assignment, set_match_id],
        ),
    ]
}

fn list_id_of(item: &PbListItem) -> &str {
    item.list_id.as_deref().unwrap_or_default()
}

/// Cross off or uncross an item. Crossing off also files the item under the
/// list's Recent Items.
pub fn set_item_checked(
    ids: &mut Ids,
    item: &PbListItem,
    checked: bool,
    recent_items: Option<&PbStarterList>,
) -> Vec<Request> {
    let operation = PbListOperation {
        updated_value: Some(if checked { "y" } else { "n" }.to_string()),
        ..item_operation(
            ids,
            "set-list-item-checked",
            list_id_of(item),
            &item.identifier,
        )
    };
    let mut requests = vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])];
    if checked {
        requests.extend(add_to_recent_items(
            ids,
            recent_items,
            std::slice::from_ref(item),
            false,
        ));
    }
    requests
}

/// Delete items (in the list's order), filing them under Recent Items.
pub fn delete_items(
    ids: &mut Ids,
    list_id: &str,
    items: &[PbListItem],
    recent_items: Option<&PbStarterList>,
) -> Vec<Request> {
    let mut requests = vec![bulk_remove_items(ids, list_id, items)];
    requests.extend(add_to_recent_items(ids, recent_items, items, false));
    requests
}

/// Delete the crossed-off items. They were filed under Recent Items when they
/// were crossed off, so only ones missing from it are added.
pub fn delete_crossed_off_items(
    ids: &mut Ids,
    list_id: &str,
    items: &[PbListItem],
    recent_items: Option<&PbStarterList>,
) -> Vec<Request> {
    let mut requests = add_to_recent_items(ids, recent_items, items, true);
    requests.push(bulk_remove_items(ids, list_id, items));
    requests
}

fn bulk_remove_items(ids: &mut Ids, list_id: &str, items: &[PbListItem]) -> Request {
    let operation = PbListOperation {
        metadata: Some(ids.metadata("bulk-remove-list-items")),
        list_id: Some(list_id.to_string()),
        list: Some(PbShoppingList {
            identifier: list_id.to_string(),
            items: items.to_vec(),
            ..Default::default()
        }),
        ..Default::default()
    };
    Request::list(endpoint::SHOPPING_LISTS, vec![operation])
}

/// How many items Recent Items keeps.
const RECENT_ITEMS_LIMIT: usize = 200;

/// The web client's item equality (`isEqualToItem`): same name ignoring
/// case, and the same quantities, note, photos, recipe, event, stores,
/// prices, ingredients and barcode.
pub fn same_item(a: &PbListItem, b: &PbListItem) -> bool {
    let text = |s: &Option<String>| s.clone().unwrap_or_default();
    let flag = |f: Option<bool>| f.unwrap_or(false);
    let quantity = |q: &Option<PbItemQuantity>| q.clone().unwrap_or_default();
    text(&a.name).to_lowercase() == text(&b.name).to_lowercase()
        && quantity(&a.quantity_pb) == quantity(&b.quantity_pb)
        && quantity(&a.price_quantity_pb) == quantity(&b.price_quantity_pb)
        && flag(a.price_quantity_should_override_item_quantity)
            == flag(b.price_quantity_should_override_item_quantity)
        && flag(a.item_quantity_should_override_ingredient_quantity)
            == flag(b.item_quantity_should_override_ingredient_quantity)
        && a.package_size_pb.clone().unwrap_or_default()
            == b.package_size_pb.clone().unwrap_or_default()
        && a.price_package_size_pb.clone().unwrap_or_default()
            == b.price_package_size_pb.clone().unwrap_or_default()
        && flag(a.price_package_size_should_override_item_package_size)
            == flag(b.price_package_size_should_override_item_package_size)
        && flag(a.item_package_size_should_override_ingredient_package_size)
            == flag(b.item_package_size_should_override_ingredient_package_size)
        && text(&a.details) == text(&b.details)
        && a.photo_ids == b.photo_ids
        && a.recipe_id == b.recipe_id
        && a.event_id == b.event_id
        && a.store_ids == b.store_ids
        && a.prices == b.prices
        && a.ingredients == b.ingredients
        && text(&a.product_upc) == text(&b.product_upc)
}

/// File items under a list's Recent Items: a copy of each (new ID, not
/// crossed off) replaces an equal item already there, unless
/// `skip_existing`, and the oldest items go once there are more than 200.
pub fn add_to_recent_items(
    ids: &mut Ids,
    recent_items: Option<&PbStarterList>,
    items: &[PbListItem],
    skip_existing: bool,
) -> Vec<Request> {
    let Some(recent_items) = recent_items else {
        return Vec::new();
    };
    let starter_list_id = recent_items.identifier.as_str();

    let mut replaced = Vec::new();
    let mut copies = Vec::new();
    for item in items {
        let existing = recent_items.items.iter().find(|r| same_item(item, r));
        if existing.is_some() && skip_existing {
            continue;
        }
        replaced.extend(existing.cloned());
        copies.push(PbListItem {
            identifier: ids.new_id(),
            list_id: Some(starter_list_id.to_string()),
            checked: Some(false),
            ..item.clone()
        });
    }

    let mut operations = Vec::new();
    if !replaced.is_empty() {
        operations.push(starter_list_items_operation(
            ids,
            "bulk-remove-list-items",
            starter_list_id,
            replaced.clone(),
        ));
    }
    if !copies.is_empty() {
        let kept: Vec<&PbListItem> = recent_items
            .items
            .iter()
            .filter(|r| !replaced.iter().any(|x| x.identifier == r.identifier))
            .collect();
        if copies.len() > RECENT_ITEMS_LIMIT {
            copies.drain(..copies.len() - RECENT_ITEMS_LIMIT);
        }
        let overflow = (kept.len() + copies.len()).saturating_sub(RECENT_ITEMS_LIMIT);
        if overflow > 0 {
            let oldest = kept[..overflow].iter().map(|i| (*i).clone()).collect();
            operations.push(starter_list_items_operation(
                ids,
                "bulk-remove-list-items",
                starter_list_id,
                oldest,
            ));
        }
        for chunk in copies.chunks(25) {
            operations.push(starter_list_items_operation(
                ids,
                "bulk-add-list-items",
                starter_list_id,
                chunk.to_vec(),
            ));
        }
    }

    if operations.is_empty() {
        Vec::new()
    } else {
        vec![Request::starter_list(operations)]
    }
}

fn starter_list_items_operation(
    ids: &mut Ids,
    handler_id: &str,
    starter_list_id: &str,
    items: Vec<PbListItem>,
) -> PbStarterListOperation {
    PbStarterListOperation {
        metadata: Some(ids.metadata(handler_id)),
        list_id: Some(starter_list_id.to_string()),
        list: Some(PbStarterList {
            identifier: starter_list_id.to_string(),
            items,
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ============================================================================
// Categories
// ============================================================================

/// A category group as the web client sends it in group operations: the
/// stored group without its categories.
fn category_group_copy(group: &PbListCategoryGroup) -> PbListCategoryGroup {
    PbListCategoryGroup {
        categories: Vec::new(),
        ..group.clone()
    }
}

/// The group's categories in display order.
fn sorted_categories(group: &PbListCategoryGroup) -> Vec<&PbListCategory> {
    let mut categories: Vec<&PbListCategory> = group.categories.iter().collect();
    categories.sort_by_key(|c| c.sort_index.unwrap_or_default());
    categories
}

fn category_operation(
    ids: &mut Ids,
    handler_id: &str,
    class: OperationClass,
    list_id: &str,
) -> PbListOperation {
    PbListOperation {
        metadata: Some(ids.metadata_of(handler_id, class)),
        list_id: Some(list_id.to_string()),
        ..Default::default()
    }
}

/// Reorder a group's categories: the group with only the categories' IDs.
fn set_sorted_category_ids(
    ids: &mut Ids,
    list_id: &str,
    group: &PbListCategoryGroup,
    category_ids: impl IntoIterator<Item = String>,
) -> PbListOperation {
    PbListOperation {
        updated_category_group: Some(PbListCategoryGroup {
            categories: category_ids
                .into_iter()
                .map(|id| PbListCategory {
                    identifier: Some(id),
                    ..Default::default()
                })
                .collect(),
            ..category_group_copy(group)
        }),
        ..category_operation(
            ids,
            "set-sorted-category-ids",
            OperationClass::ListCategoryGroup,
            list_id,
        )
    }
}

/// The category and the requests that create it.
pub struct CreatedCategory {
    pub category: PbListCategory,
    pub requests: Vec<Request>,
}

/// Create a category in a group. It goes last, except that "Other" stays at
/// the end.
pub fn create_category(ids: &mut Ids, group: &PbListCategoryGroup, name: &str) -> CreatedCategory {
    let list_id = group.list_id.clone().unwrap_or_default();
    let sorted = sorted_categories(group);
    let category = PbListCategory {
        identifier: Some(ids.new_id()),
        category_group_id: group.identifier.clone(),
        list_id: Some(list_id.clone()),
        name: Some(name.to_string()),
        icon: Some("other".to_string()),
        sort_index: Some(
            group
                .categories
                .iter()
                .filter_map(|c| c.sort_index)
                .max()
                .map_or(0, |max| max + 1),
        ),
        ..Default::default()
    };
    let create = PbListOperation {
        updated_category: Some(category.clone()),
        ..category_operation(
            ids,
            "create-category",
            OperationClass::ListCategory,
            &list_id,
        )
    };
    let mut requests = vec![Request::list(endpoint::SHOPPING_LISTS_V2, vec![create])];

    if let Some((last, rest)) = sorted.split_last() {
        if last.system_category.as_deref() == Some("other") {
            let order = rest
                .iter()
                .map(|c| c.identifier.clone().unwrap_or_default())
                .chain(category.identifier.clone())
                .chain(last.identifier.clone());
            let sort = set_sorted_category_ids(ids, &list_id, group, order.collect::<Vec<_>>());
            requests.push(Request::list(endpoint::SHOPPING_LISTS_V2, vec![sort]));
        }
    }
    CreatedCategory { category, requests }
}

/// Rename a category.
pub fn rename_category(ids: &mut Ids, category: &PbListCategory, name: &str) -> Vec<Request> {
    let operation = PbListOperation {
        updated_category: Some(PbListCategory {
            name: Some(name.to_string()),
            ..category.clone()
        }),
        ..category_operation(
            ids,
            "set-category-name",
            OperationClass::ListCategory,
            category.list_id.as_deref().unwrap_or_default(),
        )
    };
    vec![Request::list(endpoint::SHOPPING_LISTS_V2, vec![operation])]
}

/// Delete a category: remove it and the categorization rules that point to
/// it, close the gap in the order, then move its items to the group's
/// default category.
///
/// `list` is the list before the deletion; `items` are the list's items.
pub fn delete_category(
    ids: &mut Ids,
    list: ListContext,
    category: &PbListCategory,
    items: &[PbListItem],
) -> Vec<Request> {
    let list_id = list.list_id;
    let category_id = category.identifier.clone().unwrap_or_default();
    let Some(group) = list
        .groups()
        .find(|g| g.identifier == category.category_group_id)
    else {
        return Vec::new();
    };

    let remove_category = PbListOperation {
        updated_category_group: Some(PbListCategoryGroup {
            categories: vec![category.clone()],
            ..category_group_copy(group)
        }),
        ..category_operation(
            ids,
            "remove-category-ids",
            OperationClass::ListCategoryGroup,
            list_id,
        )
    };
    let remove_rules = PbListOperation {
        updated_category_group: Some(PbListCategoryGroup {
            categories: vec![PbListCategory {
                identifier: Some(category_id.clone()),
                ..Default::default()
            }],
            ..category_group_copy(group)
        }),
        ..category_operation(
            ids,
            "remove-categorization-rules-for-category-ids",
            OperationClass::ListCategorizationRule,
            list_id,
        )
    };
    let order: Vec<String> = sorted_categories(group)
        .into_iter()
        .filter_map(|c| c.identifier.clone())
        .filter(|id| *id != category_id)
        .collect();
    let sort = set_sorted_category_ids(ids, list_id, group, order);

    let mut requests = vec![
        Request::list(endpoint::SHOPPING_LISTS_V2, vec![remove_category]),
        Request::list(endpoint::SHOPPING_LISTS_V2, vec![remove_rules, sort]),
    ];

    // The list as it is after the deletion.
    let mut response = list.response.cloned().unwrap_or_default();
    response
        .categorization_rules
        .retain(|rule| rule.category_id.as_deref() != Some(category_id.as_str()));
    for group in response
        .category_group_responses
        .iter_mut()
        .filter_map(|g| g.category_group.as_mut())
    {
        group
            .categories
            .retain(|c| c.identifier.as_deref() != Some(category_id.as_str()));
    }
    let after = ListContext {
        response: Some(&response),
        ..list
    };
    let default_category = group
        .default_category_id
        .as_deref()
        .and_then(|id| after.category(id));
    if let Some(default_category) = default_category {
        for item in items.iter().filter(|item| {
            item.category_assignments.iter().any(|a| {
                a.category_group_id == group.identifier
                    && a.category_id.as_deref() == Some(category_id.as_str())
            })
        }) {
            requests.extend(set_item_category(ids, after, item, default_category));
        }
    }
    requests
}

// ============================================================================
// Stores
// ============================================================================

fn store_operation(
    ids: &mut Ids,
    handler_id: &str,
    class: OperationClass,
    list_id: &str,
) -> PbListOperation {
    category_operation(ids, handler_id, class, list_id)
}

/// The store and the requests that create it.
pub struct CreatedStore {
    pub store: PbStore,
    pub requests: Vec<Request>,
}

/// Create a store, and the store filter that shows its items.
pub fn create_store(
    ids: &mut Ids,
    list_id: &str,
    name: &str,
    stores: &[PbStore],
    filters: &[PbStoreFilter],
) -> CreatedStore {
    let next_index =
        |indexes: &mut dyn Iterator<Item = i32>| indexes.max().map_or(0, |max| max + 1);
    let store = PbStore {
        identifier: ids.new_id(),
        list_id: Some(list_id.to_string()),
        name: Some(name.to_string()),
        sort_index: Some(next_index(&mut stores.iter().filter_map(|s| s.sort_index))),
        ..Default::default()
    };
    let new_store = PbListOperation {
        updated_store: Some(store.clone()),
        ..store_operation(ids, "new-store", OperationClass::Store, list_id)
    };
    let filter = PbStoreFilter {
        identifier: ids.new_id(),
        list_id: Some(list_id.to_string()),
        name: Some(format!("{} Items", name)),
        store_ids: vec![store.identifier.clone()],
        includes_unassigned_items: Some(false),
        sort_index: Some(next_index(&mut filters.iter().filter_map(|f| f.sort_index))),
        ..Default::default()
    };
    let new_filter = PbListOperation {
        updated_store_filter: Some(filter),
        ..store_operation(
            ids,
            "new-store-filter",
            OperationClass::StoreFilter,
            list_id,
        )
    };
    CreatedStore {
        store,
        requests: vec![
            Request::list(endpoint::SHOPPING_LISTS_V2, vec![new_store]),
            Request::list(endpoint::SHOPPING_LISTS_V2, vec![new_filter]),
        ],
    }
}

/// Rename a store.
pub fn rename_store(ids: &mut Ids, store: &PbStore, name: &str) -> Vec<Request> {
    let operation = PbListOperation {
        updated_store: Some(PbStore {
            name: Some(name.to_string()),
            ..store.clone()
        }),
        ..store_operation(
            ids,
            "set-store-name",
            OperationClass::Store,
            store.list_id.as_deref().unwrap_or_default(),
        )
    };
    vec![Request::list(endpoint::SHOPPING_LISTS_V2, vec![operation])]
}

/// Delete a store: take it off the items of the list and its Favorite and
/// Recent Items, take it out of the store filters (deleting the ones left
/// with no store), then delete it.
pub fn delete_store(
    ids: &mut Ids,
    store: &PbStore,
    filters: &[PbStoreFilter],
    favourites: Option<&PbStarterList>,
    recent_items: Option<&PbStarterList>,
) -> Vec<Request> {
    let list_id = store.list_id.as_deref().unwrap_or_default();
    let store_id = store.identifier.as_str();
    let mut requests = Vec::new();

    let from_list = PbListOperation {
        metadata: Some(ids.metadata("remove-store-id-from-all-items")),
        list_id: Some(list_id.to_string()),
        updated_value: Some(store_id.to_string()),
        ..Default::default()
    };
    requests.push(Request::list(endpoint::SHOPPING_LISTS, vec![from_list]));

    let from_starter_list = |ids: &mut Ids, starter_list: Option<&PbStarterList>| {
        starter_list.map(|starter_list| {
            Request::starter_list(vec![PbStarterListOperation {
                metadata: Some(ids.metadata("remove-store-id-from-all-items")),
                list_id: Some(starter_list.identifier.clone()),
                updated_value: Some(store_id.to_string()),
                ..Default::default()
            }])
        })
    };
    requests.extend(from_starter_list(ids, favourites));

    for filter in filters
        .iter()
        .filter(|f| f.store_ids.iter().any(|id| id == store_id))
    {
        let store_ids: Vec<String> = filter
            .store_ids
            .iter()
            .filter(|id| *id != store_id)
            .cloned()
            .collect();
        let operation = if store_ids.is_empty() {
            PbListOperation {
                updated_store_filter: Some(filter.clone()),
                ..store_operation(
                    ids,
                    "delete-store-filter",
                    OperationClass::StoreFilter,
                    list_id,
                )
            }
        } else {
            PbListOperation {
                updated_store_filter: Some(PbStoreFilter {
                    store_ids,
                    ..filter.clone()
                }),
                ..store_operation(
                    ids,
                    "update-store-filter",
                    OperationClass::StoreFilter,
                    list_id,
                )
            }
        };
        requests.push(Request::list(endpoint::SHOPPING_LISTS_V2, vec![operation]));
    }

    requests.extend(from_starter_list(ids, recent_items));

    let delete = PbListOperation {
        updated_store: Some(store.clone()),
        ..store_operation(ids, "delete-store", OperationClass::Store, list_id)
    };
    requests.push(Request::list(endpoint::SHOPPING_LISTS_V2, vec![delete]));
    requests
}

// ============================================================================
// Favourites
// ============================================================================

/// Add a new item (made with [`new_list_item`] for the shopping list) to the
/// list's Favorite Items.
pub fn add_favourite(
    ids: &mut Ids,
    favourites_id: &str,
    item: PbListItem,
) -> (PbListItem, Vec<Request>) {
    let item = PbListItem {
        list_id: Some(favourites_id.to_string()),
        ..item
    };
    let operation = PbStarterListOperation {
        metadata: Some(ids.metadata("add-item")),
        list_id: Some(favourites_id.to_string()),
        list_item_id: Some(item.identifier.clone()),
        list_item: Some(item.clone()),
        ..Default::default()
    };
    (item, vec![Request::starter_list(vec![operation])])
}

/// Remove an item from a starter list such as Favorite Items.
pub fn remove_starter_list_items(
    ids: &mut Ids,
    starter_list_id: &str,
    items: Vec<PbListItem>,
) -> Vec<Request> {
    vec![Request::starter_list(vec![starter_list_items_operation(
        ids,
        "bulk-remove-list-items",
        starter_list_id,
        items,
    )])]
}

/// Add copies of starter list items (favourites) to a shopping list.
pub fn add_starter_list_items_to_list(
    ids: &mut Ids,
    list_id: &str,
    items: &[PbListItem],
) -> (Vec<PbListItem>, Vec<Request>) {
    let copies: Vec<PbListItem> = items
        .iter()
        .map(|item| PbListItem {
            identifier: ids.new_id(),
            list_id: Some(list_id.to_string()),
            checked: Some(false),
            ..item.clone()
        })
        .collect();
    let operation = PbListOperation {
        metadata: Some(ids.metadata("bulk-add-list-items")),
        list_id: Some(list_id.to_string()),
        list: Some(PbShoppingList {
            identifier: list_id.to_string(),
            items: copies.clone(),
            ..Default::default()
        }),
        ..Default::default()
    };
    (
        copies,
        vec![Request::list(endpoint::SHOPPING_LISTS, vec![operation])],
    )
}

// ============================================================================
// Recipes
// ============================================================================

impl Request {
    fn recipes(operations: Vec<PbRecipeOperation>) -> Self {
        Self::new(endpoint::RECIPES, &PbRecipeOperationList { operations })
    }

    fn calendar(operations: Vec<PbCalendarOperation>) -> Self {
        Self::new(endpoint::CALENDAR, &PbCalendarOperationList { operations })
    }
}

fn recipe_operation(ids: &mut Ids, handler_id: &str, recipe_data_id: &str) -> PbRecipeOperation {
    PbRecipeOperation {
        metadata: Some(ids.metadata(handler_id)),
        recipe_data_id: Some(recipe_data_id.to_string()),
        ..Default::default()
    }
}

/// Save a recipe, new or edited: the whole recipe as the editor leaves it.
pub fn save_recipe(ids: &mut Ids, recipe_data_id: &str, recipe: PbRecipe) -> Vec<Request> {
    let operation = PbRecipeOperation {
        recipe: Some(recipe),
        is_new_recipe_from_web_import: Some(false),
        ..recipe_operation(ids, "save-recipe", recipe_data_id)
    };
    vec![Request::recipes(vec![operation])]
}

/// Delete a recipe: delete its meal plan events, remove its ingredients from
/// the list recipe ingredients go to, then remove the recipe.
///
/// `event_ids` are the recipe's calendar and template events; `list` and
/// `items` are the list recipe ingredients go to, if any.
pub fn delete_recipe(
    ids: &mut Ids,
    recipe_data_id: &str,
    calendar_id: &str,
    recipe: &PbRecipe,
    event_ids: Vec<String>,
    list: Option<(&str, &[PbListItem])>,
) -> Vec<Request> {
    let delete_events = PbCalendarOperation {
        metadata: Some(ids.metadata("delete-events-for-recipe-id")),
        calendar_id: Some(calendar_id.to_string()),
        event_ids,
        updated_event: Some(PbCalendarEvent {
            identifier: ids.new_id(),
            calendar_id: Some(calendar_id.to_string()),
            recipe_id: Some(recipe.identifier.clone()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut requests = vec![Request::calendar(vec![delete_events])];

    let remove_recipe = PbRecipeOperation {
        recipe: Some(recipe.clone()),
        ..recipe_operation(ids, "remove-recipe", recipe_data_id)
    };
    requests.push(Request::recipes(vec![remove_recipe]));

    if let Some((list_id, items)) = list {
        let ingredients: Vec<PbItemIngredient> = items
            .iter()
            .flat_map(|item| &item.ingredients)
            .filter(|i| i.recipe_id.as_deref() == Some(recipe.identifier.as_str()))
            .cloned()
            .collect();
        let mut items = items.to_vec();
        let operations: Vec<PbListOperation> = ingredients
            .iter()
            .filter_map(|ingredient| {
                let item_id = list_item_id_for_ingredient(ingredient, list_id);
                let item = items.iter_mut().find(|i| i.identifier == item_id)?;
                remove_item_ingredient(item, ingredient);
                Some(remove_ingredient_operation(
                    ids, list_id, &item_id, ingredient,
                ))
            })
            .collect();
        if !operations.is_empty() {
            requests.push(Request::list(endpoint::SHOPPING_LISTS, operations));
        }
    }
    requests
}

/// An ingredient with its quantity scaled.
pub fn scaled_ingredient(ingredient: &PbIngredient, scale: f64) -> PbIngredient {
    let Some(quantity) = ingredient.quantity.as_deref().filter(|q| !q.is_empty()) else {
        return ingredient.clone();
    };
    if scale == 1.0 {
        return ingredient.clone();
    }
    let scaled = scale_quantity(quantity, scale);
    let raw_ingredient =
        ingredient
            .raw_ingredient
            .as_deref()
            .map(|raw| match raw.strip_prefix(quantity) {
                Some(rest) => format!("{}{}", scaled, rest),
                None => raw.to_string(),
            });
    PbIngredient {
        quantity: Some(scaled),
        raw_ingredient,
        ..ingredient.clone()
    }
}

/// Simple quantity scaling - attempts to parse and scale numeric quantities
fn scale_quantity(quantity: &str, scale: f64) -> String {
    // Try to parse the first number in the quantity string
    let parts: Vec<&str> = quantity.split_whitespace().collect();
    if parts.is_empty() {
        return quantity.to_string();
    }

    // Try to parse the first part as a number
    if let Ok(num) = parts[0].parse::<f64>() {
        let scaled = num * scale;
        let rest = parts[1..].join(" ");
        if rest.is_empty() {
            format!("{}", scaled)
        } else {
            format!("{} {}", scaled, rest)
        }
    } else {
        quantity.to_string()
    }
}

/// The list item ingredient for a recipe ingredient: the ingredient, its
/// recipe and its parsed quantity.
pub fn item_ingredient(recipe: &PbRecipe, ingredient: &PbIngredient) -> PbItemIngredient {
    let quantity = ingredient.quantity.as_deref().unwrap_or_default();
    PbItemIngredient {
        ingredient: Some(ingredient.clone()),
        recipe_id: Some(recipe.identifier.clone()),
        recipe_name: recipe.name.clone(),
        quantity_pb: (!quantity.trim().is_empty()).then(|| parse_quantity(quantity)),
        ..Default::default()
    }
}

/// Unit spellings the web client normalises, in order: alternatives
/// (regular expressions) and the unit they become.
const UNIT_SPELLINGS: [(&str, &str); 25] = [
    (r"cups?|c\.?", "cup"),
    (r"fluid ounces?|fl\.? oz\.?", "fl oz"),
    (r"gallons?|gal\.?", "gal"),
    (r"ounces?|oz\.?", "oz"),
    (r"pints?|pt\.?", "pt"),
    (r"pounds?|lbs?\.?", "lb"),
    (r"quarts?|qts?\.?", "qt"),
    (r"oz t\.?|t\.? oz", "troy oz"),
    (
        r"tablespoons?|tbsps?\.?|T\.?|tbs?\.?|tbl\.?|msk\.?|ss\.?|spsk\.?|el\.?|rkl\.?",
        "Tbsp",
    ),
    (r"teaspoons?|tsps?\.?|ts?\.?|tsk\.?|tl\.?", "tsp"),
    (r"grams?|gr?\.?", "g"),
    (r"kilograms?|kg\.?", "kg"),
    (r"milligrams?|mg\.?", "mg"),
    (r"liters?|l\.?", "L"),
    (r"deciliters?|dl\.?", "dl"),
    (r"milliliters?|ml\.?|krm\.?", "ml"),
    (r"doz\.?", "dozen"),
    (r"gramm", "g"),
    (r"kilogramm", "kg"),
    (r"tl\.?|teelöffel", "tsp"),
    (r"el\.?|esslöffel", "Tbsp"),
    (r"tasse|tassen|tas\.?", "Tasse"),
    (r"pfund|pf\.?", "Pfund"),
    (r"dose|dosen|do\.?", "can"),
    (r"glas|gläser|gl\.?", "glas"),
];

/// Countable units: singular and plural.
const UNIT_PLURALS: [(&str, &str); 72] = [
    ("peck", "pecks"),
    ("bushel", "bushels"),
    ("bucket", "buckets"),
    ("slice", "slices"),
    ("clove", "cloves"),
    ("loaf", "loaves"),
    ("pinch", "pinches"),
    ("package", "packages"),
    ("can", "cans"),
    ("drop", "drops"),
    ("bunch", "bunches"),
    ("dash", "dashes"),
    ("carton", "cartons"),
    ("piece", "pieces"),
    ("square", "squares"),
    ("tube", "tubes"),
    ("strip", "strips"),
    ("stem", "stems"),
    ("stalk", "stalks"),
    ("sprig", "sprigs"),
    ("spear", "spears"),
    ("sprout", "sprouts"),
    ("sheet", "sheets"),
    ("scoop", "scoops"),
    ("pouch", "pouches"),
    ("packet", "packets"),
    ("pack", "packs"),
    ("leaf", "leaves"),
    ("glass", "glasses"),
    ("cube", "cubes"),
    ("container", "containers"),
    ("cone", "cones"),
    ("box", "boxes"),
    ("bottle", "bottles"),
    ("block", "blocks"),
    ("bag", "bags"),
    ("part", "parts"),
    ("stick", "sticks"),
    ("head", "heads"),
    ("bar", "bars"),
    ("ear", "ears"),
    ("jar", "jars"),
    ("inch", "inches"),
    ("tub", "tubs"),
    ("cup", "cups"),
    ("ounce", "ounces"),
    ("gallon", "gallons"),
    ("pint", "pints"),
    ("pound", "pounds"),
    ("quart", "quarts"),
    ("tablespoon", "tablespoons"),
    ("teaspoon", "teaspoons"),
    ("gram", "grams"),
    ("kilogram", "kilograms"),
    ("milligram", "milligrams"),
    ("liter", "liters"),
    ("deciliter", "deciliters"),
    ("milliliter", "milliliters"),
    ("tasse", "tassen"),
    ("dose", "dosen"),
    ("glas", "gläser"),
    ("packung", "packungen"),
    ("flasche", "flaschen"),
    ("zehe", "zehen"),
    ("knolle", "knollen"),
    ("kopf", "köpfe"),
    ("bund", "bünde"),
    ("blatt", "blätter"),
    ("tropf", "tropfen"),
    ("prise", "prisen"),
    ("stück", "stücke"),
    ("stiel", "stiele"),
];

/// Units whose quantity the web client also uses to price an item.
const PRICED_UNITS: [&str; 16] = [
    "cup", "fl oz", "oz", "tbsp", "tsp", "g", "mg", "l", "dl", "ml", "slice", "clove", "pinch",
    "drop", "dash", "inch",
];

fn unit_regex(alternatives: &str) -> regex::Regex {
    // The web client's \b is ASCII.
    regex::Regex::new(&format!(
        r"(?i)(?:^|(?-u:\b))(?:{})(?:(?-u:\b)|$)",
        alternatives
    ))
    .expect("valid unit pattern")
}

/// The web client's normalised unit (`normalizedUnit`): "cups" → "cup",
/// "mL" → "ml", "slices" → "slice".
pub fn normalized_unit(unit: &str) -> String {
    static PATTERNS: std::sync::OnceLock<Vec<(regex::Regex, &str)>> = std::sync::OnceLock::new();
    static PLURALS: std::sync::OnceLock<Vec<(regex::Regex, &str)>> = std::sync::OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        UNIT_SPELLINGS
            .iter()
            .map(|(alternatives, unit)| (unit_regex(alternatives), *unit))
            .collect()
    });
    let plurals = PLURALS.get_or_init(|| {
        UNIT_PLURALS
            .iter()
            .map(|(singular, plural)| (unit_regex(plural), *singular))
            .collect()
    });
    let mut unit = unit.to_string();
    for (pattern, replacement) in patterns {
        unit = pattern.replace_all(&unit, *replacement).into_owned();
    }
    for (pattern, singular) in plurals {
        unit = pattern.replace(&unit, *singular).into_owned();
    }
    unit.trim().to_string()
}

/// An item name's words, stemmed (Snowball English), as the web client
/// compares item names.
fn stemmed_name(name: &str) -> Vec<String> {
    static STEMMER: std::sync::OnceLock<rust_stemmers::Stemmer> = std::sync::OnceLock::new();
    let stemmer =
        STEMMER.get_or_init(|| rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English));
    name.to_lowercase()
        .split(' ')
        .map(|word| stemmer.stem(word).into_owned())
        .collect()
}

fn raw_package_size(item_ingredient: &PbItemIngredient) -> String {
    item_ingredient
        .package_size_pb
        .as_ref()
        .and_then(|p| p.raw_package_size.clone())
        .unwrap_or_default()
}

/// The list item a recipe ingredient goes to: ingredients with the same
/// (stemmed) name, unit and package size share one item.
pub fn list_item_id_for_ingredient(item_ingredient: &PbItemIngredient, list_id: &str) -> String {
    let ingredient = item_ingredient.ingredient.clone().unwrap_or_default();
    let name = stemmed_name(ingredient.name.as_deref().unwrap_or_default()).join(" ");
    let unit = item_ingredient
        .quantity_pb
        .as_ref()
        .and_then(|q| q.unit.as_deref())
        .map(normalized_unit)
        .unwrap_or_default()
        .to_lowercase();
    let package_size = raw_package_size(item_ingredient).to_lowercase();
    let key = format!(
        "ALName::{}::ALQuantityUnit::{}::ALPackageSize::{}",
        name, unit, package_size
    );
    uuid5(&key, list_id)
}

fn same_recipe_ingredient(a: &PbItemIngredient, b: &PbItemIngredient) -> bool {
    let id = |i: &PbItemIngredient| i.ingredient.as_ref().and_then(|i| i.identifier.clone());
    id(a) == id(b)
        && a.recipe_id.clone().unwrap_or_default() == b.recipe_id.clone().unwrap_or_default()
        && a.event_id.clone().unwrap_or_default() == b.event_id.clone().unwrap_or_default()
}

fn add_item_ingredient(item: &mut PbListItem, ingredient: &PbItemIngredient) {
    match item
        .ingredients
        .iter_mut()
        .find(|i| same_recipe_ingredient(i, ingredient))
    {
        Some(existing) => *existing = ingredient.clone(),
        None => item.ingredients.push(ingredient.clone()),
    }
}

fn remove_item_ingredient(item: &mut PbListItem, ingredient: &PbItemIngredient) {
    item.ingredients
        .retain(|i| !same_recipe_ingredient(i, ingredient));
}

fn ingredient_operation(
    ids: &mut Ids,
    handler_id: &str,
    list_id: &str,
    item_id: &str,
    ingredient: &PbItemIngredient,
) -> PbListOperation {
    PbListOperation {
        list_item: Some(PbListItem {
            identifier: item_id.to_string(),
            list_id: Some(list_id.to_string()),
            ingredients: vec![ingredient.clone()],
            ..Default::default()
        }),
        ..item_operation(ids, handler_id, list_id, item_id)
    }
}

fn remove_ingredient_operation(
    ids: &mut Ids,
    list_id: &str,
    item_id: &str,
    ingredient: &PbItemIngredient,
) -> PbListOperation {
    ingredient_operation(
        ids,
        "remove-ingredient-id-from-list-item",
        list_id,
        item_id,
        ingredient,
    )
}

fn top_of(list: ListContext) -> Option<PbShoppingList> {
    list.new_items_on_top.then(|| PbShoppingList {
        identifier: list.list_id.to_string(),
        new_list_item_position: Some(pb_shopping_list::NewListItemPosition::Top as i32),
        ..Default::default()
    })
}

/// Copy a favourite's or recent item's details, photo, stores, prices,
/// price quantity, price package size and barcode to a new item, as the web
/// client does for an item made from a recipe ingredient
/// (`applyPropertiesFromListItem` with quantity and package size kept).
fn apply_properties_from_item(item: &mut PbListItem, from: &PbListItem) {
    if from.recipe_id.is_none() {
        item.details = from.details.clone();
    }
    item.photo_ids = from.photo_ids.first().cloned().into_iter().collect();
    item.store_ids = Vec::new();
    for store_id in &from.store_ids {
        if !item.store_ids.contains(store_id) {
            item.store_ids.push(store_id.clone());
        }
    }
    item.prices = Vec::new();
    for price in &from.prices {
        let empty = price.amount.unwrap_or_default() == 0.0
            && price.details.as_deref().unwrap_or_default().is_empty();
        if empty {
            item.prices.retain(|p| p.store_id != price.store_id);
            continue;
        }
        match item.prices.iter_mut().find(|p| {
            p.store_id.as_deref().unwrap_or_default()
                == price.store_id.as_deref().unwrap_or_default()
        }) {
            Some(existing) => *existing = price.clone(),
            None => item.prices.push(price.clone()),
        }
    }
    if from.price_quantity_should_override_item_quantity == Some(true) {
        item.price_quantity_pb = Some(from.price_quantity_pb.clone().unwrap_or_default());
        item.price_quantity_should_override_item_quantity =
            from.price_quantity_should_override_item_quantity;
    }
    item.price_package_size_pb = Some(from.price_package_size_pb.clone().unwrap_or_default());
    item.price_package_size_should_override_item_package_size =
        from.price_package_size_should_override_item_package_size;
    item.product_upc = from.product_upc.clone();
}

/// Add recipe ingredients to a list, as the web client does: an ingredient
/// joins the item for the same name, unit and package size, which is
/// uncrossed if it was crossed off; otherwise a new item is made, taking
/// the details, stores and prices of a favourite or recent item of the
/// same name.
pub fn add_ingredients_to_list(
    ids: &mut Ids,
    list: ListContext,
    items: &[PbListItem],
    favourites: Option<&PbStarterList>,
    recent_items: Option<&PbStarterList>,
    ingredients: &[PbItemIngredient],
) -> Vec<Request> {
    let list_id = list.list_id;
    let mut items = items.to_vec();
    let mut operations = Vec::new();

    for ingredient in ingredients {
        let item_id = list_item_id_for_ingredient(ingredient, list_id);
        if let Some(item) = items.iter_mut().find(|i| i.identifier == item_id) {
            add_item_ingredient(item, ingredient);
            operations.push(PbListOperation {
                list: top_of(list),
                ..ingredient_operation(
                    ids,
                    "add-item-ingredient-to-list-item",
                    list_id,
                    &item_id,
                    ingredient,
                )
            });
            if item.checked == Some(true) {
                item.checked = Some(false);
                operations.push(PbListOperation {
                    updated_value: Some("n".to_string()),
                    ..item_operation(ids, "set-list-item-checked", list_id, &item_id)
                });
                if item.item_quantity_should_override_ingredient_quantity == Some(true) {
                    if item.quantity_pb.clone().unwrap_or_default() != PbItemQuantity::default() {
                        item.quantity_pb = Some(PbItemQuantity::default());
                        operations.push(PbListOperation {
                            list_item: Some(PbListItem {
                                identifier: item_id.clone(),
                                list_id: Some(list_id.to_string()),
                                quantity_pb: Some(PbItemQuantity::default()),
                                ..Default::default()
                            }),
                            ..item_operation(ids, "set-list-item-quantity-v2", list_id, &item_id)
                        });
                    }
                    item.item_quantity_should_override_ingredient_quantity = Some(false);
                    operations.push(PbListOperation {
                        list_item: Some(PbListItem {
                            identifier: item_id.clone(),
                            list_id: Some(list_id.to_string()),
                            item_quantity_should_override_ingredient_quantity: Some(false),
                            ..Default::default()
                        }),
                        ..item_operation(
                            ids,
                            "set-item-quantity-should-override-ingredient-quantity",
                            list_id,
                            &item_id,
                        )
                    });
                }
                let others: Vec<PbItemIngredient> = item
                    .ingredients
                    .iter()
                    .filter(|i| !same_recipe_ingredient(i, ingredient))
                    .cloned()
                    .collect();
                for other in others {
                    remove_item_ingredient(item, &other);
                    operations.push(remove_ingredient_operation(ids, list_id, &item_id, &other));
                }
            }
            continue;
        }

        let name = ingredient
            .ingredient
            .as_ref()
            .and_then(|i| i.name.as_deref())
            .unwrap_or_default();
        let mut item = new_list_item_with_id(ids, list, item_id.clone(), name, None);
        item.ingredients = vec![ingredient.clone()];
        if let Some(package_size) = ingredient
            .package_size_pb
            .clone()
            .filter(|p| *p != Default::default())
        {
            item.package_size_pb = Some(package_size);
        }
        let unit = ingredient
            .quantity_pb
            .as_ref()
            .and_then(|q| q.unit.as_deref())
            .map(normalized_unit)
            .unwrap_or_default()
            .to_lowercase();
        if PRICED_UNITS.contains(&unit.as_str()) {
            item.price_quantity_should_override_item_quantity = Some(true);
        }

        let stemmed = stemmed_name(name);
        let package_size = raw_package_size(ingredient);
        let favourite_items = favourites.into_iter().flat_map(|l| l.items.iter());
        let recent = recent_items.into_iter().flat_map(|l| l.items.iter().rev());
        let similar = favourite_items.chain(recent).find(|candidate| {
            stemmed_name(candidate.name.as_deref().unwrap_or_default()) == stemmed
                && candidate
                    .package_size_pb
                    .as_ref()
                    .and_then(|p| p.raw_package_size.clone())
                    .unwrap_or_default()
                    == package_size
        });
        if let Some(similar) = similar {
            apply_properties_from_item(&mut item, similar);
        }

        operations.push(PbListOperation {
            list_item: Some(item.clone()),
            list: top_of(list),
            ..item_operation(ids, "add-item-ingredient-to-list-item", list_id, &item_id)
        });
        if list.new_items_on_top {
            items.insert(0, item);
        } else {
            items.push(item);
        }
    }

    if operations.is_empty() {
        Vec::new()
    } else {
        vec![Request::list(endpoint::SHOPPING_LISTS, operations)]
    }
}

/// After a recipe is saved, update its ingredients on the list recipe
/// ingredients go to, as the web client does when a recipe's name, scale
/// or ingredients change: removed ingredients come off their items, and
/// changed ones are updated or, if they now belong with another item,
/// moved there (unless their item was crossed off).
pub fn update_recipe_ingredients_on_list(
    ids: &mut Ids,
    list: ListContext,
    items: &[PbListItem],
    favourites: Option<&PbStarterList>,
    recent_items: Option<&PbStarterList>,
    old: &PbRecipe,
    new: &PbRecipe,
) -> Vec<Request> {
    let list_id = list.list_id;
    if old.name == new.name
        && old.scale_factor == new.scale_factor
        && old.ingredients == new.ingredients
    {
        return Vec::new();
    }
    let mut operations = Vec::new();
    let mut requests = Vec::new();
    for item in items {
        for linked in item
            .ingredients
            .iter()
            .filter(|i| i.recipe_id.as_deref() == Some(new.identifier.as_str()))
        {
            let ingredient_id = linked
                .ingredient
                .as_ref()
                .and_then(|i| i.identifier.as_deref());
            let ingredient = new
                .ingredients
                .iter()
                .find(|i| i.identifier.is_some() && i.identifier.as_deref() == ingredient_id);
            let Some(ingredient) = ingredient else {
                operations.push(remove_ingredient_operation(
                    ids,
                    list_id,
                    &item.identifier,
                    linked,
                ));
                continue;
            };
            let updated = PbItemIngredient {
                event_id: linked.event_id.clone(),
                event_date: linked.event_date.clone(),
                ..item_ingredient(new, ingredient)
            };
            if list_item_id_for_ingredient(&updated, list_id) != item.identifier {
                operations.push(remove_ingredient_operation(
                    ids,
                    list_id,
                    &item.identifier,
                    linked,
                ));
                if item.checked != Some(true) {
                    if !operations.is_empty() {
                        requests.push(Request::list(
                            endpoint::SHOPPING_LISTS,
                            std::mem::take(&mut operations),
                        ));
                    }
                    requests.extend(add_ingredients_to_list(
                        ids,
                        list,
                        items,
                        favourites,
                        recent_items,
                        &[updated],
                    ));
                }
            } else {
                operations.push(PbListOperation {
                    list: top_of(list),
                    ..ingredient_operation(
                        ids,
                        "add-item-ingredient-to-list-item",
                        list_id,
                        &item.identifier,
                        &updated,
                    )
                });
            }
        }
    }
    if !operations.is_empty() {
        requests.push(Request::list(endpoint::SHOPPING_LISTS, operations));
    }
    requests
}

// ============================================================================
// Recipe collections
// ============================================================================

/// The icon the web client gives a new collection.
const NEW_COLLECTION_ICON: &str = "food_and_cooking/stack_of_recipe_cards";

fn collection_operation(
    ids: &mut Ids,
    handler_id: &str,
    recipe_data_id: &str,
    collection: PbRecipeCollection,
) -> PbRecipeOperation {
    PbRecipeOperation {
        recipe_collection: Some(collection),
        ..recipe_operation(ids, handler_id, recipe_data_id)
    }
}

/// The collection and the requests that create it.
pub struct CreatedCollection {
    pub collection: PbRecipeCollection,
    pub requests: Vec<Request>,
}

/// Create a collection, then give it the default icon.
pub fn create_collection(ids: &mut Ids, recipe_data_id: &str, name: &str) -> CreatedCollection {
    let mut collection = PbRecipeCollection {
        identifier: ids.new_id(),
        name: Some(name.to_string()),
        ..Default::default()
    };
    let new = collection_operation(
        ids,
        "new-recipe-collection",
        recipe_data_id,
        collection.clone(),
    );
    collection.collection_settings = Some(PbRecipeCollectionSettings {
        icon: Some(PbIcon {
            icon_name: Some(NEW_COLLECTION_ICON.to_string()),
            tint_hex_color: None,
        }),
        ..Default::default()
    });
    let set_icon = collection_operation(
        ids,
        "set-recipe-collection-icon",
        recipe_data_id,
        collection.clone(),
    );
    CreatedCollection {
        collection,
        requests: vec![
            Request::recipes(vec![new]),
            Request::recipes(vec![set_icon]),
        ],
    }
}

/// Add recipes to a collection: the stored collection with only the recipes
/// it did not have.
pub fn add_recipes_to_collection(
    ids: &mut Ids,
    recipe_data_id: &str,
    collection: &PbRecipeCollection,
    recipe_ids: Vec<String>,
) -> Vec<Request> {
    let added = PbRecipeCollection {
        recipe_ids,
        ..collection.clone()
    };
    vec![Request::recipes(vec![collection_operation(
        ids,
        "add-recipes-to-collection",
        recipe_data_id,
        added,
    )])]
}

/// Remove a recipe from a collection: the stored collection with only that
/// recipe.
pub fn remove_recipe_from_collection(
    ids: &mut Ids,
    recipe_data_id: &str,
    collection: &PbRecipeCollection,
    recipe_id: &str,
) -> Vec<Request> {
    let removed = PbRecipeCollection {
        recipe_ids: vec![recipe_id.to_string()],
        ..collection.clone()
    };
    vec![Request::recipes(vec![collection_operation(
        ids,
        "remove-recipes-from-collection",
        recipe_data_id,
        removed,
    )])]
}

/// Delete a collection: the stored collection.
pub fn delete_collection(
    ids: &mut Ids,
    recipe_data_id: &str,
    collection: &PbRecipeCollection,
) -> Vec<Request> {
    vec![Request::recipes(vec![collection_operation(
        ids,
        "remove-recipe-collection",
        recipe_data_id,
        collection.clone(),
    )])]
}

// ============================================================================
// Meal plan
// ============================================================================

/// The list item ingredient for a recipe ingredient planned in a meal plan
/// event: scaled as the event says, and linked to the event.
pub fn event_item_ingredient(
    recipe: &PbRecipe,
    ingredient: &PbIngredient,
    event: &PbCalendarEvent,
) -> PbItemIngredient {
    let scale = event
        .recipe_scale_factor
        .filter(|s| *s != 0.0)
        .unwrap_or(1.0);
    PbItemIngredient {
        event_id: Some(event.identifier.clone()),
        event_date: event.date.clone(),
        ..item_ingredient(recipe, &scaled_ingredient(ingredient, scale))
    }
}

/// After a recipe event's date or scale changes, update its ingredients on
/// the list recipe ingredients go to.
pub fn update_event_ingredients_on_list(
    ids: &mut Ids,
    list: ListContext,
    items: &[PbListItem],
    recipe: &PbRecipe,
    old: &PbCalendarEvent,
    new: &PbCalendarEvent,
) -> Vec<Request> {
    if old.date == new.date && old.recipe_scale_factor == new.recipe_scale_factor {
        return Vec::new();
    }
    let mut operations = Vec::new();
    for item in items {
        for linked in item
            .ingredients
            .iter()
            .filter(|i| i.event_id.as_deref() == Some(new.identifier.as_str()))
        {
            let ingredient_id = linked
                .ingredient
                .as_ref()
                .and_then(|i| i.identifier.as_deref());
            let Some(ingredient) = recipe
                .ingredients
                .iter()
                .find(|i| i.identifier.is_some() && i.identifier.as_deref() == ingredient_id)
            else {
                continue;
            };
            let updated = event_item_ingredient(recipe, ingredient, new);
            operations.push(PbListOperation {
                list: top_of(list),
                ..ingredient_operation(
                    ids,
                    "add-item-ingredient-to-list-item",
                    list.list_id,
                    &item.identifier,
                    &updated,
                )
            });
        }
    }
    if operations.is_empty() {
        Vec::new()
    } else {
        vec![Request::list(endpoint::SHOPPING_LISTS, operations)]
    }
}

fn calendar_operation(ids: &mut Ids, handler_id: &str, calendar_id: &str) -> PbCalendarOperation {
    PbCalendarOperation {
        metadata: Some(ids.metadata(handler_id)),
        calendar_id: Some(calendar_id.to_string()),
        ..Default::default()
    }
}

/// The web client's sort index for an event added to a day, or moved to
/// another day or label: after the day's other events. `events` are the
/// stored calendar events.
fn order_added_sort_index(event: &PbCalendarEvent, events: &[PbCalendarEvent]) -> i32 {
    let on_date: Vec<&PbCalendarEvent> = events
        .iter()
        .filter(|e| e.date.is_some() && e.date == event.date)
        .collect();
    if on_date.is_empty() {
        return 0;
    }
    on_date
        .iter()
        .filter(|e| e.identifier != event.identifier)
        .filter_map(|e| e.order_added_sort_index)
        .fold(0, i32::max)
        + 1
}

/// A new calendar event. A recipe on its own is added the way a recipe is
/// added to the meal plan; anything else the way the event editor saves a
/// new event.
pub fn create_event(
    ids: &mut Ids,
    calendar_id: &str,
    event: PbCalendarEvent,
    events: &[PbCalendarEvent],
) -> (PbCalendarEvent, Vec<Request>) {
    let event = PbCalendarEvent {
        identifier: ids.new_id(),
        calendar_id: Some(calendar_id.to_string()),
        ..event
    };
    let event = PbCalendarEvent {
        order_added_sort_index: Some(order_added_sort_index(&event, events)),
        ..event
    };
    let recipe_only =
        event.recipe_id.is_some() && event.title.is_none() && event.label_id.is_none();
    let operation = if recipe_only {
        let mut operation = calendar_operation(ids, "save-new-events", calendar_id);
        if let Some(metadata) = operation.metadata.as_mut() {
            metadata.operation_version = Some(1);
        }
        PbCalendarOperation {
            updated_events: vec![event.clone()],
            ..operation
        }
    } else {
        PbCalendarOperation {
            updated_event: Some(event.clone()),
            event_type: Some(event.event_type.unwrap_or_default()),
            ..calendar_operation(ids, "new-event", calendar_id)
        }
    };
    (event, vec![Request::calendar(vec![operation])])
}

/// Save an edited event, as the event editor does: the whole event, moved to
/// the end of its day if its date or label changed.
pub fn update_event(
    ids: &mut Ids,
    stored: &PbCalendarEvent,
    event: PbCalendarEvent,
    events: &[PbCalendarEvent],
) -> Vec<Request> {
    let mut event = event;
    if event.date != stored.date
        || event.label_id.clone().unwrap_or_default() != stored.label_id.clone().unwrap_or_default()
    {
        event.order_added_sort_index = Some(order_added_sort_index(&event, events));
    }
    let calendar_id = stored.calendar_id.as_deref().unwrap_or_default();
    let operation = PbCalendarOperation {
        event_type: Some(event.event_type.unwrap_or_default()),
        updated_event: Some(event),
        ..calendar_operation(ids, "update-event", calendar_id)
    };
    vec![Request::calendar(vec![operation])]
}

/// Delete an event, and take its ingredients off the list recipe
/// ingredients go to.
pub fn delete_event(
    ids: &mut Ids,
    stored: &PbCalendarEvent,
    list: Option<(&str, &[PbListItem])>,
) -> Vec<Request> {
    let calendar_id = stored.calendar_id.as_deref().unwrap_or_default();
    let operation = PbCalendarOperation {
        updated_event: Some(stored.clone()),
        ..calendar_operation(ids, "delete-event", calendar_id)
    };
    let mut requests = vec![Request::calendar(vec![operation])];

    if let Some((list_id, items)) = list {
        let ingredients: Vec<PbItemIngredient> = items
            .iter()
            .flat_map(|item| &item.ingredients)
            .filter(|i| i.event_id.as_deref() == Some(stored.identifier.as_str()))
            .cloned()
            .collect();
        let operations: Vec<PbListOperation> = ingredients
            .iter()
            .filter_map(|ingredient| {
                let item_id = list_item_id_for_ingredient(ingredient, list_id);
                items
                    .iter()
                    .any(|i| i.identifier == item_id)
                    .then(|| remove_ingredient_operation(ids, list_id, &item_id, ingredient))
            })
            .collect();
        if !operations.is_empty() {
            requests.push(Request::list(endpoint::SHOPPING_LISTS, operations));
        }
    }
    requests
}
