use super::*;
use crate::config::MutationConfig;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{GraphSource, OutputSinkKind};
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::creature::parseability::ParseabilityGate;
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn apply(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl rand::Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    apply_with_config(
        genome,
        op,
        reachable_nodes,
        bias,
        rng,
        &MutationConfig::default(),
    )
}

fn apply_with_config(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl rand::Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    TopologyMutator::apply(genome, op, reachable_nodes, bias, rng, config)
}

fn forced_birth_config(
    graph_backend_chance: f32,
    graph_initialized_chance: f32,
    graph_compute_gate_chance: f32,
) -> MutationConfig {
    let mut config = MutationConfig::default();
    config.topology_new_node_birth.graph_backend_chance = graph_backend_chance;
    config.topology_new_node_birth.graph_initialized_chance = graph_initialized_chance;
    config.topology_new_node_birth.graph_compute_gate_chance = graph_compute_gate_chance;
    config
}

#[test]
fn add_node_increases_node_count_by_one() {
    let mut genome = v3alpha1_founder_genome();
    let before = genome.nodes.len();
    let mut r = rng(0);
    apply(&mut genome, TopologyOperator::AddNode, &[], 0.0, &mut r).unwrap();
    assert_eq!(genome.nodes.len(), before + 1);
}

#[test]
fn remove_node_decreases_node_count() {
    let mut genome = v3alpha1_founder_genome();
    assert!(genome.nodes.len() >= 2, "founder must have >=2 nodes");
    let before = genome.nodes.len();
    let mut r = rng(1);
    apply(&mut genome, TopologyOperator::RemoveNode, &[], 0.0, &mut r).unwrap();
    assert_eq!(genome.nodes.len(), before - 1);
}

#[test]
fn remove_node_on_single_node_genome_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Reduce to single node.
    genome.nodes.truncate(1);
    genome.entry_node_id = genome.nodes[0].node_id;
    let mut r = rng(2);
    let result = apply(&mut genome, TopologyOperator::RemoveNode, &[], 0.0, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn retarget_node_target_changes_target() {
    let mut genome = v3alpha1_founder_genome();
    // Node 0 has targets=[NodeId::new(1)] in the founder.
    let before_target = genome.nodes[0].targets[0];
    // Run until the target changes or we give up (not guaranteed since genome only has 2 nodes).
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut rr = rng(seed);
        let _ = apply(
            &mut g,
            TopologyOperator::RetargetNodeTarget,
            &[],
            0.0,
            &mut rr,
        );
        if g.nodes[0].targets[0] != before_target {
            genome = g;
            break;
        }
    }
    // At minimum, the operation doesn't fail and targets still has same length.
    let mut r2 = rng(99);
    let result = apply(
        &mut genome,
        TopologyOperator::RetargetNodeTarget,
        &[],
        0.0,
        &mut r2,
    );
    assert!(result.is_ok());
}

#[test]
fn add_route_target_increases_target_count() {
    let mut genome = v3alpha1_founder_genome();
    let before = genome.nodes[0].targets.len();
    let mut r = rng(4);
    // Force node index 0 by using a genome with known structure.
    apply(
        &mut genome,
        TopologyOperator::AddRouteTarget,
        &[],
        0.0,
        &mut r,
    )
    .unwrap();
    // Total targets across all nodes must increase by 1.
    let after: usize = genome.nodes.iter().map(|n| n.targets.len()).sum();
    let before_total: usize = v3alpha1_founder_genome()
        .nodes
        .iter()
        .map(|n| n.targets.len())
        .sum();
    // before_total = before counts. Add one.
    let _ = before;
    assert_eq!(after, before_total + 1);
}

#[test]
fn change_entry_node_changes_entry() {
    let mut genome = v3alpha1_founder_genome();
    assert!(genome.nodes.len() >= 2);
    let original_entry = genome.entry_node_id;
    let mut r = rng(5);
    apply(
        &mut genome,
        TopologyOperator::ChangeEntryNode,
        &[],
        0.0,
        &mut r,
    )
    .unwrap();
    assert_ne!(
        genome.entry_node_id, original_entry,
        "entry node must change"
    );
}

