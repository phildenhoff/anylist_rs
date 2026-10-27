use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::operations;
use crate::protobuf::anylist::{
    pb_starter_list, PbIngredient, PbRecipe, PbRecipeDataResponse, PbUserDataResponse,
};
use crate::utils::{current_timestamp, generate_id};
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

    /// The ingredient as the web client stores it: with an identifier, an
    /// empty note when it has none, and the text it was parsed from.
    fn to_pb(&self, identifier: String) -> PbIngredient {
        let quantity = self.quantity.clone().filter(|q| !q.is_empty());
        let note = self.note.clone().unwrap_or_default();
        let raw_ingredient = self.raw_ingredient.clone().unwrap_or_else(|| {
            let mut raw = match &quantity {
                Some(quantity) => format!("{} {}", quantity, self.name),
                None => self.name.clone(),
            };
            if !note.is_empty() {
                raw = format!("{}, {}", raw, note);
            }
            raw
        });
        PbIngredient {
            raw_ingredient: Some(raw_ingredient),
            name: Some(self.name.clone()),
            quantity,
            note: Some(note),
            identifier: Some(identifier),
            is_heading: None,
        }
    }

    /// Whether a stored ingredient is this one.
    fn matches(&self, stored: &PbIngredient) -> bool {
        let text = |s: &Option<String>| s.clone().unwrap_or_default();
        stored.name.as_deref() == Some(self.name.as_str())
            && text(&stored.quantity) == text(&self.quantity)
            && text(&stored.note) == text(&self.note)
            && (self.raw_ingredient.is_none() || stored.raw_ingredient == self.raw_ingredient)
    }
}

/// A section heading in a recipe's ingredient list (e.g. "Sauce").
///
/// The AnyList protocol gives every heading a stable identifier; it is
/// preserved when a recipe is fetched and reused on save, so editing a
/// recipe does not rewrite its heading identifiers. `new` generates an
/// identifier for a genuinely new section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeIngredientSection {
    identifier: String,
    name: String,
}

impl RecipeIngredientSection {
    /// Create a new section heading with a freshly generated identifier.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            identifier: generate_id(),
            name: name.into(),
        }
    }

    /// Create a section heading with a known identifier (e.g. one the
    /// server assigned).
    pub fn with_identifier(identifier: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            name: name.into(),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn to_pb(&self) -> PbIngredient {
        PbIngredient {
            raw_ingredient: None,
            name: Some(self.name.clone()),
            quantity: None,
            note: None,
            identifier: Some(self.identifier.clone()),
            is_heading: Some(true),
        }
    }
}

/// A single entry in a recipe's ingredient list: either a section heading
/// or an ingredient. Lets a recipe group its ingredients under headings
/// (e.g. "Sauce", "Topping") the way the AnyList apps display them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeIngredientEntry {
    Section(RecipeIngredientSection),
    Ingredient(Ingredient),
}

impl RecipeIngredientEntry {
    /// Create a new section heading entry with a freshly generated
    /// identifier.
    pub fn section(name: impl Into<String>) -> Self {
        Self::Section(RecipeIngredientSection::new(name))
    }

