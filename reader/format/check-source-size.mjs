import { readdir, readFile } from "node:fs/promises";
import { extname } from "node:path";

const limit = 256;
const sourceExtensions = new Set([".rs", ".js", ".mjs", ".css", ".html"]);
const reader = new URL("../", import.meta.url);
const sources = ["src/", "ui/", "format/"];
const violations = [];

async function checkDirectory(directory) {
  for (const entry of await readdir(new URL(directory, reader), { withFileTypes: true })) {
    const path = directory + entry.name;
    if (entry.isDirectory()) {
      if (path !== "ui/vendor") await checkDirectory(path + "/");
    } else if (entry.isFile() && sourceExtensions.has(extname(entry.name))) {
      const source = await readFile(new URL(path, reader), "utf8");
      const lines = source.replace(/\r?\n$/, "").split("\n").length;
      if (lines > limit) violations.push(`reader/${path} (${lines} lines)`);
    }
  }
}

await Promise.all(sources.map(checkDirectory));
if (violations.length) {
  console.error(`Source files must be at most ${limit} lines:\n${violations.sort().join("\n")}`);
  process.exitCode = 1;
}