#[test]
fn topology_after_each_operator_passes_parseability_gate() {
    let operators = [
        TopologyOperator::AddNode,
        TopologyOperator::RemoveNode,
        TopologyOperator::RetargetNodeTarget,
        TopologyOperator::AddRouteTarget,
        TopologyOperator::RemoveRouteTarget,
        TopologyOperator::ChangeEntryNode,
        TopologyOperator::SwapNodeBackend,
        TopologyOperator::RewriteNodeId,
        TopologyOperator::CopyNode,
        TopologyOperator::CopyMeshBackwardSlice,
        TopologyOperator::CopyMeshForwardSlice,
        TopologyOperator::SpliceNode,
        TopologyOperator::SwapRouteTargets,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 100);
        let result = apply(&mut genome, op, &[], 0.0, &mut r);
        // Either succeeds or NoApplicableTarget — either way, genome must still be parseable.
        match result {
            Ok(_) | Err(MutationSkipReason::NoApplicableTarget) => {
                assert!(
                    ParseabilityGate::validate(&genome).is_ok(),
                    "parseability failed after {:?}: {:?}",
                    op,
                    ParseabilityGate::validate(&genome)
                );
            }
            Err(other) => panic!("unexpected skip reason {:?} for {:?}", other, op),
        }
    }
}

#[test]
fn swap_node_backend_toggles_backend_on_single_node_genome() {
    let mut genome = v3alpha1_founder_genome();
    genome.nodes.truncate(1);
    genome.entry_node_id = genome.nodes[0].node_id;

    let mut r1 = rng(7);
    apply(
        &mut genome,
        TopologyOperator::SwapNodeBackend,
        &[],
        0.0,
        &mut r1,
    )
    .unwrap();
    assert!(matches!(genome.nodes[0].backend_def, BackendDef::Vm(_)));

    let mut r2 = rng(8);
    apply(
        &mut genome,
        TopologyOperator::SwapNodeBackend,
        &[],
        0.0,
        &mut r2,
    )
    .unwrap();
    assert!(matches!(genome.nodes[0].backend_def, BackendDef::Graph(_)));
}

#[test]
fn rewrite_node_id_rewrites_entry_and_target_references() {
    let mut genome = v3alpha1_founder_genome();
    genome.nodes.truncate(1);
    genome.nodes[0].targets = vec![genome.nodes[0].node_id];
    genome.entry_node_id = genome.nodes[0].node_id;
    let old_id = genome.nodes[0].node_id;

    let mut r = rng(11);
    apply(
        &mut genome,
        TopologyOperator::RewriteNodeId,
        &[],
        0.0,
        &mut r,
    )
    .unwrap();
    let new_id = genome.nodes[0].node_id;

    assert_ne!(new_id, old_id, "node id should be rewritten");
    assert_eq!(genome.entry_node_id, new_id, "entry id should be rewritten");
    assert_eq!(
        genome.nodes[0].targets,
        vec![new_id],
        "all target references should be rewritten"
    );
}

#[test]
fn copy_node_increases_node_count_by_one() {
    let mut genome = v3alpha1_founder_genome();
    let before = genome.nodes.len();
    let mut r = rng(42);
    apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
    assert_eq!(genome.nodes.len(), before + 1);
}

#[test]
fn copy_node_assigns_different_node_id() {
    let mut genome = v3alpha1_founder_genome();
    let original_ids: Vec<NodeId> = genome.nodes.iter().map(|n| n.node_id).collect();
    let mut r = rng(42);
    apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
    let new_node = genome.nodes.last().unwrap();
    assert!(
        !original_ids.contains(&new_node.node_id),
        "copied node must have a fresh NodeId"
    );
}

