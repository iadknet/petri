use super::*;
use crate::config::MutationConfig;
use crate::contracts::RouteTarget;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::OutputSinkKind;
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::creature::parseability::ParseabilityGate;
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn wrap_targets(ids: Vec<NodeId>) -> Vec<RouteTarget> {
    ids.into_iter()
        .enumerate()
        .map(|(i, id)| RouteTarget {
            target_id: id,
            slot: i as u8,
            gate_bias: 0.0,
        })
        .collect()
}

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn apply(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl rand::Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    apply_with_config(genome, op, targets, rng, &MutationConfig::default())
}

fn apply_with_config(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl rand::Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    TopologyMutator::apply(genome, op, targets, rng, config)
}

#[test]
fn add_node_increases_node_count_by_one() {
    let mut genome = v3alpha1_founder_genome();
    let before = genome.nodes.len();
    let mut r = rng(0);
    apply(
        &mut genome,
        TopologyOperator::AddNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), before + 1);
}

#[test]
fn remove_node_decreases_node_count() {
    let mut genome = v3alpha1_founder_genome();
    let mut unused = genome.nodes[1].clone();
    unused.node_id = NodeId::new(99);
    genome.nodes.push(unused);
    assert!(genome.nodes.len() >= 2, "founder must have >=2 nodes");
    let before = genome.nodes.len();
    let mut r = rng(1);
    apply(
        &mut genome,
        TopologyOperator::RemoveNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), before - 1);
}

#[test]
fn remove_node_on_single_node_genome_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    // Reduce to single node.
    genome.nodes.truncate(1);
    genome.entry_node_id = genome.nodes[0].node_id;
    let mut r = rng(2);
    let result = apply(
        &mut genome,
        TopologyOperator::RemoveNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn retarget_node_target_skips_without_local_alternative() {
    let mut genome = v3alpha1_founder_genome();
    assert_eq!(
        apply(
            &mut genome,
            TopologyOperator::RetargetNodeTarget,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(99)
        ),
        Err(MutationSkipReason::NoApplicableTarget)
    );
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
    assert_eq!(after, before_total + 2);
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
        TopologyOperator::CopyNode,
        TopologyOperator::CopyMeshBackwardSlice,
        TopologyOperator::CopyMeshForwardSlice,
        TopologyOperator::SpliceNode,
        TopologyOperator::SwapRouteTargets,
        TopologyOperator::MutateGateBias,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 100);
        let result = apply(
            &mut genome,
            op,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        );
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
fn copy_node_increases_node_count_by_one() {
    let mut genome = v3alpha1_founder_genome();
    let before = genome.nodes.len();
    let mut r = rng(42);
    apply(
        &mut genome,
        TopologyOperator::CopyNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), before + 1);
}

