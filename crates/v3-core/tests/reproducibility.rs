//! Cross-process reproducibility of a seeded run (T10.F11).
//!
//! Two `Simulation`s seeded with the same seed and config must produce
//! byte-identical trajectories at every tick, including living descendants
//! after the required operator categories have fired. Final persistence is
//! not this stress fixture's gate: T11.F15 reaches extinction at tick 250,
//! while default viability and goal-profile persistence are checked separately. Because std `HashMap`'s `RandomState` derives a
//! fresh key pair per map — even inside one thread — a run whose RNG draws are
//! indexed into a list built by iterating a `HashMap` diverges between the two
//! simulations built here, exactly as it diverges between two processes. So
//! this test reproduces the T10.F09 cross-process finding in one process, in
//! seconds.
//!
//! Configuration: production `SimulationConfig::default()` economics with a
//! 96-by-96 world, 150 founders, full initial food coverage and density, and
//! `mutation_probability = 1.0`, 4 to 10 mutation events per birth, and
//! `mesh_layer_probability = 0.5`, so the corrected operators are reached
//! within the unchanged 250-tick horizon. Every founder
//! genome gets four paired slot groups appended to its first VM program, which
//! is what makes a `Vm.MutatePairedSlotAddress` candidate list exist at all
//! (the v3alpha1 founder program has no slot instructions), so that operator is
//! exercised from the first birth onward.

use v3_core::config::SimulationConfig;
use v3_core::contracts::Position;
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use v3_core::mutation::MutationOperator;
use v3_core::simulation::{run_tick, seed_simulation, Simulation};

const SEED: u64 = 20_260_904;
const TICKS: u64 = 250;
/// Slots that get a load and a store appended to every founder VM program, so
/// `Vm.MutatePairedSlotAddress` has four candidates to pick from at birth.
const INJECTED_PAIRED_SLOTS: [u8; 4] = [2, 5, 9, 13];

fn reproducibility_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 96;
    cfg.world.height = 96;
    cfg.population.initial_creatures = 150;
    cfg.world.food.initial_coverage = 1.0;
    cfg.world.food.initial_density = 1.0;
    cfg.mutation.per_unit_supply_enabled = false;
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 4;
    cfg.mutation.per_birth_mutation_events_max = 10;
    cfg.mutation.mesh_layer_probability = 0.5;
    cfg
}

/// Seed a simulation and inject the paired slot groups.
fn seeded_fixture(seed: u64) -> Simulation {
    let mut sim = seed_simulation(reproducibility_config(), seed);
    for (_, creature) in sim.creatures.iter_mut() {
        let vm = creature
            .genome
            .nodes
            .iter_mut()
            .find_map(|n| match n.backend_def {
                BackendDef::Vm(ref mut vm) => Some(vm),
                BackendDef::Graph(_) => None,
            })
            .expect("every founder genome has a VM node");
        vm.program
            .extend(INJECTED_PAIRED_SLOTS.into_iter().flat_map(|slot_idx| {
                [
                    VmInstruction::LoadSlotImm { dst: 0, slot_idx },
                    VmInstruction::StoreSlotImm { slot_idx, src: 0 },
                ]
            }));
    }
    sim
}

/// One entry per creature, in `SlotMap` order: position, energy as
/// raw bits, age, generation, and the genome, compared by the genome's derived
/// `PartialEq`. Floats go through `to_bits`, so equality is bit-for-bit.
type CreatureFingerprint<'a> = (Position, u32, u64, u64, &'a CreatureGenome);

fn population_fingerprint(sim: &Simulation) -> Vec<CreatureFingerprint<'_>> {
    sim.creatures
        .values()
        .map(|c| {
            (
                c.position,
                c.energy.to_bits(),
                c.age,
                c.generation,
                &c.genome,
            )
        })
        .collect()
}

