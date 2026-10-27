// Recipes and recipe collections.
//
// Creates a throwaway recipe and collection, exercises every recipe and
// collection action, plans the recipe on the calendar, then deletes it all.
// A throwaway list receives the recipe's ingredients and is deleted too.

import {
  back,
  confirmAlert,
  createList,
  dismissIntro,
  done,
  openList,
  openRecipe,
  pickDay,
  row,
  settle,
  tab,
  tapRow,
  top,
} from "./lib.mjs";

const RECIPE = "zz capture recipe";
const RECIPE_RENAMED = "zz capture recipe 2";
const RECIPE_COPY = "zz capture recipe copy";
const COLLECTION = "zz capture collection";
const COLLECTION_RENAMED = "zz capture collection 2";
const LIST = "zz capture recipe list";

// The "+" and "..." in the left pane header, and "..." in the detail header.
async function paneAdd(page, action) {
  await page.mouse.click(364, 65);
  await settle(page, 600);
  await page.getByText(action, { exact: true }).click();
  await settle(page, 1_000);
}

async function paneMenu(page, action) {
  await page.mouse.click(410, 65);
  await settle(page, 600);
  await page.locator("span", { hasText: new RegExp(`^${action}$`) }).last().click();
  await settle(page, 1_000);
}

async function detailMenu(page, action) {
  await page.mouse.click(1380, 65);
  await settle(page, 600);
  await page.getByText(action, { exact: true }).click();
  await settle(page, 1_000);
}

// Opens a recipe through the search field, then leaves search mode so the
// Recipes pane shows its collections again.
async function showRecipe(page, name) {
  await openRecipe(page, name);
  await page.locator(".ALBarButtonDoneItem:visible").first().click();
  await settle(page);
}

async function scrollSheet(page) {
  await page.mouse.move(700, 600);
  await page.mouse.wheel(0, 3_000);
  await settle(page, 600);
}

async function deleteOpenRecipe(page) {
  await detailMenu(page, "Edit Recipe");
  await scrollSheet(page);
  await top(page, ".ALTableCellDeleteButtonTitle").click();
  await settle(page, 800);
  await confirmAlert(page);
}

