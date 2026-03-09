use crate::contracts::{DynamicIntrospectionKey, InputReference, NodeId, WorldInputKey};
use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
    NodeGenome, VmBackendDef, VmInstruction,
};

/// Return the canonical v3alpha1 founder genome.
///
/// 2-node mesh: Node 0 (Graph sensor aggregator) -> Node 1 (VM decision emitter).
/// Spec: v3-startup-seeding-spec.md Section 5.1.
pub fn v3alpha1_founder_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node0_graph_sensor(), node1_vm_decision()],
    }
}

fn node0_graph_sensor() -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::World(WorldInputKey::NeighborFoodRing),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
        ],
        backend_def: BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![
                // idx 0: food_here signal
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                // idx 1: energy_current signal
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 1,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                // idx 2: reproduce gate (energy >= 24.0)
                GraphInternalNode {
                    kind: GraphNodeKind::Threshold(24.0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // idx 3-6: neighbor food N/E/S/W via ring sub_idx
                // Direction::to_index(): N=0, E=2, S=4, W=6
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 2,
                        sub_idx: 0, // N
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 2,
                        sub_idx: 2, // E
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 2,
                        sub_idx: 4, // S
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 2,
                        sub_idx: 6, // W
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                // idx 7-10: neighbor occupied N/E/S/W via ring sub_idx
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 3,
                        sub_idx: 0, // N
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 3,
                        sub_idx: 2, // E
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 3,
                        sub_idx: 4, // S
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 3,
                        sub_idx: 6, // W
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                // idx 11-16: output writers
                // slot 0 = food_here
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // slot 1 = can_reproduce (0 or 1)
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(1),
                    inputs: vec![GraphInput {
                        source_idx: 2,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // slot 2 = food_N
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(2),
                    inputs: vec![GraphInput {
                        source_idx: 3,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // slot 3 = food_E
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(3),
                    inputs: vec![GraphInput {
                        source_idx: 4,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // slot 4 = food_S
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(4),
                    inputs: vec![GraphInput {
                        source_idx: 5,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // slot 5 = food_W
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(5),
                    inputs: vec![GraphInput {
                        source_idx: 6,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                // idx 17: route to node 1
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![],
                    plasticity: None,
                },
            ],
        }),
        targets: vec![NodeId::new(1)],
    }
}

fn node1_vm_decision() -> NodeGenome {
    // Multi-action VM decision node.
    //
    // Uses PushAction + ExecuteActionQueue instead of the removed EmitWorldAction.
    // Three priority branches, each ending with ExecuteActionQueue (terminal):
    //
    //   Priority 1: Reproduce (if energy sufficient)
    //     → PushAction(3=Reproduce) + ExecuteActionQueue
    //
    //   Priority 2: Forage (if food present on cell)
    //     → PushAction(2=Move toward food) + PushAction(1=Eat) + ExecuteActionQueue
    //     (move+eat combo saves one tick of decay per foraging cycle)
    //
    //   Priority 3: Explore (fallback)
    //     → PushAction(2=Move toward food dir) + ExecuteActionQueue
    //
    // Register usage: r0=food_here, r1=can_reproduce, r2=food_N, r3=food_E,
    //                 r4=food_S, r5=food_W, r6/r7=temp
    //
    // Program layout (PC indices):
    //   [0..5]    Read 6 inputs
    //   [6..15]   Reproduce branch (check + body, terminal)
    //   [16..25]  Forage branch (check + body, terminal)
    //   [26..37]  Move fallback (direction selection + terminal)
    NodeGenome {
        node_id: NodeId::new(1),
        input_refs: vec![
            InputReference::UpstreamSlot(0), // food_here
            InputReference::UpstreamSlot(1), // can_reproduce
            InputReference::UpstreamSlot(2), // food_N
            InputReference::UpstreamSlot(3), // food_E
            InputReference::UpstreamSlot(4), // food_S
            InputReference::UpstreamSlot(5), // food_W
        ],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 8,
            constants: vec![0.5, 1.0, 2.0, 3.0, 20.0],
            program: vec![
                // [0..5] Read inputs
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                }, // r0 = food_here
                VmInstruction::ReadInput {
                    dst: 1,
                    ref_idx: 1,
                    sub_idx: 0,
                }, // r1 = can_reproduce
                VmInstruction::ReadInput {
                    dst: 2,
                    ref_idx: 2,
                    sub_idx: 0,
                }, // r2 = food_N
                VmInstruction::ReadInput {
                    dst: 3,
                    ref_idx: 3,
                    sub_idx: 0,
                }, // r3 = food_E
                VmInstruction::ReadInput {
                    dst: 4,
                    ref_idx: 4,
                    sub_idx: 0,
                }, // r4 = food_S
                VmInstruction::ReadInput {
                    dst: 5,
                    ref_idx: 5,
                    sub_idx: 0,
                }, // r5 = food_W
                // [6..15] Priority 1: Reproduce if energy sufficient
                // r7 starts at 0.0, CmpGt(r1, r7) checks can_reproduce > 0
                VmInstruction::CmpGt { dst: 6, a: 1, b: 7 }, // [6]  r6 = can_reproduce?
                VmInstruction::JumpIfZero { cond: 6, offset: 8 }, // [7]  skip body → PC 16
                VmInstruction::Max { dst: 6, a: 2, b: 3 },   // [8]  direction heuristic
                VmInstruction::Max { dst: 7, a: 4, b: 5 },   // [9]
                VmInstruction::CmpGt { dst: 6, a: 2, b: 3 }, // [10]
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 7,
                }, // [11] meta[0]=dir
                VmInstruction::LoadConst {
                    dst: 6,
                    const_idx: 4,
                }, // [12] r6 = 20.0
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 1,
                    src: 6,
                }, // [13] meta[1]=energy
                VmInstruction::PushAction { action_type: 3 }, // [14] push Reproduce
                VmInstruction::ExecuteActionQueue,           // [15] terminal → done
                // [16..25] Priority 2: Forage if food on current cell
                VmInstruction::Sub { dst: 7, a: 7, b: 7 }, // [16] r7 = 0.0 (reset)
                VmInstruction::CmpGt { dst: 6, a: 0, b: 7 }, // [17] r6 = food_here > 0?
                VmInstruction::JumpIfZero { cond: 6, offset: 7 }, // [18] skip body → PC 26
                VmInstruction::Max { dst: 6, a: 2, b: 3 }, // [19] direction heuristic
                VmInstruction::Max { dst: 7, a: 4, b: 5 }, // [20]
                VmInstruction::CmpGt { dst: 6, a: 6, b: 7 }, // [21]
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 7,
                }, // [22] meta[0]=dir
                VmInstruction::PushAction { action_type: 2 }, // [23] push Move
                VmInstruction::PushAction { action_type: 1 }, // [24] push Eat
                VmInstruction::ExecuteActionQueue,         // [25] terminal → done
                // [26..37] Priority 3: Move toward best food direction (fallback)
                VmInstruction::Sub { dst: 7, a: 7, b: 7 }, // [26] r7 = 0.0 (reset)
                VmInstruction::Max { dst: 6, a: 2, b: 3 }, // [27] max(food_N, food_E)
                VmInstruction::Max { dst: 7, a: 4, b: 5 }, // [28] max(food_S, food_W)
                VmInstruction::CmpGt { dst: 6, a: 2, b: 4 }, // [29] r6 = food_N > food_S?
                VmInstruction::CmpGt { dst: 7, a: 3, b: 5 }, // [30] r7 = food_E > food_W?
                VmInstruction::Sub { dst: 0, a: 0, b: 0 }, // [31] r0 = 0.0 (N direction)
                VmInstruction::JumpIfZero { cond: 6, offset: 1 }, // [32] if S >= N, skip to [34]
                VmInstruction::Jump { offset: 1 },         // [33] N wins, keep r0=0 → [35]
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 3,
                }, // [34] r0 = 3.0 (S direction)
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                }, // [35] meta[0]=dir
                VmInstruction::PushAction { action_type: 2 }, // [36] push Move
                VmInstruction::ExecuteActionQueue,         // [37] terminal → done
            ],
        }),
        targets: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn founder_genome_structure() {
        let g = v3alpha1_founder_genome();
        assert_eq!(g.entry_node_id, NodeId::new(0));
        assert_eq!(g.nodes.len(), 2);

        // Node 0: Graph backend
        let node0 = &g.nodes[0];
        assert_eq!(node0.node_id, NodeId::new(0));
        assert_eq!(node0.input_refs.len(), 4);
        assert_eq!(node0.targets, vec![NodeId::new(1)]);

        if let BackendDef::Graph(ref gdef) = node0.backend_def {
            assert_eq!(gdef.internal_nodes.len(), 18);
        } else {
            panic!("Node 0 must be Graph backend");
        }

        // Node 1: VM backend
        let node1 = &g.nodes[1];
        assert_eq!(node1.node_id, NodeId::new(1));
        assert_eq!(node1.input_refs.len(), 6);
        assert!(node1.targets.is_empty());

        if let BackendDef::Vm(ref vdef) = node1.backend_def {
            assert_eq!(vdef.register_count, 8);
            assert_eq!(vdef.constants, vec![0.5, 1.0, 2.0, 3.0, 20.0]);
            assert!(!vdef.program.is_empty());
        } else {
            panic!("Node 1 must be VM backend");
        }
    }

    #[test]
    fn test_founder_priorities() {
        let g = v3alpha1_founder_genome();
        let node1 = &g.nodes[1];
        if let BackendDef::Vm(ref vdef) = node1.backend_def {
            let mut reproduce_idx = None;
            let mut eat_idx = None;

            for (i, instr) in vdef.program.iter().enumerate() {
                match instr {
                    VmInstruction::PushAction { action_type: 3 } => {
                        if reproduce_idx.is_none() {
                            reproduce_idx = Some(i);
                        }
                    }
                    VmInstruction::PushAction { action_type: 1 } => {
                        if eat_idx.is_none() {
                            eat_idx = Some(i);
                        }
                    }
                    _ => {}
                }
            }

            assert!(reproduce_idx.is_some(), "Reproduce PushAction not found");
            assert!(eat_idx.is_some(), "Eat PushAction not found");
            assert!(
                reproduce_idx.unwrap() < eat_idx.unwrap(),
                "Reproduce should be prioritized (appear earlier) than Eat"
            );
        } else {
            panic!("Node 1 must be VM backend");
        }
    }
}
