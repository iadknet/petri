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

#[cfg(test)]
#[allow(dead_code)]
fn legacy_node1_vm_decision() -> NodeGenome {
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
                VmInstruction::JumpIfZero {
                    cond: 6,
                    offset: 10,
                }, // [18] skip body → PC 29
                VmInstruction::Max { dst: 6, a: 2, b: 3 }, // [19] direction heuristic
                VmInstruction::Max { dst: 7, a: 4, b: 5 }, // [20]
                VmInstruction::CmpGt { dst: 6, a: 6, b: 7 }, // [21]
                VmInstruction::PushAction { action_type: 1 }, // [22] push Eat(type 0 via default meta)
                VmInstruction::LoadConst {
                    dst: 6,
                    const_idx: 1,
                }, // type 1
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 6,
                },
                VmInstruction::PushAction { action_type: 1 }, // push Eat(type 1)
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 7,
                }, // [23] meta[0]=dir
                VmInstruction::PushAction { action_type: 2 }, // [24] push Move
                VmInstruction::ExecuteActionQueue,            // [25] terminal → done
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
#[allow(dead_code)]
fn legacy_node1_vm_decision_forage_first(reproduce_transfer_energy: f32) -> NodeGenome {
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
                VmInstruction::PushAction { action_type: 1 }, // Eat(type 0 via default meta)
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                }, // dir for Move
                VmInstruction::PushAction { action_type: 2 }, // Move
                VmInstruction::ExecuteActionQueue,         // terminal
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
                VmInstruction::PushAction { action_type: 1 }, // Eat(type 0 via default meta)
                VmInstruction::WriteWorldActionMeta {
                    slot_idx: 0,
                    src: 0,
                }, // dir for Move
                VmInstruction::PushAction { action_type: 2 }, // Move
                VmInstruction::ExecuteActionQueue,         // terminal
            ],
        }),
        targets: vec![],
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
