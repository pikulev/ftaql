use assert_cmd::Command;
use ftaql::analyze_project;
use ftaql::config::get_default_config;
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

#[test]
fn cli_persists_project_snapshot_to_sqlite() {
    let project_path = fs::canonicalize("tests/fixtures/sample_project").unwrap();
    let project_path_str = project_path.to_str().unwrap().to_string();
    let analysis = analyze_project(&project_path_str, Some(get_default_config()));
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("ftaql.sqlite");

    let output = Command::cargo_bin("ftaql")
        .unwrap()
        .arg(&project_path_str)
        .arg("--db")
        .arg(&db_path)
        .arg("--revision")
        .arg("sample-rev")
        .arg("--ref")
        .arg("fixtures")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("Persisted analysis run"));

    let connection = Connection::open(&db_path).unwrap();
    let run_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM analysis_runs", [], |row| row.get(0))
        .unwrap();
    let file_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
        .unwrap();
    let cycle_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cycles WHERE cycle_kind = 'all'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let runtime_cycle_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cycles WHERE cycle_kind = 'runtime'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let dependency_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM file_dependencies", [], |row| {
            row.get(0)
        })
        .unwrap();
    let revision: String = connection
        .query_row("SELECT revision FROM analysis_runs", [], |row| row.get(0))
        .unwrap();
    let ref_label: String = connection
        .query_row("SELECT ref_label FROM analysis_runs", [], |row| row.get(0))
        .unwrap();
    let schema_version: i64 = connection
        .query_row("SELECT schema_version FROM analysis_runs", [], |row| {
            row.get(0)
        })
        .unwrap();
    let columns = connection
        .prepare("PRAGMA table_info(files)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();

    let expected_dependencies: usize = analysis
        .findings
        .iter()
        .map(|finding| {
            finding
                .coupling_metrics
                .as_ref()
                .map(|metrics| metrics.dependency_strength.len())
                .unwrap_or(0)
        })
        .sum();

    assert_eq!(run_count, 1);
    assert_eq!(file_count as usize, analysis.findings.len());
    assert_eq!(cycle_count as usize, analysis.project_analysis.cycles.len());
    assert_eq!(
        runtime_cycle_count as usize,
        analysis.project_analysis.runtime_cycles.len()
    );
    assert_eq!(dependency_count as usize, expected_dependencies);
    assert_eq!(revision, "sample-rev");
    assert_eq!(ref_label, "fixtures");
    assert_eq!(schema_version, 3);
    assert!(!columns.iter().any(|column| column == "assessment"));
}

#[test]
fn cli_skips_dependencies_to_unparsed_files_when_persisting_sqlite() {
    let project_path = fs::canonicalize("tests/fixtures/sqlite_invalid_import_project").unwrap();
    let project_path_str = project_path.to_str().unwrap().to_string();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("ftaql.sqlite");

    let output = Command::cargo_bin("ftaql")
        .unwrap()
        .arg(&project_path_str)
        .arg("--db")
        .arg(&db_path)
        .arg("--revision")
        .arg("broken-import")
        .arg("--ref")
        .arg("fixtures")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("Persisted analysis run"));

    let connection = Connection::open(&db_path).unwrap();
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
    let cycle_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM cycles", [], |row| row.get(0))
        .unwrap();
    let parsed_file: String = connection
        .query_row("SELECT file_path FROM files", [], |row| row.get(0))
        .unwrap();

    assert_eq!(run_count, 1);
    assert_eq!(file_count, 1);
    assert_eq!(dependency_count, 0);
    assert_eq!(cycle_count, 0);
    assert_eq!(parsed_file, "a.ts");
}
