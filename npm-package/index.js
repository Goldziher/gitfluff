"use strict";

const path = require("node:path");

module.exports = {
  binaryPath: process.env.GITFLUFF_BINARY
    || path.resolve(__dirname, "dist", process.platform === "win32" ? "gitfluff.exe" : "gitfluff"),
};
