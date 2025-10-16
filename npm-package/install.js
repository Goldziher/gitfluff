"use strict";

const path = require("node:path");
const fs = require("node:fs");

const distDir = path.resolve(__dirname, "dist");

if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

process.stdout.write(
  [
    "gitfluff npm package installed.",
    "No prebuilt binaries were downloaded automatically.",
    "Set GITFLUFF_BINARY or copy a compiled gitfluff binary into npm-package/dist before invoking the CLI.",
  ].join("\n") + "\n",
);
