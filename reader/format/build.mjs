import { build } from "esbuild";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const result = await build({
  absWorkingDir: root,
  entryPoints: ["reader/format/worker.mjs"],
  outfile: "reader/ui/vendor/markdownlint-worker.js",
  bundle: true,
  minify: true,
  platform: "browser",
  conditions: ["worker"],
  target: "safari16.4",
  legalComments: "inline",
  metafile: true,
  write: false,
});

const packageRoots = new Set();
for (const file of Object.keys(result.metafile.inputs)) {
  const segments = file.split("/");
  const index = segments.lastIndexOf("node_modules");
  if (index < 0) continue;
  const scoped = segments[index + 1].startsWith("@");
  packageRoots.add(resolve(root, ...segments.slice(0, index + (scoped ? 3 : 2))));
}

const notices = [];
for (const directory of [...packageRoots].sort()) {
  const { name, version, license } = JSON.parse(await readFile(resolve(directory, "package.json"), "utf8"));
  const names = (await readdir(directory)).filter((file) => /^(?:licen[cs]e|copying|notice)(?:[.-]|$)/i.test(file)).sort();
  if (!names.length) throw new Error("Missing license text for " + name);
  const texts = await Promise.all(names.map(async (file) => (await readFile(resolve(directory, file), "utf8")).trim()));
  notices.push([name + " " + version + " (" + license + ")", ...texts].join("\n\n"));
}

const output = result.outputFiles[0];
await writeFile(output.path, output.contents);
await writeFile(dirname(output.path) + sep + "MARKDOWNLINT-NOTICES.txt", notices.join("\n\n---\n\n") + "\n");
