#!/usr/bin/env node
// Decodes a capture directory written by capture.mjs.
//
// For every recorded request it writes decoded/<seq>.json holding the typed
// decode (via src/protobuf/anylist.proto) where the endpoint's message type is
// known, a schemaless decode otherwise, and the hex of each protobuf part so it
// can be pasted into an insta snapshot. It also writes summary.md: one row per
// request with its label, endpoint and handler IDs.
//
// Usage:
//   node decode.mjs captures/2026_10_27
//   node decode.mjs captures/2026_10_27 --snapshot 0012 webapp_new_recipe_2026_10_27

import protobuf from "protobufjs";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, "../..");
const root = protobuf.loadSync(join(repo, "src/protobuf/anylist.proto"));
const T = (name) => root.lookupType(`anylist.proto.${name}`);

// Request part types by endpoint. Unlisted endpoints fall back to a
// schemaless decode, which still shows field numbers and strings.
const REQUEST_TYPES = {
  "/data/shopping-lists/update": "PBListOperationList",
  "/data/shopping-lists/update-v2": "PBListOperationList",
  "/data/user-recipe-data/update": "PBRecipeOperationList",
  "/data/meal-planning-calendar/update": "PBCalendarOperationList",
  "/data/list-folders/update": "PBListFolderOperationList",
  "/data/list-settings/update": "PBListSettingsOperationList",
  "/data/starter-lists/update": "PBStarterListOperationList",
  "/data/favorite-products/update": "PBFavoriteProductOperationList",
  "/data/saved-recipes/update": "PBSavedRecipeOperationList",
  "/data/ordered-shopping-list-ids/update": "PBOrderedShoppingListIDsOperationList",
  "/data/ordered-starter-list-ids/update": "PBOrderedStarterListIDsOperationList",
  "/data/categorize-items/update": "PBCategorizeItemOperationList",
  "/data/mobile-app-settings/update": "PBMobileAppSettingsOperationList",
  "/data/user-categories/update": "PBUserCategoryOperationList",
};

const RESPONSE_TYPES = {
  "/data/user-data/get": "PBUserDataResponse",
};

// ── Schemaless decoding ─────────────────────────────────────────────────────

function readVarint(buf, pos) {
  let value = 0n;
  let shift = 0n;
  for (;;) {
    if (pos >= buf.length) throw new Error("truncated varint");
    const b = buf[pos++];
    value |= BigInt(b & 0x7f) << shift;
    if (!(b & 0x80)) return [value, pos];
    shift += 7n;
    if (shift > 70n) throw new Error("varint too long");
  }
}

// Returns [{ field, wire, value }] or throws if buf is not a valid message.
function decodeRaw(buf, depth = 0) {
  const fields = [];
  let pos = 0;
  while (pos < buf.length) {
    let key;
    [key, pos] = readVarint(buf, pos);
    const field = Number(key >> 3n);
    const wire = Number(key & 7n);
    if (field === 0) throw new Error("field 0");
    if (wire === 0) {
      let v;
      [v, pos] = readVarint(buf, pos);
      fields.push({ field, varint: v <= BigInt(Number.MAX_SAFE_INTEGER) ? Number(v) : v.toString() });
    } else if (wire === 1) {
      if (pos + 8 > buf.length) throw new Error("truncated fixed64");
      fields.push({ field, double: buf.readDoubleLE(pos) });
      pos += 8;
    } else if (wire === 5) {
      if (pos + 4 > buf.length) throw new Error("truncated fixed32");
      fields.push({ field, float: buf.readFloatLE(pos) });
      pos += 4;
    } else if (wire === 2) {
      let len;
      [len, pos] = readVarint(buf, pos);
      const end = pos + Number(len);
      if (end > buf.length) throw new Error("truncated bytes");
      fields.push({ field, ...decodeBytes(buf.subarray(pos, end), depth) });
      pos = end;
    } else {
      throw new Error(`wire type ${wire}`);
    }
  }
  return fields;
}

function decodeBytes(b, depth) {
  const text = b.toString("utf8");
  if (b.length === 0) return { string: "" };
  if (/^[\x20-\x7e -￿\t\r\n]*$/.test(text) && !text.includes("�")) return { string: text };
  if (depth < 32) {
    try {
      return { message: decodeRaw(b, depth + 1) };
    } catch {}
  }
  return { bytes: b.toString("hex") };
}

