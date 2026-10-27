use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::operations;
use crate::protobuf::anylist::PbListCategory;
use serde_derive::{Deserialize, Serialize};

/// Represents a category for organizing list items
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    id: String,
    name: String,
    icon: Option<String>,
    sort_index: i32,
}

impl Category {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn sort_index(&self) -> i32 {
        self.sort_index
    }
}

/// Represents a category group (category set)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryGroup {
    id: String,
    name: String,
    categories: Vec<Category>,
}

impl CategoryGroup {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories
    }
}

impl AnyListClient {
    /// Create a new category in a list
    ///
    /// Like the web client, the category goes last, before "Other".
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `category_group_id` - The ID of the category group
    /// * `name` - The name of the category
    pub async fn create_category(
        &self,
        list_id: &str,
        category_group_id: &str,
        name: &str,
    ) -> Result<Category> {
        let data = self.get_user_data().await?;
        let group = data
            .category_group(list_id, category_group_id)
            .ok_or_else(|| {
                AnyListError::NotFound(format!(
                    "Category group {} not found in list {}",
                    category_group_id, list_id
                ))
            })?;
        let created = operations::create_category(&mut self.ids(), group, name);
        self.send(created.requests).await?;
        Ok(category(&created.category))
    }

    /// Delete a category
    ///
    /// Like the web client, this also removes the list's categorization
    /// rules for the category and moves its items to the default category.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `category_id` - The ID of the category to delete
    pub async fn delete_category(&self, list_id: &str, category_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let list = data.list_context(list_id);
        let category = list.category(category_id).ok_or_else(|| {
            AnyListError::NotFound(format!(
                "Category {} not found in list {}",
                category_id, list_id
            ))
        })?;
        let items = data
            .shopping_list(list_id)
            .map(|l| l.items.as_slice())
            .unwrap_or_default();
        let requests = operations::delete_category(&mut self.ids(), list, category, items);
        self.send(requests).await
    }

    /// Rename a category
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `category_group_id` - The ID of the category group
    /// * `category_id` - The ID of the category
    /// * `new_name` - The new name for the category
    pub async fn rename_category(
        &self,
        list_id: &str,
        category_group_id: &str,
        category_id: &str,
        new_name: &str,
    ) -> Result<()> {
        let data = self.get_user_data().await?;
        let category = data
            .category_group(list_id, category_group_id)
            .and_then(|g| {
                g.categories
                    .iter()
                    .find(|c| c.identifier.as_deref() == Some(category_id))
            })
            .ok_or_else(|| {
                AnyListError::NotFound(format!(
                    "Category {} not found in list {}",
                    category_id, list_id
                ))
            })?;
        let requests = operations::rename_category(&mut self.ids(), category, new_name);
        self.send(requests).await
    }
}

fn category(category: &PbListCategory) -> Category {
    Category {
        id: category.identifier.clone().unwrap_or_default(),
        name: category.name.clone().unwrap_or_default(),
        icon: category.icon.clone(),
        sort_index: category.sort_index.unwrap_or_default(),
    }
}
