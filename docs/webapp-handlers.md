# AnyList web client handler IDs

Static scan of `app.min.js` from <https://www.anylist.com/web>, taken on 2026-10-27 with
`tools/webapp_capture/bundle_handlers.py`. It lists what the web client *can* send. The
operation type column is a best guess from where the ID appears in the minified
bundle; live captures (see `tools/webapp_capture/README.md`) are the authority.

## Handler IDs

| Handler ID | Operation type (best guess) |
|---|---|
| `add-category` | PBUserCategoryOperation |
| `add-event-list-item` | PBCalendarOperation |
| `add-grouping` | PBUserCategoryOperation |
| `add-item` | PBStarterListOperation |
| `add-list-item-store-id` | PBListOperation, PBStarterListOperation |
| `add-list-notification-location` | PBListOperation |
| `add-recipes-to-collection` | PBRecipeOperation |
| `add-shopping-list-item` | PBListOperation |
| `add-store-ids-to-items` | PBListOperation, PBStarterListOperation |
| `add-template-day-ids` | PBCalendarOperation |
| `bulk-add-list-items` | PBListOperation |
| `bulk-cross-off-list-items` | PBListOperation |
| `bulk-remove-list-items` | PBListOperation, PBStarterListOperation |
| `bulk-uncross-list-items` | PBListOperation |
| `categorize-item` | PBCategorizeItemOperation |
| `clear-starter-list` | PBStarterListOperation |
| `create-new-folder` | PBListFolderOperation |
| `create-root-template-group` | PBCalendarOperation |
| `delete-event` | PBCalendarOperation |
| `delete-events-for-recipe-id` | PBCalendarOperation |
| `delete-folder-items` | PBListFolderOperation |
| `delete-label` | PBCalendarOperation |
| `delete-template` | PBCalendarOperation |
| `delete-template-group` | PBCalendarOperation |
| `move-folder-items` | PBListFolderOperation |
| `move-shopping-list-item-to-index` | PBListOperation |
| `move-template-group-items` | PBCalendarOperation |
| `new-event` | PBCalendarOperation |
| `new-recipe-collection` | PBRecipeOperation |
| `new-shopping-list` | PBListOperation |
| `new-starter-list` | PBStarterListOperation |
| `new-template` | PBCalendarOperation |
| `new-template-group` | PBCalendarOperation |
| `remove-categorized-item` | PBCategorizeItemOperation |
| `remove-category` | PBUserCategoryOperation |
| `remove-event-list-item` | PBCalendarOperation |
| `remove-grouping` | PBUserCategoryOperation |
| `remove-item` | PBStarterListOperation |
| `remove-list-item-store-id` | PBListOperation, PBStarterListOperation |
| `remove-list-settings` | PBListSettingsOperation |
| `remove-recipe` | PBRecipeOperation |
| `remove-recipe-collection` | PBRecipeOperation |
| `remove-recipe-cooking-states` | PBMobileAppSettingsOperation |
| `remove-recipe-ids` | PBRecipeOperation |
| `remove-recipes-from-collection` | PBRecipeOperation |
| `remove-shopping-list-item` | PBListOperation |
| `remove-starter-list` | PBStarterListOperation |
| `remove-store-id-from-all-items` | PBListOperation, PBStarterListOperation |
| `remove-store-ids-from-items` | PBListOperation, PBStarterListOperation |
| `remove-template-day-ids` | PBCalendarOperation |
| `rename-list` | PBListOperation, PBStarterListOperation |
| `save-custom-theme` | PBListSettingsOperation |
| `save-item-price` | PBListOperation, PBStarterListOperation |
| `save-new-events` | PBCalendarOperation |
| `save-recipe` | PBRecipeOperation |
| `save-recipe-cooking-states` | PBMobileAppSettingsOperation |
| `set-allows-multiple-category-groups` | PBListOperation |
| `set-badge-mode` | PBListSettingsOperation |
| `set-category-icon` | PBUserCategoryOperation |
| `set-category-name` | PBUserCategoryOperation |
| `set-date-for-events` | PBCalendarOperation |
| `set-did-suppress-account-name-prompt` | PBMobileAppSettingsOperation |
| `set-event-list-item-details` | PBCalendarOperation |
| `set-event-list-item-name` | PBCalendarOperation |
| `set-event-list-item-package-size` | PBCalendarOperation |
| `set-event-list-item-quantity` | PBCalendarOperation |
| `set-favorites-autocomplete-enabled` | PBListSettingsOperation |
| `set-folder-hex-color` | PBListFolderOperation |
| `set-folder-name` | PBListFolderOperation |
| `set-folder-sort-position` | PBListFolderOperation |
| `set-generic-grocery-autocomplete-enabled` | PBListSettingsOperation |
| `set-grouping-name` | PBUserCategoryOperation |
| `set-has-migrated-user-categories-to-list-categories` | PBMobileAppSettingsOperation |
| `set-icon` | PBListFolderOperation, PBListSettingsOperation |
| `set-left-running-total-type` | PBListSettingsOperation |
| `set-list-category-group-id` | PBListSettingsOperation |
| `set-list-id-for-recipe-ingredients` | PBMobileAppSettingsOperation |
| `set-list-item-category-match-id` | PBListOperation |
| `set-list-item-checked` | PBListOperation |
| `set-list-item-details` | PBListOperation, PBStarterListOperation |
| `set-list-item-name` | PBListOperation, PBStarterListOperation |
| `set-list-item-photo-id` | PBListOperation, PBStarterListOperation |
| `set-list-item-price-matchup-tag` | PBListOperation |
| `set-list-item-product-upc` | PBListOperation, PBStarterListOperation |
| `set-list-item-sort-order` | PBListOperation, PBListSettingsOperation |
| `set-list-password` | PBListOperation |
| `set-list-theme-id` | PBListSettingsOperation |
| `set-lists-sort-order` | PBListFolderOperation |
| `set-location-notifications-enabled` | PBListSettingsOperation |
| `set-max-recipe-count` | PBRecipeOperation |
| `set-migrated-list-category-group-id` | PBListSettingsOperation |
| `set-new-list-item-position` | PBListOperation |
| `set-ordered-event-list-item-ids` | PBCalendarOperation |
| `set-ordered-folder-items` | PBListFolderOperation |
| `set-ordered-list-ids` | PBOrderedStarterListIDsOperation |
| `set-ordered-list-items` | PBListOperation |
| `set-ordered-recipe-collection-ids` | PBRecipeOperation |
| `set-ordered-recipe-ids-for-collection` | PBRecipeOperation |
| `set-ordered-template-group-items` | PBCalendarOperation |
| `set-recent-items-autocomplete-enabled` | PBListSettingsOperation |
| `set-recipe-collection-icon` | PBRecipeOperation |
| `set-recipe-collection-name` | PBRecipeOperation |
| `set-recipe-collection-sort-order` | PBRecipeOperation |
| `set-right-running-total-type` | PBListSettingsOperation |
| `set-should-exclude-new-lists-from-alexa-by-default` | PBMobileAppSettingsOperation |
| `set-should-hide-categories` | PBListSettingsOperation |
| `set-should-hide-category-group-from-browse-list-category-groups-screen` | PBUserCategoryOperation |
| `set-should-hide-completed-items` | PBListSettingsOperation |
| `set-should-hide-prices` | PBListSettingsOperation |
| `set-should-hide-running-total-bar` | PBListSettingsOperation |
| `set-should-hide-store-names` | PBListSettingsOperation |
| `set-should-remember-item-categories` | PBListSettingsOperation |
| `set-should-show-shared-list-category-order-hint-banner` | PBListSettingsOperation |
| `set-sorted-label-ids` | PBCalendarOperation |
| `set-store-filter-id` | PBListSettingsOperation |
| `set-system-collection-collections-sort-order` | PBRecipeOperation |
| `set-system-collection-sort-order` | PBRecipeOperation |
| `set-template-day-id-for-events` | PBCalendarOperation |
| `set-template-day-ids` | PBCalendarOperation |
| `set-template-group-groups-sort-position` | PBCalendarOperation |
| `set-template-group-items-sort-order` | PBCalendarOperation |
| `set-template-icon` | PBCalendarOperation |
| `set-template-name` | PBCalendarOperation |
| `set-web-has-hidden-item-prices-help` | PBMobileAppSettingsOperation |
| `set-web-has-hidden-stores-and-filters-help` | PBMobileAppSettingsOperation |
| `set-web-meal-plan-add-entries-screen-pinned-entries-collapsed` | PBMobileAppSettingsOperation |
| `set-web-meal-plan-add-entries-screen-queue-entries-collapsed` | PBMobileAppSettingsOperation |
| `set-web-meal-plan-calendar-layout` | PBMobileAppSettingsOperation |
| `set-web-meal-plan-month-event-list-type` | PBMobileAppSettingsOperation |
| `set-web-meal-plan-notes-sort-order` | PBMobileAppSettingsOperation |
| `set-web-meal-plan-week-event-list-type` | PBMobileAppSettingsOperation |
| `set-web-recipe-collection-layout-style` | PBMobileAppSettingsOperation |
| `set-web-selected-list-folder-path` | PBMobileAppSettingsOperation |
| `set-web-selected-list-id` | PBMobileAppSettingsOperation |
| `set-web-selected-meal-plan-event-id` | PBMobileAppSettingsOperation |
| `set-web-selected-meal-plan-tab-v2` | PBMobileAppSettingsOperation |
| `set-web-selected-recipe-collection-id` | PBMobileAppSettingsOperation |
| `set-web-selected-recipe-collection-type` | PBMobileAppSettingsOperation |
| `set-web-selected-recipe-id` | PBMobileAppSettingsOperation |
| `set-web-selected-tab-id` | PBMobileAppSettingsOperation |
| `share-shopping-list` | PBListOperation |
| `uncheck-all` | PBListOperation |
| `unshare-shopping-list` | PBListOperation |
| `update-list-item-category-assignment` | PBListOperation |