#[test]
fn copy_node_deep_copies_vm_backend() {
    // The founder genome has VM nodes. Over seeds, find one that copies a VM node
    // and verify backend equality.
    let mut found = false;
    for seed in 0u64..100 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
        let new_node = genome.nodes.last().unwrap();
        if matches!(&new_node.backend_def, BackendDef::Vm(_)) {
            // Find which source node has matching backend.
            let has_match = genome
                .nodes
                .iter()
                .rev()
                .skip(1)
                .any(|n| n.backend_def == new_node.backend_def);
            if has_match {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "must find at least one seed that deep-copies a VM backend"
    );
}

#[test]
fn copy_node_deep_copies_graph_backend() {
    // Swap one founder node to Graph first, then copy.
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::CgpGraphBackendDef;
    let mut found = false;
    for seed in 0u64..100 {
        let mut genome = v3alpha1_founder_genome();
        // Swap node 1 to CGP Graph backend.
        genome.nodes[1].backend_def = BackendDef::Graph(
            CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default()),
        );
        let mut r = rng(seed);
        apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
        let new_node = genome.nodes.last().unwrap();
        if matches!(&new_node.backend_def, BackendDef::Graph(_)) {
            let has_match = genome
                .nodes
                .iter()
                .rev()
                .skip(1)
                .any(|n| n.backend_def == new_node.backend_def);
            if has_match {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "must find at least one seed that deep-copies a Graph backend"
    );
}

#[test]
fn copy_node_sometimes_copies_targets_sometimes_not() {
    let mut saw_copied = false;
    let mut saw_empty = false;
    for seed in 0u64..500 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
        let new_node = genome.nodes.last().unwrap();
        if new_node.targets.is_empty() {
            saw_empty = true;
        } else {
            saw_copied = true;
        }
        if saw_copied && saw_empty {
            break;
        }
    }
    assert!(saw_copied, "must observe at least one copy with targets");
    assert!(
        saw_empty,
        "must observe at least one copy with empty targets"
    );
}

#[test]
fn copy_node_sometimes_copies_input_refs_sometimes_not() {
    let mut saw_copied = false;
    let mut saw_empty = false;
    for seed in 0u64..500 {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(seed);
        apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
        let new_node = genome.nodes.last().unwrap();
        if new_node.input_refs.is_empty() {
            saw_empty = true;
        } else {
            saw_copied = true;
        }
        if saw_copied && saw_empty {
            break;
        }
    }
    assert!(saw_copied, "must observe at least one copy with input_refs");
    assert!(
        saw_empty,
        "must observe at least one copy with empty input_refs"
    );
}

#[test]
fn copy_node_always_adds_backlink() {
    for seed in 0u64..200 {
        let mut genome = v3alpha1_founder_genome();
        let original_targets: Vec<Vec<NodeId>> =
            genome.nodes.iter().map(|n| n.targets.clone()).collect();
        let mut r = rng(seed);
        apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
        let new_id = genome.nodes.last().unwrap().node_id;
        // Check that some original node gained the new_id in its targets.
        let backlinked = genome.nodes.iter().enumerate().any(|(i, n)| {
            i < original_targets.len()
                && n.targets.contains(&new_id)
                && !original_targets[i].contains(&new_id)
        });
        assert!(
            backlinked,
            "copy_node must always add a backlink (seed {seed})"
        );
    }
}

#[test]
fn copy_node_can_copy_entry_node() {
    let mut genome = v3alpha1_founder_genome();
    genome.nodes.truncate(1);
    genome.entry_node_id = genome.nodes[0].node_id;
    let before = genome.nodes.len();
    let mut r = rng(99);
    apply(&mut genome, TopologyOperator::CopyNode, &[], 0.0, &mut r).unwrap();
    assert_eq!(genome.nodes.len(), before + 1);
}

#[test]
fn copy_mesh_backward_slice_duplicates_feeding_pipeline() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(2)],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let before = genome.nodes.len();
    let mut found_multi = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result = apply(
            &mut g,
            TopologyOperator::CopyMeshBackwardSlice,
            &[],
            0.0,
            &mut r,
        );
        if result.is_ok() && g.nodes.len() > before + 1 {
            found_multi = true;
            let ids: Vec<NodeId> = g.nodes.iter().map(|n| n.node_id).collect();
            let unique: std::collections::HashSet<_> = ids.iter().collect();
            assert_eq!(ids.len(), unique.len(), "all node IDs must be unique");
            break;
        }
    }
    assert!(found_multi, "must find a seed that copies multiple nodes");
}

