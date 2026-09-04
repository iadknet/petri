//! Cross-process reproducibility of a seeded run (T10.F11).
//!
//! Two `Simulation`s seeded with the same seed and config must produce
//! byte-identical trajectories. Because std `HashMap`'s `RandomState` derives a
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
//! within the 250-tick horizon (measured: 65 births, `Vm.MutatePairedSlotAddress`
//! applied 4 times, `Topology.CopyMeshBackwardSlice` 7, `Topology.CopyMeshForwardSlice`
//! 13; about 8 seconds in a debug build). Every founder
//! genome gets four paired slot groups appended to its first VM program, which
//! is what makes a `Vm.MutatePairedSlotAddress` candidate list exist at all
//! (the v3alpha1 founder program has no slot instructions), so that operator is
//! exercised from the first birth onward.

use v3_core::config::SimulationConfig;
use v3_core::creature::genome::{BackendDef, VmInstruction};
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
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 4;
    cfg.mutation.per_birth_mutation_events_max = 10;
    cfg.mutation.mesh_layer_probability = 0.5;
    cfg
}

/// Seed a simulation, inject the paired slot groups, and run `TICKS` ticks.
fn run(seed: u64) -> Simulation {
    let mut sim = seed_simulation(reproducibility_config(), seed);
    for (_, creature) in sim.creatures.iter_mut() {
        let Some(node) = creature
            .genome
            .nodes
            .iter_mut()
            .find(|n| matches!(n.backend_def, BackendDef::Vm(_)))
        else {
            continue;
        };
        if let BackendDef::Vm(ref mut vm) = node.backend_def {
            vm.program
                .extend(INJECTED_PAIRED_SLOTS.into_iter().flat_map(|slot_idx| {
                    [
                        VmInstruction::LoadSlotImm { dst: 0, slot_idx },
                        VmInstruction::StoreSlotImm { slot_idx, src: 0 },
                    ]
                }));
        }
    }
    for _ in 0..TICKS {
        run_tick(&mut sim, &mut None);
    }
    sim
}

/// One line per creature, in `SlotMap` order: position, energy and reserve as
/// raw bits, age, generation, and the serialized genome.
fn population_fingerprint(sim: &Simulation) -> Vec<String> {
    sim.creatures
        .values()
        .map(|c| {
            format!(
                "{:?} e={:08x} r={:08x} age={} gen={} {}",
                c.position,
                c.energy.to_bits(),
                c.reproductive_reserve.to_bits(),
                c.age,
                c.generation,
                serde_json::to_string(&c.genome).expect("genome serializes"),
            )
        })
        .collect()
}

/// The six deterministic work counters the benchmark harness reports.
fn work_counters(sim: &Simulation) -> [(&'static str, u64); 6] {
    [
        ("mesh_hops", sim.stats.mesh_hops_total),
        ("vm_steps", sim.stats.vm_steps_total),
        ("graph_relax_iters", sim.stats.graph_relax_iters_total),
        ("plasticity_updates", sim.stats.plasticity_updates_total),
        ("actions_applied", sim.stats.actions_applied_total),
        ("births", sim.stats.reproduction_actions_spawned_total),
    ]
}

fn applied(sim: &Simulation, op: MutationOperator) -> u64 {
    sim.stats
        .mutation_events_applied_total_by_operator
        .get(&op)
        .copied()
        .unwrap_or(0)
}

#[test]
fn two_simulations_with_the_same_seed_are_byte_identical() {
    let first = run(SEED);
    let second = run(SEED);

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

    assert_eq!(
        work_counters(&first),
        work_counters(&second),
        "work counters diverged between two runs of seed {SEED}"
    );
    assert_eq!(
        first.creature_count(),
        second.creature_count(),
        "population size diverged between two runs of seed {SEED}"
    );
    assert!(
        first.creature_count() > 0,
        "the run went extinct, so it proves nothing about reproducibility"
    );
    for (i, (a, b)) in population_fingerprint(&first)
        .iter()
        .zip(population_fingerprint(&second).iter())
        .enumerate()
    {
        assert_eq!(
            a, b,
            "creature {i} diverged between two runs of seed {SEED}"
        );
    }
}
