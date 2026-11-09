use ftaql::analyze_project;
use ftaql::structs::FtaQlConfigResolved;
use insta;

#[test]
fn test_runtime_cycle_detection() {
    let project_path_str = "tests/fixtures/runtime_cycles_project";
    let fta_config = FtaQlConfigResolved {
        includes: vec!["**/*.ts".to_string()],
        excludes: vec![],
        score_cap: 100,
        include_comments: false,
        exclude_under: 0,
    };

    let analysis_result = analyze_project(&project_path_str.to_string(), Some(fta_config));

    insta::assert_json_snapshot!(analysis_result);
}
