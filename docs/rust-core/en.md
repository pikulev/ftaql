# Rust core and native CLI

## Public entrypoints

The `ftaql` crate exports:

- `analyze_project(path: &String, config: Option<FtaQlConfigResolved>) -> FtaQlJsonOutput`
- `analyze_module(module: &swc_ecma_ast::Module, file_name: &str, line_count: usize) -> FileData`
- `FileData::abs_path(project_root: &str) -> PathBuf`

## What `analyze_project(...)` does

`analyze_project(...)` runs the full project-level pipeline:

1. canonicalize the project path
2. walk files through `ignore::WalkBuilder`
3. apply `.gitignore` rules and config-driven exclude patterns
4. resolve imports using `tsconfig.json` and `jsconfig.json`
5. build full cycles and runtime-only cycles
6. compute file-level metrics and assemble `FtaQlJsonOutput`

## What `analyze_module(...)` does

This function is useful for single-module analysis when the caller already has an AST.

Important details:

- it computes `cyclomatic`, `halstead`, and `file_score`
- `coupling_metrics` are always `None`
- `scores.coupling_score` is always `0.0`

For details on `file_score` and `coupling_score`, see [`../scoring/en.md`](../scoring/en.md).

## Native CLI contract

The CLI accepts:

- required positional `project`
- required `--db`
- optional `--config-path`
- optional `--revision`
- optional `--ref`

When the CLI appends multiple runs into one SQLite database, these parameters help distinguish snapshots:

- `--revision` stores the exact revision identifier, usually a commit SHA
- `--ref` stores a human-readable branch, tag, or channel label

The CLI does not print `FtaQlJsonOutput` to stdout. It reads config, runs `analyze_project(...)`, persists a snapshot via `persist_run(...)`, and prints a short summary.

## Important caveats

- `analyze_project(...)` takes `&String`, not `&str`.
- The pipeline contains `expect()` and `unwrap()`, so some failures surface as panics instead of `Result`.
- If `score_cap` is exceeded, `check_score_cap_breach(...)` terminates the process through `std::process::exit(1)`.
- `CycleInfo.graph` stores indexes into `project_analysis.cycle_members` instead of duplicating path strings inside each cycle.
