use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::operations;
use crate::protobuf::anylist::{pb_starter_list, PbListResponse, PbStore, PbUserDataResponse};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Store {
    id: String,
    name: String,
    sort_index: i32,
}

impl Store {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sort_index(&self) -> i32 {
        self.sort_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreFilter {
    id: String,
    name: String,
    store_ids: Vec<String>,
}

impl StoreFilter {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn store_ids(&self) -> &[String] {
        &self.store_ids
    }
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
    /// Like the web client, this also creates a store filter ("<name>
    /// Items") that shows the store's items, and refuses a name another of
    /// the list's stores has.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `name` - The name of the store
    pub async fn create_store(&self, list_id: &str, name: &str) -> Result<Store> {
        let data = self.get_user_data().await?;
        let response = list_response(&data, list_id)?;
        check_store_name(response, None, name)?;
        let created = operations::create_store(
            &mut self.ids(),
            list_id,
            name,
            &response.stores,
            &response.store_filters,
        );
        self.send(created.requests).await?;
        Ok(Store {
            id: created.store.identifier,
            name: name.to_string(),
            sort_index: created.store.sort_index.unwrap_or_default(),
        })
    }

    /// Update a store's name
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store
    /// * `new_name` - The new name for the store
    ///
    /// Like the web client, this refuses a name another of the list's stores
    /// has.
    pub async fn update_store(&self, list_id: &str, store_id: &str, new_name: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let store = stored_store(&data, list_id, store_id)?;
        if store.name.as_deref() == Some(new_name) {
            return Ok(());
        }
        check_store_name(list_response(&data, list_id)?, Some(store_id), new_name)?;
        let requests = operations::rename_store(&mut self.ids(), store, new_name);
        self.send(requests).await
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

    /// Delete a store
    ///
    /// Like the web client, this takes the store off the items of the list
    /// and of its Favorite and Recent Items, takes it out of the list's store
    /// filters (deleting the ones left with no store), then deletes it.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `store_id` - The ID of the store to delete
    pub async fn delete_store(&self, list_id: &str, store_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let store = stored_store(&data, list_id, store_id)?;
        let filters = &list_response(&data, list_id)?.store_filters;
        let requests = operations::delete_store(
            &mut self.ids(),
            store,
            filters,
            data.starter_list(list_id, pb_starter_list::Type::FavoriteItems),
            data.starter_list(list_id, pb_starter_list::Type::RecentItems),
        );
        self.send(requests).await
    }
}

fn list_response<'a>(data: &'a PbUserDataResponse, list_id: &str) -> Result<&'a PbListResponse> {
    data.list_response(list_id)
        .ok_or_else(|| AnyListError::NotFound(format!("List with ID {} not found", list_id)))
}

fn stored_store<'a>(
    data: &'a PbUserDataResponse,
    list_id: &str,
    store_id: &str,
) -> Result<&'a PbStore> {
    list_response(data, list_id)?
        .stores
        .iter()
        .find(|s| s.identifier == store_id)
        .ok_or_else(|| {
            AnyListError::NotFound(format!("Store {} not found in list {}", store_id, list_id))
        })
}

/// The web client refuses a store name another of the list's stores has,
/// ignoring case.
fn check_store_name(response: &PbListResponse, store_id: Option<&str>, name: &str) -> Result<()> {
    let lowercase = name.to_lowercase();
    let taken = response.stores.iter().any(|s| {
        Some(s.identifier.as_str()) != store_id
            && s.name
                .as_deref()
                .is_some_and(|n| n.to_lowercase() == lowercase)
    });
    if taken {
        return Err(AnyListError::Other(format!(
            "There is already a store named {}",
            name
        )));
    }
    Ok(())
}
