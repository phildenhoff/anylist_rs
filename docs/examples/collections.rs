use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::login("email@example.com", "password").await?;

    // Pick a recipe to file
    let recipes = client.get_recipes().await?;
    let recipe = recipes.first().expect("account has at least one recipe");

    // Create a collection, optionally pre-populated with recipes
    let weeknights = client
        .create_recipe_collection_with_recipe_ids("Weeknights", vec![recipe.id().to_string()])
        .await?;
    println!("Created collection: {}", weeknights.name());

    // Add and remove recipes
    client
        .add_recipe_to_collection(weeknights.id(), recipe.id())
        .await?;
    client
        .remove_recipe_from_collection(weeknights.id(), recipe.id())
        .await?;

    // List collections
    for collection in client.get_recipe_collections().await? {
        println!(
            "{} ({} recipes)",
            collection.name(),
            collection.recipe_ids().len()
        );
    }

    // Delete the collection
    client.delete_recipe_collection(weeknights.id()).await?;

    Ok(())
}
