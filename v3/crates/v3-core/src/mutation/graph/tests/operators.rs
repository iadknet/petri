use super::*;

#[test]
fn alter_graph_edge_weight_changes_weight() {
    let genome = v3alpha1_founder_genome();
    // Founder node 0 graph has internal nodes with inputs (e.g., idx 2 Threshold has inputs).
    let original_weight = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
        g.internal_nodes[2].inputs[0].weight
    } else {
        panic!()
    };
    let mut changed = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result = GraphMutator::apply(
            &mut g,
            GraphOperator::AlterGraphEdgeWeight,
            &[],
            0.0,
            &mut r,
        );
        if result.is_ok() {
            let new_weight = if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                gd.internal_nodes[2].inputs[0].weight
            } else {
                panic!()
            };
            if (new_weight - original_weight).abs() > 1e-7 {
                changed = true;
                break;
            }
        }
    }
    assert!(changed, "edge weight must change");
}

#[test]
fn swap_graph_operator_changes_node_kind() {
    let genome = v3alpha1_founder_genome();
    let mut swapped = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::SwapGraphOperator, &[], 0.0, &mut r).is_ok() {
            swapped = true;
            break;
        }
    }
    assert!(swapped, "swap must succeed on founder genome");
}

#[test]
fn mutate_graph_operator_param_changes_param() {
    let genome = v3alpha1_founder_genome();
    // Node 0 graph has Threshold(24.0) at internal index 2.
    let original_param = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
        if let GraphNodeKind::Threshold(p) = g.internal_nodes[2].kind {
            p
        } else {
            panic!("expected Threshold")
        }
    } else {
        panic!()
    };
    let mut changed = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(
            &mut g,
            GraphOperator::MutateGraphOperatorParam,
            &[],
            0.0,
            &mut r,
        )
        .is_ok()
        {
            let new_param = if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                if let GraphNodeKind::Threshold(p) = gd.internal_nodes[2].kind {
                    p
                } else {
                    original_param
                }
            } else {
                original_param
            };
            if (new_param - original_param).abs() > 1e-7 {
                changed = true;
                break;
            }
        }
    }
    assert!(changed, "parameter must change after mutation");
}

#[test]
fn add_internal_graph_node_increases_internal_node_count() {
    let mut genome = v3alpha1_founder_genome();
    let before = graph_node_internal_count(&genome);
    let mut r = rng(0);
    GraphMutator::apply(
        &mut genome,
        GraphOperator::AddInternalGraphNode,
        &[],
        0.0,
        &mut r,
    )
    .unwrap();
    let after = graph_node_internal_count(&genome);
    assert_eq!(after, before + 1);
}

#[test]
fn remove_internal_graph_node_decreases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before = graph_node_internal_count(&genome);
    assert!(before > 0, "founder graph node must have internal nodes");
    let mut r = rng(0);
    GraphMutator::apply(
        &mut genome,
        GraphOperator::RemoveInternalGraphNode,
        &[],
        0.0,
        &mut r,
    )
    .unwrap();
    let after = graph_node_internal_count(&genome);
    assert_eq!(after, before - 1);
}