    pub fn ingredient(ingredient: Ingredient) -> Self {
        Self::Ingredient(ingredient)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    id: String,
    name: String,
    ingredients: Vec<Ingredient>,
    #[serde(default)]
    ingredient_entries: Vec<RecipeIngredientEntry>,
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

    /// The full ingredient list in display order, including section headings.
    ///
    /// A `Recipe` deserialized from JSON written before this field existed
    /// has an empty entry list even when [`ingredients`](Self::ingredients)
    /// is populated; `RecipeBuilder::from` rebuilds the entries from the flat
    /// list in that case.
    pub fn ingredient_entries(&self) -> &[RecipeIngredientEntry] {
        &self.ingredient_entries
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
    /// Section headings and ingredients in display order.
    ingredient_entries: Vec<RecipeIngredientEntry>,
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
            ingredient_entries: Vec::new(),
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
        // Recipes deserialized from JSON written before ingredient_entries
        // existed have an empty entry list; rebuild it from the flat list so
        // later entry edits (e.g. add_ingredient_section) don't lose them.
        let ingredient_entries =
            if recipe.ingredient_entries.is_empty() && !recipe.ingredients.is_empty() {
                recipe
                    .ingredients
                    .iter()
                    .cloned()
                    .map(RecipeIngredientEntry::Ingredient)
                    .collect()
            } else {
                recipe.ingredient_entries.clone()
            };
        Self {
            id: Some(recipe.id.clone()),
            name: recipe.name.clone(),
            ingredients: recipe.ingredients.clone(),
            ingredient_entries,
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

    /// Set the recipe name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set all ingredients at once (replaces any existing ingredients and
    /// removes any section headings)
    pub fn ingredients(mut self, ingredients: Vec<Ingredient>) -> Self {
        self.ingredient_entries = ingredients
            .iter()
            .cloned()
            .map(RecipeIngredientEntry::Ingredient)
            .collect();
        self.ingredients = ingredients;
        self
    }

    /// Set the full ingredient list, including section headings, at once
    /// (replaces any existing ingredients and section headings).
    pub fn ingredient_entries(mut self, entries: Vec<RecipeIngredientEntry>) -> Self {
        self.ingredients = entries
            .iter()
            .filter_map(|entry| match entry {
                RecipeIngredientEntry::Section(_) => None,
                RecipeIngredientEntry::Ingredient(ingredient) => Some(ingredient.clone()),
            })
            .collect();
        self.ingredient_entries = entries;
        self
    }

    /// Add a single ingredient
    pub fn add_ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient_entries
            .push(RecipeIngredientEntry::Ingredient(ingredient.clone()));
        self.ingredients.push(ingredient);
        self
    }

    /// Add a section heading to the ingredient list.
    ///
    /// The heading gets a freshly generated identifier; to reuse a known
    /// identifier, push a [`RecipeIngredientSection::with_identifier`] entry
    /// via [`ingredient_entries`](Self::ingredient_entries).
    pub fn add_ingredient_section(mut self, section: impl Into<String>) -> Self {
        self.ingredient_entries
            .push(RecipeIngredientEntry::section(section));
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

    /// Set prep time in minutes (stored in seconds, as the web client does)
    pub fn prep_time(mut self, minutes: i32) -> Self {
        self.prep_time = Some(minutes);
        self
    }

    /// Set cook time in minutes (stored in seconds, as the web client does)
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
    /// The ingredient list as the web client stores it. Ingredients the
    /// stored recipe already has keep their identifiers: first exact
    /// matches, then ones with the same name.
    fn to_pb_ingredients(&self, stored: &[PbIngredient]) -> Vec<PbIngredient> {
        // Defensive fallback: a Recipe can reach a builder with an empty entry
        // list but a populated flat list (e.g. deserialized from JSON written
        // before ingredient_entries existed).
        let fallback_entries: Vec<RecipeIngredientEntry>;
        let entries: &[RecipeIngredientEntry] =
            if self.ingredient_entries.is_empty() && !self.ingredients.is_empty() {
                fallback_entries = self
                    .ingredients
                    .iter()
                    .cloned()
                    .map(RecipeIngredientEntry::Ingredient)
                    .collect();
                &fallback_entries
            } else {
                &self.ingredient_entries
            };

        let mut unused: Vec<&PbIngredient> = stored
            .iter()
            .filter(|i| i.is_heading != Some(true))
            .collect();
        let mut pb: Vec<Option<PbIngredient>> = entries
            .iter()
            .map(|entry| match entry {
                RecipeIngredientEntry::Section(section) => Some(section.to_pb()),
                RecipeIngredientEntry::Ingredient(ingredient) => {
                    let position = unused.iter().position(|s| ingredient.matches(s))?;
                    Some(unused.remove(position).clone())
                }
            })
            .collect();
        for (entry, slot) in entries.iter().zip(pb.iter_mut()) {
            let RecipeIngredientEntry::Ingredient(ingredient) = entry else {
                continue;
            };
            if slot.is_some() {
                continue;
            }
            let identifier = unused
                .iter()
                .position(|s| {
                    s.name
                        .as_deref()
                        .is_some_and(|n| n.to_lowercase() == ingredient.name.to_lowercase())
                })
                .and_then(|position| unused.remove(position).identifier.clone())
                .unwrap_or_else(generate_id);
            *slot = Some(ingredient.to_pb(identifier));
        }
        pb.into_iter().flatten().collect()
    }

    /// The recipe as the web client's editor saves it: a new recipe gets a
    /// creation time; an existing one keeps the stored fields the builder
    /// does not cover. Times are stored in seconds.
    fn to_pb_recipe(&self, stored: Option<&PbRecipe>, recipe_id: &str) -> PbRecipe {
        let base = match stored {
            Some(stored) => stored.clone(),
            None => PbRecipe {
                identifier: recipe_id.to_string(),
                creation_timestamp: Some(creation_timestamp()),
                ..Default::default()
            },
        };
        let seconds = |minutes: Option<i32>, stored: Option<i32>| match (minutes, stored) {
            (Some(minutes), Some(stored)) if stored / 60 == minutes => Some(stored),
            (minutes, _) => minutes.map(|m| m * 60),
        };
        let photo_ids = match &self.photo_id {
            Some(id) if base.photo_ids.first() == Some(id) => base.photo_ids.clone(),
            photo_id => photo_id.clone().into_iter().collect(),
        };
        let text = |value: &Option<String>| value.clone().filter(|v| !v.is_empty());
        PbRecipe {
            name: Some(self.name.clone()),
            note: text(&self.note),
            source_name: text(&self.source_name),
            source_url: text(&self.source_url),
            ingredients: self.to_pb_ingredients(&base.ingredients),
            preparation_steps: self.preparation_steps.clone(),
            photo_ids,
            rating: self.rating,
            nutritional_info: text(&self.nutritional_info),
            cook_time: seconds(self.cook_time, base.cook_time),
            prep_time: seconds(self.prep_time, base.prep_time),
            servings: text(&self.servings),
            ..base
        }
    }

    /// Save the recipe (creates if new, updates if existing)
    ///
    /// Like the web client, saving an existing recipe also updates its
    /// ingredients on the list recipe ingredients were last added to.
    pub async fn save(self, client: &AnyListClient) -> Result<Recipe> {
        let data = client.get_user_data().await?;
        let recipe_data_id = recipe_data_id(&data)?;
        let mut ids = client.ids();
        let (recipe, mut requests, old) = match &self.id {
            Some(recipe_id) => {
                let stored = stored_recipe(&data, recipe_id)?;
                let recipe = self.to_pb_recipe(Some(stored), recipe_id);
                (recipe, Vec::new(), Some(stored))
            }
            None => {
                let recipe_id = ids.new_id();
                (self.to_pb_recipe(None, &recipe_id), Vec::new(), None)
            }
        };
        requests.extend(operations::save_recipe(
            &mut ids,
            recipe_data_id,
            recipe.clone(),
        ));
        if let (Some(old), Some(list_id)) = (old, recipe_ingredients_list_id(&data)) {
            let items = data
                .shopping_list(list_id)
                .map(|l| l.items.as_slice())
                .unwrap_or_default();
            requests.extend(operations::update_recipe_ingredients_on_list(
                &mut ids,
                data.list_context(list_id),
                items,
                data.starter_list(list_id, pb_starter_list::Type::FavoriteItems),
                data.starter_list(list_id, pb_starter_list::Type::RecentItems),
                old,
                &recipe,
            ));
        }
        client.send(requests).await?;
        recipe_from_pb(recipe)
            .ok_or_else(|| AnyListError::InvalidResponse("Saved recipe has no name".to_string()))
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
    /// The recipe gets a flat ingredient list; to group ingredients under
    /// section headings, use [`RecipeBuilder::add_ingredient_section`] or
    /// [`RecipeBuilder::ingredient_entries`] instead.
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
        RecipeBuilder::new(name)
            .ingredients(ingredients)
            .preparation_steps(preparation_steps)
            .save(self)
            .await
    }

    /// Update an existing recipe's name, ingredients, and preparation steps
    ///
    /// The recipe is fetched first, so its other fields (note, source,
    /// rating, photo, servings, times) are preserved. Section headings in
    /// the ingredient list are preserved — with stable identifiers — as long
    /// as `ingredients` matches the recipe's current flat ingredient list
    /// exactly (same ingredients, same order, same quantities and notes) —
    /// the common case of editing only the name or steps. Any difference,
    /// including reordering, replaces the whole ingredient list and removes
    /// the section headings; use [`RecipeBuilder::from`] with
    /// [`RecipeBuilder::ingredient_entries`] to edit ingredients while
    /// keeping headings.
    ///
    /// Returns [`AnyListError::NotFound`] if no recipe with `recipe_id`
    /// exists.
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
        let existing = self.get_recipe_by_id(recipe_id).await?;
        let entries = entries_for_ingredient_update(&existing, ingredients);

        RecipeBuilder::from(&existing)
            .name(name)
            .ingredient_entries(entries)
            .preparation_steps(preparation_steps)
            .save(self)
            .await?;
        Ok(())
    }

    /// Delete a recipe
    ///
    /// Sends the recipe exactly as stored on the server alongside its ID,
    /// matching the delete operation of the official web client. If the
    /// recipe is not present in the user's data (e.g. it was already deleted
    /// from another device), the operation is still sent with just the recipe
    /// ID, so deleting an already-deleted recipe succeeds.
    ///
    /// # Arguments
    ///
    /// * `recipe_id` - The ID of the recipe to delete
    pub async fn delete_recipe(&self, recipe_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let recipe_data_id = recipe_data_id(&data)?;
        let recipe = stored_recipe(&data, recipe_id)?;
        let calendar = data.meal_planning_calendar_response.as_ref();
        let calendar_id = calendar
            .map(|c| c.calendar_id.as_str())
            .unwrap_or(recipe_data_id);
        let event_ids = calendar
            .into_iter()
            .flat_map(|c| c.events.iter().chain(&c.template_events))
            .filter(|e| e.recipe_id.as_deref() == Some(recipe_id))
            .map(|e| e.identifier.clone())
            .collect();
        let list = recipe_ingredients_list_id(&data).and_then(|list_id| {
            data.shopping_list(list_id)
                .map(|l| (list_id, l.items.as_slice()))
        });
        let requests = operations::delete_recipe(
            &mut self.ids(),
            recipe_data_id,
            calendar_id,
            recipe,
            event_ids,
            list,
        );
        self.send(requests).await
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
        let data = self.get_user_data().await?;
        let recipe = stored_recipe(&data, recipe_id)?;
        if data.shopping_list(list_id).is_none() {
            return Err(AnyListError::NotFound(format!(
                "List with ID {} not found",
                list_id
            )));
        }
        let scale = scale_factor
            .or(recipe.scale_factor)
            .filter(|s| *s != 0.0)
            .unwrap_or(1.0);
        let ingredients: Vec<_> = recipe
            .ingredients
            .iter()
            .filter(|i| i.is_heading != Some(true))
            .map(|i| operations::item_ingredient(recipe, &operations::scaled_ingredient(i, scale)))
            .collect();
        let items = data
            .shopping_list(list_id)
            .map(|l| l.items.as_slice())
            .unwrap_or_default();
        let requests = operations::add_ingredients_to_list(
            &mut self.ids(),
            data.list_context(list_id),
            items,
            data.starter_list(list_id, pb_starter_list::Type::FavoriteItems),
            data.starter_list(list_id, pb_starter_list::Type::RecentItems),
            &ingredients,
        );
        self.send(requests).await
    }

    /// Upload a photo for use as a recipe cover image.
    ///
    /// Returns the photo ID which can be used with `RecipeBuilder::photo_id()`.
    ///
    /// # Arguments
    ///
    /// * `data` - The image bytes; the MIME type is detected from the bytes.
    ///   JPEG, PNG, GIF, and WebP are all verified to work against the live
    ///   API: AnyList re-encodes uploads to JPEG server-side and serves the
    ///   photo at `https://photos.anylist.com/{photo_id}.jpg`. HEIC is
    ///   rejected — AnyList's own clients convert HEIC to JPEG before
    ///   uploading, so convert it first. Unrecognized data is also rejected.
    /// * `filename` - Original filename (e.g., "pasta.jpg"), sent as the
    ///   multipart part's filename. The server stores the photo under a
    ///   generated ID regardless.
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
        let mime = sniff_image_mime(&data).ok_or_else(|| {
            AnyListError::Other(
                "unrecognized image format: expected JPEG, PNG, GIF, or WebP".to_string(),
            )
        })?;
        if mime == "image/heic" {
            return Err(AnyListError::Other(
                "HEIC is not supported: AnyList's own clients convert HEIC to JPEG \
                 before uploading; convert the image to JPEG first"
                    .to_string(),
            ));
        }

        let photo_id = generate_id();
        let server_filename = format!("{}.jpg", photo_id);

        let photo_part = reqwest::multipart::Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str(mime)?;

        let form = reqwest::multipart::Form::new()
            .text("filename", server_filename)
            .part("photo", photo_part);

        self.post_multipart_form("/data/photos/upload", form)
            .await?;

        Ok(photo_id)
    }

    /// Download an existing recipe photo by photo ID.
    ///
    /// Fetches the JPEG bytes from `https://photos.anylist.com/{photo_id}.jpg`,
    /// the public CDN where AnyList serves recipe photos (no authentication
    /// required).
    ///
    /// # Arguments
    ///
    /// * `photo_id` - The bare photo ID (32 hex characters, from
    ///   `upload_photo` or `Recipe::photo_id`) — not a URL or a filename
    ///   with extension
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AnyListClient::login("user@example.com", "password").await?;
    ///
    /// let photo = client.download_photo("6694e8edc84744a496ea1feadeb8f177").await?;
    /// std::fs::write("photo.jpg", photo)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_photo(&self, photo_id: &str) -> Result<Vec<u8>> {
        let url = format!("https://photos.anylist.com/{photo_id}.jpg");
        self.get_bytes(&url).await
    }
}

