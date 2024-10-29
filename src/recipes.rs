// use crate::api::get_user_data;
use crate::api::get_user_data;
use crate::protobuf::anylist::{PbIngredient, PbRecipe as ApiRecipe, PbRecipeDataResponse};
use std::error::Error;

pub struct Ingredient {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}
#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    // pub ingredients: Vec<Ingredient>,
    // pub preparation_steps: Vec<String>,
    pub source_url: String,
}

pub async fn get_recipes(signed_user_id: &str) -> Result<Vec<Recipe>, Box<dyn Error>> {
    let data = get_user_data(signed_user_id).await?;
    let recipes = match data.recipe_data_response {
        Some(ref res) => recipes_from_response(res.clone()),
        None => Vec::new(),
    };
    Ok(recipes)
}

fn recipes_from_response(response: PbRecipeDataResponse) -> Vec<Recipe> {
    let mut recipes: Vec<Recipe> = Vec::new();

    for recipe in response.recipes {
        if let Some(name) = recipe.name {
            let recipe: Recipe = Recipe {
                id: recipe.identifier,
                name,
                // ingredients: recipe.ingredients.into_iter().map(|ingredient| Ingredient {
                //     name: ingredient.name,
                //     quantity: ingredient.quantity,
                //     unit: ingredient.unit,
                // }).collect(),
                // preparation_steps: recipe.instructions,
                source_url: recipe.source_url.unwrap_or(String::new()),
            };
            recipes.push(recipe);
        }
    }

    recipes
}
