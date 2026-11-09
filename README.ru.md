# FtaQl

[English](https://github.com/pikulev/ftaql/blob/main/packages/ftaql/README.md) | [Русский](https://github.com/pikulev/ftaql/blob/main/README.ru.md)

FtaQl помогает находить сложные файлы и тесно связанные модули в проектах на TypeScript и JavaScript. Инструмент анализирует репозиторий и сохраняет результаты в SQLite, где их можно исследовать обычными SQL-запросами.

После первого запуска можно найти файлы с высокими оценками сложности и связности. Если сохранять результаты для нескольких ревизий, можно проследить, где растут эти оценки и когда появляются циклы зависимостей. Такая история помогает оценивать изменения при рефакторинге и в CI.

FtaQl начинался как форк [`sgb-io/fta`](https://github.com/sgb-io/fta) и со временем вырос в самостоятельное продолжение этой идеи. Я благодарен автору оригинального проекта за вдохновение и за возможность развить идею дальше.

---

Ядро написано на Rust, поэтому повторные прогоны по большим кодовым базам и истории ревизий остаются практичными. На Apple M1 FtaQl анализирует до **10000 файлов в секунду**.

## Что собирает FtaQl

При анализе проекта через нативный CLI или обёртку для Node.js FtaQl сохраняет следующие метрики и данные:

| Метрика / артефакт | Что означает |
| --- | --- |
| `file_score` | Сводная оценка файла по его собственным метрикам. |
| `coupling_score` | Сводная оценка риска по связям файла в контексте проекта. |
| Цикломатическая сложность | Сколько независимых путей выполнения есть в коде. |
| Метрики Холстеда | Метрики по операторам и операндам, которые помогают оценить объём и сложность кода. |
| Входящая и исходящая связность | Кто зависит от файла и от скольких файлов зависит он сам. |
| Сила зависимостей | Насколько плотными являются связи между модулями. |
| Циклы полного графа | Циклы во всём графе зависимостей проекта, включая связи только по типам. |
| `runtime`-циклы | Циклы по зависимостям, которые реально участвуют в исполнении. |
| Результаты запусков в SQLite | Нормализованные данные для SQL-запросов и сравнения ревизий. |

Формулы, ограничения и интерпретация значений описаны в разделе [«Скоринг»](https://github.com/pikulev/ftaql/blob/main/docs/scoring/ru.md).

Циклы различаются составом зависимостей. `Runtime`-цикл включает связи, которые участвуют в исполнении кода. Цикл полного графа может также включать зависимости только по типам: они могут влиять на архитектуру и инструменты разработки, даже если не участвуют в исполнении.

## Быстрый старт: запуск, сохранение и запросы

Чтобы проанализировать проект, запустите нативный CLI или обёртку для Node.js. CLI сохранит результаты в SQLite; после этого их можно запрашивать через SQL.

Для текущей рабочей копии укажите каталог проекта и путь к базе:

```bash
npx @piklv/ftaql-cli path/to/project --db ./ftaql.sqlite
```

Если вы накапливаете результаты нескольких запусков, добавьте метаданные ревизии:

```bash
npx @piklv/ftaql-cli path/to/project \
  --db ./ftaql.sqlite \
  --revision "$(git rev-parse HEAD)" \
  --ref main
```

Если настройки находятся в отдельном файле, укажите путь к нему:

```bash
npx @piklv/ftaql-cli path/to/project \
  --db ./ftaql.sqlite \
  --config-path ./path/to/ftaql.json
```

Параметр `--db` обязателен. Параметры `--config-path`, `--revision` и `--ref` необязательны.

Если вы записываете несколько запусков в одну SQLite-базу:

- `--revision` сохраняет точный идентификатор состояния, обычно SHA коммита
- `--ref` сохраняет человекочитаемую метку ветки, тега или канала, например `main`, `release/1.2` или `nightly`

Используйте оба параметра, чтобы сравнивать конкретные ревизии и группировать запуски по ветке или линии релизов.

## Что можно спрашивать у истории запусков

Сохранённые результаты позволяют находить файлы с высокими оценками и сравнивать запуски. Начните с файлов с наибольшим `file_score` в последнем запуске:

```sql
SELECT file_path, file_score, coupling_score
FROM files
WHERE run_id = (SELECT MAX(id) FROM analysis_runs)
ORDER BY file_score DESC, coupling_score DESC
LIMIT 20;
```

Если команде удобнее работать с категориями, задайте пороги для тех же оценок через SQL. В примере ниже пороги условные: FtaQl не задаёт такой шкалы, поэтому её нужно настроить под свой проект:

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

Чтобы найти участников `runtime`-циклов в конкретной ревизии, выберите её по `revision`:

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

- Перед рефакторингом и после него сохраните результаты анализа монорепозитория, затем сравните метрики через SQL.
- Для наблюдения за изменениями накапливайте результаты по коммитам или CI-сборкам в одной базе и сравнивайте их по `revision` и `ref`.
- Для отчётов и дашбордов используйте SQL: выбирайте файлы с высокими оценками, исследуйте циклы и формируйте JSON для скриптов.

## Какие метрики считает FtaQl

Таблица выше объясняет отдельные метрики. Чтобы выбрать, какие из них смотреть первыми, исходите из задачи:

- Для поиска сложных файлов отсортируйте результаты по `file_score`.
- Для оценки связей между файлами посмотрите `coupling_score`.
- Для анализа циклических зависимостей сравните циклы полного графа и `runtime`-циклы.

Формулы и ограничения этих оценок описаны в разделе [«Скоринг»](https://github.com/pikulev/ftaql/blob/main/docs/scoring/ru.md).

## Что сохраняется в SQLite

FtaQl сохраняет нормализованные результаты анализа проекта в SQLite. Основные таблицы:

- `analysis_runs` для метаданных запуска и итоговой конфигурации
- `files` для метрик на уровне файлов
- `file_dependencies` для рёбер зависимостей между проанализированными файлами
- `cycles`, `cycle_files` и `cycle_edges` для нормализованных данных о циклах

Эти таблицы позволяют хранить результаты для нескольких ревизий в одной базе и сравнивать их через SQL. Формат данных описан в разделе [«Результаты в SQLite»](https://github.com/pikulev/ftaql/blob/main/docs/sqlite-output/ru.md).

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

Если анализ нужно запустить из JavaScript, вызовите `runFtaQl(projectPath, options)` из пакета `@piklv/ftaql-cli`. В `options.dbPath` укажите обязательный путь к SQLite-базе:

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

Обёртка сохраняет результаты анализа в SQLite и возвращает краткую сводку CLI из `stdout`. Необязательные параметры `configPath`, `revision` и `ref` передаются в нативный бинарник.

## Конфигурация

По умолчанию нативный CLI ищет `ftaql.json` в корне анализируемого проекта. Этот путь можно переопределить через `--config-path`.

В файле `ftaql.json` доступны следующие настройки:

- `extensions`
- `exclude_filenames`
- `exclude_directories`
- `score_cap`
- `include_comments`
- `exclude_under`

При анализе проекта FtaQl также учитывает `.gitignore`. Поэтому `node_modules` обычно пропускается автоматически, если уже игнорируется git-правилами. Если каталог не покрыт `.gitignore`, добавьте его в `exclude_directories`.

При разрешении импортов FtaQl также автоматически обнаруживает `tsconfig.json` и `jsconfig.json`. Поддерживаются:

- `compilerOptions.paths`
- `compilerOptions.baseUrl`
- унаследованные конфиги через `extends`
- ссылки между проектами (project references), которые учитываются при разрешении импортов

Как эти настройки применяются при анализе, описано в разделе [«Конфигурация»](https://github.com/pikulev/ftaql/blob/main/docs/configuration/ru.md).

## WebAssembly

Пакет `@piklv/ftaql-wasm` предназначен для браузера и анализирует по одному исходному файлу:

- вход: строка с исходным кодом
- выход: JSON-строка для одного файла
- нет доступа к файловой системе
- нет анализа связей между файлами и циклов зависимостей

Без графа проекта `coupling_score` всегда равен `0.0`. Порядок вызова и ограничения описаны в разделе [«WebAssembly API»](https://github.com/pikulev/ftaql/blob/main/docs/wasm/ru.md).

## Документация

Начните с [обзора публичных API](https://github.com/pikulev/ftaql/blob/main/docs/overview/ru.md), чтобы выбрать CLI, Rust API или WASM. Все разделы доступны в [оглавлении документации](https://github.com/pikulev/ftaql/tree/main/docs).

## Лицензия

MIT
