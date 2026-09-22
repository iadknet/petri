use super::*;
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, NodeId, RouteTarget, StaticIntrospectionKey,
    WorldInputKey,
};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
    OutputSink, OutputSinkKind,
};
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef};
use rand::SeedableRng;
use std::collections::BTreeSet;

// ── VM helper tests ─────────────────────────────────────────────────

#[test]
fn register_write_returns_dst_for_alu() {
    let instr = VmInstruction::Add { dst: 3, a: 1, b: 2 };
    assert_eq!(vm_register_write(&instr), Some(3));
}

#[test]
fn register_write_returns_none_for_output() {
    let instr = VmInstruction::PushAction { action_type: 0 };
    assert_eq!(vm_register_write(&instr), None);
}

#[test]
fn register_write_returns_none_for_halt() {
    assert_eq!(vm_register_write(&VmInstruction::Halt), None);
}

#[test]
fn reg_bit_normal_range() {
    assert_eq!(vm_reg_bit(0), 1);
    assert_eq!(vm_reg_bit(1), 2);
    assert_eq!(vm_reg_bit(31), 1u32 << 31);
}

#[test]
fn reg_bit_out_of_range_returns_zero() {
    assert_eq!(vm_reg_bit(32), 0);
    assert_eq!(vm_reg_bit(255), 0);
}

#[test]
fn read_mask_alu_two_sources() {
    let instr = VmInstruction::Add { dst: 0, a: 1, b: 2 };
    assert_eq!(vm_register_read_mask(&instr), vm_reg_bit(1) | vm_reg_bit(2));
}

#[test]
fn read_mask_noop_is_zero() {
    assert_eq!(vm_register_read_mask(&VmInstruction::Noop), 0);
}

#[test]
fn read_mask_write_route_gate() {
    let instr = VmInstruction::WriteRouteGate { slot: 0, src: 5 };
    assert_eq!(vm_register_read_mask(&instr), vm_reg_bit(5));
}

#[test]
fn is_output_matches_side_effecting_writes() {
    assert!(vm_is_output_instruction(&VmInstruction::PushAction {
        action_type: 0
    }));
    assert!(vm_is_output_instruction(&VmInstruction::WriteRouteGate {
        slot: 0,
        src: 0
    }));
    assert!(vm_is_output_instruction(
        &VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1
        }
    ));
    assert!(!vm_is_output_instruction(&VmInstruction::Add {
        dst: 0,
        a: 1,
        b: 2
    }));
    assert!(!vm_is_output_instruction(&VmInstruction::Halt));
}

// ── VM backward slice tests ─────────────────────────────────────────

#[test]
fn backward_slice_traces_register_deps() {
    // r0 = LoadConst     (idx 0)
    // r1 = Add r0, r0    (idx 1)
    // WriteRoute r1      (idx 2, anchor)
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::Add { dst: 1, a: 0, b: 0 },
        VmInstruction::WriteRouteGate { slot: 0, src: 1 },
    ];
    let gene = vm_backward_slice(&program, 2).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn backward_slice_skips_unrelated_instructions() {
    // r0 = LoadConst     (idx 0)
    // r1 = LoadConst     (idx 1) — unrelated
    // r2 = Add r0, r0    (idx 2)
    // WriteRoute r2      (idx 3, anchor)
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        },
        VmInstruction::Add { dst: 2, a: 0, b: 0 },
        VmInstruction::WriteRouteGate { slot: 0, src: 2 },
    ];
    let gene = vm_backward_slice(&program, 3).unwrap();
    assert_eq!(gene.indices, vec![0, 2, 3]);
}

#[test]
fn backward_slice_returns_none_for_non_output_anchor() {
    let program = vec![VmInstruction::Add { dst: 0, a: 1, b: 2 }];
    assert_eq!(vm_backward_slice(&program, 0), None);
}

#[test]
fn backward_slice_returns_none_for_out_of_bounds() {
    let program = vec![VmInstruction::Halt];
    assert_eq!(vm_backward_slice(&program, 5), None);
}

#[test]
fn backward_slice_anchor_only_when_no_deps() {
    // PushAction reads no registers
    let program = vec![VmInstruction::PushAction { action_type: 1 }];
    let gene = vm_backward_slice(&program, 0).unwrap();
    assert_eq!(gene.indices, vec![0]);
}

