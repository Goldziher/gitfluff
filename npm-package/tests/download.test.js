const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const { once } = require("node:events");
const test = require("node:test");
const { download, downloadText } = require("../install.js");

for (const textDownload of [false, true]) {
  test(`should follow relative redirects when downloading ${textDownload ? "text" : "archives"}`, async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "gitfluff-test-"));
    const server = http.createServer((request, response) => {
      if (request.url === "/redirect") {
        response.writeHead(302, { Location: "/asset" });
        response.end();
      } else {
        response.end("downloaded");
      }
    });
    server.listen(0, "127.0.0.1");
    await once(server, "listening");
    try {
      const url = `http://127.0.0.1:${server.address().port}/redirect`;
      const destination = path.join(directory, "archive");
      if (textDownload) {
        assert.equal(await downloadText(url), "downloaded");
      } else {
        await download(url, destination);
        assert.equal(fs.readFileSync(destination, "utf8"), "downloaded");
      }
    } finally {
      server.close();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}

for (const textDownload of [false, true]) {
  test(`should stop redirect loops when downloading ${textDownload ? "text" : "archives"}`, async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "gitfluff-test-"));
    let requests = 0;
    const server = http.createServer((_request, response) => {
      requests += 1;
      response.writeHead(requests > 10 ? 500 : 302, { Location: `http://127.0.0.1:${server.address().port}/loop` });
      response.end();
    });
    server.listen(0, "127.0.0.1");
    await once(server, "listening");
    try {
      const url = `http://127.0.0.1:${server.address().port}/loop`;
      const request = textDownload ? downloadText(url) : download(url, path.join(directory, "archive"));
      await assert.rejects(request, /Too many redirects/u);
      assert.equal(requests, 6);
    } finally {
      server.close();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
