use crate::config::{EnergyLifecycleConfig, FounderProfile, MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, NodeId, RouteTarget, StaticIntrospectionKey,
    WorldInputKey,
};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};

use super::cgp_founder::build_cgp_founder_graph_with_thresholds;

/// `genome_size()` of the canonical V3Alpha1 founder. The genome replication
/// cost (T03.F11) charges only the units above this anchor, so the founder's
/// reproduce charge is unchanged; `creature::state` tests pin the founder's
/// measured size to this constant.
pub const FOUNDER_GENOME_SIZE_UNITS: u32 = 111;

/// Return the canonical v3alpha1 founder genome.
///
/// 2-node mesh: Node 0 (Graph sensor aggregator) -> Node 1 (VM decision emitter).
/// Spec: v3-startup-seeding-spec.md Section 5.1.
pub fn v3alpha1_founder_genome() -> CreatureGenome {
    founder_genome(FounderProfile::V3Alpha1)
}

/// The energy gate and offspring investment a founder profile emits
/// (T17.F01, v3-startup-seeding-spec.md Section 5.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FounderReproducePolicy {
    /// The founder attempts to reproduce only while `EnergyCurrent` — its
    /// energy as a fraction of `max_energy` — is strictly above this (a
    /// `Threshold` compute node in node 0). The founder gates on fullness:
    /// the fraction is its own constant, not recomputed from `max_energy`.
    pub energy_threshold: f32,
    /// VM constant index 5: the fraction of the parent's post-cost energy
    /// the child starts with, written to `meta[1]` of the reproduce action.
    pub transfer_fraction: f32,
}

/// At the default `max_energy` (200) each unit threshold is the T17.F01 raw
/// gate (32/32/60/40/50) divided by 200, which sits at least 2.0 above
/// `min_reproduce_energy` (30), and each fraction clears the `initial_energy`
/// (20) litter floor at its own threshold after the age-1.0 reproduce cost,
/// so a founder attempt at or above its gate is never refused on energy
/// (T17.F01 invariant 5). A lifecycle where `energy_threshold * max_energy -
/// 1.0 < min_reproduce_energy` pins the founder at its gate.
#[must_use]
pub const fn founder_reproduce_policy(profile: FounderProfile) -> FounderReproducePolicy {
    let (energy_threshold, transfer_fraction) = match profile {
        FounderProfile::V3Alpha1 | FounderProfile::ForageFirstSparse => (0.16, 2.0 / 3.0),
        FounderProfile::ForageFirstSparseConservative => (0.30, 0.35),
        FounderProfile::ForageFirstSparseRichOffspring => (0.20, 0.60),
        FounderProfile::ForageFirstSparseBalanced => (0.25, 0.45),
    };
    FounderReproducePolicy {
        energy_threshold,
        transfer_fraction,
    }
}

/// Return a founder genome variant selected by profile, with the age gate
/// derived from the default lifecycle config.
#[must_use]
pub fn founder_genome(profile: FounderProfile) -> CreatureGenome {
    founder_genome_with_age_gate(profile, &EnergyLifecycleConfig::default())
}

/// Return a founder genome variant selected by profile with its age gate
/// derived from `lifecycle`: `min_reproduce_age` on the `age_reference_ticks`
/// scale the brain reads `AgeTicks` on (T17.F02 invariant 5).
#[must_use]
pub fn founder_genome_with_age_gate(
    profile: FounderProfile,
    lifecycle: &EnergyLifecycleConfig,
) -> CreatureGenome {
    let forage_first = profile != FounderProfile::V3Alpha1;
    let policy = founder_reproduce_policy(profile);
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            node0_graph_sensor(
                policy.energy_threshold,
                lifecycle.min_reproduce_age,
                lifecycle.age_reference_ticks,
            ),
            node1_vm_decision(forage_first, policy.transfer_fraction),
        ],
    }
}

