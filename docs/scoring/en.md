# Scoring

## What `file_score` and `coupling_score` are for

FtaQl computes two composite scores:

- `file_score` shows how "heavy" a file looks on its own
- `coupling_score` shows how risky the file's relationships look in the context of the whole project

Both scores are best read as relative indexes inside one repository and across revision history, not as a universal external quality scale.

## `file_score` formula

In the current codebase, `file_score` is calculated through `FileScoreFormula::Original`.

Its inputs are:

- `line_count`
- `cyclomatic`
- `halstead.program_length`
- `halstead.vocabulary_size`
- `halstead.difficulty`

The full `HalsteadMetrics` structure contains more fields, but only these are used directly in the `file_score` formula.

In simplified form, the calculation looks like this:

```text
factor =
  1, if ln(cyclomatic) < 1
  otherwise sqrt(line_count * difficulty) / sqrt(sqrt(ln(cyclomatic) * ln(difficulty)) * ln(program_length))

absolute_file_score =
  171 - 5.2 * ln(vocabulary_size) - 0.23 * cyclomatic - 16.2 * ln(factor)

file_score =
  max(0, 100 - absolute_file_score * 100 / 171)
```

Practical meaning:

- higher `file_score` usually means the file is harder to maintain
- it is useful for hotspot sorting
- `score_cap` in config is checked against `file_score`

Important:

- `include_comments` changes `line_count`, which also changes `file_score`
- this is a heuristic, not an absolute scientific measure of file quality

## `coupling_score` formula

In the current codebase, `coupling_score` is calculated through `CouplingScoreFormula::Original`.

Its inputs are:

- `Ca` = `afferent_coupling`
- `Ce` = `efferent_coupling`
- `instability`
- `dependency_strength`
- participation in a static cycle through `cycle_id`
- additional penalties for instability, "bad core", and "orphan abstraction" cases

In simplified form, the calculation looks like this:

```text
if Ca + Ce == 0:
  coupling_score = 0

Ds = sum(dependency_strength)
S_bar = Ds / max(1, Ce)
cycle_factor = 1 + static_cycle_size if cycle_id exists, otherwise 1

coupling_score =
  (Ca + Ce)
  + 12 * ln((S_bar / 10) + 1)
  + 90 * ln(cycle_factor)
  + penalty_instability(instability)
  + 50 * key_risk(Ca, instability)
  + 50 * orphan_risk(Ca, instability)
```

Practical meaning:

- higher `coupling_score` means more architectural tension around the file
- it is most useful when comparing files inside one project and across revisions
- this is not normalized to a `0..100` scale: the current code returns the raw final score

Important:

- the formula uses static `cycle_id`, not `runtime_cycle_id`
- runtime cycles appear in project analysis output, but they do not directly feed the current `coupling_score`
- if a file has no relationships, `coupling_score` is immediately `0.0`

## What Halstead means in FtaQl

Halstead metrics in FtaQl are computed from operators and operands in the AST. The structure includes:

- unique and total operators
- unique and total operands
- `program_length`
- `vocabulary_size`
- `volume`
- `difficulty`
- `effort`
- `time`
- `bugs`

For users, the practical reading is:

- Halstead helps estimate code volume and cognitive density
- the whole structure is not injected into `file_score` as-is
- the current `file_score` formula uses `program_length`, `vocabulary_size`, and `difficulty`

## How to interpret the numbers

For a first pass, three steps are usually enough:

1. Sort files by `file_score`
2. Separately sort by `coupling_score`
3. Check where high scores and cycles overlap

You can build empirical ranks with plain SQL on top of the `files` table. This is not a built-in FtaQl scale, just a local team interpretation.

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

Those thresholds should be tuned to your own project.

## Caveats

- `file_score` and `coupling_score` are best used for comparison inside one project and across time, not as a universal external grade.
- `score_cap` only applies to `file_score`.
- In WASM, `coupling_score` is always `0.0` because there is no project-level dependency graph or cycle analysis.
- Type-only dependencies can form a full-graph cycle that affects architecture and tooling without necessarily affecting runtime.
- `crates/COUPLING_SCORE.md` is useful as an engineering note, but it does not fully match the current implementation and should not be treated as the user-facing specification without caveats.

## Related docs

- [`../configuration/en.md`](../configuration/en.md)
- [`../sqlite-output/en.md`](../sqlite-output/en.md)
- [`../rust-core/en.md`](../rust-core/en.md)
- [`../wasm/en.md`](../wasm/en.md)
