use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    let list = client.get_list_by_name("Groceries").await?;

    // Create a category
    let category = client.create_category(
        &list.id,
        "category-group-id",
        "Produce"
    ).await?;

    // Rename a category
    client.rename_category(
        &list.id,
        "category-group-id",
        &category.id,
        "Fresh Produce"
    ).await?;

    Ok(())
}
