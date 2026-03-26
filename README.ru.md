# FtaQl

[English](https://github.com/pikulev/ftaql/blob/main/packages/ftaql/README.md) | [Русский](https://github.com/pikulev/ftaql/blob/main/README.ru.md)

FtaQl помогает быстро увидеть, где в TS/JS-проекте реально копится риск. Инструмент проходит по репозиторию, сохраняет результаты анализа в SQLite и превращает анализ кода в обычные SQL-запросы.

После запуска можно сразу проверить, какие файлы остаются самыми дорогими от ревизии к ревизии, где растёт связность и когда впервые появились runtime cycles. Это удобно для рефакторинга, CI и анализа истории, когда нужен не одноразовый отчёт, а накопленная история запусков.

FtaQl начинался как форк [`sgb-io/fta`](https://github.com/sgb-io/fta) и со временем вырос в самостоятельное продолжение этой идеи. Я благодарен автору оригинального проекта за вдохновение и за возможность развить идею дальше.

---

Ядро написано на Rust, поэтому повторные прогоны по большим кодовым базам и истории ревизий остаются практичными. На Apple M1 FtaQl анализирует до **10000 файлов в секунду**.

## Что собирает FtaQl

Для project-level анализа через native CLI и Node wrapper FtaQl сохраняет:

| Метрика / артефакт | Что означает |
| --- | --- |
| `file_score` | Сводная оценка файла по его собственным метрикам. |
| `coupling_score` | Сводная оценка риска по связям файла в контексте проекта. |
| Цикломатическая сложность | Сколько независимых путей выполнения есть в коде. |
| Метрики Холстеда | Метрики по операторам и операндам, которые помогают оценить объём и сложность кода. |
| Входящая и исходящая связность | Кто зависит от файла и от скольких файлов зависит он сам. |
| Сила зависимостей | Насколько плотными являются связи между модулями. |
| Циклы полного графа | Циклы во всём графе зависимостей проекта, включая type-only связи. |
| `runtime`-циклы | Циклы по зависимостям, которые реально участвуют в исполнении. |
| SQLite snapshots | Нормализованные результаты запусков для SQL-запросов и сравнения истории. |

Подробные формулы, caveats и интерпретация значений вынесены в [`docs/scoring/ru.md`](https://github.com/pikulev/ftaql/blob/main/docs/scoring/ru.md).

`Runtime`-цикл - это циклическая зависимость по рантайм-сущностям. Полный цикл может включать type-only зависимости: такой цикл может тормозить dev tooling и портить архитектуру, даже если на runtime он не влияет.

## Быстрый старт: запуск -> сохранение -> запросы

Анализ на уровне всего проекта доступен через нативный CLI и обёртку для Node.js. WASM-сборка анализирует только один файл и возвращает JSON только для него.
В WASM `coupling_score` всегда равен `0.0`, а граф зависимостей проекта и cycle analysis не строятся.

Базовый цикл простой: сохранить результаты анализа в SQLite, при необходимости пометить их `revision` и `ref`, а затем запрашивать накопленные данные через SQL.

Сохранить результаты анализа для текущей рабочей копии:

```bash
npx @piklv/ftaql-cli path/to/project --db ./ftaql.sqlite
```

Добавить метаданные ревизии, если вы накапливаете несколько запусков:

```bash
npx @piklv/ftaql-cli path/to/project \
  --db ./ftaql.sqlite \
  --revision "$(git rev-parse HEAD)" \
  --ref main
```

Подключить собственный конфигурационный файл при необходимости:

```bash
npx @piklv/ftaql-cli path/to/project \
  --db ./ftaql.sqlite \
  --config-path ./path/to/ftaql.json
```

Основные опции:

- `--db`
- `--config-path`
- `--revision`
- `--ref`

Если вы записываете несколько запусков в одну SQLite-базу:

- `--revision` сохраняет точный идентификатор состояния, обычно SHA коммита
- `--ref` сохраняет человекочитаемую метку ветки, тега или канала, например `main`, `release/1.2` или `nightly`
- вместе они позволяют сравнивать точные состояния и одновременно группировать запуски по ветке или линии релизов

## Что можно спрашивать у истории запусков

После сохранения результатов начинается самое интересное.

Самые проблемные файлы в последнем запуске:

```sql
SELECT file_path, file_score, coupling_score
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

Если нужно сразу разбить файлы по уровням, можно добавить к этим колонкам эмпирические ранги. Это не встроенная шкала FtaQl, а лишь пример интерпретации, поэтому пороги стоит калибровать под свой проект:

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

Сравнить средний `file_score` по ревизиям одной ветки:

```sql
SELECT ar.revision, AVG(f.file_score) AS avg_file_score
FROM analysis_runs ar
JOIN files f ON f.run_id = ar.id
WHERE ar.ref_label = 'main'
GROUP BY ar.revision
ORDER BY ar.created_at;
```

Посмотреть участников runtime cycles в конкретной ревизии:

```sql
SELECT cf.cycle_id, cf.file_path
FROM cycle_files cf
JOIN analysis_runs ar ON ar.id = cf.run_id
WHERE ar.revision = 'abc123'
  AND cf.cycle_kind = 'runtime'
ORDER BY cf.cycle_id, cf.file_path;
```

Подготовить hotspots по связности в JSON для следующего шага пайплайна:

```sql
SELECT json_group_array(
  json_object(
    'file_path', hotspot.file_path,
    'file_score', hotspot.file_score,
    'coupling_score', hotspot.coupling_score
  )
) AS hotspots
FROM (
  SELECT file_path, file_score, coupling_score
  FROM files
  WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
  ORDER BY coupling_score DESC, file_score DESC
  LIMIT 10
) AS hotspot;
```

## Типичные сценарии использования

- Снимают состояние большого TS/JS-монорепозитория до и после рефакторинга, а затем SQL-запросом проверяют, где на самом деле сосредоточен риск.
- Складывают в одну SQLite-базу результаты запусков по коммитам или CI-сборкам и сравнивают динамику по `revision` и `ref`.
- Используют SQL как основной слой аналитики: ищут hotspots, исследуют циклы и формируют JSON-подобные выборки для скриптов и дашбордов.

## Какие метрики считает FtaQl

Краткая расшифровка основных метрик уже есть в таблице выше, а формулы и caveats собраны в [`docs/scoring/ru.md`](https://github.com/pikulev/ftaql/blob/main/docs/scoring/ru.md). На практике чаще всего начинают с трёх осей:

- `file_score`, если нужно быстро найти самые "тяжёлые" файлы
- `coupling_score`, если нужно понять, где связи проекта создают основной риск
- циклов полного графа и `runtime`-циклов, если важно отделить architectural smell от runtime-проблем

## Что сохраняется в SQLite

FtaQl сохраняет нормализованные результаты анализа проекта в SQLite. Основные таблицы:

- `analysis_runs` для метаданных запуска и итоговой конфигурации
- `files` для метрик на уровне файлов
- `file_dependencies` для рёбер зависимостей между проанализированными файлами
- `cycles`, `cycle_files` и `cycle_edges` для нормализованных данных о циклах

Этого достаточно, чтобы накапливать множество ревизий в одной базе и работать с ними обычным SQL. Полное описание схемы есть в [`docs/sqlite-output/ru.md`](https://github.com/pikulev/ftaql/blob/main/docs/sqlite-output/ru.md).

## Использование FtaQl в npm-скриптах

Установите `@piklv/ftaql-cli`:

```bash
yarn add -D @piklv/ftaql-cli
# или
npm install --save-dev @piklv/ftaql-cli
# или
pnpm add -D @piklv/ftaql-cli
```

Затем добавьте скрипт:

```json
{
  "scripts": {
    "ftaql": "ftaql . --db ./ftaql.sqlite"
  }
}
```

## Использование FtaQl из кода

`@piklv/ftaql-cli` также экспортирует `runFtaQl(projectPath, options)`.

```javascript
import { runFtaQl } from "@piklv/ftaql-cli";
// Альтернатива для CommonJS:
// const { runFtaQl } = require("@piklv/ftaql-cli");

const output = runFtaQl("path/to/project", {
  dbPath: "./ftaql.sqlite",
  revision: process.env.GIT_SHA,
  ref: process.env.GIT_BRANCH,
});

console.log(output);
```

Что важно знать про обёртку для Node.js:

- `options.dbPath` обязателен
- обёртка сохраняет результат анализа и возвращает краткую сводку CLI из `stdout`
- `configPath`, `revision` и `ref` передаются в нативный бинарник

## Конфигурация

По умолчанию нативный CLI ищет `ftaql.json` в корне анализируемого проекта. Этот путь можно переопределить через `--config-path`.

Файл `ftaql.json` управляет поведением анализа. Например, в нём можно настроить:

- `extensions`
- `exclude_filenames`
- `exclude_directories`
- `score_cap`
- `include_comments`
- `exclude_under`

При разрешении импортов FtaQl также автоматически обнаруживает `tsconfig.json` и `jsconfig.json`. Поддерживаются:

- `compilerOptions.paths`
- `compilerOptions.baseUrl`
- унаследованные конфиги через `extends`
- project references, которые обнаруживает используемый резолвер

Примеры для monorepo и вложенных конфигов есть в [`docs/usage-patterns/ru.md`](https://github.com/pikulev/ftaql/blob/main/docs/usage-patterns/ru.md). Точный контракт конфигурации описан в [`docs/configuration/ru.md`](https://github.com/pikulev/ftaql/blob/main/docs/configuration/ru.md).

## WebAssembly

Пакет `@piklv/ftaql-wasm` предназначен для браузера и анализирует по одному исходному файлу:

- вход: строка с исходным кодом
- выход: JSON-строка для одного файла
- нет доступа к файловой системе
- нет project-level coupling analysis

## Документация

Полная документация находится в [`docs/`](https://github.com/pikulev/ftaql/tree/main/docs). В первую очередь стоит посмотреть [`docs/overview/ru.md`](https://github.com/pikulev/ftaql/blob/main/docs/overview/ru.md).

## Лицензия

MIT
