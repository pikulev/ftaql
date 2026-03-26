# Скоринг

## Что дают `file_score` и `coupling_score`

FtaQl считает две сводные оценки:

- `file_score` показывает, насколько "тяжёлым" выглядит файл сам по себе
- `coupling_score` показывает, насколько рискованными выглядят связи файла в контексте всего проекта

Обе оценки лучше читать как относительные индексы внутри одного репозитория и по истории ревизий, а не как универсальную внешнюю шкалу качества.

## Формула `file_score`

В текущем коде `file_score` считается по формуле `FileScoreFormula::Original`.

На вход идут:

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
- значение удобно использовать для сортировки hotspots
- `score_cap` в конфиге сравнивается именно с `file_score`

Важно:

- `include_comments` меняет `line_count`, а значит и `file_score`
- это эвристика, а не "объективная сложность файла" в абсолютном смысле

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

- чем выше `coupling_score`, тем больше архитектурное напряжение вокруг файла
- значение особенно полезно для сравнения файлов внутри одного проекта и по истории
- это не нормализованная шкала `0..100`: в текущем коде возвращается сырой итоговый score

Важно:

- в формулу входит статический `cycle_id`, а не `runtime_cycle_id`
- `runtime`-циклы попадают в отчёт проекта, но не участвуют напрямую в текущем `coupling_score`
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

Для пользователя это полезно понимать так:

- Halstead помогает оценить объём и когнитивную насыщенность кода
- в `file_score` не подставляется вся структура целиком
- в текущей формуле `file_score` используются `program_length`, `vocabulary_size` и `difficulty`

## Как интерпретировать значения

Для первого просмотра обычно хватает трёх шагов:

1. Отсортировать файлы по `file_score`
2. Отдельно посмотреть сортировку по `coupling_score`
3. Проверить, где пересекаются высокие scores и циклы

Эмпирические ранги можно строить обычным SQL поверх `files`. Это не встроенная шкала FtaQl, а локальная интерпретация команды.

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

Пороги в этом примере нужно калибровать под свой проект.

## Caveats

- `file_score` и `coupling_score` лучше использовать для сравнения внутри проекта и по времени, а не как универсальную внешнюю оценку.
- `score_cap` работает только для `file_score`.
- В WASM `coupling_score` всегда `0.0`, потому что там нет project-level dependency graph и cycle analysis.
- `type-only` зависимости могут образовывать цикл полного графа, который влияет на архитектуру и tooling, но не обязательно на runtime.
- Файл `crates/COUPLING_SCORE.md` полезен как инженерный черновик, но не полностью совпадает с текущей реализацией и не должен считаться пользовательской спецификацией без оговорок.

## Связанные разделы

- [`../configuration/ru.md`](../configuration/ru.md)
- [`../sqlite-output/ru.md`](../sqlite-output/ru.md)
- [`../rust-core/ru.md`](../rust-core/ru.md)
- [`../wasm/ru.md`](../wasm/ru.md)