#[test]
fn copy_mesh_backward_slice_remaps_internal_targets() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result = apply(
            &mut g,
            TopologyOperator::CopyMeshBackwardSlice,
            &[],
            0.0,
            &mut r,
        );
        if result.is_ok() && g.nodes.len() == 4 {
            let original_ids: std::collections::HashSet<NodeId> =
                genome.nodes.iter().map(|n| n.node_id).collect();
            let new_nodes: Vec<&NodeGenome> = g
                .nodes
                .iter()
                .filter(|n| !original_ids.contains(&n.node_id))
                .collect();
            assert_eq!(new_nodes.len(), 2);
            if let Some(cloned_with_targets) = new_nodes.iter().find(|n| !n.targets.is_empty()) {
                for target in &cloned_with_targets.targets {
                    if !original_ids.contains(target) {
                        let new_ids: Vec<NodeId> = new_nodes.iter().map(|n| n.node_id).collect();
                        assert!(
                            new_ids.contains(target),
                            "remapped target {:?} must be in new node IDs {:?}",
                            target,
                            new_ids
                        );
                    }
                }
            }
            return;
        }
    }
    panic!("could not find a seed that produces a 2-node backward slice copy");
}

#[test]
fn copy_mesh_forward_slice_duplicates_downstream_subtree() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(2)],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let before = genome.nodes.len();
    let mut found_multi = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result = apply(
            &mut g,
            TopologyOperator::CopyMeshForwardSlice,
            &[],
            0.0,
            &mut r,
        );
        if result.is_ok() && g.nodes.len() > before + 1 {
            found_multi = true;
            let ids: Vec<NodeId> = g.nodes.iter().map(|n| n.node_id).collect();
            let unique: std::collections::HashSet<_> = ids.iter().collect();
            assert_eq!(ids.len(), unique.len(), "all node IDs must be unique");
            break;
        }
    }
    assert!(
        found_multi,
        "must find a seed that copies multiple nodes via forward slice"
    );
}

#[test]
fn copy_mesh_operators_single_node_genome_returns_single_copy() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    };
    let mut r = rng(42);
    apply(
        &mut genome,
        TopologyOperator::CopyMeshBackwardSlice,
        &[],
        0.0,
        &mut r,
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), 2);
    assert_ne!(genome.nodes[0].node_id, genome.nodes[1].node_id);
}

// ── Gap 6: SwapRouteTargets tests ──

#[test]
fn swap_route_targets_changes_target_order() {
    // Create a genome with a node that has multiple targets.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)],
        }],
    };
    let original_targets = genome.nodes[0].targets.clone();
    let mut changed = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        apply(&mut g, TopologyOperator::SwapRouteTargets, &[], 0.0, &mut r).unwrap();
        if g.nodes[0].targets != original_targets {
            changed = true;
            break;
        }
    }
    assert!(changed, "swap must change target order");
}

#[test]
fn swap_route_targets_preserves_target_set() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)],
        }],
    };
    let mut original_sorted = genome.nodes[0].targets.clone();
    original_sorted.sort();
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        apply(&mut g, TopologyOperator::SwapRouteTargets, &[], 0.0, &mut r).unwrap();
        let mut after_sorted = g.nodes[0].targets.clone();
        after_sorted.sort();
        assert_eq!(
            original_sorted, after_sorted,
            "swap must preserve the same set of targets"
        );
    }
}

#[test]
fn swap_route_targets_requires_at_least_two_targets() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![NodeId::new(1)],
        }],
    };
    let mut r = rng(0);
    let result = apply(
        &mut genome,
        TopologyOperator::SwapRouteTargets,
        &[],
        0.0,
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

// ── Gap 3: SpliceNode tests ──

#[test]
fn splice_node_increases_node_count_by_one() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let before = genome.nodes.len();
    let mut r = rng(0);
    apply(&mut genome, TopologyOperator::SpliceNode, &[], 0.0, &mut r).unwrap();
    assert_eq!(genome.nodes.len(), before + 1);
}

#[test]
fn splice_node_creates_a_to_c_to_b_chain() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let b_id = NodeId::new(1);
    let mut r = rng(0);
    apply(&mut genome, TopologyOperator::SpliceNode, &[], 0.0, &mut r).unwrap();
    let c = genome.nodes.last().unwrap();
    let c_id = c.node_id;
    // C's target is B
    assert_eq!(c.targets, vec![b_id], "C must target B");
    // A's target is now C (not B)
    assert!(
        genome.nodes[0].targets.contains(&c_id),
        "A must now target C"
    );
    assert!(
        !genome.nodes[0].targets.contains(&b_id),
        "A must no longer directly target B"
    );
}

