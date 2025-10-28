use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::pb_operation_metadata::OperationClass;
use crate::protobuf::anylist::{
    PbListOperation, PbListOperationList, PbOperationMetadata, PbStore,
};
use crate::utils::generate_id;
use prost::Message;

#[derive(Debug, Clone)]
pub struct Store {
    pub id: String,
    pub name: String,
    pub sort_index: i32,
}

#[derive(Debug, Clone)]
pub struct StoreFilter {
    pub id: String,
    pub name: String,
    pub store_ids: Vec<String>,
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
            updated_store: Some(new_store),
            ..Default::default()
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
            updated_value: Some(new_name.to_string()),
            updated_store: Some(updated_store),
            ..Default::default()
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

    /// Get all store filters for a specific list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    pub async fn get_store_filters_for_list(&self, list_id: &str) -> Result<Vec<StoreFilter>> {
        let data = self.get_user_data().await?;

        if let Some(shopping_lists_response) = data.shopping_lists_response {
            for list_response in shopping_lists_response.list_responses {
                if let Some(id) = &list_response.list_id {
                    if id == list_id {
                        let filters: Vec<StoreFilter> = list_response
                            .store_filters
                            .into_iter()
                            .filter_map(|pb_filter| {
                                pb_filter.name.map(|name| StoreFilter {
                                    id: pb_filter.identifier,
                                    name,
                                    store_ids: pb_filter.store_ids,
                                })
                            })
                            .collect();

                        return Ok(filters);
                    }
                }
            }
        }

        Err(AnyListError::NotFound(format!(
            "List with ID {} not found",
            list_id
        )))
    }

    /// Remove a store ID from all items in shopping lists
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store to remove from items
    async fn remove_store_from_shopping_list_items(&self, list_id: &str, store_id: &str) -> Result<()> {
        let operation_id = generate_id();

        // Imperative shell: gather runtime values
        let params = crate::operations::RemoveStoreFromItemsParams {
            list_id: list_id.to_string(),
            store_id: store_id.to_string(),
            operation_id,
            user_id: self.user_id(),
        };

        // Functional core: pure operation building
        let operation_list = crate::operations::build_remove_store_from_items_operation(params);

        // Imperative shell: side effects
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update", buf).await?;
        Ok(())
    }

    /// Remove a store ID from all items in starter lists
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store to remove from items
    async fn remove_store_from_starter_list_items(&self, list_id: &str, store_id: &str) -> Result<()> {
        let operation_id = generate_id();

        // Imperative shell: gather runtime values
        let params = crate::operations::RemoveStoreFromItemsParams {
            list_id: list_id.to_string(),
            store_id: store_id.to_string(),
            operation_id,
            user_id: self.user_id(),
        };

        // Functional core: pure operation building
        let operation_list = crate::operations::build_remove_store_from_items_operation(params);

        // Imperative shell: side effects
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/starter-lists/update", buf).await?;
        Ok(())
    }

    /// Update or delete store filters that contain a specific store ID
    ///
    /// This follows the webapp's orchestration logic:
    /// - If a filter contains the deleted store and still has other stores after removal: UPDATE the filter
    /// - If a filter contains only the deleted store and becomes empty: DELETE the filter
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store to remove from filters
    async fn delete_store_filters_with_store(&self, list_id: &str, store_id: &str) -> Result<()> {
        let filters = self.get_store_filters_for_list(list_id).await?;

        // Find filters that contain this store_id
        let affected_filters: Vec<_> = filters
            .into_iter()
            .filter(|filter| filter.store_ids.contains(&store_id.to_string()))
            .collect();

        if affected_filters.is_empty() {
            return Ok(());
        }

        // Process each affected filter
        for filter in affected_filters {
            let operation_id = generate_id();

            // Remove the store ID from the filter's store_ids array
            let mut updated_store_ids = filter.store_ids.clone();
            updated_store_ids.retain(|id| id != store_id);

            // Decide whether to UPDATE or DELETE based on remaining stores
            if updated_store_ids.is_empty() {
                // DELETE: Filter has no stores left
                let params = crate::operations::DeleteStoreFilterParams {
                    filter_id: filter.id.clone(),
                    list_id: list_id.to_string(),
                    filter_name: filter.name.clone(),
                    operation_id,
                    user_id: self.user_id(),
                };

                let operation_list = crate::operations::build_delete_store_filter_operation(params);

                let mut buf = Vec::new();
                operation_list.encode(&mut buf).map_err(|e| {
                    AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
                })?;

                self.post("data/shopping-lists/update-v2", buf).await?;
            } else {
                // UPDATE: Filter still has other stores
                let params = crate::operations::UpdateStoreFilterParams {
                    filter_id: filter.id.clone(),
                    list_id: list_id.to_string(),
                    filter_name: filter.name.clone(),
                    store_ids: updated_store_ids,
                    operation_id,
                    user_id: self.user_id(),
                };

                let operation_list = crate::operations::build_update_store_filter_operation(params);

                let mut buf = Vec::new();
                operation_list.encode(&mut buf).map_err(|e| {
                    AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
                })?;

                self.post("data/shopping-lists/update-v2", buf).await?;
            }
        }

        Ok(())
    }

    /// Delete a store (with proper cleanup of items and filters)
    ///
    /// This method performs the following operations in order:
    /// 1. Remove the store from all items in shopping lists (data/shopping-lists/update)
    /// 2. Remove the store from all items in starter lists (data/starter-lists/update)
    /// 3. Delete any store filters that contain this store (data/shopping-lists/update-v2)
    /// 4. Remove the store from starter list items again (data/starter-lists/update)
    /// 5. Delete the store itself (data/shopping-lists/update-v2)
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store to delete
    pub async fn delete_store(&self, list_id: &str, store_id: &str) -> Result<()> {
        // Step 1: Remove store from all items in shopping lists
        self.remove_store_from_shopping_list_items(list_id, store_id).await?;

        // Step 2: Remove store from all items in starter lists
        self.remove_store_from_starter_list_items(list_id, store_id).await?;

        // Step 3: Delete store filters containing this store
        self.delete_store_filters_with_store(list_id, store_id).await?;

        // Step 4: Remove store from starter list items again
        self.remove_store_from_starter_list_items(list_id, store_id).await?;

        // Step 5: Delete the store itself
        let operation_id = generate_id();

        // Imperative shell: gather runtime values
        let params = crate::operations::DeleteStoreParams {
            store_id: store_id.to_string(),
            store_name: None,
            list_id: list_id.to_string(),
            operation_id,
            user_id: self.user_id(),
        };

        // Functional core: pure operation building
        let operation_list = crate::operations::build_delete_store_operation(params);

        // Imperative shell: side effects
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update-v2", buf).await?;
        Ok(())
    }
}
