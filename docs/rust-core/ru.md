# Rust core и native CLI

## Публичные точки входа

Крейт `ftaql` экспортирует:

- `analyze_project(path: &String, config: Option<FtaQlConfigResolved>) -> FtaQlJsonOutput`
- `analyze_module(module: &swc_ecma_ast::Module, file_name: &str, line_count: usize) -> FileData`
- `FileData::abs_path(project_root: &str) -> PathBuf`

## Что делает `analyze_project(...)`

`analyze_project(...)` выполняет полный project-level pipeline:

1. канонизирует путь проекта
2. обходит файлы через `ignore::WalkBuilder`
3. применяет `.gitignore` и exclude-паттерны из конфига
4. резолвит импорты через resolver с учетом `tsconfig.json` и `jsconfig.json`
5. строит full cycles и runtime-only cycles
6. считает file-level metrics и собирает `FtaQlJsonOutput`

## Что делает `analyze_module(...)`

Эта функция полезна для single-module анализа, когда AST уже построен вызывающей стороной.

Важно:

- она считает `cyclomatic`, `halstead` и `file_score`
- `coupling_metrics` всегда `None`
- `scores.coupling_score` всегда `0.0`

Подробно про `file_score` и `coupling_score`: [`../scoring/ru.md`](../scoring/ru.md).

## Контракт native CLI

CLI принимает:

- обязательный positional `project`
- обязательный `--db`
- опциональный `--config-path`
- опциональный `--revision`
- опциональный `--ref`

Если CLI пишет несколько запусков в одну SQLite-базу, эти параметры помогают различать снапшоты:

- `--revision` хранит точный идентификатор ревизии, обычно commit SHA
- `--ref` хранит человекочитаемую метку ветки, тега или канала

CLI не печатает `FtaQlJsonOutput` в stdout. Он читает конфиг, запускает `analyze_project(...)`, пишет snapshot в SQLite через `persist_run(...)` и печатает короткое summary.

## Важные нюансы

- `analyze_project(...)` принимает `&String`, а не `&str`.
- В pipeline есть `expect()` и `unwrap()`, поэтому часть ошибок выражается паникой, а не `Result`.
- Если `score_cap` превышен, `check_score_cap_breach(...)` завершает процесс через `std::process::exit(1)`.
- `CycleInfo.graph` хранит индексы в `project_analysis.cycle_members`, а не дублирует пути внутри каждого цикла.
