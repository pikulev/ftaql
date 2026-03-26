# Обзор публичных API

FtaQl предоставляет три разные публичные поверхности, и у них разные реальные возможности:

| Артефакт | Для чего подходит | Что принимает | Что возвращает | Ограничения |
| --- | --- | --- | --- | --- |
| `ftaql` crate + native CLI | Полный анализ проекта, CI, локальные snapshots | Путь к проекту, конфиг, SQLite metadata | `FtaQlJsonOutput` в библиотеке, SQLite snapshot + stdout summary в CLI | CLI не печатает полный JSON |
| `@piklv/ftaql-cli` | npm CLI и запуск из JS/TS | CLI args или `runFtaQl(projectPath, options)` | Строку stdout нативного бинарника | Это thin wrapper, не отдельный движок |
| `@piklv/ftaql-wasm` | Browser playground, sandbox, editor integration | Строку исходника, `use_tsx`, `include_comments` | JSON-строку одного `FileData` | Нет ФС и project-level coupling |

## Быстрый выбор

- Используйте native CLI или `@piklv/ftaql-cli`, если нужен project-level analysis, cycles и SQLite history.
- Используйте Rust library API, если вы интегрируете анализ в Rust и хотите работать с `FtaQlJsonOutput`.
- Используйте WASM только для isolated single-file analysis в браузере.

## Общие нюансы

- Библиотечный и CLI пути используют один и тот же analysis engine из `crates/ftaql`.
- Node wrapper синхронно запускает нативный бинарник через `execFileSync`.
- WASM не знает о проекте целиком, поэтому `coupling_metrics` там отсутствует, а `file_name` всегда `source.ts`.
- Конфигурация анализа живет в `ftaql.json`; CLI-флаги в основном управляют путем к конфигу и metadata snapshot-а.