#[test]
fn graph_mutator_on_vm_only_genome_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Replace all nodes with VM-only nodes.
    genome.nodes = vec![NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![],
    }];
    let mut r = rng(0);
    let result = GraphMutator::apply(
        &mut genome,
        GraphOperator::AlterGraphEdgeWeight,
        &[],
        0.0,
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn add_graph_edge_increases_input_count() {
    let mut genome = v3alpha1_founder_genome();
    // Count total inputs across all graph internal nodes before.
    let before: usize = genome
        .nodes
        .iter()
        .filter_map(|n| {
            if let BackendDef::Graph(ref g) = n.backend_def {
                Some(
                    g.internal_nodes
                        .iter()
                        .map(|i| i.inputs.len())
                        .sum::<usize>(),
                )
            } else {
                None
            }
        })
        .sum();
    let mut r = rng(0);
    GraphMutator::apply(&mut genome, GraphOperator::AddGraphEdge, &[], 0.0, &mut r).unwrap();
    let after: usize = genome
        .nodes
        .iter()
        .filter_map(|n| {
            if let BackendDef::Graph(ref g) = n.backend_def {
                Some(
                    g.internal_nodes
                        .iter()
                        .map(|i| i.inputs.len())
                        .sum::<usize>(),
                )
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        after,
        before + 1,
        "add_graph_edge must add exactly one input"
    );
}

#[test]
fn add_graph_edge_on_empty_internals_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Clear all internal nodes from graph backends.
    for node in &mut genome.nodes {
        if let BackendDef::Graph(ref mut g) = node.backend_def {
            g.internal_nodes.clear();
        }
    }
    let mut r = rng(0);
    let result = GraphMutator::apply(&mut genome, GraphOperator::AddGraphEdge, &[], 0.0, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn constant_is_parameterized() {
    assert!(
        is_parameterized(&GraphNodeKind::Constant(1.0)),
        "Constant must be parameterized"
    );
}

#[test]
fn mutate_operator_param_changes_constant_value() {
    let mut genome = v3alpha1_founder_genome();
    // Add a Constant internal node to the graph node.
    if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
        g.internal_nodes.push(GraphInternalNode {
            kind: GraphNodeKind::Constant(0.5),
            inputs: vec![],
            plasticity: None,
        });
    }
    let original = 0.5f32;
    let mut changed = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(
            &mut g,
            GraphOperator::MutateGraphOperatorParam,
            &[],
            0.0,
            &mut r,
        )
        .is_ok()
        {
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                // Check last internal node (the Constant we added).
                let last = gd.internal_nodes.last().unwrap();
                if let GraphNodeKind::Constant(p) = last.kind {
                    if (p - original).abs() > 1e-7 {
                        changed = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(changed, "mutate_operator_param must change Constant value");
}

#[test]
fn swap_operator_can_produce_parameterized_kinds() {
    // Over many seeds, swap must sometimes produce parameterized kinds
    // (Threshold, DecayIntegrator, Momentum, Oscillator, Constant).
    let mut found_parameterized = false;
    for seed in 0u64..200 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        if GraphMutator::apply(
            &mut genome,
            GraphOperator::SwapGraphOperator,
            &[],
            0.0,
            &mut r,
        )
        .is_ok()
        {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                for node in &g.internal_nodes {
                    if is_parameterized(&node.kind) {
                        found_parameterized = true;
                        break;
                    }
                }
            }
        }
        if found_parameterized {
            break;
        }
    }
    assert!(
        found_parameterized,
        "swap must sometimes produce parameterized kinds"
    );
}

#[test]
fn random_graph_node_kind_bounds_custom_output_to_valid_range() {
    for seed in 0u64..5_000 {
        let mut r = rng(seed);
        let kind = random_graph_node_kind(4, &mut r);
        if let GraphNodeKind::CustomOutput(slot) = kind {
            assert!(
                slot < 12,
                "CustomOutput slot {slot} must be < OUTPUT_SLOT_COUNT (12)"
            );
        }
        if let GraphNodeKind::InputRef { ref_idx, .. } = kind {
            assert!(
                ref_idx < 4,
                "InputRef ref_idx {ref_idx} must be < input_ref_count (4)"
            );
        }
    }
}

#[test]
fn random_graph_node_kind_bounds_write_action_meta() {
    for seed in 0u64..5_000 {
        let mut r = rng(seed);
        if let GraphNodeKind::WriteActionMeta(slot) = random_graph_node_kind(1, &mut r) {
            assert!(slot < 8, "WriteActionMeta slot {slot} must be < 8");
        }
    }
}

#[test]
fn random_graph_node_kind_bounds_push_action() {
    for seed in 0u64..5_000 {
        let mut r = rng(seed);
        if let GraphNodeKind::PushAction(slot) = random_graph_node_kind(1, &mut r) {
            assert!(slot < 5, "PushAction slot {slot} must be < 5");
        }
    }
}

#[test]
fn retarget_graph_edge_changes_source_idx() {
    let genome = v3alpha1_founder_genome();
    // Founder graph node 0 has internal nodes with inputs.
    let mut changed = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::RetargetGraphEdge, &[], 0.0, &mut r).is_ok() {
            // Check if any edge source_idx differs from original.
            if let (BackendDef::Graph(ref orig), BackendDef::Graph(ref mutated)) =
                (&genome.nodes[0].backend_def, &g.nodes[0].backend_def)
            {
                for (o, m) in orig
                    .internal_nodes
                    .iter()
                    .zip(mutated.internal_nodes.iter())
                {
                    for (oi, mi) in o.inputs.iter().zip(m.inputs.iter()) {
                        if oi.source_idx != mi.source_idx {
                            changed = true;
                            break;
                        }
                    }
                    if changed {
                        break;
                    }
                }
            }
        }
        if changed {
            break;
        }
    }
    assert!(changed, "retarget must change a source_idx");
}

#[test]
fn retarget_graph_edge_no_edges_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Clear all inputs from graph internal nodes.
    if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
        for node in &mut g.internal_nodes {
            node.inputs.clear();
        }
    }
    let mut r = rng(0);
    let result = GraphMutator::apply(
        &mut genome,
        GraphOperator::RetargetGraphEdge,
        &[],
        0.0,
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn remove_graph_edge_decreases_input_count() {
    let genome = v3alpha1_founder_genome();
    let before: usize = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
        g.internal_nodes.iter().map(|n| n.inputs.len()).sum()
    } else {
        panic!("expected graph backend");
    };
    assert!(before > 0, "founder graph must have edges");
    let mut success = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::RemoveGraphEdge, &[], 0.0, &mut r).is_ok() {
            let after: usize = if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                gd.internal_nodes.iter().map(|n| n.inputs.len()).sum()
            } else {
                before
            };
            assert_eq!(after, before - 1, "remove_graph_edge must remove one input");
            success = true;
            break;
        }
    }
    assert!(success, "remove_graph_edge must succeed on founder genome");
}

