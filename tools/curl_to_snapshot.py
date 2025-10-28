#!/usr/bin/env python3
"""
Extract protobuf from browser cURL and save as insta snapshot.

Usage:
    pbpaste | python3 curl_to_snapshot.py "add_item_from_webapp_2025_01_27"

This creates: src/snapshots/webapp_captures__add_item_from_webapp_2025_01_27.snap
"""

import sys
import re
from pathlib import Path

def parse_curl_data(curl_text):
    """Extract protobuf bytes from cURL --data-raw."""

    # Find the boundary
    boundary_match = re.search(r'boundary=([^\s\'";]+)', curl_text)
    if not boundary_match:
        raise ValueError("Could not find multipart boundary in cURL command")

    boundary = boundary_match.group(1)
    print(f"✓ Found boundary: {boundary}", file=sys.stderr)

    # Extract --data-raw content
    data_match = re.search(r'--data-raw\s+(.+?)(?:\s*$|\s+--)', curl_text, re.DOTALL)
    if not data_match:
        raise ValueError("Could not find --data-raw in cURL command")

    data = data_match.group(1).strip().strip("'").strip('"')

    # Find operations field
    operations_pattern = r'Content-Disposition:\s*form-data;\s*name="operations"'
    if not re.search(operations_pattern, data):
        raise ValueError("Could not find 'operations' field in multipart data")

    print(f"✓ Found operations field", file=sys.stderr)

    # Split on operations header
    parts = re.split(operations_pattern, data, maxsplit=1)
    if len(parts) < 2:
        raise ValueError("Could not parse operations field")

    after_header = parts[1]

    # Find start of body (after \r\n\r\n or similar)
    # Handle both escaped and unescaped versions
    body_start = 0
    for pattern in [r'\\r\\n\\r\\n', r'\r\n\r\n', r'\\n\\n', r'\n\n']:
        match = re.search(pattern, after_header)
        if match:
            body_start = match.end()
            break

    if body_start == 0:
        # Try just after first newline
        match = re.search(r'[\r\n]+', after_header)
        if match:
            body_start = match.end()

    body = after_header[body_start:]

    # Find end of body (before next boundary)
    boundary_end_patterns = [
        rf'\\r\\n--{re.escape(boundary)}',
        rf'\r\n--{re.escape(boundary)}',
        rf'\\n--{re.escape(boundary)}',
        rf'\n--{re.escape(boundary)}',
    ]

    end_pos = len(body)
    for pattern in boundary_end_patterns:
        match = re.search(pattern, body)
        if match:
            end_pos = match.start()
            break

    body = body[:end_pos]

    # Convert escape sequences to bytes
    result_bytes = bytearray()
    i = 0

    while i < len(body):
        if body[i:i+2] == r'\n':
            result_bytes.append(ord('\n'))
            i += 2
        elif body[i:i+2] == r'\r':
            result_bytes.append(ord('\r'))
            i += 2
        elif body[i:i+2] == r'\t':
            result_bytes.append(ord('\t'))
            i += 2
        elif body[i:i+6].startswith(r'\u'):
            # Unicode escape: \u0093
            hex_str = body[i+2:i+6]
            try:
                value = int(hex_str, 16)
                # Treat as raw byte value for protobuf
                result_bytes.append(value & 0xFF)
                i += 6
            except ValueError:
                result_bytes.append(ord(body[i]))
                i += 1
        elif body[i:i+2] == r'\\':
            result_bytes.append(ord('\\'))
            i += 2
        else:
            result_bytes.append(ord(body[i]))
            i += 1

    # Remove any trailing newlines/whitespace added during extraction
    while result_bytes and result_bytes[-1] in (ord('\n'), ord('\r'), ord(' ')):
        result_bytes.pop()

    print(f"✓ Extracted {len(result_bytes)} bytes", file=sys.stderr)
    return bytes(result_bytes)


def write_snapshot(snapshot_name, protobuf_bytes):
    """Write snapshot file in insta format."""

    hex_string = protobuf_bytes.hex()

    # Create snapshots directory if needed
    snapshots_dir = Path('src/snapshots')
    snapshots_dir.mkdir(exist_ok=True)

    # Create snapshot file
    snapshot_filename = f"webapp_captures__{snapshot_name}.snap"
    snapshot_path = snapshots_dir / snapshot_filename

    # Write in insta format
    snapshot_content = f"""---
source: tools/curl_to_snapshot.py
expression: "Captured from webapp"
---
{hex_string}
"""

    snapshot_path.write_text(snapshot_content)

    print(f"\n✅ Wrote snapshot: {snapshot_path}", file=sys.stderr)
    print(f"\nTo test against this:", file=sys.stderr)
    print(f"  1. Build your operation with test params", file=sys.stderr)
    print(f"  2. Use: insta::assert_snapshot!(hex::encode(&buf));", file=sys.stderr)
    print(f"  3. Name your test: test_{snapshot_name}", file=sys.stderr)

    return snapshot_path


def main():
    if len(sys.argv) < 2:
        print("Usage: pbpaste | python3 curl_to_snapshot.py <snapshot_name>", file=sys.stderr)
        print("", file=sys.stderr)
        print("Example:", file=sys.stderr)
        print("  pbpaste | python3 curl_to_snapshot.py 'add_item_webapp_2025_01_27'", file=sys.stderr)
        sys.exit(1)

    snapshot_name = sys.argv[1]

    # Read cURL from stdin
    curl_text = sys.stdin.read()

    if not curl_text.strip():
        print("Error: No input received", file=sys.stderr)
        print("Copy the cURL command and pipe it to this script", file=sys.stderr)
        sys.exit(1)

    try:
        # Extract protobuf bytes
        protobuf_bytes = parse_curl_data(curl_text)

        # Write snapshot
        snapshot_path = write_snapshot(snapshot_name, protobuf_bytes)

        print(f"\n📸 Snapshot captured!", file=sys.stderr)
        print(f"Hex: {protobuf_bytes.hex()[:60]}...", file=sys.stderr)

    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
