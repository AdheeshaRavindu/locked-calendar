const { spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

function loadEnvFile(filePath) {
  if (!fs.existsSync(filePath)) {
    return;
  }

  for (const line of fs.readFileSync(filePath, "utf8").split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) {
      continue;
    }

    const equals = trimmed.indexOf("=");
    if (equals === -1) {
      continue;
    }

    const key = trimmed.slice(0, equals).trim();
    let value = trimmed.slice(equals + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }

    if (key && process.env[key] == null) {
      process.env[key] = value;
    }
  }
}

loadEnvFile(path.resolve(__dirname, "..", ".env"));
loadEnvFile(path.resolve(__dirname, "..", ".env.local"));

const cargoBin =
  process.env.CARGO_HOME != null
    ? path.join(process.env.CARGO_HOME, "bin")
    : path.join(os.homedir(), ".cargo", "bin");

const sep = path.delimiter;
const pathKey = process.platform === "win32" ? "Path" : "PATH";
const currentPath = process.env[pathKey] || "";

if (!currentPath.split(sep).some((p) => p.toLowerCase() === cargoBin.toLowerCase())) {
  process.env[pathKey] = `${cargoBin}${sep}${currentPath}`;
}

const check = spawnSync("cargo", ["--version"], {
  env: process.env,
  shell: true,
  encoding: "utf8",
});

if (check.status !== 0) {
  console.error(`
Rust/Cargo was not found.

Install Rust: https://rustup.rs/
  winget install Rustlang.Rustup

Then close and reopen your terminal, or run:
  $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"
`);
  process.exit(1);
}

const args = process.argv.slice(2);
const result = spawnSync("npx", ["tauri", ...args], {
  env: process.env,
  shell: true,
  stdio: "inherit",
});

process.exit(result.status ?? 1);
