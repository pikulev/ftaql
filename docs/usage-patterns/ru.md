# Паттерны использования

## Когда использовать CLI

Используйте native CLI или `@piklv/ftaql-cli`, если вам нужно:

- проанализировать весь проект
- сохранить snapshot в SQLite
- запускать анализ в CI/CD
- сравнивать ревизии через SQL

См. также [`../node-cli/ru.md`](../node-cli/ru.md) и [`../sqlite-output/ru.md`](../sqlite-output/ru.md).

## Когда использовать Rust API

Используйте `ftaql` crate, если вы:

- пишете Rust integration
- хотите получить `FtaQlJsonOutput` в памяти
- уже работаете с AST и хотите вызвать `analyze_module(...)`

См. [`../rust-core/ru.md`](../rust-core/ru.md).

## Когда использовать WASM

Используйте `@piklv/ftaql-wasm`, если вы:

- анализируете одиночный snippet в браузере
- строите playground или editor integration
- не имеете доступа к файловой системе

См. [`../wasm/ru.md`](../wasm/ru.md).
