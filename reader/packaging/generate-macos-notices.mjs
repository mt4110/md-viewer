import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const output = join(root, "reader/resources/notices");
const targets = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
const registry = "registry+https://github.com/rust-lang/crates.io-index";
const objcSource = "https://raw.githubusercontent.com/madsmtm/objc2/";
const maxBytes = 2 * 1024 * 1024;
const packageId = ({ name, version }) => `${name}@${version}`;
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const normalize = (bytes) => new TextDecoder("utf-8", { fatal: true })
  .decode(bytes).replaceAll("\r\n", "\n");
const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));

function licenseBytes(path) {
  const stat = lstatSync(path);
  if (!stat.isFile() || stat.size > maxBytes) throw new Error(`Invalid license file: ${path}`);
  return readFileSync(path);
}

function bundledFiles(base, directory = base) {
  if (!lstatSync(directory).isDirectory()) throw new Error(`Invalid license directory: ${directory}`);
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...bundledFiles(base, path));
    } else if (/^(LICENSE|LICENCE|COPYING|NOTICE|COPYRIGHT|AUTHORS|UNLICENSE)/i.test(entry.name)
      && !/\.(rs|c|h|py|toml|json)$/i.test(entry.name) && !/\.(png|jpg|jpeg|webp|gif|svg|ico)$/i.test(entry.name)) {
      files.push([relative(base, path).split(sep).join("/"), licenseBytes(path)]);
    }
  }
  return files;
}

function lockChecksums() {
  const text = readFileSync(join(root, "Cargo.lock"), "utf8");
  const checksums = new Map();
  // Cargo writes these fields as quoted scalars in each [[package]] section.
  for (const section of text.split(/^\[\[package\]\][\t ]*\r?$/m).slice(1)) {
    const fields = Object.fromEntries([...section.matchAll(/^(name|version|source|checksum) = "([^"\r\n]+)"\r?$/gm)]
      .map(([, key, value]) => [key, value]));
    if (fields.source === registry && /^[a-f0-9]{64}$/.test(fields.checksum)) {
      checksums.set(packageId(fields), fields.checksum);
    }
  }
  return checksums;
}

function upstreamFiles(pkg, base, overrides, objc) {
  const key = packageId(pkg);
  const objcRule = objc.packages[key];
  const rule = objcRule ?? overrides[key];
  if (!rule) return [];
  const { git } = readJson(join(base, ".cargo_vcs_info.json"));
  if (git?.sha1 !== rule.revision || pkg.license !== rule.license) {
    throw new Error(`License override no longer matches ${key}`);
  }
  const originals = objcRule
    ? [
        { ...objc.declaration, source_url: objcSource + rule.revision + "/LICENSE.md" },
        ...rule.license.split(" OR ").map((license) => objc.texts[license]),
      ]
    : rule.files;
  const directory = join(root, "tools/licenses");
  if (!lstatSync(directory).isDirectory()) throw new Error("Invalid pinned license directory");
  return originals.map(({ file, sha256: expected, source_url: origin }) => {
    if (file !== basename(file) || !/^[\w.-]+$/.test(file)) {
      throw new Error(`Invalid pinned license path: ${file}`);
    }
    const bytes = licenseBytes(join(directory, file));
    if (sha256(bytes) !== expected) throw new Error(`Pinned license hash mismatch: ${file}`);
    return [file, bytes, origin];
  });
}

