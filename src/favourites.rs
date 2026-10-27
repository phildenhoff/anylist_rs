use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::lists::{transform_api_list_item, ListItem};
use crate::operations;
use crate::protobuf::anylist::{PbListItem, PbStarterList, PbUserDataResponse};
use serde_derive::{Deserialize, Serialize};

/// Represents a favourite item that can be quickly added to shopping lists
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavouriteItem {
    pub(crate) id: String,
    pub(crate) list_id: String,
    pub(crate) name: String,
    pub(crate) quantity: Option<String>,
    pub(crate) details: Option<String>,
    pub(crate) category: Option<String>,
}

impl FavouriteItem {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn list_id(&self) -> &str {
        &self.list_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn quantity(&self) -> Option<&str> {
        self.quantity.as_deref()
    }

    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
}

/// Represents a favourites list (starter list)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavouritesList {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) items: Vec<FavouriteItem>,
    /// Associated shopping list ID (if linked to a specific list)
    pub(crate) shopping_list_id: Option<String>,
}

impl FavouritesList {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn items(&self) -> &[FavouriteItem] {
        &self.items
    }

    /// Get the associated shopping list ID (if linked)
    pub fn shopping_list_id(&self) -> Option<&str> {
        self.shopping_list_id.as_deref()
    }
}

impl AnyListClient {
    /// Get all favourite items across all favourites lists
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password").await?;
    ///
    /// let favourites = client.get_favourites().await?;
    /// println!("You have {} favourite items", favourites.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_favourites(&self) -> Result<Vec<FavouriteItem>> {
        let lists = self.get_favourites_lists().await?;
        let items: Vec<FavouriteItem> = lists.into_iter().flat_map(|list| list.items).collect();
        Ok(items)
    }

    /// Get all favourites lists
    ///
    /// Returns all starter lists of type `FavoriteItems`.
    pub async fn get_favourites_lists(&self) -> Result<Vec<FavouritesList>> {
        let data = self.get_user_data().await?;

        let lists = match data.starter_lists_response {
            Some(ref res) => match &res.favorite_item_lists_response {
                Some(batch) => favourites_lists_from_batch_response(batch),
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        Ok(lists)
    }

    /// Get a favourites list by its associated shopping list ID
    ///
    /// # Arguments
    ///
    /// * `shopping_list_id` - The ID of the shopping list to find favourites for
    pub async fn get_favourites_for_list(&self, shopping_list_id: &str) -> Result<FavouritesList> {
        let lists = self.get_favourites_lists().await?;
        lists
            .into_iter()
            .find(|l| l.shopping_list_id.as_deref() == Some(shopping_list_id))
            .ok_or_else(|| {
                AnyListError::NotFound(format!(
                    "No favourites list for shopping list {}",
                    shopping_list_id
                ))
            })
    }

    /// Add a favourite item to the default favourites list
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the item to add as favourite
    /// * `category` - Optional category for the item
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password").await?;
    ///
    /// // Add a simple favourite
    /// client.add_favourite("Milk", None).await?;
    ///
    /// // Add a favourite with category
    /// client.add_favourite("Apples", Some("Produce")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_favourite(&self, name: &str, category: Option<&str>) -> Result<FavouriteItem> {
        // Get the first favourites list, or return an error if none exists
        let lists = self.get_favourites_lists().await?;
        let list = lists
            .first()
            .ok_or_else(|| AnyListError::NotFound("No favourites list found".to_string()))?;

        self.add_favourite_to_list(&list.id, name, category).await
    }

    /// Add a favourite item to a specific favourites list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the favourites list
    /// * `name` - The name of the item to add
    /// * `category` - Optional category for the item
    pub async fn add_favourite_to_list(
        &self,
        list_id: &str,
        name: &str,
        category: Option<&str>,
    ) -> Result<FavouriteItem> {
        let data = self.get_user_data().await?;
        let favourites = favourites_list(&data, list_id)?;
        let shopping_list_id = favourites.list_id.as_deref().unwrap_or_default();
        let list = data.list_context(shopping_list_id);
        let mut ids = self.ids();
        let item = operations::new_list_item(&mut ids, list, name, category);
        let (item, requests) = operations::add_favourite(&mut ids, list_id, item);
        self.send(requests).await?;
        Ok(transform_favourite_items(&[item], list_id).remove(0))
    }

    /// Remove a favourite item
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the favourites list containing the item
    /// * `item_id` - The ID of the favourite item to remove
    pub async fn remove_favourite(&self, list_id: &str, item_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let item = favourite_item(&data, list_id, item_id)?;
        let requests =
            operations::remove_starter_list_items(&mut self.ids(), list_id, vec![item.clone()]);
        self.send(requests).await
    }

    /// Add a favourite item to a shopping list
    ///
    /// Like the web client, this adds a copy of the favourite, with its
    /// quantity, note and category.
    ///
    /// # Arguments
    ///
    /// * `favourite` - The favourite item to add
    /// * `shopping_list_id` - The ID of the shopping list to add to
    pub async fn add_favourite_to_shopping_list(
        &self,
        favourite: &FavouriteItem,
        shopping_list_id: &str,
    ) -> Result<ListItem> {
        let data = self.get_user_data().await?;
        let item = favourite_item(&data, &favourite.list_id, &favourite.id)?;
        let (copies, requests) = operations::add_starter_list_items_to_list(
            &mut self.ids(),
            shopping_list_id,
            std::slice::from_ref(item),
        );
        self.send(requests).await?;
        transform_api_list_item(copies)
            .pop()
            .ok_or_else(|| AnyListError::InvalidResponse("Favourite has no name".to_string()))
    }
}

