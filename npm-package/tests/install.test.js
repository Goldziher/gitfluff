const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");
const tar = require("tar");
const AdmZip = require("adm-zip");
const { extractBinary } = require("../install.js");

for (const ext of ["tar.gz", "zip"]) {
  test(`should install only the exact regular binary from a nested ${ext} archive`, async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "gitfluff-test-"));
    try {
      const source = path.join(directory, "source");
      fs.mkdirSync(path.join(source, "nested"), { recursive: true });
      const binaryName = ext === "zip" ? "gitfluff.exe" : "gitfluff";
      fs.mkdirSync(path.join(source, binaryName));
      fs.writeFileSync(path.join(source, `not${binaryName}`), "wrong");
      fs.writeFileSync(path.join(source, "nested", binaryName), "correct");
      const archive = path.join(directory, `release.${ext}`);
      if (ext === "zip") {
        const zip = new AdmZip();
        zip.addLocalFolder(source);
        zip.writeZip(archive);
      } else {
        await tar.create({ gzip: true, file: archive, cwd: source }, ["."]);
      }
      const destination = path.join(directory, binaryName);
      await extractBinary(archive, ext, destination);
      assert.equal(fs.readFileSync(destination, "utf8"), "correct");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}

test("should leave no binary or staging files when extraction fails", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "gitfluff-test-"));
  try {
    const archive = path.join(directory, "release.zip");
    const zip = new AdmZip();
    zip.addFile("notgitfluff.exe", Buffer.from("wrong"));
    zip.writeZip(archive);
    await assert.rejects(extractBinary(archive, "zip", path.join(directory, "gitfluff.exe")), /Binary not found/u);
    assert.deepEqual(fs.readdirSync(directory), ["release.zip"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("should reject symbolic links in zip archives", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "gitfluff-test-"));
  try {
    const archive = path.join(directory, "release.zip");
    const zip = new AdmZip();
    zip.addFile("gitfluff.exe", Buffer.from("target"));
    zip.getEntry("gitfluff.exe").attr = (0o120777 << 16) >>> 0;
    zip.writeZip(archive);
    await assert.rejects(extractBinary(archive, "zip", path.join(directory, "gitfluff.exe")), /Binary not found/u);
    assert.deepEqual(fs.readdirSync(directory), ["release.zip"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
