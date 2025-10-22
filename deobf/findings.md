# Deobfuscation findings

## API Endpoints

### Complete list of `/data/*` endpoints

#### Shopping Lists
- `/data/shopping-lists/update` - Legacy update endpoint
- `/data/shopping-lists/update-v2` - Current update endpoint using PBListOperation
- `/data/shopping-lists/all` - Fetch all shopping lists
- `/data/shopping-lists/share-list` - Share a list with another user
- `/data/shopping-lists/send-as-email` - Email a shopping list

#### Starter Lists
- `/data/starter-lists/update` - Update starter lists (PBStarterListOperation)
- `/data/starter-lists/update-ordered-ids` - Update ordered starter list IDs (PBOrderedStarterListIDsOperation)
- `/data/starter-lists/all-v2` - Fetch all starter lists
- `/data/starter-lists/ordered-ids` - Fetch ordered starter list IDs

#### Starter List Settings
- `/data/starter-list-settings/update` - Update starter list settings
- `/data/starter-list-settings/all` - Fetch all starter list settings

#### Recipes
- `/data/user-recipe-data/update` - Update recipe data (PBRecipeOperation)
- `/data/user-recipe-data/all` - Fetch all user recipe data
- `/data/user-recipe-data/request-recipe-link-v2` - Request to link recipes with another user
- `/data/user-recipe-data/cancel-recipe-link-request` - Cancel a recipe link request
- `/data/user-recipe-data/accept-recipe-link-request` - Accept a recipe link request
- `/data/user-recipe-data/unlink-recipes` - Unlink shared recipes
- `/data/user-recipe-data/desktop-recipe-import-extension` - Import recipe via desktop extension
- `/data/recipes/web-import` - Import recipe from web URL
- `/data/recipes/send-as-email` - Email a recipe

#### Meal Planning Calendar
- `/data/meal-planning-calendar/update` - Update meal plan (PBCalendarOperation)
- `/data/meal-planning-calendar/get` - Fetch meal planning calendar
- `/data/meal-planning-calendar/send-as-email` - Email meal plan
- `/data/meal-planning-calendar/set-icalendar-enabled` - Enable/disable iCalendar export

#### List Folders
- `/data/list-folders/update` - Update list folders (PBListFolderOperation)
- `/data/list-folders/all` - Fetch all list folders

#### List Settings
- `/data/list-settings/update` - Update list settings (PBListSettingsOperation)
- `/data/list-settings/all` - Fetch all list settings

#### Categorized Items
- `/data/categorized-items/update` - Update categorized items (PBCategorizeItemOperation)
- `/data/categorized-items/all` - Fetch all categorized items

#### User Categories
- `/data/user-categories/update` - Update user categories (PBUserCategoryOperation)
- `/data/user-categories/all` - Fetch all user categories

#### Mobile App Settings
- `/data/mobile-app-settings/update` - Update mobile app settings (PBMobileAppSettingsOperation)
- `/data/mobile-app-settings/by-id` - Fetch mobile app settings by ID

#### Photos
- `/data/photos/upload-url` - Get S3 upload URL for photo
- `/data/photos/upload` - Upload photo directly (used with Dropzone)

#### User Data
- `/data/user-data/get` - Fetch user data

#### Account
- `/data/account/info` - Fetch account information

#### Alexa Integration
- `/data/alexa/link-list` - Link a list to Alexa
- `/data/alexa/unlink-list` - Unlink a list from Alexa
- `/data/alexa/set-is-enabled-for-alexa-for-list-ids` - Enable/disable lists for Alexa
- `/data/alexa/unlink-anylist-list` - Unlink AnyList list from Alexa

#### Web Utility Endpoints
- `/data/web/set-mac-app-download-prompt-cookie` - Set cookie for Mac app download prompt
- `/data/web/set-welcome-screen-cookie` - Set cookie for welcome screen

#### WebSocket
- `/data/add-user-listener` - WebSocket endpoint for real-time updates

## Protobuf Message Types

### Operation Types

All operation types follow a similar pattern with a single operation message and a corresponding list message:

#### Shopping List Operations
- `PBListOperation` - Individual shopping list operation
- `PBListOperationList` - List of shopping list operations
- `PBOrderedShoppingListIDsOperation` - Operation for ordering shopping lists
- `PBOrderedShoppingListIDsOperationList` - List of ordering operations

