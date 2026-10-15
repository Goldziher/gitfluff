const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const test = require("node:test");

test(
  "should propagate termination when the binary is killed by a signal",
  { skip: process.platform === "win32" },
  () => {
    const result = spawnSync(
      process.execPath,
      [path.join(__dirname, "../bin/gitfluff.js"), "-e", 'process.kill(process.pid, "SIGTERM")'],
      {
        env: { ...process.env, GITFLUFF_BINARY: process.execPath },
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, "SIGTERM");
  },
);

test("should forward binary exit status on every platform", () => {
  const result = spawnSync(process.execPath, [path.join(__dirname, "../bin/gitfluff.js"), "-e", "process.exit(7)"], {
    env: { ...process.env, GITFLUFF_BINARY: process.execPath },
  });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 7);
});
