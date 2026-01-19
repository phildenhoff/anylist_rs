# anylist_rs

A Rust crate for interacting with the grocery list management app
[AnyList](https://www.anylist.com/)'s undocumented API.

```rust
use anylist_rs::{AnyListClient, Ingredient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Authenticate with email and password
    let client = AnyListClient::login("your-email@example.com", "your-password").await?;

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

- **Authentication**: Bearer token authentication with automatic token refresh
- **Lists**: Create, read, update, and delete shopping lists
- **Items**: Full CRUD operations for list items including quantity,
  details, and categories
- **Recipes**: Create and manage recipes with ingredients and steps
- **Categories, stores, and meal plans**
- **Token persistence**: Save and restore authentication sessions
- Uses Protobuf-based AnyList API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anylist_rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### TLS Backend

By default, anylist_rs uses the system's native TLS implementation (`native-tls`), which relies on OpenSSL on Linux, Secure Transport on macOS, and SChannel on Windows.

For environments where OpenSSL is unavailable or undesirable (such as musl-based Linux distributions, static builds, or cross-compilation), you can use `rustls` instead:

```toml
[dependencies]
anylist_rs = { version = "0.1.0", default-features = false, features = ["rustls-tls"] }
```

| Feature | TLS Implementation | Use Case |
|---------|-------------------|----------|
| `native-tls` (default) | System TLS (OpenSSL/SecureTransport/SChannel) | Standard deployments, system certificate store integration |
| `rustls-tls` | rustls (pure Rust) | Static builds, musl/Alpine Linux, cross-compilation, no OpenSSL dependency |

## Examples

- [Lists](./docs/examples/lists.rs)
- [List items](./docs/examples/list_items.rs)
- [Recipes](./docs/examples/recipes.rs)
- [Meal planning](./docs/examples/meal_planning.rs)
- [Categories](./docs/examples/categories.rs)
- [Stores](./docs/examples/stores.rs)

... or you can see my [anylist_cli](https://github.com/phildenhoff/anylist_cli)

## Token Management

The client automatically manages authentication tokens and refreshes them on 401 errors. You can also save and restore sessions:

```rust
use anylist_rs::{AnyListClient, SavedTokens};

// Initial login
let client = AnyListClient::login("email@example.com", "password").await?;

// Export tokens for persistence (e.g., save to keychain/config)
let tokens: SavedTokens = client.export_tokens()?;
// save_to_storage(&tokens)?;

// Later, restore from saved tokens
let tokens: SavedTokens = load_from_storage()?;
let client = AnyListClient::from_tokens(tokens)?;

// Use the client - tokens automatically refresh on 401
let lists = client.get_lists().await?;
```

### Monitoring Token Refresh

You can optionally track authentication events:

```rust
use anylist_rs::{AnyListClient, AuthEvent};

let client = AnyListClient::login("email@example.com", "password")
    .await?
    .on_auth_event(|event| {
        match event {
            AuthEvent::TokensRefreshed => println!("Tokens refreshed!"),
            AuthEvent::RefreshFailed(err) => eprintln!("Refresh failed: {}", err),
        }
    });
```

### Disabling Auto-Refresh

If you want manual control over token refresh:

```rust
let client = AnyListClient::login("email@example.com", "password")
    .await?
    .disable_auto_refresh();

// Now 401 errors will be returned instead of automatically refreshing
```

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
