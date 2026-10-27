// Lists, items, categories, stores, favourites, folders and list ordering.
//
// Creates two throwaway lists and a folder, exercises every list and item
// action the web client offers, then deletes everything it made.

import {
  addItem,
  back,
  confirmAlert,
  createList,
  crossOff,
  dismissIntro,
  done,
  item,
  itemAction,
  openList,
  removeRow,
  row,
  selectItem,
  settle,
  tapRow,
  tab,
  top,
} from "./lib.mjs";

const LIST = "zz capture list";
const LIST_RENAMED = "zz capture list 2";
const LIST_B = "zz capture list B";
const FOLDER = "zz capture folder";
const FOLDER_RENAMED = "zz capture folder 2";
const CATEGORY = "zz Capture Cat";
const STORE = "zz capture store";

async function listMenu(page, name, action) {
  await page.locator(".ALTableCell", { hasText: name }).locator(".ALTableCellAccessoryView").first().click();
  await settle(page, 600);
  await page.getByText(action, { exact: true }).click();
  await settle(page, 800);
}

// The "..." menus in the Lists pane header and the list header.
async function listsPaneMenu(page, action) {
  await page.mouse.click(410, 65);
  await settle(page, 600);
  await page.locator("span", { hasText: new RegExp(`^${action}$`) }).click();
  await settle(page, 800);
}

async function listHeaderMenu(page, action) {
  await page.mouse.click(1380, 65);
  await settle(page, 600);
  await page.getByText(action, { exact: true }).click();
  await settle(page, 800);
}

// Clicks Done on each open sheet until none is left.
async function closeSheets(page) {
  const button = page.locator(".ALBarButtonDoneItem:visible");
  while (await button.count()) {
    await button.last().click();
    await settle(page);
  }
}

async function toggleRow(page, text) {
  const r = await row(page, text);
  const box = await r.boundingBox();
  await page.mouse.click(box.x + box.width - 40, box.y + box.height / 2);
  await settle(page);
}

