# AnyList web client operations (live capture)

Recorded on 2026-10-27 by driving the web client at <https://www.anylist.com/web>
with Playwright (`tools/webapp_capture`, scenarios in `tools/webapp_capture/scenarios`).
Every scenario uses throwaway data whose names start with `zz` and deletes it
afterwards. Unlike the static scan in [webapp-handlers.md](webapp-handlers.md),
this shows what the client *actually sends* for each action, in what order and
with which fields.

The request bodies are saved as fixtures in
`tests/fixtures/webapp_2026_10_27/{lists,recipes,meal-plan}`. The
`webapp_captures` integration test decodes each one with the crate's prost types,
checks the handler IDs, and snapshots how many bytes prost drops because
`anylist.proto` lacks the field.

## How the capture works

- The client POSTs `multipart/form-data`. The `operations` part is a
  `PB*OperationList`, and each operation's `metadata.handlerId` names the handler.
- The server answers 200 to unknown handler IDs and drops them silently, so a
  200 proves nothing. Read the data back to check that an operation applied.
- Chromium hides `FormData` bodies that contain `Blob`s from
  `request.postDataBuffer()`. The recorder reads them over CDP with
  `Network.getRequestPostData` instead.
- The account email appears in the sign-in form and inside `new-shopping-list`
  (the list's `sharedUsers`). It is replaced with a same-length placeholder so
  the protobuf lengths stay valid.

## Where the crate differs from the web client

### Handler IDs the web client never sends

None of these appear in the capture or in the bundle under the operation type
the crate uses. The server may accept them as silent no-ops. Each one should be
checked live (send, then read back) before it is trusted.

| Crate function | Crate sends | Web client sends instead |
|---|---|---|
| `update_item` | `update-list-item` with a whole `PBListItem` | one handler per field: `set-list-item-details`, `set-list-item-quantity-v2`, `update-list-item-category-assignment` with `set-list-item-category-match-id`, `add-list-item-store-id`, `save-item-price` |
| `add_favourite` | `add-starter-list-item` | `add-item` (adding in the favourites panel) or `bulk-add-list-items` (favouriting from a list) |
| `remove_favourite` | `remove-starter-list-item` | `bulk-remove-list-items` |
| `delete_category` | `remove-category` (`PBListOperation`) | `remove-category-ids`, `remove-categorization-rules-for-category-ids`, `set-sorted-category-ids`. `remove-category` exists in the bundle only as a `PBUserCategoryOperation`. |
| `update_meal_plan_event` | `update-event` | `set-event-title`, `set-date-for-events` (and other `set-*` handlers per field) |

### Handler IDs that match, but the web client sends more

- **Create list** (`create_list`): `new-shopping-list`, then two
  `new-starter-list` operations (the list's favourites and recents), the
  list settings `set-*` operations, and `set-icon`.
- **Create store** (`create_store`): `new-store`, then `new-store-filter`.
- **Delete recipe** (`delete_recipe`): `remove-recipe`, then
  `delete-events-for-recipe-id` and `remove-ingredient-id-from-list-item`.
- **Add recipe to list** (`add_recipe_to_list`): the crate sends
  `add-shopping-list-item`; the web client sends `add-item-ingredient-to-list-item`,
  which links the list item to the recipe ingredient.
- **Create recipe collection**: `new-recipe-collection`, then
  `set-recipe-collection-icon`. The crate never sets the icon.

### Recipe collections (confirms PR #33 and #34)

- Add and remove send `add-recipes-to-collection` and
  `remove-recipes-from-collection` with the stored collection whose `recipeIds`
  are narrowed to the affected recipe.
- Delete sends `remove-recipe-collection` with the collection and no `recipeIds`.
- Reorder sends `set-ordered-recipe-collection-ids`; sort order sends
  `set-recipe-collection-sort-order`.
- **Risk:** the web client always sends `collectionSettings` with
  `useReversedSortDirection` (field 4) and `icon` (field 8). `anylist.proto`
  has neither, so prost drops them when the crate decodes a stored collection,
  and the collection the crate sends back is missing 42 to 45 bytes. If the
  server stores the sent collection whole, the crate would clear the
  collection's icon and sort direction. Not checked live.

### Handler IDs that match

`new-shopping-list`, `rename-list`, `delete-folder-items` with
`remove-list-settings` (delete list), `add-shopping-list-item`,
`set-list-item-checked`, `bulk-remove-list-items`, `create-category`,
`set-category-name`, `new-store`, `set-store-name`, `remove-store-id-from-all-items`
with `delete-store-filter` and `delete-store` (delete store), `new-event`,
`delete-event`, `save-recipe`, `remove-recipe`.

### Things the crate does not do yet

- Lists: `uncheck-all`, `set-should-hide-prices`, `save-categorization-rule`.
- Folders: `create-new-folder`, `set-folder-name`, `move-folder-items`,
  `set-ordered-folder-items`, `set-icon`.
- Meal plan: `save-new-events` (adding a recipe to the calendar, copying
  entries), labels (`new-label`, `update-label`, `delete-label`) and templates
  (`new-template`, `set-template-name`, `remove-template-day-ids`,
  `delete-template`). Pinned, queued and template entries are all `new-event`
  with a different `eventType`.

## Schema gaps

[webapp-schema-diff.md](webapp-schema-diff.md) compares the protobuf.js schema
embedded in the web bundle with `anylist.proto`: 39 messages and 93 fields of
shared messages are missing, and no field has a different type. The ones that
matter most for round trips are `ListItem` 21 to 29 (`quantityPb`,
`ingredients` and others), `CalendarEvent` 11 to 17 (`eventType`,
`templateId`, `isLeftover`, `icon`), `RecipeCollectionSettings` 3 to 8,
the `icon` on list and folder settings, and `OperationMetadata` 5
(`operationVersion`).

## The static scan is incomplete

These handler IDs were captured live but are missing from
[webapp-handlers.md](webapp-handlers.md): `set-list-item-quantity-v2`,
`save-categorization-rule`, `create-category`, `set-sorted-category-ids`,
`remove-category-ids`, `remove-categorization-rules-for-category-ids`,
`new-store`, `new-store-filter`, `set-store-name`, `delete-store-filter`,
`delete-store`, `add-item-ingredient-to-list-item`,
`remove-ingredient-id-from-list-item`, `new-label`, `set-event-title`,
`update-label`.

## Every operation captured

One row per handler ID, with the actions that sent it and one example fixture.

| Handler ID | Endpoint | Sent when | Operation fields | Example |
|---|---|---|---|---|
| `new-shopping-list` | `/data/shopping-lists/update` | list: create; list: create second list; list: create (for recipe ingredients) | listFolderId, listId, list{identifier, name, creator, sharedUsers, listItemSortOrder}, updatedCategoryGroup{identifier, listId, categories, defaultCategoryId} | [0004-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0004-list-create.json) |
| `new-starter-list` | `/data/starter-lists/update` | list: create; list: create second list; list: create (for recipe ingredients) | listId, list{identifier, name, listId, starterListType} | [0005-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0005-list-create.json) |
| `set-list-theme-id` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, listThemeId} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-should-hide-categories` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, shouldHideCategories} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-generic-grocery-autocomplete-enabled` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, genericGroceryAutocompleteEnabled} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-list-item-sort-order` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, listItemSortOrder} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-list-category-group-id` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, listCategoryGroupId} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-should-remember-item-categories` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, shouldRememberItemCategories} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-favorites-autocomplete-enabled` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, favoritesAutocompleteEnabled} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-recent-items-autocomplete-enabled` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId, recentItemsAutocompleteEnabled} | [0006-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0006-list-create.json) |
| `set-icon` | `/data/list-settings/update` | list: create; list: create second list; list: create (for recipe ingredients) | updatedSettings{identifier, userId, listId} | [0010-list-create.json](../tests/fixtures/webapp_2026_10_27/lists/0010-list-create.json) |
| `add-shopping-list-item` | `/data/shopping-lists/update` | item: add | listId, listItemId, listItem{identifier, listId, name, priceMatchupTag, category, userId, categoryMatchId, categoryAssignments} | [0011-item-add.json](../tests/fixtures/webapp_2026_10_27/lists/0011-item-add.json) |
| `set-list-item-checked` | `/data/shopping-lists/update` | item: cross off; item: uncross; list: uncross all; list: delete crossed off | listId, listItemId, updatedValue | [0014-item-cross-off.json](../tests/fixtures/webapp_2026_10_27/lists/0014-item-cross-off.json) |
| `bulk-add-list-items` | `/data/starter-lists/update` | item: cross off; favourites: add list item to favourites; item: delete; item: move to other list; list: uncross all; list: delete crossed off; list: delete all items | listId, list{identifier, items} | [0015-item-cross-off.json](../tests/fixtures/webapp_2026_10_27/lists/0015-item-cross-off.json) |
| `set-list-item-details` | `/data/shopping-lists/update` | item: edit note and quantity | listId, listItemId, updatedValue | [0017-item-edit-note-and-quantity.json](../tests/fixtures/webapp_2026_10_27/lists/0017-item-edit-note-and-quantity.json) |
| `set-list-item-quantity-v2` | `/data/shopping-lists/update` | item: edit note and quantity | listId, listItemId, listItem{identifier, listId} | [0018-item-edit-note-and-quantity.json](../tests/fixtures/webapp_2026_10_27/lists/0018-item-edit-note-and-quantity.json) |
| `save-categorization-rule` | `/data/shopping-lists/update-v2` | item: change category; category: delete | listId, updatedCategorizationRule{identifier, listId, categoryGroupId, itemName, categoryId} | [0019-item-change-category.json](../tests/fixtures/webapp_2026_10_27/lists/0019-item-change-category.json) |
| `update-list-item-category-assignment` | `/data/shopping-lists/update` | item: change category; category: delete | listId, listItemId, listItem{identifier, listId, name, details, priceMatchupTag, category, userId, categoryMatchId, categoryAssignments} | [0020-item-change-category.json](../tests/fixtures/webapp_2026_10_27/lists/0020-item-change-category.json) |
| `set-list-item-category-match-id` | `/data/shopping-lists/update` | item: change category; category: delete | listId, listItemId, listItem{identifier, listId, name, details, priceMatchupTag, category, userId, categoryMatchId, categoryAssignments}, originalValue | [0020-item-change-category.json](../tests/fixtures/webapp_2026_10_27/lists/0020-item-change-category.json) |
| `create-category` | `/data/shopping-lists/update-v2` | category: create | listId, updatedCategory{identifier, categoryGroupId, listId, name, icon, sortIndex} | [0021-category-create.json](../tests/fixtures/webapp_2026_10_27/lists/0021-category-create.json) |
| `set-sorted-category-ids` | `/data/shopping-lists/update-v2` | category: create; category: delete | listId, updatedCategoryGroup{identifier, logicalTimestamp, listId, categories, categoriesLogicalTimestamp, defaultCategoryId} | [0022-category-create.json](../tests/fixtures/webapp_2026_10_27/lists/0022-category-create.json) |
| `set-category-name` | `/data/shopping-lists/update-v2` | category: rename | listId, updatedCategory{identifier, categoryGroupId, listId, name, icon, sortIndex} | [0023-category-rename.json](../tests/fixtures/webapp_2026_10_27/lists/0023-category-rename.json) |
| `remove-category-ids` | `/data/shopping-lists/update-v2` | category: delete | listId, updatedCategoryGroup{identifier, logicalTimestamp, listId, categories, categoriesLogicalTimestamp, defaultCategoryId} | [0024-category-delete.json](../tests/fixtures/webapp_2026_10_27/lists/0024-category-delete.json) |
| `remove-categorization-rules-for-category-ids` | `/data/shopping-lists/update-v2` | category: delete | listId, updatedCategoryGroup{identifier, logicalTimestamp, listId, categories, categoriesLogicalTimestamp, defaultCategoryId} | [0025-category-delete.json](../tests/fixtures/webapp_2026_10_27/lists/0025-category-delete.json) |
| `rename-list` | `/data/shopping-lists/update` | list: rename | listId, updatedValue | [0028-list-rename.json](../tests/fixtures/webapp_2026_10_27/lists/0028-list-rename.json) |
| `set-should-hide-prices` | `/data/list-settings/update` | list settings: toggle show prices | updatedSettings{identifier, userId, listId, shouldHidePrices} | [0029-list-settings-toggle-show-prices.json](../tests/fixtures/webapp_2026_10_27/lists/0029-list-settings-toggle-show-prices.json) |
| `new-store` | `/data/shopping-lists/update-v2` | store: create | listId, updatedStore{identifier, listId, name, sortIndex} | [0030-store-create.json](../tests/fixtures/webapp_2026_10_27/lists/0030-store-create.json) |
| `new-store-filter` | `/data/shopping-lists/update-v2` | store: create | listId, updatedStoreFilter{identifier, listId, name, storeIds, includesUnassignedItems, sortIndex} | [0031-store-create.json](../tests/fixtures/webapp_2026_10_27/lists/0031-store-create.json) |
| `set-store-name` | `/data/shopping-lists/update-v2` | store: rename | listId, updatedStore{identifier, listId, name, sortIndex} | [0032-store-rename.json](../tests/fixtures/webapp_2026_10_27/lists/0032-store-rename.json) |
| `add-list-item-store-id` | `/data/shopping-lists/update` | item: assign store | listId, listItemId, updatedValue | [0033-item-assign-store.json](../tests/fixtures/webapp_2026_10_27/lists/0033-item-assign-store.json) |
| `save-item-price` | `/data/shopping-lists/update` | item: edit price | itemPrice{amount, date}, listId, listItemId | [0034-item-edit-price.json](../tests/fixtures/webapp_2026_10_27/lists/0034-item-edit-price.json) |
| `remove-store-id-from-all-items` | `/data/shopping-lists/update` | store: delete (with an item assigned) | listId, updatedValue | [0035-store-delete-with-an-item-assigned.json](../tests/fixtures/webapp_2026_10_27/lists/0035-store-delete-with-an-item-assigned.json) |
| `remove-store-id-from-all-items` | `/data/starter-lists/update` | store: delete (with an item assigned) | listId, updatedValue | [0036-store-delete-with-an-item-assigned.json](../tests/fixtures/webapp_2026_10_27/lists/0036-store-delete-with-an-item-assigned.json) |
| `delete-store-filter` | `/data/shopping-lists/update-v2` | store: delete (with an item assigned) | listId, updatedStoreFilter{identifier, listId, name, storeIds, includesUnassignedItems, sortIndex} | [0037-store-delete-with-an-item-assigned.json](../tests/fixtures/webapp_2026_10_27/lists/0037-store-delete-with-an-item-assigned.json) |
| `delete-store` | `/data/shopping-lists/update-v2` | store: delete (with an item assigned) | listId, updatedStore{identifier, listId, name, sortIndex} | [0039-store-delete-with-an-item-assigned.json](../tests/fixtures/webapp_2026_10_27/lists/0039-store-delete-with-an-item-assigned.json) |
| `add-item` | `/data/starter-lists/update` | favourites: add item in favourites panel | listId, listItemId, listItem{identifier, listId, name, priceMatchupTag, category, userId, categoryMatchId, categoryAssignments} | [0041-favourites-add-item-in-favourites-panel.json](../tests/fixtures/webapp_2026_10_27/lists/0041-favourites-add-item-in-favourites-panel.json) |
| `bulk-add-list-items` | `/data/shopping-lists/update` | favourites: add favourite to list; item: copy to other list; item: move to other list | listId, list{identifier, items} | [0042-favourites-add-favourite-to-list.json](../tests/fixtures/webapp_2026_10_27/lists/0042-favourites-add-favourite-to-list.json) |
| `bulk-remove-list-items` | `/data/starter-lists/update` | favourites: delete favourite; list: delete crossed off | listId, list{identifier, items} | [0043-favourites-delete-favourite.json](../tests/fixtures/webapp_2026_10_27/lists/0043-favourites-delete-favourite.json) |
| `bulk-remove-list-items` | `/data/shopping-lists/update` | item: delete; item: move to other list; list: delete crossed off; list: delete all items | listId, list{identifier, items} | [0044-item-delete.json](../tests/fixtures/webapp_2026_10_27/lists/0044-item-delete.json) |
| `uncheck-all` | `/data/shopping-lists/update` | list: uncross all | listId | [0059-list-uncross-all.json](../tests/fixtures/webapp_2026_10_27/lists/0059-list-uncross-all.json) |
| `create-new-folder` | `/data/list-folders/update` | folder: create | listDataId, listFolder{identifier, name, folderSettings}, updatedParentFolderId | [0066-folder-create.json](../tests/fixtures/webapp_2026_10_27/lists/0066-folder-create.json) |
| `set-icon` | `/data/list-folders/update` | folder: create | listDataId, listFolder{identifier, folderSettings} | [0067-folder-create.json](../tests/fixtures/webapp_2026_10_27/lists/0067-folder-create.json) |
| `move-folder-items` | `/data/list-folders/update` | folder: move list into folder | folderItems, listDataId, originalParentFolderId, updatedParentFolderId | [0069-folder-move-list-into-folder.json](../tests/fixtures/webapp_2026_10_27/lists/0069-folder-move-list-into-folder.json) |
| `set-folder-name` | `/data/list-folders/update` | folder: rename | listDataId, listFolder{identifier, name} | [0070-folder-rename.json](../tests/fixtures/webapp_2026_10_27/lists/0070-folder-rename.json) |
| `set-ordered-folder-items` | `/data/list-folders/update` | lists: reorder | folderItems, listDataId, originalParentFolderId | [0071-lists-reorder.json](../tests/fixtures/webapp_2026_10_27/lists/0071-lists-reorder.json) |
| `delete-folder-items` | `/data/list-folders/update` | list: delete; folder: delete (with a list inside); list: delete (recipe ingredients list) | folderItems, listDataId, originalParentFolderId | [0072-list-delete.json](../tests/fixtures/webapp_2026_10_27/lists/0072-list-delete.json) |
| `remove-list-settings` | `/data/list-settings/update` | list: delete; folder: delete (with a list inside); list: delete (recipe ingredients list) | updatedSettings{identifier, userId, listId} | [0073-list-delete.json](../tests/fixtures/webapp_2026_10_27/lists/0073-list-delete.json) |
| `save-recipe` | `/data/user-recipe-data/update` | recipe: create; recipe: edit (rename); recipe: duplicate | isNewRecipeFromWebImport, recipeDataId, recipe{identifier, name, note, sourceName, sourceUrl, ingredients, preparationSteps, rating, creationTimestamp, nutritionalInfo, prepTime, servings} | [0080-recipe-create.json](../tests/fixtures/webapp_2026_10_27/recipes/0080-recipe-create.json) |
| `add-item-ingredient-to-list-item` | `/data/shopping-lists/update` | recipe: add ingredients to list; recipe: edit (rename) | listId, listItemId, listItem{identifier, listId, name, priceMatchupTag, category, userId, categoryMatchId, categoryAssignments}, listItem{identifier, listId} | [0083-recipe-add-ingredients-to-list.json](../tests/fixtures/webapp_2026_10_27/recipes/0083-recipe-add-ingredients-to-list.json) |
| `delete-events-for-recipe-id` | `/data/meal-planning-calendar/update` | recipe: delete (the duplicate); recipe: delete | calendarId, updatedEvent{identifier, calendarId, recipeId} | [0088-recipe-delete-the-duplicate.json](../tests/fixtures/webapp_2026_10_27/recipes/0088-recipe-delete-the-duplicate.json) |
| `remove-recipe` | `/data/user-recipe-data/update` | recipe: delete (the duplicate); recipe: delete | recipeDataId, recipe{identifier, name, note, sourceName, sourceUrl, ingredients, preparationSteps, rating, creationTimestamp, nutritionalInfo, prepTime, servings} | [0089-recipe-delete-the-duplicate.json](../tests/fixtures/webapp_2026_10_27/recipes/0089-recipe-delete-the-duplicate.json) |
| `new-recipe-collection` | `/data/user-recipe-data/update` | collection: create | recipeCollection{identifier, name}, recipeDataId | [0092-collection-create.json](../tests/fixtures/webapp_2026_10_27/recipes/0092-collection-create.json) |
| `set-recipe-collection-icon` | `/data/user-recipe-data/update` | collection: create | recipeCollection{identifier, name, collectionSettings}, recipeDataId | [0093-collection-create.json](../tests/fixtures/webapp_2026_10_27/recipes/0093-collection-create.json) |
| `add-recipes-to-collection` | `/data/user-recipe-data/update` | collection: add recipe (from the recipe); collection: add recipe (from the collection) | recipeCollection{identifier, name, recipeIds, collectionSettings}, recipeDataId | [0094-collection-add-recipe-from-the-recipe.json](../tests/fixtures/webapp_2026_10_27/recipes/0094-collection-add-recipe-from-the-recipe.json) |
| `set-recipe-collection-name` | `/data/user-recipe-data/update` | collection: rename | recipeCollection{identifier, name, recipeIds, collectionSettings}, recipeDataId | [0095-collection-rename.json](../tests/fixtures/webapp_2026_10_27/recipes/0095-collection-rename.json) |
| `set-recipe-collection-sort-order` | `/data/user-recipe-data/update` | collection: change sort order | recipeCollection{identifier, name, recipeIds, collectionSettings}, recipeDataId | [0098-collection-change-sort-order.json](../tests/fixtures/webapp_2026_10_27/recipes/0098-collection-change-sort-order.json) |
| `remove-recipes-from-collection` | `/data/user-recipe-data/update` | collection: remove recipe (from the recipe); collection: remove recipe (from the collection) | recipeCollection{identifier, name, recipeIds, collectionSettings}, recipeDataId | [0099-collection-remove-recipe-from-the-recipe.json](../tests/fixtures/webapp_2026_10_27/recipes/0099-collection-remove-recipe-from-the-recipe.json) |
| `set-ordered-recipe-collection-ids` | `/data/user-recipe-data/update` | collection: reorder | recipeCollectionIds, recipeDataId | [0105-collection-reorder.json](../tests/fixtures/webapp_2026_10_27/recipes/0105-collection-reorder.json) |
| `remove-recipe-collection` | `/data/user-recipe-data/update` | collection: delete | recipeCollection{identifier, name, collectionSettings}, recipeDataId | [0122-collection-delete.json](../tests/fixtures/webapp_2026_10_27/recipes/0122-collection-delete.json) |
| `save-new-events` | `/data/meal-planning-calendar/update` | meal plan: add recipe to calendar; meal plan: copy entries | calendarId, updatedEvents | [0171-meal-plan-add-recipe-to-calendar.json](../tests/fixtures/webapp_2026_10_27/recipes/0171-meal-plan-add-recipe-to-calendar.json) |
| `new-event` | `/data/meal-planning-calendar/update` | meal plan: add recipe to queue; meal plan: plan leftovers; meal plan: note event ingredient, then save; meal plan: pin entry; meal plan: add to queue; template: add note entry; template: apply to calendar | calendarId, updatedEvent{identifier, calendarId, date, recipeId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, date, title, details, labelId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, date, title, orderAddedSortIndex}, updatedEvent{identifier, calendarId, recipeId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, title, details, labelId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, title, orderAddedSortIndex} | [0192-meal-plan-add-recipe-to-queue.json](../tests/fixtures/webapp_2026_10_27/recipes/0192-meal-plan-add-recipe-to-queue.json) |
| `delete-event` | `/data/meal-planning-calendar/update` | meal plan: remove recipe event; meal plan: remove leftovers event; queue: remove recipe; meal plan: remove event from calendar; queue: remove entry; pinned: remove entry; meal plan: remove entries (day 21); meal plan: remove entries (day 22) | calendarId, updatedEvent{identifier, calendarId, date, recipeId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, date, title, details, labelId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, date, title, orderAddedSortIndex}, updatedEvent{identifier, calendarId, recipeId, orderAddedSortIndex}, updatedEvent{identifier, calendarId, title, details, labelId, orderAddedSortIndex} | [0256-meal-plan-remove-recipe-event.json](../tests/fixtures/webapp_2026_10_27/recipes/0256-meal-plan-remove-recipe-event.json) |
| `remove-ingredient-id-from-list-item` | `/data/shopping-lists/update` | recipe: delete | listId, listItemId, listItem{identifier, listId} | [0352-recipe-delete.json](../tests/fixtures/webapp_2026_10_27/recipes/0352-recipe-delete.json) |
| `new-label` | `/data/meal-planning-calendar/update` | label: create | calendarId, updatedLabel{identifier, calendarId, hexColor, name, sortIndex} | [0384-label-create.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0384-label-create.json) |
| `set-event-title` | `/data/meal-planning-calendar/update` | meal plan: edit event title | calendarId, updatedEvent{identifier, calendarId, date, title, details, labelId, orderAddedSortIndex} | [0386-meal-plan-edit-event-title.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0386-meal-plan-edit-event-title.json) |
| `set-date-for-events` | `/data/meal-planning-calendar/update` | meal plan: change date; meal plan: move entries | calendarId, updatedEvents | [0388-meal-plan-change-date.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0388-meal-plan-change-date.json) |
| `new-template` | `/data/meal-planning-calendar/update` | template: create | calendarId | [0468-template-create.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0468-template-create.json) |
| `set-template-name` | `/data/meal-planning-calendar/update` | template: rename | calendarId | [0477-template-rename.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0477-template-rename.json) |
| `remove-template-day-ids` | `/data/meal-planning-calendar/update` | template: set number of days | calendarId | [0478-template-set-number-of-days.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0478-template-set-number-of-days.json) |
| `update-label` | `/data/meal-planning-calendar/update` | label: rename | calendarId, updatedLabel{identifier, calendarId, hexColor, name, sortIndex} | [0512-label-rename.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0512-label-rename.json) |
| `delete-label` | `/data/meal-planning-calendar/update` | label: delete | calendarId, updatedLabel{identifier, calendarId, hexColor, name, sortIndex} | [0513-label-delete.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0513-label-delete.json) |
| `delete-template` | `/data/meal-planning-calendar/update` | template: delete | calendarId, eventIds | [0514-template-delete.json](../tests/fixtures/webapp_2026_10_27/meal-plan/0514-template-delete.json) |
