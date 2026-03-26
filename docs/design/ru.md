# Технический дизайн

FtaQl состоит из общего Rust-ядра и двух thin integration layers:

- `crates/ftaql` - analysis engine, JSON data model, native CLI, SQLite persistence
- `packages/ftaql` - Node.js wrapper над нативным бинарником
- `crates/ftaql-wasm` - browser-side single-file API

## Project-level pipeline

1. чтение `ftaql.json`
2. обход файлов через `ignore`
3. парсинг через SWC
4. разрешение импортов через resolver с учетом `tsconfig/jsconfig`
5. coupling and cycle analysis
6. file-level scoring
7. запись snapshot в SQLite

## Главная архитектурная граница

Project-level analysis живет в Rust core и доступен через library API, native CLI и Node wrapper. WASM намеренно ограничен одним файлом и не строит межфайловый граф зависимостей.
