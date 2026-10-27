# Testing

## Snapshot testing

Snapshot tests verify the Protobuf output of functions in a byte-for-byte manner.

Testing in this way ensures that wire formats do not change, business logic
matches the web app, and that output remains the same across refactors.

### Comparing with the web client's captures

`src/operations_webapp_tests.rs` checks each operation builder against
requests captured from the web client in `tests/fixtures/webapp_2026_10_27`
(see [webapp-operations.md](webapp-operations.md)). A test replays the
identifiers the web client generated, in the order the builder asks for them,
gives the builder the state the web client had, and compares every request.
When one differs, set `WEBAPP_DIFF_DIR` to a folder to get both sides decoded
and in hex.

### Creating snapshots from AnyList Web

We support two types of snapshot testing:

1. **Request snapshots** - Validate that we generate correct request bodies (operations we send to the API)
2. **Response snapshots** - Validate that we correctly parse API response data

#### Testing Requests (Operations)

Use this when you want to verify the protobuf messages your code **sends** to the API.

**Requirements:** Chrome or Firefox (Safari lacks the necessary export tools)

##### 1. Capture cURL command from browser

1. Visit AnyList on the Web
2. Open DevTools → **Network** tab
3. Clear requests
4. Perform an action in AnyList (e.g., add an item, create a list)
5. Find the matching API request
6. Right click → **Copy → Copy as cURL**

##### 2. Create snapshot

```bash
pbpaste | python3 tools/curl_to_snapshot.py "add_item_webapp_2025_10_28"
```

This extracts the protobuf body from the cURL command and creates a snapshot file.

##### 3. Write Rust test

Create a test that builds the same operation and compares its encoded bytes to the snapshot.

##### 4. Run test

```bash
cargo test <your-test-name>
```

If it matches, you're good! If it fails, see the diff with `cargo insta review`.

---

#### Testing Responses (Parsing)

Use this when you want to verify that your code correctly **parses** API responses.

**Why this matters:** Ensures `shared_users`, `user_id`, and other fields are properly extracted from protobuf.

##### 1. Capture response using a proxy

**IMPORTANT:** Browser DevTools and clipboard corrupt binary protobuf data with UTF-8 conversion.
You MUST use a proxy tool to capture raw binary responses.

**Recommended tools:**
- **Proxyman** (macOS) - https://proxyman.io
- **Charles Proxy** (cross-platform)
- **mitmproxy** (CLI, cross-platform)

**Setup with Proxyman:**

1. Install and launch Proxyman
2. Configure your system/browser to use Proxyman's proxy
3. In Proxyman, enable SSL Proxying for `*.anylist.com`
4. In AnyList web app, perform the action (e.g., load a shared list)
5. Find the request in Proxyman (e.g., `POST /data/user-data/get`)
6. Right-click the request → **Export** → **Response Body** → Save as `.bin` file

**Example:**
```bash
# Save the response body to:
/tmp/api_response.bin
```

##### 2. Create test and snapshot

```bash
cat /tmp/api_response.bin | python3 tools/response_to_test.py "parse_list_with_shared_users"
```

This will:
- Create a snapshot at `src/snapshots/webapp_captures__<name>.snap`
- Output test code you can copy into your test file

##### 3. Add test to source

Copy the generated test code into the appropriate test module (e.g., `src/lists.rs`).

##### 4. Run test

```bash
cargo test parse_list_with_shared_users -- --nocapture
```

The test will decode the response, parse it, and verify fields are populated correctly.

---

**Troubleshooting:** If you see `efbfbd` (UTF-8 replacement character `�`) in the hex output,
the binary data was corrupted. Re-capture using a proxy tool, not the clipboard.

### Checking handler IDs against the web client

The web client's bundle is public, so a handler ID can be confirmed without
signing in. Read the current script URL from the page at
<https://www.anylist.com/web> (it is `/static/webapp/js/app.min.js?v=...`) and
grep the file:

```bash
curl -sL "https://www.anylist.com/static/webapp/js/app.min.js?v=<hash>" -o /tmp/app.min.js
grep -oE '"[a-z-]*recipe-collection[a-z-]*"' /tmp/app.min.js | sort -u
```

A 200 from an update endpoint does not mean the operation was applied. AnyList
returns 200 for unknown handler IDs and silently drops the operation, so a live
check must read the data back and assert on it.

Safari can still produce request captures even without "Copy as cURL": enable
"Allow JavaScript from Apple Events" in Safari's Developer settings, patch
`XMLHttpRequest.prototype.send` in the signed-in tab to stash request bodies,
perform the action, and read the bodies back out. The `webapp_*_2026_10_25`
collection snapshots were captured this way.

### Debugging failing snapshot tests

If the base64-encoded snapshot doesn't match the output from your function,
you'll want to compare the two outputs. Thankfully, there's already a tool for
decoding base64-stored protobuf messages in `tools/decode_snapshot.py`. That can
be used to turn the stored snapshot into bin files that you can pass into protoc
to view the intended protobuf message.

#### Naming rules for webapp-derived "golden" snapshots

If you pull a protobuf message from the webapp, prepend the test name with
`webapp_` (after the appropriate `insta` naming, likely
`anylist_rs__operations__tests__`) and include the date it was saved at the end
in the format `_YYYY_MM_DD`.
