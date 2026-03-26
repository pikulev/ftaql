use ftaql::analyze_project;
use ftaql::structs::FtaQlConfigResolved;
use insta;
use std::fs;
use std::path::Path;

#[test]
fn coupling_analysis_on_sample_project() {
    let project_path = "tests/fixtures/sample_project";
    let snapshot_path = Path::new("fixtures").join("snapshots");
    let fta_config = FtaQlConfigResolved {
        extensions: vec![".ts".to_string(), ".tsx".to_string()],
        exclude_filenames: vec![],
        exclude_directories: vec![],
        score_cap: 100,
        include_comments: false,
        exclude_under: 0,
    };
    let absolute_path = fs::canonicalize(project_path).expect("Failed to canonicalize path");
    let repo_path = absolute_path.to_str().unwrap().to_string();
    let analysis_result = analyze_project(&repo_path, Some(fta_config));

    for finding in analysis_result.findings {
        let mut settings = insta::Settings::new();
        settings.set_snapshot_path(&snapshot_path);
        settings.set_snapshot_suffix(finding.file_name.replace("/", "@"));
        settings.bind(|| {
            insta::assert_json_snapshot!(finding.file_name.clone(), finding);
        });
    }
}

#[test]
fn tsconfig_paths_resolution() {
    let project_path = "tests/fixtures/tsconfig_project";
    let fta_config = FtaQlConfigResolved {
        extensions: vec![".ts".to_string(), ".tsx".to_string()],
        exclude_filenames: vec![],
        exclude_directories: vec![],
        score_cap: 100,
        include_comments: false,
        exclude_under: 0,
    };
    let absolute_path = fs::canonicalize(project_path).expect("Failed to canonicalize path");
    let repo_path = absolute_path.to_str().unwrap().to_string();
    let analysis_result = analyze_project(&repo_path, Some(fta_config));
    insta::assert_json_snapshot!(analysis_result);
}

#[test]
fn ordered_cycle_reporting() {
    let project_path = "tests/fixtures/large_cycle_project";
    let fta_config = FtaQlConfigResolved {
        extensions: vec![".js".to_string()],
        exclude_filenames: vec![],
        exclude_directories: vec![],
        score_cap: 100,
        include_comments: false,
        exclude_under: 0,
    };
    let absolute_path = fs::canonicalize(project_path).expect("Failed to canonicalize path");
    let repo_path = absolute_path.to_str().unwrap().to_string();
    let analysis_result = analyze_project(&repo_path, Some(fta_config.clone()));
    insta::assert_json_snapshot!(analysis_result.project_analysis);
}