#[test]
fn remove_graph_edge_no_edges_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
        for node in &mut g.internal_nodes {
            node.inputs.clear();
        }
    }
    let mut r = rng(0);
    let result = GraphMutator::apply(
        &mut genome,
        GraphOperator::RemoveGraphEdge,
        &[],
        0.0,
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn raw_field_mutation_bounds_input_ref_to_valid_range() {
    for seed in 0u64..512 {
        let mut genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 0,
            },
            inputs: vec![],
            plasticity: None,
        }]);
        let mut r = rng(seed);
        GraphMutator::apply(
            &mut genome,
            GraphOperator::GraphRawFieldMutation,
            &[],
            0.0,
            &mut r,
        )
        .unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            if let GraphNodeKind::InputRef { ref_idx, .. } = g.internal_nodes[0].kind {
                // Genome has 0 input_refs, so max(1) = 1 → ref_idx must be 0.
                assert!(
                    ref_idx < 1,
                    "InputRef ref_idx {ref_idx} must be < max(input_ref_count, 1)"
                );
            }
        }
    }
}

#[test]
fn raw_field_mutation_bounds_custom_output_to_valid_range() {
    for seed in 0u64..512 {
        let mut genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::CustomOutput(0),
            inputs: vec![],
            plasticity: None,
        }]);
        let mut r = rng(seed);
        GraphMutator::apply(
            &mut genome,
            GraphOperator::GraphRawFieldMutation,
            &[],
            0.0,
            &mut r,
        )
        .unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            if let GraphNodeKind::CustomOutput(slot) = g.internal_nodes[0].kind {
                assert!(
                    slot < 12,
                    "CustomOutput slot {slot} must be < OUTPUT_SLOT_COUNT (12)"
                );
            }
        }
    }
}

