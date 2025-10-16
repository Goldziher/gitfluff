#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const path = require("node:path");

const platformBinary = process.platform === "win32" ? "gitfluff.exe" : "gitfluff";
const envBinary = process.env.GITFLUFF_BINARY;
const packageBinary = path.resolve(__dirname, "..", "bin", platformBinary);
const candidate = envBinary && envBinary.trim() ? envBinary : packageBinary;

if (!existsSync(candidate)) {
  console.error(
    [
      "gitfluff: unable to locate binary.",
      "The npm package expects to download release binaries during install.",
      "If you are running from source, set GITFLUFF_BINARY to a compiled binary path.",
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