fn favourites_list<'a>(data: &'a PbUserDataResponse, list_id: &str) -> Result<&'a PbStarterList> {
    data.starter_lists_response
        .as_ref()
        .and_then(|r| r.favorite_item_lists_response.as_ref())
        .into_iter()
        .flat_map(|batch| &batch.list_responses)
        .filter_map(|r| r.starter_list.as_ref())
        .find(|l| l.identifier == list_id)
        .ok_or_else(|| AnyListError::NotFound(format!("Favourites list {} not found", list_id)))
}

fn favourite_item<'a>(
    data: &'a PbUserDataResponse,
    list_id: &str,
    item_id: &str,
) -> Result<&'a PbListItem> {
    favourites_list(data, list_id)?
        .items
        .iter()
        .find(|i| i.identifier == item_id)
        .ok_or_else(|| {
            AnyListError::NotFound(format!(
                "Favourite {} not found in list {}",
                item_id, list_id
            ))
        })
}

/// Convert a batch response to a list of FavouritesList
fn favourites_lists_from_batch_response(
    batch: &crate::protobuf::anylist::PbStarterListBatchResponse,
) -> Vec<FavouritesList> {
    batch
        .list_responses
        .iter()
        .filter_map(|response| {
            response.starter_list.as_ref().map(|list| FavouritesList {
                id: list.identifier.clone(),
                name: list.name.clone().unwrap_or_default(),
                items: transform_favourite_items(&list.items, &list.identifier),
                shopping_list_id: list.list_id.clone(),
            })
        })
        .collect()
}

/// Transform protobuf items to FavouriteItem structs
fn transform_favourite_items(items: &[PbListItem], list_id: &str) -> Vec<FavouriteItem> {
    items
        .iter()
        .filter_map(|item| {
            item.name.as_ref().map(|name| FavouriteItem {
                id: item.identifier.clone(),
                list_id: list_id.to_string(),
                name: name.clone(),
                quantity: item.quantity.clone().or_else(|| {
                    item.quantity_pb
                        .as_ref()
                        .and_then(|q| q.raw_quantity.clone())
                }),
                details: item.details.clone(),
                category: item.category.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_favourite_items_handles_empty() {
        let items: Vec<PbListItem> = vec![];
        let result = transform_favourite_items(&items, "test-list");
        assert!(result.is_empty());
    }

    #[test]
    fn test_transform_favourite_items_filters_nameless() {
        let items = vec![
            PbListItem {
                identifier: "id1".to_string(),
                name: Some("Milk".to_string()),
                ..Default::default()
            },
            PbListItem {
                identifier: "id2".to_string(),
                name: None, // Should be filtered out
                ..Default::default()
            },
        ];

        let result = transform_favourite_items(&items, "test-list");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Milk");
    }

    #[test]
    fn test_transform_favourite_items_preserves_fields() {
        let items = vec![PbListItem {
            identifier: "item-123".to_string(),
            name: Some("Organic Apples".to_string()),
            quantity: Some("2 lbs".to_string()),
            details: Some("Honeycrisp preferred".to_string()),
            category: Some("Produce".to_string()),
            ..Default::default()
        }];

        let result = transform_favourite_items(&items, "list-456");

        assert_eq!(result.len(), 1);
        let item = &result[0];
        assert_eq!(item.id, "item-123");
        assert_eq!(item.list_id, "list-456");
        assert_eq!(item.name, "Organic Apples");
        assert_eq!(item.quantity, Some("2 lbs".to_string()));
        assert_eq!(item.details, Some("Honeycrisp preferred".to_string()));
        assert_eq!(item.category, Some("Produce".to_string()));
    }
}