/// Entry list for a legacy ingredient update: keep the existing entry
/// structure (section headings, stable identifiers) when the new flat
/// ingredient list is unchanged; otherwise the new ingredients replace the
/// whole list, without headings.
fn entries_for_ingredient_update(
    existing: &Recipe,
    ingredients: Vec<Ingredient>,
) -> Vec<RecipeIngredientEntry> {
    if existing.ingredients() == ingredients && !existing.ingredient_entries().is_empty() {
        existing.ingredient_entries().to_vec()
    } else {
        ingredients
            .into_iter()
            .map(RecipeIngredientEntry::Ingredient)
            .collect()
    }
}

fn recipes_from_response(response: PbRecipeDataResponse) -> Vec<Recipe> {
    response
        .recipes
        .into_iter()
        .filter_map(recipe_from_pb)
        .collect()
}

/// A stored recipe; times are stored in seconds and given in minutes.
fn recipe_from_pb(recipe: PbRecipe) -> Option<Recipe> {
    let name = recipe.name?;
    let mut ingredients: Vec<Ingredient> = Vec::new();
    let mut ingredient_entries: Vec<RecipeIngredientEntry> = Vec::new();
    for i in &recipe.ingredients {
        if i.is_heading == Some(true) {
            if let Some(name) = &i.name {
                // Preserve the server-assigned identifier so saving
                // the recipe again keeps headings stable.
                let section = match &i.identifier {
                    Some(id) => RecipeIngredientSection::with_identifier(id.clone(), name.clone()),
                    None => RecipeIngredientSection::new(name.clone()),
                };
                ingredient_entries.push(RecipeIngredientEntry::Section(section));
            }
        } else if let Some(name) = &i.name {
            let ingredient = Ingredient {
                name: name.clone(),
                quantity: i.quantity.clone(),
                note: i.note.clone(),
                raw_ingredient: i.raw_ingredient.clone(),
            };
            ingredients.push(ingredient.clone());
            ingredient_entries.push(RecipeIngredientEntry::Ingredient(ingredient));
        }
    }

    let photo_id = recipe.photo_ids.first().cloned();
    Some(Recipe {
        id: recipe.identifier,
        name,
        ingredients,
        ingredient_entries,
        preparation_steps: recipe.preparation_steps,
        note: recipe.note,
        source_name: recipe.source_name,
        source_url: recipe.source_url,
        servings: recipe.servings,
        prep_time: recipe.prep_time.map(|seconds| seconds / 60),
        cook_time: recipe.cook_time.map(|seconds| seconds / 60),
        rating: recipe.rating,
        nutritional_info: recipe.nutritional_info,
        photo_id,
        photo_urls: recipe.photo_urls,
    })
}

