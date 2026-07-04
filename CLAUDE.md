# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FtaQl is a static analysis tool for TypeScript/JavaScript codebases, written in Rust. It computes cyclomatic complexity, Halstead metrics, and coupling/dependency analysis to identify technical debt. Results are persisted to SQLite for historical comparison.

## Build & Test Commands

```bash
# Build
cargo build --release

# Run all tests
cargo test --all

# Run a single test
cargo test <test_name>

# Format check
cargo fmt --check

# Lint
cargo clippy --all --all-targets

# Coverage (requires cargo-tarpaulin installed manually)
cargo tarpaulin --fail-under 75

# Build WASM
wasm-pack build --target web crates/ftaql-wasm

# Run CLI against a project
cargo run -- ./path/to/project --db ./ftaql.sqlite --revision test --ref main
```

## Architecture

**Workspace layout:**
- `crates/ftaql/` — Main Rust library and CLI binary
- `crates/ftaql-wasm/` — WebAssembly bindings for single-file browser analysis (no filesystem access, no coupling)
- `packages/ftaql/` — Thin npm CLI wrapper (`@piklv/ftaql-cli`) that dispatches to platform-specific native binaries

**Two-pass analysis pipeline (in `src/lib.rs`):**
1. **Pass 1:** Parallel file discovery (via `ignore` crate respecting `.gitignore`) → SWC parsing → import/export extraction
2. **Coupling analysis:** Build dependency graph with `petgraph`, detect cycles via Kosaraju algorithm, compute afferent/efferent coupling and instability. Dual-graph approach separates type-only vs runtime dependencies.
3. **Pass 2:** Parallel metric calculation — cyclomatic complexity, Halstead metrics, file scores, coupling scores
4. **Persist** results to SQLite (`src/sqlite.rs`)

**Key modules in `crates/ftaql/src/`:**
- `parse/` — SWC-based JS/TS parsing, import/export extraction
- `coupling/` — Dependency graph construction, Kosaraju SCC detection, coupling metrics
- `cyclo/` — Cyclomatic complexity via AST visitor pattern
- `halstead/` — Halstead metrics (volume, difficulty, effort, bugs estimate)
- `resolve.rs` — Module resolution supporting `tsconfig.json` paths, baseUrl, extends, project references (via `rspack_resolver`)
- `sqlite.rs` — SQLite persistence layer (tables: `analysis_runs`, `files`, `file_dependencies`, `cycles`)
- `walk/` — Filesystem traversal with configurable exclusions
- `structs/` — Core data types (`FileData`, `ProjectAnalysis`, `ComplexityMetrics`, `CouplingMetrics`)
- `config/` — Loads optional `ftaql.json` config, merges with defaults

**Feature flags (Cargo):**
- `cli` — Enables `clap` for CLI binary
- `project-analysis` — Enables filesystem, SQLite, parallelism deps (tokio, rayon, rusqlite, ignore, etc.)
- WASM crate uses `default-features = false` to minimize binary size

## Coding Conventions

- Error handling via `anyhow::Result` with descriptive messages. Avoid `unwrap()` in production code; use `?` operator.
- Analysis structs must derive `Serialize` + `Deserialize`.
- Public API methods take `&str` for paths, not `PathBuf`.
- Logging via `log` crate. Inter-thread communication via `crossbeam` channels.
- Snapshot testing with `insta` crate.
- Prefer immutable data structures and functional style where practical.

## Version Alignment

All four manifests must stay in sync when bumping versions:
- `package.json` (root)
- `packages/ftaql/package.json`
- `crates/ftaql/Cargo.toml`
- `crates/ftaql-wasm/Cargo.toml`

## Semantic Map

`docs/graph.xml` is the project's knowledge graph describing architecture, components, and relationships. Reference it for detailed architectural context.
