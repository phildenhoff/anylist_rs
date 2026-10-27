#!/usr/bin/env node
// Diffs the protobuf schema embedded in the AnyList web bundle against
// src/protobuf/anylist.proto.
//
// The web client ships its schema as protobuf.js JSON
// ({name:"PBFoo",fields:[{rule,type,name,id}, ...]}). Fields it knows that our
// proto lacks are dropped by prost when we decode, and so are missing when we
// send a stored message back.
//
// Usage:
//   node bundle_schema.mjs                 # fetches the live bundle
//   node bundle_schema.mjs app.min.js      # uses a saved copy

import protobuf from "protobufjs";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const BUNDLE_URL = "https://www.anylist.com/static/webapp/js/app.min.js";
const here = dirname(fileURLToPath(import.meta.url));
const root = protobuf.loadSync(join(resolve(here, "../.."), "src/protobuf/anylist.proto"));

// Finds every `{name:"...",fields:[...]}` and returns { name: [field] }.
export function extractMessages(js) {
  const messages = {};
  for (const m of js.matchAll(/\{name:"(\w+)",fields:\[/g)) {
    let depth = 1;
    let i = m.index + m[0].length;
    for (; i < js.length && depth; i++) {
      if (js[i] === "[") depth++;
      else if (js[i] === "]") depth--;
    }
    const body = js.slice(m.index + m[0].length, i - 1);
    messages[m[1]] = [...body.matchAll(/\{rule:"(\w+)",type:"([\w.]+)",name:"(\w+)",id:(\d+)/g)].map((f) => ({
      rule: f[1],
      type: f[2],
      name: f[3],
      id: Number(f[4]),
    }));
  }
  return messages;
}

const ours = (name) => {
  for (const candidate of [name, `PB${name}`]) {
    try {
      return root.lookupType(`anylist.proto.${candidate}`);
    } catch {}
  }
  return null;
};

// The bundle registers some messages under app class names ("ListItem" for
// PBListItem), so compare type names without a package or "PB" prefix.
const bare = (t) => t.replace(/^.*\./, "").replace(/^PB/, "");

const snake = (s) => s.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);

async function main() {
  const js = process.argv[2] ? readFileSync(process.argv[2], "utf8") : await (await fetch(BUNDLE_URL)).text();
  const theirs = extractMessages(js);

  const missingMessages = [];
  const missingFields = [];
  const mismatched = [];
  for (const [name, fields] of Object.entries(theirs).sort(([a], [b]) => a.localeCompare(b))) {
    const type = ours(name);
    if (!type) {
      missingMessages.push(name);
      continue;
    }
    for (const f of fields) {
      const mine = type.fieldsById[f.id];
      if (!mine) missingFields.push({ message: name, ...f });
      else if (bare(mine.type) !== bare(f.type)) {
        mismatched.push({ message: name, ...f, ours: `${mine.type} ${mine.name}` });
      }
    }
  }

  const out = [];
  out.push(`Bundle has ${Object.keys(theirs).length} messages; ${missingMessages.length} are not in anylist.proto, `);
  out.push(`${missingFields.length} fields of shared messages are missing, ${mismatched.length} have a different type.`, "");
  out.push("### Fields missing from anylist.proto", "", "| Message | Id | Field | Type |", "|---|---|---|---|");
  for (const f of missingFields) out.push(`| ${f.message} | ${f.id} | \`${snake(f.name)}\` | ${f.rule} ${f.type} |`);
  out.push("", "### Fields with a different type", "", "| Message | Id | Web client | anylist.proto |", "|---|---|---|---|");
  for (const f of mismatched) out.push(`| ${f.message} | ${f.id} | ${f.rule} ${f.type} ${f.name} | ${f.ours} |`);
  out.push("", "### Messages missing from anylist.proto", "", missingMessages.map((m) => `\`${m}\``).join(", "));
  console.log(out.join("\n"));
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