// ── VM forward slice tests ──────────────────────────────────────────

#[test]
fn forward_slice_traces_downstream() {
    // r0 = LoadConst     (idx 0, seed)
    // r1 = Add r0, r0    (idx 1)
    // WriteRoute r1      (idx 2)
    // Halt               (idx 3)
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::Add { dst: 1, a: 0, b: 0 },
        VmInstruction::WriteRouteGate { slot: 0, src: 1 },
        VmInstruction::Halt,
    ];
    let gene = vm_forward_slice(&program, 0).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn forward_slice_skips_unrelated() {
    // r0 = LoadConst     (idx 0, seed)
    // r1 = LoadConst     (idx 1) — unrelated
    // r2 = Add r0, r0    (idx 2)
    // WriteRoute r1      (idx 3) — reads r1, not r0/r2
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        },
        VmInstruction::Add { dst: 2, a: 0, b: 0 },
        VmInstruction::WriteRouteGate { slot: 0, src: 1 },
    ];
    let gene = vm_forward_slice(&program, 0).unwrap();
    // Starts at 0, picks up 2 (reads r0), does NOT pick up 3 (reads r1, not in produced set)
    assert_eq!(gene.indices, vec![0, 2]);
}

#[test]
fn forward_slice_returns_none_for_non_writer() {
    let program = vec![VmInstruction::PushAction { action_type: 0 }];
    assert_eq!(vm_forward_slice(&program, 0), None);
}

#[test]
fn forward_slice_returns_none_for_out_of_bounds() {
    let program = vec![VmInstruction::Halt];
    assert_eq!(vm_forward_slice(&program, 5), None);
}

#[test]
fn forward_slice_seed_only_when_no_consumers() {
    // r0 = LoadConst — nothing reads r0 afterward
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::Halt,
    ];
    let gene = vm_forward_slice(&program, 0).unwrap();
    assert_eq!(gene.indices, vec![0]);
}

// ── VM random slice tests ───────────────────────────────────────────

#[test]
fn backward_slice_random_returns_none_for_no_outputs() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::Halt,
    ];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    assert_eq!(vm_backward_slice_random(&program, &mut rng), None);
}

#[test]
fn forward_slice_random_returns_none_for_no_writers() {
    let program = vec![VmInstruction::Halt, VmInstruction::Noop];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    assert_eq!(vm_forward_slice_random(&program, &mut rng), None);
}

#[test]
fn backward_slice_random_returns_some_for_valid_program() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
    ];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let gene = vm_backward_slice_random(&program, &mut rng).unwrap();
    assert_eq!(gene.indices, vec![0, 1]);
}

#[test]
fn forward_slice_random_returns_some_for_valid_program() {
    let program = vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
    ];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let gene = vm_forward_slice_random(&program, &mut rng).unwrap();
    assert_eq!(gene.indices, vec![0, 1]);
}

// ── Mesh reachability tests ─────────────────────────────────────────

fn simple_vm_node(id: u32, targets: Vec<u32>) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: targets
            .into_iter()
            .enumerate()
            .map(|(i, id)| RouteTarget {
                target_id: NodeId::new(id),
                slot: i as u8,
                gate_bias: 0.0,
            })
            .collect(),
    }
}

#[test]
fn mesh_reachable_both_nodes_in_founder_style() {
    // Node 0 -> Node 1, entry at node 0
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![])],
    };
    assert_eq!(mesh_reachable_nodes(&genome), vec![0, 1]);
}

#[test]
fn mesh_reachable_excludes_unreachable_node() {
    // Node 0 -> Node 1, Node 2 is unreachable
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![]),
            simple_vm_node(2, vec![]),
        ],
    };
    assert_eq!(mesh_reachable_nodes(&genome), vec![0, 1]);
}

#[test]
fn mesh_reachable_entry_only_when_no_targets() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![])],
    };
    assert_eq!(mesh_reachable_nodes(&genome), vec![0]);
}

#[test]
fn mesh_reachable_empty_genome() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![],
    };
    assert_eq!(mesh_reachable_nodes(&genome), Vec::<usize>::new());
}