#### Starter List Operations
- `PBStarterListOperation` - Individual starter list operation
- `PBStarterListOperationList` - List of starter list operations
- `PBOrderedStarterListIDsOperation` - Operation for ordering starter lists
- `PBOrderedStarterListIDsOperationList` - List of ordering operations

#### Recipe Operations
- `PBRecipeOperation` - Individual recipe operation
- `PBRecipeOperationList` - List of recipe operations
- `PBSavedRecipeOperation` - Saved recipe operation
- `PBSavedRecipeOperationList` - List of saved recipe operations
- `PBFavoriteProductOperation` - Favorite product operation (deprecated)
- `PBFavoriteProductOperationList` - List of favorite product operations

#### Calendar Operations
- `PBCalendarOperation` - Individual meal planning calendar operation
- `PBCalendarOperationList` - List of calendar operations

#### List Folder Operations
- `PBListFolderOperation` - Individual list folder operation
- `PBListFolderOperationList` - List of folder operations

#### List Settings Operations
- `PBListSettingsOperation` - Individual list settings operation
- `PBListSettingsOperationList` - List of settings operations

#### Categorization Operations
- `PBCategorizeItemOperation` - Individual item categorization operation
- `PBCategorizeItemOperationList` - List of categorization operations
- `PBUserCategoryOperation` - User category operation
- `PBUserCategoryOperationList` - List of user category operations

#### App Settings Operations
- `PBMobileAppSettingsOperation` - Mobile app settings operation
- `PBMobileAppSettingsOperationList` - List of mobile app settings operations
- `PBAppNoticeOperation` - App notice operation
- `PBAppNoticeOperationList` - List of app notice operations

#### Integration Operations
- `PBAlexaListOperation` - Alexa integration operation
- `PBGoogleAssistantListOperation` - Google Assistant integration operation

### Response Types
- `PBEditOperationResponse` - Response containing processed operations with timestamps
- `PBShareListOperationResponse` - Response for list sharing operations
- `PBWatchSyncResponse` - Response for watch sync (likely Apple Watch)
- `PBWatchSyncMultipartResponse` - Multipart response for watch sync
- `PBWatchSyncMultipartResponsePart` - Individual part of multipart watch sync

### Core Message Types
- `PBSyncOperation` - General sync operation
- `ShoppingList` - Shopping list entity
- `PBShoppingListArchive` - Archived shopping list
- `ShoppingListsResponse` - Response containing shopping lists
- `PBOperationMetadata` - Metadata attached to all operations
- `PBTimestamp` - Timestamp message
- `PBTimestampList` - List of timestamps
- `PBLogicalTimestamp` - Logical timestamp for conflict resolution
- `PBLogicalTimestampList` - List of logical timestamps
- `PBCategoryGrouping` - Category grouping configuration
- `PBListFolderTimestamps` - Timestamps for list folders

### Operation Metadata Structure
All operations include `PBOperationMetadata` with:
- `operationId` (string) - Unique identifier for the operation (UUID v4)
- `handlerId` (string) - Handler identifier for tracking operation purpose
- `operationQueueId` (string) - Queue identifier for operation tracking
- `logicalTimestamp` - For conflict resolution
- `timestamp` - Physical timestamp
- Operation class indicators

## Protobuf operation / endpoint mapping

### Update Endpoints (POST with protobuf)

All update endpoints follow the pattern:
1. Accept `PBXXXOperationList` as request body (protobuf encoded)
2. Return `PBEditOperationResponse` (protobuf encoded)
3. Use `arraybuffer` response type
4. Send operations in batches

