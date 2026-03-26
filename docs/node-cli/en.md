# Node CLI wrapper

## What gets published

The `@piklv/ftaql-cli` package publishes:

- the `ftaql` CLI command
- the programmatic API `runFtaQl(projectPath, options): string`

It is a thin wrapper over the native Rust binary.

## CLI usage

All arguments passed to `ftaql` are forwarded directly to the native binary:

```bash
npx @piklv/ftaql-cli . --db ./ftaql.sqlite
npx @piklv/ftaql-cli . --db ./ftaql.sqlite --revision "$GITHUB_SHA" --ref "$GITHUB_REF_NAME"
```

If you append multiple runs into the same SQLite database, it is useful to pass both parameters:

- `revision` is the exact snapshot identifier, usually a commit SHA
- `ref` is a human-readable branch, tag, or channel label such as `main`, `release/1.2`, or `nightly`
- together they let you compare exact revisions while still grouping runs by branch or release line

## Programmatic API

```js
import { runFtaQl } from "@piklv/ftaql-cli";

const output = runFtaQl(".", {
  dbPath: "./ftaql.sqlite",
  revision: process.env.GIT_SHA,
  ref: process.env.GIT_BRANCH,
});
```

Rules:

- `options.dbPath` is required
- `configPath`, `revision`, and `ref` are optional
- the function is synchronous and uses `execFileSync`
- the return value is the native CLI stdout string, not a JSON object

## Binary matrix

- Windows: `x64`, `arm64`
- macOS: `x64`, `arm64`
- Linux: `x64`, `arm64`, `arm`

Linux binaries target `musl`, not `glibc`.

## Important caveats

- On `darwin` and `linux`, the wrapper tries to set mode `755` via `chmodSync(...)`.
- Unsupported platforms throw `Error("Unsupported platform: ...")`.
- Package publishing verifies that all target binaries exist via `node check.js`.
- The types in `@types/ftaql-cli.d.ts` also describe `FtaQlJsonOutput`, but `runFtaQl(...)` does not return that object.