export default async function recipes({ page, rec }) {
  page.setDefaultTimeout(10_000);
  await dismissIntro(page);

  rec.mark("list: create (for recipe ingredients)");
  await createList(page, LIST);

  await tab(page, "Recipes");

  rec.mark("recipe: create");
  await paneAdd(page, "Create Recipe");
  await top(page, 'input[placeholder="Recipe Name"]').fill(RECIPE);
  await tapRow(page, "Ingredients");
  await page.getByText("Add Ingredients", { exact: true }).click();
  await settle(page, 800);
  await top(page, "textarea.ALTextView").fill("2 cups zz flour\n1 tsp zz salt\n250 mL zz water");
  await done(page);
  await back(page);
  await tapRow(page, "Preparation Steps");
  await top(page, "textarea.ALTextView").fill("Mix zz flour and zz salt.\n\nAdd zz water and knead.");
  await back(page);
  const rating = await (await row(page, "Rating")).boundingBox();
  await page.mouse.click(rating.x + rating.width - 45, rating.y + rating.height / 2);
  await settle(page, 500);
  await tapRow(page, "Notes");
  await top(page, "textarea").fill("zz capture note");
  await back(page);
  await tapRow(page, "Recipe Source");
  await page.locator("input.ALEditableTextCell:visible").nth(0).fill("zz capture source");
  await page.locator("input.ALEditableTextCell:visible").nth(1).fill("https://example.com/zz-capture");
  await back(page);
  await scrollSheet(page);
  await top(page, 'input[placeholder="Not Set"]').fill("4 servings");
  await tapRow(page, "Prep Time");
  await top(page, "select.ALEditRecipeDurationCellMinuteSelect").selectOption("15");
  await back(page);
  await tapRow(page, "Nutrition");
  await top(page, "textarea.ALTextView").fill("zz 100 kcal");
  await back(page);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page, 2_000);

  rec.mark("recipe: add ingredients to list");
  await page.getByText("Choose Destination List", { exact: true }).first().click();
  await settle(page, 800);
  await tapRow(page, LIST);
  await settle(page);
  await page.getByText("Add all ingredients to list", { exact: true }).first().click();
  await settle(page, 2_000);
  await confirmAlert(page).catch(() => {});

  rec.mark("recipe: edit (rename)");
  await detailMenu(page, "Edit Recipe");
  await top(page, 'input[placeholder="Recipe Name"]').fill(RECIPE_RENAMED);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page, 2_000);

  rec.mark("recipe: duplicate");
  await detailMenu(page, "Duplicate Recipe");
  await top(page, 'input[placeholder="Recipe Name"]').fill(RECIPE_COPY);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page, 2_000);

  rec.mark("recipe: delete (the duplicate)");
  await deleteOpenRecipe(page);

  await showRecipe(page, RECIPE_RENAMED);

  rec.mark("collection: create");
  await paneAdd(page, "Create Collection");
  await page.getByPlaceholder("Collection Name").fill(COLLECTION);
  await done(page);

  rec.mark("collection: add recipe (from the recipe)");
  await page.locator(".recipe-detail-collections-value").click();
  await settle(page, 800);
  await tapRow(page, COLLECTION);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page, 1_500);

  rec.mark("collection: rename");
  await page.locator(".ALTableCell", { hasText: COLLECTION }).locator(".ALTableCellAccessoryView").first().click();
  await settle(page, 600);
  await page.getByText("Collection Name & Icon", { exact: true }).click();
  await settle(page, 800);
  await top(page, "input").fill(COLLECTION_RENAMED);
  await done(page);

  rec.mark("collection: change sort order");
  await tapRow(page, COLLECTION_RENAMED);
  await page.getByText("Sort Recipes", { exact: true }).first().click();
  await settle(page, 600);
  await page.getByText("By Date Created", { exact: true }).click();
  await settle(page);

  rec.mark("collection: remove recipe (from the recipe)");
  await page.locator(".recipe-detail-collections-value").click();
  await settle(page, 800);
  await tapRow(page, COLLECTION_RENAMED);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page, 1_500);

  rec.mark("collection: add recipe (from the collection)");
  await page.locator(".ALPinnedTableFooterButton-Button", { hasText: "Add Recipe" }).click();
  await settle(page, 1_000);
  await top(page, "input.ALSearchFieldHeaderView-SearchField").click();
  await page.keyboard.type(RECIPE_RENAMED, { delay: 20 });
  await settle(page, 1_200);
  await top(page, ".ALTableCell", { hasText: "from zz capture source" }).click();
  await settle(page, 800);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page, 1_500);

  rec.mark("collection: remove recipe (from the collection)");
  await paneMenu(page, "Reorder & Delete Recipes");
  await page.locator(".ALRecipeGridCell-RecipeName", { hasText: RECIPE_RENAMED }).click();
  await settle(page, 500);
  await page.getByText("Remove from collection", { exact: true }).click();
  await settle(page, 800);
  await confirmAlert(page);
  await top(page, ".ALBarButtonDoneItem").click().catch(() => {});
  await settle(page, 600);
  await back(page);

  rec.mark("collection: reorder");
  await paneMenu(page, "Reorder & Delete Collections");
  const collection = page.locator(".ALTableCell", { hasText: COLLECTION_RENAMED }).first();
  const firstCollection = page.locator(".ALTableCell", { has: page.locator(".ALTableCellRemoveControl") }).first();
  await collection.dragTo(firstCollection, { targetPosition: { x: 100, y: 5 } });
  await settle(page, 2_000);

  rec.mark("collection: delete");
  await collection.locator(".ALTableCellRemoveControl").click();
  await settle(page, 500);
  await collection.locator(".ALTableCellConfirmRemoveControl").click();
  await settle(page, 800);
  await confirmAlert(page);
  await top(page, ".ALBarButtonDoneItem").click().catch(() => {});
  await settle(page);

  rec.mark("meal plan: add recipe to calendar");
  await showRecipe(page, RECIPE_RENAMED);
  await detailMenu(page, "Add to Calendar");
  await pickDay(page, 15);
  await settle(page);

  rec.mark("meal plan: add recipe to queue");
  await detailMenu(page, "Add to Queue");
  await settle(page);

  await tab(page, "Meal Plan");
  await page.locator(".ALSegmentedControlItemTitle", { hasText: "Calendar" }).click();
  await settle(page);
  await pickDay(page, 15);
  await page.getByText(RECIPE_RENAMED, { exact: true }).first().click();
  await settle(page);

  rec.mark("meal plan: plan leftovers");
  await detailMenu(page, "Plan Leftovers");
  await page.getByText("Add to Calendar", { exact: true }).last().click();
  await settle(page, 1_000);
  await pickDay(page, 16);

  rec.mark("meal plan: remove recipe event");
  await detailMenu(page, "Remove From Calendar");
  await confirmAlert(page).catch(() => {});

  rec.mark("meal plan: remove leftovers event");
  await pickDay(page, 16);
  await page.locator(".ALTableCellSubtitleText", { hasText: RECIPE_RENAMED }).first().click();
  await settle(page);
  await detailMenu(page, "Remove From Calendar");
  await confirmAlert(page).catch(() => {});

  rec.mark("queue: remove recipe");
  await page.locator(".ALSegmentedControlItemTitle", { hasText: "Queue" }).click();
  await settle(page);
  await page.locator(".ALRecipeGridCell-RecipeName", { hasText: RECIPE_RENAMED }).click();
  await settle(page);
  await page.mouse.click(429, 158);
  await settle(page, 600);
  await page.getByText("Remove From Queue", { exact: true }).click();
  await settle(page, 800);
  await confirmAlert(page).catch(() => {});
  await page.keyboard.press("Escape");
  await settle(page, 500);
  await page.locator(".ALSegmentedControlItemTitle", { hasText: "Calendar" }).click();
  await settle(page);

  rec.mark("recipe: delete");
  await tab(page, "Recipes");
  await showRecipe(page, RECIPE_RENAMED);
  await deleteOpenRecipe(page);

  rec.mark("list: delete (recipe ingredients list)");
  await tab(page, "Lists");
  await openList(page, LIST);
  await page.mouse.click(410, 65);
  await settle(page, 600);
  await page.locator("span", { hasText: /^Reorder, Move & Delete$/ }).click();
  await settle(page, 800);
  await page.locator(".ALTableCell", { hasText: LIST }).first().locator(".ALTableCellEditControl").click();
  await page.locator(".ALDestructiveBarButtonItem").click();
  await settle(page, 800);
  await confirmAlert(page);
  await top(page, ".ALBarButtonDoneItem").click().catch(() => {});
  await settle(page);
}
