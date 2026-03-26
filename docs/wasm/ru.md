# WebAssembly API

## Публичный API

Пакет `@piklv/ftaql-wasm` предназначен для browser-side single-file analysis.

Экспортируемая функция:

```ts
analyze_file_ftaql(source_code: string, use_tsx: boolean, include_comments: boolean): string
```

Ожидаемый browser flow:

```js
import init, { analyze_file_ftaql } from "@piklv/ftaql-wasm";

await init();
const json = analyze_file_ftaql(sourceCode, true, false);
const analysis = JSON.parse(json);
```

## Что возвращается

Результат всегда является JSON-строкой одного `FileData`.

- есть `size_metrics`
- есть `complexity_metrics`
- есть `scores.file_score`
- `coupling_metrics` отсутствует
- `scores.coupling_score` равен `0.0`

## Ограничения и нюансы

- WASM анализирует только один исходный файл.
- Доступа к файловой системе нет.
- Project-level dependency graph и cycle analysis не строятся.
- Поле `file_name` всегда фиксировано как `source.ts`.
- Ошибки парсинга пробрасываются как JS exception через `wasm_bindgen::throw_str(...)`.
- `use_tsx` управляет режимом парсинга TSX/не-TSX.
- `include_comments` влияет на `line_count`.

Подробно про `file_score` и почему `coupling_score` в WASM равен `0.0`: [`../scoring/ru.md`](../scoring/ru.md).