fn recipe_data_id(data: &PbUserDataResponse) -> Result<&str> {
    data.recipe_data_response
        .as_ref()
        .and_then(|r| r.recipe_data_id.as_deref())
        .ok_or_else(|| AnyListError::InvalidResponse("No recipe data ID".to_string()))
}

fn stored_recipe<'a>(data: &'a PbUserDataResponse, recipe_id: &str) -> Result<&'a PbRecipe> {
    data.recipe_data_response
        .as_ref()
        .into_iter()
        .flat_map(|r| &r.recipes)
        .find(|r| r.identifier == recipe_id)
        .ok_or_else(|| AnyListError::NotFound(format!("Recipe with ID {} not found", recipe_id)))
}

/// The list recipe ingredients were last added to in the apps, which the web
/// client keeps in step with recipe edits and deletions.
fn recipe_ingredients_list_id(data: &PbUserDataResponse) -> Option<&str> {
    data.mobile_app_settings_response
        .as_ref()?
        .list_id_for_recipe_ingredients
        .as_deref()
        .filter(|id| !id.is_empty())
}

/// The web client's creation time: now, in seconds, to the millisecond.
fn creation_timestamp() -> f64 {
    (current_timestamp() * 1000.0).floor() / 1000.0
}

/// Detect an image's MIME type from its magic bytes.
///
/// AnyList re-encodes uploads to JPEG server-side, so the upload just needs
/// to label the part with the actual format (matching the official web
/// client, which sends the browser-reported MIME type). HEIC is detected
/// only so `upload_photo` can reject it with a targeted error: official
/// clients convert HEIC to JPEG before uploading, so the server never sees
/// it and its support is unverified.
fn sniff_image_mime(data: &[u8]) -> Option<&'static str> {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png");
    }
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        if let b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx" | b"heif" | b"mif1"
        | b"msf1" = &data[8..12]
        {
            return Some("image/heic");
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::SavedTokens;

    #[test]
    fn test_sniff_image_mime() {
        assert_eq!(
            sniff_image_mime(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]),
            Some("image/jpeg")
        );
        assert_eq!(
            sniff_image_mime(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            Some("image/png")
        );
        assert_eq!(sniff_image_mime(b"GIF89a\x01\x00"), Some("image/gif"));
        assert_eq!(
            sniff_image_mime(b"RIFF\x24\x00\x00\x00WEBPVP8 "),
            Some("image/webp")
        );
        assert_eq!(
            sniff_image_mime(b"\x00\x00\x00\x18ftypheic\x00\x00\x00\x00"),
            Some("image/heic")
        );
        assert_eq!(sniff_image_mime(b"not an image"), None);
        assert_eq!(sniff_image_mime(&[]), None);
        // Truncated multi-byte signatures must not panic on the length guards
        assert_eq!(sniff_image_mime(b"RIFF"), None);
        assert_eq!(sniff_image_mime(b"\x00\x00\x00\x18ftyp"), None);
    }

    #[tokio::test]
    async fn test_upload_photo_rejects_unrecognized_data() {
        let client =
            AnyListClient::from_tokens(SavedTokens::new("access", "refresh", "user", false))
                .unwrap();

        // Must fail with a clear error before any network request is made
        let err = client
            .upload_photo(b"definitely not an image".to_vec(), "notes.txt")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("unrecognized image format"));
    }

    #[tokio::test]
    async fn test_upload_photo_rejects_heic() {
        let client =
            AnyListClient::from_tokens(SavedTokens::new("access", "refresh", "user", false))
                .unwrap();

        let heic_data = b"\x00\x00\x00\x18ftypheic\x00\x00\x00\x00".to_vec();
        let err = client
            .upload_photo(heic_data, "photo.heic")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("HEIC"));
    }

    fn heading_pb(identifier: Option<&str>, name: &str) -> PbIngredient {
        PbIngredient {
            name: Some(name.to_string()),
            identifier: identifier.map(str::to_string),
            is_heading: Some(true),
            ..Default::default()
        }
    }

    fn ingredient_pb(name: &str) -> PbIngredient {
        PbIngredient {
            name: Some(name.to_string()),
            ..Default::default()
        }
    }

    fn response_with_ingredients(ingredients: Vec<PbIngredient>) -> PbRecipeDataResponse {
        PbRecipeDataResponse {
            recipes: vec![PbRecipe {
                identifier: "recipe-1".to_string(),
                name: Some("Pasta".to_string()),
                ingredients,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn decode_preserves_heading_identifier_and_order() {
        let response = response_with_ingredients(vec![
            heading_pb(Some("sec-1"), "Sauce"),
            ingredient_pb("Tomato"),
            heading_pb(Some("sec-2"), "Topping"),
            ingredient_pb("Basil"),
        ]);

        let recipes = recipes_from_response(response);
        assert_eq!(recipes.len(), 1);
        let recipe = &recipes[0];

        // Flat list excludes headings.
        let names: Vec<&str> = recipe.ingredients().iter().map(Ingredient::name).collect();
        assert_eq!(names, ["Tomato", "Basil"]);

        // Entries keep display order and server identifiers.
        let entries = recipe.ingredient_entries();
        assert_eq!(entries.len(), 4);
        match &entries[0] {
            RecipeIngredientEntry::Section(section) => {
                assert_eq!(section.identifier(), "sec-1");
                assert_eq!(section.name(), "Sauce");
            }
            other => panic!("expected section, got {:?}", other),
        }
        match &entries[2] {
            RecipeIngredientEntry::Section(section) => {
                assert_eq!(section.identifier(), "sec-2");
                assert_eq!(section.name(), "Topping");
            }
            other => panic!("expected section, got {:?}", other),
        }
    }

    #[test]
    fn decode_generates_identifier_when_server_omits_it() {
        let response = response_with_ingredients(vec![heading_pb(None, "Sauce")]);
        let recipes = recipes_from_response(response);
        match &recipes[0].ingredient_entries()[0] {
            RecipeIngredientEntry::Section(section) => {
                assert!(!section.identifier().is_empty());
            }
            other => panic!("expected section, got {:?}", other),
        }
    }

    #[test]
    fn save_round_trip_keeps_heading_identifiers_stable() {
        let response = response_with_ingredients(vec![
            heading_pb(Some("sec-1"), "Sauce"),
            ingredient_pb("Tomato"),
        ]);
        let recipe = recipes_from_response(response).remove(0);

        let stored = PbRecipe {
            identifier: "recipe-1".to_string(),
            ingredients: vec![
                heading_pb(Some("sec-1"), "Sauce"),
                PbIngredient {
                    identifier: Some("tomato-1".to_string()),
                    ..ingredient_pb("Tomato")
                },
            ],
            ..Default::default()
        };
        let builder = RecipeBuilder::from(&recipe);
        let first = builder.to_pb_recipe(Some(&stored), "recipe-1");
        let second = builder.to_pb_recipe(Some(&stored), "recipe-1");

        assert_eq!(first.ingredients[0].identifier.as_deref(), Some("sec-1"));
        assert_eq!(first.ingredients[0].is_heading, Some(true));
        assert_eq!(first.ingredients[0].name.as_deref(), Some("Sauce"));
        // Saving again keeps the stored ingredients' identifiers.
        assert_eq!(first.ingredients[1].identifier.as_deref(), Some("tomato-1"));
        assert_eq!(first.ingredients, second.ingredients);
    }

    #[test]
    fn new_section_gets_one_identifier_stable_across_encodes() {
        let builder = RecipeBuilder::new("Pasta")
            .add_ingredient_section("Sauce")
            .add_ingredient(Ingredient::new("Tomato"));

        let first = builder.to_pb_recipe(None, "recipe-1");
        let second = builder.to_pb_recipe(None, "recipe-1");

        let heading = &first.ingredients[0];
        assert_eq!(heading.is_heading, Some(true));
        assert!(heading
            .identifier
            .as_deref()
            .is_some_and(|id| !id.is_empty()));
        assert_eq!(first.ingredients[0], second.ingredients[0]);

        // The plain ingredient is not marked as a heading.
        assert_eq!(first.ingredients[1].is_heading, None);
        assert_eq!(first.ingredients[1].name.as_deref(), Some("Tomato"));
    }

    #[test]
    fn ingredients_setter_replaces_sections() {
        let builder = RecipeBuilder::new("Pasta")
            .add_ingredient_section("Sauce")
            .ingredients(vec![Ingredient::new("Tomato")]);

        assert_eq!(builder.ingredient_entries.len(), 1);
        assert!(matches!(
            builder.ingredient_entries[0],
            RecipeIngredientEntry::Ingredient(_)
        ));
    }

    #[test]
    fn ingredient_entries_setter_syncs_flat_list() {
        let builder = RecipeBuilder::new("Pasta").ingredient_entries(vec![
            RecipeIngredientEntry::section("Sauce"),
            RecipeIngredientEntry::ingredient(Ingredient::new("Tomato")),
            RecipeIngredientEntry::section("Topping"),
            RecipeIngredientEntry::ingredient(Ingredient::new("Basil")),
        ]);

        let names: Vec<&str> = builder.ingredients.iter().map(Ingredient::name).collect();
        assert_eq!(names, ["Tomato", "Basil"]);
        assert_eq!(builder.ingredient_entries.len(), 4);
    }

    #[test]
    fn ingredient_update_with_unchanged_list_keeps_headings() {
        let response = response_with_ingredients(vec![
            heading_pb(Some("sec-1"), "Sauce"),
            ingredient_pb("Tomato"),
            ingredient_pb("Basil"),
        ]);
        let recipe = recipes_from_response(response).remove(0);

        // Same flat list (e.g. the caller only renamed the recipe).
        let entries = entries_for_ingredient_update(&recipe, recipe.ingredients().to_vec());
        assert_eq!(entries, recipe.ingredient_entries());
        match &entries[0] {
            RecipeIngredientEntry::Section(section) => assert_eq!(section.identifier(), "sec-1"),
            other => panic!("expected section, got {:?}", other),
        }
    }

    #[test]
    fn ingredient_update_with_reordered_list_replaces_entries() {
        let response = response_with_ingredients(vec![
            heading_pb(Some("sec-1"), "Sauce"),
            ingredient_pb("Tomato"),
            ingredient_pb("Basil"),
        ]);
        let recipe = recipes_from_response(response).remove(0);

        // The comparison is exact: a reordered list counts as changed, so
        // the headings are replaced (documented on update_recipe).
        let mut reordered = recipe.ingredients().to_vec();
        reordered.reverse();
        let entries = entries_for_ingredient_update(&recipe, reordered);
        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .all(|e| matches!(e, RecipeIngredientEntry::Ingredient(_))));
    }

    #[test]
    fn ingredient_update_with_changed_list_replaces_entries() {
        let response = response_with_ingredients(vec![
            heading_pb(Some("sec-1"), "Sauce"),
            ingredient_pb("Tomato"),
        ]);
        let recipe = recipes_from_response(response).remove(0);

        let entries = entries_for_ingredient_update(&recipe, vec![Ingredient::new("Garlic")]);
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0], RecipeIngredientEntry::Ingredient(_)));
    }

    #[test]
    fn builder_from_rebuilds_entries_for_pre_entries_recipes() {
        // A Recipe deserialized from JSON written before ingredient_entries
        // existed has an empty entry list.
        let old_json = r#"{
            "id": "r1",
            "name": "Bread",
            "ingredients": [
                {"name": "Flour", "quantity": null, "note": null, "raw_ingredient": null}
            ],
            "preparation_steps": [],
            "note": null,
            "source_name": null,
            "source_url": null,
            "servings": null,
            "prep_time": null,
            "cook_time": null,
            "rating": null,
            "nutritional_info": null,
            "photo_id": null,
            "photo_urls": []
        }"#;
        let recipe: Recipe = serde_json::from_str(old_json).unwrap();
        assert!(recipe.ingredient_entries().is_empty());

        // Entry-level edits must not lose the flat ingredients.
        let builder = RecipeBuilder::from(&recipe).add_ingredient_section("Toppings");
        let pb = builder.to_pb_recipe(None, "r1");
        let names: Vec<Option<&str>> = pb.ingredients.iter().map(|i| i.name.as_deref()).collect();
        assert_eq!(names, [Some("Flour"), Some("Toppings")]);
    }

    /// The recipe the web client's editor saved on 2026-10-27, made with the
    /// builder: times in seconds, empty notes, raw ingredient text.
    #[test]
    fn new_recipe_matches_web_client_capture() {
        use prost::Message;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/webapp_2026_10_27/recipes/0080-recipe-create.json");
        let fixture: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let bytes = hex::decode(fixture["hex"].as_str().unwrap()).unwrap();
        let captured = crate::protobuf::anylist::PbRecipeOperationList::decode(bytes.as_slice())
            .unwrap()
            .operations[0]
            .recipe
            .clone()
            .unwrap();

        let builder = RecipeBuilder::new("zz capture recipe")
            .note("zz capture note")
            .source_name("zz capture source")
            .source_url("https://example.com/zz-capture")
            .add_ingredient(Ingredient::new("zz flour").quantity_of("2 cups"))
            .add_ingredient(Ingredient::new("zz salt").quantity_of("1 tsp"))
            .add_ingredient(Ingredient::new("zz water").quantity_of("250 mL"))
            .add_step("Mix zz flour and zz salt.")
            .add_step("Add zz water and knead.")
            .rating(4)
            .nutritional_info("zz 100 kcal")
            .prep_time(15)
            .servings("4 servings");
        let mut recipe = builder.to_pb_recipe(None, &captured.identifier);
        recipe.creation_timestamp = captured.creation_timestamp;
        for (ours, theirs) in recipe.ingredients.iter_mut().zip(&captured.ingredients) {
            ours.identifier = theirs.identifier.clone();
        }
        assert_eq!(recipe, captured);
    }
}