#[test]
fn mesh_reachable_handles_dangling_target() {
    // Node 0 targets node 99 which doesn't exist
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![99])],
    };
    assert_eq!(mesh_reachable_nodes(&genome), vec![0]);
}

#[test]
fn mesh_reachable_handles_cycle() {
    // Node 0 -> Node 1 -> Node 0 (cycle)
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![0])],
    };
    assert_eq!(mesh_reachable_nodes(&genome), vec![0, 1]);
}

/// Cycle membership (T19.F02): reachable nodes that can route back to
/// themselves, self-targets included; unreachable cycles and dangling
/// targets never count.
#[test]
fn mesh_cycle_nodes_names_reachable_nodes_on_a_cycle() {
    let ids = |ids: &[u32]| ids.iter().map(|&i| NodeId::new(i)).collect::<BTreeSet<_>>();
    let genome = |nodes| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes,
    };
    assert!(mesh_cycle_nodes(&genome(vec![
        simple_vm_node(0, vec![1]),
        simple_vm_node(1, vec![])
    ]))
    .is_empty());
    assert_eq!(
        mesh_cycle_nodes(&genome(vec![simple_vm_node(0, vec![0])])),
        ids(&[0])
    );
    assert_eq!(
        mesh_cycle_nodes(&genome(vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2, 1]),
            simple_vm_node(2, vec![]),
        ])),
        ids(&[1])
    );
    assert_eq!(
        mesh_cycle_nodes(&genome(vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![1, 3]),
            simple_vm_node(3, vec![99]),
        ])),
        ids(&[1, 2])
    );
    assert!(mesh_cycle_nodes(&genome(vec![
        simple_vm_node(0, vec![1]),
        simple_vm_node(1, vec![]),
        simple_vm_node(2, vec![3]),
        simple_vm_node(3, vec![2]),
    ]))
    .is_empty());
    assert!(mesh_cycle_nodes(&genome(vec![])).is_empty());
}

// ── Mesh backward slice tests ───────────────────────────────────────

#[test]
fn mesh_backward_slice_finds_feeding_pipeline() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 2, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_backward_slice_excludes_unconnected_nodes() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![]),
            simple_vm_node(2, vec![1]),
            simple_vm_node(3, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 1, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_backward_slice_anchor_only_when_nothing_targets_it() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![])],
    };
    let gene = mesh_backward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0]);
}

#[test]
fn mesh_backward_slice_respects_max_size() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 2, 2).unwrap();
    assert_eq!(gene.indices.len(), 2);
}

#[test]
fn mesh_backward_slice_out_of_bounds_returns_none() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![])],
    };
    assert_eq!(mesh_backward_slice(&genome, 5, 8), None);
}

#[test]
fn mesh_backward_slice_empty_genome_returns_none() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![],
    };
    assert_eq!(mesh_backward_slice(&genome, 0, 8), None);
}

#[test]
fn mesh_backward_slice_handles_cycle() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![0])],
    };
    let gene = mesh_backward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1]);
}

// ── Mesh forward slice tests ────────────────────────────────────────

#[test]
fn mesh_forward_slice_finds_downstream_subtree() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_forward_slice_excludes_upstream_nodes() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 1, 8).unwrap();
    assert_eq!(gene.indices, vec![1, 2]);
}

#[test]
fn mesh_forward_slice_seed_only_when_no_targets() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![])],
    };
    let gene = mesh_forward_slice(&genome, 1, 8).unwrap();
    assert_eq!(gene.indices, vec![1]);
}

#[test]
fn mesh_forward_slice_handles_branching() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1, 2]),
            simple_vm_node(1, vec![]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_forward_slice_respects_max_size() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 2).unwrap();
    assert_eq!(gene.indices.len(), 2);
}

#[test]
fn mesh_forward_slice_out_of_bounds_returns_none() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![])],
    };
    assert_eq!(mesh_forward_slice(&genome, 5, 8), None);
}

#[test]
fn mesh_forward_slice_handles_dangling_target() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![99])],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0]);
}

#[test]
fn mesh_forward_slice_handles_cycle() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![0])],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1]);
}

// ── Functional complexity tests ──────────────────────────────────────

#[test]
fn functional_complexity_empty_genome() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![],
    };
    assert_eq!(functional_complexity(&genome), 0);
}