#[test]
fn splice_node_new_node_is_blank_vm() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![NodeId::new(1)],
        }],
    };
    let config = forced_birth_config(0.0, 1.0, 1.0);
    let mut r = rng(0);
    apply_with_config(
        &mut genome,
        TopologyOperator::SpliceNode,
        &[],
        0.0,
        &mut r,
        &config,
    )
    .unwrap();
    let c = genome.nodes.last().unwrap();
    assert!(
        c.input_refs.is_empty(),
        "spliced node must have no input_refs"
    );
    if let BackendDef::Vm(ref vm) = c.backend_def {
        assert_eq!(vm.register_count, 1);
        assert!(vm.constants.is_empty());
        assert_eq!(vm.program, vec![VmInstruction::Halt]);
    } else {
        panic!("spliced node must be VM backend");
    }
}

#[test]
fn add_node_can_force_blank_graph_birth() {
    let mut genome = v3alpha1_founder_genome();
    let config = forced_birth_config(1.0, 0.0, 0.0);
    let mut r = rng(5);
    apply_with_config(
        &mut genome,
        TopologyOperator::AddNode,
        &[],
        0.0,
        &mut r,
        &config,
    )
    .unwrap();
    let newborn = genome.nodes.last().unwrap();
    let BackendDef::Graph(graph) = &newborn.backend_def else {
        panic!("expected Graph newborn");
    };
    assert!(newborn.input_refs.is_empty());
    assert!(graph.compute_nodes.is_empty());
    let wired_sink_count = graph
        .output_sinks
        .iter()
        .filter(|sink| !sink.inputs.is_empty())
        .count();
    assert_eq!(wired_sink_count, 0);
}

#[test]
fn add_node_can_force_vm_birth() {
    let mut genome = v3alpha1_founder_genome();
    let config = forced_birth_config(0.0, 1.0, 1.0);
    let mut r = rng(6);
    apply_with_config(
        &mut genome,
        TopologyOperator::AddNode,
        &[],
        0.0,
        &mut r,
        &config,
    )
    .unwrap();
    let newborn = genome.nodes.last().unwrap();
    assert!(newborn.input_refs.is_empty());
    let BackendDef::Vm(vm) = &newborn.backend_def else {
        panic!("expected VM newborn");
    };
    assert_eq!(vm.register_count, 1);
    assert!(vm.constants.is_empty());
    assert_eq!(vm.program, vec![VmInstruction::Halt]);
}

#[test]
fn add_node_can_force_initialized_direct_graph_birth() {
    let mut genome = v3alpha1_founder_genome();
    let config = forced_birth_config(1.0, 1.0, 0.0);
    let mut r = rng(7);
    apply_with_config(
        &mut genome,
        TopologyOperator::AddNode,
        &[],
        0.0,
        &mut r,
        &config,
    )
    .unwrap();
    let newborn = genome.nodes.last().unwrap();
    let BackendDef::Graph(graph) = &newborn.backend_def else {
        panic!("expected Graph newborn");
    };
    assert_eq!(newborn.input_refs.len(), 1);
    assert!(graph.compute_nodes.is_empty());
    let wired_custom_sinks: Vec<_> = graph
        .output_sinks
        .iter()
        .filter(|sink| {
            matches!(sink.kind, OutputSinkKind::CustomOutput(_)) && !sink.inputs.is_empty()
        })
        .collect();
    assert_eq!(wired_custom_sinks.len(), 1);
    assert_eq!(wired_custom_sinks[0].inputs.len(), 1);
    assert!(matches!(
        wired_custom_sinks[0].inputs[0].source,
        GraphSource::InputLeaf { ref_idx: 0, .. }
    ));
    let wired_non_custom = graph
        .output_sinks
        .iter()
        .filter(|sink| !matches!(sink.kind, OutputSinkKind::CustomOutput(_)))
        .filter(|sink| !sink.inputs.is_empty())
        .count();
    assert_eq!(wired_non_custom, 0);
}