#[test]
fn copy_node_assigns_different_node_id() {
    let mut genome = v3alpha1_founder_genome();
    let original_ids: Vec<NodeId> = genome.nodes.iter().map(|n| n.node_id).collect();
    let mut r = rng(42);
    apply(
        &mut genome,
        TopologyOperator::CopyNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
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
        apply(
            &mut genome,
            TopologyOperator::CopyNode,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        )
        .unwrap();
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
        apply(
            &mut genome,
            TopologyOperator::CopyNode,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        )
        .unwrap();
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
                targets: wrap_targets(vec![NodeId::new(1)]),
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: wrap_targets(vec![NodeId::new(2)]),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
                targets: wrap_targets(vec![NodeId::new(1)]),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
                    if !original_ids.contains(&target.target_id) {
                        let new_ids: Vec<NodeId> = new_nodes.iter().map(|n| n.node_id).collect();
                        assert!(
                            new_ids.contains(&target.target_id),
                            "remapped target {:?} must be in new node IDs {:?}",
                            target.target_id,
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
                targets: wrap_targets(vec![NodeId::new(1)]),
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: wrap_targets(vec![NodeId::new(2)]),
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
            &mut TargetSelector::reachable_only(&[], 0.0),
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
        &mut TargetSelector::reachable_only(&[], 0.0),
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
            targets: wrap_targets(vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)]),
        }],
    };
    let original_targets = genome.nodes[0].targets.clone();
    let mut changed = false;
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        apply(
            &mut g,
            TopologyOperator::SwapRouteTargets,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        )
        .unwrap();
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
            targets: wrap_targets(vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)]),
        }],
    };
    let mut original_ids: Vec<NodeId> = genome.nodes[0]
        .targets
        .iter()
        .map(|t| t.target_id)
        .collect();
    original_ids.sort();
    for seed in 0u64..50 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        apply(
            &mut g,
            TopologyOperator::SwapRouteTargets,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        )
        .unwrap();
        let mut after_ids: Vec<NodeId> = g.nodes[0].targets.iter().map(|t| t.target_id).collect();
        after_ids.sort();
        assert_eq!(
            original_ids, after_ids,
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
            targets: wrap_targets(vec![NodeId::new(1)]),
        }],
    };
    let mut r = rng(0);
    let result = apply(
        &mut genome,
        TopologyOperator::SwapRouteTargets,
        &mut TargetSelector::reachable_only(&[], 0.0),
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
                targets: wrap_targets(vec![NodeId::new(1)]),
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
    apply(
        &mut genome,
        TopologyOperator::SpliceNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
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
                targets: wrap_targets(vec![NodeId::new(1)]),
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
    apply(
        &mut genome,
        TopologyOperator::SpliceNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
    let c = genome.nodes.last().unwrap();
    let c_id = c.node_id;
    // C's target is B
    assert_eq!(c.targets.len(), 1, "C must have one target");
    assert_eq!(c.targets[0].target_id, b_id, "C must target B");
    // A's target is now C (not B)
    assert!(
        genome.nodes[0].targets.iter().any(|t| t.target_id == c_id),
        "A must now target C"
    );
    assert!(
        !genome.nodes[0].targets.iter().any(|t| t.target_id == b_id),
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
            targets: wrap_targets(vec![NodeId::new(1)]),
        }],
    };
    genome.nodes.push(crate::mutation::topology::birth::detour(
        NodeId::new(1),
        NodeId::new(0),
    ));
    let config = MutationConfig::default();
    let mut r = rng(0);
    apply_with_config(
        &mut genome,
        TopologyOperator::SpliceNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
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
    let result = apply(
        &mut genome,
        TopologyOperator::SpliceNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn splice_node_passes_parseability_gate() {
    let mut genome = v3alpha1_founder_genome();
    let mut r = rng(42);
    let result = apply(
        &mut genome,
        TopologyOperator::SpliceNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    );
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
    apply(
        &mut genome,
        TopologyOperator::SpliceNode,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    )
    .unwrap();
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
        let result = apply(
            &mut genome,
            op,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        );
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
        .get(&TopologyOperator::MutateGateBias)
        .copied()
        .unwrap_or(0);
    let add_node = counts.get(&TopologyOperator::AddNode).copied().unwrap_or(0);
    assert!(
        rewrite > add_node * 2,
        "MutateGateBias (weight 4) must appear >2x AddNode (weight 1); got {} vs {}",
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
fn removal_prefers_unreachable_even_with_reachable_bias() {
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
            &mut TargetSelector::reachable_only(&reachable, 1.0),
            &mut r,
        );
        match result {
            Ok(reachability) => {
                assert_eq!(
                    reachability,
                    TargetReachability::Unreachable,
                    "unreachable removal is preferred"
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
fn removal_prefers_unreachable_at_uniform_bias() {
    for seed in 0..20 {
        let mut genome = v3alpha1_founder_genome();
        let mut unused = genome.nodes[1].clone();
        unused.node_id = NodeId::new(99);
        genome.nodes.push(unused);
        assert_eq!(
            apply(
                &mut genome,
                TopologyOperator::RemoveNode,
                &mut TargetSelector::reachable_only(&[0, 1], 0.0),
                &mut rng(seed)
            ),
            Ok(TargetReachability::Unreachable)
        );
        assert_eq!(genome, v3alpha1_founder_genome());
    }
}

#[test]
fn exempt_operators_return_not_applicable() {
    let mut genome = v3alpha1_founder_genome();
    let reachable = [0usize, 1];
    let mut r = rng(42);
    let result = apply(
        &mut genome,
        TopologyOperator::AddNode,
        &mut TargetSelector::reachable_only(&reachable, 1.0),
        &mut r,
    );
    assert_eq!(result, Ok(TargetReachability::Reachable));

    let mut r2 = rng(43);
    let result = apply(
        &mut genome,
        TopologyOperator::ChangeEntryNode,
        &mut TargetSelector::reachable_only(&reachable, 1.0),
        &mut r2,
    );
    assert_eq!(result, Ok(TargetReachability::NotApplicable));
}

// ── MutateGateBias tests ──

#[test]
fn mutate_gate_bias_changes_bias() {
    use crate::contracts::RouteTarget;
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(crate::creature::genome::VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![crate::creature::genome::VmInstruction::Halt],
            }),
            targets: vec![RouteTarget {
                target_id: NodeId::new(0),
                slot: 0,
                gate_bias: 0.0,
            }],
        }],
    };
    let mut r = rng(42);
    let result = apply(
        &mut genome,
        TopologyOperator::MutateGateBias,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    );
    assert!(result.is_ok());
    let bias = genome.nodes[0].targets[0].gate_bias;
    assert_ne!(bias, 0.0, "gate_bias must have changed");
    assert!(
        (-4.0..=4.0).contains(&bias),
        "gate_bias {bias} must be within [-4.0, 4.0]"
    );
}

#[test]
fn mutate_gate_bias_noop_on_empty_targets() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(crate::creature::genome::VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![crate::creature::genome::VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    };
    let mut r = rng(0);
    let result = apply(
        &mut genome,
        TopologyOperator::MutateGateBias,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn mutate_gate_bias_clamps_to_range() {
    use crate::contracts::RouteTarget;
    // Set gate_bias near the upper boundary.
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(crate::creature::genome::VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![crate::creature::genome::VmInstruction::Halt],
            }),
            targets: vec![RouteTarget {
                target_id: NodeId::new(0),
                slot: 0,
                gate_bias: 3.9,
            }],
        }],
    };
    // Apply multiple times -- at least one should push toward the boundary.
    let mut clamped_high = false;
    let mut clamped_low = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = apply(
            &mut g,
            TopologyOperator::MutateGateBias,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        );
        let bias = g.nodes[0].targets[0].gate_bias;
        assert!(
            (-4.0..=4.0).contains(&bias),
            "gate_bias {bias} out of range after seed {seed}"
        );
        if (bias - 4.0).abs() < f32::EPSILON {
            clamped_high = true;
        }
    }
    // Also test clamping at the low end.
    genome.nodes[0].targets[0].gate_bias = -3.9;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let _ = apply(
            &mut g,
            TopologyOperator::MutateGateBias,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
        );
        let bias = g.nodes[0].targets[0].gate_bias;
        assert!(
            (-4.0..=4.0).contains(&bias),
            "gate_bias {bias} out of range after seed {seed}"
        );
        if (bias - (-4.0)).abs() < f32::EPSILON {
            clamped_low = true;
        }
    }
    assert!(
        clamped_high,
        "expected at least one high-boundary clamp across 200 seeds"
    );
    assert!(
        clamped_low,
        "expected at least one low-boundary clamp across 200 seeds"
    );
}

