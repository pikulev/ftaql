# Maintenance of FtaQl

FtaQl currently ships four maintainer-facing deliverables:

- the Rust crate `ftaql` in `crates/ftaql`
- the Rust crate `ftaql-wasm` in `crates/ftaql-wasm`
- the npm package `@piklv/ftaql-cli` in `packages/ftaql`
- the npm package `@piklv/ftaql-wasm`, generated from `crates/ftaql-wasm/pkg`

The CLI npm package is intentionally thin. It only dispatches to a platform-specific `ftaql` binary that must be present inside `packages/ftaql/binaries` at publish time.

## What CI verifies

Use PRs into `main`. The `test` workflow is the pre-release gate and is expected to stay green on `main`.

Current CI covers:

- aligned version checks across all publish manifests
- Rust build, formatting, tests, and coverage reporting
- native binary builds for macOS, Windows, and Linux
- smoke tests against built binaries
- npm CLI dry-run publish via Verdaccio
- WASM package dry-run build and metadata verification

The npm CLI package itself is plain JavaScript, so there is no extra Node build step.

## Release surface

The first stable release consists of these public artefacts:

- a draft GitHub Release with native archives for:
  - `ftaql-x86_64-apple-darwin.tar.gz`
  - `ftaql-aarch64-apple-darwin.tar.gz`
  - `ftaql-x86_64-pc-windows-msvc.zip`
  - `ftaql-aarch64-pc-windows-msvc.zip`
  - `ftaql-x86_64-unknown-linux-musl.tar.gz`
  - `ftaql-aarch64-unknown-linux-musl.tar.gz`
  - `ftaql-arm-unknown-linux-musleabi.tar.gz`
- the Rust crate `ftaql` on crates.io
- the npm package `@piklv/ftaql-cli` on npm
- the npm package `@piklv/ftaql-wasm` on npm, published from the same release workflow after a manual approval gate

## Author prerequisites

Before tagging a release, verify all of the following:

- GitHub Actions secrets exist at the repository level:
  - `CARGO_REGISTRY_TOKEN`
  - `NPM_TOKEN`
- GitHub workflow permissions allow release creation and asset uploads
  - the workflow requests `contents: write`
  - repository Actions are enabled
- the repository has a protected Actions environment named `wasm-publish`
  - add the required reviewers who should approve `@piklv/ftaql-wasm`
- your npm account is ready for both automated npm publishes:
  - `npm whoami` succeeds
- the crates.io account that will publish `ftaql` is ready for a first publish

Useful checks:

```sh
gh secret list --app actions
gh api repos/pikulev/ftaql/actions/permissions
gh api repos/pikulev/ftaql/actions/permissions/workflow
npm whoami
```

## Release prep PR

Do release preparation in a normal PR, then merge it into `main`.

For a version bump:

1. Update `CHANGELOG.md`.
2. Keep the version aligned in:
   - `package.json`
   - `packages/ftaql/package.json`
   - `crates/ftaql/Cargo.toml`
   - `crates/ftaql-wasm/Cargo.toml`
3. Run `cargo update` so the lockfile stays in sync.
4. Merge the PR and wait for `test` to pass on `main`.

For `v1.0.0`, the version is already aligned in the files above.

## Pre-tag checklist

Run through this checklist immediately before creating the release tag:

- `git status --short --branch` is clean
- `gh run list --limit 5` shows the latest `main` run as successful
- `cargo publish --dry-run` succeeds in `crates/ftaql`
- the latest `CHANGELOG.md` entry is ready to become release notes
- the `wasm-publish` environment is configured with the intended reviewers
- you understand that `packages/ftaql/binaries` is populated by CI, not by the repository checkout

That last point matters because `packages/ftaql/check.js` enforces the presence of all platform binaries before the npm CLI package can be published. A plain local `npm publish` from the repository will fail unless the binaries have already been extracted into `packages/ftaql/binaries`.

## Automated release flow

The automated release covers GitHub Release assets, `ftaql` on crates.io, `@piklv/ftaql-cli` on npm, and `@piklv/ftaql-wasm` on npm.

Create and push the tag from a clean local checkout:

```sh
git tag v1.0.0
git push origin v1.0.0
```

Then monitor the workflow:

```sh
gh run list --workflow release.yml --limit 5
gh run watch
```

The release workflow should produce:

- aligned-version validation before any publish step runs
- Rust formatting, build, and test checks before any publish step runs
- a draft GitHub Release with all seven platform archives attached
- a published `ftaql` crate on crates.io
- a published `@piklv/ftaql-cli` package on npm
- a generated `@piklv/ftaql-wasm` package artifact ready for publish
- a paused approval step on the `wasm-publish` environment before the WASM npm publish
- a published `@piklv/ftaql-wasm` package on npm after approval

The WASM publish is no longer a separate manual post-release procedure. It is built and verified in the same tagged workflow, then waits for approval before the final npm publish step.

## Post-release verification

After all publishes complete:

- verify the draft GitHub Release contains all seven archives
- verify `ftaql` is visible on crates.io
- verify `@piklv/ftaql-cli` is visible on npm
- verify `@piklv/ftaql-wasm` is visible on npm
- verify install smoke tests work from the published npm package
- review and publish the GitHub Release from draft state

Suggested smoke checks:

```sh
npx @piklv/ftaql-cli . --db ./ftaql.sqlite
npm view @piklv/ftaql-cli version
npm view @piklv/ftaql-wasm version
```

## Code coverage

Coverage is reported during the `test` workflow.

To inspect it locally:

```sh
cargo install cargo-tarpaulin
cargo tarpaulin
```

`cargo-tarpaulin` is intentionally not installed as a build dependency and should be installed manually when needed.
