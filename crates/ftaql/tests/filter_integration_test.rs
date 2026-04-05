use ftaql::analyze_project;
use ftaql::structs::FtaQlConfigResolved;
use std::collections::HashSet;

fn file_names(config: FtaQlConfigResolved) -> HashSet<String> {
    let project_path = "tests/fixtures/filter_project";
    let result = analyze_project(&project_path.to_string(), Some(config));
    result
        .findings
        .into_iter()
        .map(|f| f.file_name)
        .collect()
}

#[test]
fn default_excludes_filter_dts_and_dist() {
    let config = FtaQlConfigResolved {
        includes: vec!["**/*.ts".to_string(), "**/*.js".to_string()],
        excludes: vec!["**/*.d.ts".to_string(), "dist/**".to_string()],
        score_cap: 10000,
        include_comments: false,
        exclude_under: 0,
    };

    let names = file_names(config);

    // src/index.ts and src/helper.ts should be included
    assert!(
        names.iter().any(|n| n.contains("src/index.ts")),
        "expected src/index.ts in results, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("src/helper.ts")),
        "expected src/helper.ts in results, got: {:?}",
        names
    );
    // .d.ts should be excluded
    assert!(
        !names.iter().any(|n| n.contains("types.d.ts")),
        "types.d.ts should be excluded, got: {:?}",
        names
    );
    // dist/ should be excluded
    assert!(
        !names.iter().any(|n| n.contains("dist/")),
        "dist/ files should be excluded, got: {:?}",
        names
    );
}

#[test]
fn excludes_directory_by_glob() {
    let config = FtaQlConfigResolved {
        includes: vec!["**/*.ts".to_string()],
        excludes: vec!["__tests__/**".to_string()],
        score_cap: 10000,
        include_comments: false,
        exclude_under: 0,
    };

    let names = file_names(config);

    // __tests__/ should be excluded
    assert!(
        !names.iter().any(|n| n.contains("__tests__/")),
        "__tests__/ files should be excluded, got: {:?}",
        names
    );
    // src files should still be present
    assert!(
        names.iter().any(|n| n.contains("src/index.ts")),
        "expected src/index.ts in results, got: {:?}",
        names
    );
}

#[test]
fn includes_limits_to_matching_files_only() {
    // Only include .js files — no .ts files should appear
    let config = FtaQlConfigResolved {
        includes: vec!["**/*.js".to_string()],
        excludes: vec![],
        score_cap: 10000,
        include_comments: false,
        exclude_under: 0,
    };

    let names = file_names(config);

    assert!(
        !names.iter().any(|n| n.ends_with(".ts")),
        "no .ts files should be included, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("dist/index.js")),
        "expected dist/index.js in results, got: {:?}",
        names
    );
}

#[test]
fn empty_excludes_includes_everything() {
    let config = FtaQlConfigResolved {
        includes: vec!["**/*.ts".to_string()],
        excludes: vec![],
        score_cap: 10000,
        include_comments: false,
        exclude_under: 0,
    };

    let names = file_names(config);

    // All .ts files should be present, including .d.ts and test files
    assert!(
        names.iter().any(|n| n.contains("types.d.ts")),
        "expected types.d.ts with empty excludes, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.contains("index.test.ts")),
        "expected index.test.ts with empty excludes, got: {:?}",
        names
    );
}