| Endpoint | Request Type | Response Type |
|----------|--------------|---------------|
| `/data/shopping-lists/update` | `PBListOperationList` | `PBEditOperationResponse` |
| `/data/shopping-lists/update-v2` | `PBListOperationList` | `PBEditOperationResponse` |
| `/data/starter-lists/update` | `PBStarterListOperationList` | `PBEditOperationResponse` |
| `/data/starter-lists/update-ordered-ids` | `PBOrderedStarterListIDsOperationList` | `PBEditOperationResponse` |
| `/data/user-recipe-data/update` | `PBRecipeOperationList` | `PBEditOperationResponse` |
| `/data/meal-planning-calendar/update` | `PBCalendarOperationList` | `PBEditOperationResponse` |
| `/data/list-folders/update` | `PBListFolderOperationList` | `PBEditOperationResponse` |
| `/data/list-settings/update` | `PBListSettingsOperationList` | `PBEditOperationResponse` |
| `/data/categorized-items/update` | `PBCategorizeItemOperationList` | `PBEditOperationResponse` |
| `/data/user-categories/update` | `PBUserCategoryOperationList` | `PBEditOperationResponse` |
| `/data/mobile-app-settings/update` | `PBMobileAppSettingsOperationList` | `PBEditOperationResponse` |
| `/data/starter-list-settings/update` | ? | `PBEditOperationResponse` |

### Fetch Endpoints (GET/POST with protobuf response)

| Endpoint | Request | Response Type | Notes |
|----------|---------|---------------|-------|
| `/data/shopping-lists/all` | protobuf | `ShoppingListsResponse` | Returns all lists |
| `/data/starter-lists/all-v2` | protobuf | ? | Returns all starter lists |
| `/data/starter-lists/ordered-ids` | protobuf | ? | Returns ordered IDs |
| `/data/user-recipe-data/all` | protobuf | ? | Returns all recipes |
| `/data/meal-planning-calendar/get` | protobuf | ? | Returns meal plan |
| `/data/list-folders/all` | protobuf | ? | Returns all folders |
| `/data/list-settings/all` | protobuf | ? | Returns all settings |
| `/data/categorized-items/all` | protobuf | ? | Returns all categorizations |
| `/data/user-categories/all` | protobuf | ? | Returns all categories |
| `/data/user-data/get` | protobuf | ? | Returns user data |
| `/data/account/info` | protobuf | ? | Returns account info |

### Special Endpoints

| Endpoint | Method | Request | Response | Notes |
|----------|--------|---------|----------|-------|
| `/data/shopping-lists/share-list` | POST | protobuf (`PBListOperation`) | `PBShareListOperationResponse` | Share list operation |
| `/data/shopping-lists/send-as-email` | POST | JSON | JSON | Email list |
| `/data/recipes/send-as-email` | POST | JSON | JSON | Email recipe |
| `/data/meal-planning-calendar/send-as-email` | POST | JSON | JSON | Email meal plan |
| `/data/recipes/web-import` | POST | protobuf | ? | Import recipe from URL |
| `/data/photos/upload-url` | POST | protobuf | JSON with S3 URL | Get upload URL |
| `/data/photos/upload` | POST | multipart/form-data | JSON | Direct upload |

## Authentication

### Headers

All authenticated requests require the following headers:

```
Authorization: Bearer <access_token>
X-AnyLeaf-API-Version: 3
X-AnyLeaf-Client-Identifier: <client_uuid>
```

For legacy/alternate authentication (possibly deprecated):
```
X-AnyLeaf-Signed-User-ID: <signed_user_id>
```

### Token Management

#### Token Refresh Flow
1. Access tokens expire and need to be refreshed
2. Refresh endpoint: `/auth/token/refresh`
3. Request includes `refresh_token` parameter
4. Response contains new `access_token` and `refresh_token`
5. Global variables used: `ACCESS_TOKEN`, `REFRESH_TOKEN`

#### Token Refresh Logic
- When a request fails due to invalid token, automatically triggers refresh
- Queues subsequent requests while refresh is in progress
- All queued requests execute with new token after refresh completes
- Error notification sent on refresh failure: `ALAuthTokenManagerDidDetectInvalidRefreshToken`

#### Token Acquisition
- Initial token endpoint: `/auth/token`
- Tokens stored and managed by `$oj$ff` class (AuthTokenManager)

### Client Identifier
- Generated once per client using UUID v4
- Stored in `$oj_oj._g.$oj$e`
- Sent with every API request
- Used for device/session tracking

### User Agent
- Sent for Node.js environments
- Format: `i18next-http-backend (node/{version}; {platform} {arch})`
- Web environments use standard browser user agent

## WebSocket Connection

### Connection Details

#### WebSocket URL
- Base URL: `wss://{host}` where host is determined from API configuration
- Endpoint: `/data/add-user-listener`
- Full URL pattern: `wss://{host}/data/add-user-listener?access_token={encoded_access_token}`

