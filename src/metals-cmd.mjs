// Helper installed by the Metals proxy at `~/.metals-zed/cmd.mjs`.
// Tasks invoke it as `node $HOME/.metals-zed/cmd.mjs <metals-command>`.
// It looks up the proxy's HTTP port for the current workspace and POSTs
// `workspace/executeCommand` to dispatch the command.
//
// The request is fire-and-forget: the proxy returns 202 as soon as it hands
// the command off to Metals. Progress and notifications surface in Zed's UI
// the same way they do when Metals triggers a command on its own.

import { Buffer } from "node:buffer";
import { readFileSync, realpathSync } from "node:fs";
import { request } from "node:http";
import { homedir } from "node:os";
import { join } from "node:path";

const cmd = process.argv[2];
if (!cmd) {
  console.error("Usage: metals-cmd <command>");
  process.exit(1);
}

// Use ZED_WORKTREE_ROOT (set by Zed for tasks) rather than process.cwd():
// interactive shells in user .zshrc/.bashrc may `cd` before node runs.
// `realpathSync` canonicalizes - resolves symlinks and trailing slashes - so
// the hex matches whatever the proxy wrote, regardless of how the user opened
// the workspace.
const workspace = realpathSync(process.env.ZED_WORKTREE_ROOT ?? process.cwd());
const portFile = join(
  homedir(),
  ".metals-zed",
  `${Buffer.from(workspace).toString("hex")}.port`,
);

let port;
try {
  port = Number(readFileSync(portFile, "utf8").trim());
} catch {
  console.error(
    `Could not find the Metals proxy port file at ${portFile}.\n` +
      `Make sure Metals is running for this workspace (open a Scala file first), and\n` +
      `that 'lsp.metals.binary.arguments' is not set in your Zed settings - it disables\n` +
      `the proxy that these tasks rely on.`,
  );
  process.exit(1);
}

const body = JSON.stringify({
  method: "workspace/executeCommand",
  params: { command: cmd },
  fireAndForget: true,
});

// node:http rather than fetch() so HTTP_PROXY env vars don't intercept us.
const req = request(
  {
    host: "127.0.0.1",
    port,
    path: "/",
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": Buffer.byteLength(body),
    },
  },
  (res) => {
    res.on("data", () => {}); // drain so the socket can close
    res.on("end", () => {
      if (res.statusCode === 202) {
        console.log(`Metals: ${cmd} dispatched. Watch Zed for progress.`);
      } else {
        console.error(`Metals: ${cmd} - unexpected response ${res.statusCode}`);
        process.exit(1);
      }
    });
  },
);
req.on("error", (e) => {
  if (e.code === "ECONNREFUSED") {
    console.error(
      `The Metals proxy isn't running on port ${port}.\n` +
        `It may have stopped after Zed last restarted Metals. To recover:\n` +
        `  - cmd-shift-p -> "zed: restart language server"\n` +
        `  - or close and reopen a Scala file in this workspace`,
    );
  } else {
    console.error(
      `Failed to reach the Metals proxy on port ${port}: ${e.code ?? ""} ${e.message}`,
    );
  }
  process.exit(1);
});
req.end(body);
