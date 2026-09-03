use crate::config::{FounderProfile, MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, NodeId, RouteTarget, StaticIntrospectionKey,
    WorldInputKey,
};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};

use super::cgp_founder::build_cgp_founder_graph_with_thresholds_and_reserve;

const DEFAULT_MIN_REPRODUCE_AGE_TICKS: u64 = 20;

/// Return the canonical v3alpha1 founder genome.
///
/// 2-node mesh: Node 0 (Graph sensor aggregator) -> Node 1 (VM decision emitter).
/// Spec: v3-startup-seeding-spec.md Section 5.1.
pub fn v3alpha1_founder_genome() -> CreatureGenome {
    v3alpha1_founder_genome_with_min_reproduce_age(DEFAULT_MIN_REPRODUCE_AGE_TICKS)
}

/// Return the canonical v3alpha1 founder genome with a configurable minimum
/// reproduction age gate (in ticks).
#[must_use]
pub fn v3alpha1_founder_genome_with_min_reproduce_age(
    min_reproduce_age_ticks: u64,
) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            node0_graph_sensor(min_reproduce_age_ticks),
            node1_vm_deficiency_decision(false, 30.0, 20.0, 4.0),
        ],
    }
}

/// Return a founder genome variant selected by profile.
#[must_use]
pub fn founder_genome(profile: FounderProfile) -> CreatureGenome {
    founder_genome_with_min_reproduce_age(profile, DEFAULT_MIN_REPRODUCE_AGE_TICKS)
}

/// Return a founder genome variant selected by profile with a configurable
/// minimum reproduction age gate (in ticks).
#[must_use]
pub fn founder_genome_with_min_reproduce_age(
    profile: FounderProfile,
    min_reproduce_age_ticks: u64,
) -> CreatureGenome {
    founder_genome_with_min_reproduce_age_and_reserve_cost(profile, min_reproduce_age_ticks, 4.0)
}

/// Return a founder genome with runtime-configured age and reserve gates.
#[must_use]
pub fn founder_genome_with_min_reproduce_age_and_reserve_cost(
    profile: FounderProfile,
    min_reproduce_age_ticks: u64,
    reproductive_reserve_cost: f32,
) -> CreatureGenome {
    match profile {
        FounderProfile::V3Alpha1 => CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                node0_graph_sensor_with_reserve(min_reproduce_age_ticks, reproductive_reserve_cost),
                node1_vm_deficiency_decision(false, 30.0, 20.0, reproductive_reserve_cost),
            ],
        },
        FounderProfile::ForageFirstSparse => forage_first_founder_genome(
            30.0,
            10.0,
            min_reproduce_age_ticks,
            reproductive_reserve_cost,
        ),
        FounderProfile::ForageFirstSparseConservative => forage_first_founder_genome(
            60.0,
            10.0,
            min_reproduce_age_ticks,
            reproductive_reserve_cost,
        ),
        FounderProfile::ForageFirstSparseRichOffspring => forage_first_founder_genome(
            40.0,
            20.0,
            min_reproduce_age_ticks,
            reproductive_reserve_cost,
        ),
        FounderProfile::ForageFirstSparseBalanced => forage_first_founder_genome(
            50.0,
            15.0,
            min_reproduce_age_ticks,
            reproductive_reserve_cost,
        ),
    }
}

fn forage_first_founder_genome(
    reproduce_energy_threshold: f32,
    reproduce_transfer_energy: f32,
    min_reproduce_age_ticks: u64,
    reproductive_reserve_cost: f32,
) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            node0_graph_sensor_with_threshold_and_reserve(
                reproduce_energy_threshold,
                min_reproduce_age_ticks,
                reproductive_reserve_cost,
            ),
            node1_vm_deficiency_decision(
                true,
                reproduce_energy_threshold,
                reproduce_transfer_energy,
                reproductive_reserve_cost,
            ),
        ],
    }
}

fn node0_graph_sensor(min_reproduce_age_ticks: u64) -> NodeGenome {
    node0_graph_sensor_with_reserve(min_reproduce_age_ticks, 4.0)
}

