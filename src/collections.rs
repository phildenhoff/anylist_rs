use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    PbOperationMetadata, PbRecipeCollection, PbRecipeDataResponse, PbRecipeOperation,
    PbRecipeOperationList,
};
use crate::utils::generate_id;
use prost::Message;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeCollection {
    id: String,
    name: String,
    recipe_ids: Vec<String>,
}

impl RecipeCollection {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn recipe_ids(&self) -> &[String] {
        &self.recipe_ids
    }
}

/// Identifiers every recipe collection operation carries.
struct OperationIds {
    operation_id: String,
    user_id: String,
    /// The account's recipe data ID, from the user data response.
    recipe_data_id: Option<String>,
}

impl AnyListClient {
    async fn get_recipe_data(&self) -> Result<Option<PbRecipeDataResponse>> {
        Ok(self.get_user_data().await?.recipe_data_response)
    }

    fn operation_ids(&self, recipe_data: Option<&PbRecipeDataResponse>) -> OperationIds {
        OperationIds {
            operation_id: generate_id(),
            user_id: self.user_id(),
            recipe_data_id: recipe_data.and_then(|response| response.recipe_data_id.clone()),
        }
    }

    async fn send_recipe_operation(&self, operation_list: PbRecipeOperationList) -> Result<()> {
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;
        self.post("data/user-recipe-data/update", buf).await?;
        Ok(())
    }

    pub async fn get_recipe_collections(&self) -> Result<Vec<RecipeCollection>> {
        let data = self.get_recipe_data().await?;
        Ok(collections_from(data.as_ref()))
    }

    /// Create a new recipe collection
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection
    pub async fn create_recipe_collection(&self, name: &str) -> Result<RecipeCollection> {
        self.create_recipe_collection_with_recipe_ids(name, Vec::new())
            .await
    }

    /// Create a new recipe collection pre-populated with recipe IDs
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection
    /// * `recipe_ids` - Initial recipe IDs to include in the collection
    pub async fn create_recipe_collection_with_recipe_ids(
        &self,
        name: &str,
        recipe_ids: Vec<String>,
    ) -> Result<RecipeCollection> {
        let recipe_data = self.get_recipe_data().await?;
        let ids = self.operation_ids(recipe_data.as_ref());

        let collection = PbRecipeCollection {
            identifier: generate_id(),
            name: Some(name.to_string()),
            recipe_ids: recipe_ids.clone(),
            ..Default::default()
        };
        let created = RecipeCollection {
            id: collection.identifier.clone(),
            name: name.to_string(),
            recipe_ids,
        };

        self.send_recipe_operation(new_collection_operation(ids, collection))
            .await?;
        Ok(created)
    }

    /// Delete a recipe collection
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection to delete
    pub async fn delete_recipe_collection(&self, collection_id: &str) -> Result<()> {
        let recipe_data = self.get_recipe_data().await?;
        let ids = self.operation_ids(recipe_data.as_ref());
        let collection = find_collection(recipe_data.as_ref(), collection_id)?;

        self.send_recipe_operation(remove_collection_operation(ids, collection))
            .await
    }

    /// Add a recipe to a collection
    ///
    /// Sends nothing when the recipe is already in the collection.
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection
    /// * `recipe_id` - The ID of the recipe to add
    pub async fn add_recipe_to_collection(
        &self,
        collection_id: &str,
        recipe_id: &str,
    ) -> Result<()> {
        // One fetch gives both the recipe data ID and the current collection
        let recipe_data = self.get_recipe_data().await?;
        let ids = self.operation_ids(recipe_data.as_ref());
        let collection = find_collection(recipe_data.as_ref(), collection_id)?;

        if collection.recipe_ids.iter().any(|id| id == recipe_id) {
            return Ok(());
        }
        self.send_recipe_operation(add_recipes_operation(ids, collection, recipe_id))
            .await
    }

    /// Remove a recipe from a collection
    ///
    /// Sends nothing when the recipe is not in the collection.
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection
    /// * `recipe_id` - The ID of the recipe to remove
    pub async fn remove_recipe_from_collection(
        &self,
        collection_id: &str,
        recipe_id: &str,
    ) -> Result<()> {
        // One fetch gives both the recipe data ID and the current collection
        let recipe_data = self.get_recipe_data().await?;
        let ids = self.operation_ids(recipe_data.as_ref());
        let collection = find_collection(recipe_data.as_ref(), collection_id)?;

        if !collection.recipe_ids.iter().any(|id| id == recipe_id) {
            return Ok(());
        }
        self.send_recipe_operation(remove_recipes_operation(ids, collection, recipe_id))
            .await
    }
}

// The operation shapes below match what the AnyList web client sends, captured
// on 2026-10-25 (see the `webapp_*` snapshots in the tests). Each operation
// carries only metadata, the recipe data ID, and a recipe collection.

/// `new-recipe-collection`: the collection to create. The server honours
/// `recipe_ids` set on it, so a collection can start populated.
fn new_collection_operation(
    ids: OperationIds,
    collection: PbRecipeCollection,
) -> PbRecipeOperationList {
    collection_operation("new-recipe-collection", ids, collection)
}

/// `remove-recipe-collection`: the stored collection, sent as is.
fn remove_collection_operation(
    ids: OperationIds,
    collection: PbRecipeCollection,
) -> PbRecipeOperationList {
    collection_operation("remove-recipe-collection", ids, collection)
}

