# Public API Overview

FtaQl exposes three public surfaces, and each one has different practical capabilities:

| Artifact | Best for | Input | Output | Limits |
| --- | --- | --- | --- | --- |
| `ftaql` crate + native CLI | Full project analysis, CI, local snapshots | Project path, config, SQLite metadata | `FtaQlJsonOutput` in the library, SQLite snapshot + stdout summary in the CLI | The CLI does not print the full JSON model |
| `@piklv/ftaql-cli` | npm CLI usage and JS/TS integrations | CLI args or `runFtaQl(projectPath, options)` | The native binary stdout as a string | It is a thin wrapper, not a separate engine |
| `@piklv/ftaql-wasm` | Browser playgrounds, sandboxes, editor integrations | Source string, `use_tsx`, `include_comments` | A JSON string for a single `FileData` | No filesystem and no project-level coupling |

## Quick choice

- Use the native CLI or `@piklv/ftaql-cli` when you need project-level analysis, cycles, and SQLite history.
- Use the Rust library API when you want to embed analysis in Rust and work with `FtaQlJsonOutput`.
- Use WASM only for isolated single-file analysis in the browser.

## Cross-cutting caveats

- The library and CLI flows share the same analysis engine from `crates/ftaql`.
- The Node wrapper runs the native binary synchronously via `execFileSync`.
- WASM has no project context, so `coupling_metrics` are absent there and `file_name` is always `source.ts`.
- Analysis configuration lives in `ftaql.json`; CLI flags mostly control config path and snapshot metadata.
