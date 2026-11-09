# CLI и API для Node.js

## Что публикуется

Пакет `@piklv/ftaql-cli` запускает нативный бинарник FtaQl из Node.js. Для этого он предоставляет два способа вызова:

- CLI-команду `ftaql`
- функцию `runFtaQl(projectPath, options): string`

## Запуск из командной строки

Если вы запускаете `ftaql` через npm, обёртка передаёт все аргументы в нативный бинарник:

```bash
npx @piklv/ftaql-cli . --db ./ftaql.sqlite
npx @piklv/ftaql-cli . --db ./ftaql.sqlite --revision "$GITHUB_SHA" --ref "$GITHUB_REF_NAME"
```

Чтобы различать запуски в одной SQLite-базе, передавайте метаданные ревизии:

- `--revision` — точный идентификатор ревизии, обычно SHA коммита;
- `--ref` — человекочитаемая метка ветки, тега или канала, например `main`, `release/1.2` или `nightly`.

По `revision` можно сравнивать конкретные ревизии, а по `ref` — группировать запуски одной ветки или линии релизов.

## Вызов из JavaScript

Если анализ нужно запустить из программы, вызовите `runFtaQl(...)`. Путь к SQLite-базе передаётся обязательным параметром `options.dbPath`:

```js
import { runFtaQl } from "@piklv/ftaql-cli";

const output = runFtaQl(".", {
  dbPath: "./ftaql.sqlite",
  revision: process.env.GIT_SHA,
  ref: process.env.GIT_BRANCH,
});
```

Функция синхронно запускает CLI через `execFileSync` и возвращает строку его stdout. В этой строке находится краткая сводка; результаты анализа сохраняются в указанную SQLite-базу.

Параметры `configPath`, `revision` и `ref` необязательны. Описание `FtaQlJsonOutput` есть в `@types/ftaql-cli.d.ts`, но `runFtaQl(...)` этот объект не возвращает.

## Поддерживаемые платформы

Пакет выбирает бинарник под операционную систему и архитектуру:

- Windows: `x64`, `arm64`
- macOS: `x64`, `arm64`
- Linux: `x64`, `arm64`, `arm`

Бинарники для Linux собраны под `musl`.

## Важные нюансы

- На `darwin` и `linux` обёртка пытается выставить бинарнику права `755` через `chmodSync(...)`.
- Для неподдерживаемой платформы выбрасывается `Error("Unsupported platform: ...")`.
- Перед публикацией `node check.js` проверяет наличие бинарников для всех поддерживаемых платформ.
