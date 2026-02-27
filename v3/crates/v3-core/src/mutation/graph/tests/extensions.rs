use super::*;

// ── Gap 2: Randomize AddInternalGraphNode kind tests ──

#[test]
fn add_internal_node_uses_varied_kinds() {
    use std::collections::HashSet;
    let mut discriminants: HashSet<std::mem::Discriminant<GraphNodeKind>> = HashSet::new();
    for seed in 0u64..500 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let last = g.internal_nodes.last().unwrap();
            discriminants.insert(std::mem::discriminant(&last.kind));
        }
    }
    assert!(
        discriminants.len() >= 5,
        "must see \u{2265}5 distinct kinds; got {}",
        discriminants.len()
    );
}

#[test]
fn add_internal_node_sometimes_adds_initial_edge() {
    let mut saw_with_edge = false;
    let mut saw_without_edge = false;
    for seed in 0u64..200 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let last = g.internal_nodes.last().unwrap();
            if last.inputs.is_empty() {
                saw_without_edge = true;
            } else {
                saw_with_edge = true;
            }
        }
        if saw_with_edge && saw_without_edge {
            break;
        }
    }
    assert!(saw_with_edge, "must sometimes add initial edge");
    assert!(saw_without_edge, "must sometimes have no initial edge");
}

#[test]
fn add_internal_node_edge_points_to_existing_internal() {
    for seed in 0u64..200 {
        let mut genome = v3alpha1_founder_genome();
        let pre_count = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            g.internal_nodes.len()
        } else {
            continue;
        };
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let last = g.internal_nodes.last().unwrap();
            for edge in &last.inputs {
                assert!(
                    (edge.source_idx as usize) < pre_count,
                    "edge source_idx {} must be < pre-existing count {}",
                    edge.source_idx,
                    pre_count
                );
            }
        }
    }
}

#[test]
fn add_internal_node_skips_edge_when_no_existing_internals() {
    let mut genome = graph_only_genome(vec![]);
    let mut r = rng(42);
    GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
    if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
        assert_eq!(g.internal_nodes.len(), 1);
        assert!(
            g.internal_nodes[0].inputs.is_empty(),
            "new node in empty graph must have no inputs"
        );
    }
}

// ── Gap 7: Proportional edge weight step tests ──

#[test]
fn alter_edge_weight_proportional_for_large_weights() {
    // Weight=10.0 should be scaled by ±20%, so result in [8.0, 12.0].
    let genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::Add,
        inputs: vec![GraphInput {
            source_idx: 0,
            weight: 10.0,
        }],
        hebbian: None,
    }]);
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        GraphMutator::apply(&mut g, GraphOperator::AlterGraphEdgeWeight, &mut r).unwrap();
        if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
            let w = gd.internal_nodes[0].inputs[0].weight;
            assert!(
                (8.0..=12.0).contains(&w),
                "weight {} must be in [8.0, 12.0] for proportional step on 10.0",
                w
            );
        }
    }
}

#[test]
fn alter_edge_weight_absolute_for_near_zero_weights() {
    // Weight=0.0 should use absolute step, result in [-0.1, 0.1].
    let genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::Add,
        inputs: vec![GraphInput {
            source_idx: 0,
            weight: 0.0,
        }],
        hebbian: None,
    }]);
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        GraphMutator::apply(&mut g, GraphOperator::AlterGraphEdgeWeight, &mut r).unwrap();
        if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
            let w = gd.internal_nodes[0].inputs[0].weight;
            assert!(
                (-0.1..=0.1).contains(&w),
                "weight {} must be in [-0.1, 0.1] for absolute step near zero",
                w
            );
        }
    }
}

// ── Gap 1: CustomOutput/InputRef parameterization tests ──

#[test]
fn custom_output_is_parameterized() {
    assert!(
        is_parameterized(&GraphNodeKind::CustomOutput(5)),
        "CustomOutput must be parameterized"
    );
}

#[test]
fn input_ref_is_parameterized() {
    assert!(
        is_parameterized(&GraphNodeKind::InputRef(3)),
        "InputRef must be parameterized"
    );
}

#[test]
fn mutate_operator_param_changes_custom_output_slot() {
    let genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::CustomOutput(5),
        inputs: vec![],
        hebbian: None,
    }]);
    let mut changed = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok() {
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                if let GraphNodeKind::CustomOutput(slot) = gd.internal_nodes[0].kind {
                    if slot != 5 {
                        changed = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(changed, "CustomOutput slot must change by \u{00b1}1");
}

#[test]
fn mutate_operator_param_changes_input_ref_index() {
    let genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::InputRef(3),
        inputs: vec![],
        hebbian: None,
    }]);
    let mut changed = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok() {
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                if let GraphNodeKind::InputRef(idx) = gd.internal_nodes[0].kind {
                    if idx != 3 {
                        changed = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(changed, "InputRef index must change by \u{00b1}1");
}

#[test]
fn mutate_operator_param_wraps_custom_output_at_boundary() {
    // CustomOutput(0) should eventually wrap to 255.
    let genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::CustomOutput(0),
        inputs: vec![],
        hebbian: None,
    }]);
    let mut saw_255 = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok() {
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                if let GraphNodeKind::CustomOutput(slot) = gd.internal_nodes[0].kind {
                    if slot == 255 {
                        saw_255 = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(saw_255, "CustomOutput(0) must wrap to 255 via wrapping_sub");
}

#[test]
fn mutate_operator_param_wraps_input_ref_at_boundary() {
    // InputRef(255) should eventually wrap to 0.
    let genome = graph_only_genome(vec![GraphInternalNode {
        kind: GraphNodeKind::InputRef(255),
        inputs: vec![],
        hebbian: None,
    }]);
    let mut saw_0 = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok() {
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                if let GraphNodeKind::InputRef(idx) = gd.internal_nodes[0].kind {
                    if idx == 0 {
                        saw_0 = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(saw_0, "InputRef(255) must wrap to 0 via wrapping_add");
}

#[test]
fn copy_edge_bundle_preserves_source_edges() {
    let nodes = vec![
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 3.0,
            }],
            hebbian: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::Relu,
            inputs: vec![],
            hebbian: None,
        },
    ];
    for seed in 0u64..100 {
        let mut genome = graph_only_genome(nodes.clone());
        let mut r = rng(seed);
        if GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r).is_ok() {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                // Source node (idx 0) must still have its original edge.
                assert!(
                    g.internal_nodes[0]
                        .inputs
                        .iter()
                        .any(|e| (e.weight - 3.0).abs() < 1e-7),
                    "source edges must be preserved"
                );
            }
        }
    }
}

#[test]
fn graph_weighted_random_favors_refinement() {
    let mut counts = std::collections::HashMap::new();
    let mut r = rng(42);
    for _ in 0..10_000 {
        let op = GraphOperator::random(&mut r);
        *counts.entry(op).or_insert(0u32) += 1;
    }
    let alter = counts
        .get(&GraphOperator::AlterGraphEdgeWeight)
        .copied()
        .unwrap_or(0);
    let add_node = counts
        .get(&GraphOperator::AddInternalGraphNode)
        .copied()
        .unwrap_or(0);
    assert!(
        alter > add_node * 2,
        "AlterGraphEdgeWeight (weight 4) must appear >2x AddInternalGraphNode (weight 1); got {} vs {}",
        alter, add_node,
    );
}

#[test]
fn graph_operator_weights_are_positive() {
    let all = GraphOperator::ALL;
    assert_eq!(all.len(), 17, "ALL must cover every GraphOperator variant");
    for &op in &all {
        assert!(op.weight() > 0, "weight must be positive for {:?}", op);
    }
}