export default async function lists({ page, rec }) {
  page.setDefaultTimeout(10_000);
  await dismissIntro(page);
  await tab(page, "Lists");

  rec.mark("list: create");
  await createList(page, LIST);

  rec.mark("item: add");
  for (const name of ["zz milk", "zz bread", "zz eggs"]) await addItem(page, name);

  rec.mark("item: cross off");
  await crossOff(page, "zz bread");
  rec.mark("item: uncross");
  await crossOff(page, "zz bread");

  rec.mark("item: edit note and quantity");
  await selectItem(page, "zz milk");
  await itemAction(page, "EditItemAction");
  await top(page, ".ALEditItemNoteTextArea").fill("zz note");
  await page.locator('input[placeholder="Not set"]:visible').first().fill("2 L");
  await top(page, ".NavigationTitleView").click();
  await settle(page);

  rec.mark("item: change category");
  await tapRow(page, "Category");
  await tapRow(page, "Beverages");
  await settle(page);

  rec.mark("category: create");
  await tapRow(page, "Category");
  await page.getByText("New Category", { exact: true }).click();
  await settle(page, 800);
  await top(page, "input.ALEditableTextCell").fill(CATEGORY);
  await done(page);

  rec.mark("item: assign custom category");
  await tapRow(page, CATEGORY);
  await settle(page);

  rec.mark("category: rename");
  await page.getByText("Edit Categories", { exact: true }).click();
  await settle(page, 800);
  await tapRow(page, CATEGORY);
  await top(page, "input.ALEditableTextCell").fill(`${CATEGORY} 2`);
  await done(page);
  await done(page);

  rec.mark("category: delete");
  await page.getByText("Edit Categories", { exact: true }).click();
  await settle(page, 800);
  await removeRow(page, `${CATEGORY} 2`);
  await closeSheets(page);

  rec.mark("list: rename");
  await listMenu(page, LIST, "List Name & Icon");
  await tapRow(page, "List Name");
  await top(page, "input").fill(LIST_RENAMED);
  await page.keyboard.press("Enter");
  await settle(page);
  await done(page);

  rec.mark("list settings: toggle show prices");
  await listMenu(page, LIST_RENAMED, "List Settings");
  await toggleRow(page, "Show Prices in List");

  rec.mark("store: create");
  await tapRow(page, "Stores");
  await page.getByText("Create a new store", { exact: true }).click();
  await settle(page, 800);
  await top(page, "input").fill(STORE);
  await done(page);

  rec.mark("store: rename");
  await tapRow(page, STORE);
  await top(page, "input").fill(`${STORE} 2`);
  await done(page);
  await back(page);
  await done(page);

  rec.mark("item: assign store");
  await selectItem(page, "zz bread");
  await itemAction(page, "AssignStoresAction");
  await tapRow(page, `${STORE} 2`);
  await done(page);

  rec.mark("item: edit price");
  await selectItem(page, "zz bread");
  await itemAction(page, "EditPricesAction");
  await top(page, "input.ALItemPriceField").fill("3.49");
  await done(page);

  rec.mark("store: delete (with an item assigned)");
  await listMenu(page, LIST_RENAMED, "List Settings");
  await tapRow(page, "Stores");
  await tapRow(page, `${STORE} 2`);
  await top(page, ".ALTableCellDeleteButtonTitle").click();
  await settle(page, 800);
  await confirmAlert(page).catch(() => {});
  await closeSheets(page);
  await back(page).catch(() => {});
  await closeSheets(page);

  rec.mark("favourites: add list item to favourites");
  await selectItem(page, "zz eggs");
  await itemAction(page, "FavoritesAction");

  rec.mark("favourites: add item in favourites panel");
  await page.getByTitle("Favorite Items (⌥f)").click();
  await settle(page);
  await addItem(page, "zz fav butter");

  rec.mark("favourites: add favourite to list");
  await item(page, "zz fav butter").locator(".ALShoppingListCell-ItemName").click();
  await settle(page, 500);
  await top(page, "#AddToListAction").click();
  await settle(page);

  rec.mark("favourites: delete favourite");
  await item(page, "zz fav butter").locator(".ALShoppingListCell-ItemName").click();
  await settle(page, 500);
  await top(page, "#DeleteAction").click();
  await settle(page, 800);
  await confirmAlert(page);
  await done(page);

  rec.mark("item: delete");
  await selectItem(page, "zz fav butter");
  await itemAction(page, "DeleteAction");
  await confirmAlert(page).catch(() => {});

  rec.mark("list: create second list");
  await createList(page, LIST_B);
  await openList(page, LIST_RENAMED);

  rec.mark("item: copy to other list");
  await selectItem(page, "zz eggs");
  await itemAction(page, "MoveCopyAction");
  await page.getByText("Copy Item", { exact: true }).click();
  await settle(page, 800);
  await tapRow(page, LIST_B);
  await settle(page);

  rec.mark("item: move to other list");
  await selectItem(page, "zz milk");
  await itemAction(page, "MoveCopyAction");
  await page.getByText("Move Item", { exact: true }).click();
  await settle(page, 800);
  await tapRow(page, LIST_B);
  await settle(page);

  rec.mark("list: uncross all");
  await crossOff(page, "zz eggs");
  await listHeaderMenu(page, "Uncross All Items");
  await confirmAlert(page).catch(() => {});

  rec.mark("list: delete crossed off");
  await crossOff(page, "zz eggs");
  await listHeaderMenu(page, "Delete Crossed Off Items");
  await confirmAlert(page).catch(() => {});

  rec.mark("list: delete all items");
  await openList(page, LIST_B);
  await listHeaderMenu(page, "Delete All Items");
  await confirmAlert(page).catch(() => {});

  rec.mark("folder: create");
  await page.mouse.click(364, 65);
  await settle(page, 600);
  await page.getByText("Create Folder", { exact: true }).click();
  await settle(page, 800);
  await page.getByPlaceholder("Folder Name").fill(FOLDER);
  await done(page);

  rec.mark("folder: move list into folder");
  await listMenu(page, LIST_B, "Move to Folder");
  await page.getByText(FOLDER, { exact: true }).last().click();
  await settle(page);

  rec.mark("folder: rename");
  await listMenu(page, FOLDER, "Folder Name & Icon");
  await top(page, "input").fill(FOLDER_RENAMED);
  await done(page);

  rec.mark("lists: reorder");
  await listsPaneMenu(page, "Reorder, Move & Delete");
  const folderRow = page.locator(".ALTableCell", { hasText: FOLDER_RENAMED }).first();
  const listRow = page.locator(".ALTableCell", { hasText: LIST_RENAMED }).first();
  await folderRow.dragTo(listRow, { targetPosition: { x: 100, y: 5 } });
  await settle(page, 2_000);
  // Leave edit mode: the drag can leave the dragged row selected.
  await closeSheets(page);

  rec.mark("list: delete");
  await listsPaneMenu(page, "Reorder, Move & Delete");
  await listRow.locator(".ALTableCellEditControl").click();
  await page.locator(".ALDestructiveBarButtonItem").click();
  await settle(page, 800);
  await confirmAlert(page);
  await closeSheets(page);

  rec.mark("folder: delete (with a list inside)");
  await listsPaneMenu(page, "Reorder, Move & Delete");
  await folderRow.locator(".ALTableCellEditControl").click();
  await page.locator(".ALDestructiveBarButtonItem").click();
  await settle(page, 800);
  await confirmAlert(page);
  await closeSheets(page);
}
