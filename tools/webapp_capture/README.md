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

The scenarios in `scenarios/` cover lists, items, categories, stores,
favourites and folders (`lists.mjs`), recipes, collections and recipe events
(`recipes.mjs`), and meal-plan notes, labels, the queue, pinned entries and
templates (`meal-plan.mjs`). They create data named `zz …`, delete it at the
end, and rely on the fixed 1400×900 viewport for a few header buttons. Pass
several `--scenario` flags to run them in one session.

The recorder reads request bodies over CDP (`Network.getRequestPostData`)
because Chromium returns no body from `postDataBuffer()` when the form holds a
`Blob`. The account email is replaced with a placeholder of the same length,
so protobuf length prefixes stay valid.

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

To save the operations parts of a run as test fixtures (request body only,
no response, email redacted):

```bash
node decode.mjs captures/2026_10_27 --fixtures ../../tests/fixtures/webapp_2026_10_27/lists --from 1 --to 70
```

`tests/it/webapp_captures.rs` decodes every fixture with the crate's types and
snapshots the handler IDs and the bytes prost drops. The decoded operations are
described in `docs/webapp-operations.md`.

## What the client can send

`bundle_handlers.py` scans the public web bundle for handler IDs and endpoints.
Its output for 2026-10-27 is in `docs/webapp-handlers.md`.
The scan misses some handler IDs the client builds at runtime; see
`docs/webapp-operations.md` for the ones the live capture found.

`bundle_schema.mjs` compares the protobuf schema embedded in the bundle with
`src/protobuf/anylist.proto`. Its output is in `docs/webapp-schema-diff.md`.