/// The deterministic work counters a report carries: the six compute counters
/// the benchmark harness compares, plus terminal cognition and exhaustion totals.
fn work_counters(sim: &Simulation) -> [(&'static str, u64); 13] {
    [
        ("mesh_hops", sim.stats.mesh_hops_total),
        ("vm_steps", sim.stats.vm_steps_total),
        ("graph_relax_iters", sim.stats.graph_relax_iters_total),
        ("plasticity_updates", sim.stats.plasticity_updates_total),
        ("plasticity_changes", sim.stats.plasticity_changes_total),
        ("hebbian_updates", sim.stats.hebbian_updates_total),
        ("hebbian_changes", sim.stats.hebbian_changes_total),
        (
            "reward_modulated_updates",
            sim.stats.reward_modulated_updates_total,
        ),
        (
            "reward_modulated_changes",
            sim.stats.reward_modulated_changes_total,
        ),
        (
            "shared_memory_writes_changed",
            sim.stats.shared_memory_writes_changed_total,
        ),
        ("actions_applied", sim.stats.actions_applied_total),
        ("births", sim.stats.reproduction_actions_spawned_total),
        (
            "mesh_dispatches_energy_exhausted",
            sim.stats.mesh_dispatches_energy_exhausted_total,
        ),
    ]
}

fn applied(sim: &Simulation, op: MutationOperator) -> u64 {
    sim.stats
        .mutation_events_applied_total_by_operator
        .get(&op)
        .copied()
        .unwrap_or(0)
}

fn accounting_fingerprint(sim: &Simulation) -> Vec<u64> {
    let f = &sim.stats.energy_flows;
    let mut bits = vec![
        sim.stats.mortality.deaths_total,
        f.genome_size_creature_ticks,
    ];
    bits.extend(sim.stats.mortality.by_cause);
    bits.extend(
        sim.stats
            .reproductive_success_by_cognitive_class
            .by_class
            .iter()
            .flat_map(|row| {
                [
                    row.creatures_observed_total,
                    row.offspring_spawned_sum,
                    row.survival_ticks_sum,
                ]
            }),
    );
    bits.extend(f.food_intake_by_type.iter().map(|value| value.to_bits()));
    bits.extend(
        [
            f.action_charges.noop,
            f.action_charges.eat,
            f.action_charges.r#move,
            f.action_charges.reproduce,
            f.action_charges.steal_energy,
            f.failed_action_penalty,
            f.vm_compute,
            f.priority_bid,
            f.graph_compute,
            f.hebbian_learning,
            f.reward_learning,
            f.lifecycle_decay,
            f.genome_carrying,
            f.parental_transfer_debit,
            f.offspring_energy_credit,
            f.predation_victim_debit,
            f.predation_attacker_credit,
            f.predation_kill_bonus_credit,
            f.maximum_energy_clamp_loss,
            f.zero_floor_credit,
            f.external_removal_loss,
        ]
        .map(f64::to_bits),
    );
    bits
}

#[test]
fn two_simulations_with_the_same_seed_are_byte_identical() {
    let mut first = seeded_fixture(SEED);
    let mut second = seeded_fixture(SEED);
    let mut compared_mutated_descendant = false;
    for tick in 0..=TICKS {
        assert_eq!(
            accounting_fingerprint(&first),
            accounting_fingerprint(&second),
            "accounting tick {tick}"
        );
        assert_eq!(
            work_counters(&first),
            work_counters(&second),
            "work counters diverged at tick {tick}"
        );
        assert_eq!(
            first.creature_count(),
            second.creature_count(),
            "population diverged at tick {tick}"
        );
        for (index, (a, b)) in population_fingerprint(&first)
            .iter()
            .zip(population_fingerprint(&second).iter())
            .enumerate()
        {
            assert_eq!(a, b, "creature {index} diverged at tick {tick}");
        }
        compared_mutated_descendant |= applied(&first, MutationOperator::VmMutatePairedSlotAddress)
            > 0
            && applied(&first, MutationOperator::TopologyCopyMeshBackwardSlice)
                + applied(&first, MutationOperator::TopologyCopyMeshForwardSlice)
                > 0
            && first
                .creatures
                .values()
                .any(|creature| creature.generation > 0);
        if tick < TICKS {
            run_tick(&mut first, &mut None);
            run_tick(&mut second, &mut None);
        }
    }

    let paired = applied(&first, MutationOperator::VmMutatePairedSlotAddress);
    let backward = applied(&first, MutationOperator::TopologyCopyMeshBackwardSlice);
    let forward = applied(&first, MutationOperator::TopologyCopyMeshForwardSlice);
    eprintln!(
        "seed {SEED}, {TICKS} ticks: births={}, final_population={}, \
         applied operators={}, Vm.MutatePairedSlotAddress={paired}, \
         Topology.CopyMeshBackwardSlice={backward}, Topology.CopyMeshForwardSlice={forward}",
        first.stats.reproduction_actions_spawned_total,
        first.creature_count(),
        first.stats.mutation_events_applied_total_by_operator.len(),
    );
    assert!(
        paired > 0,
        "the run never applied Vm.MutatePairedSlotAddress, so it does not cover the \
         corrected candidate order"
    );
    assert!(
        backward + forward > 0,
        "the run never applied a Topology.CopyMesh*Slice operator, so it does not cover \
         the corrected backlink target"
    );

    assert!(
        compared_mutated_descendant,
        "must compare living descendants after paired-slot and mesh-slice mutations have applied"
    );
}

fn terrain_config() -> SimulationConfig {
    let mut cfg = reproducibility_config();
    cfg.world.world_seed = Some(817);
    cfg.world.terrain = vec![v3_core::config::TerrainLayer {
        params: v3_core::patterns::PatternParams::Noise {
            density: 0.001,
            cluster_size: 20,
        },
        bounds: Some(v3_core::patterns::PatternBounds {
            x: 0,
            y: 0,
            width: 40,
            height: 40,
        }),
        seed: None,
    }];
    cfg.world.terrain.push(v3_core::config::TerrainLayer {
        params: v3_core::patterns::PatternParams::FbmThreshold {
            octaves: 4,
            frequency: 0.04,
            lacunarity: 2.0,
            persistence: 0.5,
            threshold: 0.2,
        },
        bounds: Some(v3_core::patterns::PatternBounds {
            x: 40,
            y: 40,
            width: 40,
            height: 40,
        }),
        seed: Some(901),
    });
    cfg.world.food.types.push(v3_core::config::FoodTypeConfig {
        energy_per_unit: Some(12.0),
        growth_rate: Some(0.025),
        recovery_spawn_rate: Some(0.002),
        initial_fertility_only: true,
        ..v3_core::config::FoodTypeConfig::default()
    });
    cfg
}

fn world_fingerprint(sim: &Simulation) -> Vec<(bool, u32, u32)> {
    (0..sim.world.height)
        .flat_map(|y| {
            (0..sim.world.width).map(move |x| {
                let p = Position::new(x, y);
                (
                    sim.world.is_barrier(p),
                    sim.world.food().fertility().get(x, y).to_bits(),
                    sim.world.food_at(p).to_bits(),
                )
            })
        })
        .collect()
}

#[test]
fn terrain_is_identical_across_independent_initialization_and_thread_counts() {
    let one = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let four = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let mut first = one.install(|| seed_simulation(terrain_config(), SEED));
    let mut second = four.install(|| seed_simulation(terrain_config(), SEED));
    // The exact count this fixture's barrier layers produce. Pinned rather
    // than bounded so a terrain change that alters the map cannot pass by
    // still drawing "some" barriers.
    assert_eq!(
        world_fingerprint(&first)
            .iter()
            .filter(|cell| cell.0)
            .count(),
        117
    );
    for tick in 0..=20 {
        assert_eq!(
            world_fingerprint(&first),
            world_fingerprint(&second),
            "world tick {tick}"
        );
        assert_eq!(
            population_fingerprint(&first),
            population_fingerprint(&second),
            "founders/state tick {tick}"
        );
        assert_eq!(work_counters(&first), work_counters(&second));
        assert_eq!(
            accounting_fingerprint(&first),
            accounting_fingerprint(&second),
            "accounting across thread counts tick {tick}"
        );
        if tick < 20 {
            one.install(|| run_tick(&mut first, &mut None));
            four.install(|| run_tick(&mut second, &mut None));
        }
    }
}

#[test]
fn fixed_map_seed_isolates_maps_from_run_seeded_placement() {
    let mut cfg = terrain_config();
    cfg.world.food.types[0].initial_coverage = 0.5;
    let first = seed_simulation(cfg.clone(), 11);
    let second = seed_simulation(cfg, 22);
    let a = world_fingerprint(&first);
    let b = world_fingerprint(&second);
    assert_eq!(
        a.iter()
            .map(|&(barrier, fertility, _)| (barrier, fertility))
            .collect::<Vec<_>>(),
        b.iter()
            .map(|&(barrier, fertility, _)| (barrier, fertility))
            .collect::<Vec<_>>()
    );
    assert_ne!(
        a.iter().map(|cell| cell.2).collect::<Vec<_>>(),
        b.iter().map(|cell| cell.2).collect::<Vec<_>>()
    );
    assert_ne!(
        population_fingerprint(&first),
        population_fingerprint(&second)
    );
}
