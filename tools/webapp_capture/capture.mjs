#!/usr/bin/env node
// Signs into the AnyList web app in Chromium and records every API request.
//
// Credentials come only from ANYLIST_EMAIL and ANYLIST_PASSWORD; they are
// never written to disk.
//
// Usage:
//   node capture.mjs --out captures/2026_10_27                  # interactive
//   node capture.mjs --out captures/x --scenario scenarios/recipes.mjs
//   HEADED=1 node capture.mjs ...                               # show the browser
//
// Interactive mode reads labels from stdin: type a label (e.g. "add item"),
// press Enter, perform the action in the browser, then type the next label.
// Type "quit" to finish.

import { createRequire } from "node:module";
import { createInterface } from "node:readline/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { Recorder } from "./recorder.mjs";

// CommonJS resolution also finds a globally installed Playwright (NODE_PATH).
const { chromium } = createRequire(import.meta.url)("playwright");

export async function signIn(page) {
  const email = process.env.ANYLIST_EMAIL;
  const password = process.env.ANYLIST_PASSWORD;
  if (!email || !password) throw new Error("Set ANYLIST_EMAIL and ANYLIST_PASSWORD");

  await page.goto("https://www.anylist.com/web");
  const emailBox = page.locator('input[name="sign_in_email"], input[type="email"]').first();
  if (!(await emailBox.isVisible({ timeout: 10_000 }).catch(() => false))) {
    // Already signed in from a stored session.
    return;
  }
  await emailBox.fill(email);
  await page.locator('input[name="sign_in_pw"], input[type="password"]').first().fill(password);
  await page.locator('button[type="submit"], input[type="submit"], button:has-text("Sign In")').first().click();
  await page.waitForLoadState("networkidle");
}

async function main() {
  const { values: args } = parseArgs({
    options: {
      out: { type: "string", default: `captures/${new Date().toISOString().slice(0, 10).replaceAll("-", "_")}` },
      scenario: { type: "string", multiple: true },
      "storage-state": { type: "string" },
    },
  });

  const browser = await chromium.launch({ headless: !process.env.HEADED });
  const context = await browser.newContext({
    viewport: { width: 1400, height: 900 },
    // Session cookies only live in memory unless --storage-state is passed.
    storageState: args["storage-state"],
  });
  const page = await context.newPage();
  const rec = new Recorder(resolve(args.out));
  await rec.attach(page);

  rec.mark("sign-in");
  await signIn(page);
  await rec.flush();

  if (args.scenario?.length) {
    for (const file of args.scenario) {
      const { default: run } = await import(pathToFileURL(resolve(file)).href);
      await run({ page, rec });
      await rec.flush();
    }
  } else {
    const rl = createInterface({ input: process.stdin, output: process.stderr });
    for (;;) {
      const label = (await rl.question("label> ")).trim();
      await page.waitForTimeout(1_000);
      await rec.flush();
      if (label === "quit") break;
      if (label) rec.mark(label);
    }
    rl.close();
  }

  await page.waitForTimeout(2_000);
  await rec.flush();
  await browser.close();
  console.error(`[capture] wrote ${rec.seq} requests to ${rec.outDir}`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