// Every operation list puts operations at field 1 and their metadata at
// field 1, whose field 2 is the handler ID.
function handlerIds(raw) {
  const ids = [];
  for (const op of raw.filter((f) => f.field === 1 && f.message)) {
    for (const meta of op.message.filter((f) => f.field === 1 && f.message)) {
      const h = meta.message.find((f) => f.field === 2 && typeof f.string === "string");
      if (h) ids.push(h.string);
    }
  }
  return ids;
}

function decodeTyped(typeName, buf) {
  if (!typeName) return undefined;
  try {
    const type = T(typeName);
    const msg = type.decode(buf);
    return { type: typeName, value: type.toObject(msg, { longs: String, bytes: String, defaults: false }) };
  } catch (e) {
    return { type: typeName, error: String(e.message ?? e) };
  }
}

function decodePart(endpoint, part) {
  if (!part.base64) return { ...part };
  const buf = Buffer.from(part.base64, "base64");
  let raw;
  try {
    raw = decodeRaw(buf);
  } catch (e) {
    return { name: part.name, hex: buf.toString("hex"), error: `not protobuf: ${e.message}` };
  }
  return {
    name: part.name,
    handlerIds: handlerIds(raw),
    typed: decodeTyped(REQUEST_TYPES[endpoint], buf),
    raw,
    hex: buf.toString("hex"),
  };
}

function decodeResponse(endpoint, response) {
  if (!response?.base64) return response;
  const buf = Buffer.from(response.base64, "base64");
  const text = buf.toString("utf8");
  try {
    return { json: JSON.parse(text) };
  } catch {}
  const typed = decodeTyped(RESPONSE_TYPES[endpoint] ?? (endpoint.endsWith("/update") ? "PBEditOperationResponse" : undefined), buf);
  if (typed && !typed.error) return { typed, length: buf.length };
  try {
    return { raw: decodeRaw(buf), length: buf.length };
  } catch {
    return { text: text.slice(0, 2000), length: buf.length };
  }
}

// ── Main ────────────────────────────────────────────────────────────────────

const [dir, flag, seq, snapName] = process.argv.slice(2);
if (!dir) {
  console.error("usage: node decode.mjs <capture-dir> [--snapshot <seq> <name>]");
  process.exit(2);
}

const files = readdirSync(dir).filter((f) => /^\d{4}-.*\.json$/.test(f)).sort();

if (flag === "--snapshot") {
  const file = files.find((f) => f.startsWith(seq.padStart(4, "0")));
  const rec = JSON.parse(readFileSync(join(dir, file), "utf8"));
  const part = rec.parts.find((p) => p.base64);
  const hex = Buffer.from(part.base64, "base64").toString("hex");
  const out = join(repo, "src/snapshots", `webapp_captures__${snapName}.snap`);
  writeFileSync(out, `---\nsource: tools/webapp_capture/decode.mjs\nexpression: "Captured from webapp"\n---\n${hex}\n`);
  console.error(`wrote ${out}`);
  process.exit(0);
}

mkdirSync(join(dir, "decoded"), { recursive: true });
const rows = [];
for (const file of files) {
  const rec = JSON.parse(readFileSync(join(dir, file), "utf8"));
  const parts = rec.parts?.map((p) => decodePart(rec.endpoint, p));
  const decoded = {
    seq: rec.seq,
    label: rec.label,
    endpoint: rec.endpoint,
    status: rec.status,
    parts,
    response: decodeResponse(rec.endpoint, rec.response),
  };
  writeFileSync(join(dir, "decoded", file), JSON.stringify(decoded, null, 2) + "\n");
  const handlers = parts?.flatMap((p) => p.handlerIds ?? []) ?? [];
  const fields = rec.parts?.map((p) => p.name + (p.base64 ? "" : p.redacted ? "=<redacted>" : `=${p.text}`)).join(", ") ?? "";
  rows.push(`| ${rec.seq} | ${rec.label} | \`${rec.endpoint}\` | ${handlers.map((h) => `\`${h}\``).join(", ")} | ${fields} | ${rec.status ?? ""} |`);
}

const summary = [
  "| # | Action | Endpoint | Handler IDs | Form fields | Status |",
  "|---|---|---|---|---|---|",
  ...rows,
].join("\n");
writeFileSync(join(dir, "summary.md"), summary + "\n");
console.log(summary);
