# Configuration

## Where config lives

By default, FtaQl looks for `ftaql.json` in the analyzed project root. The CLI only lets you override the path to that file:

```bash
ftaql /path/to/project --db ./ftaql.sqlite --config-path ./ftaql.json
```

If the config path is not explicitly provided and the file is missing, FtaQl falls back to the default config. If the path was explicitly provided and the file is missing, the CLI exits with an error.

## Supported fields

- `extensions`
- `exclude_filenames`
- `exclude_directories`
- `score_cap`
- `include_comments`
- `exclude_under`

## Default values

- `extensions`: `.js`, `.jsx`, `.ts`, `.tsx`
- `exclude_filenames`: `.d.ts`, `.min.js`, `.bundle.js`
- `exclude_directories`: `/dist`, `/bin`, `/build`
- `score_cap`: `1000`
- `include_comments`: `false`
- `exclude_under`: `6`

## Merge behavior

- `extensions`, `exclude_filenames`, and `exclude_directories` are appended to the defaults.
- `score_cap`, `include_comments`, and `exclude_under` override defaults.
- The resolved config is serialized and stored in SQLite as `analysis_runs.config_json`.

## `tsconfig.json` and `jsconfig.json`

The resolver understands:

- `compilerOptions.paths`
- `compilerOptions.baseUrl`
- `extends`
- project references

This affects dependency resolution and coupling analysis correctness.

## Important caveats

- The default `exclude_directories` only include `/dist`, `/bin`, and `/build`.
- During project-level analysis, FtaQl separately applies `.gitignore` and git exclude rules in addition to `ftaql.json`.
- This means `node_modules` is often skipped automatically when it is already ignored by git rules, but it is not part of the default `exclude_directories`.
- If a directory is not covered by `.gitignore` and you still want to skip it, add it explicitly to `exclude_directories`.
- `include_comments` affects `line_count`, which also affects `file_score`.
- `score_cap` exits the process with code `1` when a file exceeds the threshold.
- `exclude_under` is part of the public config and is persisted into SQLite, but the current pipeline does not apply it during traversal or file filtering.
- For the meaning and formulas behind `file_score` and `coupling_score`, see [`../scoring/en.md`](../scoring/en.md).
