// This file includes code originally from:
//   Project: zed-extensions / java
//   Source: https://github.com/zed-extensions/java/blob/main/src/proxy.mjs
// Original code is licensed under the Apache License, Version 2.0.
// Modifications copyright (c) 2025 Scalameta Maintainers.
// Licensed under the Apache License, Version 2.0.
//
// This is a proxy for communication between Zed and Metals.
// It provides HTTP port to send commands to LSP from within an extension,
// as Zed doesn't support such functionality yet,
// and Scala DAP server has to be initialized by sending `debug-adapter-start` to Metals.
// The proxy should not interfere with the communication betweend the editor and language server.

import { Buffer } from "node:buffer";
import { spawn } from "node:child_process";
import { EventEmitter } from "node:events";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:http";
import { homedir, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { Transform } from "node:stream";
import { text } from "node:stream/consumers";

const HTTP_PORT = 0; // 0 - random free one
const HEADER_SEPARATOR = Buffer.from("\r\n", "ascii");
const CONTENT_SEPARATOR = Buffer.from("\r\n\r\n", "ascii");
const NAME_VALUE_SEPARATOR = Buffer.from(": ", "ascii");
const LENGTH_HEADER = "Content-Length";
const TIMEOUT = 5_000;

const workdir = process.argv[1];
const bin = process.argv[2];
const args = process.argv.slice(3);

const PROXY_ID = Buffer.from(process.cwd().replace(/\/+$/, "")).toString("hex");
const PROXY_HTTP_PORT_FILE = join(workdir, "proxy", PROXY_ID);
// Tasks defined in `languages/scala/tasks.json` invoke a helper from a stable
// path (Zed's task variables can't resolve the extension dir). The helper code
// is passed in via env var by the Rust side.
const HELPER_DIR = join(homedir(), ".metals-zed");
const HELPER_FILE = join(HELPER_DIR, "cmd.mjs");
const HELPER_PORT_FILE = join(HELPER_DIR, `${PROXY_ID}.port`);
const command = process.platform === "win32" ? `"${bin}"` : bin;

const lsp = spawn(command, args, { shell: process.platform === "win32" });
const proxy = createLspProxy({ server: lsp, proxy: process });

proxy.on("client", (data, passthrough) => {
  passthrough();
});
proxy.on("server", (data, passthrough) => {
  passthrough();
});

const server = createServer(async (req, res) => {
  if (req.method !== "POST") {
    res.status = 405;
    res.end("Method not allowed");
    return;
  }

  const data = await text(req)
    .then(safeJsonParse)
    .catch(() => null);

  if (!data) {
    res.status = 400;
    res.end("Bad Request");
    return;
  }

  // Fire-and-forget mode is used by the Metals task helper. It sidesteps
  // cosmetic post-command exceptions Metals emits when refreshing client
  // capabilities Zed doesn't implement. DAP omits this flag and gets Metals'
  // full JSON-RPC response below.
  if (data.fireAndForget === true) {
    proxy.send(data.method, data.params);
    res.statusCode = 202;
    res.end(JSON.stringify({ accepted: true }));
    return;
  }

  const result = await proxy.request(data.method, data.params);
  res.statusCode = 200;
  res.setHeader("Content-Type", "application/json");
  res.write(JSON.stringify(result));
  res.end();
}).listen(HTTP_PORT, "127.0.0.1", () => {
  const portStr = server.address().port.toString();
  mkdirSync(dirname(PROXY_HTTP_PORT_FILE), { recursive: true });
  writeFileSync(PROXY_HTTP_PORT_FILE, portStr);

  // Mirror the port file to a workspace-independent location so tasks can find it,
  // and install the helper script there if Rust passed it in.
  try {
    mkdirSync(HELPER_DIR, { recursive: true });
    writeFileSync(HELPER_PORT_FILE, portStr);
    const helperCode = process.env.METALS_ZED_HELPER_CODE;
    if (helperCode) {
      writeFileSync(HELPER_FILE, helperCode);
    }
  } catch (err) {
    process.stderr.write(`Failed to install Metals task helper: ${err}\n`);
  }
});

// If Metals dies, drop with it so Zed respawns the whole pair cleanly.
lsp.on("exit", () => process.exit(0));

export function createLspProxy({
  server: { stdin: serverStdin, stdout: serverStdout, stderr: serverStderr },
  proxy: { stdin: proxyStdin, stdout: proxyStdout, stderr: proxyStderr },
}) {
  const events = new EventEmitter();
  const queue = new Map();
  const nextid = iterid();

  proxyStdin.pipe(lspMessageSeparator()).on("data", (data) => {
    events.emit("client", parse(data), () => serverStdin.write(data));
  });

  serverStdout.pipe(lspMessageSeparator()).on("data", (data) => {
    const message = parse(data);

    const pending = queue.get(message?.id);
    if (pending) {
      pending(message);
      queue.delete(message.id);
      return;
    }

    events.emit("server", message, () => proxyStdout.write(data));
  });

  serverStderr.pipe(proxyStderr);

  return Object.assign(events, {
    /**
     *
     * @param {string} method
     * @param {any} params
     * @returns void
     */
    notification(method, params) {
      proxyStdout.write(stringify({ jsonrpc: "2.0", method, params }));
    },

    /**
     *
     * @param {string} method
     * @param {any} params
     * @returns Promise<any>
     */
    request(method, params) {
      return new Promise((resolve, reject) => {
        const id = nextid();
        queue.set(id, resolve);

        setTimeout(() => {
          if (queue.has(id)) {
            reject({
              jsonrpc: "2.0",
              id,
              error: {
                code: -32803,
                message: "Request to language server timed out after 5000ms.",
              },
            });
            this.cancel(id);
          }
        }, TIMEOUT);

        serverStdin.write(stringify({ jsonrpc: "2.0", id, method, params }));
      });
    },

    /**
     * Send a request without waiting for the response. The eventual reply
     * still arrives on stdout - register a no-op handler so the queue swallows
     * it instead of forwarding to Zed (which never asked for it).
     *
     * @param {string} method
     * @param {any} params
     */
    send(method, params) {
      const id = nextid();
      queue.set(id, () => {});
      serverStdin.write(stringify({ jsonrpc: "2.0", id, method, params }));
    },

    cancel(id) {
      queue.delete(id);

      serverStdin.write(
        stringify({
          jsonrpc: "2.0",
          method: "$/cancelRequest",
          params: { id },
        }),
      );
    },
  });
}

function iterid() {
  let acc = 1;
  return () => PROXY_ID + "-" + acc++;
}

/**
 * The base protocol consists of a header and a content part (comparable to HTTP).
 * The header and content part are separated by a ‘\r\n’.
 *
 * The header part consists of header fields.
 * Each header field is comprised of a name and a value,
 * separated by ‘: ‘ (a colon and a space).
 * The structure of header fields conforms to the HTTP semantic.
 * Each header field is terminated by ‘\r\n’.
 * Considering the last header field and the overall header
 * itself are each terminated with ‘\r\n’,
 * and that at least one header is mandatory,
 * this means that two ‘\r\n’ sequences always immediately precede
 * the content part of a message.
 *
 * @returns {Transform}
 * @see [language-server-protocol](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#headerPart)
 */
function lspMessageSeparator() {
  let buffer = Buffer.alloc(0);
  let contentLength = null;
  let headersLength = null;

  return new Transform({
    transform(chunk, encoding, callback) {
      buffer = Buffer.concat([buffer, chunk]);

      // A single chunk may contain multiple messages
      while (true) {
        // Wait until we get the whole headers block
        if (buffer.indexOf(CONTENT_SEPARATOR) === -1) {
          break;
        }

        if (!headersLength) {
          const headersEnd = buffer.indexOf(CONTENT_SEPARATOR);
          const headers = Object.fromEntries(
            buffer
              .subarray(0, headersEnd)
              .toString()
              .split(HEADER_SEPARATOR)
              .map((header) => header.split(NAME_VALUE_SEPARATOR))
              .map(([name, value]) => [name.toLowerCase(), value]),
          );

          // A "Content-Length" header must always be present
          contentLength = parseInt(headers[LENGTH_HEADER.toLowerCase()], 10);
          headersLength = headersEnd + CONTENT_SEPARATOR.length;
        }

        const msgLength = headersLength + contentLength;

        // Wait until we get the whole content part
        if (buffer.length < msgLength) {
          break;
        }

        this.push(buffer.subarray(0, msgLength));

        buffer = buffer.subarray(msgLength);
        contentLength = null;
        headersLength = null;
      }

      callback();
    },
  });
}

/**
 *
 * @param {any} content
 * @returns {string}
 */
function stringify(content) {
  const json = JSON.stringify(content);
  return (
    LENGTH_HEADER +
    NAME_VALUE_SEPARATOR +
    json.length +
    CONTENT_SEPARATOR +
    json
  );
}

/**
 *
 * @param {string} message
 * @returns {any | null}
 */
function parse(message) {
  const content = message.slice(message.indexOf(CONTENT_SEPARATOR));
  return safeJsonParse(content);
}

/**
 *
 * @param {string} json
 * @returns {any | null}
 */
function safeJsonParse(json) {
  try {
    return JSON.parse(json);
  } catch (err) {
    return null;
  }
}