function collect(metadata) {
  const checksums = lockChecksums();
  const overrides = readJson(join(root, "tools/license_overrides.json"));
  const objc = readJson(fileURLToPath(new URL("macos-objc2-licenses.json", import.meta.url)));
  const texts = new Map();
  const packages = [];
  for (const pkg of metadata.packages) {
    if (metadata.workspace_members.includes(pkg.id)) continue;
    const checksum = checksums.get(packageId(pkg));
    if (pkg.source !== registry || !checksum || !pkg.license) {
      throw new Error(`Review source, checksum and license for ${packageId(pkg)}`);
    }
    const base = dirname(pkg.manifest_path);
    const files = [...bundledFiles(base), ...upstreamFiles(pkg, base, overrides, objc)];
    if (!files.length) throw new Error(`Missing license files: ${packageId(pkg)}`);
    const notices = files.sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).map(([file, bytes, origin]) => {
      const text = normalize(bytes);
      if (!text.trim()) throw new Error(`Empty license in ${packageId(pkg)}: ${file}`);
      const textHash = sha256(text);
      texts.set(textHash, text);
      return { file, text_sha256: textHash, ...(origin ? { source_url: origin, source_sha256: sha256(bytes) } : {}) };
    });
    packages.push({
      name: pkg.name,
      version: pkg.version,
      license: pkg.license,
      repository: pkg.repository,
      source_url: `https://static.crates.io/crates/${pkg.name}/${pkg.name}-${pkg.version}.crate`,
      source_sha256: checksum,
      notices,
    });
  }
  packages.sort((a, b) => packageId(a) < packageId(b) ? -1 : packageId(a) > packageId(b) ? 1 : 0);
  if (!packages.length) throw new Error("Cargo metadata returned no third-party packages");
  return { packages, texts };
}

function render({ packages, texts }) {
  const header = [
    "md-viewer — macOS third-party notices",
    `Cargo targets: ${targets.join(", ")}. Includes build-time dependencies from the locked target graphs.`,
    "The md-viewer license is in LICENSE.txt; the bundled formatter notices are in MARKDOWNLINT-NOTICES.txt.",
    "The exact source archive and its SHA-256 are listed for each crate, including MPL-covered source.",
    "License text identifiers are SHA-256 hashes of the included UTF-8 texts with LF line endings.",
  ].join("\n");
  const index = packages.map((pkg) => [
    `${pkg.name} ${pkg.version} — ${pkg.license}`,
    `Source: ${pkg.source_url}`,
    `Source SHA-256: ${pkg.source_sha256}`,
    ...pkg.notices.map((file) => `  ${file.file}: ${file.text_sha256}${file.source_url ? ` (${file.source_url})` : ""}`),
  ].join("\n")).join("\n\n");
  const body = [...texts.keys()].sort().map((hash) => [
    `----- License text SHA-256: ${hash} -----\n`, texts.get(hash),
  ].join("\n")).join("\n\n");
  return {
    "MACOS_THIRD_PARTY_NOTICES.txt": `${header}\n\n=== Package index ===\n\n${index}\n\n=== License texts ===\n\n${body}\n`,
    "MACOS_THIRD_PARTY_NOTICES.json": JSON.stringify({ targets, packages }, null, 2) + "\n",
  };
}

function main() {
  const flags = process.argv.slice(2);
  if (flags.some((flag) => flag !== "--check" && flag !== "--offline")) {
    throw new Error("Usage: node reader/packaging/generate-macos-notices.mjs [--check] [--offline]");
  }
  const workspaceIds = new Set();
  const packages = new Map();
  for (const target of targets) {
    const args = ["run", "1.95.0", "cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", target];
    if (flags.includes("--offline")) args.push("--offline");
    const metadata = JSON.parse(execFileSync("rustup", args, { cwd: root, maxBuffer: 16 * 1024 * 1024 }));
    for (const id of metadata.workspace_members) workspaceIds.add(id);
    for (const pkg of metadata.packages) packages.set(pkg.id, pkg);
  }
  const inventory = collect({ packages: packages.values(), workspace_members: [...workspaceIds] });
  for (const [name, text] of Object.entries(render(inventory))) {
    const path = join(output, name);
    if (flags.includes("--check")) {
      if (readFileSync(path, "utf8") !== text) throw new Error(`Stale notices: ${name}; regenerate and review`);
    } else {
      writeFileSync(path, text, "utf8");
    }
  }
  console.log(`macOS notices: ${inventory.packages.length} packages, ${inventory.texts.size} distinct license texts`);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