fn node0_graph_sensor_with_reserve(
    min_reproduce_age_ticks: u64,
    reproductive_reserve_cost: f32,
) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            InputReference::World(WorldInputKey::neighbor_food_ring(
                OrdinaryFoodTypeId::default(),
            )),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::new(1))),
            InputReference::DynamicIntrospection(
                DynamicIntrospectionKey::ReproductiveReserveCurrent,
            ),
            InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                1,
            ))),
        ],
        backend_def: BackendDef::Graph(build_cgp_founder_graph_with_thresholds_and_reserve(
            &MutationConfig::default(),
            30.0,
            min_reproduce_age_ticks as f32,
            reproductive_reserve_cost,
        )),
        targets: vec![RouteTarget {
            target_id: NodeId::new(1),
            slot: 0,
            gate_bias: 0.0,
        }],
    }
}

fn node0_graph_sensor_with_threshold_and_reserve(
    reproduce_energy_threshold: f32,
    min_reproduce_age_ticks: u64,
    reproductive_reserve_cost: f32,
) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            InputReference::World(WorldInputKey::neighbor_food_ring(
                OrdinaryFoodTypeId::default(),
            )),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
            InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::new(1))),
            InputReference::DynamicIntrospection(
                DynamicIntrospectionKey::ReproductiveReserveCurrent,
            ),
            InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                1,
            ))),
        ],
        backend_def: BackendDef::Graph(build_cgp_founder_graph_with_thresholds_and_reserve(
            &MutationConfig::default(),
            reproduce_energy_threshold,
            min_reproduce_age_ticks as f32,
            reproductive_reserve_cost,
        )),
        targets: vec![RouteTarget {
            target_id: NodeId::new(1),
            slot: 0,
            gate_bias: 0.0,
        }],
    }
}

struct FounderVmBuilder {
    instructions: Vec<VmInstruction>,
    labels: Vec<Option<usize>>,
    jumps: Vec<(usize, usize)>,
}

impl FounderVmBuilder {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: Vec::new(),
            jumps: Vec::new(),
        }
    }

    fn label(&mut self) -> usize {
        let label = self.labels.len();
        self.labels.push(None);
        label
    }

    fn mark(&mut self, label: usize) {
        self.labels[label] = Some(self.instructions.len());
    }

    fn push(&mut self, instruction: VmInstruction) {
        self.instructions.push(instruction);
    }

    fn jump_if_zero(&mut self, cond: u8, target: usize) {
        let index = self.instructions.len();
        self.instructions
            .push(VmInstruction::JumpIfZero { cond, offset: 0 });
        self.jumps.push((index, target));
    }

    fn jump(&mut self, target: usize) {
        let index = self.instructions.len();
        self.instructions.push(VmInstruction::Jump { offset: 0 });
        self.jumps.push((index, target));
    }

    fn finish(mut self) -> Vec<VmInstruction> {
        for (index, label) in self.jumps {
            let target = self.labels[label].expect("founder VM label must be marked");
            let offset = target as i32 - index as i32 - 1;
            match &mut self.instructions[index] {
                VmInstruction::JumpIfZero { offset: value, .. }
                | VmInstruction::Jump { offset: value } => *value = offset,
                _ => unreachable!("founder VM jump table points to non-jump"),
            }
        }
        self.instructions
    }
}

fn set_founder_direction(builder: &mut FounderVmBuilder, direction: usize) {
    if direction == 0 {
        builder.push(VmInstruction::Sub { dst: 0, a: 0, b: 0 });
    } else {
        builder.push(VmInstruction::LoadConst {
            dst: 0,
            const_idx: direction as u8 + 1,
        });
    }
}