#[test]
fn raw_field_mutation_bounds_edge_source_to_valid_range() {
    for seed in 0u64..512 {
        let mut genome = graph_only_genome(vec![
            GraphInternalNode {
                kind: GraphNodeKind::Constant(1.0),
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            },
        ]);
        let mut r = rng(seed);
        GraphMutator::apply(
            &mut genome,
            GraphOperator::GraphRawFieldMutation,
            &[],
            0.0,
            &mut r,
        )
        .unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            for (i, node) in g.internal_nodes.iter().enumerate() {
                for (j, edge) in node.inputs.iter().enumerate() {
                    assert!(
                        (edge.source_idx as usize) < g.internal_nodes.len(),
                        "seed {seed}: node {i} edge {j} source_idx {} >= internal_nodes.len() {}",
                        edge.source_idx,
                        g.internal_nodes.len()
                    );
                }
            }
        }
    }
}

#[test]
fn graph_after_mutation_passes_parseability_gate() {
    let operators = [
        GraphOperator::AlterGraphEdgeWeight,
        GraphOperator::SwapGraphOperator,
        GraphOperator::MutateGraphOperatorParam,
        GraphOperator::AddInternalGraphNode,
        GraphOperator::RemoveInternalGraphNode,
        GraphOperator::AddGraphEdge,
        GraphOperator::RetargetGraphEdge,
        GraphOperator::RemoveGraphEdge,
        GraphOperator::GraphRawFieldMutation,
        GraphOperator::CopyInternalNode,
        GraphOperator::CopySubgraph,
        GraphOperator::CopyEdgeBundle,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 300);
        let _ = GraphMutator::apply(&mut genome, op, &[], 0.0, &mut r);
        assert!(
            ParseabilityGate::validate(&genome).is_ok(),
            "parseability failed after {:?}",
            op
        );
    }
}

#[test]
fn random_graph_node_kind_covers_all_30_variants() {
    use std::collections::HashSet;
    let mut discriminants: HashSet<std::mem::Discriminant<GraphNodeKind>> = HashSet::new();
    for seed in 0u64..5000 {
        let mut r = rng(seed);
        let kind = random_graph_node_kind(4, &mut r);
        discriminants.insert(std::mem::discriminant(&kind));
    }
    assert_eq!(
        discriminants.len(),
        30,
        "all 30 GraphNodeKind variants must be reachable; got {}",
        discriminants.len()
    );
}

#[test]
fn is_parameterized_returns_true_for_slot_kinds() {
    assert!(is_parameterized(&GraphNodeKind::ReadSlot(0)));
    assert!(is_parameterized(&GraphNodeKind::ReadSlotPrev(0)));
    assert!(is_parameterized(&GraphNodeKind::WriteSlot(0)));
    assert!(is_parameterized(&GraphNodeKind::ClearSlot(0)));
}

#[test]
fn mutate_operator_param_does_not_panic_on_slot_kinds() {
    // Build a genome with graph nodes containing slot kinds.
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
        // Replace first 4 internal nodes with slot kinds.
        g.internal_nodes[0].kind = GraphNodeKind::ReadSlot(5);
        g.internal_nodes[1].kind = GraphNodeKind::ReadSlotPrev(10);
        g.internal_nodes[2].kind = GraphNodeKind::WriteSlot(15);
        g.internal_nodes[3].kind = GraphNodeKind::ClearSlot(0);
    }
    // Run mutate_operator_param many times — must not panic.
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = mutate_operator_param(&mut g, 0, &mut r);
    }
    // Verify values actually changed.
    let original_kinds: Vec<GraphNodeKind> =
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            g.internal_nodes[0..4]
                .iter()
                .map(|n| n.kind.clone())
                .collect()
        } else {
            panic!()
        };
    let mut any_changed = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = mutate_operator_param(&mut g, 0, &mut r);
        if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
            for (i, orig) in original_kinds.iter().enumerate() {
                if gd.internal_nodes[i].kind != *orig {
                    any_changed = true;
                }
            }
        }
    }
    assert!(any_changed, "mutate_operator_param must modify slot kinds");
}

