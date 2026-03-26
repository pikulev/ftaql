# Usage Patterns

## When to use the CLI

Use the native CLI or `@piklv/ftaql-cli` when you need to:

- analyze a whole project
- persist a snapshot into SQLite
- run analysis in CI/CD
- compare revisions through SQL

See also [`../node-cli/en.md`](../node-cli/en.md) and [`../sqlite-output/en.md`](../sqlite-output/en.md).

## When to use the Rust API

Use the `ftaql` crate when you:

- are building a Rust integration
- want `FtaQlJsonOutput` in memory
- already have an AST and want to call `analyze_module(...)`

See [`../rust-core/en.md`](../rust-core/en.md).

## When to use WASM

Use `@piklv/ftaql-wasm` when you:

- analyze a single snippet in the browser
- build a playground or editor integration
- do not have filesystem access

See [`../wasm/en.md`](../wasm/en.md).
