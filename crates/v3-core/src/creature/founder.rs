use crate::config::{EnergyLifecycleConfig, FounderProfile, OrdinaryFoodTypeId};
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, NodeId, RouteTarget, StaticIntrospectionKey,
    WorldInputKey,
};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};

use super::cgp_founder::{
    build_cgp_founder_decision_graph, build_cgp_founder_graph_with_thresholds,
};

/// `genome_size()` of the canonical V3Alpha1 founder. The genome replication
/// cost (T03.F11) charges only the units above this anchor, so the founder's
/// reproduce charge is unchanged; `creature::state` tests pin the founder's
/// measured size to this constant.
pub const FOUNDER_GENOME_SIZE_UNITS: u32 = 97;

/// Return the canonical v3alpha1 founder genome.
///
/// 2-node mesh: Node 0 (Graph sensor aggregator) -> Node 1 (Graph vote decision).
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
    /// The fraction of the parent's post-cost energy the child starts with:
    /// node 1's constant on `ActionParam(Reproduce, 1)` (T19.F04).
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
            node1_graph_decision(forage_first, policy.transfer_fraction),
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

/// Node 1 reads node 0's six slots and the `ActionQueue` compound input
/// (`FOUNDER_QUEUE_REF`) and votes the founder's decision (T19.F04).
fn node1_graph_decision(forage_first: bool, reproduce_transfer_fraction: f32) -> NodeGenome {
    let mut input_refs: Vec<InputReference> = (0..6).map(InputReference::UpstreamSlot).collect();
    input_refs.push(InputReference::ActionQueue);
    debug_assert_eq!(
        input_refs.len(),
        usize::from(super::cgp_founder::FOUNDER_QUEUE_REF) + 1
    );
    NodeGenome {
        node_id: NodeId::new(1),
        input_refs,
        backend_def: BackendDef::Graph(build_cgp_founder_decision_graph(
            forage_first,
            reproduce_transfer_fraction,
        )),
        targets: vec![],
    }
}

/// The V3Alpha1 founder with its decision node as a VM program voting the
/// same formulas as [`build_cgp_founder_decision_graph`]: a test fixture for
/// the VM mutation operators, which need a VM node carrying the founder's
/// program shape (reads, a constant pool with the transfer fraction at index
/// 5, votes, and a parameter write).
#[cfg(test)]
pub(crate) fn vm_decision_founder_genome() -> CreatureGenome {
    use crate::creature::genome::vote::VoteSink;
    use crate::creature::genome::{VmBackendDef, VmInstruction};
    let fraction = founder_reproduce_policy(FounderProfile::V3Alpha1).transfer_fraction;
    let mut program: Vec<VmInstruction> = (0..7u8)
        .map(|register| VmInstruction::ReadInput {
            dst: register,
            ref_idx: u16::from(register),
            sub_idx: 0,
        })
        .collect();
    let vote = |sink: VoteSink, src: u8| VmInstruction::AddVote {
        sink: sink.index() as u8,
        src,
    };
    let load = |dst: u8, const_idx: u8| VmInstruction::LoadConst { dst, const_idx };
    program.extend([
        // r7 = 0, r9 = f = [food > 0].
        load(7, 0),
        VmInstruction::CmpGt { dst: 9, a: 0, b: 7 },
        // r11 = q = [type > 2.5] - [type > 3.5].
        load(10, 2),
        VmInstruction::CmpGt {
            dst: 11,
            a: 6,
            b: 10,
        },
        load(10, 3),
        VmInstruction::CmpGt {
            dst: 12,
            a: 6,
            b: 10,
        },
        VmInstruction::Sub {
            dst: 11,
            a: 11,
            b: 12,
        },
        vote(VoteSink::Terminate, 11),
        // r12 = 2 (g + q) with g = can (r1); Eat = f - r12.
        VmInstruction::Add {
            dst: 12,
            a: 1,
            b: 11,
        },
        VmInstruction::Add {
            dst: 12,
            a: 12,
            b: 12,
        },
        VmInstruction::Sub {
            dst: 13,
            a: 9,
            b: 12,
        },
        vote(VoteSink::Eat, 13),
        load(14, 4),
        load(16, 6),
    ]);
    for (ring, d) in [(2u8, 0u8), (3, 2), (4, 4), (5, 6)] {
        program.extend([
            // r15 = 0.4 ring + 0.5.
            VmInstruction::Mul {
                dst: 15,
                a: ring,
                b: 14,
            },
            VmInstruction::Add {
                dst: 15,
                a: 15,
                b: 16,
            },
            VmInstruction::Sub {
                dst: 17,
                a: 15,
                b: 12,
            },
            vote(VoteSink::Move(d), 17),
            VmInstruction::Mul {
                dst: 18,
                a: 15,
                b: 1,
            },
            vote(VoteSink::Reproduce(d), 18),
        ]);
    }
    program.extend([
        load(19, 5),
        // params[Reproduce][1].
        VmInstruction::WriteActionParam {
            slot_idx: 5,
            src: 19,
        },
        VmInstruction::Halt,
    ]);
    let mut genome = v3alpha1_founder_genome();
    genome.nodes[1].backend_def = BackendDef::Vm(VmBackendDef {
        register_count: 20,
        constants: vec![0.0, 1.0, 2.5, 3.5, 0.4, fraction, 0.5],
        program,
    });
    genome
}

