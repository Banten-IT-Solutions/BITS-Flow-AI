import { readFileSync, writeFileSync } from "node:fs";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+([-+][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("Usage: node scripts/set-release-version.mjs <MAJOR.MINOR.PATCH>");
  process.exit(1);
}

// Patch only the version fields so the formatter-approved formatting of each
// file survives the release commit.
function patch(path, pattern, replacement) {
  const content = readFileSync(path, "utf8");
  if (!pattern.test(content)) {
    console.error(`Could not find the version field in ${path}`);
    process.exit(1);
  }
  writeFileSync(path, content.replace(pattern, replacement));
}

patch("package.json", /"version":\s*"[^"]*"/, `"version": "${version}"`);
patch("src-tauri/tauri.conf.json", /"version":\s*"[^"]*"/, `"version": "${version}"`);
// The [package] version is the only `version =` at line start in Cargo.toml.
patch("src-tauri/Cargo.toml", /^version\s*=\s*"[^"]*"/m, `version = "${version}"`);

console.log(`Aligned release version to ${version}`);