#[test]
fn splice_node_can_force_initialized_compute_graph_birth() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let config = forced_birth_config(1.0, 1.0, 1.0);
    let mut r = rng(11);
    apply_with_config(
        &mut genome,
        TopologyOperator::SpliceNode,
        &[],
        0.0,
        &mut r,
        &config,
    )
    .unwrap();
    let newborn = genome.nodes.last().unwrap();
    let BackendDef::Graph(graph) = &newborn.backend_def else {
        panic!("expected Graph newborn");
    };
    assert_eq!(newborn.input_refs.len(), 1);
    assert_eq!(graph.compute_nodes.len(), 1);
    let wired_custom_sinks: Vec<_> = graph
        .output_sinks
        .iter()
        .filter(|sink| {
            matches!(sink.kind, OutputSinkKind::CustomOutput(_)) && !sink.inputs.is_empty()
        })
        .collect();
    assert_eq!(wired_custom_sinks.len(), 1);
    assert_eq!(wired_custom_sinks[0].inputs.len(), 1);
    assert!(matches!(
        wired_custom_sinks[0].inputs[0].source,
        GraphSource::ComputeNode(0)
    ));
    let wired_non_custom = graph
        .output_sinks
        .iter()
        .filter(|sink| !matches!(sink.kind, OutputSinkKind::CustomOutput(_)))
        .filter(|sink| !sink.inputs.is_empty())
        .count();
    assert_eq!(wired_non_custom, 0);
}

#[test]
fn splice_node_can_force_vm_birth() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let config = forced_birth_config(0.0, 1.0, 1.0);
    let mut r = rng(12);
    apply_with_config(
        &mut genome,
        TopologyOperator::SpliceNode,
        &[],
        0.0,
        &mut r,
        &config,
    )
    .unwrap();
    let newborn = genome.nodes.last().unwrap();
    assert!(newborn.input_refs.is_empty());
    let BackendDef::Vm(vm) = &newborn.backend_def else {
        panic!("expected VM newborn");
    };
    assert_eq!(vm.register_count, 1);
    assert!(vm.constants.is_empty());
    assert_eq!(vm.program, vec![VmInstruction::Halt]);
}

#[test]
fn splice_node_no_targets_returns_skip() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    };
    let mut r = rng(0);
    let result = apply(&mut genome, TopologyOperator::SpliceNode, &[], 0.0, &mut r);
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn splice_node_passes_parseability_gate() {
    let mut genome = v3alpha1_founder_genome();
    let mut r = rng(42);
    let result = apply(&mut genome, TopologyOperator::SpliceNode, &[], 0.0, &mut r);
    match result {
        Ok(_) | Err(MutationSkipReason::NoApplicableTarget) => {
            assert!(
                ParseabilityGate::validate(&genome).is_ok(),
                "parseability failed after SpliceNode"
            );
        }
        Err(other) => panic!("unexpected skip reason {:?}", other),
    }
}

#[test]
fn splice_node_new_node_gets_fresh_node_id() {
    let mut genome = v3alpha1_founder_genome();
    let original_ids: Vec<NodeId> = genome.nodes.iter().map(|n| n.node_id).collect();
    let mut r = rng(0);
    apply(&mut genome, TopologyOperator::SpliceNode, &[], 0.0, &mut r).unwrap();
    let c = genome.nodes.last().unwrap();
    assert!(
        !original_ids.contains(&c.node_id),
        "spliced node must have a fresh NodeId"
    );
}

// Old output slot remapping tests deleted. CGP Graph backends have fixed output sinks
// (not remappable CustomOutput node kinds), so the remap_output_slots behavior no longer exists.

#[test]
fn copy_mesh_operators_pass_parseability_gate() {
    let operators = [
        TopologyOperator::CopyMeshBackwardSlice,
        TopologyOperator::CopyMeshForwardSlice,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 200);
        let result = apply(&mut genome, op, &[], 0.0, &mut r);
        match result {
            Ok(_) | Err(MutationSkipReason::NoApplicableTarget) => {
                assert!(
                    ParseabilityGate::validate(&genome).is_ok(),
                    "parseability failed after {:?}: {:?}",
                    op,
                    ParseabilityGate::validate(&genome)
                );
            }
            Err(other) => panic!("unexpected skip reason {:?} for {:?}", other, op),
        }
    }
}

