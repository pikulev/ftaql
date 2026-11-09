# Результаты в SQLite

## Роль SQLite в FtaQl

При анализе проекта CLI и обёртка для Node.js сохраняют результаты в SQLite. Каждый запуск добавляет в базу отдельный набор метрик и связей. Эти данные можно запрашивать через SQL и сравнивать между ревизиями; stdout CLI содержит только краткую сводку.

Основная функция записи:

```rust
persist_run(
    db_path: &str,
    analysis_output: &FtaQlJsonOutput,
    options: &PersistRunOptions<'_>,
) -> Result<PersistRunSummary>
```

## Что хранится

Текущая версия схемы: `3`.

Основные таблицы:

- `analysis_runs`
- `files`
- `file_dependencies`
- `cycles`
- `cycle_files`
- `cycle_edges`

В `analysis_runs` можно опционально хранить два поля ревизии:

- `revision` для точного идентификатора ревизии, обычно SHA коммита
- `ref_label` для человекочитаемой метки ветки, тега или канала

## Практические нюансы

- Повторные запуски добавляют в ту же базу новые строки `analysis_runs`.
- По `revision` можно сравнивать конкретные ревизии, а по `ref_label` — фильтровать и группировать запуски по ветке, тегу или линии релизов.
- `PersistRunSummary` возвращает `run_id`, `file_count`, `cycle_count`, `runtime_cycle_count`.
- В `file_dependencies`, `cycle_files` и `cycle_edges` попадают только связи между файлами, которые реально присутствуют в `findings`.
- `config_json` хранит итоговую конфигурацию после объединения с настройками по умолчанию.
- Данные из `project_analysis.cycle_members` и `CycleInfo.graph` разворачиваются в таблицы циклов, их участников и рёбер. Так их можно запрашивать через SQL.

## Пример запроса

Чтобы найти файлы с самыми высокими оценками в последнем запуске, отсортируйте строки таблицы `files` по `file_score` и `coupling_score`:

```sql
SELECT file_path, file_score, coupling_score
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

Интерпретация этих оценок и формулы их расчёта описаны в разделе [«Скоринг»](../scoring/ru.md).
