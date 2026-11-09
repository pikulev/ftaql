# Конфигурация

## Где живет конфиг

По умолчанию FtaQl ищет `ftaql.json` в корне анализируемого проекта. CLI позволяет переопределить только путь к этому файлу:

```bash
ftaql /path/to/project --db ./ftaql.sqlite --config-path ./ftaql.json
```

Если путь к конфигу явно не указан и файла нет, используется default config. Если путь был указан явно и файл отсутствует, CLI завершится ошибкой.

## Поддерживаемые поля

- `includes` — glob-паттерны для включения файлов в анализ
- `excludes` — glob-паттерны для исключения файлов из анализа
- `score_cap`
- `include_comments`
- `exclude_under`

## Пример конфига

```json
{
  "includes": ["**/*.ts", "**/*.tsx"],
  "excludes": ["**/*.d.ts", "dist/**", "__tests__/**"],
  "score_cap": 1000
}
```

## Дефолтные значения

- `includes`: `**/*.js`, `**/*.jsx`, `**/*.ts`, `**/*.tsx`
- `excludes`: `**/*.d.ts`, `**/*.min.js`, `**/*.bundle.js`, `dist/**`, `bin/**`, `build/**`
- `score_cap`: `1000`
- `include_comments`: `false`
- `exclude_under`: `6`

## Поведение при переопределении

- `includes` и `excludes`, если указаны, **полностью заменяют** дефолтные значения.
- `score_cap`, `include_comments` и `exclude_under` переопределяют дефолты.
- Resolved config сериализуется и сохраняется в SQLite как `analysis_runs.config_json`.

## `tsconfig.json` и `jsconfig.json`

Resolver учитывает:

- `compilerOptions.paths`
- `compilerOptions.baseUrl`
- `extends`
- project references

Это влияет на корректность dependency graph и coupling analysis.

## Важные нюансы

- При project-level анализе FtaQl отдельно применяет правила `.gitignore` и git exclude, помимо `ftaql.json`.
- Поэтому `node_modules` часто не попадает в обход автоматически, если уже игнорируется git-правилами, но это не часть дефолтных `excludes`.
- Если каталог не покрыт `.gitignore`, но его все равно нужно пропустить, добавьте его в `excludes`.
- `include_comments` влияет на `line_count`, а значит и на `file_score`.
- `score_cap` завершает процесс с кодом `1`, если файл превысил порог.
- `exclude_under` присутствует в публичной конфигурации и сохраняется в SQLite, но текущий pipeline его не применяет при обходе или фильтрации файлов.
- Подробно про смысл `file_score`, `coupling_score` и их формулы: [`../scoring/ru.md`](../scoring/ru.md).