#[test]
fn functional_complexity_excludes_unreachable_nodes() {
    // Node 0 (entry) → Node 1. Node 2 is unreachable.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    constants: vec![],
                    program: vec![
                        VmInstruction::PushAction { action_type: 0 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![RouteTarget {
                    target_id: NodeId::new(1),
                    slot: 0,
                    gate_bias: 0.0,
                }],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    constants: vec![],
                    program: vec![
                        VmInstruction::PushAction { action_type: 0 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 4,
                    constants: vec![],
                    program: vec![
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![],
            },
        ],
    };
    let fc = functional_complexity(&genome);
    // Node 0: 1 + 1 target + 2 live instrs = 4
    // Node 1: 1 + 0 targets + 2 live instrs = 3
    // Node 2: excluded
    assert_eq!(fc, 7);
}

#[test]
fn functional_complexity_excludes_dead_vm_instructions() {
    // Single reachable node with dead instructions
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![1.0, 2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    }, // live (feeds Add → output)
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    }, // DEAD (r1 never consumed)
                    VmInstruction::Add { dst: 2, a: 0, b: 0 }, // live (feeds output)
                    VmInstruction::WriteRouteGate { slot: 0, src: 2 }, // output
                ],
            }),
            targets: vec![],
        }],
    };
    let fc = functional_complexity(&genome);
    // 1 node + 0 targets + 3 live instrs (0,2,3) + 1 live const (const_idx=0) + 0 refs = 5
    assert_eq!(fc, 5);
}

#[test]
fn functional_complexity_excludes_dead_graph_nodes() {
    // CGP graph: CN0 (Add) wired to sink, CN1 (Sigmoid) disconnected (dead)
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            })],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![
                    ComputeNode {
                        kind: ComputeNodeKind::Add,
                        inputs: vec![GraphEdge {
                            source: GraphSource::InputLeaf {
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            weight: 1.0,
                        }],
                        plasticity: None,
                    },
                    ComputeNode {
                        kind: ComputeNodeKind::Sigmoid,
                        inputs: vec![], // DEAD — disconnected from output
                        plasticity: None,
                    },
                ],
                output_sinks: vec![OutputSink {
                    kind: OutputSinkKind::CustomOutput(0),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                }],
                action_bank: vec![],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
            targets: vec![],
        }],
    };
    let fc = functional_complexity(&genome);
    // 1 mesh node + 0 targets
    // CGP: 1 wired sink + 1 live compute node (CN0) + 1 edge (CN0's InputLeaf) + 1 consumed ref
    // Total: 1 + 0 + 1 + 1 + 1 + 1 = 5
    assert_eq!(fc, 5);
}

#[test]
fn functional_complexity_counts_store_slot_as_output() {
    // StoreSlot should be treated as output instruction
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![1.0, 2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    }, // live (feeds StoreSlot)
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    }, // DEAD
                    VmInstruction::StoreSlot {
                        slot_reg: 0,
                        src: 0,
                    }, // output
                ],
            }),
            targets: vec![],
        }],
    };
    let fc = functional_complexity(&genome);
    // 1 node + 0 targets + 2 live instrs (LoadConst@0, StoreSlot@2) + 1 const + 0 refs = 4
    assert_eq!(fc, 4);
}

#[test]
fn functional_complexity_counts_only_consumed_input_refs() {
    // Node with 3 input_refs but only ref_idx=0 consumed by live code
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![
                InputReference::World(WorldInputKey::FoodHere {
                    type_idx: crate::config::OrdinaryFoodTypeId::default(),
                }),
                InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
                InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            ],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                ],
            }),
            targets: vec![],
        }],
    };
    let fc = functional_complexity(&genome);
    // 1 node + 0 targets + 2 live instrs + 0 consts + 1 consumed ref = 4
    // (not 6, which would include all 3 input_refs)
    assert_eq!(fc, 4);
}

#[test]
fn functional_complexity_equals_genome_size_fully_connected() {
    // All nodes reachable, all instructions live, all input_refs consumed
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                type_idx: crate::config::OrdinaryFoodTypeId::default(),
            })],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                ],
            }),
            targets: vec![],
        }],
    };
    assert_eq!(functional_complexity(&genome), genome.genome_size());
}
