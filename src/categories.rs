use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbListCategory, PbListOperation, PbListOperationList,
    PbOperationMetadata,
};
use crate::utils::generate_id;
use prost::Message;

/// Represents a category for organizing list items
#[derive(Debug, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub sort_index: i32,
}

/// Represents a category group (category set)
#[derive(Debug, Clone)]
pub struct CategoryGroup {
    pub id: String,
    pub name: String,
    pub categories: Vec<Category>,
}

impl AnyListClient {
    /// Create a new category in a list
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
        let category_id = generate_id();
        let operation_id = generate_id();

        let new_category = PbListCategory {
            identifier: Some(category_id.clone()),
            logical_timestamp: Some(1),
            category_group_id: Some(category_group_id.to_string()),
            list_id: Some(list_id.to_string()),
            name: Some(name.to_string()),
            icon: None,
            system_category: None,
            sort_index: Some(0),
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("create-category".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::ListCategory as i32),
            }),
            list_id: Some(list_id.to_string()),
            updated_category: Some(new_category),
            ..Default::default()
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update-v2", buf).await?;

        Ok(Category {
            id: category_id,
            name: name.to_string(),
            icon: None,
            sort_index: 0,
        })
    }

    /// Delete a category
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list
    /// * `category_id` - The ID of the category to delete
    pub async fn delete_category(&self, list_id: &str, category_id: &str) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("remove-category".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::ListCategory as i32),
            }),
            list_id: Some(list_id.to_string()),
            original_value: Some(category_id.to_string()),
            ..Default::default()
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update-v2", buf).await?;
        Ok(())
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
        let operation_id = generate_id();

        let updated_category = PbListCategory {
            identifier: Some(category_id.to_string()),
            logical_timestamp: Some(1),
            category_group_id: Some(category_group_id.to_string()),
            list_id: Some(list_id.to_string()),
            name: Some(new_name.to_string()),
            icon: None,
            system_category: None,
            sort_index: Some(0),
        };

        let operation = PbListOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("set-category-name".to_string()),
                user_id: Some(self.user_id()),
                operation_class: Some(OperationClass::ListCategory as i32),
            }),
            list_id: Some(list_id.to_string()),
            updated_value: Some(new_name.to_string()),
            updated_category: Some(updated_category),
            ..Default::default()
        };

        let operation_list = PbListOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("data/shopping-lists/update-v2", buf).await?;
        Ok(())
    }
}
