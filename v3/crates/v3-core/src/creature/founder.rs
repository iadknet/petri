use crate::contracts::{Direction, DynamicIntrospectionKey, InputReference, NodeId, WorldInputKey};
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
            InputReference::World(WorldInputKey::NeighborCellFood(Direction::N)),
            InputReference::World(WorldInputKey::NeighborCellFood(Direction::E)),
            InputReference::World(WorldInputKey::NeighborCellFood(Direction::S)),
            InputReference::World(WorldInputKey::NeighborCellFood(Direction::W)),
            InputReference::World(WorldInputKey::NeighborCellOccupied(Direction::N)),
            InputReference::World(WorldInputKey::NeighborCellOccupied(Direction::E)),
            InputReference::World(WorldInputKey::NeighborCellOccupied(Direction::S)),
            InputReference::World(WorldInputKey::NeighborCellOccupied(Direction::W)),
        ],
        backend_def: BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![
                // idx 0: food_here signal
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(0),
                    inputs: vec![],
                },
                // idx 1: energy_current signal
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(1),
                    inputs: vec![],
                },
                // idx 2: reproduce gate (energy >= 24.0)
                GraphInternalNode {
                    kind: GraphNodeKind::Threshold(24.0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                },
                // idx 3-6: neighbor food N/E/S/W
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(2),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(3),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(4),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(5),
                    inputs: vec![],
                },
                // idx 7-10: neighbor occupied N/E/S/W
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(6),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(7),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(8),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(9),
                    inputs: vec![],
                },
                // idx 11-16: output writers
                // slot 0 = food_here
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                },
                // slot 1 = can_reproduce (0 or 1)
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(1),
                    inputs: vec![GraphInput {
                        source_idx: 2,
                        weight: 1.0,
                    }],
                },
                // slot 2 = food_N
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(2),
                    inputs: vec![GraphInput {
                        source_idx: 3,
                        weight: 1.0,
                    }],
                },
                // slot 3 = food_E
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(3),
                    inputs: vec![GraphInput {
                        source_idx: 4,
                        weight: 1.0,
                    }],
                },
                // slot 4 = food_S
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(4),
                    inputs: vec![GraphInput {
                        source_idx: 5,
                        weight: 1.0,
                    }],
                },
                // slot 5 = food_W
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(5),
                    inputs: vec![GraphInput {
                        source_idx: 6,
                        weight: 1.0,
                    }],
                },
                // idx 17: route to node 1
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![],
                },
            ],
        }),
        targets: vec![NodeId::new(1)],
    }
}

fn node1_vm_decision() -> NodeGenome {
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
                // Read inputs into registers
                VmInstruction::ReadInput {
                    dst: 0,
                    input_idx: 0,
                }, // r0 = food_here
                VmInstruction::ReadInput {
                    dst: 1,
                    input_idx: 1,
                }, // r1 = can_reproduce
                VmInstruction::ReadInput {
                    dst: 2,
                    input_idx: 2,
                }, // r2 = food_N
                VmInstruction::ReadInput {
                    dst: 3,
                    input_idx: 3,
                }, // r3 = food_E
                VmInstruction::ReadInput {
                    dst: 4,
                    input_idx: 4,
                }, // r4 = food_S
                VmInstruction::ReadInput {
                    dst: 5,
                    input_idx: 5,
                }, // r5 = food_W
                // Priority 1: Eat if food here
                VmInstruction::CmpGt { dst: 6, a: 0, b: 7 }, // r6 = food_here > 0? (r7=0.0)
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // skip eat if no food
                VmInstruction::EmitWorldAction { action_type: 1 }, // Eat
                VmInstruction::Noop,                         // (jumped past)
                // Priority 2: Reproduce if energy sufficient
                VmInstruction::CmpGt { dst: 6, a: 1, b: 7 }, // r6 = can_reproduce?
                VmInstruction::JumpIfZero { cond: 6, offset: 7 }, // skip reproduce block
                // Find direction for reproduce (use food dirs as proxy)
                VmInstruction::Max { dst: 6, a: 2, b: 3 }, // r6 = max(food_N, food_E)
                VmInstruction::Max { dst: 7, a: 4, b: 5 }, // r7 = max(food_S, food_W)
                VmInstruction::CmpGt { dst: 6, a: 2, b: 3 }, // r6 = food_N > food_E?
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 7,
                }, // direction placeholder
                VmInstruction::LoadConst {
                    dst: 6,
                    const_idx: 4,
                }, // const[4] = 20.0
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 1,
                    src: 6,
                }, // offspring energy
                VmInstruction::EmitWorldAction { action_type: 3 }, // Reproduce
                // Priority 3: Move toward highest food direction
                VmInstruction::Max { dst: 6, a: 2, b: 3 }, // max(food_N, food_E)
                VmInstruction::Max { dst: 7, a: 4, b: 5 }, // max(food_S, food_W)
                VmInstruction::CmpGt { dst: 6, a: 2, b: 4 }, // r6 = food_N > food_S?
                VmInstruction::CmpGt { dst: 7, a: 3, b: 5 }, // r7 = food_E > food_W?
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                }, // const[0] = 0.5
                VmInstruction::Sub { dst: 0, a: 0, b: 0 }, // r0 = 0 (N)
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // if food_S >= food_N skip
                VmInstruction::Jump { offset: 2 },         // food_N wins, keep r0=0
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 3,
                }, // const[3] = 3.0 ~ S dir
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                }, // set direction
                VmInstruction::EmitWorldAction { action_type: 2 }, // Move
                // Priority 4: Fallback NoOp
                VmInstruction::EmitWorldAction { action_type: 0 }, // NoOp
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
        assert_eq!(node0.input_refs.len(), 10);
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
}
