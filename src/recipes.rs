use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbIngredient, PbOperationMetadata, PbRecipe,
    PbRecipeDataResponse, PbRecipeOperation, PbRecipeOperationList,
};
use prost::Message;
use crate::utils::{current_timestamp, generate_id};

#[derive(Debug, Clone)]
pub struct Ingredient {
    pub name: String,
    pub quantity: Option<String>,
    pub note: Option<String>,
    pub raw_ingredient: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub preparation_steps: Vec<String>,
    pub note: Option<String>,
    pub source_name: Option<String>,
    pub source_url: Option<String>,
    pub servings: Option<String>,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub rating: Option<i32>,
    pub photo_urls: Vec<String>,
}

impl AnyListClient {
    /// Get all recipes for the authenticated user
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::new("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let recipes = client.get_recipes().await.expect("Failed to get recipes");
    /// for recipe in recipes {
    ///     println!("Recipe: {}", recipe.name);
    /// }
    /// # }
    /// ```
    pub async fn get_recipes(&self) -> Result<Vec<Recipe>> {
        let data = self.get_user_data().await?;
        let recipes = match data.recipe_data_response {
            Some(ref res) => recipes_from_response(res.clone()),
            None => Vec::new(),
        };
        Ok(recipes)
    }

    /// Get a specific recipe by ID
    ///
    /// # Arguments
    ///
    /// * `recipe_id` - The ID of the recipe to retrieve
    pub async fn get_recipe_by_id(&self, recipe_id: &str) -> Result<Recipe> {
        let recipes = self.get_recipes().await?;
        recipes
            .into_iter()
            .find(|r| r.id == recipe_id)
            .ok_or_else(|| {
                AnyListError::NotFound(format!("Recipe with ID {} not found", recipe_id))
            })
    }

    /// Get a specific recipe by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the recipe to retrieve
    pub async fn get_recipe_by_name(&self, name: &str) -> Result<Recipe> {
        let recipes = self.get_recipes().await?;
        recipes
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| AnyListError::NotFound(format!("Recipe with name '{}' not found", name)))
    }

