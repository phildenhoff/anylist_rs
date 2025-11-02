#!/usr/bin/env python3
"""
Create a test from raw API response data.

Usage:
    pbpaste | python3 response_to_test.py <test_name>

Example:
    pbpaste | python3 response_to_test.py "parse_list_with_shared_users"

Input can be:
  - Raw protobuf bytes (binary)
  - Hex string (detected if all chars are [0-9a-fA-F])
  - Base64 string (detected if ends with = or looks like base64)
"""

import sys
import re
import base64
from pathlib import Path


def detect_and_decode(data):
    """Detect format and decode to raw bytes."""

    # If data is already bytes, use it directly
    if isinstance(data, bytes):
        print(f"✓ Using as raw binary ({len(data)} bytes)", file=sys.stderr)
        return data

    # Otherwise it's a string - check format
    data = data.strip()

    # Check if it's hex (all hex chars, even length)
    if re.match(r'^[0-9a-fA-F]+$', data) and len(data) % 2 == 0:
        print(f"✓ Detected hex format ({len(data)} chars)", file=sys.stderr)
        return bytes.fromhex(data)

    # Check if it's base64
    if re.match(r'^[A-Za-z0-9+/]+=*$', data):
        try:
            decoded = base64.b64decode(data)
            print(f"✓ Decoded base64 ({len(decoded)} bytes)", file=sys.stderr)
            return decoded
        except Exception:
            pass

    # Try to encode as UTF-8 or latin-1
    try:
        return data.encode('utf-8')
    except UnicodeEncodeError:
        try:
            return data.encode('latin-1')
        except Exception:
            print(f"⚠ Warning: Could not encode string, using as-is", file=sys.stderr)
            return data.encode('utf-8', errors='replace')


def write_test_file(test_name, protobuf_hex):
    """Write a test to src/lists.rs or create new test file."""

    test_function = f'''
    #[test]
    fn test_{test_name}() {{
        // Response from webapp: POST /data/user-data/get with shared list
        let response_hex = "{protobuf_hex}";

        let bytes = hex::decode(response_hex).unwrap();
        let user_data = PbUserDataResponse::decode(bytes.as_ref()).unwrap();

        let lists = lists_from_response(
            user_data.shopping_lists_response.unwrap()
        );

        // Verify shared_users was parsed
        // TODO: Update assertions based on actual data
        assert!(lists.len() > 0, "Should have at least one list");

        // Check if any list has shared users
        let has_shared_users = lists.iter().any(|l| !l.shared_users.is_empty());
        if has_shared_users {{
            let list_with_users = lists.iter().find(|l| !l.shared_users.is_empty()).unwrap();
            println!("List '{{}}' has {{}} shared users",
                list_with_users.name,
                list_with_users.shared_users.len()
            );

            for user in &list_with_users.shared_users {{
                println!("  - {{:?}} ({{:?}})", user.email, user.user_id);
            }}

            // Assertions
            assert!(!list_with_users.shared_users.is_empty());
            // Uncomment and adjust based on actual data:
            // assert_eq!(list_with_users.shared_users[0].email, Some("user@example.com".to_string()));
        }}
    }}
'''

    # Also create snapshot file
    snapshots_dir = Path('src/snapshots')
    snapshots_dir.mkdir(exist_ok=True)

    snapshot_filename = f"webapp_captures__{test_name}.snap"
    snapshot_path = snapshots_dir / snapshot_filename

    snapshot_content = f"""---
source: tools/response_to_test.py
expression: "Response from webapp: /data/user-data/get"
---
{protobuf_hex}
"""

    snapshot_path.write_text(snapshot_content)

    print(f"\n✅ Created snapshot: {snapshot_path}", file=sys.stderr)
    print(f"\n📝 Test code to add to src/lists.rs (in #[cfg(test)] mod tests):", file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    print(test_function, file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    print("\nOr run this to see the test code:", file=sys.stderr)
    print(f"  cat <<'EOF' >> /tmp/test_{test_name}.rs{test_function}EOF", file=sys.stderr)

    return snapshot_path


def main():
    if len(sys.argv) < 2:
        print("Usage: pbpaste | python3 response_to_test.py <test_name>", file=sys.stderr)
        print("", file=sys.stderr)
        print("Example:", file=sys.stderr)
        print("  pbpaste | python3 response_to_test.py 'parse_list_with_shared_users'", file=sys.stderr)
        sys.exit(1)

    test_name = sys.argv[1]

    # Try to read as binary first, fall back to text
    try:
        # Read binary data (for raw protobuf)
        data = sys.stdin.buffer.read()
        if not data:
            raise ValueError("No data")
    except Exception:
        # Fall back to text mode (for hex/base64)
        data = sys.stdin.read()
        if not data or not data.strip():
            print("Error: No input received", file=sys.stderr)
            print("Copy the response data and pipe it to this script", file=sys.stderr)
            sys.exit(1)

    try:
        # Decode to bytes
        protobuf_bytes = detect_and_decode(data)

        # Convert to hex
        protobuf_hex = protobuf_bytes.hex()

        # Write test and snapshot
        write_test_file(test_name, protobuf_hex)

        print(f"\n📸 Test and snapshot created!", file=sys.stderr)
        print(f"Captured {len(protobuf_bytes)} bytes", file=sys.stderr)

    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
