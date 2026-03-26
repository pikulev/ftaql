use super::*;
use std::collections::{HashMap, HashSet};

fn create_test_graph() -> (HashMap<String, HashSet<String>>, String, String) {
    let mut graph = HashMap::new();
    graph.insert(
        "a".to_string(),
        ["b".to_string(), "c".to_string()].iter().cloned().collect(),
    );
    graph.insert("b".to_string(), ["c".to_string()].iter().cloned().collect());
    graph.insert("c".to_string(), HashSet::new());
    graph.insert(
        "d".to_string(),
        ["a".to_string(), "c".to_string()].iter().cloned().collect(),
    );
    // Add an isolated node
    let isolated_node = "isolated".to_string();
    graph.insert(isolated_node.clone(), HashSet::new());

    // Add a node that is a dependency but has no outgoing dependencies
    let sink_node = "sink".to_string();
    graph.insert(
        "e".to_string(),
        [sink_node.clone()].iter().cloned().collect(),
    );
    graph.insert(sink_node.clone(), HashSet::new());

    (graph, isolated_node, sink_node)
}

#[test]
fn test_calculate_ce() {
    let (graph, isolated_node, sink_node) = create_test_graph();
    assert_eq!(calculate_ce("a", &graph), 2);
    assert_eq!(calculate_ce("b", &graph), 1);
    assert_eq!(calculate_ce("c", &graph), 0);
    assert_eq!(calculate_ce("d", &graph), 2);
    assert_eq!(calculate_ce(&isolated_node, &graph), 0);
    assert_eq!(calculate_ce(&sink_node, &graph), 0);
    assert_eq!(calculate_ce("e", &graph), 1);
}

#[test]
fn test_calculate_ca() {
    let (graph, isolated_node, sink_node) = create_test_graph();
    assert_eq!(calculate_ca("a", &graph), 1);
    assert_eq!(calculate_ca("b", &graph), 1);
    assert_eq!(calculate_ca("c", &graph), 3);
    assert_eq!(calculate_ca("d", &graph), 0);
    assert_eq!(calculate_ca(&isolated_node, &graph), 0);
    assert_eq!(calculate_ca(&sink_node, &graph), 1);
}

#[test]
fn test_calculate_instability() {
    assert_eq!(calculate_instability(0, 0), 0.0);
    assert_eq!(calculate_instability(10, 0), 0.0);
    assert_eq!(calculate_instability(0, 10), 1.0);
    assert_eq!(calculate_instability(5, 5), 0.5);
}

#[test]
fn test_analyze_cycles() {
    let mut graph = HashMap::new();
    graph.insert("a".to_string(), ["b".to_string()].iter().cloned().collect());
    graph.insert("b".to_string(), ["c".to_string()].iter().cloned().collect());
    graph.insert("c".to_string(), ["d".to_string()].iter().cloned().collect());
    graph.insert("d".to_string(), ["b".to_string()].iter().cloned().collect()); // Cycle: b -> c -> d -> b
    let cycle_info = analyze_cycles(&graph);
    assert_eq!(cycle_info.has_cycles, true);
    assert_eq!(cycle_info.cycle_count, 1);
    assert_eq!(cycle_info.largest_cycle_size, 3);

    // Ensure the cycle members are correct
    let cycle_path = &cycle_info.cycles[0];
    assert!(cycle_path.contains(&"b".to_string()));
    assert!(cycle_path.contains(&"c".to_string()));
    assert!(cycle_path.contains(&"d".to_string()));
}

#[test]
fn test_analyze_cycles_no_cycles() {
    let mut graph = HashMap::new();
    graph.insert("a".to_string(), ["b".to_string()].iter().cloned().collect());
    graph.insert("b".to_string(), ["c".to_string()].iter().cloned().collect());
    graph.insert("c".to_string(), HashSet::new());
    let cycle_info = analyze_cycles(&graph);
    assert_eq!(cycle_info.has_cycles, false);
    assert_eq!(cycle_info.cycle_count, 0);
    assert_eq!(cycle_info.largest_cycle_size, 0);
}

#[test]
fn test_analyze_cycles_multiple_cycles() {
    let mut graph = HashMap::new();
    // Cycle 1: a -> b -> a
    graph.insert("a".to_string(), ["b".to_string()].iter().cloned().collect());
    graph.insert("b".to_string(), ["a".to_string()].iter().cloned().collect());
    // Cycle 2: c -> d -> e -> c
    graph.insert("c".to_string(), ["d".to_string()].iter().cloned().collect());
    graph.insert("d".to_string(), ["e".to_string()].iter().cloned().collect());
    graph.insert("e".to_string(), ["c".to_string()].iter().cloned().collect());
    // Independent node
    graph.insert("f".to_string(), HashSet::new());

    let cycle_info = analyze_cycles(&graph);
    assert_eq!(cycle_info.has_cycles, true);
    assert_eq!(cycle_info.cycle_count, 2);
    assert_eq!(cycle_info.largest_cycle_size, 3);

    // Check that both cycles are found
    let mut found_cycle1 = false;
    let mut found_cycle2 = false;
    for cycle_path in cycle_info.cycles {
        if cycle_path.len() == 2
            && cycle_path.contains(&"a".to_string())
            && cycle_path.contains(&"b".to_string())
        {
            found_cycle1 = true;
        }
        if cycle_path.len() == 3
            && cycle_path.contains(&"c".to_string())
            && cycle_path.contains(&"d".to_string())
            && cycle_path.contains(&"e".to_string())
        {
            found_cycle2 = true;
        }
    }
    assert!(found_cycle1);
    assert!(found_cycle2);
}

use swc_ecma_ast::{
    Ident, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier,
    ImportStarAsSpecifier, Module as SwcModule, ModuleItem, Str,
};

#[test]
fn test_extract_imports() {
    let module = SwcModule {
        span: Default::default(),
        body: vec![
            // import defaultExport from "test-module-1";
            ModuleItem::ImportDecl(ImportDecl {
                span: Default::default(),
                specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
                    span: Default::default(),
                    local: Ident::new("defaultExport".into(), Default::default()),
                })],
                src: Box::new(Str {
                    span: Default::default(),
                    value: "test-module-1".into(),
                    raw: None,
                }),
                type_only: false,
                with: None,
            }),
            // import { namedExport } from "test-module-2";
            ModuleItem::ImportDecl(ImportDecl {
                span: Default::default(),
                specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                    span: Default::default(),
                    local: Ident::new("namedExport".into(), Default::default()),
                    imported: None,
                    is_type_only: false,
                })],
                src: Box::new(Str {
                    span: Default::default(),
                    value: "test-module-2".into(),
                    raw: None,
                }),
                type_only: false,
                with: None,
            }),
            // import * as namespaceExport from "test-module-3";
            ModuleItem::ImportDecl(ImportDecl {
                span: Default::default(),
                specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                    span: Default::default(),
                    local: Ident::new("namespaceExport".into(), Default::default()),
                })],
                src: Box::new(Str {
                    span: Default::default(),
                    value: "test-module-3".into(),
                    raw: None,
                }),
                type_only: false,
                with: None,
            }),
        ],
        shebang: None,
    };

    let (imports, imported_symbols) = extract_imports(&module);

    assert_eq!(imports.len(), 3);
    assert!(imports.contains("test-module-1"));
    assert!(imports.contains("test-module-2"));
    assert!(imports.contains("test-module-3"));

    assert_eq!(imported_symbols.len(), 3);
    assert!(imported_symbols.contains("defaultExport"));
    assert!(imported_symbols.contains("namedExport"));
    assert!(imported_symbols.contains("namespaceExport"));
}
