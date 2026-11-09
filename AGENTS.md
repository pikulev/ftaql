# Repository guidance

FtaQl is a Rust static analysis tool for TypeScript and JavaScript. It computes cyclomatic complexity, Halstead metrics, and dependency metrics. The native CLI stores project results in SQLite so users can compare runs and revisions.

## Tasks and planning

The repository is `pikulev/ftaql`. Work is tracked in [Project 1: ftaql](https://github.com/users/pikulev/projects/1).

When asked to take a task from the project, read its issue description, linked PRs, and current status before editing. The issue contains the plan and acceptance criteria; do not rely on a previous chat or treat an existing green check as proof that the current branch passes.

The dependency adoption plan is recorded in three issues:

- [#25](https://github.com/pikulev/ftaql/issues/25): accept checkout and serde_json after fresh checks.
- [#26](https://github.com/pikulev/ftaql/issues/26): verify the release flow and accept the remaining Actions updates.
- [#27](https://github.com/pikulev/ftaql/issues/27): update the four SWC crates together in a separate branch, adapt string handling, and check the failures reported in #20.

Use the current issues as the source for scope, order, and completion criteria. A task list is not an instruction to start all tasks in every session. Work on the task requested by the user, record the validation results when reporting completion, and update project status when authorized and access allows it.

For local GitHub access, use the connected GitHub tools or authenticated `gh`. Reading Projects requires `read:project`; changing project cards requires `project`. If a write is unavailable, report the pending update accurately.

## Build and validation

Run commands from the repository root unless a command specifies another directory. Choose checks that exercise the change; prose-only edits need documentation checks rather than a full Rust build.

```bash
# Build the native library and CLI
cargo build --release

# Run workspace tests
cargo test --all

# Run one test
cargo test <test_name>

# Check formatting
cargo fmt --check

# Lint all crates and targets
cargo clippy --all --all-targets

# Measure coverage (install cargo-tarpaulin separately)
cargo tarpaulin --fail-under 75

# Build the browser package (requires wasm-pack and the WASM target)
wasm-pack build --target web crates/ftaql-wasm

# Analyze a project and save a run
cargo run -- ./path/to/project --db ./ftaql.sqlite --revision test --ref main
```

For changes to publishing, also read [MAINTENANCE.md](MAINTENANCE.md) and the relevant files in `.github/workflows/`. PR CI does not execute every release-only step; validate the steps that the change affects.

## Architecture

The workspace has three components:

- `crates/ftaql/`: the Rust library, native CLI, and SQLite persistence.
- `packages/ftaql/`: the npm package `@piklv/ftaql-cli`, which runs a platform-specific native binary.
- `crates/ftaql-wasm/`: browser bindings for single-file analysis, without filesystem access or project-level coupling analysis.

Project analysis has two passes in `crates/ftaql/src/lib.rs`. The first discovers files through `ignore`, parses them with SWC, and extracts imports and exports. Coupling analysis then builds dependency graphs with `petgraph`, detects strongly connected components with Kosaraju's algorithm, and computes coupling and instability. Separate graphs distinguish all dependencies from runtime dependencies. The second pass computes file metrics and scores. The CLI writes the resulting analysis to SQLite through `sqlite.rs`.

Key modules in `crates/ftaql/src/`:

- `parse/`: JS/TS parsing and import/export extraction.
- `coupling/`: dependency graphs, cycle detection, and coupling metrics.
- `cyclo/`: cyclomatic complexity through AST visitors.
- `halstead/`: metrics derived from operators and operands.
- `resolve.rs`: module resolution through `rspack_resolver`, including TypeScript configuration.
- `sqlite.rs`: persistence in `analysis_runs`, `files`, `file_dependencies`, and the cycle tables.
- `walk/`: filesystem traversal and file filtering.
- `structs/`: analysis data types, including `FileData`, `ProjectAnalysis`, `ComplexityMetrics`, and `CouplingMetrics`.
- `config/`: loading `ftaql.json` and resolving defaults.

The default Cargo features are `cli` and `project-analysis`. `cli` enables the command-line argument parser; `project-analysis` enables native dependencies such as `ignore`, `tokio`, and `rusqlite`. The WASM crate disables default features on the core crate to use the single-file APIs.

Use [docs/overview/en.md](docs/overview/en.md) for public contracts and [docs/design/en.md](docs/design/en.md) for the analysis flow. Check the implementation when documentation and code disagree.

## Coding conventions

- Use `anyhow::Result` with descriptive errors and `?` where appropriate. Avoid adding production `unwrap()` calls.
- Keep serialization support for public analysis data types and review changes to the JSON and SQLite contracts.
- Preserve existing public signatures unless the task explicitly changes an API. For example, `analyze_project` currently accepts `&String`, while `analyze_module` accepts a file name as `&str`.
- Use the `log` crate for logging and follow the existing `crossbeam` channel patterns for thread communication.
- Use `insta` for snapshot tests; inspect changed snapshots before accepting them.
- Prefer immutable data and a functional style where practical.

## Documentation

Topic documentation lives in `docs/<topic>/ru.md` and `docs/<topic>/en.md`. Keep both versions consistent when changing facts or examples. `README.md` is a symlink to `packages/ftaql/README.md`; edit the target without replacing the symlink.

For Russian prose, apply the `ux-theme-rheme` skill when available. Its essential rules are:

- Start from what the reader already knows, then introduce the new information.
- Connect each sentence and paragraph to the preceding explanation.
- In instructions, state the situation or prerequisite before the action, then explain the result.
- Prefer concrete Russian wording to mixed-language jargon, while preserving API names and established technical terms.
- Preserve facts, limitations, code examples, formulas, and the author's intent. Do not add claims solely to make a transition read smoothly.

Check relative links, headings, and code fences after edits. Use descriptive link labels so readers know which explanation a link opens.

## Release versions

When changing the release version, keep these four manifests aligned:

- `package.json`
- `packages/ftaql/package.json`
- `crates/ftaql/Cargo.toml`
- `crates/ftaql-wasm/Cargo.toml`

Follow the release preparation and verification steps in [MAINTENANCE.md](MAINTENANCE.md).
