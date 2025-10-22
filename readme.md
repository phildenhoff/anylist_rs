# anylist_rs

A Rust crate for interacting with the grocery list management app
[AnyList](https://www.anylist.com/)'s undocumented API.

## Overview

- **List items**: create, read, update, and delete list items, includ quantity, details, and categories
- Authentication

AnyList is a grocery list management and meal planning platform with real-time
collaboration across iOS, Android, web, Mac, and Apple Watch. This Rust library
provides programmatic access to AnyList's extensive functionality through
reverse-engineered API endpoints using Protocol Buffers for data serialization.


This library implements comprehensive support for:

### Core Features (Phase 1)
- ✅ **Authentication & Session Management** - Email/password login with credential management
- ✅ **List Management** - Create, read, update, and delete shopping lists
- ✅ **Item Management** - Full CRUD operations for list items with quantity, details, and categories
- ✅ **Recipe Management** - Create and manage recipes with ingredients and preparation steps
- ✅ **Real-time Data Sync** - Protobuf-based communication with the AnyList API

### Advanced Features (Phase 2)
- ✅ **Recipe Collections** - Organize recipes into collections
- ✅ **Category Management** - Create and manage item categories
- ✅ **Store Management** - Manage stores and assign items to specific stores
- ✅ **Meal Planning** - Create, update, and delete meal plan calendar events
- ✅ **Recipe-to-List Integration** - Add recipe ingredients to shopping lists with automatic scaling

### API Coverage

Based on the comprehensive AnyList API feature set, this library implements:

- **Lists**: `get_lists()`, `get_list_by_id()`, `get_list_by_name()`, `create_list()`, `delete_list()`, `rename_list()`
- **Items**: `add_item()`, `add_item_with_details()`, `update_item()`, `delete_item()`, `cross_off_item()`, `uncheck_item()`, `delete_all_crossed_off_items()`
- **Recipes**: `get_recipes()`, `get_recipe_by_id()`, `create_recipe()`, `update_recipe()`, `delete_recipe()`, `add_recipe_to_list()`
- **Collections**: `get_recipe_collections()`, `create_recipe_collection()`, `delete_recipe_collection()`, `add_recipe_to_collection()`, `remove_recipe_from_collection()`
- **Categories**: `create_category()`, `delete_category()`, `rename_category()`
- **Stores**: `create_store()`, `update_store()`, `delete_store()`
- **Meal Planning**: `get_meal_plan_events()`, `create_meal_plan_event()`, `update_meal_plan_event()`, `delete_meal_plan_event()`, `add_meal_plan_ingredients_to_list()`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anylist_rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

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

## Usage Examples

### Working with Lists

```rust
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
```

### Managing List Items

```rust
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
```

### Recipe Management

```rust
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
```

### Recipe-to-List Integration

```rust
use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    let recipe = client.get_recipe_by_name("Simple Cake").await?;
    let list = client.get_list_by_name("Groceries").await?;

    // Add recipe ingredients to shopping list
    client.add_recipe_to_list(&recipe.id, &list.id, None).await?;

    // Add recipe ingredients with 2x scaling
    client.add_recipe_to_list(&recipe.id, &list.id, Some(2.0)).await?;

    Ok(())
}
```

### Recipe Collections

```rust
use anylist_rs::{AnyListClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AnyListClient::new("email@example.com", "password").await?;

    // Create a collection
    let collection = client.create_recipe_collection("Desserts").await?;

    // Add recipes to collection
    let recipe1 = client.get_recipe_by_name("Simple Cake").await?;
    let recipe2 = client.get_recipe_by_name("Chocolate Cookies").await?;

    client.add_recipe_to_collection(&collection.id, &recipe1.id).await?;
    client.add_recipe_to_collection(&collection.id, &recipe2.id).await?;

    // Get all collections
    let collections = client.get_recipe_collections().await?;
    for coll in &collections {
        println!("Collection: {} ({} recipes)", coll.name, coll.recipe_ids.len());
    }

    // Remove a recipe from collection
    client.remove_recipe_from_collection(&collection.id, &recipe1.id).await?;

    Ok(())
}
```

### Meal Planning

```rust
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
```

### Categories and Stores

```rust
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

    // Create a store
    let store = client.create_store(&list.id, "Whole Foods").await?;

    // Update store name
    client.update_store(&list.id, &store.id, "Whole Foods Market").await?;

    Ok(())
}
```

## Architecture

The library is built on:

- **Protocol Buffers** - Efficient data serialization matching AnyList's internal format
- **Tokio** - Async runtime for concurrent operations
- **Reqwest** - HTTP client for API communication
- **Prost** - Protocol buffer implementation

### Module Structure

```
src/
├── client.rs          - Core client with authentication and HTTP methods
├── error.rs           - Error types and Result alias
├── lists.rs           - List CRUD operations
├── items.rs           - Item CRUD operations
├── recipes.rs         - Recipe management
├── collections.rs     - Recipe collection management
├── categories.rs      - Category management
├── stores.rs          - Store management
├── meal_planning.rs   - Meal planning calendar
└── utils.rs          - Utility functions (ID generation, timestamps)
```

## Implementation Status

### Completed Features (Phase 1-2)
- ✅ Core client structure with authentication
- ✅ List CRUD operations (create, update, delete)
- ✅ Item CRUD operations with full metadata support
- ✅ Recipe CRUD with ingredients and preparation steps
- ✅ Recipe collections
- ✅ Category and store management
- ✅ Meal planning calendar operations
- ✅ Recipe-to-list integration with scaling

### Future Enhancements (Phase 3-4)
- ⏳ Recipe web import (schema.org parsing)
- ⏳ Barcode scanning integration
- ⏳ Online shopping integration (Instacart, etc.)
- ⏳ Location-based reminders
- ⏳ Price tracking and totals
- ⏳ Sharing and collaboration features
- ⏳ WebSocket real-time sync
- ⏳ Widget data providers
- ⏳ Bulk data export/import

## API Design Philosophy

This library follows these principles:

1. **Type Safety** - Strong typing with custom error types
2. **Ergonomic API** - Clean, intuitive method names matching domain concepts
3. **Comprehensive Coverage** - Implements the full feature set from the planning document
4. **Documentation** - Extensive examples and doc comments
5. **Modularity** - Organized into logical modules for different feature areas

## Error Handling

The library uses a custom `AnyListError` enum covering common failure modes:

```rust
pub enum AnyListError {
    AuthenticationFailed(String),
    NetworkError(String),
    InvalidResponse(String),
    NotFound(String),
    PermissionDenied(String),
    ProtobufError(String),
    Other(String),
}
```

All operations return `Result<T, AnyListError>` for consistent error handling.

## Credential Management

For reusing credentials without re-authenticating:

```rust
use anylist_rs::AnyListClient;

// Initial login
let client = AnyListClient::new("email@example.com", "password").await?;

// Save credentials (implement secure storage yourself)
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

## Limitations

- **Unofficial API** - This library reverse-engineers AnyList's private API
- **No Official Support** - Changes to AnyList's backend may break functionality
- **Premium Features** - Some operations require an AnyList Complete subscription
- **Rate Limiting** - Aggressive use may result in throttling

## Contributing

This library implements features from the comprehensive AnyList API planning document. Future contributions could include:

- WebSocket-based real-time sync
- Recipe web import with schema.org parsing
- Smart ingredient combining for multi-recipe meal plans
- Export functionality (JSON, CSV, PDF)
- Enhanced search and filtering

## License

This project is provided as-is for educational and personal use. AnyList is a trademark of Purple Cover, Inc.

## Acknowledgments

Built based on comprehensive research into AnyList's feature set, inspired by the unofficial npm package `codetheweb/anylist` and extensive analysis of the platform's capabilities across iOS, Android, web, and native apps.

## Disclaimer

This is an unofficial library and is not affiliated with, endorsed by, or connected to AnyList or Purple Cover, Inc. Use at your own risk.
