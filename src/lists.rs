use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::pb_operation_metadata::OperationClass;
use crate::protobuf::anylist::{
    PbListItem, PbListOperation, PbListOperationList, PbOperationMetadata, PbShoppingList,
    PbShoppingListsResponse, PbUserDataResponse,
};
use crate::utils::{current_timestamp, generate_id};
use prost::Message;

/// Represents a shopping list item
#[derive(Debug, Clone)]
pub struct ListItem {
    pub id: String,
    pub list_id: String,
    pub name: String,
    pub details: String,
    pub is_checked: bool,
    pub quantity: Option<String>,
    pub category: Option<String>,
}

/// Represents a shopping list
#[derive(Debug, Clone)]
pub struct List {
    pub id: String,
    pub name: String,
    pub items: Vec<ListItem>,
}

impl AnyListClient {
    /// Get all shopping lists for the authenticated user
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let lists = client.get_lists().await.expect("Failed to get lists");
    /// for list in lists {
    ///     println!("List: {} ({} items)", list.name, list.items.len());
    /// }
    /// # }
    /// ```
    pub async fn get_lists(&self) -> Result<Vec<List>> {
        let data = self.get_user_data().await?;
        let lists = match data.shopping_lists_response {
            Some(ref res) => lists_from_response(res.clone()),
            None => Vec::new(),
        };
        Ok(lists)
    }

    /// Get a specific list by ID
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to retrieve
    pub async fn get_list_by_id(&self, list_id: &str) -> Result<List> {
        let lists = self.get_lists().await?;
        lists
            .into_iter()
            .find(|l| l.id == list_id)
            .ok_or_else(|| AnyListError::NotFound(format!("List with ID {} not found", list_id)))
    }

    /// Get a specific list by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the list to retrieve
    pub async fn get_list_by_name(&self, name: &str) -> Result<List> {
        let lists = self.get_lists().await?;
        lists
            .into_iter()
            .find(|l| l.name == name)
            .ok_or_else(|| AnyListError::NotFound(format!("List with name '{}' not found", name)))
    }

    /// Create a new shopping list
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the new list
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let list = client.create_list("Groceries").await.expect("Failed to create list");
    /// println!("Created list: {}", list.name);
    /// # }
    /// ```
    pub async fn create_list(&self, name: &str) -> Result<List> {
        let list_id = generate_id();
        let operation_id = generate_id();

        let new_list = PbShoppingList {
            identifier: list_id.clone(),
            timestamp: Some(current_timestamp()),
            name: Some(name.to_string()),
            items: vec![],
            creator: Some(self.user_id()),
            unusedattribute: vec![],
            shared_users: vec![],
            password: None,
            notification_locations: vec![],
            logical_clock_time: Some(1),
            built_in_alexa_list_type: None,
            allows_multiple_list_category_groups: Some(true),
            list_item_sort_order: Some(0),   // Manual
            new_list_item_position: Some(0), // Bottom
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("new-shopping-list".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.clone()),
            list_item_id: None,
            updated_value: None,
            original_value: None,
            list_item: None,
            list: Some(new_list),
            list_folder_id: None,
            notification_location: None,
            updated_store: None,
            original_store: None,
            sorted_store_ids: vec![],
            updated_store_filter: None,
            original_store_filter: None,
            sorted_store_filter_ids: vec![],
            item_price: None,
            updated_category: None,
            original_category: None,
            updated_category_group: None,
            original_category_group: None,
            updated_categorization_rule: None,
            original_categorization_rule: None,
            updated_categorization_rules: vec![],
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;

        Ok(List {
            id: list_id,
            name: name.to_string(),
            items: vec![],
        })
    }

    /// Delete a shopping list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to delete
    pub async fn delete_list(&self, list_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("delete-list".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: None,
            updated_value: None,
            original_value: None,
            list_item: None,
            list: None,
            list_folder_id: None,
            notification_location: None,
            updated_store: None,
            original_store: None,
            sorted_store_ids: vec![],
            updated_store_filter: None,
            original_store_filter: None,
            sorted_store_filter_ids: vec![],
            item_price: None,
            updated_category: None,
            original_category: None,
            updated_category_group: None,
            original_category_group: None,
            updated_categorization_rule: None,
            original_categorization_rule: None,
            updated_categorization_rules: vec![],
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;
        Ok(())
    }

    /// Rename a shopping list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to rename
    /// * `new_name` - The new name for the list
    pub async fn rename_list(&self, list_id: &str, new_name: &str) -> Result<()> {
        let operation_id = generate_id();

        // Get the current list to preserve other fields
        let current_list = self.get_list_by_id(list_id).await?;

        let updated_list = PbShoppingList {
            identifier: list_id.to_string(),
            timestamp: Some(current_timestamp()),
            name: Some(new_name.to_string()),
            items: vec![],
            creator: Some(self.user_id()),
            unusedattribute: vec![],
            shared_users: vec![],
            password: None,
            notification_locations: vec![],
            logical_clock_time: Some(1),
            built_in_alexa_list_type: None,
            allows_multiple_list_category_groups: Some(true),
            list_item_sort_order: Some(0),
            new_list_item_position: Some(0),
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("rename-list".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: None,
            updated_value: Some(new_name.to_string()),
            original_value: Some(current_list.name),
            list_item: None,
            list: Some(updated_list),
            list_folder_id: None,
            notification_location: None,
            updated_store: None,
            original_store: None,
            sorted_store_ids: vec![],
            updated_store_filter: None,
            original_store_filter: None,
            sorted_store_filter_ids: vec![],
            item_price: None,
            updated_category: None,
            original_category: None,
            updated_category_group: None,
            original_category_group: None,
            updated_categorization_rule: None,
            original_categorization_rule: None,
            updated_categorization_rules: vec![],
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;
        Ok(())
    }

    /// Get user data from the API
    pub async fn get_user_data(&self) -> Result<PbUserDataResponse> {
        let bytes = self.post("data/user-data/get", vec![]).await?;
        let data = PbUserDataResponse::decode(bytes.as_ref())?;
        Ok(data)
    }
}

fn transform_api_list_item(items: Vec<PbListItem>) -> Vec<ListItem> {
    let mut result: Vec<ListItem> = Vec::new();
    for item in items {
        if let (Some(name), Some(list_id)) = (item.name, item.list_id) {
            let item = ListItem {
                id: item.identifier,
                list_id: list_id.clone(),
                name,
                details: item.details.unwrap_or("".to_string()),
                is_checked: item.checked.unwrap_or(false),
                quantity: item.quantity,
                category: item.category,
            };
            result.push(item);
        }
    }
    result
}

fn lists_from_response(response: PbShoppingListsResponse) -> Vec<List> {
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
