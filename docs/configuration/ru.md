# Конфигурация

## Где находится файл конфигурации

По умолчанию FtaQl ищет `ftaql.json` в корне анализируемого проекта. Если настройки лежат в другом месте, укажите путь через `--config-path`:

```bash
ftaql /path/to/project --db ./ftaql.sqlite --config-path ./ftaql.json
```

Если путь не указан и в корне проекта нет `ftaql.json`, FtaQl использует настройки по умолчанию. Если вы явно указали путь, а файла по нему нет, CLI завершится ошибкой.

## Поля и значения по умолчанию

Файл `ftaql.json` поддерживает следующие поля:

| Поле | Значение по умолчанию |
| --- | --- |
| `extensions` | `.js`, `.jsx`, `.ts`, `.tsx` |
| `exclude_filenames` | `.d.ts`, `.min.js`, `.bundle.js` |
| `exclude_directories` | `/dist`, `/bin`, `/build` |
| `score_cap` | `1000` |
| `include_comments` | `false` |
| `exclude_under` | `6` |

## Как применяются настройки

Списки `extensions`, `exclude_filenames` и `exclude_directories` из файла дополняют значения по умолчанию. Поля `score_cap`, `include_comments` и `exclude_under` заменяют соответствующие значения по умолчанию.

После объединения настроек FtaQl сохраняет итоговую конфигурацию в SQLite, в поле `analysis_runs.config_json`.

## Исключение каталогов

В список `exclude_directories` по умолчанию входят только `/dist`, `/bin` и `/build`. При обходе проекта FtaQl дополнительно учитывает `.gitignore` и правила git exclude.

Поэтому `node_modules` часто пропускается благодаря git-правилам, хотя его нет в списке исключений FtaQl по умолчанию. Если нужный каталог не покрыт git-правилами, укажите его в `exclude_directories`.

## Разрешение импортов

Чтобы определить, на какие файлы ссылаются импорты, FtaQl учитывает настройки `tsconfig.json` и `jsconfig.json`:

- `compilerOptions.paths`;
- `compilerOptions.baseUrl`;
- наследование через `extends`;
- ссылки между проектами (project references).

От разрешения импортов зависит, какие связи попадут в граф проекта и расчёт связности.

## Влияние настроек на результат

- `include_comments` меняет число строк `line_count`, которое участвует в расчёте `file_score`.
- Если `file_score` превышает `score_cap`, процесс завершается с кодом `1`.
- `exclude_under` сохраняется в конфигурации SQLite, но текущая реализация не использует это поле при обходе или фильтрации файлов.

Смысл и формулы `file_score` и `coupling_score` описаны в разделе [«Скоринг»](../scoring/ru.md).
