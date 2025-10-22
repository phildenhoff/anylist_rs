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

- **Authentication**: Bearer token authentication with explicit token refresh
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

... or you can see my [anylist_cli](https://github.com/phildenhoff/anylist_cli)

## Token Management

The client uses bearer token authentication. Tokens can be saved and reused:

```rust
use anylist_rs::AnyListClient;

// Initial login
let client = AnyListClient::new("email@example.com", "password").await?;

// Save tokens for later use (tokens are public fields)
let access_token = &client.access_token;
let refresh_token = &client.refresh_token;
let user_id = &client.user_id;
let is_premium = client.is_premium_user;

// Later, restore from saved tokens
let client = AnyListClient::from_tokens(
    access_token.clone(),
    refresh_token.clone(),
    user_id.to_string(),
    is_premium
);
```

### Token Refresh

When tokens expire (you'll get a 401 error), explicitly refresh them:

```rust
use anylist_rs::AnyListClient;

let client = AnyListClient::new("email@example.com", "password").await?;

// ... use client for API calls ...

// When you get a 401 error, refresh the token
let client = client.refresh().await?;

// Continue using the refreshed client
```

The `refresh()` method consumes the old client and returns a new one with updated tokens.

## Possible future features

- Real-time sync via WebSockets
  - Connect to WebSocket `wss://www.anylist.com/data/add-user-listener?access_token=<JWT>`
  - Heartbeat protocol for connection health
  - Handle "refresh-shopping-lists" messages for collaborative updates
- Logical timestamps for conflict resolution
- Photo upload/download support (S3 presigned URLs)
- List folders and organization
- Recipe web import from URLs
- iCalendar feed export for meal planning


## Disclaimer

I made this on my own, without help or knowledge from the AnyList folks.
Please, don't use this for malice.