#[cfg(test)]
mod tests {
    use super::super::cgp_founder::age_gate_threshold;
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
    /// strict energy threshold as a fraction of `max_energy`, the T17.F01 raw
    /// threshold it re-expresses at the default `max_energy` (200), transfer
    /// fraction). Pinned here independently of `founder_reproduce_policy`
    /// so a change in either shows up.
    const PROFILES: [(FounderProfile, f32, f32, f32); 5] = [
        (FounderProfile::V3Alpha1, 0.16, 32.0, 2.0 / 3.0),
        (FounderProfile::ForageFirstSparse, 0.16, 32.0, 2.0 / 3.0),
        (
            FounderProfile::ForageFirstSparseConservative,
            0.30,
            60.0,
            0.35,
        ),
        (
            FounderProfile::ForageFirstSparseRichOffspring,
            0.20,
            40.0,
            0.60,
        ),
        (FounderProfile::ForageFirstSparseBalanced, 0.25, 50.0, 0.45),
    ];

    /// T17.F02 invariant 6, energy half: on the unit scale the founder makes
    /// the same gate decision as on the raw scale at every f32 energy in
    /// [t - 1, t + 1] around its raw threshold, except Conservative at
    /// 60.000004 (one ulp above 60; the quotient rounds down onto 0.30).
    #[test]
    fn founder_energy_gates_scan_identically_on_the_unit_scale() {
        let max_energy = EnergyLifecycleConfig::default().max_energy;
        for (profile, unit, raw, _) in PROFILES {
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
                let threshold = age_gate_threshold(min_age, reference);
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
        // Outside the documented constraint (`min_reproduce_age >
        // age_reference_ticks`) the threshold exceeds the saturated 1.0 read,
        // so the founder never attempts a birth at any age.
        let reference = 500_u64;
        let threshold = age_gate_threshold(501, reference);
        assert!(threshold > 1.0);
        for age in [0_u64, 500, 501, 10_000, u64::MAX] {
            assert!(
                age_fraction(age, reference) <= threshold,
                "age={age} passed a gate above the saturated read"
            );
        }
    }

    #[test]
    fn founder_reproduce_policy_matches_the_profile_table() {
        for (profile, threshold, _, fraction) in PROFILES {
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
        run_founder(&genome, &lifecycle, food, energy, age, cardinal, limit)
    }

    /// One tick of `genome` at the default lifecycle and action limit.
    fn actions_of(
        genome: &CreatureGenome,
        food: f32,
        energy: f32,
        age: u64,
        cardinal: [f32; 4],
    ) -> Vec<WorldAction> {
        run_founder(
            genome,
            &EnergyLifecycleConfig::default(),
            food,
            energy,
            age,
            cardinal,
            10,
        )
    }

    fn run_founder(
        genome: &CreatureGenome,
        lifecycle: &EnergyLifecycleConfig,
        food: f32,
        energy: f32,
        age: u64,
        cardinal: [f32; 4],
        limit: usize,
    ) -> Vec<WorldAction> {
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
            genome,
            &sensors,
            &mut energy,
            &mut [0.0; 16],
            &[0.0; 16],
            &mut runtime,
            &config,
        )
        .actions
    }

    #[test]
    fn founder_profiles_execute_energy_age_boundaries_and_priorities() {
        // The `actions` helper takes raw energy, so the boundaries sit on the
        // raw threshold.
        for (profile, _, threshold, transfer) in PROFILES {
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
        for (profile, _, threshold, _) in PROFILES {
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

    /// T19.F04 invariant 9: the truth table the vote founder reproduces,
    /// every profile, with `d` the first cardinal argmax.
    #[test]
    fn founder_vote_truth_table() {
        let cardinal = [0.2, 0.9, 0.1, 0.4];
        let d = Direction::E;
        for (profile, _, threshold, transfer) in PROFILES {
            let reproduce = vec![WorldAction::Reproduce {
                direction: d,
                energy_transfer_fraction: transfer,
            }];
            let forage = vec![
                WorldAction::eat(OrdinaryFoodTypeId::default()),
                WorldAction::Move(d),
            ];
            let can_energy = threshold + 1.0;
            for (can, food) in [(true, 1.0), (true, 0.0), (false, 1.0), (false, 0.0)] {
                let energy = if can { can_energy } else { 20.0 };
                let queue = actions(profile, food, energy, 20, 20, cardinal, 10);
                let expected = match (profile == FounderProfile::V3Alpha1, can, food > 0.0) {
                    (true, true, _) | (false, true, false) => reproduce.clone(),
                    (true, false, true) | (false, _, true) | (false, false, false) => {
                        forage.clone()
                    }
                    (true, false, false) => vec![WorldAction::Move(d)],
                };
                assert_eq!(queue, expected, "{profile:?} can={can} food={food}");
            }
        }
    }

    /// 2,000 rings from a fixed stream: the committed direction is the first
    /// cardinal argmax on every profile and branch.
    #[test]
    fn founder_two_thousand_ring_direction_check() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0x7419_f004);
        for _ in 0..2_000 {
            let cardinal: [f32; 4] = std::array::from_fn(|_| rng.gen_range(0.0f32..1.0));
            assert_directions(cardinal);
        }
    }

    #[test]
    fn founder_structure_is_a_sensor_graph_feeding_a_vote_graph() {
        assert_eq!(
            v3alpha1_founder_genome(),
            founder_genome(FounderProfile::V3Alpha1)
        );
        for (profile, ..) in PROFILES {
            let genome = founder_genome(profile);
            assert_eq!(genome.nodes.len(), 2);
            assert_eq!(genome.nodes[0].input_refs.len(), 5);
            assert_eq!(genome.nodes[1].input_refs.len(), 7);
            assert_eq!(genome.nodes[1].input_refs[6], InputReference::ActionQueue);
            let BackendDef::Graph(graph) = &genome.nodes[0].backend_def else {
                panic!("graph")
            };
            assert_eq!(graph.compute_nodes.len(), 3);
            assert_eq!(
                graph
                    .output_sinks
                    .iter()
                    .filter(|sink| !sink.inputs.is_empty())
                    .count(),
                6
            );
            let BackendDef::Graph(decision) = &genome.nodes[1].backend_def else {
                panic!("graph")
            };
            // Eat, four Move, four Reproduce, Terminate, and one parameter.
            assert_eq!(
                decision
                    .output_sinks
                    .iter()
                    .filter(|sink| !sink.inputs.is_empty())
                    .count(),
                11
            );
            assert!(genome.nodes[1].targets.is_empty());
        }
    }

    /// The VM fixture votes the founder's formulas, so it commits exactly
    /// the vote founder's queue on every truth-table state and ring.
    #[test]
    fn the_vm_decision_fixture_acts_as_the_vote_founder() {
        use rand::{Rng, SeedableRng};
        let graph = v3alpha1_founder_genome();
        let vm = vm_decision_founder_genome();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(19);
        for _ in 0..200 {
            let cardinal: [f32; 4] = std::array::from_fn(|_| rng.gen_range(0.0f32..1.0));
            for (food, energy) in [(1.0, 20.0), (0.0, 20.0), (1.0, 40.0), (0.0, 40.0)] {
                assert_eq!(
                    actions_of(&graph, food, energy, 20, cardinal),
                    actions_of(&vm, food, energy, 20, cardinal),
                    "{cardinal:?} food={food} energy={energy}"
                );
            }
        }
    }
}
