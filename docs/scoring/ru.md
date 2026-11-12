# Скоринг

## Что дают `file_score` и `coupling_score`

FtaQl считает две сводные оценки:

- `file_score` оценивает сложность файла по его собственным метрикам;
- `coupling_score` оценивает риск, связанный с зависимостями файла в проекте.

Эти оценки помогают сравнивать файлы внутри одного репозитория и отслеживать изменения между ревизиями. Для сравнения качества разных проектов общей шкалы у FtaQl нет.

## Формула `file_score`

В текущем коде `file_score` считается по формуле `FileScoreFormula::Original`.

Формула использует следующие метрики:

- `line_count`
- `cyclomatic`
- `halstead.program_length`
- `halstead.vocabulary_size`
- `halstead.difficulty`

Полная структура `HalsteadMetrics` шире, но в саму формулу `file_score` входят именно эти поля.

В упрощённом виде расчёт выглядит так:

```text
factor =
  1, если ln(cyclomatic) < 1
  иначе sqrt(line_count * difficulty) / sqrt(sqrt(ln(cyclomatic) * ln(difficulty)) * ln(program_length))

absolute_file_score =
  171 - 5.2 * ln(vocabulary_size) - 0.23 * cyclomatic - 16.2 * ln(factor)

file_score =
  max(0, 100 - absolute_file_score * 100 / 171)
```

Практический смысл:

- чем выше `file_score`, тем файл обычно сложнее поддерживать
- сортировка по этой оценке помогает выбрать файлы для дальнейшего изучения
- `score_cap` в конфиге сравнивается именно с `file_score`

Важно:

- `include_comments` меняет `line_count`, а значит и `file_score`
- оценка основана на эвристике и требует интерпретации с учётом кода

## Формула `coupling_score`

В текущем коде `coupling_score` считается по формуле `CouplingScoreFormula::Original`.

На вход идут:

- `Ca` = `afferent_coupling`
- `Ce` = `efferent_coupling`
- `instability`
- `dependency_strength`
- участие в статическом цикле через `cycle_id`
- дополнительные штрафы за нестабильность, "плохое ядро" и "лишнюю абстракцию"

В упрощённом виде расчёт выглядит так:

```text
если Ca + Ce == 0:
  coupling_score = 0

Ds = сумма dependency_strength
S_bar = Ds / max(1, Ce)
cycle_factor = 1 + размер статического цикла, если cycle_id есть, иначе 1

coupling_score =
  (Ca + Ce)
  + 12 * ln((S_bar / 10) + 1)
  + 90 * ln(cycle_factor)
  + penalty_instability(instability)
  + 50 * key_risk(Ca, instability)
  + 50 * orphan_risk(Ca, instability)
```

Практический смысл:

- высокое значение `coupling_score` указывает на связи файла, которые стоит изучить
- значение особенно полезно для сравнения файлов внутри одного проекта и по истории
- результат не нормализуется к шкале `0..100`: функция возвращает итоговую сумму по формуле

Важно:

- в формулу входит статический `cycle_id`, а не `runtime_cycle_id`
- циклы зависимостей времени исполнения (`runtime`-циклы) попадают в отчёт проекта, но не участвуют напрямую в текущем `coupling_score`
- если у файла нет связей, `coupling_score` сразу равен `0.0`

## Что такое Halstead в контексте FtaQl

Метрики Холстеда в FtaQl считаются по операторам и операндам в AST. В структуре есть:

- уникальные и общие операторы
- уникальные и общие операнды
- `program_length`
- `vocabulary_size`
- `volume`
- `difficulty`
- `effort`
- `time`
- `bugs`

Метрики Холстеда помогают оценить объём кода и сложность его понимания. Из перечисленных полей текущая формула `file_score` использует три: `program_length`, `vocabulary_size` и `difficulty`.

## Как интерпретировать значения

Для первого просмотра обычно хватает трёх шагов:

1. Отсортируйте файлы по `file_score`, чтобы найти файлы с высокой оценкой собственной сложности.
2. Посмотрите сортировку по `coupling_score`, чтобы оценить связи между файлами.
3. Проверьте, участвуют ли файлы с высокими оценками в циклах зависимостей.

Если команде нужны категории вместо чисел, задайте пороги через SQL для таблицы `files`. В следующем примере пороги условные: их нужно подобрать под свой проект, поскольку встроенной шкалы категорий у FtaQl нет.

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

## Ограничения интерпретации

- `file_score` и `coupling_score` лучше использовать для сравнения внутри проекта и по времени, а не как универсальную внешнюю оценку.
- `score_cap` работает только для `file_score`.
- В WASM `coupling_score` всегда равен `0.0`, потому что анализ одного файла не строит граф зависимостей проекта и не ищет циклы.
- Зависимости только по типам могут образовывать цикл полного графа. Такой цикл влияет на архитектуру и инструменты разработки, хотя может не участвовать в исполнении кода.

## Связанные разделы

- [Конфигурация](../configuration/ru.md)
- [Результаты в SQLite](../sqlite-output/ru.md)
- [Rust API и нативный CLI](../rust-core/ru.md)
- [WebAssembly API](../wasm/ru.md)
