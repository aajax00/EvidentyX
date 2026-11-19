#!/usr/bin/env node
// Version unique d'EVIDENTYX.
//
// La version est déclarée à deux endroits, qui doivent rester identiques :
//   - Cargo.toml (workspace.package.version) : crates Rust et hôte Tauri ;
//   - apps/desktop/package.json : interface, et paquets (tauri.conf.json y renvoie).
//
// Usage :
//   node scripts/version.mjs                 affiche la version et vérifie la cohérence
//   node scripts/version.mjs check v0.1.0    vérifie aussi la correspondance avec une étiquette
//   node scripts/version.mjs set 0.1.0       fixe la version partout (Cargo.lock, package-lock inclus)

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const cargoPath = join(root, "Cargo.toml");
const desktop = join(root, "apps/desktop");
const packagePath = join(desktop, "package.json");
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;
const CARGO_VERSION = /(\[workspace\.package\][^[]*?\nversion = ")([^"]+)(")/;

function fail(message) {
  console.error(`✗ ${message}`);
  process.exit(1);
}

function readVersions() {
  const cargo = readFileSync(cargoPath, "utf8").match(CARGO_VERSION)?.[2];
  const pkg = JSON.parse(readFileSync(packagePath, "utf8")).version;
  if (!cargo) fail("version introuvable dans Cargo.toml ([workspace.package])");
  return { cargo, pkg };
}

function check(tag) {
  const { cargo, pkg } = readVersions();
  if (cargo !== pkg) fail(`versions différentes : Cargo.toml ${cargo}, package.json ${pkg}`);
  if (tag !== undefined) {
    const expected = `v${cargo}`;
    if (tag !== expected) fail(`l'étiquette ${tag} ne correspond pas à la version ${cargo} (attendu : ${expected})`);
  }
  console.log(`✓ version ${cargo}${tag ? ` (étiquette ${tag})` : ""}`);
}

function set(version) {
  if (!SEMVER.test(version ?? "")) fail(`version invalide : « ${version ?? ""} » (attendu : X.Y.Z)`);
  const cargoText = readFileSync(cargoPath, "utf8");
  writeFileSync(cargoPath, cargoText.replace(CARGO_VERSION, `$1${version}$3`));
  execFileSync("npm", ["version", version, "--no-git-tag-version", "--allow-same-version"], {
    cwd: desktop,
    stdio: "ignore",
  });
  try {
    execFileSync("cargo", ["update", "--workspace", "--offline"], { cwd: root, stdio: "ignore" });
  } catch {
    console.warn("! Cargo.lock non mis à jour (cargo indisponible) : lancez « cargo update --workspace ».");
  }
  check();
}

const [command, argument] = process.argv.slice(2);
if (command === "set") set(argument);
else if (command === "check" || command === undefined) check(argument);
else fail(`commande inconnue : ${command} (attendu : check ou set)`);
