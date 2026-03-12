use crate::config::{FounderProfile, MutationConfig};
use crate::contracts::{DynamicIntrospectionKey, InputReference, NodeId, WorldInputKey};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};

use super::cgp_founder::{build_cgp_founder_graph, build_cgp_founder_graph_with_threshold};

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

/// Return a founder genome variant selected by profile.
#[must_use]
pub fn founder_genome(profile: FounderProfile) -> CreatureGenome {
    match profile {
        FounderProfile::V3Alpha1 => v3alpha1_founder_genome(),
        FounderProfile::ForageFirstSparse => forage_first_founder_genome(30.0, 10.0),
        FounderProfile::ForageFirstSparseConservative => forage_first_founder_genome(60.0, 10.0),
        FounderProfile::ForageFirstSparseRichOffspring => forage_first_founder_genome(40.0, 20.0),
        FounderProfile::ForageFirstSparseBalanced => forage_first_founder_genome(50.0, 15.0),
    }
}

fn forage_first_founder_genome(
    reproduce_energy_threshold: f32,
    reproduce_transfer_energy: f32,
) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            node0_graph_sensor_with_threshold(reproduce_energy_threshold),
            node1_vm_decision_forage_first(reproduce_transfer_energy),
        ],
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
        backend_def: BackendDef::Graph(build_cgp_founder_graph(&MutationConfig::default())),
        targets: vec![NodeId::new(1)],
    }
}

fn node0_graph_sensor_with_threshold(reproduce_energy_threshold: f32) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::World(WorldInputKey::NeighborFoodRing),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
        ],
        backend_def: BackendDef::Graph(build_cgp_founder_graph_with_threshold(
            &MutationConfig::default(),
            reproduce_energy_threshold,
        )),
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

