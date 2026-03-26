use crate::structs::{
    CouplingAnalysis, CouplingMetrics, CycleData, CycleInfo, ExportInfo, ExportKind, ImportInfo,
};
use petgraph::algo::kosaraju_scc;
use petgraph::graph::DiGraph;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

// This function now only detects the members of each cycle.
// It returns a list of cycles, where each cycle is a list of file paths.
fn find_cycle_members(graph: &DiGraph<String, ()>) -> (Vec<Vec<String>>, HashMap<String, usize>) {
    let scc = kosaraju_scc(&graph);
    let mut raw_cycles: Vec<Vec<String>> = scc
        .into_iter()
        .filter(|c| c.len() > 1)
        .map(|component| {
            let mut component_files: Vec<String> = component
                .iter()
                .map(|&node_index| graph[node_index].clone())
                .collect();
            component_files.sort();
            component_files
        })
        .collect();

    // Sort the cycles themselves lexicographically based on their members
    // to ensure a deterministic order before assigning IDs.
    raw_cycles.sort();

    let mut file_to_cycle_id = HashMap::new();
    for (id, component_files) in raw_cycles.iter().enumerate() {
        for file_path in component_files {
            file_to_cycle_id.insert(file_path.clone(), id);
        }
    }

    (raw_cycles, file_to_cycle_id)
}

// This function builds the final CycleInfo graph from the raw cycle members
// and the global file-to-index map.
fn build_cycle_info(
    raw_cycles: Vec<Vec<String>>,
    all_imports: &HashMap<String, Vec<ImportInfo>>,
    file_to_final_index: &HashMap<String, usize>,
) -> Vec<CycleInfo> {
    raw_cycles
        .into_par_iter()
        .enumerate()
        .map(|(id, component_files)| {
            let mut cycle_graph: HashMap<usize, HashMap<usize, usize>> = HashMap::new();

            for file_path in &component_files {
                if let Some(imports) = all_imports.get(file_path) {
                    let from_index = *file_to_final_index.get(file_path).unwrap();

                    for import_info in imports {
                        // Ensure the imported file is part of the current cycle component
                        if component_files.contains(&import_info.path) {
                            if let Some(to_index) = file_to_final_index.get(&import_info.path) {
                                let strength = if import_info.specifiers.contains(&"*".to_string())
                                    || import_info.specifiers.contains(&"dynamic".to_string())
                                {
                                    1
                                } else {
                                    import_info.specifiers.len()
                                };
                                cycle_graph
                                    .entry(from_index)
                                    .or_default()
                                    .insert(*to_index, strength);
                            }
                        }
                    }
                }
            }

            CycleInfo {
                id,
                size: component_files.len(),
                graph: cycle_graph,
            }
        })
        .collect()
}

