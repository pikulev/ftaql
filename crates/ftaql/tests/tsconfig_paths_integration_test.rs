#![cfg(test)]

use ftaql::{analyze_project, config::get_default_config};
use insta::assert_debug_snapshot;
use std::fs;

#[test]
fn test_tsconfig_paths_resolution() {
    let project_path_str = "tests/fixtures/tsconfig_project";
    let _ = env_logger::builder().is_test(true).try_init();
    let absolute_path = fs::canonicalize(project_path_str).expect("Failed to canonicalize path");
    let mut analysis_result = analyze_project(
        &absolute_path.to_str().unwrap().to_string(),
        Some(get_default_config()),
    );

    let app_main_analysis = analysis_result
        .findings
        .iter()
        .find(|f| f.file_name == "packages/app/src/main.ts")
        .expect("Analysis for app/src/main.ts not found");

    let coupling_metrics = app_main_analysis
        .coupling_metrics
        .as_ref()
        .expect("Coupling metrics not found");

    assert_eq!(
        coupling_metrics.efferent_coupling, 4,
        "Should have four efferent dependencies"
    );

    let mut actual_deps: Vec<_> = coupling_metrics
        .dependency_strength
        .keys()
        .cloned()
        .collect();
    actual_deps.sort();

    let mut expected_deps = vec![
        "packages/app/src/utils.ts",
        "packages/lib/src/fallback/one.ts",
        "packages/lib/src/helpers.ts",
        "packages/lib/src/utils.ts",
    ];
    expected_deps.sort();

    assert_eq!(actual_deps, expected_deps);

    insta::assert_json_snapshot!(analysis_result.findings);
}
