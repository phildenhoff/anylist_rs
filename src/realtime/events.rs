/// Events that can be received via real-time sync
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncEvent {
    /// Shopping lists have changed - re-fetch with get_lists()
    ShoppingListsChanged,

    /// Categorized items have changed
    CategorizedItemsChanged,

    /// List folders have changed
    ListFoldersChanged,

    /// List settings have changed
    ListSettingsChanged,

    /// Starter lists (Today's/Weekly) have changed
    StarterListsChanged,

    /// Starter list order has changed
    StarterListOrderChanged,

    /// Starter list settings have changed
    StarterListSettingsChanged,

    /// Mobile app settings have changed
    MobileAppSettingsChanged,

    /// User categories have changed
    UserCategoriesChanged,

    /// Recipe data has changed
    RecipeDataChanged,

    /// Meal plan calendar has changed
    MealPlanCalendarChanged,

    /// Account info has changed
    AccountInfoChanged,

    /// Subscription info has changed
    SubscriptionInfoChanged,

    /// User account has been deleted - should logout
    AccountDeleted,

    /// Heartbeat received (internal, usually not exposed to consumers)
    Heartbeat,
}

impl SyncEvent {
    /// Parse a WebSocket message into a SyncEvent
    pub fn from_message(msg: &str) -> Option<Self> {
        match msg {
            "--heartbeat--" => Some(SyncEvent::Heartbeat),
            "refresh-shopping-lists" => Some(SyncEvent::ShoppingListsChanged),
            "refresh-categorized-items" => Some(SyncEvent::CategorizedItemsChanged),
            "refresh-list-folders" => Some(SyncEvent::ListFoldersChanged),
            "refresh-list-settings" => Some(SyncEvent::ListSettingsChanged),
            "refresh-starter-lists" => Some(SyncEvent::StarterListsChanged),
            "refresh-ordered-starter-list-ids" => Some(SyncEvent::StarterListOrderChanged),
            "refresh-starter-list-settings" => Some(SyncEvent::StarterListSettingsChanged),
            "refresh-mobile-app-settings" => Some(SyncEvent::MobileAppSettingsChanged),
            "refresh-user-categories" => Some(SyncEvent::UserCategoriesChanged),
            "refresh-user-recipe-data" => Some(SyncEvent::RecipeDataChanged),
            "refresh-meal-plan-calendar" => Some(SyncEvent::MealPlanCalendarChanged),
            "refresh-account-info" => Some(SyncEvent::AccountInfoChanged),
            "refresh-subscription-info" => Some(SyncEvent::SubscriptionInfoChanged),
            "did-delete-account" => Some(SyncEvent::AccountDeleted),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heartbeat() {
        assert_eq!(
            SyncEvent::from_message("--heartbeat--"),
            Some(SyncEvent::Heartbeat)
        );
    }

    #[test]
    fn test_parse_shopping_lists_changed() {
        assert_eq!(
            SyncEvent::from_message("refresh-shopping-lists"),
            Some(SyncEvent::ShoppingListsChanged)
        );
    }

    #[test]
    fn test_parse_account_deleted() {
        assert_eq!(
            SyncEvent::from_message("did-delete-account"),
            Some(SyncEvent::AccountDeleted)
        );
    }

    #[test]
    fn test_parse_unknown_message() {
        assert_eq!(SyncEvent::from_message("unknown-message"), None);
    }

    #[test]
    fn test_all_known_messages() {
        let messages = vec![
            ("--heartbeat--", SyncEvent::Heartbeat),
            ("refresh-shopping-lists", SyncEvent::ShoppingListsChanged),
            (
                "refresh-categorized-items",
                SyncEvent::CategorizedItemsChanged,
            ),
            ("refresh-list-folders", SyncEvent::ListFoldersChanged),
            ("refresh-list-settings", SyncEvent::ListSettingsChanged),
            ("refresh-starter-lists", SyncEvent::StarterListsChanged),
            (
                "refresh-ordered-starter-list-ids",
                SyncEvent::StarterListOrderChanged,
            ),
            (
                "refresh-starter-list-settings",
                SyncEvent::StarterListSettingsChanged,
            ),
            (
                "refresh-mobile-app-settings",
                SyncEvent::MobileAppSettingsChanged,
            ),
            ("refresh-user-categories", SyncEvent::UserCategoriesChanged),
            ("refresh-user-recipe-data", SyncEvent::RecipeDataChanged),
            (
                "refresh-meal-plan-calendar",
                SyncEvent::MealPlanCalendarChanged,
            ),
            ("refresh-account-info", SyncEvent::AccountInfoChanged),
            (
                "refresh-subscription-info",
                SyncEvent::SubscriptionInfoChanged,
            ),
            ("did-delete-account", SyncEvent::AccountDeleted),
        ];

        for (msg, expected) in messages {
            assert_eq!(
                SyncEvent::from_message(msg),
                Some(expected),
                "Failed to parse: {}",
                msg
            );
        }
    }
}
