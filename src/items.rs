use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::lists::ListItem;
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbListItem, PbListOperation, PbListOperationList,
    PbOperationMetadata,
};
use crate::utils::{current_timestamp, generate_id};
use prost::Message;

impl AnyListClient {
    /// Add an item to a shopping list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to add the item to
    /// * `name` - The name of the item
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
    /// client.add_item("list-id", "Milk")
    ///     .await
    ///     .expect("Failed to add item");
    /// # }
    /// ```
    pub async fn add_item(&self, list_id: &str, name: &str) -> Result<ListItem> {
        self.add_item_with_details(list_id, name, None, None, None)
            .await
    }

    /// Add an item to a shopping list with additional details
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to add the item to
    /// * `name` - The name of the item
    /// * `quantity` - Optional quantity (e.g., "2", "1 lb", "500g")
    /// * `details` - Optional additional details/notes
    /// * `category` - Optional category name
    pub async fn add_item_with_details(
        &self,
        list_id: &str,
        name: &str,
        quantity: Option<&str>,
        details: Option<&str>,
        category: Option<&str>,
    ) -> Result<ListItem> {
        let item_id = generate_id();
        let operation_id = generate_id();

        let new_item = PbListItem {
            identifier: item_id.clone(),
            server_mod_time: Some(current_timestamp()),
            list_id: Some(list_id.to_string()),
            name: Some(name.to_string()),
            quantity: quantity.map(|q| q.to_string()),
            details: details.map(|d| d.to_string()),
            checked: Some(false),
            recipe_id: None,
            raw_ingredient: None,
            price_matchup_tag: None,
            price_id: None,
            category: category.map(|c| c.to_string()),
            user_id: Some(self.user_id()),
            category_match_id: None,
            photo_ids: vec![],
            event_id: None,
            store_ids: vec![],
            prices: vec![],
            category_assignments: vec![],
            manual_sort_index: Some(0),
            product_upc: None,
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("add-shopping-list-item".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: Some(item_id.clone()),
            list_item: Some(new_item),
            ..Default::default()
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;

        Ok(ListItem {
            id: item_id,
            list_id: list_id.to_string(),
            name: name.to_string(),
            details: details.unwrap_or("").to_string(),
            is_checked: false,
            quantity: quantity.map(|q| q.to_string()),
            category: category.map(|c| c.to_string()),
            user_id: Some(self.user_id()),
            product_upc: None,
        })
    }

    /// Update an existing item
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to update
    /// * `name` - The new name for the item
    /// * `quantity` - Optional new quantity
    /// * `details` - Optional new details
    /// * `category` - Optional new category
    pub async fn update_item(
        &self,
        list_id: &str,
        item_id: &str,
        name: &str,
        quantity: Option<&str>,
        details: Option<&str>,
        category: Option<&str>,
    ) -> Result<()> {
        let operation_id = generate_id();

        let updated_item = PbListItem {
            identifier: item_id.to_string(),
            server_mod_time: Some(current_timestamp()),
            list_id: Some(list_id.to_string()),
            name: Some(name.to_string()),
            quantity: quantity.map(|q| q.to_string()),
            details: details.map(|d| d.to_string()),
            checked: Some(false),
            recipe_id: None,
            raw_ingredient: None,
            price_matchup_tag: None,
            price_id: None,
            category: category.map(|c| c.to_string()),
            user_id: Some(self.user_id()),
            category_match_id: None,
            photo_ids: vec![],
            event_id: None,
            store_ids: vec![],
            prices: vec![],
            category_assignments: vec![],
            manual_sort_index: Some(0),
            product_upc: None,
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("update-list-item".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: Some(item_id.to_string()),
            list_item: Some(updated_item),
            ..Default::default()
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        // Note: "update-list-item" handler not found in API docs - may need to use specific handlers
        // like set-list-item-name, set-list-item-quantity-v2, set-list-item-details
        self.post("data/shopping-lists/update", buf).await?;
        Ok(())
    }

    /// Delete an item from a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to delete
    pub async fn delete_item(&self, list_id: &str, item_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("remove-shopping-list-item".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: Some(item_id.to_string()),
            ..Default::default()
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

    /// Delete multiple items from a list in a single operation
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the items
    /// * `item_ids` - The IDs of the items to delete
    ///
    /// # Note
    ///
    /// This method requires fetching the list to get full item details,
    /// but is more efficient than multiple `delete_item()` calls when
    /// deleting multiple items.
    pub async fn bulk_delete_items(&self, list_id: &str, item_ids: &[&str]) -> Result<()> {
        if item_ids.is_empty() {
            return Ok(());
        }

        // Fetch the list to get full item details
        let list = self.get_list_by_id(list_id).await?;

        let items_to_remove: Vec<crate::operations::ItemToRemove> = item_ids
            .iter()
            .filter_map(|&item_id| {
                list.items().iter().find(|i| i.id() == item_id).map(|item| {
                    crate::operations::ItemToRemove {
                        item_id: item.id().to_string(),
                        list_id: item.list_id().to_string(),
                        name: item.name().to_string(),
                        category: item.category().map(|s| s.to_string()),
                        user_id: self.user_id(),
                        category_match_id: item.category().map(|s| s.to_string()),
                        category_assignment: None,
                    }
                })
            })
            .collect();

        if items_to_remove.is_empty() {
            return Err(AnyListError::NotFound(
                "No matching items found".to_string(),
            ));
        }

        let operation_id = generate_id();
        let params = crate::operations::BulkRemoveItemsParams {
            operation_id,
            user_id: self.user_id(),
            list_id: list_id.to_string(),
            items: items_to_remove,
        };

        let operation_list = crate::operations::build_bulk_remove_items_operation(params);

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;
        Ok(())
    }

    /// Cross off (check) an item on a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to cross off
    pub async fn cross_off_item(&self, list_id: &str, item_id: &str) -> Result<()> {
        self.set_item_checked(list_id, item_id, true).await
    }

    /// Uncheck (uncross) an item on a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to uncheck
    pub async fn uncheck_item(&self, list_id: &str, item_id: &str) -> Result<()> {
        self.set_item_checked(list_id, item_id, false).await
    }

    /// Set the checked status of an item
    async fn set_item_checked(&self, list_id: &str, item_id: &str, checked: bool) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("set-list-item-checked".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: Some(item_id.to_string()),
            updated_value: Some(if checked { "y" } else { "n" }.to_string()),
            ..Default::default()
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

    /// Delete all crossed-off (checked) items from a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to clear crossed-off items from
    pub async fn delete_all_crossed_off_items(&self, list_id: &str) -> Result<()> {
        let list = self.get_list_by_id(list_id).await?;
        let checked_items: Vec<&ListItem> =
            list.items().iter().filter(|i| i.is_checked()).collect();

        if checked_items.is_empty() {
            return Ok(());
        }

        // Use bulk remove operation for all checked items at once
        let operation_id = generate_id();
        let items_to_remove: Vec<crate::operations::ItemToRemove> = checked_items
            .iter()
            .map(|item| crate::operations::ItemToRemove {
                item_id: item.id().to_string(),
                list_id: item.list_id().to_string(),
                name: item.name().to_string(),
                category: item.category().map(|s| s.to_string()),
                user_id: self.user_id(),
                category_match_id: item.category().map(|s| s.to_string()), // Use same as category for now
                category_assignment: None, // TODO: Store and use actual category assignment
            })
            .collect();

        let params = crate::operations::BulkRemoveItemsParams {
            operation_id,
            user_id: self.user_id(),
            list_id: list_id.to_string(),
            items: items_to_remove,
        };

        let operation_list = crate::operations::build_bulk_remove_items_operation(params);

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;
        Ok(())
    }
}
