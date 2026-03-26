use crate::structs::{CycleInfo, FtaQlConfigResolved, FtaQlJsonOutput};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, Transaction};
use std::collections::{BTreeSet, HashSet};

const SCHEMA_VERSION: i64 = 3;

fn sqlite_int(value: usize) -> i64 {
    value as i64
}

#[derive(Debug, Clone)]
pub struct PersistRunOptions<'a> {
    pub project_root: &'a str,
    pub revision: Option<&'a str>,
    pub ref_label: Option<&'a str>,
    pub elapsed_seconds: f64,
    pub config: &'a FtaQlConfigResolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistRunSummary {
    pub run_id: i64,
    pub file_count: usize,
    pub cycle_count: usize,
    pub runtime_cycle_count: usize,
}

pub fn persist_run(
    db_path: &str,
    analysis_output: &FtaQlJsonOutput,
    options: &PersistRunOptions<'_>,
) -> Result<PersistRunSummary> {
    let tracked_files = tracked_file_paths(analysis_output);
    let mut connection = Connection::open(db_path)
        .with_context(|| format!("failed to open sqlite database at {}", db_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    initialize_schema(&connection)?;

    let tx = connection.transaction()?;
    let run_id = insert_analysis_run(&tx, options)?;
    insert_files(&tx, run_id, analysis_output, &tracked_files)?;
    let cycle_count = insert_cycles(
        &tx,
        run_id,
        "all",
        &analysis_output.project_analysis.cycles,
        &analysis_output.project_analysis.cycle_members,
        &tracked_files,
    )?;
    let runtime_cycle_count = insert_cycles(
        &tx,
        run_id,
        "runtime",
        &analysis_output.project_analysis.runtime_cycles,
        &analysis_output.project_analysis.cycle_members,
        &tracked_files,
    )?;
    tx.commit()?;

    Ok(PersistRunSummary {
        run_id,
        file_count: analysis_output.findings.len(),
        cycle_count,
        runtime_cycle_count,
    })
}

fn tracked_file_paths(analysis_output: &FtaQlJsonOutput) -> HashSet<&str> {
    analysis_output
        .findings
        .iter()
        .map(|finding| finding.file_name.as_str())
        .collect()
}

fn initialize_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS analysis_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schema_version INTEGER NOT NULL,
            project_root TEXT NOT NULL,
            revision TEXT,
            ref_label TEXT,
            created_at TEXT NOT NULL,
            elapsed_seconds REAL NOT NULL,
            ftaql_version TEXT NOT NULL,
            config_json TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS files (
            run_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            line_count INTEGER NOT NULL,
            cyclomatic INTEGER NOT NULL,
            halstead_uniq_operators INTEGER NOT NULL,
            halstead_uniq_operands INTEGER NOT NULL,
            halstead_total_operators INTEGER NOT NULL,
            halstead_total_operands INTEGER NOT NULL,
            halstead_program_length INTEGER NOT NULL,
            halstead_vocabulary_size INTEGER NOT NULL,
            halstead_volume REAL NOT NULL,
            halstead_difficulty REAL NOT NULL,
            halstead_effort REAL NOT NULL,
            halstead_time REAL NOT NULL,
            halstead_bugs REAL NOT NULL,
            afferent_coupling INTEGER,
            efferent_coupling INTEGER,
            instability REAL,
            file_score REAL NOT NULL,
            coupling_score REAL NOT NULL,
            cycle_id INTEGER,
            runtime_cycle_id INTEGER,
            PRIMARY KEY (run_id, file_path),
            FOREIGN KEY (run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS file_dependencies (
            run_id INTEGER NOT NULL,
            from_file TEXT NOT NULL,
            to_file TEXT NOT NULL,
            strength INTEGER NOT NULL,
            PRIMARY KEY (run_id, from_file, to_file),
            FOREIGN KEY (run_id, from_file) REFERENCES files(run_id, file_path) ON DELETE CASCADE,
            FOREIGN KEY (run_id, to_file) REFERENCES files(run_id, file_path) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS cycles (
            run_id INTEGER NOT NULL,
            cycle_kind TEXT NOT NULL CHECK (cycle_kind IN ('all', 'runtime')),
            cycle_id INTEGER NOT NULL,
            size INTEGER NOT NULL,
            PRIMARY KEY (run_id, cycle_kind, cycle_id),
            FOREIGN KEY (run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS cycle_files (
            run_id INTEGER NOT NULL,
            cycle_kind TEXT NOT NULL CHECK (cycle_kind IN ('all', 'runtime')),
            cycle_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            PRIMARY KEY (run_id, cycle_kind, cycle_id, file_path),
            FOREIGN KEY (run_id, cycle_kind, cycle_id) REFERENCES cycles(run_id, cycle_kind, cycle_id) ON DELETE CASCADE,
            FOREIGN KEY (run_id, file_path) REFERENCES files(run_id, file_path) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS cycle_edges (
            run_id INTEGER NOT NULL,
            cycle_kind TEXT NOT NULL CHECK (cycle_kind IN ('all', 'runtime')),
            cycle_id INTEGER NOT NULL,
            from_file TEXT NOT NULL,
            to_file TEXT NOT NULL,
            strength INTEGER NOT NULL,
            PRIMARY KEY (run_id, cycle_kind, cycle_id, from_file, to_file),
            FOREIGN KEY (run_id, cycle_kind, cycle_id) REFERENCES cycles(run_id, cycle_kind, cycle_id) ON DELETE CASCADE,
            FOREIGN KEY (run_id, from_file) REFERENCES files(run_id, file_path) ON DELETE CASCADE,
            FOREIGN KEY (run_id, to_file) REFERENCES files(run_id, file_path) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_analysis_runs_revision ON analysis_runs(revision);
        CREATE INDEX IF NOT EXISTS idx_files_scores ON files(run_id, file_score DESC, coupling_score DESC);
        CREATE INDEX IF NOT EXISTS idx_file_dependencies_target ON file_dependencies(run_id, to_file);
        CREATE INDEX IF NOT EXISTS idx_cycle_files_path ON cycle_files(run_id, file_path);
        "#,
    )?;

    Ok(())
}

fn insert_analysis_run(tx: &Transaction<'_>, options: &PersistRunOptions<'_>) -> Result<i64> {
    let config_json = serde_json::to_string(options.config)?;
    tx.execute(
        r#"
        INSERT INTO analysis_runs (
            schema_version,
            project_root,
            revision,
            ref_label,
            created_at,
            elapsed_seconds,
            ftaql_version,
            config_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            SCHEMA_VERSION,
            options.project_root,
            options.revision,
            options.ref_label,
            Utc::now().to_rfc3339(),
            options.elapsed_seconds,
            env!("CARGO_PKG_VERSION"),
            config_json,
        ],
    )?;

    Ok(tx.last_insert_rowid())
}

fn insert_files(
    tx: &Transaction<'_>,
    run_id: i64,
    analysis_output: &FtaQlJsonOutput,
    tracked_files: &HashSet<&str>,
) -> Result<()> {
    let mut file_statement = tx.prepare(
        r#"
        INSERT INTO files (
            run_id,
            file_path,
            line_count,
            cyclomatic,
            halstead_uniq_operators,
            halstead_uniq_operands,
            halstead_total_operators,
            halstead_total_operands,
            halstead_program_length,
            halstead_vocabulary_size,
            halstead_volume,
            halstead_difficulty,
            halstead_effort,
            halstead_time,
            halstead_bugs,
            afferent_coupling,
            efferent_coupling,
            instability,
            file_score,
            coupling_score,
            cycle_id,
            runtime_cycle_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22
        )
        "#,
    )?;
    let mut dependency_statement = tx.prepare(
        r#"
        INSERT INTO file_dependencies (run_id, from_file, to_file, strength)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )?;

    for finding in &analysis_output.findings {
        let coupling = finding.coupling_metrics.as_ref();
        file_statement.execute(params![
            run_id,
            finding.file_name,
            sqlite_int(finding.size_metrics.line_count),
            sqlite_int(finding.complexity_metrics.cyclomatic),
            sqlite_int(finding.complexity_metrics.halstead.uniq_operators),
            sqlite_int(finding.complexity_metrics.halstead.uniq_operands),
            sqlite_int(finding.complexity_metrics.halstead.total_operators),
            sqlite_int(finding.complexity_metrics.halstead.total_operands),
            sqlite_int(finding.complexity_metrics.halstead.program_length),
            sqlite_int(finding.complexity_metrics.halstead.vocabulary_size),
            finding.complexity_metrics.halstead.volume,
            finding.complexity_metrics.halstead.difficulty,
            finding.complexity_metrics.halstead.effort,
            finding.complexity_metrics.halstead.time,
            finding.complexity_metrics.halstead.bugs,
            coupling.map(|metrics| sqlite_int(metrics.afferent_coupling)),
            coupling.map(|metrics| sqlite_int(metrics.efferent_coupling)),
            coupling.map(|metrics| metrics.instability),
            finding.scores.file_score,
            finding.scores.coupling_score,
            coupling.and_then(|metrics| {
                metrics
                    .cycles
                    .as_ref()
                    .and_then(|cycles| cycles.cycle_id)
                    .map(sqlite_int)
            }),
            coupling.and_then(|metrics| {
                metrics
                    .cycles
                    .as_ref()
                    .and_then(|cycles| cycles.runtime_cycle_id)
                    .map(sqlite_int)
            }),
        ])?;
    }

    for finding in &analysis_output.findings {
        if let Some(coupling_metrics) = finding.coupling_metrics.as_ref() {
            for (dependency_path, strength) in &coupling_metrics.dependency_strength {
                if !tracked_files.contains(finding.file_name.as_str())
                    || !tracked_files.contains(dependency_path.as_str())
                {
                    continue;
                }
                dependency_statement.execute(params![
                    run_id,
                    finding.file_name,
                    dependency_path,
                    sqlite_int(*strength),
                ])?;
            }
        }
    }

    Ok(())
}

fn insert_cycles(
    tx: &Transaction<'_>,
    run_id: i64,
    cycle_kind: &str,
    cycles: &[CycleInfo],
    cycle_members: &[String],
    tracked_files: &HashSet<&str>,
) -> Result<usize> {
    let mut cycle_statement = tx.prepare(
        r#"
        INSERT INTO cycles (run_id, cycle_kind, cycle_id, size)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )?;
    let mut cycle_file_statement = tx.prepare(
        r#"
        INSERT INTO cycle_files (run_id, cycle_kind, cycle_id, file_path)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )?;
    let mut cycle_edge_statement = tx.prepare(
        r#"
        INSERT INTO cycle_edges (run_id, cycle_kind, cycle_id, from_file, to_file, strength)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )?;

    let mut inserted_cycles = 0;
    for cycle in cycles {
        let mut files_in_cycle = BTreeSet::new();
        let mut cycle_edges = Vec::new();
        for (&from_idx, dependencies) in &cycle.graph {
            let from_file = resolve_cycle_member(cycle_members, from_idx, cycle_kind, cycle.id)?;
            if !tracked_files.contains(from_file) {
                continue;
            }
            files_in_cycle.insert(from_file.to_string());

            for (&to_idx, &strength) in dependencies {
                let to_file = resolve_cycle_member(cycle_members, to_idx, cycle_kind, cycle.id)?;
                if !tracked_files.contains(to_file) {
                    continue;
                }
                files_in_cycle.insert(to_file.to_string());
                cycle_edges.push((from_file.to_string(), to_file.to_string(), strength));
            }
        }

        if files_in_cycle.len() < 2 || cycle_edges.is_empty() {
            continue;
        }

        cycle_statement.execute(params![
            run_id,
            cycle_kind,
            sqlite_int(cycle.id),
            sqlite_int(cycle.size)
        ])?;
        inserted_cycles += 1;

        for (from_file, to_file, strength) in cycle_edges {
            cycle_edge_statement.execute(params![
                run_id,
                cycle_kind,
                sqlite_int(cycle.id),
                from_file,
                to_file,
                sqlite_int(strength),
            ])?;
        }

        for file_path in files_in_cycle {
            cycle_file_statement.execute(params![
                run_id,
                cycle_kind,
                sqlite_int(cycle.id),
                file_path
            ])?;
        }
    }

    Ok(inserted_cycles)
}

fn resolve_cycle_member<'a>(
    cycle_members: &'a [String],
    member_idx: usize,
    cycle_kind: &str,
    cycle_id: usize,
) -> Result<&'a str> {
    cycle_members
        .get(member_idx)
        .map(|path| path.as_str())
        .with_context(|| {
            format!(
                "cycle {}:{} references missing member index {}",
                cycle_kind, cycle_id, member_idx
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{persist_run, PersistRunOptions, SCHEMA_VERSION};
    use crate::structs::{
        ComplexityMetrics, CouplingMetrics, CycleData, CycleInfo, FileData, FtaQlConfigResolved,
        FtaQlJsonOutput, HalsteadMetrics, ProjectAnalysis, Scores, SizeMetrics,
    };
    use rusqlite::{params, Connection};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn persists_normalized_project_snapshot() {
        let db_file = NamedTempFile::new().unwrap();
        let analysis_output = sample_analysis_output();
        let config = sample_config();

        let summary = persist_run(
            db_file.path().to_str().unwrap(),
            &analysis_output,
            &PersistRunOptions {
                project_root: "/tmp/project",
                revision: Some("abc123"),
                ref_label: Some("main"),
                elapsed_seconds: 0.42,
                config: &config,
            },
        )
        .unwrap();

        assert_eq!(summary.file_count, 2);
        assert_eq!(summary.cycle_count, 1);
        assert_eq!(summary.runtime_cycle_count, 1);

        let connection = Connection::open(db_file.path()).unwrap();
        let run_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM analysis_runs", [], |row| row.get(0))
            .unwrap();
        let file_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        let dependency_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM file_dependencies", [], |row| {
                row.get(0)
            })
            .unwrap();
        let cycle_file_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cycle_files", [], |row| row.get(0))
            .unwrap();
        let cycle_edge_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cycle_edges", [], |row| row.get(0))
            .unwrap();
        let revision: String = connection
            .query_row("SELECT revision FROM analysis_runs", [], |row| row.get(0))
            .unwrap();
        let schema_version: i64 = connection
            .query_row("SELECT schema_version FROM analysis_runs", [], |row| {
                row.get(0)
            })
            .unwrap();
        let config_json: String = connection
            .query_row("SELECT config_json FROM analysis_runs", [], |row| {
                row.get(0)
            })
            .unwrap();
        let edge_strength: i64 = connection
            .query_row(
                "SELECT strength FROM cycle_edges WHERE from_file = ?1 AND to_file = ?2",
                params!["src/a.ts", "src/b.ts"],
                |row| row.get(0),
            )
            .unwrap();
        let columns = connection
            .prepare("PRAGMA table_info(files)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();

        assert_eq!(run_count, 1);
        assert_eq!(file_count, 2);
        assert_eq!(dependency_count, 2);
        assert_eq!(cycle_file_count, 4);
        assert_eq!(cycle_edge_count, 4);
        assert_eq!(revision, "abc123");
        assert_eq!(schema_version, SCHEMA_VERSION);
        assert!(config_json.contains("\"exclude_under\":0"));
        assert_eq!(edge_strength, 2);
        assert!(!columns.iter().any(|column| column == "assessment"));
    }

    #[test]
    fn appends_multiple_runs_to_same_database() {
        let db_file = NamedTempFile::new().unwrap();
        let analysis_output = sample_analysis_output();
        let config = sample_config();

        let first = persist_run(
            db_file.path().to_str().unwrap(),
            &analysis_output,
            &PersistRunOptions {
                project_root: "/tmp/project",
                revision: Some("rev-1"),
                ref_label: None,
                elapsed_seconds: 0.1,
                config: &config,
            },
        )
        .unwrap();
        let second = persist_run(
            db_file.path().to_str().unwrap(),
            &analysis_output,
            &PersistRunOptions {
                project_root: "/tmp/project",
                revision: Some("rev-2"),
                ref_label: None,
                elapsed_seconds: 0.2,
                config: &config,
            },
        )
        .unwrap();

        let connection = Connection::open(db_file.path()).unwrap();
        let run_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM analysis_runs", [], |row| row.get(0))
            .unwrap();

        assert!(second.run_id > first.run_id);
        assert_eq!(run_count, 2);
    }

    #[test]
    fn skips_foreign_key_violations_for_missing_findings() {
        let db_file = NamedTempFile::new().unwrap();
        let analysis_output = mismatched_analysis_output();
        let config = sample_config();

        let summary = persist_run(
            db_file.path().to_str().unwrap(),
            &analysis_output,
            &PersistRunOptions {
                project_root: "/tmp/project",
                revision: Some("rev-mismatch"),
                ref_label: None,
                elapsed_seconds: 0.1,
                config: &config,
            },
        )
        .unwrap();

        let connection = Connection::open(db_file.path()).unwrap();
        let file_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        let dependency_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM file_dependencies", [], |row| {
                row.get(0)
            })
            .unwrap();
        let cycle_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cycles", [], |row| row.get(0))
            .unwrap();
        let cycle_file_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cycle_files", [], |row| row.get(0))
            .unwrap();
        let cycle_edge_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cycle_edges", [], |row| row.get(0))
            .unwrap();

        assert_eq!(summary.file_count, 1);
        assert_eq!(summary.cycle_count, 0);
        assert_eq!(summary.runtime_cycle_count, 0);
        assert_eq!(file_count, 1);
        assert_eq!(dependency_count, 0);
        assert_eq!(cycle_count, 0);
        assert_eq!(cycle_file_count, 0);
        assert_eq!(cycle_edge_count, 0);
    }

    fn sample_analysis_output() -> FtaQlJsonOutput {
        let mut a_dependencies = HashMap::new();
        a_dependencies.insert("src/b.ts".to_string(), 2);

        let mut b_dependencies = HashMap::new();
        b_dependencies.insert("src/a.ts".to_string(), 1);

        let cycle_graph =
            HashMap::from([(0, HashMap::from([(1, 2)])), (1, HashMap::from([(0, 1)]))]);

        FtaQlJsonOutput {
            project_analysis: ProjectAnalysis {
                cycle_members: vec!["src/a.ts".to_string(), "src/b.ts".to_string()],
                cycles: vec![CycleInfo {
                    id: 7,
                    size: 2,
                    graph: cycle_graph.clone(),
                }],
                runtime_cycles: vec![CycleInfo {
                    id: 9,
                    size: 2,
                    graph: cycle_graph,
                }],
            },
            findings: vec![
                sample_file(
                    "src/a.ts",
                    12.5,
                    10.0,
                    Some(CouplingMetrics {
                        afferent_coupling: 1,
                        efferent_coupling: 1,
                        instability: 0.5,
                        dependency_strength: a_dependencies,
                        cycles: Some(CycleData {
                            cycle_id: Some(7),
                            runtime_cycle_id: Some(9),
                        }),
                    }),
                ),
                sample_file(
                    "src/b.ts",
                    9.0,
                    12.0,
                    Some(CouplingMetrics {
                        afferent_coupling: 1,
                        efferent_coupling: 1,
                        instability: 0.5,
                        dependency_strength: b_dependencies,
                        cycles: Some(CycleData {
                            cycle_id: Some(7),
                            runtime_cycle_id: Some(9),
                        }),
                    }),
                ),
            ],
        }
    }

    fn mismatched_analysis_output() -> FtaQlJsonOutput {
        let mut a_dependencies = HashMap::new();
        a_dependencies.insert("src/b.ts".to_string(), 2);

        let cycle_graph =
            HashMap::from([(0, HashMap::from([(1, 2)])), (1, HashMap::from([(0, 1)]))]);

        FtaQlJsonOutput {
            project_analysis: ProjectAnalysis {
                cycle_members: vec!["src/a.ts".to_string(), "src/b.ts".to_string()],
                cycles: vec![CycleInfo {
                    id: 7,
                    size: 2,
                    graph: cycle_graph.clone(),
                }],
                runtime_cycles: vec![CycleInfo {
                    id: 9,
                    size: 2,
                    graph: cycle_graph,
                }],
            },
            findings: vec![sample_file(
                "src/a.ts",
                12.5,
                10.0,
                Some(CouplingMetrics {
                    afferent_coupling: 0,
                    efferent_coupling: 1,
                    instability: 1.0,
                    dependency_strength: a_dependencies,
                    cycles: Some(CycleData {
                        cycle_id: Some(7),
                        runtime_cycle_id: Some(9),
                    }),
                }),
            )],
        }
    }

    fn sample_file(
        file_name: &str,
        file_score: f64,
        coupling_score: f64,
        coupling_metrics: Option<CouplingMetrics>,
    ) -> FileData {
        FileData {
            file_name: file_name.to_string(),
            size_metrics: SizeMetrics { line_count: 42 },
            complexity_metrics: ComplexityMetrics {
                cyclomatic: 3,
                halstead: HalsteadMetrics {
                    uniq_operators: 7,
                    uniq_operands: 11,
                    total_operators: 17,
                    total_operands: 19,
                    program_length: 36,
                    vocabulary_size: 18,
                    volume: 150.0,
                    difficulty: 5.0,
                    effort: 750.0,
                    time: 41.6,
                    bugs: 0.05,
                },
            },
            coupling_metrics,
            scores: Scores {
                file_score: file_score,
                coupling_score: coupling_score,
            },
        }
    }

    fn sample_config() -> FtaQlConfigResolved {
        FtaQlConfigResolved {
            extensions: vec![".ts".to_string()],
            exclude_filenames: vec![],
            exclude_directories: vec![],
            score_cap: 1000,
            include_comments: false,
            exclude_under: 0,
        }
    }
}