pub fn analyze_coupling(
    all_imports: &HashMap<String, Vec<ImportInfo>>,
    all_exports: &HashMap<String, Vec<ExportInfo>>,
) -> (
    Vec<CouplingAnalysis>,
    Vec<CycleInfo>,
    HashMap<String, usize>,
    Vec<CycleInfo>,
    HashMap<String, usize>,
    Vec<String>,
) {
    // Build the main dependency graph
    let mut graph = DiGraph::<String, ()>::new();
    let mut node_map = HashMap::new();
    for file_path in all_imports.keys() {
        node_map.insert(file_path.clone(), graph.add_node(file_path.clone()));
    }
    for (file_path, imports) in all_imports {
        let from_node = *node_map.get(file_path).unwrap();
        for import_info in imports {
            if let Some(to_node) = node_map.get(&import_info.path) {
                graph.add_edge(from_node, *to_node, ());
            }
        }
    }

    // Filter for runtime-only imports
    let runtime_imports: HashMap<String, Vec<ImportInfo>> = all_imports
        .par_iter()
        .map(|(path, imports)| {
            let runtime_imports_for_file: Vec<ImportInfo> = imports
                .iter()
                .filter(|import| {
                    if import.is_type_only {
                        return false;
                    }
                    if let Some(exports) = all_exports.get(&import.path) {
                        if import.specifiers.contains(&"*".to_string())
                            || import.specifiers.contains(&"dynamic".to_string())
                        {
                            return true;
                        }
                        return import.specifiers.iter().any(|specifier| {
                            exports.iter().any(|export| {
                                &export.name == specifier
                                    && matches!(export.kind, ExportKind::Value | ExportKind::Class)
                            })
                        });
                    }
                    true
                })
                .cloned()
                .collect();
            (path.clone(), runtime_imports_for_file)
        })
        .filter(|(_, imports)| !imports.is_empty())
        .collect();

    // Build the runtime dependency graph
    let mut runtime_graph = DiGraph::<String, ()>::new();
    let mut runtime_node_map = HashMap::new();
    for file_path in runtime_imports.keys() {
        runtime_node_map.insert(file_path.clone(), runtime_graph.add_node(file_path.clone()));
    }
    for (file_path, imports) in &runtime_imports {
        let from_node = *runtime_node_map.get(file_path).unwrap();
        for import_info in imports {
            if let Some(to_node) = runtime_node_map.get(&import_info.path) {
                runtime_graph.add_edge(from_node, *to_node, ());
            }
        }
    }

    // --- Stage 1: Parallel Cycle Member Detection ---
    let ((raw_all_cycles, file_to_all_cycle_id), (raw_runtime_cycles, file_to_runtime_cycle_id)) =
        rayon::join(
            || find_cycle_members(&graph),
            || find_cycle_members(&runtime_graph),
        );

    // --- Stage 2: Sequential Unification ---
    let mut all_member_files = HashSet::new();
    for cycle in &raw_all_cycles {
        for member in cycle {
            all_member_files.insert(member.clone());
        }
    }
    for cycle in &raw_runtime_cycles {
        for member in cycle {
            all_member_files.insert(member.clone());
        }
    }

    let mut cycle_members: Vec<String> = all_member_files.into_iter().collect();
    cycle_members.sort();
    let file_to_final_index: HashMap<String, usize> = cycle_members
        .iter()
        .enumerate()
        .map(|(i, path)| (path.clone(), i))
        .collect();

    // --- Stage 3: Final Graph Assembly (can also be parallel) ---
    let (all_cycles, runtime_cycles) = rayon::join(
        || build_cycle_info(raw_all_cycles, all_imports, &file_to_final_index),
        || build_cycle_info(raw_runtime_cycles, &runtime_imports, &file_to_final_index),
    );

    // --- Final Analysis Data ---
    let keys: Vec<_> = all_imports.keys().cloned().collect();
    let analysis_results: Vec<CouplingAnalysis> = keys
        .into_par_iter()
        .map(|file_path| {
            let node_idx = *node_map.get(&file_path).unwrap();
            let dependencies: HashMap<String, Vec<String>> = graph
                .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
                .map(|neighbor_idx| {
                    let dependency_path = graph[neighbor_idx].clone();
                    let specifiers = all_imports
                        .get(&file_path)
                        .unwrap()
                        .iter()
                        .find(|i| i.path == dependency_path)
                        .map_or(vec![], |i| i.specifiers.clone());
                    (dependency_path, specifiers)
                })
                .collect();
            let dependents: HashMap<String, Vec<String>> = graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .map(|neighbor_idx| {
                    let dependent_path = graph[neighbor_idx].clone();
                    let specifiers = all_imports
                        .get(&dependent_path)
                        .unwrap()
                        .iter()
                        .find(|i| i.path == file_path)
                        .map_or(vec![], |i| i.specifiers.clone());
                    (dependent_path, specifiers)
                })
                .collect();

            CouplingAnalysis {
                file_path,
                dependencies,
                dependents,
            }
        })
        .collect();

    (
        analysis_results,
        all_cycles,
        file_to_all_cycle_id,
        runtime_cycles,
        file_to_runtime_cycle_id,
        cycle_members,
    )
}

pub fn coupling(analysis: &CouplingAnalysis, cycle_data: Option<CycleData>) -> CouplingMetrics {
    let afferent_coupling = analysis.dependents.len();
    let efferent_coupling = analysis.dependencies.len();
    let instability = if (afferent_coupling + efferent_coupling) == 0 {
        0.0
    } else {
        (efferent_coupling as f64) / (efferent_coupling + afferent_coupling) as f64
    };
    let dependency_strength = analysis
        .dependencies
        .iter()
        .map(|(k, v)| (k.clone(), v.len()))
        .collect();
    let cycles = cycle_data;
    CouplingMetrics {
        afferent_coupling,
        efferent_coupling,
        instability,
        dependency_strength,
        cycles,
    }
}
