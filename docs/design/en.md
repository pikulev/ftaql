# Technical Design

FtaQl consists of a shared Rust core and two thin integration layers:

- `crates/ftaql` - analysis engine, JSON data model, native CLI, SQLite persistence
- `packages/ftaql` - a Node.js wrapper over the native binary
- `crates/ftaql-wasm` - a browser-side single-file API

## Project-level pipeline

1. read `ftaql.json`
2. walk files via `ignore`
3. parse through SWC
4. resolve imports with `tsconfig/jsconfig` support
5. run coupling and cycle analysis
6. compute file-level scoring
7. persist a snapshot into SQLite

## Main architectural boundary

Project-level analysis lives in the Rust core and is available through the library API, the native CLI, and the Node wrapper. WASM is intentionally limited to one file and does not build a cross-file dependency graph.
