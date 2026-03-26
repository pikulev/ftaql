#!/usr/bin/env node

const { execFileSync } = require("node:child_process");
const path = require("node:path");
const fs = require("node:fs");

const platform = process.platform;
const architecture = process.arch;

function getBinaryPath() {
  const targetDirectory = path.join(__dirname, "binaries");

  switch (platform) {
    case "win32":
      if (architecture === "x64") {
        return path.join(
          targetDirectory,
          "ftaql-x86_64-pc-windows-msvc",
          "ftaql.exe"
        );
      } else if (architecture === "arm64") {
        return path.join(
          targetDirectory,
          "ftaql-aarch64-pc-windows-msvc",
          "ftaql.exe"
        );
      }
    case "darwin":
      if (architecture === "x64") {
        return path.join(targetDirectory, "ftaql-x86_64-apple-darwin", "ftaql");
      } else if (architecture === "arm64") {
        return path.join(targetDirectory, "ftaql-aarch64-apple-darwin", "ftaql");
      }
    case "linux":
      if (architecture === "x64") {
        return path.join(
          targetDirectory,
          "ftaql-x86_64-unknown-linux-musl",
          "ftaql"
        );
      } else if (architecture === "arm64") {
        return path.join(
          targetDirectory,
          "ftaql-aarch64-unknown-linux-musl",
          "ftaql"
        );
      } else if (architecture === "arm") {
        return path.join(
          targetDirectory,
          "ftaql-arm-unknown-linux-musleabi",
          "ftaql"
        );
      }
      break;
    default:
      throw new Error("Unsupported platform: " + platform);
  }

  throw new Error("Binary not found for the current platform");
}

function setUnixPerms(binaryPath) {
  if (platform === "darwin" || platform === "linux") {
    try {
      fs.chmodSync(binaryPath, "755");
    } catch (e) {
      console.warn("Could not chmod ftaql binary: ", e);
    }
  }
}

// Run the binary from code
// We build arguments that get sent to the binary
function runFtaQl(project, options) {
  if (!options || !options.dbPath) {
    throw new Error("runFtaQl(project, options) requires options.dbPath");
  }

  const binaryPath = getBinaryPath();
  setUnixPerms(binaryPath);
  const binaryArgs = [project, "--db", options.dbPath];

  if (options.configPath) {
    binaryArgs.push("--config-path", options.configPath);
  }
  if (options.revision) {
    binaryArgs.push("--revision", options.revision);
  }
  if (options.ref) {
    binaryArgs.push("--ref", options.ref);
  }

  const result = execFileSync(binaryPath, binaryArgs);
  return result.toString();
}

// Run the binary directly if executed as a standalone script
// Arguments are directly forwarded to the binary
if (require.main === module) {
  const args = process.argv.slice(2); // Exclude the first two arguments (node binary and project path)
  const binaryPath = getBinaryPath();
  setUnixPerms(binaryPath);

  execFileSync(binaryPath, args, { stdio: "inherit" });
}

module.exports.runFtaQl = runFtaQl;
