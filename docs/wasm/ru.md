# WebAssembly API

## Публичный API

Пакет `@piklv/ftaql-wasm` анализирует один исходный файл в браузере. Исходный код передаётся строкой, поэтому доступ к файловой системе не требуется.

Для анализа пакет экспортирует функцию:

```ts
analyze_file_ftaql(source_code: string, use_tsx: boolean, include_comments: boolean): string
```

Перед первым вызовом загрузите WASM через `init()`. Функция анализа вернёт JSON-строку, которую можно преобразовать в объект через `JSON.parse(...)`:

```js
import init, { analyze_file_ftaql } from "@piklv/ftaql-wasm";

await init();
const json = analyze_file_ftaql(sourceCode, true, false);
const analysis = JSON.parse(json);
```

## Что возвращается

При успешном анализе результат содержит один объект `FileData`, сериализованный в JSON:

- `size_metrics` — метрики размера файла;
- `complexity_metrics` — метрики сложности;
- `scores.file_score` — сводную оценку файла.

Поскольку WASM не получает другие файлы проекта, он не строит граф зависимостей и не ищет циклы. Поэтому `coupling_metrics` отсутствует, а `scores.coupling_score` равен `0.0`.

## Ограничения и нюансы

- Поле `file_name` всегда фиксировано как `source.ts`.
- Ошибки разбора передаются в JavaScript как исключения через `wasm_bindgen::throw_str(...)`.
- `use_tsx` включает или отключает разбор синтаксиса TSX.
- `include_comments` влияет на `line_count`.

Формула и интерпретация `file_score` описаны в разделе [«Скоринг»](../scoring/ru.md).
