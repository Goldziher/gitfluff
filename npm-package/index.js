const path = require("node:path");

function resolveBinary() {
	if (process.env.GITFLUFF_BINARY) {
		return process.env.GITFLUFF_BINARY;
	}
	const binaryName = process.platform === "win32" ? "gitfluff.exe" : "gitfluff";
	return path.resolve(__dirname, "bin", binaryName);
}

module.exports = {
	binaryPath: resolveBinary(),
};
