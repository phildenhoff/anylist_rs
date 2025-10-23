use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    let list = client.get_list_by_name("Groceries").await?;

    // Create a store
    let store = client.create_store(&list.id, "Whole Foods").await?;

    // Update store name
    client.update_store(&list.id, &store.id, "Whole Foods Market").await?;

    Ok(())
}
