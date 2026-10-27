// Records every API request the AnyList web client sends from a Playwright page.
//
// Each request is written as one JSON file holding the endpoint, the form
// parts (protobuf parts base64-encoded) and the raw response body. Secrets are
// never written: auth headers, cookies, passwords and tokens are redacted
// before anything touches disk.

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const API_HOSTS = new Set(["www.anylist.com", "anylist.com"]);
const SECRET_FIELDS = new Set(["password", "sign_in_pw", "_xsrf", "refresh_token", "access_token", "token"]);

export function isApiRequest(url, method) {
  const u = new URL(url);
  if (!API_HOSTS.has(u.hostname)) return false;
  return method === "POST" && !u.pathname.startsWith("/static/");
}

// Splits a multipart/form-data body into { name, filename, contentType, data } parts.
export function parseMultipart(body, contentType) {
  const m = /boundary=(?:"([^"]+)"|([^;]+))/i.exec(contentType ?? "");
  if (!m) return null;
  const boundary = Buffer.from(`--${m[1] ?? m[2]}`);
  const parts = [];
  let pos = body.indexOf(boundary);
  while (pos !== -1) {
    const start = pos + boundary.length;
    if (body.subarray(start, start + 2).toString() === "--") break;
    const next = body.indexOf(boundary, start);
    if (next === -1) break;
    // Strip the CRLF after the boundary and the CRLF before the next one.
    const chunk = body.subarray(start + 2, next - 2);
    const headerEnd = chunk.indexOf("\r\n\r\n");
    const headers = chunk.subarray(0, headerEnd).toString("latin1");
    const data = chunk.subarray(headerEnd + 4);
    const disposition = /content-disposition:([^\r\n]*)/i.exec(headers)?.[1] ?? "";
    parts.push({
      name: /name="([^"]*)"/.exec(disposition)?.[1],
      filename: /filename="([^"]*)"/.exec(disposition)?.[1],
      contentType: /content-type:\s*([^\r\n]*)/i.exec(headers)?.[1],
      data,
    });
    pos = next;
  }
  return parts;
}

function parseUrlEncoded(body) {
  return [...new URLSearchParams(body.toString("latin1"))].map(([name, value]) => ({
    name,
    data: Buffer.from(value, "latin1"),
  }));
}

function serialisePart(part) {
  const out = { name: part.name };
  if (part.filename) out.filename = part.filename;
  if (part.contentType) out.contentType = part.contentType;
  if (SECRET_FIELDS.has(part.name)) {
    out.redacted = true;
    return out;
  }
  const text = part.data.toString("utf8");
  const printable = /^[\x20-\x7e\t\r\n]*$/.test(text);
  if (printable && part.data.length < 512) out.text = text;
  else out.base64 = part.data.toString("base64");
  return out;
}

function redactHeaders(headers) {
  const out = {};
  for (const [k, v] of Object.entries(headers)) {
    out[k] = /^(authorization|cookie|set-cookie|x-.*token.*)$/i.test(k) ? "<redacted>" : v;
  }
  return out;
}

function redactResponse(pathname, body) {
  // Auth responses carry tokens; keep only the field names.
  if (pathname.startsWith("/auth/") || pathname.startsWith("/web/sign-in")) {
    try {
      const json = JSON.parse(body.toString("utf8"));
      return { json: Object.fromEntries(Object.keys(json).map((k) => [k, "<redacted>"])) };
    } catch {
      return { redacted: true, length: body.length };
    }
  }
  return { base64: body.toString("base64") };
}

export class Recorder {
  constructor(outDir) {
    this.outDir = outDir;
    this.seq = 0;
    this.label = "unlabelled";
    this.pending = new Set();
    this.websocket = [];
    mkdirSync(outDir, { recursive: true });
  }

  // Tags every request sent from now on until the next mark().
  mark(label) {
    this.label = label;
    console.error(`[capture] ── ${label}`);
  }

  attach(page) {
    page.on("websocket", (ws) => {
      const url = new URL(ws.url());
      if (!API_HOSTS.has(url.hostname)) return;
      const log = (dir) => (frame) => {
        const payload = typeof frame.payload === "string" ? frame.payload : frame.payload.toString("base64");
        this.websocket.push({ time: new Date().toISOString(), label: this.label, dir, payload });
      };
      ws.on("framesent", log("sent"));
      ws.on("framereceived", log("received"));
    });
    page.on("requestfinished", (req) => {
      if (!isApiRequest(req.url(), req.method())) return;
      const p = this.#record(req).catch((e) => console.error(`[capture] failed: ${e}`));
      this.pending.add(p);
      p.finally(() => this.pending.delete(p));
    });
  }

  // Waits for in-flight writes so the caller can safely mark the next step.
  async flush() {
    await Promise.all(this.pending);
    writeFileSync(join(this.outDir, "websocket.json"), JSON.stringify(this.websocket, null, 2) + "\n");
  }

  async #record(req) {
    const url = new URL(req.url());
    const headers = await req.allHeaders();
    const body = req.postDataBuffer() ?? Buffer.alloc(0);
    const ct = headers["content-type"] ?? "";
    let parts = null;
    if (ct.startsWith("multipart/form-data")) parts = parseMultipart(body, ct);
    else if (ct.startsWith("application/x-www-form-urlencoded")) parts = parseUrlEncoded(body);

    const res = await req.response();
    const resBody = res ? await res.body().catch(() => Buffer.alloc(0)) : Buffer.alloc(0);

    const seq = String(++this.seq).padStart(4, "0");
    const slug = url.pathname.replace(/^\//, "").replace(/[^a-z0-9]+/gi, "_");
    const record = {
      seq: Number(seq),
      label: this.label,
      time: new Date().toISOString(),
      method: req.method(),
      endpoint: url.pathname,
      query: url.search || undefined,
      requestHeaders: redactHeaders(headers),
      parts: parts?.map(serialisePart),
      rawBodyBase64: parts || /^\/(auth|web)\//.test(url.pathname) ? undefined : body.toString("base64"),
      status: res?.status(),
      responseHeaders: res ? redactHeaders(await res.allHeaders()) : undefined,
      response: res ? redactResponse(url.pathname, resBody) : undefined,
    };
    writeFileSync(join(this.outDir, `${seq}-${slug}.json`), JSON.stringify(record, null, 2) + "\n");
    console.error(`[capture] ${seq} ${url.pathname} (${this.label})`);
  }
}