#### Connection Parameters
- Access token passed as query parameter (URL encoded)
- WebSocket established using native `WebSocket` class

### Connection States
WebSocket uses standard states tracked via `readyState`:
- CONNECTING
- OPEN
- CLOSING
- CLOSED

### Heartbeat Mechanism

#### Heartbeat Protocol
- Heartbeat message: `--heartbeat--` (bidirectional)
- Server sends heartbeat, client responds with heartbeat
- Heartbeat timer initialized when connection opens
- Connection closed if too many heartbeats missed

#### Heartbeat Timing
- Heartbeat interval managed by timer
- Max missed heartbeats before force close (specific count not visible in code)
- Error logged: "Too many missed heartbeats, force closing WebSocket"

### Message Types

#### Server-to-Client Messages
1. `--heartbeat--` - Heartbeat ping
2. `refresh-shopping-lists` - Notification to refresh shopping list data
3. Other sync operation messages (format not fully visible)

#### Client-to-Server Messages
1. `--heartbeat--` - Heartbeat pong response

### Auto-Reconnect Logic

#### Retry Delay Configuration
- Initial retry delay: 500ms (`this.$oj$Ya = 500`)
- Delay doubles on each retry: `this.$oj$Ya = 2 * this.$oj$Ya`
- Maximum retry delay: 120 seconds (120000ms)
- Reset to 500ms on successful connection

#### Retry Behavior
- Automatic retry on connection close
- User notification shown for connection loss
- "Lost Connection" dialog prevents multiple simultaneous displays
- Retry continues in background
- Logs: "Retrying WebSocket connection in X seconds"

### Connection Events

#### Event Handlers
- `onopen` - Connection established
- `onmessage` - Message received
- `onerror` - Error occurred
- `onclose` - Connection closed

#### Notifications Posted
- `ALWebSocketConnectionStateDidChangeNotification` - Connection state changed
- `ALUserDidRefreshShoppingListNotification` - Data refresh needed

### Error Handling
- All WebSocket errors logged with `R.ws_error()`
- Connection close events logged with code and reason
- Heartbeat exceptions caught and logged
- Force close on heartbeat timeout

## Rate Limiting and Retry Logic

### Rate Limiting

#### Operation Queue Rate Limiting
- Rate limiter class: `$oj$NH` with method `$oj$jH`
- Shopping list fetches limited: 5 operations (`ALFetchShoppingListsRateLimiter`, limit 5)
- Rate limiter initialized per operation queue type

#### HTTP Request Throttling
- Throttle function available: `h.throttle`
- Used for UI interactions and API calls
- Implementation uses standard JavaScript throttling pattern

### Retry Configuration

#### Backend Connector Retry Settings
- Max retries: 5 (configurable, default: `$.maxRetries >= 0 ? $.maxRetries : 5`)
- Retry timeout: 350ms (configurable, default: `$.retryTimeout >= 1 ? $.retryTimeout : 350`)
- Max parallel reads: 10 (`$.maxParallelReads || 10`)

#### WebSocket Retry (detailed above)
- Exponential backoff: 500ms → 1000ms → 2000ms → ... → 120000ms (max)
- No maximum retry count (continues indefinitely)
- User can manually trigger reconnection from UI

### Operation Queue Management

#### Queue Sync States
- Operations queued before sending
- Sync state tracked per queue
- Notifications sent on state change:
  - `ALEditOperationsQueueDidChangeSyncStateNotification`
  - `ALEditOperationsSyncMonitorDidChangeSyncStateNotification`

#### Pending Operations
- Pending operations archived for Mac app: `window.ALMacAppBridge.archivePendingWebAppOperations()`
- Operations include metadata for tracking
- Operation ID matching for confirmation

#### Sync Monitoring
- Sync state changes tracked and reported
- User notifications for sync failures
- "Unsynced Changes" dialog shown on persistent failures
- Options to save or discard unsynced changes

### Error Recovery

#### Network Error Handling
- Failed operations remain in queue
- Retry on next sync attempt
- User notified of sync failures
- Option to quit/leave and save changes for later sync

#### Status Code Handling
- Success: HTTP 200
- Error responses include `statusCode` field in protobuf response
- Error messages logged via Bugsnag
- Metadata attached to error reports includes:
  - Operation queue ID
  - Operation ID
  - Status code
  - Response content