// --- Cross-process reproducibility of the mesh-slice clone (T10.F11) ---

/// A ring of `n` VM nodes, each targeting the next at slot 0. Every backward
/// slice of a ring is the whole ring whatever anchor is drawn, so the clone
/// always has `n` candidates for its backlink target, and slot 1 is free on
/// every node so the backlink is always added.
fn ring_genome(n: u32) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: (0..n)
            .map(|i| NodeGenome {
                node_id: NodeId::new(i),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: wrap_targets(vec![NodeId::new((i + 1) % n)]),
            })
            .collect(),
    }
}

#[test]
fn copy_mesh_backward_slice_is_reproducible_for_a_seed() {
    let genome = ring_genome(4);
    let target_count =
        |g: &CreatureGenome| -> usize { g.nodes.iter().map(|n| n.targets.len()).sum() };
    let results: Vec<CreatureGenome> = (0..16)
        .map(|_| {
            let mut mutated = genome.clone();
            let mut r = rng(0x5EED_0F11);
            apply(
                &mut mutated,
                TopologyOperator::CopyMeshBackwardSlice,
                &mut TargetSelector::reachable_only(&[], 0.0),
                &mut r,
            )
            .expect("a ring genome always has a backward slice to clone");
            mutated
        })
        .collect();
    for (i, mutated) in results.iter().enumerate() {
        assert_eq!(
            mutated.nodes.len(),
            8,
            "application {i} must append the whole four-node ring"
        );
        assert_eq!(
            target_count(mutated),
            target_count(&genome) + 4 + 1,
            "application {i} must add the backlink on top of the cloned ring's targets"
        );
        assert_eq!(
            mutated, &results[0],
            "application {i} produced a different clone than application 0 for the same \
             seed: the backlink candidate order is not a function of the genome"
        );
    }
}

