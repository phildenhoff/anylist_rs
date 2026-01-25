use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbIngredient, PbOperationMetadata, PbRecipe,
    PbRecipeDataResponse, PbRecipeOperation, PbRecipeOperationList,
};
use crate::utils::{current_timestamp, generate_id};
use prost::Message;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ingredient {
    pub(crate) name: String,
    pub(crate) quantity: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) raw_ingredient: Option<String>,
}

impl Ingredient {
    /// Create a new ingredient with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            quantity: None,
            note: None,
            raw_ingredient: None,
        }
    }

    pub fn quantity_of(mut self, quantity: impl Into<String>) -> Self {
        self.quantity = Some(quantity.into());
        self
    }

    pub fn note_of(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn raw_ingredient_of(mut self, raw: impl Into<String>) -> Self {
        self.raw_ingredient = Some(raw.into());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn quantity(&self) -> Option<&str> {
        self.quantity.as_deref()
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn raw_ingredient(&self) -> Option<&str> {
        self.raw_ingredient.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    id: String,
    name: String,
    ingredients: Vec<Ingredient>,
    preparation_steps: Vec<String>,
    note: Option<String>,
    source_name: Option<String>,
    source_url: Option<String>,
    servings: Option<String>,
    prep_time: Option<i32>,
    cook_time: Option<i32>,
    rating: Option<i32>,
    nutritional_info: Option<String>,
    photo_id: Option<String>,
    photo_urls: Vec<String>,
}

impl Recipe {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ingredients(&self) -> &[Ingredient] {
        &self.ingredients
    }

    pub fn preparation_steps(&self) -> &[String] {
        &self.preparation_steps
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    pub fn servings(&self) -> Option<&str> {
        self.servings.as_deref()
    }

    pub fn prep_time(&self) -> Option<i32> {
        self.prep_time
    }

    pub fn cook_time(&self) -> Option<i32> {
        self.cook_time
    }

    pub fn rating(&self) -> Option<i32> {
        self.rating
    }

    pub fn nutritional_info(&self) -> Option<&str> {
        self.nutritional_info.as_deref()
    }

    pub fn photo_id(&self) -> Option<&str> {
        self.photo_id.as_deref()
    }

    pub fn photo_urls(&self) -> &[String] {
        &self.photo_urls
    }
}

/// Builder for creating or updating recipes with all available fields.
///
/// Use `RecipeBuilder::new(name)` to create a new recipe, or
/// `RecipeBuilder::from(&existing_recipe)` to update an existing one.
///
/// # Example
///
/// ```no_run
/// # use anylist_rs::{AnyListClient, Ingredient, RecipeBuilder};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = AnyListClient::login("user@example.com", "password").await?;
/// // Create a new recipe
/// let recipe = RecipeBuilder::new("Pasta Carbonara")
///     .add_ingredient(Ingredient::new("Spaghetti").quantity_of("400g"))
///     .add_ingredient(Ingredient::new("Eggs").quantity_of("4"))
///     .add_step("Boil pasta in salted water")
///     .add_step("Mix eggs with cheese")
///     .prep_time(10)
///     .cook_time(20)
///     .servings("4")
///     .rating(5)
///     .save(&client)
///     .await?;
///
/// // Update an existing recipe
/// let updated = RecipeBuilder::from(&recipe)
///     .note("Family favorite!")
///     .save(&client)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RecipeBuilder {
    /// Recipe ID - None for new recipes, Some for updates
    id: Option<String>,
    /// Recipe name (required)
    name: String,
    /// List of ingredients
    ingredients: Vec<Ingredient>,
    /// Preparation steps (each step can be multiline, supports markdown headers/bold/italic)
    preparation_steps: Vec<String>,
    /// Recipe notes/description
    note: Option<String>,
    /// Source name (e.g., "Web", "Cookbook")
    source_name: Option<String>,
    /// Source URL
    source_url: Option<String>,
    /// Serving size (e.g., "4 servings")
    servings: Option<String>,
    /// Prep time in minutes
    prep_time: Option<i32>,
    /// Cook time in minutes
    cook_time: Option<i32>,
    /// Rating from 1-5
    rating: Option<i32>,
    /// Nutritional information (freeform text, can be multiline)
    nutritional_info: Option<String>,
    /// Cover photo ID (from upload_photo)
    photo_id: Option<String>,
}

impl RecipeBuilder {
    /// Create a new recipe builder for a new recipe
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            ingredients: Vec::new(),
            preparation_steps: Vec::new(),
            note: None,
            source_name: None,
            source_url: None,
            servings: None,
            prep_time: None,
            cook_time: None,
            rating: None,
            nutritional_info: None,
            photo_id: None,
        }
    }

    /// Create a builder from an existing recipe for updates
    pub fn from(recipe: &Recipe) -> Self {
        Self {
            id: Some(recipe.id.clone()),
            name: recipe.name.clone(),
            ingredients: recipe.ingredients.clone(),
            preparation_steps: recipe.preparation_steps.clone(),
            note: recipe.note.clone(),
            source_name: recipe.source_name.clone(),
            source_url: recipe.source_url.clone(),
            servings: recipe.servings.clone(),
            prep_time: recipe.prep_time,
            cook_time: recipe.cook_time,
            rating: recipe.rating,
            nutritional_info: recipe.nutritional_info.clone(),
            photo_id: recipe.photo_id.clone(),
        }
    }

    /// Set all ingredients at once (replaces any existing)
    pub fn ingredients(mut self, ingredients: Vec<Ingredient>) -> Self {
        self.ingredients = ingredients;
        self
    }

    /// Add a single ingredient
    pub fn add_ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    /// Set all preparation steps at once (replaces any existing)
    pub fn preparation_steps(mut self, steps: Vec<String>) -> Self {
        self.preparation_steps = steps;
        self
    }

    /// Add a single preparation step
    ///
    /// Steps can be multiline and support markdown-like formatting:
    /// - `# Header` for section headers
    /// - `**bold**` for bold text
    /// - `_italic_` for italic text
    pub fn add_step(mut self, step: impl Into<String>) -> Self {
        self.preparation_steps.push(step.into());
        self
    }

    /// Set the recipe note/description
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Set the source name (e.g., "Web", "Cookbook", "Grandma")
    pub fn source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// Set the source URL
    pub fn source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    /// Set the serving size (e.g., "4 servings", "6-8 portions")
    pub fn servings(mut self, servings: impl Into<String>) -> Self {
        self.servings = Some(servings.into());
        self
    }

    /// Set prep time in minutes
    pub fn prep_time(mut self, minutes: i32) -> Self {
        self.prep_time = Some(minutes);
        self
    }

    /// Set cook time in minutes
    pub fn cook_time(mut self, minutes: i32) -> Self {
        self.cook_time = Some(minutes);
        self
    }

    /// Set rating (1-5, will be clamped to valid range)
    pub fn rating(mut self, rating: i32) -> Self {
        self.rating = Some(rating.clamp(1, 5));
        self
    }

    /// Set nutritional information (freeform text, can be multiline)
    ///
    /// Example: "350 cals\n15mg Vitamin D"
    pub fn nutritional_info(mut self, info: impl Into<String>) -> Self {
        self.nutritional_info = Some(info.into());
        self
    }

    /// Set the cover photo ID (from upload_photo)
    pub fn photo_id(mut self, id: impl Into<String>) -> Self {
        self.photo_id = Some(id.into());
        self
    }

    /// Convert to protobuf recipe
    fn to_pb_recipe(&self, recipe_id: &str, timestamp: f64) -> PbRecipe {
        let pb_ingredients: Vec<PbIngredient> = self
            .ingredients
            .iter()
            .map(|i| PbIngredient {
                raw_ingredient: i.raw_ingredient.clone(),
                name: Some(i.name.clone()),
                quantity: i.quantity.clone(),
                note: i.note.clone(),
            })
            .collect();

        let photo_ids = self.photo_id.clone().into_iter().collect();

        PbRecipe {
            identifier: recipe_id.to_string(),
            timestamp: Some(timestamp),
            name: Some(self.name.clone()),
            icon: None,
            note: self.note.clone(),
            source_name: self.source_name.clone(),
            source_url: self.source_url.clone(),
            ingredients: pb_ingredients,
            preparation_steps: self.preparation_steps.clone(),
            photo_ids,
            ad_campaign_id: None,
            photo_urls: vec![],
            scale_factor: Some(1.0),
            rating: self.rating,
            creation_timestamp: Some(timestamp),
            nutritional_info: self.nutritional_info.clone(),
            cook_time: self.cook_time,
            prep_time: self.prep_time,
            servings: self.servings.clone(),
            paprika_identifier: None,
        }
    }

    /// Save the recipe (creates if new, updates if existing)
    pub async fn save(self, client: &AnyListClient) -> Result<Recipe> {
        if self.id.is_some() {
            self.update(client).await
        } else {
            self.create(client).await
        }
    }

    /// Create a new recipe
    async fn create(self, client: &AnyListClient) -> Result<Recipe> {
        let recipe_id = generate_id();
        let operation_id = generate_id();
        let timestamp = current_timestamp();

        let pb_recipe = self.to_pb_recipe(&recipe_id, timestamp);

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("save-recipe".to_string()),
                user_id: Some(client.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            recipe_data_id: None,
            recipe: Some(pb_recipe),
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

        client.post("data/user-recipe-data/update", buf).await?;

        Ok(Recipe {
            id: recipe_id,
            name: self.name,
            ingredients: self.ingredients,
            preparation_steps: self.preparation_steps,
            note: self.note,
            source_name: self.source_name,
            source_url: self.source_url,
            servings: self.servings,
            prep_time: self.prep_time,
            cook_time: self.cook_time,
            rating: self.rating,
            nutritional_info: self.nutritional_info,
            photo_id: self.photo_id,
            photo_urls: vec![],
        })
    }

    /// Update an existing recipe
    async fn update(self, client: &AnyListClient) -> Result<Recipe> {
        let recipe_id = self.id.clone().expect("update called without recipe ID");
        let operation_id = generate_id();
        let timestamp = current_timestamp();

        let pb_recipe = self.to_pb_recipe(&recipe_id, timestamp);

        let operation = PbRecipeOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("save-recipe".to_string()),
                user_id: Some(client.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
            }),
            recipe_data_id: None,
            recipe: Some(pb_recipe),
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

        client.post("data/user-recipe-data/update", buf).await?;

        Ok(Recipe {
            id: recipe_id,
            name: self.name,
            ingredients: self.ingredients,
            preparation_steps: self.preparation_steps,
            note: self.note,
            source_name: self.source_name,
            source_url: self.source_url,
            servings: self.servings,
            prep_time: self.prep_time,
            cook_time: self.cook_time,
            rating: self.rating,
            nutritional_info: self.nutritional_info,
            photo_id: self.photo_id,
            photo_urls: vec![],
        })
    }
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
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let recipes = client.get_recipes().await.expect("Failed to get recipes");
    /// for recipe in recipes {
    ///     println!("Recipe: {}", recipe.name());
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
    /// # use anylist_rs::{AnyListClient, Ingredient};
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// let ingredients = vec![
    ///     Ingredient::new("Flour").quantity_of("2 cups"),
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
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
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
            nutritional_info: None,
            photo_id: None,
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
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
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
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::Undefined as i32),
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
    /// let client = AnyListClient::login("user@example.com", "password")
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

    /// Upload a photo for use as a recipe cover image.
    ///
    /// Returns the photo ID which can be used with `RecipeBuilder::photo_id()`.
    ///
    /// # Arguments
    ///
    /// * `data` - The image bytes (JPEG recommended)
    /// * `filename` - Original filename (e.g., "pasta.jpg")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::{AnyListClient, RecipeBuilder};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password").await?;
    ///
    /// // Upload a photo
    /// let photo_data = std::fs::read("pasta.jpg")?;
    /// let photo_id = client.upload_photo(photo_data, "pasta.jpg").await?;
    ///
    /// // Use in a recipe
    /// let recipe = RecipeBuilder::new("Pasta")
    ///     .photo_id(photo_id)
    ///     .save(&client)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_photo(&self, data: Vec<u8>, filename: &str) -> Result<String> {
        let photo_id = generate_id();
        let server_filename = format!("{}.jpg", photo_id);

        let photo_part = reqwest::multipart::Part::bytes(data)
            .file_name(filename.to_string());

        let form = reqwest::multipart::Form::new()
            .text("filename", server_filename)
            .part("photo", photo_part);

        self.post_multipart_form("/data/photos/upload", form).await?;

        Ok(photo_id)
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

            let photo_id = recipe.photo_ids.first().cloned();

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
                nutritional_info: recipe.nutritional_info,
                photo_id,
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
