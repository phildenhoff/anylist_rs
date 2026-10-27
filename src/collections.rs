use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::operations;
use crate::protobuf::anylist::{PbRecipeCollection, PbRecipeDataResponse, PbUserDataResponse};
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

impl AnyListClient {
    pub async fn get_recipe_collections(&self) -> Result<Vec<RecipeCollection>> {
        let data = self.get_user_data().await?;
        Ok(collections_from(data.recipe_data_response.as_ref()))
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
        let data = self.get_user_data().await?;
        let recipe_data = recipe_data(&data)?;
        let recipe_data_id = recipe_data_id(recipe_data);
        check_recipes_exist(recipe_data, &recipe_ids)?;
        let mut ids = self.ids();
        let created = operations::create_collection(&mut ids, recipe_data_id, name);
        let mut requests = created.requests;
        let recipe_ids = new_recipe_ids(&created.collection, &recipe_ids);
        if !recipe_ids.is_empty() {
            requests.extend(operations::add_recipes_to_collection(
                &mut ids,
                recipe_data_id,
                &created.collection,
                recipe_ids.clone(),
            ));
        }
        self.send(requests).await?;
        Ok(RecipeCollection {
            id: created.collection.identifier,
            name: name.to_string(),
            recipe_ids,
        })
    }

    /// Delete a recipe collection
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection to delete
    pub async fn delete_recipe_collection(&self, collection_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let recipe_data = recipe_data(&data)?;
        let collection = find_collection(recipe_data, collection_id)?;
        let requests =
            operations::delete_collection(&mut self.ids(), recipe_data_id(recipe_data), collection);
        self.send(requests).await
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
        let data = self.get_user_data().await?;
        let recipe_data = recipe_data(&data)?;
        let collection = find_collection(recipe_data, collection_id)?;
        check_recipes_exist(recipe_data, &[recipe_id.to_string()])?;
        if collection.recipe_ids.iter().any(|id| id == recipe_id) {
            return Ok(());
        }
        let recipe_ids = vec![recipe_id.to_string()];
        let requests = operations::add_recipes_to_collection(
            &mut self.ids(),
            recipe_data_id(recipe_data),
            collection,
            recipe_ids,
        );
        self.send(requests).await
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
        let data = self.get_user_data().await?;
        let recipe_data = recipe_data(&data)?;
        let collection = find_collection(recipe_data, collection_id)?;
        if !collection.recipe_ids.iter().any(|id| id == recipe_id) {
            return Ok(());
        }
        let requests = operations::remove_recipe_from_collection(
            &mut self.ids(),
            recipe_data_id(recipe_data),
            collection,
            recipe_id,
        );
        self.send(requests).await
    }
}

fn recipe_data(data: &PbUserDataResponse) -> Result<&PbRecipeDataResponse> {
    data.recipe_data_response
        .as_ref()
        .ok_or_else(|| AnyListError::InvalidResponse("No recipe data".to_string()))
}

fn recipe_data_id(recipe_data: &PbRecipeDataResponse) -> &str {
    recipe_data.recipe_data_id.as_deref().unwrap_or_default()
}

/// The web client only adds recipes it has; the crate says when one is
/// missing.
fn check_recipes_exist(recipe_data: &PbRecipeDataResponse, recipe_ids: &[String]) -> Result<()> {
    match recipe_ids
        .iter()
        .find(|id| !recipe_data.recipes.iter().any(|r| r.identifier == **id))
    {
        Some(missing) => Err(AnyListError::NotFound(format!(
            "Recipe with ID {} not found",
            missing
        ))),
        None => Ok(()),
    }
}

/// The recipes the web client adds to a collection: ones not in it yet.
fn new_recipe_ids(collection: &PbRecipeCollection, recipe_ids: &[String]) -> Vec<String> {
    let mut added: Vec<String> = Vec::new();
    for recipe_id in recipe_ids {
        if !collection.recipe_ids.contains(recipe_id) && !added.contains(recipe_id) {
            added.push(recipe_id.clone());
        }
    }
    added
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

fn find_collection<'a>(
    recipe_data: &'a PbRecipeDataResponse,
    collection_id: &str,
) -> Result<&'a PbRecipeCollection> {
    recipe_data
        .recipe_collections
        .iter()
        .find(|c| c.identifier == collection_id)
        .ok_or_else(|| {
            AnyListError::NotFound(format!("Collection with ID {} not found", collection_id))
        })
}

#[cfg(test)]
mod tests {
    //! Byte-exact comparisons against requests the AnyList web client sent on
    //! 2026-10-25, captured from the official client with its own IDs.
    use crate::operations::{self, Ids};
    use crate::protobuf::anylist::PbRecipeCollection;

    const USER_ID: &str = "cda21b0078644a01b640c84d3d74187e";
    const RECIPE_DATA_ID: &str = "404f4849109b465e8e641b32762434eb";
    const COLLECTION_ID: &str = "c61f7b81ba01495eaf99f08c8547d67c";
    const COLLECTION_NAME: &str = "anylist_rs capture 2 2026-10-25";
    const RECIPE_ID: &str = "916cbbf27354447c806725c710b5a20f";

    fn ids(replayed: &[&str]) -> Ids {
        Ids::replay(USER_ID, replayed.iter().copied())
    }

    fn stored_collection() -> PbRecipeCollection {
        PbRecipeCollection {
            identifier: COLLECTION_ID.to_string(),
            name: Some(COLLECTION_NAME.to_string()),
            ..Default::default()
        }
    }

    fn hex_of(requests: &[operations::Request]) -> String {
        hex::encode(&requests[0].body)
    }

    #[test]
    fn webapp_new_recipe_collection_2026_10_25() {
        let created = operations::create_collection(
            &mut ids(&[COLLECTION_ID, "b1d16a806e7c4addb8e81163cf7bfbcb", "icon"]),
            RECIPE_DATA_ID,
            COLLECTION_NAME,
        );
        insta::assert_snapshot!(hex_of(&created.requests));
    }

    #[test]
    fn webapp_add_recipes_to_collection_2026_10_25() {
        let requests = operations::add_recipes_to_collection(
            &mut ids(&["35c10cf164494a7b9bf34f1297db02b1"]),
            RECIPE_DATA_ID,
            &stored_collection(),
            vec![RECIPE_ID.to_string()],
        );
        insta::assert_snapshot!(hex_of(&requests));
    }

    #[test]
    fn webapp_remove_recipes_from_collection_2026_10_25() {
        // The web client sends only the recipe being removed, however many
        // other recipes the collection holds.
        let stored = PbRecipeCollection {
            recipe_ids: vec![RECIPE_ID.to_string(), "other".to_string()],
            ..stored_collection()
        };
        let requests = operations::remove_recipe_from_collection(
            &mut ids(&["16eebbbc62f0450ea702f7faf1222a87"]),
            RECIPE_DATA_ID,
            &stored,
            RECIPE_ID,
        );
        insta::assert_snapshot!(hex_of(&requests));
    }

    #[test]
    fn webapp_remove_recipe_collection_2026_10_25() {
        let stored = PbRecipeCollection {
            identifier: "dfa5021db26349a1876a295990c819f7".to_string(),
            name: Some("anylist_rs capture 2026-10-25".to_string()),
            ..Default::default()
        };
        let requests = operations::delete_collection(
            &mut ids(&["8b20e907ca914eb7a37035118c9542e7"]),
            RECIPE_DATA_ID,
            &stored,
        );
        insta::assert_snapshot!(hex_of(&requests));
    }
}
