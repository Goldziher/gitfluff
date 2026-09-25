const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const https = require("node:https");
const http = require("node:http");
const crypto = require("node:crypto");
const tar = require("tar");
const AdmZip = require("adm-zip");

const { version } = require("./package.json");

function platformTriple() {
  const type = os.type();
  const arch = os.arch();

  if (type === "Windows_NT") {
    if (arch === "x64") return "x86_64-pc-windows-gnu";
    if (arch === "ia32") throw new Error("32-bit Windows is not supported");
  } else if (type === "Linux") {
    if (arch === "x64") return "x86_64-unknown-linux-gnu";
    if (arch === "arm64") return "aarch64-unknown-linux-gnu";
  } else if (type === "Darwin") {
    if (arch === "x64") return "x86_64-apple-darwin";
    if (arch === "arm64") return "aarch64-apple-darwin";
  }

  throw new Error(`Unsupported platform: ${type} ${arch}`);
}

function releaseAsset() {
  const triple = platformTriple();
  const isWindows = triple.includes("windows");
  const ext = isWindows ? "zip" : "tar.gz";
  // `name` is returned rather than recomputed by the caller: it is the key the checksum manifest
  // is looked up by, and a name derived separately from the URL could drift out of step with it.
  const name = `gitfluff-${triple}.${ext}`;
  return {
    url: `https://github.com/Goldziher/gitfluff/releases/download/v${version}/${name}`,
    ext,
    name,
  };
}

function ensureBinDir() {
  const binDir = path.join(__dirname, "bin");
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }
  return binDir;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const urlObj = new URL(url);
    const client = urlObj.protocol === "https:" ? https : http;

    const req = client.get(
      url,
      {
        headers: {
          "User-Agent": "gitfluff-npm-wrapper",
        },
      },
      (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          req.destroy();
          return download(res.headers.location, dest).then(resolve, reject);
        }

        if (res.statusCode !== 200) {
          return reject(new Error(`Download failed with status ${res.statusCode}`));
        }

        const file = fs.createWriteStream(dest);
        res.pipe(file);

        file.on("finish", () => {
          file.close(resolve);
        });

        file.on("error", (err) => {
          fs.unlink(dest, () => reject(err));
        });

        // pipe() does not forward the source's errors, so a connection drop after the headers
        // would otherwise be an unhandled 'error' event that crashes postinstall outright.
        res.on("error", (err) => {
          fs.unlink(dest, () => reject(err));
        });
      },
    );

    req.on("error", reject);
    req.setTimeout(45_000, () => {
      req.destroy(new Error("Request timed out"));
    });
  });
}

function downloadText(url) {
  return new Promise((resolve, reject) => {
    const urlObj = new URL(url);
    const client = urlObj.protocol === "https:" ? https : http;

    const req = client.get(
      url,
      {
        headers: {
          "User-Agent": "gitfluff-npm-wrapper",
        },
      },
      (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          req.destroy();
          return downloadText(res.headers.location).then(resolve, reject);
        }

        if (res.statusCode !== 200) {
          return reject(new Error(`Download failed with status ${res.statusCode}`));
        }

        let data = "";
        res.on("data", (chunk) => {
          data += chunk;
        });
        res.on("end", () => resolve(data));
        res.on("error", reject);
      },
    );

    req.on("error", reject);
    req.setTimeout(45_000, () => {
      req.destroy(new Error("Request timed out"));
    });
  });
}

function sha256File(filePath) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash("sha256");
    const stream = fs.createReadStream(filePath);

    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", () => resolve(hash.digest("hex")));
    stream.on("error", reject);
  });
}

async function verifyChecksum(filePath, archiveName) {
  const checksumUrl = `https://github.com/Goldziher/gitfluff/releases/download/v${version}/gitfluff_${version}_checksums.txt`;

  console.log("Downloading checksums manifest...");
  const checksumText = await downloadText(checksumUrl);

  // sha256sum format is "<hash>  <name>". Match the name field exactly rather than with a
  // substring test, so a manifest entry such as "<name>.sig" can never satisfy the lookup.
  const entry = checksumText
    .split("\n")
    .map((line) => line.trim().split(/\s+/u))
    .find((fields) => fields.length >= 2 && path.basename(fields.at(-1)) === archiveName);

  if (!entry) {
    throw new Error(`Checksum not found for ${archiveName} in manifest`);
  }

  const expectedHash = entry[0];

  console.log("Verifying archive checksum...");
  const actualHash = await sha256File(filePath);

  if (actualHash !== expectedHash) {
    throw new Error(`Checksum mismatch for ${archiveName}\nExpected: ${expectedHash}\nActual: ${actualHash}`);
  }

  console.log("Checksum verified successfully.");
}

async function install() {
  try {
    const binDir = ensureBinDir();
    const { url, ext, name: archiveName } = releaseAsset();
    const archivePath = path.join(binDir, `gitfluff.${ext}`);
    const binaryName = os.type() === "Windows_NT" ? "gitfluff.exe" : "gitfluff";
    const binaryPath = path.join(binDir, binaryName);

    if (fs.existsSync(binaryPath)) {
      return;
    }

    console.log(`Downloading gitfluff binary from ${url} ...`);

    // The archive is removed in `finally`: a checksum mismatch must not leave the rejected
    // download sitting in bin/ where a later run could mistake it for a good one.
    try {
      await download(url, archivePath);
      await verifyChecksum(archivePath, archiveName);

      console.log("Extracting binary...");
      if (ext === "zip") {
        const zip = new AdmZip(archivePath);
        const entry = zip.getEntries().find((e) => e.entryName.endsWith(binaryName));
        if (!entry) {
          throw new Error("Binary not found in downloaded archive");
        }
        zip.extractEntryTo(entry, binDir, false, true);
      } else {
        await tar.extract({
          file: archivePath,
          cwd: binDir,
          filter: (entryPath) => entryPath.endsWith(binaryName),
        });
      }
    } finally {
      fs.rmSync(archivePath, { force: true });
    }

    if (os.type() !== "Windows_NT") {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log("gitfluff binary installed successfully.");
  } catch (err) {
    console.error(`Failed to install gitfluff binary: ${err.message}`);
    process.exit(1);
  }
}

install();
