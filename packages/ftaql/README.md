# FtaQl

[English](https://github.com/pikulev/ftaql/blob/main/README.md) | [Русский](https://github.com/pikulev/ftaql/blob/main/README.ru.md)

FtaQl helps you see where risk is actually accumulating in a TS/JS project. It walks the repository, stores a snapshot in SQLite, and turns code analysis into plain SQL instead of guesswork.

After one run you can ask which files stay expensive across revisions, where coupling keeps growing, and when runtime cycles first appeared. That makes it useful for refactors, CI, and history analysis when you need an accumulated dataset instead of a one-off report.

FtaQl started as a fork of [`sgb-io/fta`](https://github.com/sgb-io/fta) and gradually grew into its own continuation of that idea. I am grateful to that project for the original inspiration and for making it possible to take the idea further.

---

The core is written in Rust, so repeated runs across large codebases and revision history stay practical. On Apple M1 hardware FtaQl can analyze up to **10000 files per second**.

## What FtaQl Collects

For project-level analysis through the native CLI and Node wrapper, FtaQl stores:

| Metric / artifact | What it means |
| --- | --- |
| `file_score` | A composite score for a file based on its own metrics. |
| `coupling_score` | A composite score for relationship risk in the context of the whole project. |
| Cyclomatic complexity | How many independent execution paths the code contains. |
| Halstead metrics | Operator/operand metrics that help estimate code volume and complexity. |
| Afferent and efferent coupling | Who depends on the file, and how many files the file depends on. |
| Dependency strength | How tight module-to-module relationships are. |
| Full-graph cycles | Cycles in the whole project dependency graph, including type-only edges. |
| `runtime` cycles | Cycles over dependencies that actually participate in execution. |
| SQLite snapshots | Normalized runs for SQL queries and historical comparisons. |

Detailed formulas, caveats, and interpretation notes are documented in [`docs/scoring/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/scoring/en.md).

A `runtime` cycle is a cyclic dependency through runtime entities. A full-graph cycle can include type-only dependencies, which may hurt tooling and architecture even when runtime is unaffected.

## Quickstart: run -> persist -> query

Project-level analysis is available through the native CLI and the Node.js wrapper. The WASM build analyzes a single file and returns JSON for that file only.
In WASM, `coupling_score` is always `0.0`, and there is no project-level dependency graph or cycle analysis.

The basic loop is simple: persist a snapshot to SQLite, attach `revision` and `ref` when needed, then query the accumulated runs with SQL.

Persist the current checkout:

```bash
npx @piklv/ftaql-cli path/to/project --db ./ftaql.sqlite
```

Append revision metadata as you collect more snapshots:

```bash
npx @piklv/ftaql-cli path/to/project \
  --db ./ftaql.sqlite \
  --revision "$(git rev-parse HEAD)" \
  --ref main
```

Use a custom config file when needed:

```bash
npx @piklv/ftaql-cli path/to/project \
  --db ./ftaql.sqlite \
  --config-path ./path/to/ftaql.json
```

Main options:

- `--db`
- `--config-path`
- `--revision`
- `--ref`

When you append multiple runs into the same SQLite database:

- `--revision` stores the exact snapshot identifier, usually a commit SHA
- `--ref` stores a human-readable branch, tag, or channel label such as `main`, `release/1.2`, or `nightly`
- using both lets you compare exact snapshots while still grouping runs by branch or release line

## Querying the Snapshot History

Once snapshots are in SQLite, the interesting part starts.

Worst files in the latest snapshot:

```sql
SELECT file_path, file_score, coupling_score
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

If your team prefers buckets right away, you can layer empirical ranks on top of the same columns. This is not an official FtaQl scale, just one example of a project-level interpretation, so the thresholds should be tuned to your own codebase:

```sql
SELECT
  file_path,
  file_score,
  CASE
    WHEN file_score <= 50 THEN 'OK'
    WHEN file_score <= 60 THEN 'Could be better'
    ELSE 'Needs improvement'
  END AS file_rank,
  coupling_score,
  CASE
    WHEN coupling_score <= 100 THEN 'OK'
    WHEN coupling_score <= 200 THEN 'Could be better'
    ELSE 'Needs improvement'
  END AS coupling_rank
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

Compare average `file_score` across revisions on one branch:

```sql
SELECT ar.revision, AVG(f.file_score) AS avg_file_score
FROM analysis_runs ar
JOIN files f ON f.run_id = ar.id
WHERE ar.ref_label = 'main'
GROUP BY ar.revision
ORDER BY ar.created_at;
```

Inspect runtime cycles for a specific revision:

```sql
SELECT cf.cycle_id, cf.file_path
FROM cycle_files cf
JOIN analysis_runs ar ON ar.id = cf.run_id
WHERE ar.revision = 'abc123'
  AND cf.cycle_kind = 'runtime'
ORDER BY cf.cycle_id, cf.file_path;
```

Shape coupling hotspots into JSON for downstream tooling:

```sql
SELECT json_group_array(
  json_object(
    'file_path', hotspot.file_path,
    'file_score', hotspot.file_score,
    'coupling_score', hotspot.coupling_score
  )
) AS hotspots
FROM (
  SELECT file_path, file_score, coupling_score
  FROM files
  WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
  ORDER BY coupling_score DESC, file_score DESC
  LIMIT 10
) AS hotspot;
```

## Typical Use Cases

- Snapshot a large TS/JS monorepo before and after a refactor, then query the worst files instead of guessing where the risk lives.
- Append runs for many commits or CI builds into the same SQLite database and compare trends by `revision` or `ref`.
- Use SQL as the analysis layer itself: aggregate hotspots, inspect cycles, or shape rows into JSON payloads for scripts and dashboards.

## What FtaQl Measures

The table above is the fastest way to understand the main metrics, while formulas and caveats live in [`docs/scoring/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/scoring/en.md). In practice, most teams start from three views:

- `file_score`, when they want the quickest list of the heaviest files
- `coupling_score`, when they want to see where project relationships create the most risk
- full-graph cycles and `runtime` cycles, when they need to separate architectural smells from runtime trouble

## What Gets Persisted

FtaQl stores normalized project snapshots in SQLite. The core tables are:

- `analysis_runs` for run metadata and resolved config
- `files` for per-file metrics
- `file_dependencies` for dependency edges between analyzed files
- `cycles`, `cycle_files`, and `cycle_edges` for normalized cycle data

That layout is enough to accumulate many revisions in one database and query them with plain SQL. For the full contract, see [`docs/sqlite-output/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/sqlite-output/en.md).

## Using FtaQl In Package Scripts

Install `@piklv/ftaql-cli`:

```bash
yarn add -D @piklv/ftaql-cli
# or
npm install --save-dev @piklv/ftaql-cli
# or
pnpm add -D @piklv/ftaql-cli
```

Then add a script:

```json
{
  "scripts": {
    "ftaql": "ftaql . --db ./ftaql.sqlite"
  }
}
```

## Using FtaQl From Code

`@piklv/ftaql-cli` also exports `runFtaQl(projectPath, options)`.

```javascript
import { runFtaQl } from "@piklv/ftaql-cli";
// CommonJS alternative:
// const { runFtaQl } = require("@piklv/ftaql-cli");

const output = runFtaQl("path/to/project", {
  dbPath: "./ftaql.sqlite",
  revision: process.env.GIT_SHA,
  ref: process.env.GIT_BRANCH,
});

console.log(output);
```

Important notes about the Node.js wrapper:

- `options.dbPath` is required
- the wrapper persists a snapshot and returns the CLI summary from stdout
- `configPath`, `revision`, and `ref` are forwarded to the native binary

## Configuration

By default, the native CLI looks for `ftaql.json` in the analyzed project root. You can override that path with `--config-path`.

The `ftaql.json` file controls analysis behavior such as:

- `includes` — glob patterns for files to include
- `excludes` — glob patterns for files to exclude
- `score_cap`
- `include_comments`
- `exclude_under`

During project-level analysis, FtaQl also respects `.gitignore`. That means `node_modules` is usually skipped automatically when it is already ignored by git rules. If a directory is not covered by `.gitignore`, add it to `excludes`. See [`docs/configuration/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/configuration/en.md) for details.

FtaQl also auto-detects `tsconfig.json` and `jsconfig.json` files when resolving imports. It supports:

- `compilerOptions.paths`
- `compilerOptions.baseUrl`
- inherited configs via `extends`
- project references discovered through the resolver

For monorepo and nested-config examples, see [`docs/usage-patterns/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/usage-patterns/en.md). For the exact config contract, see [`docs/configuration/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/configuration/en.md).

## WebAssembly

The `@piklv/ftaql-wasm` package is intended for browser usage and analyzes one source file at a time:

- input: source code string
- output: JSON string for a single file
- no filesystem access
- no project-level coupling analysis

## Docs

Read the full documentation in [`docs/`](https://github.com/pikulev/ftaql/tree/main/docs), especially [`docs/overview/en.md`](https://github.com/pikulev/ftaql/blob/main/docs/overview/en.md).

## License

MIT