#[test]
fn graph_raw_field_mutation_handles_slot_kinds() {
    let mut genome = v3alpha1_founder_genome();
    if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
        g.internal_nodes[0].kind = GraphNodeKind::ReadSlot(5);
        g.internal_nodes[1].kind = GraphNodeKind::WriteSlot(10);
    }
    // Run many times — must not panic and should sometimes mutate slot fields.
    let mut any_mutated = false;
    for seed in 0u64..500 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = apply_graph_raw_field_mutation(&mut g, 0, &mut r);
        if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
            if gd.internal_nodes[0].kind != GraphNodeKind::ReadSlot(5)
                || gd.internal_nodes[1].kind != GraphNodeKind::WriteSlot(10)
            {
                any_mutated = true;
                break;
            }
        }
    }
    assert!(
        any_mutated,
        "raw field mutation must sometimes mutate slot kind fields"
    );
}

// ── InputRef leaf protection + edge reindexing tests ──

#[test]
fn remove_internal_node_never_removes_input_ref() {
    // Graph with ONLY InputRef nodes → should return NoApplicableTarget.
    let mut genome = graph_only_genome(vec![
        GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 0,
            },
            inputs: vec![],
            plasticity: None,
        },
        GraphInternalNode {
            kind: GraphNodeKind::InputRef {
                ref_idx: 0,
                sub_idx: 1,
            },
            inputs: vec![],
            plasticity: None,
        },
    ]);
    let mut r = rng(42);
    let result = GraphMutator::apply(
        &mut genome,
        GraphOperator::RemoveInternalGraphNode,
        &[],
        0.0,
        &mut r,
    );
    assert_eq!(
        result,
        Err(MutationSkipReason::NoApplicableTarget),
        "remove must not target InputRef nodes"
    );
    // Nodes unchanged
    if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
        assert_eq!(g.internal_nodes.len(), 2);
    }
}

#[test]
fn remove_internal_node_only_removes_non_input_ref() {
    // Mixed graph: InputRef nodes + computation nodes. After many removals,
    // InputRef count must never decrease.

    let make_genome = || {
        graph_only_genome(vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 1,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Sigmoid,
                inputs: vec![GraphInput {
                    source_idx: 1,
                    weight: 0.5,
                }],
                plasticity: None,
            },
        ])
    };
    for seed in 0u64..100 {
        let mut genome = make_genome();
        let mut r = rng(seed);
        if GraphMutator::apply(
            &mut genome,
            GraphOperator::RemoveInternalGraphNode,
            &[],
            0.0,
            &mut r,
        )
        .is_ok()
        {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let input_ref_count = g
                    .internal_nodes
                    .iter()
                    .filter(|n| matches!(n.kind, GraphNodeKind::InputRef { .. }))
                    .count();
                assert_eq!(
                    input_ref_count, 2,
                    "InputRef nodes must never be removed (seed {})",
                    seed
                );
                // Total should be 3 (one computation node removed)
                assert_eq!(g.internal_nodes.len(), 3, "one node removed (seed {})", seed);
            }
        }
    }
}

#[test]
fn remove_internal_node_reindexes_edges() {
    // After removing a computation node, edges on surviving nodes should be
    // reindexed correctly.

    let make_genome = || {
        graph_only_genome(vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Sigmoid,
                inputs: vec![GraphInput {
                    source_idx: 1,
                    weight: 0.5,
                }],
                plasticity: None,
            },
        ])
    };
    let mut found_valid_reindex = false;
    for seed in 0u64..200 {
        let mut genome = make_genome();
        let mut r = rng(seed);
        if GraphMutator::apply(
            &mut genome,
            GraphOperator::RemoveInternalGraphNode,
            &[],
            0.0,
            &mut r,
        )
        .is_ok()
        {
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                // All surviving edges should point within valid range or u16::MAX
                for node in &g.internal_nodes {
                    for edge in &node.inputs {
                        assert!(
                            (edge.source_idx as usize) < g.internal_nodes.len()
                                || edge.source_idx == u16::MAX,
                            "edge source_idx {} out of range (len={})",
                            edge.source_idx,
                            g.internal_nodes.len()
                        );
                    }
                }
                found_valid_reindex = true;
            }
        }
    }
    assert!(
        found_valid_reindex,
        "must successfully remove a node and reindex"
    );
}
