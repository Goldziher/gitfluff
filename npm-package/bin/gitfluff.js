#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const path = require("node:path");

const platform = process.platform === "win32" ? "gitfluff.exe" : "gitfluff";
const envBinary = process.env.GITFLUFF_BINARY;
const bundledBinary = path.resolve(__dirname, "..", "dist", platform);

const candidate = envBinary && envBinary.trim().length > 0 ? envBinary : bundledBinary;

if (!existsSync(candidate)) {
  console.error(
    [
      "gitfluff: unable to locate packaged binary.",
      "Set GITFLUFF_BINARY to the binary path or run `cargo install gitfluff` and add it to PATH.",
    ].join("\n"),
  );
  process.exit(1);
}

const result = spawnSync(candidate, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`gitfluff: failed to execute binary: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 0);