fn node1_vm_decision_forage_first(reproduce_transfer_energy: f32) -> NodeGenome {
    // Forage-first founder profile:
    //   Priority 1: Forage (if food present)
    //   Priority 2: Reproduce (if energy gate is open)
    //   Priority 3: Explore move fallback
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
            constants: vec![0.0, 2.0, 4.0, 6.0, reproduce_transfer_energy],
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
                // Priority 1: Forage if food on current cell.
                VmInstruction::Sub { dst: 7, a: 7, b: 7 }, // r7 = 0.0
                VmInstruction::CmpGt { dst: 6, a: 0, b: 7 }, // r6 = food_here > 0?
                VmInstruction::JumpIfZero {
                    cond: 6,
                    offset: 19,
                }, // skip forage -> reproduce check
                // Direction select (cardinal argmax over N/E/S/W) -> r0
                VmInstruction::Max { dst: 6, a: 2, b: 4 }, // vertical_best = max(N, S)
                VmInstruction::Max { dst: 7, a: 3, b: 5 }, // horizontal_best = max(E, W)
                VmInstruction::CmpGt { dst: 6, a: 6, b: 7 }, // choose_vertical?
                VmInstruction::JumpIfZero { cond: 6, offset: 6 }, // horizontal branch
                VmInstruction::CmpGt { dst: 6, a: 2, b: 4 }, // N > S?
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // choose S
                VmInstruction::Sub { dst: 0, a: 0, b: 0 }, // N = 0
                VmInstruction::Jump { offset: 7 },         // emit direction
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 2,
                }, // S = 4
                VmInstruction::Jump { offset: 5 },         // emit direction
                VmInstruction::CmpGt { dst: 6, a: 3, b: 5 }, // E > W?
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // choose W
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 1,
                }, // E = 2
                VmInstruction::Jump { offset: 1 },         // emit direction
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 3,
                }, // W = 6
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::PushAction { action_type: 1 }, // Eat
                VmInstruction::PushAction { action_type: 2 }, // Move
                VmInstruction::ExecuteActionQueue,            // terminal
                // Priority 2: Reproduce if energy sufficient.
                VmInstruction::CmpGt { dst: 6, a: 1, b: 7 }, // r6 = can_reproduce?
                VmInstruction::JumpIfZero {
                    cond: 6,
                    offset: 20,
                }, // skip reproduce -> fallback
                // Direction select (cardinal argmax over N/E/S/W) -> r0
                VmInstruction::Max { dst: 6, a: 2, b: 4 }, // vertical_best = max(N, S)
                VmInstruction::Max { dst: 7, a: 3, b: 5 }, // horizontal_best = max(E, W)
                VmInstruction::CmpGt { dst: 6, a: 6, b: 7 }, // choose_vertical?
                VmInstruction::JumpIfZero { cond: 6, offset: 6 }, // horizontal branch
                VmInstruction::CmpGt { dst: 6, a: 2, b: 4 }, // N > S?
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // choose S
                VmInstruction::Sub { dst: 0, a: 0, b: 0 }, // N = 0
                VmInstruction::Jump { offset: 7 },         // emit direction
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 2,
                }, // S = 4
                VmInstruction::Jump { offset: 5 },         // emit direction
                VmInstruction::CmpGt { dst: 6, a: 3, b: 5 }, // E > W?
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // choose W
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 1,
                }, // E = 2
                VmInstruction::Jump { offset: 1 },         // emit direction
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 3,
                }, // W = 6
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::LoadConst {
                    dst: 6,
                    const_idx: 4,
                }, // r6 = 10.0
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 1,
                    src: 6,
                }, // meta[1]=energy
                VmInstruction::PushAction { action_type: 3 }, // Reproduce
                VmInstruction::ExecuteActionQueue,            // terminal
                // Priority 3: Move fallback.
                // Direction select (cardinal argmax over N/E/S/W) -> r0
                VmInstruction::Max { dst: 6, a: 2, b: 4 }, // vertical_best = max(N, S)
                VmInstruction::Max { dst: 7, a: 3, b: 5 }, // horizontal_best = max(E, W)
                VmInstruction::CmpGt { dst: 6, a: 6, b: 7 }, // choose_vertical?
                VmInstruction::JumpIfZero { cond: 6, offset: 6 }, // horizontal branch
                VmInstruction::CmpGt { dst: 6, a: 2, b: 4 }, // N > S?
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // choose S
                VmInstruction::Sub { dst: 0, a: 0, b: 0 }, // N = 0
                VmInstruction::Jump { offset: 7 },         // emit direction
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 2,
                }, // S = 4
                VmInstruction::Jump { offset: 5 },         // emit direction
                VmInstruction::CmpGt { dst: 6, a: 3, b: 5 }, // E > W?
                VmInstruction::JumpIfZero { cond: 6, offset: 2 }, // choose W
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 1,
                }, // E = 2
                VmInstruction::Jump { offset: 1 },         // emit direction
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 3,
                }, // W = 6
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::PushAction { action_type: 1 }, // Eat
                VmInstruction::PushAction { action_type: 2 }, // Move
                VmInstruction::ExecuteActionQueue,            // terminal
            ],
        }),
        targets: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::Direction;
    use crate::contracts::WorldAction;
    use crate::creature::genome::cgp::ComputeNodeKind;
    use crate::runtime::types::MeshSideOutputs;
    use crate::runtime::vm::execute_vm_node;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;

    fn run_founder_actions(
        profile: FounderProfile,
        food_here: f32,
        can_reproduce: f32,
        food_n: f32,
        food_e: f32,
        food_s: f32,
        food_w: f32,
    ) -> Vec<WorldAction> {
        let genome = founder_genome(profile);
        let node1 = genome
            .nodes
            .iter()
            .find(|node| node.node_id == NodeId::new(1))
            .expect("founder must contain VM decision node");
        let BackendDef::Vm(vm) = &node1.backend_def else {
            panic!("node 1 must be VM backend");
        };

        let mut upstream = [0.0f32; 12];
        upstream[0] = food_here;
        upstream[1] = can_reproduce;
        upstream[2] = food_n;
        upstream[3] = food_e;
        upstream[4] = food_s;
        upstream[5] = food_w;

        let mut energy = 10_000.0;
        let mut shared_memory = [0.0f32; 16];
        let prev_shared_memory = [0.0f32; 16];
        let sensors = SensorSnapshot {
            local: StaticInputs {
                food_here: 0.0,
                neighbor_food: [0.0; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                generation: 0.0,
                age_ticks: 0.0,
            },
            perception: PerceptionSnapshot::zero(),
        };
        let runtime = RuntimeConfig::default();
        let mut side_outputs = MeshSideOutputs::new(runtime.max_actions_per_turn);
        let _result = execute_vm_node(
            vm,
            &node1.input_refs,
            &upstream,
            &mut energy,
            0.0,
            &mut shared_memory,
            &prev_shared_memory,
            &sensors,
            &runtime,
            &mut side_outputs,
        );

        side_outputs.action_queue.into_actions()
    }

    fn run_founder_move_direction(
        profile: FounderProfile,
        food_here: f32,
        can_reproduce: f32,
        food_n: f32,
        food_e: f32,
        food_s: f32,
        food_w: f32,
    ) -> Option<Direction> {
        run_founder_actions(
            profile,
            food_here,
            can_reproduce,
            food_n,
            food_e,
            food_s,
            food_w,
        )
        .into_iter()
        .find_map(|action| match action {
            WorldAction::Move(direction) => Some(direction),
            WorldAction::Reproduce { direction, .. } => Some(direction),
            _ => None,
        })
    }

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
            // CGP founder: 1 compute node (Threshold)
            assert_eq!(gdef.compute_nodes.len(), 1);
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

    #[test]
    fn profile_v3alpha1_matches_legacy_founder() {
        let a = v3alpha1_founder_genome();
        let b = founder_genome(FounderProfile::V3Alpha1);
        assert_eq!(a, b);
    }

    #[test]
    fn forage_first_profile_prioritizes_forage_before_reproduce() {
        let g = founder_genome(FounderProfile::ForageFirstSparse);
        let node1 = &g.nodes[1];
        let BackendDef::Vm(vdef) = &node1.backend_def else {
            panic!("Node 1 must be VM backend");
        };

        let first_eat = vdef
            .program
            .iter()
            .position(|instr| matches!(instr, VmInstruction::PushAction { action_type: 1 }))
            .expect("Eat PushAction should exist");
        let first_reproduce = vdef
            .program
            .iter()
            .position(|instr| matches!(instr, VmInstruction::PushAction { action_type: 3 }))
            .expect("Reproduce PushAction should exist");

        assert!(
            first_eat < first_reproduce,
            "forage-first profile should prioritize Eat before Reproduce"
        );
    }

    #[test]
    fn forage_first_profile_uses_higher_reproduction_threshold() {
        let g = founder_genome(FounderProfile::ForageFirstSparse);
        let node0 = &g.nodes[0];
        let BackendDef::Graph(graph) = &node0.backend_def else {
            panic!("Node 0 must be Graph backend");
        };
        assert_eq!(graph.compute_nodes.len(), 1);
        assert_eq!(
            graph.compute_nodes[0].kind,
            ComputeNodeKind::Threshold(30.0)
        );
    }

    #[test]
    fn forage_first_profile_fallback_direction_uses_full_cardinal_argmax() {
        let profile = FounderProfile::ForageFirstSparse;
        // food_here=0 and can_reproduce=0 force fallback move branch.
        assert_eq!(
            run_founder_move_direction(profile, 0.0, 0.0, 0.2, 0.9, 0.1, 0.3),
            Some(Direction::E)
        );
        assert_eq!(
            run_founder_move_direction(profile, 0.0, 0.0, 0.95, 0.3, 0.2, 0.1),
            Some(Direction::N)
        );
        assert_eq!(
            run_founder_move_direction(profile, 0.0, 0.0, 0.1, 0.2, 0.97, 0.3),
            Some(Direction::S)
        );
        assert_eq!(
            run_founder_move_direction(profile, 0.0, 0.0, 0.1, 0.2, 0.3, 0.99),
            Some(Direction::W)
        );
    }

    #[test]
    fn forage_first_profile_forage_branch_moves_toward_best_cardinal_food() {
        // food_here > 0 and can_reproduce = 0 force forage branch.
        assert_eq!(
            run_founder_move_direction(
                FounderProfile::ForageFirstSparse,
                1.0,
                0.0,
                0.1,
                0.95,
                0.2,
                0.3,
            ),
            Some(Direction::E)
        );
    }

    #[test]
    fn forage_first_profile_forage_branch_eats_before_move() {
        let actions = run_founder_actions(
            FounderProfile::ForageFirstSparse,
            1.0,
            0.0,
            0.2,
            0.1,
            0.3,
            0.1,
        );
        assert!(
            actions.len() >= 2,
            "forage branch should enqueue at least Eat + Move actions"
        );
        assert_eq!(actions[0], WorldAction::Eat);
        assert!(matches!(actions[1], WorldAction::Move(_)));
    }

    #[test]
    fn forage_first_profile_fallback_eats_before_move() {
        let actions = run_founder_actions(
            FounderProfile::ForageFirstSparse,
            0.0,
            0.0,
            0.2,
            0.1,
            0.3,
            0.1,
        );
        assert!(
            actions.len() >= 2,
            "fallback branch should enqueue at least Eat + Move actions"
        );
        assert_eq!(actions[0], WorldAction::Eat);
        assert!(matches!(actions[1], WorldAction::Move(_)));
    }

    #[test]
    fn forage_first_profile_reproduce_branch_moves_toward_best_cardinal_food() {
        // food_here = 0 and can_reproduce > 0 force reproduce branch.
        assert_eq!(
            run_founder_move_direction(
                FounderProfile::ForageFirstSparse,
                0.0,
                1.0,
                0.1,
                0.2,
                0.3,
                0.99,
            ),
            Some(Direction::W)
        );
    }

    #[test]
    fn forage_first_tuned_profiles_map_to_expected_threshold_and_transfer() {
        let cases = [
            (FounderProfile::ForageFirstSparse, 30.0, 10.0),
            (FounderProfile::ForageFirstSparseConservative, 60.0, 10.0),
            (FounderProfile::ForageFirstSparseRichOffspring, 40.0, 20.0),
            (FounderProfile::ForageFirstSparseBalanced, 50.0, 15.0),
        ];

        for (profile, expected_threshold, expected_transfer) in cases {
            let genome = founder_genome(profile);
            let node0 = &genome.nodes[0];
            let BackendDef::Graph(graph) = &node0.backend_def else {
                panic!("Node 0 must be Graph backend");
            };
            assert_eq!(graph.compute_nodes.len(), 1);
            assert_eq!(
                graph.compute_nodes[0].kind,
                ComputeNodeKind::Threshold(expected_threshold)
            );

            let node1 = &genome.nodes[1];
            let BackendDef::Vm(vm) = &node1.backend_def else {
                panic!("Node 1 must be VM backend");
            };
            assert!(
                vm.constants.len() >= 5,
                "forage-first VM constants should include transfer slot"
            );
            assert!(
                (vm.constants[4] - expected_transfer).abs() < f32::EPSILON,
                "profile {:?} expected transfer {}, got {}",
                profile,
                expected_transfer,
                vm.constants[4]
            );
        }
    }
}
