# WebAssembly API

## Public API

The `@piklv/ftaql-wasm` package is intended for browser-side single-file analysis.

Exported function:

```ts
analyze_file_ftaql(source_code: string, use_tsx: boolean, include_comments: boolean): string
```

Expected browser flow:

```js
import init, { analyze_file_ftaql } from "@piklv/ftaql-wasm";

await init();
const json = analyze_file_ftaql(sourceCode, true, false);
const analysis = JSON.parse(json);
```

## What it returns

The result is always a JSON string for a single `FileData`.

- it includes `size_metrics`
- it includes `complexity_metrics`
- it includes `scores.file_score`
- `coupling_metrics` are absent
- `scores.coupling_score` is `0.0`

## Limits and caveats

- WASM analyzes exactly one source file.
- There is no filesystem access.
- No project-level dependency graph or cycle analysis is built.
- The `file_name` field is always hardcoded to `source.ts`.
- Parse failures are surfaced as JS exceptions through `wasm_bindgen::throw_str(...)`.
- `use_tsx` controls TSX vs non-TSX parsing mode.
- `include_comments` affects `line_count`.

For `file_score` details and why `coupling_score` is `0.0` in WASM, see [`../scoring/en.md`](../scoring/en.md).
