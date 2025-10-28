#!/usr/bin/env python3
"""
Decode a hex snapshot to see the protobuf structure.
Outputs raw bytes that can be used with protoc --decode.

Usage:
    python3 tools/decode_snapshot.py src/snapshots/webapp_captures__add_item_webapp_test.snap

    # Or decode hex directly:
    python3 tools/decode_snapshot.py "0a93030a5c0a20306461..."

    # Output raw bytes for protoc:
    python3 tools/decode_snapshot.py snapshot.snap --raw > message.bin
    protoc --decode=anylist.proto.PBListOperationList src/protobuf/anylist.proto < message.bin
"""

import sys
from pathlib import Path

def read_snapshot(path_or_hex):
    """Read hex from snapshot file or direct hex string."""

    if Path(path_or_hex).exists():
        # Read from file
        content = Path(path_or_hex).read_text()
        # Extract hex (last non-empty line)
        lines = [l.strip() for l in content.split('\n') if l.strip() and not l.startswith('---') and not l.startswith('source:') and not l.startswith('expression:')]
        if not lines:
            raise ValueError("No hex data found in snapshot file")
        hex_data = lines[-1]
    else:
        # Assume it's hex directly
        hex_data = path_or_hex.strip()

    return hex_data


def decode_hex_to_bytes(hex_data):
    """Decode hex string to bytes."""
    return bytes.fromhex(hex_data)


def parse_protobuf_structure(data):
    """
    Basic protobuf parser to show structure.
    This is a simple implementation that shows field numbers and types.
    """

    def read_varint(data, pos):
        """Read a varint from position."""
        value = 0
        shift = 0
        while pos < len(data):
            byte = data[pos]
            pos += 1
            value |= (byte & 0x7F) << shift
            if not (byte & 0x80):
                break
            shift += 7
        return value, pos

    def parse_field(data, pos, indent=0):
        """Parse a single field."""
        if pos >= len(data):
            return pos

        prefix = "  " * indent

        # Read tag
        tag, pos = read_varint(data, pos)
        field_number = tag >> 3
        wire_type = tag & 0x07

        wire_type_names = {
            0: "varint",
            1: "64-bit",
            2: "length-delimited",
            3: "start-group",
            4: "end-group",
            5: "32-bit",
        }

        print(f"{prefix}Field {field_number} ({wire_type_names.get(wire_type, 'unknown')}): ", end="")

        if wire_type == 0:  # Varint
            value, pos = read_varint(data, pos)
            print(f"{value}")
        elif wire_type == 1:  # 64-bit
            if pos + 8 <= len(data):
                value = int.from_bytes(data[pos:pos+8], 'little')
                print(f"{value}")
                pos += 8
        elif wire_type == 2:  # Length-delimited (string/bytes/embedded message)
            length, pos = read_varint(data, pos)
            if pos + length <= len(data):
                field_data = data[pos:pos+length]

                # Try to decode as string
                try:
                    string_val = field_data.decode('utf-8')
                    if string_val.isprintable():
                        print(f'"{string_val}"')
                    else:
                        print(f"<{length} bytes, embedded message>")
                        # Try to parse as embedded message
                        if length > 2:
                            print(f"{prefix}  Embedded message:")
                            inner_pos = 0
                            while inner_pos < len(field_data):
                                try:
                                    inner_pos = parse_field(field_data, inner_pos, indent + 2)
                                except:
                                    break
                except:
                    print(f"<{length} bytes, hex: {field_data.hex()[:40]}{'...' if length > 20 else ''}>")

                pos += length
        elif wire_type == 5:  # 32-bit
            if pos + 4 <= len(data):
                value = int.from_bytes(data[pos:pos+4], 'little')
                print(f"{value}")
                pos += 4
        else:
            print(f"<unknown wire type {wire_type}>")

        return pos

    print("\n=== Protobuf Structure ===\n")
    pos = 0
    field_count = 0
    while pos < len(data):
        try:
            pos = parse_field(data, pos)
            field_count += 1
        except Exception as e:
            print(f"\nError parsing at position {pos}: {e}")
            break

    print(f"\n=== Total fields parsed: {field_count} ===\n")


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 decode_snapshot.py <snapshot_file_or_hex> [--raw]", file=sys.stderr)
        print("", file=sys.stderr)
        print("Examples:", file=sys.stderr)
        print("  python3 decode_snapshot.py src/snapshots/webapp_captures__add_item.snap", file=sys.stderr)
        print("  python3 decode_snapshot.py '0a93030a5c...'", file=sys.stderr)
        print("  python3 decode_snapshot.py snapshot.snap --raw > message.bin", file=sys.stderr)
        sys.exit(1)

    path_or_hex = sys.argv[1]
    output_raw = len(sys.argv) > 2 and sys.argv[2] == '--raw'

    try:
        # Read hex
        hex_data = read_snapshot(path_or_hex)
        print(f"Hex length: {len(hex_data)} chars ({len(hex_data)//2} bytes)", file=sys.stderr)

        # Decode to bytes
        data = decode_hex_to_bytes(hex_data)

        if output_raw:
            # Output raw bytes to stdout
            sys.stdout.buffer.write(data)
        else:
            # Parse and display structure
            parse_protobuf_structure(data)

            print("\n=== Hex Data ===")
            print(hex_data)
            print("\n=== To decode with protoc ===")
            print(f"python3 tools/decode_snapshot.py '{path_or_hex}' --raw | protoc --decode=anylist.proto.PBListOperationList src/protobuf/anylist.proto")

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