## Operation Flow Patterns

### Creating Operations

#### Operation ID Generation
- All operations get unique ID via UUID v4: `_.uuid4()`
- Some operations use UUID v5 for deterministic IDs: `_.uuid5(value, namespace)`
- Namespace UUID: `6d86f27f66474ca6a540fcf62af29e59`

#### Operation Metadata
Each operation includes:
```
PBOperationMetadata {
  operationId: string (UUID v4)
  handlerId: string (operation purpose)
  timestamp: PBTimestamp
  logicalTimestamp: PBLogicalTimestamp
  operationQueueId: string
  operationClass: enum
}
```

#### Common Handler IDs
- `add-grouping` - Add category grouping
- `remove-grouping` - Remove category grouping
- `set-grouping-name` - Rename grouping
- `set-grouping-categories` - Update grouping categories
- `set-grouping-category-order` - Reorder categories
- `new-shopping-list` - Create new shopping list
- `add-shopping-list-item` - Add item to list
- `remove-shopping-list-item` - Remove item from list
- `move-shopping-list-item-to-index` - Reorder list item
- `share-shopping-list` - Share list
- `unshare-shopping-list` - Unshare list

### Response Processing

#### PBEditOperationResponse Processing
1. Response decoded from arraybuffer
2. `getProcessedOperations()` returns list of `PBTimestamp`
3. Each timestamp matched to local operation by ID
4. Timestamp applied to local operation
5. Operation removed from pending queue
6. Delegate notified of successful processing

#### Error Handling in Response
- Try/catch around response decoding
- Bugsnag notification on decode failure
- Metadata includes operation queue ID and response JSON
- Error types:
  - `ALEditOperationsNetworkQueueOperationIDMismatch` - ID mismatch
  - "Error processing PBEditOperationResponse" - Decode failure
  - "Delegate failed to process PBEditOperationResponse" - Handler failure

### Timestamp Management

#### Timestamp Types
1. **Physical Timestamps** (`PBTimestamp`)
   - Regular server timestamps
   - Fields: Various `*Timestamp` fields (e.g., `orderedShoppingListIdsTimestamp`)

2. **Logical Timestamps** (`PBLogicalTimestamp`)
   - For conflict resolution
   - Used in distributed sync
   - Fields: `logicalTimestamp`, `categoriesLogicalTimestamp`, etc.

#### Timestamp Usage
- Sent with data fetch requests to get only updates
- Stored per data type
- Updated after successful operation
- Used to determine if refresh needed
- Fields with "RequireRefresh" suffix indicate forced refresh needed

### Sync Operation Structure

#### Full Sync vs Partial Sync
- `PBSyncOperation` includes `isFullSync` boolean
- Full sync: All data retrieved
- Partial sync: Only changes since last timestamp

#### Watch Sync (Apple Watch)
- `PBWatchSyncResponse` for watch synchronization
- `PBWatchSyncMultipartResponse` for large sync operations
- Includes `processedOperationIds` for confirmation

## Implementation Notes for Rust Library

### Critical Patterns to Implement

1. **Operation Batching**
   - Operations batched into `XXXOperationList` messages
   - Sent as single HTTP POST with protobuf body
   - Response contains timestamps for all operations

2. **Timestamp Tracking**
   - Store last timestamp for each data type
   - Send timestamps with fetch requests
   - Update after successful sync
   - Handle both physical and logical timestamps

3. **Operation Metadata**
   - Generate UUID v4 for each operation ID
   - Include handler ID for operation tracking
   - Set appropriate operation class
   - Include logical timestamp for conflict resolution

4. **Authentication Flow**
   - Store access and refresh tokens
   - Include Bearer token in Authorization header
   - Auto-refresh on 401/403 errors
   - Queue requests during refresh
   - Include API version (3) and client identifier

5. **WebSocket Integration**
   - Connect to `/data/add-user-listener` with access token
   - Implement heartbeat ping/pong
   - Handle `refresh-shopping-lists` message
   - Exponential backoff retry (500ms to 120s)
   - Auto-reconnect on disconnect

6. **Error Handling**
   - Check HTTP status codes
   - Parse protobuf error responses
   - Match operation IDs in responses
   - Retry failed operations
   - Handle network timeouts

