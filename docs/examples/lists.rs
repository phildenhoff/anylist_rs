use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    // Create a new list
    let list = client.create_list("Groceries").await?;

    // Get a list by name
    let list = client.get_list_by_name("Groceries").await?;
    println!("List ID: {}", list.id);

    // Rename a list
    client.rename_list(&list.id, "Weekly Groceries").await?;

    // Delete a list
    client.delete_list(&list.id).await?;

    Ok(())
}
