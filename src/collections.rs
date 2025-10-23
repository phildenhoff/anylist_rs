use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbOperationMetadata, PbRecipeCollection,
    PbRecipeCollectionSettings, PbRecipeOperation, PbRecipeOperationList,
};
use prost::Message;
use crate::utils::{current_timestamp, generate_id};

#[derive(Debug, Clone)]
pub struct RecipeCollection {
    pub id: String,
    pub name: String,
    pub recipe_ids: Vec<String>,
}

impl AnyListClient {
    pub async fn get_recipe_collections(&self) -> Result<Vec<RecipeCollection>> {
        let data = self.get_user_data().await?;
        let collections = match data.recipe_data_response {
            Some(ref res) => res
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
        };
        Ok(collections)
    }

    /// Create a new recipe collection
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection
    pub async fn create_recipe_collection(&self, name: &str) -> Result<RecipeCollection> {
        let collection_id = generate_id();
        let operation_id = generate_id();

        let new_collection = PbRecipeCollection {
            identifier: collection_id.clone(),
            timestamp: Some(current_timestamp()),
            name: Some(name.to_string()),
            recipe_ids: vec![],
            collection_settings: Some(PbRecipeCollectionSettings {
                recipes_sort_order: Some(0), // Manual
                show_only_recipes_with_no_collection: Some(false),
            }),
        };

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("new-recipe-collection".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: None,
            recipe_collection: Some(new_collection),
            recipe_link_request: None,
            recipe_collection_ids: vec![],
            recipes: vec![],
            is_new_recipe_from_web_import: Some(false),
            recipe_ids: vec![],
        };

        let operation_list = PbRecipeOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/user-recipe-data/update", buf).await?;

        Ok(RecipeCollection {
            id: collection_id,
            name: name.to_string(),
            recipe_ids: vec![],
        })
    }

    /// Delete a recipe collection
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection to delete
    pub async fn delete_recipe_collection(&self, collection_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("remove-recipe-collection".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: None,
            recipe_collection: None,
            recipe_link_request: None,
            recipe_collection_ids: vec![collection_id.to_string()],
            recipes: vec![],
            is_new_recipe_from_web_import: Some(false),
            recipe_ids: vec![],
        };

        let operation_list = PbRecipeOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/user-recipe-data/update", buf).await?;
        Ok(())
    }

    /// Add a recipe to a collection
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
        let operation_id = generate_id();

        // Get the current collection
        let collections = self.get_recipe_collections().await?;
        let collection = collections
            .iter()
            .find(|c| c.id == collection_id)
            .ok_or_else(|| {
                AnyListError::NotFound(format!("Collection with ID {} not found", collection_id))
            })?;

        let mut new_recipe_ids = collection.recipe_ids.clone();
        if !new_recipe_ids.contains(&recipe_id.to_string()) {
            new_recipe_ids.push(recipe_id.to_string());
        }

        let updated_collection = PbRecipeCollection {
            identifier: collection_id.to_string(),
            timestamp: Some(current_timestamp()),
            name: Some(collection.name.clone()),
            recipe_ids: new_recipe_ids,
            collection_settings: Some(PbRecipeCollectionSettings {
                recipes_sort_order: Some(0),
                show_only_recipes_with_no_collection: Some(false),
            }),
        };

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("add-recipes-to-collection".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: None,
            recipe_collection: Some(updated_collection),
            recipe_link_request: None,
            recipe_collection_ids: vec![],
            recipes: vec![],
            is_new_recipe_from_web_import: Some(false),
            recipe_ids: vec![],
        };

        let operation_list = PbRecipeOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/user-recipe-data/update", buf).await?;
        Ok(())
    }

    /// Remove a recipe from a collection
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
        let operation_id = generate_id();

        // Get the current collection
        let collections = self.get_recipe_collections().await?;
        let collection = collections
            .iter()
            .find(|c| c.id == collection_id)
            .ok_or_else(|| {
                AnyListError::NotFound(format!("Collection with ID {} not found", collection_id))
            })?;

        let new_recipe_ids: Vec<String> = collection
            .recipe_ids
            .iter()
            .filter(|id| *id != recipe_id)
            .cloned()
            .collect();

        let updated_collection = PbRecipeCollection {
            identifier: collection_id.to_string(),
            timestamp: Some(current_timestamp()),
            name: Some(collection.name.clone()),
            recipe_ids: new_recipe_ids,
            collection_settings: Some(PbRecipeCollectionSettings {
                recipes_sort_order: Some(0),
                show_only_recipes_with_no_collection: Some(false),
            }),
        };

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("remove-recipes-from-collection".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: None,
            recipe_collection: Some(updated_collection),
            recipe_link_request: None,
            recipe_collection_ids: vec![],
            recipes: vec![],
            is_new_recipe_from_web_import: Some(false),
            recipe_ids: vec![],
        };

        let operation_list = PbRecipeOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/user-recipe-data/update", buf).await?;
        Ok(())
    }
}
