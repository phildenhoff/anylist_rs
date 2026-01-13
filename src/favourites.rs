use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::lists::ListItem;
use crate::protobuf::anylist::PbListItem;
use crate::utils::generate_id;
use prost::Message;

/// Represents a favourite item that can be quickly added to shopping lists
#[derive(Debug, Clone)]
pub struct FavouriteItem {
    pub id: String,
    pub list_id: String,
    pub name: String,
    pub quantity: Option<String>,
    pub details: Option<String>,
    pub category: Option<String>,
}

/// Represents a favourites list (starter list)
#[derive(Debug, Clone)]
pub struct FavouritesList {
    pub id: String,
    pub name: String,
    pub items: Vec<FavouriteItem>,
    /// Associated shopping list ID (if linked to a specific list)
    pub shopping_list_id: Option<String>,
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
        let items: Vec<FavouriteItem> = lists
            .into_iter()
            .flat_map(|list| list.items)
            .collect();
        Ok(items)
    }

    /// Get all favourites lists
    ///
    /// Returns all starter lists of type `FavoriteItems`.
    pub async fn get_favourites_lists(&self) -> Result<Vec<FavouritesList>> {
        let data = self.get_user_data().await?;

        let lists = match data.starter_lists_response {
            Some(ref res) => {
                match &res.favorite_item_lists_response {
                    Some(batch) => favourites_lists_from_batch_response(batch),
                    None => Vec::new(),
                }
            }
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
        let item_id = generate_id();
        let operation_id = generate_id();

        let params = crate::operations::AddFavouriteParams {
            item_id: item_id.clone(),
            list_id: list_id.to_string(),
            operation_id,
            user_id: self.user_id(),
            name: name.to_string(),
            category: category.map(|c| c.to_string()),
        };

        let operation_list = crate::operations::build_add_favourite_operation(params);

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/starter-lists/update", buf).await?;

        Ok(FavouriteItem {
            id: item_id,
            list_id: list_id.to_string(),
            name: name.to_string(),
            quantity: None,
            details: None,
            category: category.map(|c| c.to_string()),
        })
    }

    /// Remove a favourite item
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the favourites list containing the item
    /// * `item_id` - The ID of the favourite item to remove
    pub async fn remove_favourite(&self, list_id: &str, item_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let params = crate::operations::RemoveFavouriteParams {
            item_id: item_id.to_string(),
            list_id: list_id.to_string(),
            operation_id,
            user_id: self.user_id(),
        };

        let operation_list = crate::operations::build_remove_favourite_operation(params);

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/starter-lists/update", buf).await?;
        Ok(())
    }

    /// Add a favourite item to a shopping list
    ///
    /// This is a convenience method that copies a favourite to a shopping list.
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
        self.add_item_with_details(
            shopping_list_id,
            &favourite.name,
            favourite.quantity.as_deref(),
            favourite.details.as_deref(),
            favourite.category.as_deref(),
        )
        .await
    }
}

/// Convert a batch response to a list of FavouritesList
fn favourites_lists_from_batch_response(
    batch: &crate::protobuf::anylist::PbStarterListBatchResponse,
) -> Vec<FavouritesList> {
    batch
        .list_responses
        .iter()
        .filter_map(|response| {
            response.starter_list.as_ref().map(|list| {
                FavouritesList {
                    id: list.identifier.clone(),
                    name: list.name.clone().unwrap_or_default(),
                    items: transform_favourite_items(&list.items, &list.identifier),
                    shopping_list_id: list.list_id.clone(),
                }
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
                quantity: item.quantity.clone(),
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
