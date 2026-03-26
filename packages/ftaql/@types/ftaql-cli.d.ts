declare module "@piklv/ftaql-cli" {
  /**
   * Полная структура метрик и анализа для одного исходного файла (1:1 с Rust FileData).
   */
  export type AnalyzedFile = {
    /**
     * Полный путь или имя исходного файла.
     */
    file_name: string;
    /**
     * Метрики размера файла.
     */
    size_metrics: {
      /** Количество строк кода в файле (без пустых строк и, опционально, комментариев). */
      line_count: number;
    };
    /**
     * Метрики сложности файла.
     */
    complexity_metrics: {
      /** Цикломатическая сложность (количество независимых путей исполнения). */
      cyclomatic: number;
      /** Метрики Холстеда — количественная оценка сложности кода. */
      halstead: {
        uniq_operators: number;
        uniq_operands: number;
        total_operators: number;
        total_operands: number;
        program_length: number;
        vocabulary_size: number;
        volume: number;
        difficulty: number;
        effort: number;
        time: number;
        bugs: number;
      };
    };
    /**
     * Метрики связанности (coupling) для файла. Может отсутствовать или быть null, если не применимо.
     */
    coupling_metrics?: {
      /** Входящая связанность (afferent coupling, Ca): сколько других файлов зависят от этого файла. */
      afferent_coupling: number;
      /** Исходящая связанность (efferent coupling, Ce): от скольких файлов зависит этот файл. */
      efferent_coupling: number;
      /** Instability: Ce / (Ca + Ce). Чем ближе к 1, тем менее устойчив модуль. */
      instability: number;
      /** Сила зависимости: карта "путь к файлу" → "количество импортируемых идентификаторов". */
      dependency_strength: Record<string, number>;
      /** Информация о циклах в графе зависимостей. */
      cycles?: {
        /** ID цикла, в котором участвует файл. Ссылается на `project_analysis.cycles`. */
        cycle_id?: number;
        /** ID runtime-цикла, если цикл существует во время выполнения. Ссылается на `project_analysis.runtime_cycles`. */
        runtime_cycle_id?: number;
      };
    } | null;
    /**
     * Итоговые оценки файла.
     */
    scores: {
      /** Итоговый File Score (чем ниже, тем лучше). */
      file_score: number;
      /** Итоговый Coupling Score (чем ниже, тем лучше). */
      coupling_score: number;
    };
  };

  /**
   * Информация о циклических зависимостях (сильно связанных компонентах) в проекте.
   */
  export type CycleInfo = {
    /** Уникальный идентификатор цикла. */
    id: number;
    /** Количество файлов в цикле. */
    size: number;
    /**
     * Граф цикла, где ключи верхнего и вложенного уровней - это индексы из
     * `project_analysis.cycle_members`, а значения - вес связи между файлами.
     */
    graph: Record<string, Record<string, number>>;
  };

  /**
   * Глобальные метрики и анализ для всего проекта.
   */
  export type ProjectAnalysis = {
    /** Общий отсортированный список файлов, участвующих хотя бы в одном цикле. */
    cycle_members: string[];
    /** Список всех обнаруженных в проекте циклических зависимостей. */
    cycles: CycleInfo[];
    /** Список циклов, которые сохраняются во время выполнения. */
    runtime_cycles: CycleInfo[];
  };

  /**
   * Корневой объект, возвращаемый FtaQl при анализе в формате JSON.
   */
  export type FtaQlJsonOutput = {
    /** Глобальный анализ проекта. */
    project_analysis: ProjectAnalysis;
    /** Список проанализированных файлов. */
    findings: AnalyzedFile[];
  };

  /**
   * Options for persisting a project analysis snapshot into SQLite.
   *
   * `dbPath` is required. The remaining fields are stored as metadata on the analysis run.
   */
  export type FtaQlRunOptions = {
    /**
     * Path to the SQLite database file that should receive the snapshot.
     */
    dbPath: string;
    /**
     * Optional custom path to `ftaql.json`.
     */
    configPath?: string;
    /**
     * Optional revision identifier, for example a git commit SHA.
     */
    revision?: string;
    /**
     * Optional human-readable label such as branch or tag.
     */
    ref?: string;
  };

  /**
   * Runs the project analysis for the given project and persists the result into SQLite.
   *
   * @param projectPath - The path to the root of the project to analyze
   * @param options - SQLite persistence options. `dbPath` is required.
   */
  export function runFtaQl(
    projectPath: string,
    options: FtaQlRunOptions
  ): string;
}
