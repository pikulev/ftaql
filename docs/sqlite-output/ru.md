# SQLite output

## Роль SQLite в FtaQl

Project-level CLI и Node wrapper работают в SQLite-first режиме. Они не выдают полный analysis JSON в stdout, а сохраняют нормализованный snapshot в базу данных.

Основная функция записи:

```rust
persist_run(
    db_path: &str,
    analysis_output: &FtaQlJsonOutput,
    options: &PersistRunOptions<'_>,
) -> Result<PersistRunSummary>
```

## Что хранится

Текущая schema version: `3`.

Основные таблицы:

- `analysis_runs`
- `files`
- `file_dependencies`
- `cycles`
- `cycle_files`
- `cycle_edges`

В `analysis_runs` можно опционально хранить два поля ревизии:

- `revision` для точного идентификатора снапшота, обычно commit SHA
- `ref_label` для человекочитаемой метки ветки, тега или канала

## Практические нюансы

- Повторные запуски append-ятся в ту же базу как новые строки `analysis_runs`.
- `revision` удобен для точного сравнения снапшотов, а `ref_label` для фильтрации или группировки по ветке, тегу или release line.
- `PersistRunSummary` возвращает `run_id`, `file_count`, `cycle_count`, `runtime_cycle_count`.
- В `file_dependencies`, `cycle_files` и `cycle_edges` попадают только связи между файлами, которые реально присутствуют в `findings`.
- `config_json` хранит уже resolved config, а не исходный текст файла.
- Нормализованная модель циклов в SQLite разворачивает `project_analysis.cycle_members` и `CycleInfo.graph` в SQL-friendly таблицы.

## Пример запроса

```sql
SELECT file_path, file_score, coupling_score
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

Смысл и формулы для `file_score` и `coupling_score` вынесены в [`../scoring/ru.md`](../scoring/ru.md).
