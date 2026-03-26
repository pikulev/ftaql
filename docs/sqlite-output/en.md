# SQLite output

## SQLite's role in FtaQl

The project-level CLI and Node wrapper run in a SQLite-first mode. They do not emit the full analysis JSON to stdout. Instead, they persist a normalized snapshot into SQLite.

Core persistence function:

```rust
persist_run(
    db_path: &str,
    analysis_output: &FtaQlJsonOutput,
    options: &PersistRunOptions<'_>,
) -> Result<PersistRunSummary>
```

## What gets stored

Current schema version: `3`.

Main tables:

- `analysis_runs`
- `files`
- `file_dependencies`
- `cycles`
- `cycle_files`
- `cycle_edges`

`analysis_runs` can also store two optional revision metadata fields:

- `revision` for the exact snapshot identifier, usually a commit SHA
- `ref_label` for a human-readable branch, tag, or channel label

## Practical caveats

- Repeated runs are appended into the same database as new `analysis_runs` rows.
- `revision` is useful for exact snapshot comparisons, while `ref_label` is useful for filtering or grouping by branch, tag, or release line.
- `PersistRunSummary` returns `run_id`, `file_count`, `cycle_count`, and `runtime_cycle_count`.
- `file_dependencies`, `cycle_files`, and `cycle_edges` only include relationships between files that are actually present in `findings`.
- `config_json` stores the resolved config, not the raw source file text.
- The normalized cycle model expands `project_analysis.cycle_members` and `CycleInfo.graph` into SQL-friendly tables.

## Query example

```sql
SELECT file_path, file_score, coupling_score
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

The meaning and formulas behind `file_score` and `coupling_score` are documented in [`../scoring/en.md`](../scoring/en.md).
