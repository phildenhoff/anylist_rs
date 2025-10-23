use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;
    let list = client.get_list_by_name("Groceries").await?;

    // Add items with various details
    let item = client.add_item_with_details(
        &list.id,
        "Flour",
        Some("2 cups"),
        Some("For baking"),
        Some("Baking")
    ).await?;

    // Cross off (check) an item
    client.cross_off_item(&list.id, &item.id).await?;

    // Uncheck an item
    client.uncheck_item(&list.id, &item.id).await?;

    // Update an item
    client.update_item(
        &list.id,
        &item.id,
        "Whole Wheat Flour",
        Some("3 cups"),
        Some("Organic"),
        Some("Baking")
    ).await?;

    // Delete a specific item
    client.delete_item(&list.id, &item.id).await?;

    // Delete all crossed-off items
    client.delete_all_crossed_off_items(&list.id).await?;

    Ok(())
}