/// `add-recipes-to-collection`: the stored collection with `recipe_ids`
/// narrowed to just the recipes being added.
fn add_recipes_operation(
    ids: OperationIds,
    collection: PbRecipeCollection,
    recipe_id: &str,
) -> PbRecipeOperationList {
    collection_operation(
        "add-recipes-to-collection",
        ids,
        with_only(collection, recipe_id),
    )
}

/// `remove-recipes-from-collection`: the stored collection with `recipe_ids`
/// narrowed to just the recipes being removed. Sending the recipes that
/// should remain instead makes the server remove those.
fn remove_recipes_operation(
    ids: OperationIds,
    collection: PbRecipeCollection,
    recipe_id: &str,
) -> PbRecipeOperationList {
    collection_operation(
        "remove-recipes-from-collection",
        ids,
        with_only(collection, recipe_id),
    )
}

fn with_only(collection: PbRecipeCollection, recipe_id: &str) -> PbRecipeCollection {
    PbRecipeCollection {
        recipe_ids: vec![recipe_id.to_string()],
        ..collection
    }
}

fn collection_operation(
    handler_id: &str,
    ids: OperationIds,
    collection: PbRecipeCollection,
) -> PbRecipeOperationList {
    PbRecipeOperationList {
        operations: vec![PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(ids.operation_id),
                handler_id: Some(handler_id.to_string()),
                user_id: Some(ids.user_id),
                operation_class: None,
            }),
            recipe_data_id: ids.recipe_data_id,
            recipe_collection: Some(collection),
            ..Default::default()
        }],
    }
}

fn collections_from(data: Option<&PbRecipeDataResponse>) -> Vec<RecipeCollection> {
    match data {
        Some(res) => res
            .recipe_collections
            .iter()
            .filter_map(|c| {
                c.name.as_ref().map(|name| RecipeCollection {
                    id: c.identifier.clone(),
                    name: name.clone(),
                    recipe_ids: c.recipe_ids.clone(),
                })
            })
            .collect(),
        None => Vec::new(),
    }
}

fn find_collection(
    data: Option<&PbRecipeDataResponse>,
    collection_id: &str,
) -> Result<PbRecipeCollection> {
    data.and_then(|res| {
        res.recipe_collections
            .iter()
            .find(|c| c.identifier == collection_id)
            .cloned()
    })
    .ok_or_else(|| {
        AnyListError::NotFound(format!("Collection with ID {} not found", collection_id))
    })
}

#[cfg(test)]
mod tests {
    //! Byte-exact comparisons against requests the AnyList web client sent on
    //! 2026-10-25, captured from the official client with its own IDs.
    use super::*;

    const USER_ID: &str = "cda21b0078644a01b640c84d3d74187e";
    const RECIPE_DATA_ID: &str = "404f4849109b465e8e641b32762434eb";
    const COLLECTION_ID: &str = "c61f7b81ba01495eaf99f08c8547d67c";
    const COLLECTION_NAME: &str = "anylist_rs capture 2 2026-10-25";
    const RECIPE_ID: &str = "916cbbf27354447c806725c710b5a20f";

    fn ids(operation_id: &str) -> OperationIds {
        OperationIds {
            operation_id: operation_id.to_string(),
            user_id: USER_ID.to_string(),
            recipe_data_id: Some(RECIPE_DATA_ID.to_string()),
        }
    }

    /// The collection as stored on the server at capture time.
    fn stored_collection() -> PbRecipeCollection {
        PbRecipeCollection {
            identifier: COLLECTION_ID.to_string(),
            name: Some(COLLECTION_NAME.to_string()),
            ..Default::default()
        }
    }

    fn hex_of(operation_list: PbRecipeOperationList) -> String {
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();
        hex::encode(&buf)
    }

    #[test]
    fn webapp_new_recipe_collection_2026_10_25() {
        let operation_list =
            new_collection_operation(ids("b1d16a806e7c4addb8e81163cf7bfbcb"), stored_collection());
        insta::assert_snapshot!(hex_of(operation_list));
    }

    #[test]
    fn webapp_add_recipes_to_collection_2026_10_25() {
        let operation_list = add_recipes_operation(
            ids("35c10cf164494a7b9bf34f1297db02b1"),
            stored_collection(),
            RECIPE_ID,
        );
        insta::assert_snapshot!(hex_of(operation_list));
    }

    #[test]
    fn webapp_remove_recipes_from_collection_2026_10_25() {
        // The web client sends only the recipe being removed, however many
        // other recipes the collection holds.
        let stored = PbRecipeCollection {
            recipe_ids: vec![RECIPE_ID.to_string(), "other".to_string()],
            ..stored_collection()
        };
        let operation_list =
            remove_recipes_operation(ids("16eebbbc62f0450ea702f7faf1222a87"), stored, RECIPE_ID);
        insta::assert_snapshot!(hex_of(operation_list));
    }

    #[test]
    fn webapp_remove_recipe_collection_2026_10_25() {
        let stored = PbRecipeCollection {
            identifier: "dfa5021db26349a1876a295990c819f7".to_string(),
            name: Some("anylist_rs capture 2026-10-25".to_string()),
            ..Default::default()
        };
        let operation_list =
            remove_collection_operation(ids("8b20e907ca914eb7a37035118c9542e7"), stored);
        insta::assert_snapshot!(hex_of(operation_list));
    }
}
