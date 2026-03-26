# Maintenance of FtaQl

This project currently consists of 4 components:

- The Rust `ftaql` crate, in `crates/ftaql`
- The Rust `ftaql-wasm` crate, in `crates/ftaql-wasm`
- The NPM `@piklv/ftaql-cli` package, in `packages/ftaql`
- The NPM `@piklv/ftaql-wasm` package, an artefact of the `ftaql-wasm` Rust crate

The NPM `@piklv/ftaql-cli` package is a super thin layer that simply calls the relevant `ftaql` binary. For this to work, the NPM package is designed to contain pre-built binaries.

## Development

Use PRs into `main`. GitHub Actions are set up to:

- Compile the Rust crate & run Rust tests, output test coverage (Ubuntu)
- Build binaries for all targets on windows/macos/linux
- Smoke test all built binaries against a sample file
- Construct the NPM package, i.e. install the compiled binaries into `packages/ftaql/binaries`
- Publish the NPM package locally using Verdaccio
- Smoke test the verdaccio-published NPM package via a sample file
- Build the scoped WASM package in dry-run mode and verify the generated `pkg/package.json`

The NPM CLI package itself is plain JavaScript without any Node.js tests or build step, since those things aren't really warranted.

## Publishing and releasing (`ftaql` crate, `@piklv/ftaql-cli` npm package)

1. Merge changes over time to `main`, with green builds to verify everything is healthy
2. Bump versions and update `CHANGELOG.md`
   1. Set the version in the root `package.json` for workspace metadata consistency
   1. Set the version in `packages/ftaql/package.json`
   2. Set the version in `crates/ftaql/Cargo.toml`
   3. Set the version in `crates/ftaql-wasm/Cargo.toml`
   4. Update any release checks with hard-coded versions, especially `.github/workflows/wasm-dry-run.yml`
   5. Run `cargo update` so that the lockfile stays in sync. Do this in a PR and merge it to `main`.
3. When you're satisfied everything is ready on `main` (and the build is green), locally tag the repo with a new version e.g. `v1.0.0`. Push this tag to trigger the release.
4. The release workflow publishes:
   1. GitHub Release assets with all native binaries
   2. The Rust crate `ftaql` to crates.io
   3. The scoped npm package `@piklv/ftaql-cli` to npm
5. Ensure these GitHub Actions secrets exist before tagging:
   1. `CARGO_REGISTRY_TOKEN`
   2. `NPM_TOKEN`

## Pre-release checklist

- Confirm GitHub Actions secrets are configured: `CARGO_REGISTRY_TOKEN` and `NPM_TOKEN`
- Confirm the repository allows `GITHUB_TOKEN` to create draft releases and upload release assets
- Confirm `CHANGELOG.md`, `packages/ftaql/package.json`, `crates/ftaql/Cargo.toml`, `crates/ftaql-wasm/Cargo.toml`, and `.github/workflows/wasm-dry-run.yml` all point at the same release version
- Merge the release-prep PR to `main` and wait for the `test` workflow to pass on `main`
- Create and push the release tag locally:

```sh
git tag v1.0.0
git push origin v1.0.0
```

- Verify the triggered release workflow produces:
  - a draft GitHub Release with all platform assets attached
  - a published `ftaql` crate on crates.io
  - a published `@piklv/ftaql-cli` package on npm
- Review the draft GitHub Release before publishing it from draft state

## First real publish commands

For the automated CLI/crate release:

```sh
git tag v1.0.0
git push origin v1.0.0
```

## WASM npm package

This should be published manually. From the `crates/ftaql-wasm` directory:

1. Ensure the crate version is in sync. Similar to the `@piklv/ftaql-cli` package, it usually makes sense for the core `ftaql` crate to be published first.
2. If you already have the `crates/ftaql-wasm/pkg` dir, delete it / clear it out.
3. Run `wasm-pack build --target web --scope piklv`. This generates a scoped npm package in `pkg` as `@piklv/ftaql-wasm`.
4. If you want to locally debug before publish, you can paste the contents of `pkg` to override an existing version in `node_modules.`
5. Run `cd pkg && npm publish --access public`. Scoped npm packages must be published with public access.

First real manual WASM publish:

```sh
cd crates/ftaql-wasm
rm -rf pkg
wasm-pack build --target web --scope piklv
cd pkg
npm publish --access public
```

## Code Coverage

Code coverage is reported during the `test` workflow.

To check the coverage locally, install and run `tarpaulin`:

```
cargo install cargo-tarpaulin
cargo tarpaulin
```

Note that `tarpaulin` is not installed as a build dependency, hence should be installed manually to generate coverage.
