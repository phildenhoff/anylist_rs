# Web bundle schema vs anylist.proto

Generated on 2026-10-27 with `node tools/webapp_capture/bundle_schema.mjs`, which reads the protobuf.js schema embedded in the web client's `app.min.js`. Message names are as the bundle registers them; some use app class names (`ListItem` for `PBListItem`).

Bundle has 156 messages; 39 are not in anylist.proto, 
93 fields of shared messages are missing, 0 have a different type.

### Fields missing from anylist.proto

| Message | Id | Field | Type |
|---|---|---|---|
| ListItem | 21 | `quantity_pb` | optional PBItemQuantity |
| ListItem | 22 | `price_quantity_pb` | optional PBItemQuantity |
| ListItem | 23 | `price_quantity_should_override_item_quantity` | optional bool |
| ListItem | 24 | `package_size_pb` | optional PBItemPackageSize |
| ListItem | 25 | `price_package_size_pb` | optional PBItemPackageSize |
| ListItem | 26 | `price_package_size_should_override_item_package_size` | optional bool |
| ListItem | 27 | `ingredients` | repeated PBItemIngredient |
| ListItem | 28 | `item_quantity_should_override_ingredient_quantity` | optional bool |
| ListItem | 29 | `item_package_size_should_override_ingredient_package_size` | optional bool |
| PBAccountChangePasswordResponse | 4 | `refresh_token` | optional string |
| PBAccountChangePasswordResponse | 5 | `access_token` | optional string |
| PBAccountInfoResponse | 13 | `subscription_is_canceled` | optional bool |
| PBAccountInfoResponse | 14 | `subscription_is_pending_downgrade` | optional bool |
| PBAccountInfoResponse | 12 | `icalendar_id` | optional string |
| PBAlexaUser | 12 | `did_hear_welcome_message` | optional bool |
| PBCalendarEvent | 12 | `event_type` | optional int32 |
| PBCalendarEvent | 16 | `icon` | optional PBIcon |
| PBCalendarEvent | 17 | `is_leftover` | optional bool |
| PBCalendarEvent | 11 | `label_sort_index` | optional int32 |
| PBCalendarEvent | 15 | `event_list_items` | repeated PBCalendarEventListItem |
| PBCalendarEvent | 13 | `template_id` | optional string |
| PBCalendarEvent | 14 | `template_day_id` | optional string |
| PBCalendarOperation | 11 | `event_type` | optional int32 |
| PBCalendarOperation | 19 | `updated_event_list_item` | optional PBCalendarEventListItem |
| PBCalendarOperation | 20 | `original_event_list_item` | optional PBCalendarEventListItem |
| PBCalendarOperation | 21 | `ordered_event_list_item_ids` | repeated string |
| PBCalendarOperation | 12 | `updated_template` | optional PBMealPlanTemplate |
| PBCalendarOperation | 13 | `original_template` | optional PBMealPlanTemplate |
| PBCalendarOperation | 14 | `template_ids` | repeated string |
| PBCalendarOperation | 15 | `template_group` | optional PBMealPlanTemplateGroup |
| PBCalendarOperation | 16 | `template_group_items` | repeated PBMealPlanTemplateGroupItem |
| PBCalendarOperation | 17 | `original_parent_template_group_id` | optional string |
| PBCalendarOperation | 18 | `updated_parent_template_group_id` | optional string |
| PBCalendarResponse | 12 | `response_version` | optional int32 |
| PBCalendarResponse | 8 | `templates` | repeated PBMealPlanTemplate |
| PBCalendarResponse | 9 | `deleted_template_ids` | repeated string |
| PBCalendarResponse | 10 | `template_events` | repeated PBCalendarEvent |
| PBCalendarResponse | 11 | `deleted_template_event_ids` | repeated string |
| PBCalendarResponse | 13 | `template_groups` | repeated PBMealPlanTemplateGroup |
| PBCalendarResponse | 14 | `deleted_template_group_ids` | repeated string |
| PBGoogleAssistantListOperation | 6 | `google_assistant_list_id` | optional string |
| PBListFolderSettings | 4 | `icon` | optional PBIcon |
| PBListSettings | 31 | `custom_dark_theme` | optional PBListTheme |
| PBListSettings | 32 | `icon` | optional PBIcon |
| PBListSettings | 33 | `has_shown_account_name_prompt` | optional bool |
| PBListSettings | 34 | `is_enabled_for_alexa` | optional bool |
| PBMobileAppSettings | 44 | `keep_screen_on_behavior` | optional int32 |
| PBMobileAppSettings | 43 | `web_selected_list_id` | optional string |
| PBMobileAppSettings | 40 | `web_selected_recipe_collection_type` | optional int32 |
| PBMobileAppSettings | 48 | `web_recipe_collection_layout_style` | optional int32 |
| PBMobileAppSettings | 49 | `web_selected_meal_plan_tab` | optional int32 |
| PBMobileAppSettings | 50 | `web_meal_plan_calendar_layout` | optional int32 |
| PBMobileAppSettings | 51 | `web_meal_plan_month_event_list_type` | optional int32 |
| PBMobileAppSettings | 52 | `web_meal_plan_week_event_list_type` | optional int32 |
| PBMobileAppSettings | 53 | `web_meal_plan_notes_sort_order` | optional int32 |
| PBMobileAppSettings | 54 | `web_meal_plan_add_entries_screen_pinned_entries_collapsed` | optional bool |
| PBMobileAppSettings | 55 | `web_meal_plan_add_entries_screen_queue_entries_collapsed` | optional bool |
| PBMobileAppSettings | 38 | `should_exclude_new_lists_from_alexa_by_default` | optional bool |
| PBMobileAppSettings | 45 | `default_list_id_for_alexa` | optional string |
| PBMobileAppSettings | 46 | `client_has_shown_alexa_onboarding` | optional bool |
| PBMobileAppSettings | 37 | `recipe_cooking_states` | repeated PBRecipeCookingState |
| PBMobileAppSettings | 41 | `did_suppress_account_name_prompt` | optional bool |
| PBMobileAppSettings | 42 | `is_online_shopping_disabled` | optional bool |
| PBMobileAppSettings | 47 | `meal_plan_week_start_day` | optional int32 |
| PBMobileAppSettings | 39 | `should_not_link_new_lists_with_google_assistant_by_default` | optional bool |
| PBOperationMetadata | 5 | `operation_version` | optional int32 |
| PBRecipe | 23 | `recipe_data_id` | optional string |
| PBRecipeCollectionSettings | 5 | `timestamp` | optional double |
| PBRecipeCollectionSettings | 4 | `use_reversed_sort_direction` | optional bool |
| PBRecipeCollectionSettings | 6 | `collections_sort_order` | optional int32 |
| PBRecipeCollectionSettings | 7 | `use_reversed_collections_sort_direction` | optional bool |
| PBRecipeCollectionSettings | 3 | `smart_filter` | optional PBSmartFilter |
| PBRecipeCollectionSettings | 8 | `icon` | optional PBIcon |
| PBRecipeDataResponse | 12 | `max_recipe_count` | optional int32 |
| PBRecipeOperation | 10 | `recipe_event_ids` | repeated string |
| PBRecipeOperation | 11 | `max_recipe_count` | optional int32 |
| PBUserEmailInfo | 6 | `should_send_subscription_lifecycle_messages` | optional bool |
| PBUserRecipeData | 15 | `settings_map_for_system_collections_timestamp` | optional double |
| PBUserRecipeData | 13 | `max_recipe_count` | optional int32 |
| PBUserSubscriptionInfo | 20 | `subscription_is_canceled` | optional bool |
| PBUserSubscriptionInfo | 26 | `subscription_is_pending_downgrade` | optional bool |
| PBUserSubscriptionInfo | 27 | `subscription_is_in_stripe_autorenew_migration_period` | optional bool |
| PBUserSubscriptionInfo | 18 | `stripe_customer_id` | optional string |
| PBUserSubscriptionInfo | 21 | `stripe_subscription_id` | optional string |
| PBUserSubscriptionInfo | 19 | `stripe_subscription_invoices` | repeated PBStripeSubscriptionInvoice |
| PBUserSubscriptionInfo | 22 | `stripe_payment_method_last4` | optional string |
| PBUserSubscriptionInfo | 23 | `stripe_payment_method_expiration_month` | optional int32 |
| PBUserSubscriptionInfo | 24 | `stripe_payment_method_expiration_year` | optional int32 |
| PBUserSubscriptionInfo | 25 | `stripe_payment_method_brand` | optional string |
| Tag | 9 | `localized_display_names` | repeated PBValue |
| Tag | 10 | `localized_search_terms` | repeated PBValue |
| Tag | 11 | `exclusive_locales` | repeated string |
| Tag | 12 | `excluded_locales` | repeated string |

