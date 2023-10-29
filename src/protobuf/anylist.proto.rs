#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tag {
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "7")]
    pub display_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub image_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "6")]
    pub implied_tag_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "3")]
    pub search_terms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "2")]
    pub product_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub price_stats: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(enumeration = "tag::TagType", optional, tag = "5")]
    pub tag_type: ::core::option::Option<i32>,
}
/// Nested message and enum types in `Tag`.
pub mod tag {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum TagType {
        Generic = 0,
        Product = 1,
        Category = 2,
        Attribute = 3,
    }
    impl TagType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                TagType::Generic => "TAG_TYPE_GENERIC",
                TagType::Product => "TAG_TYPE_PRODUCT",
                TagType::Category => "TAG_TYPE_CATEGORY",
                TagType::Attribute => "TAG_TYPE_ATTRIBUTE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TAG_TYPE_GENERIC" => Some(Self::Generic),
                "TAG_TYPE_PRODUCT" => Some(Self::Product),
                "TAG_TYPE_CATEGORY" => Some(Self::Category),
                "TAG_TYPE_ATTRIBUTE" => Some(Self::Attribute),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserListData {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, repeated, tag = "3")]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "4")]
    pub user_ids_timestamp: ::core::option::Option<f64>,
    #[prost(string, repeated, tag = "5")]
    pub list_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "6")]
    pub list_ids_timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "7")]
    pub root_folder_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "8")]
    pub root_folder_id_timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "9")]
    pub categorized_items_timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "10")]
    pub categorized_items_require_refresh_timestamp: ::core::option::Option<f64>,
    #[prost(bool, optional, tag = "11")]
    pub has_migrated_list_ordering: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShoppingList {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub items: ::prost::alloc::vec::Vec<ListItem>,
    #[prost(string, optional, tag = "5")]
    pub creator: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "6")]
    pub unusedattribute: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "7")]
    pub shared_users: ::prost::alloc::vec::Vec<PbEmailUserIdPair>,
    #[prost(string, optional, tag = "8")]
    pub password: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "9")]
    pub notification_locations: ::prost::alloc::vec::Vec<PbNotificationLocation>,
    #[prost(uint64, optional, tag = "10")]
    pub logical_clock_time: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "11")]
    pub built_in_alexa_list_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "16")]
    pub allows_multiple_list_category_groups: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "17")]
    pub list_item_sort_order: ::core::option::Option<i32>,
    #[prost(int32, optional, tag = "18")]
    pub new_list_item_position: ::core::option::Option<i32>,
}
/// Nested message and enum types in `ShoppingList`.
pub mod shopping_list {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum ListItemSortOrder {
        Manual = 0,
        Alphabetical = 1,
    }
    impl ListItemSortOrder {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ListItemSortOrder::Manual => "Manual",
                ListItemSortOrder::Alphabetical => "Alphabetical",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "Manual" => Some(Self::Manual),
                "Alphabetical" => Some(Self::Alphabetical),
                _ => None,
            }
        }
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum NewListItemPosition {
        Bottom = 0,
        Top = 1,
    }
    impl NewListItemPosition {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                NewListItemPosition::Bottom => "Bottom",
                NewListItemPosition::Top => "Top",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "Bottom" => Some(Self::Bottom),
                "Top" => Some(Self::Top),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListItem {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub server_mod_time: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "18")]
    pub quantity: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub details: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "6")]
    pub checked: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "7")]
    pub recipe_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub raw_ingredient: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub price_matchup_tag: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "10")]
    pub price_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "11")]
    pub category: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "12")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "13")]
    pub category_match_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "14")]
    pub photo_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "15")]
    pub event_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "16")]
    pub store_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "19")]
    pub prices: ::prost::alloc::vec::Vec<PbItemPrice>,
    #[prost(message, repeated, tag = "20")]
    pub category_assignments: ::prost::alloc::vec::Vec<PbListItemCategoryAssignment>,
    #[prost(int32, optional, tag = "17")]
    pub manual_sort_index: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbItemPrice {
    #[prost(double, optional, tag = "2")]
    pub amount: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub details: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub store_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub date: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFolderItem {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(int32, optional, tag = "2")]
    pub item_type: ::core::option::Option<i32>,
}
/// Nested message and enum types in `PBListFolderItem`.
pub mod pb_list_folder_item {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum ItemType {
        ListType = 0,
        FolderType = 1,
    }
    impl ItemType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ItemType::ListType => "ListType",
                ItemType::FolderType => "FolderType",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ListType" => Some(Self::ListType),
                "FolderType" => Some(Self::FolderType),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFolderSettings {
    #[prost(int32, optional, tag = "1")]
    pub lists_sort_order: ::core::option::Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub folder_sort_position: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "3")]
    pub folder_hex_color: ::core::option::Option<::prost::alloc::string::String>,
}
/// Nested message and enum types in `PBListFolderSettings`.
pub mod pb_list_folder_settings {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum SortOrder {
        ManualSortOrder = 0,
        AlphabeticalSortOrder = 1,
    }
    impl SortOrder {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                SortOrder::ManualSortOrder => "ManualSortOrder",
                SortOrder::AlphabeticalSortOrder => "AlphabeticalSortOrder",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ManualSortOrder" => Some(Self::ManualSortOrder),
                "AlphabeticalSortOrder" => Some(Self::AlphabeticalSortOrder),
                _ => None,
            }
        }
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum FolderSortPosition {
        AfterLists = 0,
        BeforeLists = 1,
        WithLists = 2,
    }
    impl FolderSortPosition {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                FolderSortPosition::AfterLists => "FolderSortPositionAfterLists",
                FolderSortPosition::BeforeLists => "FolderSortPositionBeforeLists",
                FolderSortPosition::WithLists => "FolderSortPositionWithLists",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "FolderSortPositionAfterLists" => Some(Self::AfterLists),
                "FolderSortPositionBeforeLists" => Some(Self::BeforeLists),
                "FolderSortPositionWithLists" => Some(Self::WithLists),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFolder {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub items: ::prost::alloc::vec::Vec<PbListFolderItem>,
    #[prost(message, optional, tag = "5")]
    pub folder_settings: ::core::option::Option<PbListFolderSettings>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFoldersResponse {
    #[prost(string, optional, tag = "1")]
    pub list_data_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub root_folder_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "3")]
    pub includes_all_folders: ::core::option::Option<bool>,
    #[prost(message, repeated, tag = "4")]
    pub list_folders: ::prost::alloc::vec::Vec<PbListFolder>,
    #[prost(string, repeated, tag = "5")]
    pub deleted_folder_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "6")]
    pub has_migrated_list_ordering: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFolderTimestamps {
    #[prost(string, optional, tag = "1")]
    pub root_folder_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub folder_timestamps: ::prost::alloc::vec::Vec<PbTimestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListCategoryGroupResponse {
    #[prost(message, optional, tag = "1")]
    pub category_group: ::core::option::Option<PbListCategoryGroup>,
    #[prost(string, repeated, tag = "2")]
    pub deleted_category_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShoppingListsResponse {
    #[prost(message, repeated, tag = "1")]
    pub new_lists: ::prost::alloc::vec::Vec<ShoppingList>,
    #[prost(message, repeated, tag = "2")]
    pub modified_lists: ::prost::alloc::vec::Vec<ShoppingList>,
    #[prost(string, repeated, tag = "3")]
    pub unmodified_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub unknown_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub ordered_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "6")]
    pub list_responses: ::prost::alloc::vec::Vec<PbListResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListResponse {
    #[prost(string, optional, tag = "1")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "2")]
    pub is_full_sync: ::core::option::Option<bool>,
    #[prost(uint64, optional, tag = "3")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(message, repeated, tag = "7")]
    pub category_group_responses: ::prost::alloc::vec::Vec<PbListCategoryGroupResponse>,
    #[prost(string, repeated, tag = "8")]
    pub deleted_category_group_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "13")]
    pub categorization_rules: ::prost::alloc::vec::Vec<PbListCategorizationRule>,
    #[prost(string, repeated, tag = "14")]
    pub deleted_categorization_rule_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "9")]
    pub stores: ::prost::alloc::vec::Vec<PbStore>,
    #[prost(string, repeated, tag = "10")]
    pub deleted_store_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "11")]
    pub store_filters: ::prost::alloc::vec::Vec<PbStoreFilter>,
    #[prost(string, repeated, tag = "12")]
    pub deleted_store_filter_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StarterList {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub items: ::prost::alloc::vec::Vec<ListItem>,
    #[prost(string, optional, tag = "5")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "7")]
    pub starter_list_type: ::core::option::Option<i32>,
}
/// Nested message and enum types in `StarterList`.
pub mod starter_list {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Type {
        UserType = 0,
        RecentItemsType = 1,
        FavoriteItemsType = 2,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::UserType => "UserType",
                Type::RecentItemsType => "RecentItemsType",
                Type::FavoriteItemsType => "FavoriteItemsType",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UserType" => Some(Self::UserType),
                "RecentItemsType" => Some(Self::RecentItemsType),
                "FavoriteItemsType" => Some(Self::FavoriteItemsType),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StarterListResponse {
    #[prost(message, optional, tag = "1")]
    pub starter_list: ::core::option::Option<StarterList>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StarterListBatchResponse {
    #[prost(message, repeated, tag = "1")]
    pub list_responses: ::prost::alloc::vec::Vec<StarterListResponse>,
    #[prost(bool, optional, tag = "2")]
    pub includes_all_lists: ::core::option::Option<bool>,
    #[prost(string, repeated, tag = "3")]
    pub unknown_list_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StarterListsResponseV2 {
    #[prost(message, optional, tag = "1")]
    pub user_lists_response: ::core::option::Option<StarterListBatchResponse>,
    #[prost(message, optional, tag = "2")]
    pub recent_item_lists_response: ::core::option::Option<StarterListBatchResponse>,
    #[prost(message, optional, tag = "3")]
    pub favorite_item_lists_response: ::core::option::Option<StarterListBatchResponse>,
    #[prost(bool, optional, tag = "4")]
    pub has_migrated_user_favorites: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StarterListsResponse {
    #[prost(message, repeated, tag = "1")]
    pub new_lists: ::prost::alloc::vec::Vec<StarterList>,
    #[prost(message, repeated, tag = "2")]
    pub modified_lists: ::prost::alloc::vec::Vec<StarterList>,
    #[prost(string, repeated, tag = "3")]
    pub unmodified_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub unknown_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub ordered_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbStore {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "5")]
    pub sort_index: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbStoreFilter {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub store_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "6")]
    pub includes_unassigned_items: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "7")]
    pub sort_index: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "8")]
    pub list_category_group_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "9")]
    pub shows_all_items: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListCategory {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub category_group_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub icon: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub system_category: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "9")]
    pub sort_index: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListCategoryGroup {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "5")]
    pub categories: ::prost::alloc::vec::Vec<PbListCategory>,
    #[prost(string, optional, tag = "8")]
    pub default_category_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "6")]
    pub categories_logical_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "7")]
    pub deleted_categories_logical_timestamp: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListCategorizationRule {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub category_group_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub item_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub category_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListCategorizationRuleList {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub categorization_rules: ::prost::alloc::vec::Vec<PbListCategorizationRule>,
    #[prost(uint64, optional, tag = "5")]
    pub categorization_rules_logical_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "6")]
    pub deleted_categorization_rules_logical_timestamp: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListItemCategoryAssignment {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub category_group_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub category_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipe {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub icon: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub note: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub source_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub source_url: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "8")]
    pub ingredients: ::prost::alloc::vec::Vec<PbIngredient>,
    #[prost(string, repeated, tag = "9")]
    pub preparation_steps: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "11")]
    pub photo_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "12")]
    pub ad_campaign_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "13")]
    pub photo_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "14")]
    pub scale_factor: ::core::option::Option<f64>,
    #[prost(int32, optional, tag = "15")]
    pub rating: ::core::option::Option<i32>,
    #[prost(double, optional, tag = "16")]
    pub creation_timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "17")]
    pub nutritional_info: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "18")]
    pub cook_time: ::core::option::Option<i32>,
    #[prost(int32, optional, tag = "19")]
    pub prep_time: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "20")]
    pub servings: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "21")]
    pub paprika_identifier: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbIngredient {
    #[prost(string, optional, tag = "1")]
    pub raw_ingredient: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub quantity: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub note: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAndroidEditableIngredient {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "2")]
    pub ingredient: ::core::option::Option<PbIngredient>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAndroidEditableIngredientList {
    #[prost(message, repeated, tag = "1")]
    pub editable_ingredients: ::prost::alloc::vec::Vec<PbAndroidEditableIngredient>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeCollectionSettings {
    #[prost(int32, optional, tag = "1")]
    pub recipes_sort_order: ::core::option::Option<i32>,
    #[prost(bool, optional, tag = "2")]
    pub show_only_recipes_with_no_collection: ::core::option::Option<bool>,
}
/// Nested message and enum types in `PBRecipeCollectionSettings`.
pub mod pb_recipe_collection_settings {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum SortOrder {
        ManualSortOrder = 0,
        AlphabeticalSortOrder = 1,
        RatingSortOrder = 2,
        DateCreatedSortOrder = 3,
        PrepTimeSortOrder = 4,
        CookTimeSortOrder = 5,
    }
    impl SortOrder {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                SortOrder::ManualSortOrder => "ManualSortOrder",
                SortOrder::AlphabeticalSortOrder => "AlphabeticalSortOrder",
                SortOrder::RatingSortOrder => "RatingSortOrder",
                SortOrder::DateCreatedSortOrder => "DateCreatedSortOrder",
                SortOrder::PrepTimeSortOrder => "PrepTimeSortOrder",
                SortOrder::CookTimeSortOrder => "CookTimeSortOrder",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ManualSortOrder" => Some(Self::ManualSortOrder),
                "AlphabeticalSortOrder" => Some(Self::AlphabeticalSortOrder),
                "RatingSortOrder" => Some(Self::RatingSortOrder),
                "DateCreatedSortOrder" => Some(Self::DateCreatedSortOrder),
                "PrepTimeSortOrder" => Some(Self::PrepTimeSortOrder),
                "CookTimeSortOrder" => Some(Self::CookTimeSortOrder),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeCollection {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub recipe_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "5")]
    pub collection_settings: ::core::option::Option<PbRecipeCollectionSettings>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserRecipeData {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "3")]
    pub recipes_timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "4")]
    pub all_recipes_timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "5")]
    pub recipe_collections_timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "6")]
    pub recipe_collection_ids_timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "7")]
    pub all_recipes_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "8")]
    pub recipe_collection_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "9")]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "10")]
    pub user_ids_timestamp: ::core::option::Option<f64>,
    #[prost(bool, optional, tag = "11")]
    pub has_imported_punchfork_recipes: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "12")]
    pub meal_planning_calendar_id: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeLinkRequest {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub requesting_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub requesting_email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub requesting_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub confirming_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub confirming_email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub confirming_name: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeLinkRequestResponse {
    #[prost(int32, optional, tag = "1")]
    pub status_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "2")]
    pub recipe_data_response: ::core::option::Option<PbRecipeDataResponse>,
    #[prost(string, optional, tag = "3")]
    pub error_title: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub error_message: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeDataResponse {
    #[prost(double, optional, tag = "1")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(message, optional, tag = "2")]
    pub all_recipes_collection: ::core::option::Option<PbRecipeCollection>,
    #[prost(message, repeated, tag = "3")]
    pub recipes: ::prost::alloc::vec::Vec<PbRecipe>,
    #[prost(string, repeated, tag = "4")]
    pub recipe_collection_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "5")]
    pub recipe_collections: ::prost::alloc::vec::Vec<PbRecipeCollection>,
    #[prost(message, repeated, tag = "6")]
    pub pending_recipe_link_requests: ::prost::alloc::vec::Vec<PbRecipeLinkRequest>,
    #[prost(message, repeated, tag = "7")]
    pub recipe_link_requests_to_confirm: ::prost::alloc::vec::Vec<PbRecipeLinkRequest>,
    #[prost(message, repeated, tag = "8")]
    pub linked_users: ::prost::alloc::vec::Vec<PbEmailUserIdPair>,
    #[prost(string, optional, tag = "9")]
    pub recipe_data_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "10")]
    pub has_imported_punchfork_recipes: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "11")]
    pub includes_recipe_collection_ids: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub recipe_data_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "3")]
    pub recipe: ::core::option::Option<PbRecipe>,
    #[prost(message, optional, tag = "4")]
    pub recipe_collection: ::core::option::Option<PbRecipeCollection>,
    #[prost(message, optional, tag = "5")]
    pub recipe_link_request: ::core::option::Option<PbRecipeLinkRequest>,
    #[prost(string, repeated, tag = "6")]
    pub recipe_collection_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "7")]
    pub recipes: ::prost::alloc::vec::Vec<PbRecipe>,
    #[prost(bool, optional, tag = "8")]
    pub is_new_recipe_from_web_import: ::core::option::Option<bool>,
    #[prost(string, repeated, tag = "9")]
    pub recipe_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbRecipeOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeList {
    #[prost(message, repeated, tag = "1")]
    pub recipes: ::prost::alloc::vec::Vec<PbRecipe>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeWebImportResponse {
    #[prost(int32, optional, tag = "1")]
    pub status_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "2")]
    pub recipe: ::core::option::Option<PbRecipe>,
    #[prost(bool, optional, tag = "3")]
    pub is_premium_user: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "4")]
    pub site_specific_help_text: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "5")]
    pub free_recipe_imports_remaining_count: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCalendar {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub logical_clock_time: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCalendarEvent {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub calendar_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub date: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub title: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub details: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub recipe_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub label_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "9")]
    pub order_added_sort_index: ::core::option::Option<i32>,
    #[prost(double, optional, tag = "10")]
    pub recipe_scale_factor: ::core::option::Option<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCalendarLabel {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub calendar_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "6")]
    pub sort_index: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCalendarResponse {
    #[prost(string, required, tag = "1")]
    pub calendar_id: ::prost::alloc::string::String,
    #[prost(bool, optional, tag = "2")]
    pub is_full_sync: ::core::option::Option<bool>,
    #[prost(uint64, optional, tag = "3")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(message, repeated, tag = "4")]
    pub events: ::prost::alloc::vec::Vec<PbCalendarEvent>,
    #[prost(string, repeated, tag = "5")]
    pub deleted_event_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "6")]
    pub labels: ::prost::alloc::vec::Vec<PbCalendarLabel>,
    #[prost(string, repeated, tag = "7")]
    pub deleted_label_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCalendarOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub calendar_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "3")]
    pub updated_event: ::core::option::Option<PbCalendarEvent>,
    #[prost(message, optional, tag = "4")]
    pub original_event: ::core::option::Option<PbCalendarEvent>,
    #[prost(message, optional, tag = "5")]
    pub updated_label: ::core::option::Option<PbCalendarLabel>,
    #[prost(message, optional, tag = "6")]
    pub original_label: ::core::option::Option<PbCalendarLabel>,
    #[prost(string, repeated, tag = "7")]
    pub sorted_label_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "8")]
    pub event_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "9")]
    pub updated_events: ::prost::alloc::vec::Vec<PbCalendarEvent>,
    #[prost(message, repeated, tag = "10")]
    pub original_events: ::prost::alloc::vec::Vec<PbCalendarEvent>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCalendarOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbCalendarOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbOperationMetadata {
    #[prost(string, optional, tag = "1")]
    pub operation_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub handler_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "4")]
    pub operation_class: ::core::option::Option<i32>,
}
/// Nested message and enum types in `PBOperationMetadata`.
pub mod pb_operation_metadata {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum OperationClass {
        UndefinedOperation = 0,
        StoreOperation = 1,
        StoreFilterOperation = 2,
        ListCategoryOperation = 3,
        ListCategoryGroupOperation = 4,
        ListCategorizationRuleOperation = 5,
    }
    impl OperationClass {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                OperationClass::UndefinedOperation => "UndefinedOperation",
                OperationClass::StoreOperation => "StoreOperation",
                OperationClass::StoreFilterOperation => "StoreFilterOperation",
                OperationClass::ListCategoryOperation => "ListCategoryOperation",
                OperationClass::ListCategoryGroupOperation => {
                    "ListCategoryGroupOperation"
                }
                OperationClass::ListCategorizationRuleOperation => {
                    "ListCategorizationRuleOperation"
                }
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UndefinedOperation" => Some(Self::UndefinedOperation),
                "StoreOperation" => Some(Self::StoreOperation),
                "StoreFilterOperation" => Some(Self::StoreFilterOperation),
                "ListCategoryOperation" => Some(Self::ListCategoryOperation),
                "ListCategoryGroupOperation" => Some(Self::ListCategoryGroupOperation),
                "ListCategorizationRuleOperation" => {
                    Some(Self::ListCategorizationRuleOperation)
                }
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbFavoriteProductOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub product_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbFavoriteProductOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbFavoriteProductOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbSavedRecipeOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub recipe_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbSavedRecipeOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbSavedRecipeOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbOrderedShoppingListIDsOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, repeated, tag = "2")]
    pub ordered_list_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbOrderedShoppingListIDsOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbOrderedShoppingListIDsOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbOrderedStarterListIDsOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, repeated, tag = "2")]
    pub ordered_list_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbOrderedStarterListIDsOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbOrderedStarterListIDsOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub list_item_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub updated_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub original_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "6")]
    pub list_item: ::core::option::Option<ListItem>,
    #[prost(message, optional, tag = "7")]
    pub list: ::core::option::Option<ShoppingList>,
    #[prost(string, optional, tag = "8")]
    pub list_folder_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "9")]
    pub notification_location: ::core::option::Option<PbNotificationLocation>,
    #[prost(message, optional, tag = "10")]
    pub updated_store: ::core::option::Option<PbStore>,
    #[prost(message, optional, tag = "11")]
    pub original_store: ::core::option::Option<PbStore>,
    #[prost(string, repeated, tag = "12")]
    pub sorted_store_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "13")]
    pub updated_store_filter: ::core::option::Option<PbStoreFilter>,
    #[prost(message, optional, tag = "14")]
    pub original_store_filter: ::core::option::Option<PbStoreFilter>,
    #[prost(string, repeated, tag = "15")]
    pub sorted_store_filter_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, optional, tag = "16")]
    pub item_price: ::core::option::Option<PbItemPrice>,
    #[prost(message, optional, tag = "17")]
    pub updated_category: ::core::option::Option<PbListCategory>,
    #[prost(message, optional, tag = "18")]
    pub original_category: ::core::option::Option<PbListCategory>,
    #[prost(message, optional, tag = "19")]
    pub updated_category_group: ::core::option::Option<PbListCategoryGroup>,
    #[prost(message, optional, tag = "20")]
    pub original_category_group: ::core::option::Option<PbListCategoryGroup>,
    #[prost(message, optional, tag = "21")]
    pub updated_categorization_rule: ::core::option::Option<PbListCategorizationRule>,
    #[prost(message, optional, tag = "22")]
    pub original_categorization_rule: ::core::option::Option<PbListCategorizationRule>,
    #[prost(message, repeated, tag = "23")]
    pub updated_categorization_rules: ::prost::alloc::vec::Vec<PbListCategorizationRule>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbListOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbShareListOperationResponse {
    #[prost(message, optional, tag = "1")]
    pub shared_user: ::core::option::Option<PbEmailUserIdPair>,
    #[prost(double, optional, tag = "2")]
    pub original_list_timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "3")]
    pub updated_list_timestamp: ::core::option::Option<f64>,
    #[prost(int32, optional, tag = "4")]
    pub status_code: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "5")]
    pub error_title: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub error_message: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFolderOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub list_data_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "3")]
    pub list_folder: ::core::option::Option<PbListFolder>,
    #[prost(message, repeated, tag = "4")]
    pub folder_items: ::prost::alloc::vec::Vec<PbListFolderItem>,
    #[prost(string, optional, tag = "5")]
    pub original_parent_folder_id: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(string, optional, tag = "6")]
    pub updated_parent_folder_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListFolderOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbListFolderOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbStarterListOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(string, optional, tag = "2")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub list_item_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub updated_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub original_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "6")]
    pub list_item: ::core::option::Option<ListItem>,
    #[prost(message, optional, tag = "7")]
    pub list: ::core::option::Option<StarterList>,
    #[prost(message, optional, tag = "8")]
    pub item_price: ::core::option::Option<PbItemPrice>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbStarterListOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbStarterListOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCategorizeItemOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(message, optional, tag = "2")]
    pub list_item: ::core::option::Option<ListItem>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCategorizeItemOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbCategorizeItemOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCategorizedItemsList {
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, repeated, tag = "2")]
    pub categorized_items: ::prost::alloc::vec::Vec<ListItem>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCategoryOrdering {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "3")]
    pub categories: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListSettings {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "4")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(bool, optional, tag = "5")]
    pub should_hide_categories: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "6")]
    pub selected_category_ordering: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "7")]
    pub category_orderings: ::prost::alloc::vec::Vec<PbCategoryOrdering>,
    #[prost(bool, optional, tag = "8")]
    pub generic_grocery_autocomplete_enabled: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "9")]
    pub list_item_sort_order: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "10")]
    pub category_grouping_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "11")]
    pub should_remember_item_categories: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "12")]
    pub favorites_autocomplete_enabled: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "13")]
    pub recent_items_autocomplete_enabled: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "14")]
    pub should_hide_completed_items: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "15")]
    pub list_color_type: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "16")]
    pub list_theme_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "17")]
    pub custom_theme: ::core::option::Option<PbListTheme>,
    #[prost(string, optional, tag = "18")]
    pub badge_mode: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "19")]
    pub location_notifications_enabled: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "20")]
    pub store_filter_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "21")]
    pub should_hide_store_names: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "22")]
    pub should_hide_running_totals: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "23")]
    pub should_hide_prices: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "24")]
    pub left_running_total_type: ::core::option::Option<i32>,
    #[prost(int32, optional, tag = "25")]
    pub right_running_total_type: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "26")]
    pub linked_alexa_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "27")]
    pub list_category_group_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "28")]
    pub migration_list_category_group_id_for_new_list: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(bool, optional, tag = "29")]
    pub should_show_shared_list_category_order_hint_banner: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "30")]
    pub linked_google_assistant_list_id: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListSettingsList {
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, repeated, tag = "2")]
    pub settings: ::prost::alloc::vec::Vec<PbListSettings>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListSettingsOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(message, optional, tag = "2")]
    pub updated_settings: ::core::option::Option<PbListSettings>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListSettingsOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbListSettingsOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListTheme {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub font_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub banner_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub background_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub background_texture: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub item_name_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "10")]
    pub item_details_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "11")]
    pub control_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "12")]
    pub separator_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "13")]
    pub navigation_bar_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "14")]
    pub cell_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "15")]
    pub cell_texture: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "16")]
    pub table_hex_color: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "17")]
    pub table_texture: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "18")]
    pub background_image: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "19")]
    pub selection_hex_color: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbListThemeList {
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, repeated, tag = "2")]
    pub themes: ::prost::alloc::vec::Vec<PbListTheme>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbMobileAppSettings {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "3")]
    pub default_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub cross_off_gesture: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub lists_sort_order: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub starter_lists_sort_order: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "7")]
    pub reminders_app_import_enabled: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "8")]
    pub app_badge_mode: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "9")]
    pub should_auto_import_reminders: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "10")]
    pub should_prevent_screen_autolock: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "11")]
    pub prompt_to_load_photos_over_cellular_data: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "12")]
    pub list_id_for_recipe_ingredients: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(string, optional, tag = "13")]
    pub web_selected_recipe_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "14")]
    pub web_selected_recipe_collection_id: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(string, optional, tag = "15")]
    pub web_selected_tab_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "16")]
    pub web_selected_list_folder_path: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(int32, optional, tag = "17")]
    pub web_selected_meal_plan_tab: ::core::option::Option<i32>,
    #[prost(bool, optional, tag = "18")]
    pub web_has_hidden_stores_and_filters_help: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "22")]
    pub web_has_hidden_item_prices_help: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "23")]
    pub web_decimal_separator: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "24")]
    pub web_currency_code: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "25")]
    pub web_currency_symbol: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "19")]
    pub hint_banner_display_stats: ::prost::alloc::vec::Vec<PbHintBannerDisplayStats>,
    #[prost(message, optional, tag = "20")]
    pub web_selected_recipe_collection_settings_override: ::core::option::Option<
        PbRecipeCollectionSettings,
    >,
    #[prost(bool, optional, tag = "21")]
    pub should_use_metric_units: ::core::option::Option<bool>,
    #[prost(message, repeated, tag = "26")]
    pub unlinked_alexa_lists: ::prost::alloc::vec::Vec<PbAlexaList>,
    #[prost(bool, optional, tag = "27")]
    pub alexa_skill_has_list_read_permission: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "28")]
    pub alexa_skill_has_list_write_permission: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "29")]
    pub is_account_linked_to_alexa_skill: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "30")]
    pub alexa_api_endpoint: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "31")]
    pub alexa_skill_only_supports_built_in_lists: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "32")]
    pub has_migrated_user_categories_to_list_categories: ::core::option::Option<bool>,
    #[prost(message, repeated, tag = "33")]
    pub unlinked_google_assistant_lists: ::prost::alloc::vec::Vec<PbGoogleAssistantList>,
    #[prost(bool, optional, tag = "34")]
    pub is_account_linked_to_google_assistant: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "35")]
    pub is_active_google_assistant_provider: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "36")]
    pub client_has_shown_google_assistant_onboarding: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbHintBannerDisplayStats {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, repeated, packed = "false", tag = "2")]
    pub display_timestamps: ::prost::alloc::vec::Vec<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbMobileAppSettingsOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(message, optional, tag = "2")]
    pub updated_settings: ::core::option::Option<PbMobileAppSettings>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbMobileAppSettingsOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbMobileAppSettingsOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserCategory {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub icon: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub system_category: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub category_match_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "7")]
    pub from_shared_list: ::core::option::Option<bool>,
    #[prost(double, optional, tag = "8")]
    pub timestamp: ::core::option::Option<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbCategoryGrouping {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "4")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "5")]
    pub sharing_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "6")]
    pub category_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "7")]
    pub should_hide_from_browse_list_category_groups_screen: ::core::option::Option<
        bool,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserCategoryData {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "3")]
    pub requires_refresh_timestamp: ::core::option::Option<f64>,
    #[prost(message, repeated, tag = "4")]
    pub categories: ::prost::alloc::vec::Vec<PbUserCategory>,
    #[prost(message, repeated, tag = "5")]
    pub groupings: ::prost::alloc::vec::Vec<PbCategoryGrouping>,
    #[prost(bool, optional, tag = "6")]
    pub has_migrated_category_orderings: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserCategoryOperation {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<PbOperationMetadata>,
    #[prost(message, optional, tag = "2")]
    pub category: ::core::option::Option<PbUserCategory>,
    #[prost(message, optional, tag = "3")]
    pub grouping: ::core::option::Option<PbCategoryGrouping>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserCategoryOperationList {
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<PbUserCategoryOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbTimestamp {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "2")]
    pub timestamp: ::core::option::Option<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbTimestampList {
    #[prost(message, repeated, tag = "1")]
    pub timestamps: ::prost::alloc::vec::Vec<PbTimestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbLogicalTimestamp {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbLogicalTimestampList {
    #[prost(message, repeated, tag = "1")]
    pub timestamps: ::prost::alloc::vec::Vec<PbLogicalTimestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbEditOperationResponse {
    #[prost(message, repeated, tag = "1")]
    pub original_timestamps: ::prost::alloc::vec::Vec<PbTimestamp>,
    #[prost(message, repeated, tag = "2")]
    pub new_timestamps: ::prost::alloc::vec::Vec<PbTimestamp>,
    #[prost(string, repeated, tag = "3")]
    pub processed_operations: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub original_logical_timestamps: ::prost::alloc::vec::Vec<PbLogicalTimestamp>,
    #[prost(message, repeated, tag = "5")]
    pub current_logical_timestamps: ::prost::alloc::vec::Vec<PbLogicalTimestamp>,
    #[prost(string, repeated, tag = "6")]
    pub full_refresh_timestamp_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserSubscriptionInfo {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(bool, optional, tag = "16")]
    pub subscription_is_active: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "14")]
    pub subscription_management_system: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "2")]
    pub expiration_timestamp_ms_str: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(int64, optional, tag = "15")]
    pub expiration_timestamp_ms: ::core::option::Option<i64>,
    #[prost(int32, optional, tag = "3")]
    pub subscription_type: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub master_user: ::core::option::Option<PbEmailUserIdPair>,
    #[prost(message, repeated, tag = "5")]
    pub subusers: ::prost::alloc::vec::Vec<PbEmailUserIdPair>,
    #[prost(message, repeated, tag = "6")]
    pub nonrenew_iap_receipts: ::prost::alloc::vec::Vec<PbiapReceipt>,
    #[prost(message, repeated, tag = "7")]
    pub autorenew_iap_receipts: ::prost::alloc::vec::Vec<PbiapReceipt>,
    #[prost(message, repeated, tag = "9")]
    pub nonrenew_stripe_charges: ::prost::alloc::vec::Vec<PbStripeCharge>,
    #[prost(message, repeated, tag = "12")]
    pub google_play_purchases: ::prost::alloc::vec::Vec<PbGooglePlayPurchase>,
    #[prost(string, optional, tag = "13")]
    pub google_play_purchase_token: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(string, repeated, tag = "17")]
    pub google_play_order_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "8", default = "5")]
    pub subuser_limit: ::core::option::Option<i32>,
    #[prost(string, repeated, tag = "10")]
    pub sent_email_identifiers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "11")]
    pub user_confirmed_not_renewing: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbiapReceipt {
    #[prost(string, optional, tag = "1")]
    pub transaction_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub receipt_data: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "3")]
    pub parsed_receipt: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbStripeCharge {
    #[prost(string, optional, tag = "1")]
    pub charge_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub charge: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbGooglePlayPurchase {
    #[prost(string, optional, tag = "1")]
    pub order_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub purchase_token: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub purchase_info: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserEmailInfo {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub unsubscribe_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub sent_message_identifiers: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(bool, optional, tag = "3")]
    pub should_send_newsletters: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub should_send_onboarding_tips: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAccountInfoResponse {
    #[prost(int32, optional, tag = "1")]
    pub status_code: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "2")]
    pub first_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub last_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "5")]
    pub is_premium_user: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "6")]
    pub subscription_type: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "7")]
    pub expiration_timestamp_ms_str: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(int64, optional, tag = "11")]
    pub expiration_timestamp_ms: ::core::option::Option<i64>,
    #[prost(message, optional, tag = "8")]
    pub master_user: ::core::option::Option<PbEmailUserIdPair>,
    #[prost(message, repeated, tag = "9")]
    pub subusers: ::prost::alloc::vec::Vec<PbEmailUserIdPair>,
    #[prost(int32, optional, tag = "10")]
    pub subscription_management_system: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAccountChangePasswordResponse {
    #[prost(int32, optional, tag = "1")]
    pub status_code: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "2")]
    pub error_title: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub error_message: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRedemptionCodeInfo {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub redemption_code: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub purchasing_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub redeeming_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "5")]
    pub redemption_timestamp: ::core::option::Option<f64>,
    #[prost(int32, optional, tag = "6")]
    pub subscription_type: ::core::option::Option<i32>,
    #[prost(double, optional, tag = "7")]
    pub creation_timestamp: ::core::option::Option<f64>,
    #[prost(bool, optional, tag = "8")]
    pub was_purchased: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRedemptionCodeResponse {
    #[prost(int32, optional, tag = "1")]
    pub status_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "2")]
    pub account_info: ::core::option::Option<PbAccountInfoResponse>,
    #[prost(string, optional, tag = "3")]
    pub error_title: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub error_message: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbIdentifierList {
    #[prost(double, optional, tag = "1")]
    pub timestamp: ::core::option::Option<f64>,
    #[prost(string, repeated, tag = "2")]
    pub identifiers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbEmailUserIdPair {
    #[prost(string, optional, tag = "1")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub full_name: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbNotificationLocation {
    #[prost(string, required, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(double, optional, tag = "2")]
    pub latitude: ::core::option::Option<f64>,
    #[prost(double, optional, tag = "3")]
    pub longitude: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserDataClientTimestamps {
    #[prost(message, optional, tag = "1")]
    pub shopping_list_timestamps: ::core::option::Option<PbTimestampList>,
    #[prost(message, optional, tag = "2")]
    pub list_folder_timestamps: ::core::option::Option<PbListFolderTimestamps>,
    #[prost(message, optional, tag = "3")]
    pub user_recipe_data_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "4")]
    pub meal_planning_calendar_timestamp: ::core::option::Option<PbLogicalTimestamp>,
    #[prost(message, optional, tag = "5")]
    pub categorized_items_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "6")]
    pub user_categories_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "7")]
    pub starter_list_timestamps: ::core::option::Option<PbTimestampList>,
    #[prost(message, optional, tag = "8")]
    pub recent_item_timestamps: ::core::option::Option<PbTimestampList>,
    #[prost(message, optional, tag = "9")]
    pub favorite_item_timestamps: ::core::option::Option<PbTimestampList>,
    #[prost(message, optional, tag = "10")]
    pub ordered_starter_list_ids_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "11")]
    pub list_settings_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "12")]
    pub starter_list_settings_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "13")]
    pub mobile_app_settings_timestamp: ::core::option::Option<PbTimestamp>,
    #[prost(message, optional, tag = "14")]
    pub shopping_list_logical_timestamps: ::core::option::Option<PbLogicalTimestampList>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbUserDataResponse {
    #[prost(message, optional, tag = "1")]
    pub shopping_lists_response: ::core::option::Option<ShoppingListsResponse>,
    #[prost(message, optional, tag = "2")]
    pub list_folders_response: ::core::option::Option<PbListFoldersResponse>,
    #[prost(message, optional, tag = "3")]
    pub recipe_data_response: ::core::option::Option<PbRecipeDataResponse>,
    #[prost(message, optional, tag = "4")]
    pub meal_planning_calendar_response: ::core::option::Option<PbCalendarResponse>,
    #[prost(message, optional, tag = "5")]
    pub categorized_items_response: ::core::option::Option<PbCategorizedItemsList>,
    #[prost(message, optional, tag = "6")]
    pub user_categories_response: ::core::option::Option<PbUserCategoryData>,
    #[prost(message, optional, tag = "7")]
    pub starter_lists_response: ::core::option::Option<StarterListsResponseV2>,
    #[prost(message, optional, tag = "8")]
    pub ordered_starter_list_ids_response: ::core::option::Option<PbIdentifierList>,
    #[prost(message, optional, tag = "9")]
    pub list_settings_response: ::core::option::Option<PbListSettingsList>,
    #[prost(message, optional, tag = "10")]
    pub starter_list_settings_response: ::core::option::Option<PbListSettingsList>,
    #[prost(message, optional, tag = "11")]
    pub mobile_app_settings_response: ::core::option::Option<PbMobileAppSettings>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbValue {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "2")]
    pub string_value: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "3")]
    pub bool_value: ::core::option::Option<bool>,
    #[prost(int32, optional, tag = "4")]
    pub int_value: ::core::option::Option<i32>,
    #[prost(double, optional, tag = "5")]
    pub double_value: ::core::option::Option<f64>,
    #[prost(bytes = "vec", optional, tag = "6")]
    pub data_value: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "7")]
    pub encoded_pb: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "8")]
    pub pb_class_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "9")]
    pub logical_timestamp_value: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbValueList {
    #[prost(message, repeated, tag = "1")]
    pub values: ::prost::alloc::vec::Vec<PbValue>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbDeletedObjectId {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "2")]
    pub logical_timestamp: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbDeletedObjectIdList {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub container_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub logical_clock_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "4")]
    pub creation_logical_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "5")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(message, repeated, tag = "6")]
    pub deleted_object_ids: ::prost::alloc::vec::Vec<PbDeletedObjectId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbEmailUserIdPairList {
    #[prost(message, repeated, tag = "1")]
    pub email_user_id_pair: ::prost::alloc::vec::Vec<PbEmailUserIdPair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbRecipeLinkRequestList {
    #[prost(message, repeated, tag = "1")]
    pub recipe_link_request: ::prost::alloc::vec::Vec<PbRecipeLinkRequest>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbSyncOperation {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub operation_queue_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub operation_class_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub encoded_operation: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbWatchSyncResponse {
    #[prost(string, optional, tag = "23")]
    pub watch_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "1")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "2")]
    pub is_premium_user: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "3")]
    pub root_folder_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "4")]
    pub logical_timestamp: ::core::option::Option<u64>,
    #[prost(bool, optional, tag = "22")]
    pub is_full_sync: ::core::option::Option<bool>,
    #[prost(message, repeated, tag = "5")]
    pub shopping_lists: ::prost::alloc::vec::Vec<ShoppingList>,
    #[prost(string, repeated, tag = "6")]
    pub deleted_shopping_list_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "7")]
    pub list_items: ::prost::alloc::vec::Vec<ListItem>,
    #[prost(string, repeated, tag = "8")]
    pub deleted_list_item_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "9")]
    pub stores: ::prost::alloc::vec::Vec<PbStore>,
    #[prost(string, repeated, tag = "10")]
    pub deleted_stores_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "11")]
    pub store_filters: ::prost::alloc::vec::Vec<PbStoreFilter>,
    #[prost(string, repeated, tag = "12")]
    pub deleted_store_filter_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "13")]
    pub list_settings: ::prost::alloc::vec::Vec<PbListSettings>,
    #[prost(string, repeated, tag = "14")]
    pub deleted_list_setting_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "15")]
    pub category_groups: ::prost::alloc::vec::Vec<PbCategoryGrouping>,
    #[prost(string, repeated, tag = "16")]
    pub deleted_category_group_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "17")]
    pub categories: ::prost::alloc::vec::Vec<PbUserCategory>,
    #[prost(string, repeated, tag = "18")]
    pub deleted_category_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "24")]
    pub list_categories: ::prost::alloc::vec::Vec<PbListCategory>,
    #[prost(string, repeated, tag = "25")]
    pub deleted_list_category_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "26")]
    pub list_category_groups: ::prost::alloc::vec::Vec<PbListCategoryGroup>,
    #[prost(string, repeated, tag = "27")]
    pub deleted_list_category_group_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "28")]
    pub list_categorization_rules: ::prost::alloc::vec::Vec<PbListCategorizationRule>,
    #[prost(string, repeated, tag = "29")]
    pub deleted_list_categorization_rule_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(message, repeated, tag = "19")]
    pub list_folders: ::prost::alloc::vec::Vec<PbListFolder>,
    #[prost(string, repeated, tag = "20")]
    pub deleted_list_folder_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[prost(string, repeated, tag = "21")]
    pub processed_operation_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbWatchSyncMultipartResponsePart {
    #[prost(string, optional, tag = "1")]
    pub full_response_hash: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "2")]
    pub part_index: ::core::option::Option<i32>,
    #[prost(int32, optional, tag = "3")]
    pub parts_count: ::core::option::Option<i32>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub response_part: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbWatchSyncMultipartResponse {
    #[prost(message, repeated, tag = "1")]
    pub reponse_part: ::prost::alloc::vec::Vec<PbWatchSyncMultipartResponsePart>,
    #[prost(string, optional, tag = "2")]
    pub full_response_hash: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "3")]
    pub response_logical_timestamp: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAlexaUser {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub alexa_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub anylist_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "4")]
    pub has_list_read_permission: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub has_list_write_permission: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "6")]
    pub is_skill_enabled: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "7")]
    pub account_linked_timestamp: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub skill_enabled_timestamp: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "10")]
    pub skill_permission_timestamp: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(string, optional, tag = "11")]
    pub alexa_api_endpoint: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAlexaList {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub alexa_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub anylist_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub alexa_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "6")]
    pub items: ::prost::alloc::vec::Vec<PbAlexaListItem>,
    #[prost(string, optional, tag = "7")]
    pub state: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "8")]
    pub version: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAlexaListItem {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub alexa_item_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub anylist_item_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub alexa_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub alexa_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "6")]
    pub version: ::core::option::Option<i32>,
    #[prost(string, optional, tag = "7")]
    pub item_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub status: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAlexaListOperation {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub operation_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub alexa_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub operation_items: ::prost::alloc::vec::Vec<PbAlexaListItem>,
    #[prost(message, repeated, tag = "5")]
    pub operation_lists: ::prost::alloc::vec::Vec<PbAlexaList>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbAlexaTask {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub alexa_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub event_json: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub list_operation: ::core::option::Option<PbAlexaListOperation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbGoogleAssistantUser {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub anylist_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub list_actions_api_refresh_token: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    #[prost(bool, optional, tag = "4")]
    pub is_google_assistant_account_linked: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "5")]
    pub is_active_google_assistant_provider: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "6")]
    pub anylist_refresh_token: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub anylist_access_token: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbGoogleAssistantList {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub google_assistant_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub anylist_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub anylist_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub title: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "6")]
    pub items: ::prost::alloc::vec::Vec<PbGoogleAssistantListItem>,
    #[prost(bool, optional, tag = "7")]
    pub is_archived: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "8")]
    pub create_time: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub update_time: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbGoogleAssistantListItem {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub google_assistant_item_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub anylist_item_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub google_assistant_list_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub anylist_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub content: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "7")]
    pub is_checked: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "8")]
    pub create_time: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub update_time: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbGoogleAssistantListOperation {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub operation_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub anylist_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub operation_items: ::prost::alloc::vec::Vec<PbGoogleAssistantListItem>,
    #[prost(message, repeated, tag = "5")]
    pub operation_lists: ::prost::alloc::vec::Vec<PbGoogleAssistantList>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbGoogleAssistantTask {
    #[prost(string, optional, tag = "1")]
    pub identifier: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub anylist_user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "3")]
    pub list_operation: ::core::option::Option<PbGoogleAssistantListOperation>,
}