## Endpoints

- `/auth/logout`
- `/auth/token`
- `/auth/token/refresh`
- `/data/account/info`
- `/data/add-user-listener`
- `/data/alexa/link-list`
- `/data/alexa/set-is-enabled-for-alexa-for-list-ids`
- `/data/alexa/unlink-anylist-list`
- `/data/alexa/unlink-list`
- `/data/categorized-items/all`
- `/data/categorized-items/update`
- `/data/list-folders/all`
- `/data/list-folders/update`
- `/data/list-settings/all`
- `/data/list-settings/update`
- `/data/meal-planning-calendar/get`
- `/data/meal-planning-calendar/send-as-email`
- `/data/meal-planning-calendar/set-icalendar-enabled`
- `/data/meal-planning-calendar/update`
- `/data/mobile-app-settings/by-id`
- `/data/mobile-app-settings/update`
- `/data/photos/image-search`
- `/data/photos/upload`
- `/data/photos/upload-url`
- `/data/recipes/send-as-email`
- `/data/recipes/web-import`
- `/data/shopping-lists/all`
- `/data/shopping-lists/send-as-email`
- `/data/shopping-lists/share-list`
- `/data/shopping-lists/update`
- `/data/shopping-lists/update-v2`
- `/data/starter-list-settings/all`
- `/data/starter-list-settings/update`
- `/data/starter-lists/all-v2`
- `/data/starter-lists/ordered-ids`
- `/data/starter-lists/update`
- `/data/starter-lists/update-ordered-ids`
- `/data/user-categories/all`
- `/data/user-categories/update`
- `/data/user-data/get`
- `/data/user-recipe-data/accept-recipe-link-request`
- `/data/user-recipe-data/all`
- `/data/user-recipe-data/cancel-recipe-link-request`
- `/data/user-recipe-data/desktop-recipe-import-extension`
- `/data/user-recipe-data/request-recipe-link-v2`
- `/data/user-recipe-data/unlink-recipes`
- `/data/user-recipe-data/update`
- `/data/web/set-mac-app-download-prompt-cookie`
- `/data/web/set-welcome-screen-cookie`
