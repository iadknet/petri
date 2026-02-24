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
                // Priority 1: Reproduce if energy sufficient
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
                // Priority 2: Eat if food here
                // Note: r7 might be dirty from Reproduce block (Max instructions), but
                // since we only reach here if Reproduce SKIPPED (JumpIfZero),
                // r7 is still 0.0 from start.
                // If we executed Reproduce, we emitted action and stopped?
                // Wait, EmitWorldAction terminates the tick for the creature?
                // The VM spec usually implies one action per tick or explicit termination.
                // Assuming EmitWorldAction is terminal or we want priority.
                // The original code fell through:
                // Eat -> Noop (jumped past) -> Reproduce check.
                // Ah, `EmitWorldAction` does NOT stop execution?
                // The original code:
                // Eat block: ... EmitWorldAction(1), Noop (jumped past).
                // Then Reproduce block.
                // This implies multiple actions COULD be emitted?
                // But `v3-mesh-execution-spec.md` likely says "First action wins" or "Last wins"?
                // Usually "EmitWorldAction" pushes to a buffer.
                // If the VM runs to completion, we need to know if we should stop.
                // The original code structure:
                // If Eat: Emit Eat.
                // Then Continue to Reproduce?
                // Wait, `JumpIfZero { ... offset: 2 }` skips `Emit` and `Noop`.
                // If we Emit Eat, we execute `Noop`.
                // Then we fall through to Reproduce check.
                // So the original code allowed checking BOTH?
                // If so, we might emit Eat AND Reproduce?
                // I need to check if multiple actions are allowed.
                // If not, we should probably Jump to end or simple fallthrough if the system takes the *last* or *first*.
                //
                // Looking at the original again:
                // VmInstruction::CmpGt ...
                // VmInstruction::JumpIfZero ...
                // VmInstruction::EmitWorldAction { action_type: 1 } // Eat
                // VmInstruction::Noop // (jumped past)
                // // Priority 2: Reproduce ...
                //
                // There is no "Jump to End" after Eat.
                // So it *does* fall through.
                // If `EmitWorldAction` doesn't terminate, then:
                // 1. Check Eat -> Emit Eat.
                // 2. Check Reproduce -> Emit Reproduce.
                //
                // If both emitted, which wins?
                // Usually the system takes the first one, or acts on all (unlikely for Move/Eat conflict).
                //
                // However, I will stick to the *swapping* requested.
                // I will reset r7 to 0.0 before the second check to be safe,
                // OR realize that if we jumped over the first block, r7 is clean.
                // If we executed the first block (Reproduce), we presumably successfully reproduced?
                // Actually, if we reproduce, we might not want to Eat?
                //
                // Let's assume I should just swap them and ensure r7 is 0.0 for the comparison.
                //
                // In the new order:
                // 1. Reproduce check.
                //    - If false, jump to Eat. r7 is 0.
                //    - If true, execute Reproduce logic.
                //      - Uses `Max { dst: 7 ... }`, so r7 becomes dirty.
                //      - Emit Reproduce.
                //      - Fall through to Eat check.
                // 2. Eat check.
                //    - `CmpGt { dst: 6, a: 0, b: 7 }`.
                //    - If we came from Reproduce block, r7 is garbage (Max value).
                //    - This is BAD.
                //
                // FIX: I must ensure `r7` is 0.0 for the Eat check.
                // The easiest way is to NOT rely on `r7` being 0, or reset it.
                // Or, since I know `r7` is 0 at start:
                // I can use a different register for the 0.0 comparison if available?
                // I have 8 registers (0-7).
                // r0-r5 are inputs.
                // r6, r7 are temp.
                //
                // Alternative: Load 0.0 into a register.
                // Or `Sub r7, r7, r7`.
                //
                // Better approach for the swap:
                // If I reproduce, I probably shouldn't Eat in the same tick (energy transfer etc).
                // So I should probably Jump to End (or Noop) after Reproduce?
                // The original code didn't Jump to End after Eat.
                // Maybe `EmitWorldAction` is terminal?
                // I'll search for `EmitWorldAction` semantics.
                //
                // BUT, to be safe and "translate" the v1 logic (high freq reproduction):
                // I will add `Sub r7, r7, r7` (or equivalent `Xor`) before the Eat check?
                // No Xor instruction shown.
                // `Sub { dst: 7, a: 7, b: 7 }` would work.
                //
                // Let's look at the instructions available in `founder.rs` or `v3-vm-isa-spec.md`.
                // `Sub` is used later: `VmInstruction::Sub { dst: 0, a: 0, b: 0 }`.
                //
                // So I will insert `VmInstruction::Sub { dst: 7, a: 7, b: 7 }` before the Eat check.
                //
                // Revised Plan:
                // 1. Priority 1: Reproduce.
                //    - Check. JumpIfZero to Priority 2.
                //    - Body. Emit Reproduce.
                // 2. Reset r7: `Sub { dst: 7, a: 7, b: 7 }`.
                // 3. Priority 2: Eat.
                //    - Check. JumpIfZero to Priority 3.
                //    - Body. Emit Eat.
                //
                // Wait, if I add an instruction, I change offsets.
                // Eat block skip offset was 2.
                // Reproduce block skip offset was 7.
                //
                // Let's refine the "Reset r7" placement.
                // Only needed if we *did not skip* Reproduce?
                // No, if we skipped Reproduce, r7 is 0.
                // If we did Reproduce, r7 is dirty.
                // So we only need to clean r7 if we fell through.
                //
                // Actually, if we Emit Reproduce, does it matter if we check Eat?
                // If `EmitWorldAction` terminates, we are fine.
                // If not, we might queue a second action.
                //
                // Let's assume we want to be clean.
                //
                // Implementation:
                //
                // // Priority 1: Reproduce
                // CmpGt r6, r1, r7  (r7 is 0 initially)
                // JumpIfZero r6, +8 (Skip body + reset)  <-- Offset increased by 1 for the reset?
                //    Body (6 instrs)
                //    Emit
                //    Sub r7, r7, r7 (Reset r7 for next check)
                //
                // // Priority 2: Eat
                // CmpGt r6, r0, r7
                // ...
                //
                // Wait, `Sub r7, r7, r7` is 1 instruction.
                //
                // Is there a simpler way?
                // `r0` is `food_here`.
                // `r1` is `can_reproduce`.
                //
                // I can use `r1` as 0.0? No, `r1` is `can_reproduce` (0 or 1).
                // I need a zero.
                //
                // The `Sub r0, r0, r0` is used later to make a 0.
                //
                // Let's stick to the `Sub r7, r7, r7` plan.
                //
                // New Reproduce Block:
                // 1. CmpGt (1)
                // 2. JumpIfZero (1) -> Offset 8
                // 3. Max (1)
                // 4. Max (1)
                // 5. CmpGt (1)
                // 6. WriteMeta (1)
                // 7. LoadConst (1)
                // 8. WriteMeta (1)
                // 9. Emit (1)
                // 10. Sub r7, r7, r7 (1)
                //
                // Total 10 instructions?
                // Original Reproduce block body size:
                // Max, Max, CmpGt, Write, Load, Write, Emit = 7 instructions.
                // Plus CmpGt, Jump = 2.
                // Total 9.
                //
                // If I add Sub at the end, the Jump offset from the CmpGt needs to be 7 + 1 = 8.
                //
                // Eat Block:
                // CmpGt (1)
                // JumpIfZero (1) -> Offset 2
                // Emit (1)
                // Noop (1)
                //
                // Structure:
                // [Reproduce Check & Body & Reset]
                // [Eat Check & Body]
                // [Move]
                //
                // Let's write the string.

                // Priority 1: Reproduce if energy sufficient
                VmInstruction::CmpGt { dst: 6, a: 1, b: 7 }, // r6 = can_reproduce?
                VmInstruction::JumpIfZero { cond: 6, offset: 8 }, // skip reproduce block + reset
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
                VmInstruction::Sub { dst: 7, a: 7, b: 7 }, // r7 = 0.0 (clean up for Eat check)

                // Priority 2: Eat if food here
                VmInstruction::CmpGt { dst: 6, a: 0, b: 7 }, // r6 = food_here > 0? (r7=0.0)
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // skip eat if no food
                VmInstruction::EmitWorldAction { action_type: 1 }, // Eat
                VmInstruction::Noop,                         // (jumped past)

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

    #[test]
    fn test_founder_priorities() {
        let g = v3alpha1_founder_genome();
        let node1 = &g.nodes[1];
        if let BackendDef::Vm(ref vdef) = node1.backend_def {
            let mut reproduce_idx = None;
            let mut eat_idx = None;

            for (i, instr) in vdef.program.iter().enumerate() {
                match instr {
                    VmInstruction::EmitWorldAction { action_type: 3 } => {
                        if reproduce_idx.is_none() {
                            reproduce_idx = Some(i);
                        }
                    }
                    VmInstruction::EmitWorldAction { action_type: 1 } => {
                        if eat_idx.is_none() {
                            eat_idx = Some(i);
                        }
                    }
                    _ => {}
                }
            }

            assert!(reproduce_idx.is_some(), "Reproduce instruction not found");
            assert!(eat_idx.is_some(), "Eat instruction not found");
            assert!(
                reproduce_idx.unwrap() < eat_idx.unwrap(),
                "Reproduce should be prioritized (appear earlier) than Eat"
            );
        } else {
            panic!("Node 1 must be VM backend");
        }
    }
}