7. **Rate Limiting**
   - Implement per-endpoint rate limiting
   - Use exponential backoff for retries
   - Queue operations when rate limited
   - Max 5 concurrent operations per type

### HTTP Request Pattern

All protobuf update requests follow this pattern:
```
POST /data/{resource}/update
Headers:
  Authorization: Bearer {access_token}
  X-AnyLeaf-API-Version: 3
  X-AnyLeaf-Client-Identifier: {client_uuid}
  Content-Type: application/x-protobuf
Body: PB{Resource}OperationList (protobuf encoded)
Response: PBEditOperationResponse (protobuf encoded, arraybuffer)
```

All protobuf fetch requests follow this pattern:
```
POST /data/{resource}/all
Headers:
  Authorization: Bearer {access_token}
  X-AnyLeaf-API-Version: 3
  X-AnyLeaf-Client-Identifier: {client_uuid}
Body: PB{Resource}Request with timestamps (protobuf encoded)
Response: {Resource}Response (protobuf encoded, arraybuffer)
```

### API Version
- Current API version: `3`
- Must be sent with all requests via `X-AnyLeaf-API-Version` header

### Content Types
- Protobuf requests: `application/x-protobuf` (implied)
- Protobuf responses: `arraybuffer` response type
- JSON requests: `application/json`
- Photo uploads: `multipart/form-data`

## Complete Handler ID Reference

Based on the deobfuscated code, here's the complete list of handler IDs used in operations:

### Shopping List Handler IDs
- `new-shopping-list` - Create new shopping list
- `add-shopping-list-item` - Add item to list
- `remove-shopping-list-item` - Delete item from list
- `set-list-item-checked` - Check/uncheck an item
- `set-list-item-name` - Rename an item
- `set-list-item-details` - Update item details/notes
- `set-list-item-quantity` - Update item quantity (DEPRECATED - use item update)
- `set-list-item-package-size` - Update package size
- `set-list-item-photo-id` - Set item photo
- `set-list-item-price-matchup-tag` - Set price matching tag
- `set-list-item-price-quantity` - Set price quantity
- `set-list-item-price-package-size` - Set price package size
- `set-list-item-product-upc` - Set UPC/barcode
- `set-list-item-category-match-id` - Set category match
- `set-list-password` - Set list password
- `set-list-theme-id` - Set list theme
- `share-shopping-list` - Share list with user
- `unshare-shopping-list` - Unshare list
- `set-ordered-list-items` - Reorder items
- `move-shopping-list-item-to-index` - Move item to specific position
- `add-list-item-store-id` - Add store to item
- `remove-list-item-store-id` - Remove store from item
- `add-store-ids-to-items` - Bulk add stores
- `remove-store-ids-from-items` - Bulk remove stores
- `remove-store-id-from-all-items` - Remove store from all items
- `add-item-ingredient-to-list-item` - Add recipe ingredient to item
- `remove-ingredient-id-from-list-item` - Remove recipe ingredient from item
- `set-allows-multiple-category-groups` - Enable multiple category sets
- `set-list-item-sort-order` - Set sort order (manual/alphabetical)
- `set-new-list-item-position` - Set where new items appear (top/bottom)
- `add-list-notification-location` - Add geofencing location
- `set-item-quantity-should-override-ingredient-quantity` - Override recipe quantity
- `set-item-package-size-should-override-ingredient-package-size` - Override recipe package size
- `set-list-item-price-quantity-should-override-item-quantity` - Price override
- `set-list-item-price-package-size-should-override-item-package-size` - Price package override

### Store Handler IDs
- `new-store` - Create new store
- `delete-store` - Delete store
- `set-store-name` - Rename store
- `set-sorted-store-ids` - Reorder stores
- `new-store-filter` - Create store filter
- `delete-store-filter` - Delete store filter
- `update-store-filter` - Update store filter
- `set-sorted-store-filter-ids` - Reorder store filters
- `set-store-filter-id` - Set active store filter

