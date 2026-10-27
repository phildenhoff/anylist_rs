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
`anylist.proto` lacks the field (now none).

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

## How the crate matches the web client

Every client method sends what the web client sends for the same action: the
same handlers, in the same order, with the same fields and the same derived
identifiers. The builders live in `src/operations.rs` and take the state the
web client reads first (the stored list, item, category group, recipe and so
on), so values the web client copies from storage are copied too.
`src/operations_webapp_tests.rs` replays the captured identifiers through each
builder and compares the result with the captured request (see "Wire order"
below). Every method was also run against a live account with `zz` data and
read back.

| Crate method | What it sends (captures) |
|---|---|
| `create_list` | `new-shopping-list` with the account's email in `sharedUsers`, the root folder and 20 default categories; two `new-starter-list` (Recent Items and Favorite Items, whose IDs are derived from the list ID); eight list settings `set-*`; `set-icon` (0004 to 0010) |
| `rename_list` | `rename-list` (0028) |
| `delete_list` | `delete-folder-items`, then `remove-list-settings` (0072, 0073) |
| `add_item`, `add_item_with_details` | `add-shopping-list-item` with the item categorised for every category group, plus the list when new items go on top (0011 to 0013) |
| `update_item` | only what changed: `set-list-item-name`, `set-list-item-details` (with the original value), `set-list-item-quantity-v2`, and for a category `save-categorization-rule` then `update-list-item-category-assignment` and `set-list-item-category-match-id` (0017 to 0020) |
| `cross_off_item`, `uncheck_item` | `set-list-item-checked`; crossing off also files a copy under Recent Items, replacing an equal one (0014 to 0016, 0060, 0061) |
| `delete_item`, `bulk_delete_items` | `bulk-remove-list-items` with the stored items, then their copies under Recent Items (0044, 0045) |
| `delete_all_crossed_off_items` | copies missing from Recent Items, then `bulk-remove-list-items` (0062) |
| `create_category` | `create-category` with the next sort index, then `set-sorted-category-ids` to keep "Other" last (0021, 0022) |
| `rename_category` | `set-category-name` with the stored category (0023) |
| `delete_category` | `remove-category-ids`, `remove-categorization-rules-for-category-ids` with `set-sorted-category-ids`, then each of its items moves to the default category (0024 to 0027) |
| `create_store` | `new-store`, then `new-store-filter` ("<name> Items"); a name another store has is refused (0030, 0031) |
| `update_store` | `set-store-name` with the stored store (0032) |
| `delete_store` | `remove-store-id-from-all-items` on the list and its Favorite Items, `update-store-filter` or `delete-store-filter`, the same on Recent Items, then `delete-store` (0035 to 0039) |
| `add_favourite`, `add_favourite_to_list` | `add-item` on the Favorite Items list, categorised for its list (0041) |
| `add_favourite_to_shopping_list` | `bulk-add-list-items` with a copy of the stored favourite (0042) |
| `remove_favourite` | `bulk-remove-list-items` with the stored favourite (0043) |
| `RecipeBuilder::save`, `create_recipe`, `update_recipe` | `save-recipe` with the whole recipe: ingredient identifiers (kept on edit), empty notes, times in seconds, a creation time for a new recipe. An edit also updates the recipe's ingredients on the list recipe ingredients were last added to (0080, 0084, 0085) |
| `delete_recipe` | `delete-events-for-recipe-id` with the recipe's events, `remove-recipe` with the stored recipe, then `remove-ingredient-id-from-list-item` on that list (0350 to 0352) |
| `add_recipe_to_list` | `add-item-ingredient-to-list-item`: ingredients with the same stemmed name, unit and package size share one item, whose ID is derived from them, so adding a recipe twice merges instead of duplicating (0083) |
| `create_recipe_collection(_with_recipe_ids)` | `new-recipe-collection`, `set-recipe-collection-icon`, then `add-recipes-to-collection` (0092 to 0094) |
| `add_recipe_to_collection`, `remove_recipe_from_collection`, `delete_recipe_collection` | the stored collection, with `recipeIds` narrowed to the recipe for add and remove (0094, 0099, 0122) |
| `create_meal_plan_event` | a recipe on its own: `save-new-events` (operation version 1), as when a recipe is added to the meal plan; anything else: `new-event`, as the event editor saves it. The sort index puts it after the day's other events (0171, 0385) |
| `update_meal_plan_event` | `update-event` with the whole stored event, as the event editor saves an existing event; a recipe event's ingredients on the list follow a date change |
| `delete_meal_plan_event` | `delete-event` with the stored event, then its ingredients come off the list (0401) |
| `add_meal_plan_ingredients_to_list` | `add-item-ingredient-to-list-item` for each recipe event's ingredients, linked to the event and scaled as it says |

### Known differences

- **Item database.** The web client looks new items and recipe ingredients
  up in its own grocery database, which picks a category and sets
  `priceMatchupTag`. The crate has no such database: the list's remembered
  category for the name wins, then the caller's `category`, then the list's
  default category, and `priceMatchupTag` is never set.
- **Wire order.** The web client writes some fields in the order its schema
  declares them (an ingredient's `identifier` first, for one); prost writes
  field-number order. Protobuf readers treat the two the same. The tests
  accept either, as long as decoding the capture loses nothing.
- **Interface-only settings.** `set-web-selected-*` and
  `set-list-id-for-recipe-ingredients` record what the web page shows, so the
  crate does not send them.
- **Ingredient parsing.** The crate reads a quantity as an amount and a unit;
  the web client also reads package sizes ("1 (400 g) can") and formats
  scaled amounts as fractions.
- **Collections.** Adding a recipe that does not exist returns `NotFound`, and
  adding one already in the collection sends nothing (the web client sends an
  empty addition).
- **Raw ingredient text.** The web client keeps what was typed. Without
  `raw_ingredient`, the crate writes "<quantity> <name>, <note>".

### Things the crate does not do yet

- Lists: `uncheck-all`, `set-should-hide-prices`.
- Folders: `create-new-folder`, `set-folder-name`, `move-folder-items`,
  `set-ordered-folder-items`, `set-icon`.
- Meal plan: copying and moving entries (`save-new-events`,
  `set-date-for-events`), labels (`new-label`, `update-label`, `delete-label`) and templates
  (`new-template`, `set-template-name`, `remove-template-day-ids`,
  `delete-template`). Pinned, queued and template entries are all `new-event`
  with a different `eventType`.

## Schema

`anylist.proto` now has every message and field of the schema embedded in the
web bundle (`node tools/webapp_capture/bundle_schema.mjs --write-proto` adds
missing ones), so prost keeps every field of what it reads and sends it back.
The `webapp_captures` snapshot shows 0 dropped bytes for every capture.
[webapp-schema-diff.md](webapp-schema-diff.md) records the gaps as they were.

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