### Fields with a different type

| Message | Id | Web client | anylist.proto |
|---|---|---|---|

### Messages missing from anylist.proto

`PBAppNotice`, `PBAppNoticeList`, `PBAppNoticeOperation`, `PBAppNoticeOperationList`, `PBAppNoticesResponse`, `PBAppNoticesUserData`, `PBAuthTokenInfo`, `PBCalendarEventDescriptor`, `PBCalendarEventListItem`, `PBDataRequestClientInfo`, `PBDeletedUserInfo`, `PBEmailEvent`, `PBEmailSuppressionInfo`, `PBIcon`, `PBItemIngredient`, `PBItemPackageSize`, `PBItemQuantity`, `PBItemQuantityAndPackageSize`, `PBListFolderArchive`, `PBListFolderItemArchive`, `PBMealPlanSetICalendarEnabledRequest`, `PBMealPlanSetICalendarEnabledRequestResponse`, `PBMealPlanTemplate`, `PBMealPlanTemplateGroup`, `PBMealPlanTemplateGroupItem`, `PBMealPlanTemplateGroupSettings`, `PBProductLookupResponse`, `PBRecipeCookingState`, `PBRecipeDataArchive`, `PBRetailProductSubmission`, `PBShoppingListArchive`, `PBSmartCondition`, `PBSmartFilter`, `PBStripeSubscriptionInvoice`, `PBUserDataClientInfo`, `PBXIngredient`, `PBXRecipe`, `PBXRecipeArchive`, `User`