### Category Handler IDs
- `create-category` - Create new category
- `add-category` - Add category (legacy)
- `remove-category` - Delete category
- `remove-category-ids` - Delete multiple categories
- `set-category-name` - Rename category
- `set-category-icon` - Set category icon
- `set-sorted-category-ids` - Reorder categories
- `create-category-group` - Create category group/set
- `delete-category-group` - Delete category group
- `set-category-group-name` - Rename category group
- `set-default-category-id` - Set default category for group
- `set-list-category-group-id` - Set active category group for list
- `update-list-item-category-assignment` - Assign item to category
- `remove-categorization-rules-for-category-ids` - Remove auto-categorization rules
- `add-grouping` - Add category grouping
- `remove-grouping` - Remove category grouping
- `set-grouping-name` - Rename grouping
- `set-grouping-categories` - Update grouping categories
- `set-grouping-category-order` - Reorder categories in grouping
- `set-should-hide-category-group-from-browse-list-category-groups-screen` - Hide from browse

### Recipe Handler IDs
- `new-recipe` - Create new recipe (inferred)
- `remove-recipe` - Delete recipe
- `remove-recipe-ids` - Delete multiple recipes
- `new-recipe-collection` - Create recipe collection
- `remove-recipe-collection` - Delete recipe collection
- `set-recipe-collection-name` - Rename collection
- `set-recipe-collection-icon` - Set collection icon
- `add-recipes-to-collection` - Add recipes to collection
- `remove-recipes-from-collection` - Remove recipes from collection
- `set-ordered-recipe-ids-for-collection` - Reorder recipes in collection
- `set-ordered-recipe-collection-ids` - Reorder collections
- `set-recipe-collection-sort-order` - Set collection sort order
- `set-system-collection-sort-order` - Set system collection sort
- `set-system-collection-collections-sort-order` - Set system collections sort
- `remove-recipe-cooking-states` - Clear cooking state

### Meal Planning Handler IDs
- `new-event` - Create meal plan event
- `update-event` - Update meal plan event
- `delete-event` - Delete meal plan event
- `delete-events-for-recipe-id` - Delete all events for a recipe
- `set-event-title` - Set event title
- `set-event-details` - Set event details/notes
- `set-event-label` - Set meal label (breakfast/lunch/dinner)
- `set-date-for-events` - Move event to different date
- `new-label` - Create custom meal label
- `update-label` - Update meal label
- `delete-label` - Delete meal label
- `set-sorted-label-ids` - Reorder meal labels

### List Folder Handler IDs
- `create-new-folder` - Create list folder
- `delete-folder-items` - Delete folder
- `set-folder-name` - Rename folder
- `set-folder-hex-color` - Set folder color
- `set-ordered-folder-items` - Reorder lists in folder
- `set-folder-sort-position` - Set folder sort position

### Settings Handler IDs
- `set-should-hide-categories` - Hide/show categories
- `set-should-hide-completed-items` - Hide/show checked items
- `set-should-hide-prices` - Hide/show prices
- `set-should-hide-running-total-bar` - Hide/show running totals
- `set-should-hide-store-names` - Hide/show store names
- `set-should-remember-item-categories` - Remember item categories
- `set-favorites-autocomplete-enabled` - Enable favorites autocomplete
- `set-recent-items-autocomplete-enabled` - Enable recent items autocomplete
- `set-generic-grocery-autocomplete-enabled` - Enable generic autocomplete
- `set-location-notifications-enabled` - Enable location reminders
- `set-badge-mode` - Set badge notification mode
- `set-left-running-total-type` - Set left total display
- `set-right-running-total-type` - Set right total display
- `set-lists-sort-order` - Set list sort order
- `set-list-id-for-recipe-ingredients` - Set default list for recipes
- `set-should-exclude-new-lists-from-alexa-by-default` - Alexa default setting
- `set-should-show-shared-list-category-order-hint-banner` - Show/hide hint
- `set-web-selected-list-id` - Set selected list (web)
- `set-web-selected-recipe-id` - Set selected recipe (web)
- `set-web-selected-recipe-collection-id` - Set selected collection (web)
- `set-web-selected-recipe-collection-type` - Set selected collection type
- `set-web-selected-tab-id` - Set active tab (web)
- `set-web-selected-list-folder-path` - Set folder path (web)
- `set-web-selected-meal-plan-tab` - Set meal plan tab (web)
- `set-web-has-hidden-stores-and-filters-help` - Hide help banner
- `set-web-has-hidden-item-prices-help` - Hide price help
- `set-has-migrated-user-categories-to-list-categories` - Migration flag
- `set-migrated-list-category-group-id` - Migration category group
- `set-max-recipe-count` - Set recipe limit
- `set-did-suppress-account-name-prompt` - Suppress account name prompt
- `set-icon` - Set list icon

