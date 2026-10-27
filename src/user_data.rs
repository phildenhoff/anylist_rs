//! Lookups in the user data response: the state the web client keeps in
//! memory and reads before it sends an operation.

use crate::operations::{ListContext, DEFAULT_LIST_THEME_ID};
use crate::protobuf::anylist::{
    pb_shopping_list, pb_starter_list, PbListCategoryGroup, PbListItem, PbListResponse,
    PbListSettings, PbShoppingList, PbStarterList, PbUserDataResponse,
};

/// The list settings that hold the defaults for new lists.
const NEW_LIST_SETTINGS_ID: &str = "ALCustomSettingsListID";

impl PbUserDataResponse {
    pub(crate) fn shopping_list(&self, list_id: &str) -> Option<&PbShoppingList> {
        self.shopping_lists_response
            .as_ref()?
            .new_lists
            .iter()
            .find(|l| l.identifier == list_id)
    }

    pub(crate) fn list_item(&self, list_id: &str, item_id: &str) -> Option<&PbListItem> {
        self.shopping_list(list_id)?
            .items
            .iter()
            .find(|i| i.identifier == item_id)
    }

    pub(crate) fn list_response(&self, list_id: &str) -> Option<&PbListResponse> {
        self.shopping_lists_response
            .as_ref()?
            .list_responses
            .iter()
            .find(|r| r.list_id.as_deref() == Some(list_id))
    }

    pub(crate) fn category_group(
        &self,
        list_id: &str,
        category_group_id: &str,
    ) -> Option<&PbListCategoryGroup> {
        self.list_response(list_id)?
            .category_group_responses
            .iter()
            .filter_map(|g| g.category_group.as_ref())
            .find(|g| g.identifier.as_deref() == Some(category_group_id))
    }

    pub(crate) fn list_settings(&self, list_id: &str) -> Option<&PbListSettings> {
        self.list_settings_response
            .as_ref()?
            .settings
            .iter()
            .find(|s| s.list_id.as_deref() == Some(list_id))
    }

    /// The categories, rules and settings the web client uses to add and
    /// edit the list's items.
    pub(crate) fn list_context<'a>(&'a self, list_id: &'a str) -> ListContext<'a> {
        ListContext {
            list_id,
            response: self.list_response(list_id),
            category_group_id: self
                .list_settings(list_id)
                .and_then(|s| s.list_category_group_id.as_deref()),
            new_items_on_top: self.shopping_list(list_id).is_some_and(|l| {
                l.new_list_item_position == Some(pb_shopping_list::NewListItemPosition::Top as i32)
            }),
        }
    }

    /// The list's Recent Items or Favorite Items.
    pub(crate) fn starter_list(
        &self,
        list_id: &str,
        kind: pb_starter_list::Type,
    ) -> Option<&PbStarterList> {
        let response = self.starter_lists_response.as_ref()?;
        let batch = match kind {
            pb_starter_list::Type::RecentItems => response.recent_item_lists_response.as_ref(),
            pb_starter_list::Type::FavoriteItems => response.favorite_item_lists_response.as_ref(),
            pb_starter_list::Type::User => response.user_lists_response.as_ref(),
        }?;
        batch
            .list_responses
            .iter()
            .filter_map(|r| r.starter_list.as_ref())
            .find(|l| l.list_id.as_deref() == Some(list_id))
    }

    /// The account's list data ID, which identifies its folder tree.
    pub(crate) fn list_data_id(&self) -> Option<&str> {
        self.list_folders_response.as_ref()?.list_data_id.as_deref()
    }

    /// The folder a list or folder is in.
    pub(crate) fn parent_folder_id(&self, item_id: &str) -> Option<&str> {
        let folders = self.list_folders_response.as_ref()?;
        folders
            .list_folders
            .iter()
            .find(|f| f.items.iter().any(|i| i.identifier == item_id))
            .map(|f| f.identifier.as_str())
            .or(folders.root_folder_id.as_deref())
    }

    pub(crate) fn root_folder_id(&self) -> Option<&str> {
        let folders = self.list_folders_response.as_ref()?;
        folders
            .root_folder_id
            .as_deref()
            .or(folders.list_data_id.as_deref())
    }

    /// The theme new lists get.
    pub(crate) fn new_list_theme_id(&self) -> &str {
        self.list_settings(NEW_LIST_SETTINGS_ID)
            .and_then(|s| s.list_theme_id.as_deref())
            .unwrap_or(DEFAULT_LIST_THEME_ID)
    }

    /// The user's email, as their lists record it.
    pub(crate) fn email_of(&self, user_id: &str) -> Option<&str> {
        self.shopping_lists_response
            .as_ref()?
            .new_lists
            .iter()
            .flat_map(|l| &l.shared_users)
            .find(|u| u.user_id.as_deref() == Some(user_id))
            .and_then(|u| u.email.as_deref())
    }
}