// T11.F15 behavioral red fixtures, before implementation.
#[test]
fn f15_inline_growth_redirects_existing_edge_through_halt() {
    for op in [TopologyOperator::AddNode, TopologyOperator::SpliceNode] {
        let mut genome = v3alpha1_founder_genome();
        let old = genome.nodes[0].targets[0];
        apply(
            &mut genome,
            op,
            &mut TargetSelector::reachable_only(&[0, 1], 0.5),
            &mut rng(7),
        )
        .unwrap();
        let new = genome.nodes.last().unwrap();
        assert_eq!(genome.nodes[0].targets[0].target_id, new.node_id);
        assert_eq!(genome.nodes[0].targets[0].slot, old.slot);
        assert_eq!(genome.nodes[0].targets[0].gate_bias, old.gate_bias);
        assert_eq!(new.targets[0].target_id, old.target_id);
        assert!(
            matches!(&new.backend_def, BackendDef::Vm(vm) if vm.program == vec![VmInstruction::Halt])
        );
    }
}

#[test]
fn f15_founder_connection_removals_and_local_retarget_skip_atomically() {
    for op in [
        TopologyOperator::RemoveNode,
        TopologyOperator::RemoveRouteTarget,
        TopologyOperator::RetargetNodeTarget,
    ] {
        let mut genome = v3alpha1_founder_genome();
        let before = genome.clone();
        assert_eq!(
            apply(
                &mut genome,
                op,
                &mut TargetSelector::reachable_only(&[0, 1], 0.5),
                &mut rng(7)
            ),
            Err(MutationSkipReason::NoApplicableTarget)
        );
        assert_eq!(genome, before);
    }
}

#[test]
fn f15_branch_addition_pairs_a_gate_write_with_tied_detour() {
    let mut genome = v3alpha1_founder_genome();
    let old = genome.nodes[0].targets[0];
    apply(
        &mut genome,
        TopologyOperator::AddRouteTarget,
        &mut TargetSelector::reachable_only(&[0, 1], 0.5),
        &mut rng(7),
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), 3);
    let branch = genome.nodes[0].targets[1];
    assert_eq!(branch.gate_bias, old.gate_bias);
    assert_eq!(branch.target_id, genome.nodes[2].node_id);
    assert_eq!(genome.nodes[2].targets[0].target_id, old.target_id);
    match &genome.nodes[0].backend_def {
        BackendDef::Graph(g) => assert!(g.output_sinks.iter().any(
            |s| matches!(s.kind, OutputSinkKind::RouterGate(slot) if slot == branch.slot)
                && !s.inputs.is_empty()
        )),
        BackendDef::Vm(vm) => assert!(vm.program.iter().any(
            |i| matches!(i, VmInstruction::WriteRouteGate { slot, .. } if *slot == branch.slot)
        )),
    }
}

#[test]
fn f15_backend_growth_preserves_every_original_node() {
    let mut genome = v3alpha1_founder_genome();
    let before = genome.clone();
    apply(
        &mut genome,
        TopologyOperator::SwapNodeBackend,
        &mut TargetSelector::reachable_only(&[0, 1], 0.5),
        &mut rng(7),
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), 3);
    assert_eq!(genome.nodes[1], before.nodes[1]);
    assert_eq!(genome.nodes[0].backend_def, before.nodes[0].backend_def);
    assert_eq!(
        genome.nodes[0].targets[1].target_id,
        genome.nodes[2].node_id
    );
}