fn append_founder_move(builder: &mut FounderVmBuilder, neighbors: [u8; 4]) {
    let choose_north = builder.label();
    let choose_east = builder.label();
    let choose_south = builder.label();
    let emit = builder.label();

    // First compute the maximum over the complete cardinal ring.  Comparing
    // each candidate against that value avoids the adjacent-pair trap where
    // an early local winner can hide a larger value in the remaining ring.
    builder.push(VmInstruction::Max {
        dst: 14,
        a: neighbors[0],
        b: neighbors[1],
    });
    builder.push(VmInstruction::Max {
        dst: 14,
        a: 14,
        b: neighbors[2],
    });
    builder.push(VmInstruction::Max {
        dst: 14,
        a: 14,
        b: neighbors[3],
    });

    // Check in N/E/S/W order so ties have a stable, documented preference for
    // the first direction in the ring.
    for (candidate, target) in [
        (neighbors[0], choose_north),
        (neighbors[1], choose_east),
        (neighbors[2], choose_south),
    ] {
        builder.push(VmInstruction::CmpGt {
            dst: 13,
            a: 14,
            b: candidate,
        });
        builder.jump_if_zero(13, target);
    }
    set_founder_direction(builder, 3);
    builder.jump(emit);

    builder.mark(choose_north);
    set_founder_direction(builder, 0);
    builder.jump(emit);

    builder.mark(choose_east);
    set_founder_direction(builder, 1);
    builder.jump(emit);

    builder.mark(choose_south);
    set_founder_direction(builder, 2);

    builder.mark(emit);
    builder.push(VmInstruction::WriteWorldActionMeta {
        slot_idx: 0,
        src: 0,
    });
    builder.push(VmInstruction::PushAction { action_type: 2 });
    builder.push(VmInstruction::ExecuteActionQueue);
}

fn append_founder_eat(builder: &mut FounderVmBuilder, food_type: u8) {
    if food_type == 0 {
        builder.push(VmInstruction::Sub {
            dst: 14,
            a: 14,
            b: 14,
        });
    } else {
        builder.push(VmInstruction::LoadConst {
            dst: 14,
            const_idx: 1,
        });
    }
    builder.push(VmInstruction::WriteWorldActionMeta {
        slot_idx: 0,
        src: 14,
    });
    builder.push(VmInstruction::PushAction { action_type: 1 });
    builder.push(VmInstruction::ExecuteActionQueue);
}

fn append_founder_reproduce(
    builder: &mut FounderVmBuilder,
    reproduce_label: usize,
    fallback_label: usize,
) {
    builder.mark(reproduce_label);
    builder.push(VmInstruction::CmpGt {
        dst: 13,
        a: 1,
        b: 15,
    });
    builder.jump_if_zero(13, fallback_label);
    builder.push(VmInstruction::Sub {
        dst: 14,
        a: 14,
        b: 14,
    });
    builder.push(VmInstruction::WriteWorldActionMeta {
        slot_idx: 0,
        src: 14,
    });
    builder.push(VmInstruction::LoadConst {
        dst: 14,
        const_idx: 6,
    });
    builder.push(VmInstruction::WriteWorldActionMeta {
        slot_idx: 1,
        src: 14,
    });
    builder.push(VmInstruction::PushAction { action_type: 3 });
    builder.push(VmInstruction::ExecuteActionQueue);
}

