// Windows
const exeTargets = [
  "ftaql-aarch64-pc-windows-msvc",
  "ftaql-x86_64-pc-windows-msvc",
];

const plainTargets = [
  // macOS
  "ftaql-x86_64-apple-darwin",
  "ftaql-aarch64-apple-darwin",
  // Linux
  "ftaql-x86_64-unknown-linux-musl",
  "ftaql-aarch64-unknown-linux-musl",
  "ftaql-arm-unknown-linux-musleabi",
];

module.exports.exeTargets = exeTargets;
module.exports.plainTargets = plainTargets;
