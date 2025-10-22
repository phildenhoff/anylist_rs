use anylist_rs::{AnyListClient, Ingredient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    // Create a recipe
    let ingredients = vec![
        Ingredient {
            name: "Flour".to_string(),
            quantity: Some("2 cups".to_string()),
            note: None,
            raw_ingredient: None,
        },
        Ingredient {
            name: "Sugar".to_string(),
            quantity: Some("1 cup".to_string()),
            note: None,
            raw_ingredient: None,
        },
        Ingredient {
            name: "Eggs".to_string(),
            quantity: Some("2".to_string()),
            note: None,
            raw_ingredient: None,
        },
    ];

    let steps = vec![
        "Preheat oven to 350°F".to_string(),
        "Mix dry ingredients".to_string(),
        "Add eggs and mix well".to_string(),
        "Bake for 30 minutes".to_string(),
    ];

    let recipe = client.create_recipe("Simple Cake", ingredients, steps).await?;
    println!("Created recipe: {}", recipe.name);

    // Get all recipes
    let recipes = client.get_recipes().await?;
    for recipe in &recipes {
        println!("Recipe: {} ({} ingredients)", recipe.name, recipe.ingredients.len());
    }

    // Delete a recipe
    client.delete_recipe(&recipe.id).await?;

    Ok(())
}