    /// Create a new recipe
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the recipe
    /// * `ingredients` - List of ingredients
    /// * `preparation_steps` - List of preparation steps
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::{AnyListClient, recipes::Ingredient};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::new("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let ingredients = vec![
    ///     Ingredient {
    ///         name: "Flour".to_string(),
    ///         quantity: Some("2 cups".to_string()),
    ///         note: None,
    ///         raw_ingredient: None,
    ///     },
    /// ];
    ///
    /// let steps = vec!["Mix ingredients".to_string(), "Bake for 30 minutes".to_string()];
    ///
    /// let recipe = client.create_recipe("Bread", ingredients, steps)
    ///     .await
    ///     .expect("Failed to create recipe");
    /// # }
    /// ```
    pub async fn create_recipe(
        &self,
        name: &str,
        ingredients: Vec<Ingredient>,
        preparation_steps: Vec<String>,
    ) -> Result<Recipe> {
        let recipe_id = generate_id();
        let operation_id = generate_id();

        let pb_ingredients: Vec<PbIngredient> = ingredients
            .iter()
            .map(|i| PbIngredient {
                raw_ingredient: i.raw_ingredient.clone(),
                name: Some(i.name.clone()),
                quantity: i.quantity.clone(),
                note: i.note.clone(),
            })
            .collect();

        let new_recipe = PbRecipe {
            identifier: recipe_id.clone(),
            timestamp: Some(current_timestamp()),
            name: Some(name.to_string()),
            icon: None,
            note: None,
            source_name: None,
            source_url: None,
            ingredients: pb_ingredients,
            preparation_steps: preparation_steps.clone(),
            photo_ids: vec![],
            ad_campaign_id: None,
            photo_urls: vec![],
            scale_factor: Some(1.0),
            rating: None,
            creation_timestamp: Some(current_timestamp()),
            nutritional_info: None,
            cook_time: None,
            prep_time: None,
            servings: None,
            paprika_identifier: None,
        };

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("save-recipe".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: Some(new_recipe),
            recipe_collection: None,
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

        Ok(Recipe {
            id: recipe_id,
            name: name.to_string(),
            ingredients,
            preparation_steps,
            note: None,
            source_name: None,
            source_url: None,
            servings: None,
            prep_time: None,
            cook_time: None,
            rating: None,
            photo_urls: vec![],
        })
    }

    /// Update an existing recipe
    ///
    /// # Arguments
    ///
    /// * `recipe_id` - The ID of the recipe to update
    /// * `name` - The new name
    /// * `ingredients` - The new ingredients list
    /// * `preparation_steps` - The new preparation steps
    pub async fn update_recipe(
        &self,
        recipe_id: &str,
        name: &str,
        ingredients: Vec<Ingredient>,
        preparation_steps: Vec<String>,
    ) -> Result<()> {
        let operation_id = generate_id();

        let pb_ingredients: Vec<PbIngredient> = ingredients
            .iter()
            .map(|i| PbIngredient {
                raw_ingredient: i.raw_ingredient.clone(),
                name: Some(i.name.clone()),
                quantity: i.quantity.clone(),
                note: i.note.clone(),
            })
            .collect();

        let updated_recipe = PbRecipe {
            identifier: recipe_id.to_string(),
            timestamp: Some(current_timestamp()),
            name: Some(name.to_string()),
            icon: None,
            note: None,
            source_name: None,
            source_url: None,
            ingredients: pb_ingredients,
            preparation_steps,
            photo_ids: vec![],
            ad_campaign_id: None,
            photo_urls: vec![],
            scale_factor: Some(1.0),
            rating: None,
            creation_timestamp: Some(current_timestamp()),
            nutritional_info: None,
            cook_time: None,
            prep_time: None,
            servings: None,
            paprika_identifier: None,
        };

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("save-recipe".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: Some(updated_recipe),
            recipe_collection: None,
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

    /// Delete a recipe
    ///
    /// # Arguments
    ///
    /// * `recipe_id` - The ID of the recipe to delete
    pub async fn delete_recipe(&self, recipe_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("remove-recipe".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            recipe_data_id: None,
            recipe: None,
            recipe_collection: None,
            recipe_link_request: None,
            recipe_collection_ids: vec![],
            recipes: vec![],
            is_new_recipe_from_web_import: Some(false),
            recipe_ids: vec![recipe_id.to_string()],
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

    /// Add recipe ingredients to a shopping list
    ///
    /// # Arguments
    ///
    /// * `recipe_id` - The ID of the recipe
    /// * `list_id` - The ID of the list to add ingredients to
    /// * `scale_factor` - Optional scale factor for recipe (e.g., 2.0 to double the recipe)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::new("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// // Add recipe ingredients to list, scaled 2x
    /// client.add_recipe_to_list("recipe-id", "list-id", Some(2.0))
    ///     .await
    ///     .expect("Failed to add recipe to list");
    /// # }
    /// ```
    pub async fn add_recipe_to_list(
        &self,
        recipe_id: &str,
        list_id: &str,
        scale_factor: Option<f64>,
    ) -> Result<()> {
        let recipe = self.get_recipe_by_id(recipe_id).await?;

        for ingredient in recipe.ingredients {
            let quantity = if let (Some(qty), Some(scale)) = (&ingredient.quantity, scale_factor) {
                Some(scale_quantity(qty, scale))
            } else {
                ingredient.quantity.clone()
            };

            self.add_item_with_details(
                list_id,
                &ingredient.name,
                quantity.as_deref(),
                ingredient.note.as_deref(),
                None,
            )
            .await?;
        }

        Ok(())
    }
}

fn recipes_from_response(response: PbRecipeDataResponse) -> Vec<Recipe> {
    let mut recipes: Vec<Recipe> = Vec::new();
    for recipe in response.recipes {
        if let Some(name) = recipe.name {
            let ingredients: Vec<Ingredient> = recipe
                .ingredients
                .iter()
                .filter_map(|i| {
                    i.name.as_ref().map(|name| Ingredient {
                        name: name.clone(),
                        quantity: i.quantity.clone(),
                        note: i.note.clone(),
                        raw_ingredient: i.raw_ingredient.clone(),
                    })
                })
                .collect();

            let recipe = Recipe {
                id: recipe.identifier,
                name,
                ingredients,
                preparation_steps: recipe.preparation_steps,
                note: recipe.note,
                source_name: recipe.source_name,
                source_url: recipe.source_url,
                servings: recipe.servings,
                prep_time: recipe.prep_time,
                cook_time: recipe.cook_time,
                rating: recipe.rating,
                photo_urls: recipe.photo_urls,
            };
            recipes.push(recipe);
        }
    }
    recipes
}


/// Simple quantity scaling - attempts to parse and scale numeric quantities
fn scale_quantity(quantity: &str, scale: f64) -> String {
    // Try to parse the first number in the quantity string
    let parts: Vec<&str> = quantity.split_whitespace().collect();
    if parts.is_empty() {
        return quantity.to_string();
    }

    // Try to parse the first part as a number
    if let Ok(num) = parts[0].parse::<f64>() {
        let scaled = num * scale;
        let rest = parts[1..].join(" ");
        if rest.is_empty() {
            format!("{}", scaled)
        } else {
            format!("{} {}", scaled, rest)
        }
    } else {
        quantity.to_string()
    }
}
