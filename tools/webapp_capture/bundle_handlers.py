#!/usr/bin/env python3
"""
List the operation handler IDs and data endpoints in the AnyList web bundle.

This is a static scan, so it is a checklist of what the web client can send,
not proof of what it sends. Use capture.mjs for the real requests.

Usage:
    curl -sL https://www.anylist.com/static/webapp/js/app.min.js -o /tmp/app.min.js
    python3 tools/webapp_capture/bundle_handlers.py /tmp/app.min.js > handlers.md
"""

import re
import sys

src = open(sys.argv[1], encoding="utf-8").read()

# Handler IDs reach PBOperationMetadata.setHandlerId through a few helpers:
# the per-class `$oj$RT("id")` operation builder, the list `$oj$OA("id")`
# metadata builder, direct `setHandlerId("id")`, and defaults like
# `if(e==null){e="add-shopping-list-item"}`.
HANDLER = re.compile(
    r'(?:\$oj\$(?:RT|OA)\(|setHandlerId\(|==null\)\{\w+=)"([a-z]+(?:-[a-z0-9]+)+)"'
)
OP_BUILDER = re.compile(r"\$oj\$RT=function\(t\)\{(?:var e;const o=|var e=)new ro\.(PB\w+Operation)\b")
OP_TYPE = re.compile(r"new ro\.(PB\w+Operation)\b")
ENDPOINT = re.compile(r'"(/(?:data|auth)/[a-z0-9/_-]+)"')

builders = [(m.start(), m.group(1)) for m in OP_BUILDER.finditer(src)]


def op_type(pos, text):
    if text.startswith("$oj$RT("):
        # Each class has its own $oj$RT builder; take the closest one.
        if builders:
            return min(builders, key=lambda b: abs(b[0] - pos))[1]
    window = src[max(0, pos - 1500) : pos + 300]
    found = OP_TYPE.findall(window)
    return found[-1] if found else "?"


handlers = {}
for m in HANDLER.finditer(src):
    handlers.setdefault(m.group(1), set()).add(op_type(m.start(), m.group(0)))

print("## Handler IDs\n")
print("| Handler ID | Operation type (best guess) |")
print("|---|---|")
for h in sorted(handlers):
    print(f"| `{h}` | {', '.join(sorted(handlers[h]))} |")

print("\n## Endpoints\n")
for e in sorted(set(ENDPOINT.findall(src))):
    print(f"- `{e}`")
