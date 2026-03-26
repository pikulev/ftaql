# Changelog

## v1.0.2

Patch release focused on release automation hardening, dependency refreshes, and documentation cleanup.

### Highlights

- Tighten release validation and publishing automation for GitHub Releases, crates.io, npm CLI, and the WASM approval flow
- Align core Rust dependencies, including the SWC visitor stack and `petgraph`, to keep the stable release line current
- Refine README and maintenance documentation for the published CLI and release process

## v1.0.1

Patch release to unblock npm publication after the existing `v1.0.0` versions were already published.

### Highlights

- Fix the WASM package build by separating browser-safe analysis APIs from native-only project analysis code
- Keep CLI and native project analysis enabled by default while allowing `ftaql-wasm` to compile without Tokio networking dependencies
- Prepare a fresh release version across Rust crates and npm packages

## v1.0.0

First stable release of FtaQl.

### Highlights

- Publish the Rust crate `ftaql` together with the scoped npm CLI package `@piklv/ftaql-cli`
- Ship the WebAssembly package flow for `@piklv/ftaql-wasm`
- Add npm type definitions for the CLI package
- Refresh the release automation and dry-run validation for npm and WASM packaging

### Analysis and CLI improvements

- Correct the Halstead calculation, which changes Halstead and file-level scores
- Add the `include_comments` option, defaulting to `false`
- Add the `exclude_under` option, defaulting to `6`
- Fix `output_limit` so it only affects table output and behaves as expected
- Expose `output_limit`, `score_cap`, `include_comments`, and `exclude_under` as CLI options
- Fix an `ENOBUFS` crash that could occur when analyzing very large projects

### Distribution and portability

- Target `musl` Linux binaries on `x86_64`, `arm`, and `aarch64` for broader compatibility
- Improve binary packaging reliability on macOS, Linux, and Ubuntu
- Include the WASM npm module as part of the release surface
- Request explicit GitHub Release write permissions in CI and keep WASM package version checks aligned with `Cargo.toml`
