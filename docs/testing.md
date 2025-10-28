# Testing

## Snapshot testing

Snapshot test verify the Protobuf output of functions in a byte-for-byte manner.

Testing in this way ensures that wire formats do not change, business logic
matche the web app, and that output remains the same across refactors.

### Creating new snapshots from AnyList Web

We'll use Chrome. Firefox may also be supported, but Safari appears not to have
the tools to export the request (including body) as a cURL command.

#### 1. Capture cURL command from the browser

1. Visit AnyList on the Web
1. Open DevTools
1. Swap to **Network** tab
1. Clear requests
1. Perform an action in AnyList on the web you want to validate in the library
1. Find the matching API request
1. Right click → **Copy → Copy as cURL**

#### 2. Create snapshot

On macOS,

```bash
pbpaste | python3 tools/curl_to_snapshot.py "add_item_webapp_2025_10_28"
```

#### 3. Write Rust test to match output

You'll need to get the appropriate IDs or timestamps required for the request.


#### 4. Run test

```bash
cargo test <your-new-test-file>
```

If it matches, you're good to go.
If it fails, see the diff with `cargo insta review`.
