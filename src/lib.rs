pub mod client;
pub mod error;
pub mod lists;
pub mod login;
pub mod items;
pub mod recipes;
pub mod categories;
pub mod stores;
pub mod collections;
pub mod meal_planning;
mod utils;

pub mod protobuf {
    pub mod anylist {
        include!(concat!(env!("OUT_DIR"), "/anylist.proto.rs"));
    }
}

// Re-export commonly used types
pub use client::AnyListClient;
pub use error::{AnyListError, Result};

// Re-export data structures
pub use lists::{List, ListItem};
pub use recipes::{Ingredient, Recipe};
pub use categories::{Category, CategoryGroup};
pub use stores::Store;
pub use collections::RecipeCollection;
pub use meal_planning::MealPlanEvent;
