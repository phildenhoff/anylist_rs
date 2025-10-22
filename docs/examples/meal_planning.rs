use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    // Note: You'll need to get the calendar ID from your account data
    let calendar_id = "your-calendar-id";

    // Create a meal plan event for a recipe
    let recipe = client.get_recipe_by_name("Pasta Carbonara").await?;
    let event = client.create_meal_plan_event(
        calendar_id,
        "2025-10-25",  // Date in YYYY-MM-DD format
        Some(&recipe.id),
        None,
        Some("dinner-label-id")  // Meal label (Breakfast, Lunch, Dinner)
    ).await?;

    // Create a note-based event (e.g., "Eating out")
    client.create_meal_plan_event(
        calendar_id,
        "2025-10-26",
        None,
        Some("Dinner at restaurant"),
        Some("dinner-label-id")
    ).await?;

    // Get meal plan for a date range
    let events = client.get_meal_plan_events("2025-10-25", "2025-10-31").await?;
    for event in &events {
        println!("Event on {}: {:?}", event.date, event.title);
    }

    // Add all meal plan ingredients to shopping list for the week
    let list = client.get_list_by_name("Groceries").await?;
    client.add_meal_plan_ingredients_to_list(
        &list.id,
        "2025-10-25",
        "2025-10-31"
    ).await?;

    Ok(())
}
