use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// A module for custom serde serialization functions
mod serde_helpers {
    use serde::{ser::SerializeMap, Serializer};
    use std::collections::HashMap;

    // Custom serializer for HashMap to ensure keys are sorted.
    // This provides a stable output for snapshot testing.
    pub fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Ord + serde::Serialize,
        V: serde::Serialize,
    {
        let mut sorted: Vec<_> = value.iter().collect();
        sorted.sort_by_key(|a| a.0);
        let mut map = serializer.serialize_map(Some(sorted.len()))?;
        for (k, v) in sorted {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FtaQlConfigOptional {
    pub extensions: Option<Vec<String>>,
    pub exclude_filenames: Option<Vec<String>>,
    pub exclude_directories: Option<Vec<String>>,
    pub score_cap: Option<usize>,
    pub include_comments: Option<bool>,
    pub exclude_under: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CouplingAnalysis {
    pub file_path: String,
    pub dependencies: HashMap<String, Vec<String>>, // File -> Vec of imported identifiers
    pub dependents: HashMap<String, Vec<String>>,   // File -> Vec of imported identifiers
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportInfo {
    pub path: String,
    pub specifiers: Vec<String>,
    pub is_type_only: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExportKind {
    Type,
    Value,
    Class,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportInfo {
    pub name: String,
    pub kind: ExportKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FtaQlConfigResolved {
    pub extensions: Vec<String>,
    pub exclude_filenames: Vec<String>,
    pub exclude_directories: Vec<String>,
    pub score_cap: usize,
    pub include_comments: bool,
    pub exclude_under: usize,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct HalsteadMetrics {
    pub uniq_operators: usize,  // number of unique operators
    pub uniq_operands: usize,   // number of unique operands
    pub total_operators: usize, // total number of operators
    pub total_operands: usize,  // total number of operands
    pub program_length: usize,
    pub vocabulary_size: usize,
    pub volume: f64,
    pub difficulty: f64,
    pub effort: f64,
    pub time: f64,
    pub bugs: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SizeMetrics {
    pub line_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ComplexityMetrics {
    pub cyclomatic: usize,
    pub halstead: HalsteadMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CouplingMetrics {
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f64,
    #[serde(serialize_with = "serde_helpers::ordered_map")]
    pub dependency_strength: HashMap<String, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycles: Option<CycleData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CycleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_cycle_id: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CycleInfo {
    pub id: usize,
    pub size: usize,
    #[serde(serialize_with = "serde_helpers::ordered_map")]
    pub graph: HashMap<usize, HashMap<usize, usize>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Scores {
    pub file_score: f64,
    pub coupling_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileData {
    pub file_name: String,
    pub size_metrics: SizeMetrics,
    pub complexity_metrics: ComplexityMetrics,
    pub coupling_metrics: Option<CouplingMetrics>,
    pub scores: Scores,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProjectAnalysis {
    pub cycle_members: Vec<String>,
    pub cycles: Vec<CycleInfo>,
    pub runtime_cycles: Vec<CycleInfo>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FtaQlJsonOutput {
    pub project_analysis: ProjectAnalysis,
    pub findings: Vec<FileData>,
}
