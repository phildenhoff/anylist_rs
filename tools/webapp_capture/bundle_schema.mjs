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
//   node bundle_schema.mjs --write-proto [app.min.js]
//       adds the missing fields and messages to anylist.proto

import protobuf from "protobufjs";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const BUNDLE_URL = "https://www.anylist.com/static/webapp/js/app.min.js";
const here = dirname(fileURLToPath(import.meta.url));
const protoPath = join(resolve(here, "../.."), "src/protobuf/anylist.proto");
const root = protobuf.loadSync(protoPath);

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

// Our name for a bundle type: messages the bundle registers under an app
// class name ("ListItem") are "PBListItem" in anylist.proto.
const protoType = (theirs, t) => (theirs[t] && !t.startsWith("PB") && ours(t) ? ours(t).name : t);

// A field whose name anylist.proto already uses for another id (the web
// client moved webSelectedMealPlanTab from 17 to 49) gets a "V<id>" suffix.
const fieldName = (f) => (ours(f.message)?.fields[f.name] ? `${f.name}V${f.id}` : f.name);

const fieldLine = (theirs, f) => `    ${f.rule} ${protoType(theirs, f.type)} ${fieldName(f)} = ${f.id};`;

// Adds each missing field to the end of its message and appends each missing
// message to the end of the file.
function writeProto(theirs, missingFields, missingMessages) {
  const lines = readFileSync(protoPath, "utf8").split("\n");
  const byMessage = Map.groupBy(missingFields, (f) => ours(f.message).name);
  for (const [message, fields] of byMessage) {
    const start = lines.findIndex((l) => l.startsWith(`message ${message} {`));
    const end = lines.findIndex((l, i) => i > start && l === "}");
    lines.splice(end, 0, "", "    // From the web client's schema.", ...fields.sort((a, b) => a.id - b.id).map((f) => fieldLine(theirs, f)));
  }
  while (lines.at(-1) === "") lines.pop();
  for (const name of missingMessages) {
    const fields = [...theirs[name]].sort((a, b) => a.id - b.id);
    lines.push("", `// From the web client's schema.`, `message ${name} {`, ...fields.map((f) => fieldLine(theirs, f)), "}");
  }
  writeFileSync(protoPath, lines.join("\n") + "\n");
}

async function main() {
  const args = process.argv.slice(2);
  const write = args.includes("--write-proto");
  const file = args.find((a) => !a.startsWith("--"));
  const js = file ? readFileSync(file, "utf8") : await (await fetch(BUNDLE_URL)).text();
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

  if (write) {
    writeProto(theirs, missingFields, missingMessages);
    console.log(`Added ${missingFields.length} fields and ${missingMessages.length} messages to anylist.proto`);
    return;
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
