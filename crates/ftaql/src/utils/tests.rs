use super::{calculate_file_score, FileScoreFormula, ModuleScoreInput};
use crate::structs::HalsteadMetrics;
use crate::utils::{get_file_extension, is_code_file, is_test_file};

fn dummy_halstead(vocab: usize) -> HalsteadMetrics {
    HalsteadMetrics {
        vocabulary_size: vocab,
        program_length: 100,
        difficulty: 2.0,
        ..Default::default()
    }
}

#[test]
fn test_get_file_extension() {
    assert_eq!(get_file_extension("test.ts"), Some("ts".to_string()));
    assert_eq!(get_file_extension("README"), None);
    assert_eq!(get_file_extension("archive.tar.gz"), Some("gz".to_string()));
    assert_eq!(get_file_extension(".bashrc"), None);
}

#[test]
fn test_is_code_file() {
    assert!(is_code_file("test.ts"));
    assert!(is_code_file("test.js"));
    assert!(is_code_file("test.tsx"));
    assert!(is_code_file("test.jsx"));
    assert!(!is_code_file("test.txt"));
    assert!(!is_code_file("README"));
}

#[test]
fn test_is_test_file() {
    assert!(is_test_file("foo.test.ts"));
    assert!(is_test_file("component.test.ts"));
    assert!(is_test_file("component.spec.js"));
    assert!(is_test_file("tests/my_test.ts"));
    assert!(is_test_file("__tests__/my_test.ts"));
    assert!(!is_test_file("component.ts"));
    assert!(!is_test_file("my_special_file.js"));
}

#[test]
fn test_calculate_file_score_typical() {
    let halstead = dummy_halstead(30);
    let input = ModuleScoreInput {
        cyclomatic: 10,
        halstead: &halstead,
        line_count: 100,
    };
    let score = calculate_file_score(&input, FileScoreFormula::Original);
    assert_eq!(score, 28.44385263727476);
}

#[test]
fn test_calculate_file_score_zero_cyclo() {
    let halstead = dummy_halstead(30);
    let input = ModuleScoreInput {
        cyclomatic: 0,
        halstead: &halstead,
        line_count: 100,
    };
    let score = calculate_file_score(&input, FileScoreFormula::Original);
    assert_eq!(score, 10.342822447159776);
}

#[test]
fn test_calculate_file_score_zero_lines() {
    let halstead = dummy_halstead(10);
    let input = ModuleScoreInput {
        cyclomatic: 5,
        halstead: &halstead,
        line_count: 0,
    };
    let score = calculate_file_score(&input, FileScoreFormula::Original);
    assert_eq!(score, 0.0);
}

#[test]
fn test_calculate_file_score_zero_vocab() {
    let halstead = dummy_halstead(0);
    let input = ModuleScoreInput {
        cyclomatic: 5,
        halstead: &halstead,
        line_count: 100,
    };
    let score = calculate_file_score(&input, FileScoreFormula::Original);
    assert_eq!(score, 0.0);
}
