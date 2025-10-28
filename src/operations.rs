//! Pure operation builders for AnyList API protobuf messages.
//!
//! This module contains the "functional core" - pure functions that build
//! protobuf operation messages without side effects. The client methods
//! (imperative shell) call these builders with runtime values.
//!
//! This separation enables:
//! - Easy testing with deterministic inputs
//! - Snapshot testing of actual wire format
//! - Clear separation of concerns

use crate::protobuf::anylist::pb_operation_metadata::OperationClass;
use crate::protobuf::anylist::{
    PbListOperation, PbListOperationList, PbOperationMetadata, PbShoppingList, PbStore,
    PbStoreFilter,
};

// ============================================================================
// List Operations
// ============================================================================

/// Parameters for creating a shopping list operation
pub struct CreateListParams {
    pub list_id: String,
    pub operation_id: String,
    pub user_id: String,
    pub timestamp: f64,
    pub name: String,
}

/// Build a create-list operation (pure function)
pub fn build_create_list_operation(params: CreateListParams) -> PbListOperationList {
    let new_list = PbShoppingList {
        identifier: params.list_id.clone(),
        timestamp: Some(params.timestamp),
        name: Some(params.name),
        items: vec![],
        creator: Some(params.user_id.clone()),
        unusedattribute: vec![],
        shared_users: vec![],
        password: None,
        notification_locations: vec![],
        logical_clock_time: Some(1),
        built_in_alexa_list_type: None,
        allows_multiple_list_category_groups: Some(true),
        list_item_sort_order: Some(0),   // Manual
        new_list_item_position: Some(0), // Bottom
    };

    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("new-shopping-list".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        list: Some(new_list),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

/// Parameters for deleting a list
pub struct DeleteListParams {
    pub list_id: String,
    pub operation_id: String,
    pub user_id: String,
}

/// Build a delete-list operation (pure function)
pub fn build_delete_list_operation(params: DeleteListParams) -> PbListOperationList {
    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("delete-list".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

/// Parameters for renaming a list
pub struct RenameListParams {
    pub list_id: String,
    pub operation_id: String,
    pub user_id: String,
    pub timestamp: f64,
    pub old_name: String,
    pub new_name: String,
}

/// Build a rename-list operation (pure function)
pub fn build_rename_list_operation(params: RenameListParams) -> PbListOperationList {
    let updated_list = PbShoppingList {
        identifier: params.list_id.clone(),
        timestamp: Some(params.timestamp),
        name: Some(params.new_name),
        items: vec![],
        creator: Some(params.user_id.clone()),
        unusedattribute: vec![],
        shared_users: vec![],
        password: None,
        notification_locations: vec![],
        logical_clock_time: None,
        built_in_alexa_list_type: None,
        allows_multiple_list_category_groups: Some(true),
        list_item_sort_order: Some(0),
        new_list_item_position: Some(0),
    };

    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("rename-list".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        original_value: Some(params.old_name),
        list: Some(updated_list),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

// ============================================================================
// Store Operations
// ============================================================================

/// Parameters for removing store from items
pub struct RemoveStoreFromItemsParams {
    pub list_id: String,
    pub store_id: String,
    pub operation_id: String,
    pub user_id: String,
}

/// Build a remove-store-from-items operation (pure function)
pub fn build_remove_store_from_items_operation(
    params: RemoveStoreFromItemsParams,
) -> PbListOperationList {
    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("remove-store-id-from-all-items".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        updated_value: Some(params.store_id),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

/// Parameters for updating a store filter
pub struct UpdateStoreFilterParams {
    pub filter_id: String,
    pub list_id: String,
    pub filter_name: String,
    pub store_ids: Vec<String>,
    pub operation_id: String,
    pub user_id: String,
}

/// Build an update-store-filter operation (pure function)
pub fn build_update_store_filter_operation(
    params: UpdateStoreFilterParams,
) -> PbListOperationList {
    let pb_filter = PbStoreFilter {
        identifier: params.filter_id,
        logical_timestamp: None,
        list_id: Some(params.list_id.clone()),
        name: Some(params.filter_name),
        store_ids: params.store_ids,
        includes_unassigned_items: None,
        sort_index: None,
        list_category_group_id: None,
        shows_all_items: None,
    };

    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("update-store-filter".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        updated_store_filter: Some(pb_filter),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

/// Parameters for deleting a store filter
pub struct DeleteStoreFilterParams {
    pub filter_id: String,
    pub list_id: String,
    pub filter_name: String,
    pub operation_id: String,
    pub user_id: String,
}

/// Build a delete-store-filter operation (pure function)
pub fn build_delete_store_filter_operation(
    params: DeleteStoreFilterParams,
) -> PbListOperationList {
    let pb_filter = PbStoreFilter {
        identifier: params.filter_id,
        logical_timestamp: None,
        list_id: Some(params.list_id.clone()),
        name: Some(params.filter_name),
        store_ids: vec![],
        includes_unassigned_items: None,
        sort_index: None,
        list_category_group_id: None,
        shows_all_items: None,
    };

    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("delete-store-filter".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        updated_store_filter: Some(pb_filter),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

/// Parameters for deleting a store
pub struct DeleteStoreParams {
    pub store_id: String,
    pub store_name: Option<String>,
    pub list_id: String,
    pub operation_id: String,
    pub user_id: String,
}

/// Build a delete-store operation (pure function)
pub fn build_delete_store_operation(params: DeleteStoreParams) -> PbListOperationList {
    let pb_store = PbStore {
        identifier: params.store_id,
        name: params.store_name,
        sort_index: None,
        logical_timestamp: None,
        list_id: Some(params.list_id.clone()),
    };

    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("delete-store".to_string()),
            user_id: Some(params.user_id),
            operation_class: Some(OperationClass::Undefined as i32),
        }),
        list_id: Some(params.list_id),
        updated_store: Some(pb_store),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

// ============================================================================
// Item Operations
// ============================================================================

use crate::protobuf::anylist::{PbListItem, PbListItemCategoryAssignment};

/// Parameters for adding an item to a list
pub struct AddItemParams {
    pub item_id: String,
    pub list_id: String,
    pub operation_id: String,
    pub user_id: String,
    pub name: String,
    pub category: Option<String>,
    pub category_match_id: Option<String>,
    pub category_assignment: Option<CategoryAssignment>,
}

/// Category assignment for an item
pub struct CategoryAssignment {
    pub identifier: String,
    pub category_group_id: String,
    pub category_id: String,
}

/// Build an add-item operation (pure function)
pub fn build_add_item_operation(params: AddItemParams) -> PbListOperationList {
    let category_assignments = if let Some(assignment) = params.category_assignment {
        vec![PbListItemCategoryAssignment {
            identifier: Some(assignment.identifier),
            category_group_id: Some(assignment.category_group_id),
            category_id: Some(assignment.category_id),
        }]
    } else {
        vec![]
    };

    let pb_item = PbListItem {
        identifier: params.item_id.clone(),
        server_mod_time: None,
        list_id: Some(params.list_id.clone()),
        name: Some(params.name),
        quantity: None,
        details: None,
        checked: None,
        recipe_id: None,
        raw_ingredient: None,
        price_matchup_tag: None,
        price_id: None,
        category: params.category,
        user_id: Some(params.user_id.clone()),
        category_match_id: params.category_match_id,
        photo_ids: vec![],
        event_id: None,
        store_ids: vec![],
        manual_sort_index: None,
        prices: vec![],
        category_assignments,
    };

    let operation = PbListOperation {
        metadata: Some(PbOperationMetadata {
            operation_id: Some(params.operation_id),
            handler_id: Some("add-shopping-list-item".to_string()),
            user_id: Some(params.user_id),
            operation_class: None,
        }),
        list_id: Some(params.list_id),
        list_item_id: Some(params.item_id),
        list_item: Some(pb_item),
        ..Default::default()
    };

    PbListOperationList {
        operations: vec![operation],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    // Fixed test values for deterministic testing
    fn test_create_list_params() -> CreateListParams {
        CreateListParams {
            list_id: "test-list-abc123".to_string(),
            operation_id: "test-op-xyz789".to_string(),
            user_id: "test-user-456".to_string(),
            timestamp: 1700000000.0,
            name: "Groceries".to_string(),
        }
    }

    #[test]
    fn test_create_list_operation_snapshot() {
        let params = test_create_list_params();
        let operation_list = build_create_list_operation(params);

        // Encode to bytes
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();

        // Snapshot as hex string for readability
        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_delete_list_operation_snapshot() {
        let params = DeleteListParams {
            list_id: "test-list-abc123".to_string(),
            operation_id: "test-op-delete-1".to_string(),
            user_id: "test-user-456".to_string(),
        };

        let operation_list = build_delete_list_operation(params);
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();

        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_rename_list_operation_snapshot() {
        let params = RenameListParams {
            list_id: "test-list-abc123".to_string(),
            operation_id: "test-op-rename-1".to_string(),
            user_id: "test-user-456".to_string(),
            timestamp: 1700000000.0,
            old_name: "Old Name".to_string(),
            new_name: "New Name".to_string(),
        };

        let operation_list = build_rename_list_operation(params);
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();

        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_remove_store_uses_updated_value_field() {
        // Critical test: ensure we use updated_value not original_value
        let params = RemoveStoreFromItemsParams {
            list_id: "list-1".to_string(),
            store_id: "store-abc".to_string(),
            operation_id: "op-1".to_string(),
            user_id: "user-1".to_string(),
        };

        let operation_list = build_remove_store_from_items_operation(params);
        let op = &operation_list.operations[0];

        // Assert correct field is used
        assert_eq!(op.updated_value, Some("store-abc".to_string()));
        assert_eq!(op.original_value, None);

        // Also snapshot the bytes
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();
        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_update_store_filter_operation_snapshot() {
        let params = UpdateStoreFilterParams {
            filter_id: "filter-123".to_string(),
            list_id: "list-1".to_string(),
            filter_name: "Weekend Stores".to_string(),
            store_ids: vec!["store-a".to_string(), "store-b".to_string()],
            operation_id: "op-update-filter-1".to_string(),
            user_id: "user-1".to_string(),
        };

        let operation_list = build_update_store_filter_operation(params);
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();

        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_delete_store_filter_operation_snapshot() {
        let params = DeleteStoreFilterParams {
            filter_id: "filter-123".to_string(),
            list_id: "list-1".to_string(),
            filter_name: "Old Filter".to_string(),
            operation_id: "op-delete-filter-1".to_string(),
            user_id: "user-1".to_string(),
        };

        let operation_list = build_delete_store_filter_operation(params);
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();

        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_delete_store_uses_updated_store_field() {
        // Critical test: ensure we use updated_store with full PbStore object
        let params = DeleteStoreParams {
            store_id: "store-123".to_string(),
            store_name: Some("Costco".to_string()),
            list_id: "list-1".to_string(),
            operation_id: "op-delete-store-1".to_string(),
            user_id: "user-1".to_string(),
        };

        let operation_list = build_delete_store_operation(params);
        let op = &operation_list.operations[0];

        // Assert correct field is used
        assert!(op.updated_store.is_some());
        let store = op.updated_store.as_ref().unwrap();
        assert_eq!(store.identifier, "store-123");
        assert_eq!(store.name, Some("Costco".to_string()));

        // Snapshot the bytes
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();
        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_delete_store_without_name() {
        // Test deletion without store name (as used in actual client)
        let params = DeleteStoreParams {
            store_id: "store-456".to_string(),
            store_name: None,
            list_id: "list-2".to_string(),
            operation_id: "op-delete-store-2".to_string(),
            user_id: "user-2".to_string(),
        };

        let operation_list = build_delete_store_operation(params);
        let op = &operation_list.operations[0];

        assert!(op.updated_store.is_some());
        let store = op.updated_store.as_ref().unwrap();
        assert_eq!(store.identifier, "store-456");
        assert_eq!(store.name, None);

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();
        insta::assert_snapshot!(hex::encode(&buf));
    }

    #[test]
    fn test_webapp_add_shopping_list_item_2025_10_28() {
        let params = AddItemParams {
            item_id: "d83d31d5d48d4b7591b05c7efb725a46".to_string(),
            list_id: "58ec2be417b247d7a9edc2d9d66889ab".to_string(),
            operation_id: "0da34b3d00f54ce1bd6fd501ddf62f99".to_string(),
            user_id: "cda21b0078644a01b640c84d3d74187e".to_string(),
            name: "nice new things".to_string(),
            category: Some("other".to_string()),
            category_match_id: Some("other".to_string()),
            category_assignment: Some(CategoryAssignment {
                identifier: "47868d70669a5a078f8bc4e40dc07cab".to_string(),
                category_group_id: "65564675a0de5a5fa6a69272df260fcc".to_string(),
                category_id: "f962e619ab90479894c9dcc291a7f103".to_string(),
            }),
        };

        let operation_list = build_add_item_operation(params);
        let mut buf = Vec::new();
        operation_list.encode(&mut buf).unwrap();

        // Should match webapp exactly
        insta::assert_snapshot!(hex::encode(&buf));
    }
}