fn node0_graph_sensor(
    reproduce_energy_threshold: f32,
    min_reproduce_age_ticks: u64,
    age_reference_ticks: u64,
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
        ],
        backend_def: BackendDef::Graph(build_cgp_founder_graph_with_thresholds(
            &MutationConfig::default(),
            reproduce_energy_threshold,
            min_reproduce_age_ticks,
            age_reference_ticks,
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

fn append_founder_direction(builder: &mut FounderVmBuilder, neighbors: [u8; 4]) {
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
}

fn append_founder_eat(builder: &mut FounderVmBuilder) {
    builder.push(VmInstruction::WriteWorldActionMeta {
        slot_idx: 0,
        src: 15,
    });
    builder.push(VmInstruction::PushAction { action_type: 1 });
}

fn append_founder_move(builder: &mut FounderVmBuilder) {
    append_founder_direction(builder, [2, 3, 4, 5]);
    builder.push(VmInstruction::PushAction { action_type: 2 });
    builder.push(VmInstruction::ExecuteActionQueue);
}

fn append_founder_reproduce(builder: &mut FounderVmBuilder, fallback: usize) {
    builder.push(VmInstruction::CmpGt {
        dst: 13,
        a: 1,
        b: 15,
    });
    builder.jump_if_zero(13, fallback);
    append_founder_direction(builder, [2, 3, 4, 5]);
    builder.push(VmInstruction::LoadConst {
        dst: 14,
        const_idx: 5,
    });
    builder.push(VmInstruction::WriteWorldActionMeta {
        slot_idx: 1,
        src: 14,
    });
    builder.push(VmInstruction::PushAction { action_type: 3 });
    builder.push(VmInstruction::ExecuteActionQueue);
}

fn node1_vm_decision(forage_first: bool, reproduce_transfer_fraction: f32) -> NodeGenome {
    let input_refs = (0..6).map(InputReference::UpstreamSlot).collect();
    let mut builder = FounderVmBuilder::new();
    for register in 0..6u8 {
        builder.push(VmInstruction::ReadInput {
            dst: register,
            ref_idx: u16::from(register),
            sub_idx: 0,
        });
    }
    // Keep a dedicated zero register: direction selection overwrites its scratch registers.
    builder.push(VmInstruction::Sub {
        dst: 15,
        a: 15,
        b: 15,
    });
    let forage = builder.label();
    let fallback = builder.label();
    if !forage_first {
        append_founder_reproduce(&mut builder, forage);
    }
    builder.mark(forage);
    builder.push(VmInstruction::CmpGt {
        dst: 13,
        a: 0,
        b: 15,
    });
    builder.jump_if_zero(13, fallback);
    append_founder_eat(&mut builder);
    append_founder_move(&mut builder);
    builder.mark(fallback);
    if forage_first {
        let explore = builder.label();
        append_founder_reproduce(&mut builder, explore);
        builder.mark(explore);
        append_founder_eat(&mut builder);
    }
    append_founder_move(&mut builder);
    NodeGenome {
        node_id: NodeId::new(1),
        input_refs,
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 20,
            constants: vec![0.0, 1.0, 2.0, 4.0, 6.0, reproduce_transfer_fraction],
            program: builder.finish(),
        }),
        targets: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{Direction, WorldAction};
    use crate::creature::state::GraphRuntimeState;
    use crate::runtime::mesh::execute_creature_mesh;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::{age_fraction, energy_fraction, StaticInputs};
    use crate::sensors::typed_food::TypedFoodLocalSnapshot;
    use proptest::prelude::*;

    /// The T17.F01 profile table on the unit scale (T17.F02): (profile,
    /// strict energy threshold as a fraction of `max_energy`, transfer
    /// fraction). Pinned here independently of `founder_reproduce_policy`
    /// so a change in either shows up.
    const PROFILES: [(FounderProfile, f32, f32); 5] = [
        (FounderProfile::V3Alpha1, 0.16, 2.0 / 3.0),
        (FounderProfile::ForageFirstSparse, 0.16, 2.0 / 3.0),
        (FounderProfile::ForageFirstSparseConservative, 0.30, 0.35),
        (FounderProfile::ForageFirstSparseRichOffspring, 0.20, 0.60),
        (FounderProfile::ForageFirstSparseBalanced, 0.25, 0.45),
    ];

    /// The T17.F01 raw thresholds each unit threshold re-expresses at the
    /// default `max_energy` (200).
    const RAW_THRESHOLDS: [(FounderProfile, f32); 5] = [
        (FounderProfile::V3Alpha1, 32.0),
        (FounderProfile::ForageFirstSparse, 32.0),
        (FounderProfile::ForageFirstSparseConservative, 60.0),
        (FounderProfile::ForageFirstSparseRichOffspring, 40.0),
        (FounderProfile::ForageFirstSparseBalanced, 50.0),
    ];

    /// T17.F02 invariant 6, energy half: on the unit scale the founder makes
    /// the same gate decision as on the raw scale at every f32 energy in
    /// [t - 1, t + 1] around its raw threshold, except Conservative at
    /// 60.000004 (one ulp above 60; the quotient rounds down onto 0.30).
    #[test]
    fn founder_energy_gates_scan_identically_on_the_unit_scale() {
        let max_energy = EnergyLifecycleConfig::default().max_energy;
        for (profile, raw) in RAW_THRESHOLDS {
            let unit = founder_reproduce_policy(profile).energy_threshold;
            assert_eq!(unit, raw / max_energy, "{profile:?}");
            let mut mismatches = Vec::new();
            let mut energy = raw - 1.0;
            while energy <= raw + 1.0 {
                if (energy > raw) != (energy_fraction(energy, max_energy) > unit) {
                    mismatches.push(energy);
                }
                energy = energy.next_up();
            }
            let expected: &[f32] = if profile == FounderProfile::ForageFirstSparseConservative {
                &[60.000004]
            } else {
                &[]
            };
            assert_eq!(mismatches, expected, "{profile:?}");
        }
    }

    /// T17.F02 invariant 6, age half: `age / S > (min_age - 0.5) / S` agrees
    /// with `age > min_age - 0.5` at every integer age for every reference
    /// span the spec lists, so the config-derived gate is exact.
    #[test]
    fn founder_age_gate_is_exact_at_every_integer_age() {
        for reference in [500_u64, 1_000, 1_024, 2_000, 2_048, 4_096] {
            for min_age in [0_u64, 1, 19, 20, 25, 100] {
                let threshold = super::super::cgp_founder::age_gate_threshold(min_age, reference);
                for age in 0..=(reference + 10) {
                    let raw_pass = age >= min_age;
                    let unit_pass = age_fraction(age, reference) > threshold;
                    assert_eq!(
                        raw_pass, unit_pass,
                        "reference={reference} min_age={min_age} age={age}"
                    );
                }
            }
        }
    }

    #[test]
    fn founder_reproduce_policy_matches_the_profile_table() {
        for (profile, threshold, fraction) in PROFILES {
            assert_eq!(
                founder_reproduce_policy(profile),
                FounderReproducePolicy {
                    energy_threshold: threshold,
                    transfer_fraction: fraction,
                },
                "{profile:?}"
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn actions(
        profile: FounderProfile,
        food: f32,
        energy: f32,
        age: u64,
        min_age: u64,
        cardinal: [f32; 4],
        limit: usize,
    ) -> Vec<WorldAction> {
        let lifecycle = EnergyLifecycleConfig {
            min_reproduce_age: min_age,
            ..EnergyLifecycleConfig::default()
        };
        let genome = founder_genome_with_age_gate(profile, &lifecycle);
        let mut energy = energy;
        let mut ring = [0.0; 8];
        for (i, value) in cardinal.into_iter().enumerate() {
            ring[i * 2] = value;
        }
        let sensors = SensorSnapshot {
            local: StaticInputs {
                food_here: food,
                neighbor_food: ring,
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                age_ticks: age_fraction(age, lifecycle.age_reference_ticks),
                max_energy: lifecycle.max_energy,
            },
            typed_local_food: TypedFoodLocalSnapshot {
                food_here_by_type: vec![food],
                neighbor_food_by_type: vec![ring],
            },
            perception: PerceptionSnapshot::zeroed(1),
        };
        let mut runtime = GraphRuntimeState::new();
        runtime.begin_tick(&genome.nodes, age);
        let config = RuntimeConfig {
            max_actions_per_turn: limit,
            ..RuntimeConfig::default()
        };
        execute_creature_mesh(
            &genome,
            &sensors,
            &mut energy,
            &mut [0.0; 16],
            &[0.0; 16],
            &mut runtime,
            &config,
        )
        .actions
    }

    /// The raw energy a unit threshold sits at under the default lifecycle;
    /// the `actions` helper takes raw energy.
    fn raw_threshold(unit_threshold: f32) -> f32 {
        unit_threshold * EnergyLifecycleConfig::default().max_energy
    }

    #[test]
    fn founder_profiles_execute_energy_age_boundaries_and_priorities() {
        for (profile, unit_threshold, transfer) in PROFILES {
            let threshold = raw_threshold(unit_threshold);
            let reproduction = vec![WorldAction::Reproduce {
                direction: Direction::W,
                energy_transfer_fraction: transfer,
            }];
            let forage = vec![
                WorldAction::eat(OrdinaryFoodTypeId::default()),
                WorldAction::Move(Direction::W),
            ];
            for (energy, age, min_age, eligible) in [
                (threshold, 20, 20, false),
                (threshold + 1.0, 19, 20, false),
                (threshold + 1.0, 20, 20, true),
                (threshold + 1.0, 0, 0, true),
            ] {
                let no_food = actions(profile, 0.0, energy, age, min_age, [0.1, 0.2, 0.3, 0.9], 10);
                let expected = if eligible {
                    reproduction.clone()
                } else if profile == FounderProfile::V3Alpha1 {
                    vec![WorldAction::Move(Direction::W)]
                } else {
                    forage.clone()
                };
                assert_eq!(no_food, expected, "{profile:?} energy={energy} age={age}");
                let local = actions(profile, 1.0, energy, age, min_age, [0.1, 0.2, 0.3, 0.9], 10);
                assert_eq!(
                    local,
                    if eligible && profile == FounderProfile::V3Alpha1 {
                        reproduction.clone()
                    } else {
                        forage.clone()
                    }
                );
            }
            assert_eq!(
                actions(profile, 1.0, 20.0, 0, 20, [0.0; 4], 1),
                vec![WorldAction::eat(OrdinaryFoodTypeId::default())]
            );
        }
    }

    fn expected_direction(cardinal: [f32; 4]) -> Direction {
        let mut best = 0;
        for index in 1..4 {
            if cardinal[index] > cardinal[best] {
                best = index;
            }
        }
        [Direction::N, Direction::E, Direction::S, Direction::W][best]
    }

    fn assert_directions(cardinal: [f32; 4]) {
        for (profile, unit_threshold, _) in PROFILES {
            let threshold = raw_threshold(unit_threshold);
            for (food, energy, age) in [(1.0, 20.0, 0), (0.0, 20.0, 0), (0.0, threshold + 1.0, 20)]
            {
                let output = actions(profile, food, energy, age, 20, cardinal, 10);
                let actual = output.iter().find_map(|action| match action {
                    WorldAction::Move(direction) | WorldAction::Reproduce { direction, .. } => {
                        Some(*direction)
                    }
                    _ => None,
                });
                assert_eq!(
                    actual,
                    Some(expected_direction(cardinal)),
                    "{profile:?} {cardinal:?} {output:?}"
                );
            }
        }
    }

    #[test]
    fn founder_cardinal_maxima_ties_and_adjacent_pair_trap() {
        for cardinal in [
            [0.9, 0.1, 0.2, 0.3],
            [0.1, 0.9, 0.2, 0.3],
            [0.1, 0.2, 0.9, 0.3],
            [0.1, 0.2, 0.3, 0.9],
            [0.8, 0.2, 0.9, 0.1],
            [0.0; 4],
            [0.5; 4],
            [0.1, 0.7, 0.7, 0.2],
        ] {
            assert_directions(cardinal);
        }
    }

    proptest! {
        #[test]
        fn founder_cardinal_selection_matches_first_argmax(cardinal in prop::array::uniform4(0.0f32..1.0)) { assert_directions(cardinal); }
    }

    #[test]
    fn founder_structure_keeps_only_primary_inputs_and_four_spare_registers() {
        assert_eq!(
            v3alpha1_founder_genome(),
            founder_genome(FounderProfile::V3Alpha1)
        );
        for (profile, _, _) in PROFILES {
            let genome = founder_genome(profile);
            assert_eq!(genome.nodes.len(), 2);
            assert_eq!(genome.nodes[0].input_refs.len(), 5);
            assert_eq!(genome.nodes[1].input_refs.len(), 6);
            let BackendDef::Graph(graph) = &genome.nodes[0].backend_def else {
                panic!("graph")
            };
            assert_eq!(graph.compute_nodes.len(), 3);
            assert!(graph.execute_gate.inputs.is_empty());
            assert!(graph
                .action_bank
                .iter()
                .all(|slot| slot.gate_inputs.is_empty() && slot.param_inputs.is_empty()));
            assert_eq!(
                graph
                    .output_sinks
                    .iter()
                    .filter(|sink| !sink.inputs.is_empty())
                    .count(),
                6
            );
            let BackendDef::Vm(vm) = &genome.nodes[1].backend_def else {
                panic!("vm")
            };
            assert_eq!(vm.register_count, 20);
            for instruction in &vm.program {
                assert!(
                    crate::creature::genome::analysis::vm_register_write(instruction)
                        .is_none_or(|register| register < 16)
                );
                assert_eq!(
                    crate::creature::genome::analysis::vm_register_read_mask(instruction) & !0xffff,
                    0
                );
            }
        }
    }
}
