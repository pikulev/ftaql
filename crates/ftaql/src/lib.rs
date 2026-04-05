#[cfg(feature = "project-analysis")]
use crate::resolve::ResolverCache;
use crate::structs::{ComplexityMetrics, FileData, Scores, SizeMetrics};
#[cfg(feature = "project-analysis")]
use crate::structs::{CycleData, FtaQlConfigResolved, FtaQlJsonOutput, ProjectAnalysis};
#[cfg(feature = "project-analysis")]
use crate::utils::check_score_cap_breach;
use crate::utils::{calculate_file_score, FileScoreFormula, ModuleScoreInput};
#[cfg(feature = "project-analysis")]
use rayon::prelude::*;
#[cfg(feature = "project-analysis")]
use std::collections::HashMap;
#[cfg(feature = "project-analysis")]
use std::fs;
#[cfg(feature = "project-analysis")]
use std::path::Path;
#[cfg(feature = "project-analysis")]
use std::sync::{Arc, Mutex};

pub mod config;
pub mod coupling;
pub mod cyclo;
pub mod halstead;
pub mod parse;
#[cfg(feature = "project-analysis")]
pub mod resolve;
#[cfg(feature = "project-analysis")]
pub mod sqlite;
pub mod structs;
pub mod utils;
#[cfg(feature = "project-analysis")]
pub mod walk;

#[cfg(feature = "project-analysis")]
pub fn analyze_project(path: &String, config: Option<FtaQlConfigResolved>) -> FtaQlJsonOutput {
    let fta_config: FtaQlConfigResolved =
        config.unwrap_or_else(|| crate::config::get_default_config());

    let absolute_path = fs::canonicalize(path).expect("Failed to canonicalize path");
    let repo_path = absolute_path.to_str().unwrap().to_string();

    // First pass: collect all imports
    let mut override_builder = ignore::overrides::OverrideBuilder::new(&repo_path);
    for inc in &fta_config.includes {
        override_builder.add(inc).unwrap();
    }
    for exc in &fta_config.excludes {
        override_builder.add(&format!("!{}", exc)).unwrap();
    }
    let overrides = override_builder.build().unwrap();
    let mut walk_builder_base = ignore::WalkBuilder::new(&repo_path);
    walk_builder_base
        .git_ignore(true)
        .git_exclude(true)
        .standard_filters(true)
        .overrides(overrides);
    let walk_builder = walk_builder_base;
    let resolver_cache = ResolverCache::new();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let all_imports = Arc::new(Mutex::new(HashMap::new()));
    let all_exports = Arc::new(Mutex::new(HashMap::new()));

    walk_builder.build_parallel().run(|| {
        let all_imports = Arc::clone(&all_imports);
        let all_exports = Arc::clone(&all_exports);
        let resolver_cache = &resolver_cache;
        let runtime = &runtime;
        let repo_path = &repo_path;
        let fta_config = &fta_config;
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                if crate::utils::is_valid_file(repo_path, &entry, fta_config) {
                    let abs_file_path = entry.path();
                    let resolver = resolver_cache.get_or_create(abs_file_path);
                    let abs_file_path_str = abs_file_path.to_str().unwrap();
                    let rel_file_path = Path::new(abs_file_path_str)
                        .strip_prefix(repo_path)
                        .unwrap_or_else(|_| Path::new(abs_file_path_str))
                        .to_str()
                        .unwrap()
                        .to_string();
                    let parse_result = crate::parse::parse_module(
                        abs_file_path_str,
                        &resolver,
                        runtime,
                        repo_path,
                        fta_config.include_comments,
                    );
                    all_imports
                        .lock()
                        .unwrap()
                        .insert(rel_file_path.clone(), parse_result.imports);
                    all_exports
                        .lock()
                        .unwrap()
                        .insert(rel_file_path, parse_result.exports);
                }
            }
            ignore::WalkState::Continue
        })
    });
    let all_imports = Arc::try_unwrap(all_imports).unwrap().into_inner().unwrap();
    let all_exports = Arc::try_unwrap(all_exports).unwrap().into_inner().unwrap();

    // Analyze coupling
    let (
        coupling_analysis,
        cycles,
        file_to_cycle_id,
        runtime_cycles,
        file_to_runtime_cycle_id,
        cycle_members,
    ) = crate::coupling::analyze_coupling(&all_imports, &all_exports);

    // Second pass: parallel analysis of each file with coupling info
    let analysis_results: Vec<FileData> = coupling_analysis
        .into_par_iter()
        .filter_map(|analysis| {
            let abs_file_path = Path::new(&repo_path).join(&analysis.file_path);
            let resolver = resolver_cache.get_or_create(&abs_file_path);
            let abs_file_path_str = abs_file_path.to_str().unwrap();
            let parse_result = crate::parse::parse_module(
                abs_file_path_str,
                &resolver,
                &runtime,
                &repo_path,
                fta_config.include_comments,
            );
            if let Ok(module) = parse_result.module {
                let cyclo = crate::cyclo::cyclo(&module);
                let halstead = crate::halstead::halstead(&module);
                let line_count = parse_result.line_count;
                let score_input = ModuleScoreInput {
                    cyclomatic: cyclo,
                    halstead: &halstead,
                    line_count,
                };
                let file_score = calculate_file_score(&score_input, FileScoreFormula::Original);
                let score_cap = fta_config.score_cap;
                check_score_cap_breach(analysis.file_path.clone(), file_score, score_cap);
                let cycle_id = file_to_cycle_id.get(&analysis.file_path).copied();
                let runtime_cycle_id = file_to_runtime_cycle_id.get(&analysis.file_path).copied();
                let coupling_metrics = crate::coupling::coupling(
                    &analysis,
                    Some(CycleData {
                        cycle_id,
                        runtime_cycle_id,
                    }),
                );
                let coupling_score = crate::utils::calculate_coupling_score(
                    &coupling_metrics,
                    &cycles,
                    crate::utils::CouplingScoreFormula::Original,
                );
                Some(FileData {
                    file_name: analysis.file_path.clone(),
                    size_metrics: SizeMetrics { line_count },
                    complexity_metrics: ComplexityMetrics {
                        cyclomatic: cyclo,
                        halstead,
                    },
                    coupling_metrics: Some(coupling_metrics),
                    scores: Scores {
                        file_score: file_score,
                        coupling_score: coupling_score,
                    },
                })
            } else {
                None
            }
        })
        .collect();
    let mut analysis_results = analysis_results;
    analysis_results.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    FtaQlJsonOutput {
        project_analysis: ProjectAnalysis {
            cycles,
            runtime_cycles,
            cycle_members,
        },
        findings: analysis_results,
    }
}

impl FileData {
    pub fn abs_path<'a>(&'a self, project_root: &'a str) -> std::path::PathBuf {
        std::path::Path::new(project_root).join(&self.file_name)
    }
}

pub fn analyze_module(
    module: &swc_ecma_ast::Module,
    file_name: &str,
    line_count: usize,
) -> FileData {
    let cyclo = crate::cyclo::cyclo(module);
    let halstead = crate::halstead::halstead(module);
    let score_input = ModuleScoreInput {
        cyclomatic: cyclo,
        halstead: &halstead,
        line_count,
    };

    let file_score = calculate_file_score(&score_input, FileScoreFormula::Original);

    FileData {
        file_name: file_name.to_string(),
        size_metrics: SizeMetrics { line_count },
        complexity_metrics: ComplexityMetrics {
            cyclomatic: cyclo,
            halstead,
        },
        coupling_metrics: None,
        scores: Scores {
            file_score: file_score,
            coupling_score: 0.0,
        },
    }
}