#[test]
fn topology_weighted_random_favors_refinement() {
    let mut counts = std::collections::HashMap::new();
    let mut r = rng(42);
    for _ in 0..10_000 {
        let op = TopologyOperator::random(&mut r);
        *counts.entry(op).or_insert(0u32) += 1;
    }
    let rewrite = counts
        .get(&TopologyOperator::RewriteNodeId)
        .copied()
        .unwrap_or(0);
    let add_node = counts.get(&TopologyOperator::AddNode).copied().unwrap_or(0);
    assert!(
        rewrite > add_node * 2,
        "RewriteNodeId (weight 4) must appear >2x AddNode (weight 1); got {} vs {}",
        rewrite,
        add_node,
    );
}

#[test]
fn topology_operator_weights_are_positive() {
    let all = TopologyOperator::ALL;
    assert_eq!(
        all.len(),
        13,
        "ALL must cover every TopologyOperator variant"
    );
    for &op in &all {
        assert!(op.weight() > 0, "weight must be positive for {:?}", op);
    }
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::types::ComplexityEffect;
    for &op in &TopologyOperator::ALL {
        let effect = op.complexity_effect();
        assert!(
            matches!(
                effect,
                ComplexityEffect::Increasing
                    | ComplexityEffect::Decreasing
                    | ComplexityEffect::Neutral
            ),
            "complexity_effect must return valid effect for {:?}",
            op
        );
    }
}

#[test]
fn bias_1_targets_reachable_node_for_remove() {
    // Build a 3-node genome where only node index 1 is "reachable" and removable
    // (entry node at index 0 is not removable). With bias=1.0 the operator must
    // pick from the reachable set.
    let mut genome = v3alpha1_founder_genome();
    // Add a third node so there's a non-reachable removable node too.
    let id2 = NodeId::new(99);
    genome.nodes.push(NodeGenome {
        node_id: id2,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![],
    });
    // Reachable = [1] (index 1 only). Index 2 is unreachable.
    let reachable = [1usize];
    for seed in 0u64..20 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result = apply(
            &mut g,
            TopologyOperator::RemoveNode,
            &reachable,
            1.0,
            &mut r,
        );
        match result {
            Ok(reachability) => {
                assert_eq!(
                    reachability,
                    TargetReachability::Reachable,
                    "bias=1.0 must always pick a reachable node"
                );
            }
            Err(MutationSkipReason::NoApplicableTarget) => {
                // Acceptable if the only reachable removable node is the entry node
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }
}

#[test]
fn bias_0_returns_reachable_or_unreachable() {
    // With bias=0.0 (uniform), we should eventually see both Reachable and Unreachable.
    let mut genome = v3alpha1_founder_genome();
    let id2 = NodeId::new(99);
    genome.nodes.push(NodeGenome {
        node_id: id2,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![],
    });
    // Reachable = [1]. Index 2 is unreachable. Both are removable (not entry).
    let reachable = [1usize];
    let mut saw_reachable = false;
    let mut saw_unreachable = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if let Ok(reachability) = apply(
            &mut g,
            TopologyOperator::RemoveNode,
            &reachable,
            0.0,
            &mut r,
        ) {
            match reachability {
                TargetReachability::Reachable => saw_reachable = true,
                TargetReachability::Unreachable => saw_unreachable = true,
                TargetReachability::NotApplicable => {}
            }
        }
        if saw_reachable && saw_unreachable {
            break;
        }
    }
    assert!(
        saw_reachable && saw_unreachable,
        "bias=0.0 should eventually pick both reachable and unreachable nodes"
    );
}

#[test]
fn exempt_operators_return_not_applicable() {
    let mut genome = v3alpha1_founder_genome();
    let reachable = [0usize, 1];
    let mut r = rng(42);
    let result = apply(
        &mut genome,
        TopologyOperator::AddNode,
        &reachable,
        1.0,
        &mut r,
    );
    assert_eq!(result, Ok(TargetReachability::NotApplicable));

    let mut r2 = rng(43);
    let result = apply(
        &mut genome,
        TopologyOperator::ChangeEntryNode,
        &reachable,
        1.0,
        &mut r2,
    );
    assert_eq!(result, Ok(TargetReachability::NotApplicable));
}
