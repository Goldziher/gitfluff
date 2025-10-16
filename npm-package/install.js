"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const https = require("node:https");
const http = require("node:http");
const tar = require("tar");
const AdmZip = require("adm-zip");

const { version } = require("./package.json");

function platformTriple() {
  const type = os.type();
  const arch = os.arch();

  if (type === "Windows_NT") {
    if (arch === "x64") return "x86_64-pc-windows-msvc";
    if (arch === "ia32") return "i686-pc-windows-msvc";
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
  return {
    url: `https://github.com/Goldziher/gitfluff/releases/download/v${version}/gitfluff-${triple}.${ext}`,
    ext,
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
      },
    );

    req.on("error", reject);
    req.setTimeout(45_000, () => {
      req.destroy(new Error("Request timed out"));
    });
  });
}

async function install() {
  try {
    const binDir = ensureBinDir();
    const { url, ext } = releaseAsset();
    const archivePath = path.join(binDir, `gitfluff.${ext}`);
    const binaryName = os.type() === "Windows_NT" ? "gitfluff.exe" : "gitfluff";
    const binaryPath = path.join(binDir, binaryName);

    // Skip download if binary already exists (e.g., reinstall)
    if (fs.existsSync(binaryPath)) {
      return;
    }

    console.log(`Downloading gitfluff binary from ${url} ...`);
    await download(url, archivePath);

    console.log("Extracting binary...");
    if (ext === "zip") {
      const zip = new AdmZip(archivePath);
      const entry = zip
        .getEntries()
        .find((e) => e.entryName.endsWith(binaryName));
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

    fs.unlinkSync(archivePath);

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
