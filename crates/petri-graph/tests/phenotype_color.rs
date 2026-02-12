use petri_graph::{ComputationGraph, ControllerPalette, Edge, NodeKind};

fn sample_graph() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputFoodDirection, // 0
            NodeKind::Constant(0.4),      // 1
            NodeKind::Add,                // 2
            NodeKind::OutputMoveX,        // 3
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 2,
                weight: 0.75,
            },
            Edge {
                from: 1,
                to: 2,
                weight: -0.2,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
        ],
    }
}

#[test]
fn phenotype_color_is_deterministic_for_identical_graphs() {
    let graph = sample_graph();
    let first = graph.phenotype_color();
    let second = graph.phenotype_color();
    let cloned = graph.clone().phenotype_color();

    assert_eq!(first, second);
    assert_eq!(first, cloned);
}

#[test]
fn phenotype_color_changes_when_graph_structure_changes() {
    let graph = sample_graph();
    let mut mutated = sample_graph();
    mutated.edges[0].weight = 0.95;

    assert_ne!(graph.phenotype_color(), mutated.phenotype_color());
}
