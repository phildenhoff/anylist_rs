// Helpers shared by the capture scenarios.
//
// The web client renders its own table and navigation widgets rather than
// form controls, so these locate things by the class names it uses. Modal
// sheets stack, so "the visible one, last in the DOM" is the topmost.

export const PREFIX = "zz capture";

export const settle = (page, ms = 1_500) => page.waitForTimeout(ms);

export function top(page, selector, opts) {
  return page.locator(`${selector}:visible`, opts).last();
}

export async function done(page) {
  await top(page, ".ALBarButtonDoneItem").click();
  await settle(page);
}

export async function cancel(page) {
  await top(page, ".ALBarButtonItem", { hasText: /^Cancel$/ }).click();
  await settle(page, 800);
}

const exactly = (text) =>
  typeof text === "string" ? new RegExp(`^\\s*${text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*$`) : text;

// The topmost table row whose own label is exactly `text`.
export async function row(page, text) {
  const el = top(page, ".ALTableCell", {
    has: page.locator(".ALTableCellContents *, .ALTableCellValue1Text, .ALTableCellTitleText", { hasText: exactly(text) }),
  });
  await el.scrollIntoViewIfNeeded();
  return el;
}

export async function tapRow(page, text) {
  await (await row(page, text)).locator(".ALTableCellContents, .ALTableCellValue1Text").first().click();
  await settle(page, 800);
}

// Clicks the red delete control on a row in edit mode, then Remove.
export async function removeRow(page, text) {
  const r = await row(page, text);
  await r.locator(".ALTableCellRemoveControl").click();
  await settle(page, 500);
  await r.locator(".ALTableCellConfirmRemoveControl").click();
  await settle(page, 800);
}

export async function dismissIntro(page) {
  const intro = page.getByText("Getting Started with AnyList for the Web");
  if (await intro.isVisible().catch(() => false)) await done(page);
}

export async function tab(page, name) {
  await page.locator(".ALVerticalTabBarItem", { hasText: name }).click();
  await settle(page);
}

export async function back(page) {
  await top(page, ".NavigationBackButton").click();
  await settle(page, 800);
}

// The topmost list row for the item called `name`. The name cell also holds
// the quantity, as "milk (2 L)".
export function item(page, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return top(page, ".ALTableCell", {
    has: page.locator(".ALShoppingListCell-ItemName", { hasText: new RegExp(`^\\s*${escaped}(\\s|$)`) }),
  });
}

export async function crossOff(page, name) {
  await item(page, name).locator(".ALShoppingListCell-Checkbox").click();
  await settle(page);
}

// Selects a list item by name (single click), ready for an action bar button.
export async function selectItem(page, name) {
  await item(page, name).locator(".ALShoppingListCell-ItemName").click();
  await settle(page, 500);
}

export async function itemAction(page, id) {
  await page.locator(`#${id}`).click();
  await settle(page, 800);
}

export async function confirmAlert(page) {
  await top(page, ".ALAlertView-Confirm").click();
  await settle(page);
}

export async function openList(page, name) {
  await page.locator(".ALTableCellTitleText", { hasText: exactly(name) }).click();
  await settle(page);
}

export async function createList(page, name) {
  // The "+" in the Lists pane header; its bar items carry no distinguishing
  // class, and the viewport is fixed at 1400x900.
  await page.mouse.click(364, 65);
  await settle(page, 600);
  await page.getByText("Create List", { exact: true }).click();
  await settle(page, 800);
  await page.getByPlaceholder("List Name").fill(name);
  await done(page);
}

export async function addItem(page, name) {
  await top(page, ".ALAddItemBarTextFieldContainer").click();
  await page.keyboard.type(name);
  await page.keyboard.press("Enter");
  await settle(page, 1_000);
  await page.keyboard.press("Escape");
  await settle(page, 300);
}

// Finds a recipe with the Recipes pane search field and opens it.
export async function openRecipe(page, name) {
  const search = page.getByPlaceholder("Search Recipes").first();
  await search.click();
  await search.fill("");
  await page.keyboard.type(name, { delay: 20 });
  await settle(page, 1_200);
  await page.locator(".ALTableCell:visible", { hasText: name }).first().click();
  await settle(page);
}

// Picks a day in the current month of the topmost calendar; days from the
// neighbouring months are drawn grey.
export async function pickDay(page, day) {
  const cells = page.locator("span.day-number:visible", { hasText: new RegExp(`^${day}$`) });
  for (let i = (await cells.count()) - 1; i >= 0; i--) {
    const grey = await cells.nth(i).evaluate((e) => getComputedStyle(e).color === "rgb(170, 170, 170)");
    if (!grey) {
      await cells.nth(i).click();
      await settle(page, 1_000);
      return;
    }
  }
  throw new Error(`day ${day} not found`);
}
