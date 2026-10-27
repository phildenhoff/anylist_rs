# Web client capture

Records every request the AnyList web client sends, so handler IDs and payloads
can be checked against the real thing and saved as golden snapshots.

AnyList POSTs protobuf messages as a multipart form part (usually named
`operations`). A 200 does not mean the operation was applied: unknown handler
IDs are accepted and silently dropped, so always read the data back.

## Setup

```bash
cd tools/webapp_capture
bun install            # protobufjs
bunx playwright install chromium   # skip if Playwright is already installed globally
```

## Capture

Credentials are read from `ANYLIST_EMAIL` and `ANYLIST_PASSWORD` and are never
written to disk. Passwords, the XSRF field, cookies, `Authorization` headers and
auth tokens are redacted before a request is saved. Use a throwaway account.

```bash
# Interactive: type a label, do the action in the browser, repeat; "quit" ends.
HEADED=1 node capture.mjs --out captures/2026_10_27

# Scripted: each scenario module exports `async ({ page, rec }) => {}` and
# calls rec.mark("label") before each action.
node capture.mjs --out captures/2026_10_27 --scenario scenarios/recipes.mjs
```

Each request becomes `captures/<dir>/<seq>-<endpoint>.json`, and websocket
frames go to `websocket.json`. `captures/` is git-ignored because responses
contain the account's data.

## Decode

```bash
node decode.mjs captures/2026_10_27
```

This writes `decoded/<seq>-<endpoint>.json` (typed decode against
`src/protobuf/anylist.proto` where the endpoint's type is known, a schemaless
field dump always, and the hex of each part) and prints `summary.md`: one row
per request with its label, endpoint and handler IDs.

To turn a request into a golden snapshot:

```bash
node decode.mjs captures/2026_10_27 --snapshot 12 webapp_new_recipe_collection_2026_10_27
```

## What the client can send

`bundle_handlers.py` scans the public web bundle for handler IDs and endpoints.
Its output for 2026-10-27 is in `docs/webapp-handlers.md`.
