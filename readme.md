# anylist_rs

A Rust crate for interacting with the grocery list management app
[AnyList](https://www.anylist.com/)'s undocumented API.

```rust
use anylist_rs::{AnyListClient, Ingredient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Authenticate with email and password
    let client = AnyListClient::new("your-email@example.com", "your-password").await?;

    // Get all lists
    let lists = client.get_lists().await?;
    for list in &lists {
        println!("List: {} ({} items)", list.name, list.items.len());
    }

    // Create a new list
    let grocery_list = client.create_list("Weekly Groceries").await?;
    println!("Created list: {}", grocery_list.name);

    // Add items to the list
    client.add_item(&grocery_list.id, "Milk").await?;
    client.add_item_with_details(
        &grocery_list.id,
        "Apples",
        Some("2 lbs"),
        Some("Organic if possible"),
        Some("Produce")
    ).await?;

    Ok(())
}
```

## Features

- **Authentication**: Email/password login with long-lived credentials for reuse
- **Lists**: Create, read, update, and delete shopping lists
- **Items**: Full CRUD operations for list items including quantity,
  details, and categories
- **Recipes**: Create and manage recipes with ingredients and steps
- Categories, stores, and meal plans
- Uses Protobuf-based AnyList API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anylist_rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Examples

- [Lists](./docs/examples/lists.rs)
- [List items](./docs/examples/list_items.rs)
- [Recipes](./docs/examples/recipes.rs)
- [Meal planning](./docs/examples/meal_planning.rs)
- [Categories](./docs/examples/categories.rs)
- [Stores](./docs/examples/stores.rs)

## Credential Management

To reuse credentials without re-authenticating:

```rust
use anylist_rs::AnyListClient;

// Initial login
let client = AnyListClient::new("email@example.com", "password").await?;

// Save credentials (an exercise for the reader)
let signed_user_id = &client.signed_user_id;
let user_id = &client.user_id;
let is_premium = client.is_premium_user;

// Later, restore from saved credentials
let client = AnyListClient::from_credentials(
    signed_user_id.to_string(),
    user_id.to_string(),
    is_premium
);
```

## Possible future features

- Collaboration
- Real-time sync via Websockets
  - Get JWT via https://www.anylist.com/auth/token/refresh
  - Connect to WebSocket wss://www.anylist.com/data/add-user-listener?client_id=<CLIENT_ID>&access_token=<JWT>
  - When an item is added, marked-off, or deleted, we see the same event:
    "refresh-shopping-lists". So we'd want to build a caching layer that diffs
    the list before & after to know what's changed


## Disclaimer

I made this on my own, without help or knowledge from the AnyList folks.
Please, don't use this for malice.
