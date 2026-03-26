# Node CLI wrapper

## Что публикуется

Пакет `@piklv/ftaql-cli` публикует:

- CLI-команду `ftaql`
- programmatic API `runFtaQl(projectPath, options): string`

Это thin wrapper над нативным Rust binary.

## CLI usage

Все аргументы, переданные `ftaql`, напрямую пробрасываются в нативный бинарник:

```bash
npx @piklv/ftaql-cli . --db ./ftaql.sqlite
npx @piklv/ftaql-cli . --db ./ftaql.sqlite --revision "$GITHUB_SHA" --ref "$GITHUB_REF_NAME"
```

Если вы сохраняете несколько запусков в одну SQLite-базу, полезно передавать оба параметра:

- `revision` это точный идентификатор снапшота, обычно commit SHA
- `ref` это человекочитаемая метка ветки, тега или канала, например `main`, `release/1.2` или `nightly`
- вместе они позволяют одновременно сравнивать конкретные ревизии и группировать запуски по ветке или release line

## Programmatic API

```js
import { runFtaQl } from "@piklv/ftaql-cli";

const output = runFtaQl(".", {
  dbPath: "./ftaql.sqlite",
  revision: process.env.GIT_SHA,
  ref: process.env.GIT_BRANCH,
});
```

Правила:

- `options.dbPath` обязателен
- `configPath`, `revision` и `ref` опциональны
- функция синхронная и использует `execFileSync`
- результатом является строка stdout нативного CLI, а не объект JSON

## Матрица бинарников

- Windows: `x64`, `arm64`
- macOS: `x64`, `arm64`
- Linux: `x64`, `arm64`, `arm`

Linux binaries собраны под `musl`, а не под `glibc`.

## Важные нюансы

- На `darwin` и `linux` wrapper пытается выставить права `755` через `chmodSync(...)`.
- Для неподдерживаемой платформы выбрасывается `Error("Unsupported platform: ...")`.
- Публикация пакета проверяет наличие всех target-бинарников через `node check.js`.
- Типы в `@types/ftaql-cli.d.ts` описывают и `FtaQlJsonOutput`, но сам `runFtaQl(...)` его не возвращает.