### Starter List Handler IDs
- `new-starter-list` - Create starter list (favorites/recent)
- `remove-starter-list` - Delete starter list
- `set-ordered-list-ids` - Order starter lists
- `add-item` - Add item to starter list
- `remove-item` - Remove item from starter list
- `remove-categorized-item` - Remove categorized item

### Other Observations
- Operations can be combined in a single `OperationList` for batch processing
- Each operation type has a corresponding `operationClass` enum value:
  - `0` = UndefinedOperation (default)
  - `1` = StoreOperation
  - `2` = StoreFilterOperation
  - `3` = ListCategoryOperation
  - `4` = ListCategoryGroupOperation
  - `5` = ListCategorizationRuleOperation

## Critical Bugs Found in Our Rust Implementation

Based on the deobfuscated code, here are issues with our current Rust implementation:

### 1. Wrong Endpoint for Store Operations
**Issue**: We're using `/data/lists/update` but should use `/data/shopping-lists/update-v2`

**Fix Needed**: Update `src/stores.rs` line 77:
```rust
// WRONG:
self.post("data/shopping-lists/update-v2", buf).await?;

// Should validate this is the correct endpoint
```

### 2. Missing Operation Class
**Issue**: Store, category, and other operations need `operationClass` set correctly

**Fix Needed**:
- Store operations: `operationClass = 1` (StoreOperation)
- Category operations: `operationClass = 3` (ListCategoryOperation)
- Category group operations: `operationClass = 4` (ListCategoryGroupOperation)

### 3. Wrong Handler IDs
**Issue**: Some handler IDs in our implementation don't match the actual client

**Corrections Needed**:
- Create list: Should be `"new-shopping-list"` not `"create-list"`
- Add item: Should be `"add-shopping-list-item"` not `"add-list-item"`
- Delete item: Should be `"remove-shopping-list-item"` not `"delete-list-item"`
- Check item: Should be `"set-list-item-checked"` not `"check-list-item"`/`"uncheck-list-item"`
- Create store: Should be `"new-store"` not `"create-store"`
- Delete category: Uses `originalValue` field with category ID, not just handler

### 4. Missing Authentication Pattern
**Issue**: We're using `X-AnyLeaf-Signed-User-ID` but should support Bearer token auth

**Fix Needed**: Add support for:
```rust
Authorization: Bearer <access_token>
```

### 5. Missing Logical Timestamps
**Issue**: We only use physical timestamps, but the API uses logical timestamps for conflict resolution

**Fix Needed**: Track and increment logical timestamps per resource

## Photo Upload Flow

The photo upload process uses a two-step flow:

1. **Request Upload URL**:
   ```
   POST /data/photos/upload-url
   Body: Protobuf with photo metadata
   Response: JSON with S3 presigned URL
   ```

2. **Upload to S3**:
   ```
   PUT <presigned_url>
   Body: Image data
   Headers: From presigned URL
   ```

Alternatively, direct upload:
```
POST /data/photos/upload
Content-Type: multipart/form-data
Body: Image file via Dropzone
```

## iCalendar Feed Format

Meal plans can be exported as iCalendar feeds:

- Endpoint: `/data/meal-planning-calendar/set-icalendar-enabled`
- Feed URL format: `https://icalendar.anylist.com/<unique-id>`
- Read-only feed (no write-back from external calendars)
- Sync frequency varies by calendar client (15min to 24+ hours)

## Recipe Web Import

Recipe import from URLs uses schema.org microdata:

- Endpoint: `/data/recipes/web-import`
- Request includes URL to import
- Response: `PBRecipeWebImportResponse` with:
  - `statusCode` - Success/failure code
  - `recipe` - Parsed recipe data
  - `isPremiumUser` - Whether user is premium
  - `siteSpecificHelpText` - Help text for site issues
  - `freeRecipeImportsRemainingCount` - Remaining free imports (max 5 for free users)
- Free users limited to 5 imports
- Premium users have unlimited imports
