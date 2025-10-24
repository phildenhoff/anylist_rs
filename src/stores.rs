use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbListOperation, PbListOperationList,
    PbOperationMetadata, PbStore,
};
use crate::utils::generate_id;
use prost::Message;

#[derive(Debug, Clone)]
pub struct Store {
    pub id: String,
    pub name: String,
    pub sort_index: i32,
}

impl AnyListClient {
    /// Get all stores for a specific list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    pub async fn get_stores_for_list(&self, list_id: &str) -> Result<Vec<Store>> {
        let data = self.get_user_data().await?;

        // Get the stores from the shopping lists response
        if let Some(shopping_lists_response) = data.shopping_lists_response {
            // Find the list response for this specific list
            for list_response in shopping_lists_response.list_responses {
                if let Some(id) = &list_response.list_id {
                    if id == list_id {
                        // Convert PbStore to Store
                        let stores: Vec<Store> = list_response
                            .stores
                            .into_iter()
                            .filter_map(|pb_store| {
                                pb_store.name.map(|name| Store {
                                    id: pb_store.identifier,
                                    name,
                                    sort_index: pb_store.sort_index.unwrap_or(0),
                                })
                            })
                            .collect();

                        return Ok(stores);
                    }
                }
            }
        }

        // If we didn't find the list, return an error
        Err(AnyListError::NotFound(format!(
            "List with ID {} not found",
            list_id
        )))
    }

    /// Create a new store for a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `name` - The name of the store
    pub async fn create_store(&self, list_id: &str, name: &str) -> Result<Store> {
        let store_id = generate_id();
        let operation_id = generate_id();

        let new_store = PbStore {
            identifier: store_id.clone(),
            logical_timestamp: None,
            list_id: Some(list_id.to_string()),
            name: Some(name.to_string()),
            // TODO: set this as the lists num. of stores + 1
            sort_index: Some(0),
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("new-store".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Store as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: None,
            updated_value: None,
            original_value: None,
            list_item: None,
            list: None,
            list_folder_id: None,
            notification_location: None,
            updated_store: Some(new_store),
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

        self.post("data/shopping-lists/update-v2", buf).await?;

        Ok(Store {
            id: store_id,
            name: name.to_string(),
            sort_index: 0,
        })
    }

    /// Update a store's name
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store
    /// * `new_name` - The new name for the store
    pub async fn update_store(&self, list_id: &str, store_id: &str, new_name: &str) -> Result<()> {
        let operation_id = generate_id();

        let updated_store = PbStore {
            identifier: store_id.to_string(),
            logical_timestamp: Some(1),
            list_id: Some(list_id.to_string()),
            name: Some(new_name.to_string()),
            sort_index: Some(0),
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("set-store-name".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Store as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: None,
            updated_value: Some(new_name.to_string()),
            original_value: None,
            list_item: None,
            list: None,
            list_folder_id: None,
            notification_location: None,
            updated_store: Some(updated_store),
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

        self.post("data/shopping-lists/update-v2", buf).await?;
        Ok(())
    }

    /// Delete a store
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store to delete
    pub async fn delete_store(&self, list_id: &str, store_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("delete-store".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Store as i32),
            }),
            list_id: Some(list_id.to_string()),
            list_item_id: None,
            updated_value: None,
            original_value: Some(store_id.to_string()),
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

        self.post("data/shopping-lists/update-v2", buf).await?;
        Ok(())
    }
}
