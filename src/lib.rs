//! Unofficial crate for interacting with the grocery list management app
//! [AnyList](https://www.anylist.com/)'s API.
//!
//! ```
//! use anylist_rs::{AnyListClient, Ingredient, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Authenticate with email and password
//!     let client = AnyListClient::login("your-email@example.com", "your-password").await?;
//!
//!    // Get all lists
//!    let lists = client.get_lists().await?;
//!    for list in &lists {
//!        println!("List: {} ({} items)", list.name, list.items.len());
//!    }
//!
//!    // Create a new list
//!    let grocery_list = client.create_list("Weekly Groceries").await?;
//!    println!("Created list: {}", grocery_list.name);
//!
//!    // Add items to the list
//!    client.add_item(&grocery_list.id, "Milk").await?;
//!    client.add_item_with_details(
//!        &grocery_list.id,
//!        "Apples",
//!        Some("2 lbs"),
//!        Some("Organic if possible"),
//!        Some("Produce")
//!    ).await?;
//!
//!    Ok(())
//! }
//! ```

pub mod categories;
pub mod client;
pub mod collections;
pub mod error;
pub mod items;
pub mod lists;
pub mod login;
pub mod meal_planning;
pub mod recipes;
pub mod stores;
mod utils;

mod protobuf {
    pub mod anylist {
        include!(concat!(env!("OUT_DIR"), "/anylist.proto.rs"));
    }
}

// Re-export commonly used types
pub use client::{AnyListClient, AuthEvent, SavedTokens};
pub use error::{AnyListError, Result};

// Re-export data structures
pub use categories::{Category, CategoryGroup};
pub use collections::RecipeCollection;
pub use lists::{List, ListItem};
pub use meal_planning::MealPlanEvent;
pub use recipes::{Ingredient, Recipe};
pub use stores::Store;
