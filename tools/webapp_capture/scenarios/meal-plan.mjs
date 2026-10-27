// Meal plan notes, labels, pinned entries, the queue and templates.
//
// Works on days 20 to 22 of the current month and deletes everything it
// creates. Recipe events are covered by recipes.mjs.

import { back, confirmAlert, dismissIntro, done, pickDay, removeRow, settle, tab, tapRow, top } from "./lib.mjs";

const NOTE = "zz capture note event";
const LABEL = "zz capture label";
const TEMPLATE = "zz capture template";
const TEMPLATE_NOTE = "zz template note";

async function mealPlanMenu(page, action) {
  await page.mouse.click(980, 65);
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

async function segment(page, name) {
  await page.locator(".ALSegmentedControlItemTitle", { hasText: name }).click();
  await settle(page);
}

// Selects the entries named `title` on the selected day and runs a bulk action.
async function bulk(page, title, action) {
  await mealPlanMenu(page, "Select & Edit Entries");
  await page.getByText(title, { exact: true }).first().click();
  await page.getByText("Actions", { exact: true }).click();
  await settle(page, 600);
  await page.getByText(action, { exact: true }).click();
  await settle(page, 1_000);
}

export default async function mealPlan({ page, rec }) {
  page.setDefaultTimeout(10_000);
  await dismissIntro(page);
  await tab(page, "Meal Plan");
  await segment(page, "Calendar");
  await pickDay(page, 20);

  rec.mark("meal plan: create note event");
  await page.locator(".ALMealPlanEventListSectionHeader-AddEntriesButton").first().click();
  await settle(page, 800);
  await page.getByText("Create Note", { exact: true }).click();
  await settle(page, 800);
  await top(page, 'textarea[placeholder="Title"]').fill(NOTE);
  await top(page, 'textarea[placeholder="Details"]').fill("zz details");

  rec.mark("label: create");
  await tapRow(page, "Label");
  await page.getByText("Create a new label", { exact: true }).click();
  await settle(page, 800);
  await page.getByPlaceholder("Label Name").fill(LABEL);
  await done(page);

  rec.mark("meal plan: note event ingredient, then save");
  await page.getByText("Add Ingredient", { exact: true }).click();
  await settle(page, 800);
  await top(page, 'textarea[placeholder="Name"]').fill("zz event ingredient");
  await page.locator('input[placeholder="Not set"]:visible').first().fill("2");
  await done(page);
  await page.getByText("Add to Calendar", { exact: true }).last().click();
  await settle(page, 1_500);
  await page.getByText("Done", { exact: true }).last().click().catch(() => {});
  await settle(page);

  await page.getByText(NOTE, { exact: true }).first().click();
  await settle(page);

  rec.mark("meal plan: edit event title");
  await top(page, 'textarea[placeholder="Title"]').fill(`${NOTE} 2`);
  await page.locator(".StarterListTitleViewTitle").last().click();
  await settle(page);

  rec.mark("meal plan: pin entry");
  await detailMenu(page, "Pin Entry");

  rec.mark("meal plan: change date");
  await detailMenu(page, "Change Date");
  await pickDay(page, 21);

  rec.mark("meal plan: add to queue");
  await detailMenu(page, "Add to Queue");

  rec.mark("meal plan: remove event from calendar");
  await detailMenu(page, "Remove From Calendar");
  await confirmAlert(page).catch(() => {});

  rec.mark("queue: remove entry");
  await segment(page, "Queue");
  await page.locator(".ALRecipeGridCell-RecipeName", { hasText: `${NOTE} 2` }).click();
  await settle(page);
  await page.mouse.click(429, 158);
  await settle(page, 600);
  await page.getByText("Remove From Queue", { exact: true }).click();
  await settle(page, 800);
  await confirmAlert(page).catch(() => {});
  await page.keyboard.press("Escape");
  await settle(page, 500);
  await segment(page, "Calendar");

  rec.mark("pinned: remove entry");
  await mealPlanMenu(page, "Pinned Entries");
  await top(page, ".ALRecipeGridCell, .ALTableCell", { hasText: NOTE }).locator("text=•••").click().catch(async () => {
    const card = await top(page, ".ALRecipeGridCell", { hasText: NOTE }).boundingBox();
    await page.mouse.click(card.x + card.width - 19, card.y + 19);
  });
  await settle(page, 600);
  await page.getByText("Remove From Pinned Entries", { exact: true }).click();
  await settle(page, 800);
  await confirmAlert(page).catch(() => {});
  await done(page).catch(() => {});

  rec.mark("template: create");
  await mealPlanMenu(page, "Templates");
  const add = await top(page, ".ALCircularButton, .ALFloatingActionButton").boundingBox().catch(() => null);
  if (add) await page.mouse.click(add.x + add.width / 2, add.y + add.height / 2);
  else await page.mouse.click(947, 729);
  await settle(page, 1_000);

  rec.mark("template: rename");
  await tapRow(page, "Name");
  await top(page, "input").fill(TEMPLATE);
  await page.keyboard.press("Enter");
  await settle(page);

  rec.mark("template: set number of days");
  await tapRow(page, "Number of Days").catch(() => {});
  await page.locator("span:visible", { hasText: /^2 days$/ }).last().click();
  await settle(page);

  rec.mark("template: add note entry");
  await page.locator(".ALTemplateDayHeader-AddEntriesButton:visible").first().click();
  await settle(page, 800);
  await page.getByText("Create Note", { exact: true }).last().click();
  await settle(page, 800);
  await top(page, 'textarea[placeholder="Title"]').fill(TEMPLATE_NOTE);
  await page.getByText(/^Add to /).last().click();
  await settle(page, 1_500);
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page);
  await back(page);
  await done(page);

  rec.mark("template: apply to calendar");
  await pickDay(page, 20);
  await page.locator(".ALMealPlanEventListSectionHeader-AddEntriesButton").first().click();
  await settle(page, 800);
  await page.getByText("Templates", { exact: true }).last().click();
  await settle(page, 1_000);
  await tapRow(page, TEMPLATE);
  await page.getByText("Add to Calendar", { exact: true }).last().click();
  await settle(page, 2_000);
  await page.getByText("Done", { exact: true }).last().click().catch(() => {});
  await settle(page);

  rec.mark("meal plan: copy entries");
  await bulk(page, TEMPLATE_NOTE, "Copy Entries");
  await pickDay(page, 21);

  rec.mark("meal plan: move entries");
  await bulk(page, TEMPLATE_NOTE, "Move Entries");
  await pickDay(page, 22);

  for (const day of [21, 22]) {
    rec.mark(`meal plan: remove entries (day ${day})`);
    await pickDay(page, day);
    await bulk(page, TEMPLATE_NOTE, "Remove Entries");
    await confirmAlert(page).catch(() => {});
    await top(page, ".ALBarButtonDoneItem").click().catch(() => {});
    await settle(page, 800);
  }

  rec.mark("label: rename");
  await mealPlanMenu(page, "Labels");
  await tapRow(page, LABEL);
  await top(page, "input").fill(`${LABEL} 2`);
  await done(page);

  rec.mark("label: delete");
  await removeRow(page, `${LABEL} 2`);
  await confirmAlert(page).catch(() => {});
  await done(page);

  rec.mark("template: delete");
  await mealPlanMenu(page, "Templates");
  await tapRow(page, TEMPLATE);
  await page.mouse.click(970, 158);
  await settle(page, 600);
  await page.getByText("Delete Template", { exact: true }).click();
  await settle(page, 800);
  await confirmAlert(page).catch(() => {});
  await top(page, ".ALBarButtonDoneItem").click().catch(() => {});
  await settle(page);
}