fn node1_vm_deficiency_decision(
    forage_first: bool,
    reproduce_energy_threshold: f32,
    reproduce_transfer_energy: f32,
    reproductive_reserve_cost: f32,
) -> NodeGenome {
    let input_refs = (0..13)
        .map(InputReference::UpstreamSlot)
        .collect::<Vec<_>>();
    let mut builder = FounderVmBuilder::new();
    for register in 0..13u8 {
        builder.push(VmInstruction::ReadInput {
            dst: register,
            ref_idx: u16::from(register),
            sub_idx: 0,
        });
    }
    builder.push(VmInstruction::Sub {
        dst: 15,
        a: 15,
        b: 15,
    });

    // Load the two thresholds once into scratch registers as needed. The
    // branch order changes only the profile's reproduction-vs-forage priority;
    // both profiles always choose the deficient typed resource first.
    let reproduce = builder.label();
    let reserve_check = builder.label();
    let reserve_seek = builder.label();
    let reserve_move = builder.label();
    let energy_check = builder.label();
    let energy_move = builder.label();
    let fallback = builder.label();

    if !forage_first {
        builder.push(VmInstruction::CmpGt {
            dst: 13,
            a: 1,
            b: 15,
        });
        builder.jump_if_zero(13, reserve_check);
        append_founder_reproduce(&mut builder, reproduce, fallback);
    }

    builder.mark(reserve_check);
    builder.push(VmInstruction::LoadConst {
        dst: 14,
        const_idx: 5,
    });
    builder.push(VmInstruction::CmpLt {
        dst: 13,
        a: 7,
        b: 14,
    });
    builder.jump_if_zero(13, energy_check);
    builder.push(VmInstruction::CmpGt {
        dst: 13,
        a: 6,
        b: 15,
    });
    builder.jump_if_zero(13, reserve_seek);
    append_founder_eat(&mut builder, 1);
    builder.mark(reserve_seek);
    // If reproductive food is absent locally, only pursue it when a typed
    // neighbor actually reports it. Otherwise fall through to the energy
    // deficit branch so maintenance-only worlds remain viable.
    builder.push(VmInstruction::Max {
        dst: 13,
        a: 9,
        b: 10,
    });
    builder.push(VmInstruction::Max {
        dst: 14,
        a: 11,
        b: 12,
    });
    builder.push(VmInstruction::Max {
        dst: 13,
        a: 13,
        b: 14,
    });
    builder.push(VmInstruction::CmpGt {
        dst: 13,
        a: 13,
        b: 15,
    });
    builder.jump_if_zero(13, energy_check);
    builder.jump(reserve_move);
    builder.mark(reserve_move);
    append_founder_move(&mut builder, [9, 10, 11, 12]);

    builder.mark(energy_check);
    builder.push(VmInstruction::LoadConst {
        dst: 14,
        const_idx: 7,
    });
    builder.push(VmInstruction::CmpLt {
        dst: 13,
        a: 8,
        b: 14,
    });
    builder.jump_if_zero(13, reproduce);
    builder.push(VmInstruction::CmpGt {
        dst: 13,
        a: 0,
        b: 15,
    });
    builder.jump_if_zero(13, energy_move);
    append_founder_eat(&mut builder, 0);
    builder.mark(energy_move);
    append_founder_move(&mut builder, [2, 3, 4, 5]);

    if forage_first {
        append_founder_reproduce(&mut builder, reproduce, fallback);
    }
    builder.mark(fallback);
    append_founder_move(&mut builder, [2, 3, 4, 5]);

    NodeGenome {
        node_id: NodeId::new(1),
        input_refs,
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 16,
            constants: vec![
                0.0,
                1.0,
                2.0,
                4.0,
                6.0,
                reproductive_reserve_cost,
                reproduce_transfer_energy,
                reproduce_energy_threshold,
            ],
            program: builder.finish(),
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
    use crate::runtime::types::{MeshSideOutputs, OUTPUT_SLOT_COUNT};
    use crate::runtime::vm::execute_vm_node;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;
    use crate::sensors::typed_food::TypedFoodLocalSnapshot;

    #[allow(clippy::too_many_arguments)]
    fn run_founder_actions_with_state(
        profile: FounderProfile,
        food_here: f32,
        can_reproduce: f32,
        food_n: f32,
        food_e: f32,
        food_s: f32,
        food_w: f32,
        reproductive_food_here: f32,
        reserve: f32,
        energy_value: f32,
        reproductive_food_n: f32,
        reproductive_food_e: f32,
        reproductive_food_s: f32,
        reproductive_food_w: f32,
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

        let mut upstream = [0.0f32; OUTPUT_SLOT_COUNT];
        upstream[0] = food_here;
        upstream[1] = can_reproduce;
        upstream[2] = food_n;
        upstream[3] = food_e;
        upstream[4] = food_s;
        upstream[5] = food_w;
        upstream[6] = reproductive_food_here;
        upstream[7] = reserve;
        upstream[8] = energy_value;
        upstream[9] = reproductive_food_n;
        upstream[10] = reproductive_food_e;
        upstream[11] = reproductive_food_s;
        upstream[12] = reproductive_food_w;

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
            typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
            perception: PerceptionSnapshot::zeroed(1),
        };
        let runtime = RuntimeConfig::default();
        let mut side_outputs = MeshSideOutputs::new(runtime.max_actions_per_turn);
        let _result = execute_vm_node(
            vm,
            &node1.input_refs,
            &upstream,
            &mut energy,
            0.0,
            reserve,
            &mut shared_memory,
            &prev_shared_memory,
            &sensors,
            &runtime,
            &mut side_outputs,
        );

        side_outputs.action_queue.into_actions()
    }

    fn run_founder_actions(
        profile: FounderProfile,
        food_here: f32,
        can_reproduce: f32,
        food_n: f32,
        food_e: f32,
        food_s: f32,
        food_w: f32,
    ) -> Vec<WorldAction> {
        run_founder_actions_with_state(
            profile,
            food_here,
            can_reproduce,
            food_n,
            food_e,
            food_s,
            food_w,
            0.0,
            4.0,
            10.0,
            0.0,
            0.0,
            0.0,
            0.0,
        )
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

    #[allow(clippy::too_many_arguments)]
    fn run_founder_move_direction_with_state_for_test(
        profile: FounderProfile,
        food_here: f32,
        can_reproduce: f32,
        food_n: f32,
        food_e: f32,
        food_s: f32,
        food_w: f32,
        reproductive_food_here: f32,
        reserve: f32,
        energy_value: f32,
        reproductive_food_n: f32,
        reproductive_food_e: f32,
        reproductive_food_s: f32,
        reproductive_food_w: f32,
    ) -> Option<Direction> {
        run_founder_actions_with_state(
            profile,
            food_here,
            can_reproduce,
            food_n,
            food_e,
            food_s,
            food_w,
            reproductive_food_here,
            reserve,
            energy_value,
            reproductive_food_n,
            reproductive_food_e,
            reproductive_food_s,
            reproductive_food_w,
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
        assert_eq!(node0.input_refs.len(), 8);
        assert_eq!(
            node0.targets,
            vec![RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 0.0
            }]
        );

        if let BackendDef::Graph(ref gdef) = node0.backend_def {
            // CGP founder: energy, age, and reserve gates; combined reproduction
            // gate; and aggregate typed-food forage gate.
            assert_eq!(gdef.compute_nodes.len(), 6);
        } else {
            panic!("Node 0 must be Graph backend");
        }

        // Node 1: VM backend
        let node1 = &g.nodes[1];
        assert_eq!(node1.node_id, NodeId::new(1));
        assert_eq!(node1.input_refs.len(), 13);
        assert!(node1.targets.is_empty());

        if let BackendDef::Vm(ref vdef) = node1.backend_def {
            assert_eq!(vdef.register_count, 16);
            assert_eq!(
                vdef.constants,
                vec![0.0, 1.0, 2.0, 4.0, 6.0, 4.0, 20.0, 30.0]
            );
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
        assert_eq!(
            graph.compute_nodes[0].kind,
            ComputeNodeKind::Threshold(30.0),
            "first compute node should retain energy threshold tuning"
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
    fn maintenance_food_direction_considers_all_four_neighbors() {
        // Arrange: the first adjacent comparison (N versus E) favors N, but S
        // is the global maximum.  An energy deficit selects the maintenance ring.
        // Act
        let direction = run_founder_move_direction(
            FounderProfile::ForageFirstSparse,
            0.0,
            0.0,
            0.5,
            0.4,
            0.9,
            0.1,
        );

        // Assert
        assert_eq!(direction, Some(Direction::S));
    }

    #[test]
    fn reproductive_food_direction_considers_all_four_neighbors() {
        // Arrange: the first adjacent comparison (N versus E) favors N, but S
        // is the global maximum.  A reserve deficit selects the reproductive ring.
        // Act
        let direction = run_founder_move_direction_with_state_for_test(
            FounderProfile::ForageFirstSparse,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            100.0,
            0.5,
            0.4,
            0.9,
            0.1,
        );

        // Assert
        assert_eq!(direction, Some(Direction::S));
    }

    #[test]
    fn both_food_rings_prefer_north_on_equal_maxima() {
        // Arrange: all cardinal values tie, so the ring's documented order is
        // the only source of a deterministic direction.
        // Act
        let maintenance = run_founder_move_direction(
            FounderProfile::ForageFirstSparse,
            0.0,
            0.0,
            0.5,
            0.5,
            0.5,
            0.5,
        );
        let reproductive = run_founder_move_direction_with_state_for_test(
            FounderProfile::ForageFirstSparse,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            100.0,
            0.5,
            0.5,
            0.5,
            0.5,
        );

        // Assert
        assert_eq!(maintenance, Some(Direction::N));
        assert_eq!(reproductive, Some(Direction::N));
    }

    #[test]
    fn forage_first_profile_forage_branch_moves_toward_best_cardinal_food() {
        // A deficient reserve and reproductive food to the east force typed
        // reproductive-food seeking rather than a blind primary-food action.
        assert_eq!(
            run_founder_actions_with_state(
                FounderProfile::ForageFirstSparse,
                0.0,
                0.0,
                0.1,
                0.1,
                0.1,
                0.1,
                0.0,
                0.0,
                100.0,
                0.0,
                0.95,
                0.0,
                0.0,
            ),
            vec![WorldAction::Move(Direction::E)]
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
        assert!(!actions.is_empty(), "forage branch should enqueue Eat");
        assert_eq!(actions[0], WorldAction::eat(OrdinaryFoodTypeId::default()));
    }

    #[test]
    fn v3alpha1_forage_branch_eat_targets_default_food_type() {
        let actions = run_founder_actions(FounderProfile::V3Alpha1, 1.0, 0.0, 0.1, 0.2, 0.95, 0.3);
        let eat_type = actions
            .into_iter()
            .find_map(|action| match action {
                WorldAction::Eat { type_idx } => Some(type_idx),
                _ => None,
            })
            .expect("forage branch should emit an Eat action");
        assert_eq!(eat_type, OrdinaryFoodTypeId::default());
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
        assert!(!actions.is_empty(), "fallback branch should enqueue Move");
        assert!(matches!(actions[0], WorldAction::Move(_)));
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
    fn founder_seeks_reproductive_food_when_reserve_is_deficient() {
        let actions = run_founder_actions_with_state(
            FounderProfile::ForageFirstSparse,
            0.0,
            0.0,
            0.1,
            0.1,
            0.1,
            0.1,
            0.0,
            0.0,
            100.0,
            0.0,
            0.95,
            0.0,
            0.0,
        );
        assert_eq!(actions, vec![WorldAction::Move(Direction::E)]);
    }

    #[test]
    fn founder_eats_reproductive_food_when_it_is_local_and_reserve_is_deficient() {
        let actions = run_founder_actions_with_state(
            FounderProfile::ForageFirstSparse,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.75,
            0.0,
            100.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert_eq!(
            actions,
            vec![WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::new(1)
            }]
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
            assert_eq!(
                graph.compute_nodes.len(),
                6,
                "founder node0 should combine energy + age + reserve gates and typed food"
            );
            assert_eq!(
                graph.compute_nodes[0].kind,
                ComputeNodeKind::Threshold(expected_threshold)
            );
            assert_eq!(
                graph.compute_nodes[1].kind,
                ComputeNodeKind::Threshold(19.5)
            );
            assert_eq!(graph.compute_nodes[2].kind, ComputeNodeKind::Multiply);
            assert_eq!(
                graph.compute_nodes[3].kind,
                ComputeNodeKind::Threshold(f32::from_bits(4.0f32.to_bits() - 1))
            );
            assert_eq!(graph.compute_nodes[4].kind, ComputeNodeKind::Multiply);
            assert_eq!(graph.compute_nodes[5].kind, ComputeNodeKind::Max);
            assert_eq!(
                node0.input_refs,
                vec![
                    InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default(),)),
                    InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
                    InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
                    InputReference::World(WorldInputKey::neighbor_food_ring(
                        OrdinaryFoodTypeId::default(),
                    )),
                    InputReference::World(WorldInputKey::NeighborOccupiedRing),
                    InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::new(1))),
                    InputReference::DynamicIntrospection(
                        DynamicIntrospectionKey::ReproductiveReserveCurrent,
                    ),
                    InputReference::World(WorldInputKey::neighbor_food_ring(
                        OrdinaryFoodTypeId::new(1),
                    )),
                ],
                "founder node0 inputs should include age ticks"
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
                (vm.constants[6] - expected_transfer).abs() < f32::EPSILON,
                "profile {:?} expected transfer {}, got {}",
                profile,
                expected_transfer,
                vm.constants[6]
            );
        }
    }
}
