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
        let result = GraphMutator::apply(&mut g, GraphOperator::AlterGraphEdgeWeight, &mut r);
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
        if GraphMutator::apply(&mut g, GraphOperator::SwapGraphOperator, &mut r).is_ok() {
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
        if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok() {
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
    GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
    let after = graph_node_internal_count(&genome);
    assert_eq!(after, before + 1);
}

#[test]
fn remove_internal_graph_node_decreases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before = graph_node_internal_count(&genome);
    assert!(before > 0, "founder graph node must have internal nodes");
    let mut r = rng(0);
    GraphMutator::apply(&mut genome, GraphOperator::RemoveInternalGraphNode, &mut r).unwrap();
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
    let result = GraphMutator::apply(&mut genome, GraphOperator::AlterGraphEdgeWeight, &mut r);
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
    GraphMutator::apply(&mut genome, GraphOperator::AddGraphEdge, &mut r).unwrap();
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
    let result = GraphMutator::apply(&mut genome, GraphOperator::AddGraphEdge, &mut r);
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
        if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok() {
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
        if GraphMutator::apply(&mut genome, GraphOperator::SwapGraphOperator, &mut r).is_ok() {
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
fn random_graph_node_kind_covers_all_30_variants_old() {
    use std::collections::HashSet;
    let mut discriminants: HashSet<std::mem::Discriminant<GraphNodeKind>> = HashSet::new();
    for seed in 0u64..5000 {
        let mut r = rng(seed);
        let kind = random_graph_node_kind(&mut r);
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
fn random_graph_node_kind_reaches_out_of_range_input_ref_and_custom_output() {
    let mut saw_out_of_range_input_ref = false;
    let mut saw_out_of_range_custom_output = false;
    for seed in 0u64..20_000 {
        let mut r = rng(seed);
        match random_graph_node_kind(&mut r) {
            GraphNodeKind::InputRef { ref_idx, .. } if ref_idx > 11 => {
                saw_out_of_range_input_ref = true
            }
            GraphNodeKind::CustomOutput(idx) if idx > 11 => saw_out_of_range_custom_output = true,
            _ => {}
        }
        if saw_out_of_range_input_ref && saw_out_of_range_custom_output {
            break;
        }
    }
    assert!(
        saw_out_of_range_input_ref,
        "InputRef mutation surface must include out-of-range u8 values"
    );
    assert!(
        saw_out_of_range_custom_output,
        "CustomOutput mutation surface must include out-of-range u8 values"
    );
}

#[test]
fn retarget_graph_edge_changes_source_idx() {
    let genome = v3alpha1_founder_genome();
    // Founder graph node 0 has internal nodes with inputs.
    let mut changed = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if GraphMutator::apply(&mut g, GraphOperator::RetargetGraphEdge, &mut r).is_ok() {
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
    let result = GraphMutator::apply(&mut genome, GraphOperator::RetargetGraphEdge, &mut r);
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
        if GraphMutator::apply(&mut g, GraphOperator::RemoveGraphEdge, &mut r).is_ok() {
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
    let result = GraphMutator::apply(&mut genome, GraphOperator::RemoveGraphEdge, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn raw_field_mutation_can_set_input_ref_out_of_range() {
    let mut found_out_of_range = false;
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
        GraphMutator::apply(&mut genome, GraphOperator::GraphRawFieldMutation, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            if let GraphNodeKind::InputRef { ref_idx, .. } = g.internal_nodes[0].kind {
                if ref_idx > 11 {
                    found_out_of_range = true;
                    break;
                }
            }
        }
    }
    assert!(
        found_out_of_range,
        "raw graph mutation must reach InputRef values above output slot range"
    );
}

#[test]
fn raw_field_mutation_can_set_custom_output_out_of_range() {
    let mut found_out_of_range = false;
    for seed in 0u64..512 {
        let mut genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::CustomOutput(0),
            inputs: vec![],
            plasticity: None,
        }]);
        let mut r = rng(seed);
        GraphMutator::apply(&mut genome, GraphOperator::GraphRawFieldMutation, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            if let GraphNodeKind::CustomOutput(slot) = g.internal_nodes[0].kind {
                if slot > 11 {
                    found_out_of_range = true;
                    break;
                }
            }
        }
    }
    assert!(
        found_out_of_range,
        "raw graph mutation must reach CustomOutput values above output slot range"
    );
}

#[test]
fn raw_field_mutation_can_set_edge_source_out_of_range() {
    let mut found_out_of_range = false;
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
        GraphMutator::apply(&mut genome, GraphOperator::GraphRawFieldMutation, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let source_idx = g.internal_nodes[1].inputs[0].source_idx as usize;
            if source_idx >= g.internal_nodes.len() {
                found_out_of_range = true;
                break;
            }
        }
    }
    assert!(
        found_out_of_range,
        "raw graph mutation must reach edge source indices outside internal node bounds"
    );
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
        let _ = GraphMutator::apply(&mut genome, op, &mut r);
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
        let kind = random_graph_node_kind(&mut r);
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
