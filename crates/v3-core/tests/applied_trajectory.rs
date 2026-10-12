//! Pins the sampled trajectory, including stochastic descendants and actions.
//! Telemetry and wall-clock fields are deliberately excluded.
//!
//! The digest below pins the current production-default trajectory after two
//! runs agree. Intentional changes to production defaults must update it only
//! after the new trajectory has been reproduced. T11.F20 re-pinned the
//! first digest after two agreeing runs: its fixture moved from the retired
//! two-to-four-event per-birth rule to `per_unit_rate = 0.03`. T11.F25
//! re-pinned both digests: the `AddGraphEdge` surface draw skips the
//! undecoded parameter sinks (restoring that draw alone restores both old
//! digests). The food-fixture fix re-pinned both digests: the fixtures'
//! full initial food coverage now reaches the seeder through `types[0]`
//! (it had been written to the retired shared copy, leaving coverage at the
//! 0.54 default); dropping those two lines restores both old digests.
//! T11.F27 re-pinned the first digest on top of that fix for the serialized
//! genome shape alone (three `ActionParam(ActionParamField)` sinks, 94 sinks
//! per graph); the digest recomputed without genome bytes matches the
//! food-fixture tree, so T11.F27 did not move the simulation.
//! T24.F01 re-pinned both digests: `grow` keeps spread into occupied cells
//! (the accumulate at ecology.rs line 495); restoring the assignment alone
//! restores both old digests.
//! T24.F02 re-pinned both digests: food draws come from the keyed per-cell
//! stream instead of the run RNG, so every later run-RNG draw shifts.
//! T03.F12 re-pinned both digests after two agreeing runs: every birth is
//! delivered one founder gestation later. Each test also runs the fixture at
//! gestation rate 0.0, the kept equivalence evidence that rate 0.0 is the
//! pre-gestation engine (the T24.F02 digests until T20.F16).
//! Re-pinned by T20.F16 after two agreeing runs: Graph-node `Add` draws three
//! more numbers and appends a vote edge, which moves all four digests.
//! Re-pinned by T11.F29 after two agreeing runs: the `AddFeedbackRoute`
//! entry remaps every topology draw, which moves all four digests.
//! Re-pinned by T16.F02 after two agreeing runs: every Eat pays the 0.05
//! effort, a reproduce attempt pays its effort before the gates, and the
//! failed-action penalty is gone, which moves all four digests.
//! Re-pinned after two agreeing runs for the effort-cost defaults, 2026-10-10:
//! floor 0.4, attempt 1.0, move 1.6, which moves all four digests.
//! Re-pinned by T11.F31 after two agreeing runs: five ring projection
//! operators join the Graph catalog, which changes every Graph-domain draw
//! and moves all four digests.
//! Re-pinned by T11.F30 after two agreeing runs: the founder heading draws
//! shift the run RNG and the three drive sinks remap every sink and surface
//! draw, which moves all four digests.

use sha2::{Digest, Sha256};
use slotmap::Key;
use v3_core::config::{OrdinaryFoodTypeId, SimulationConfig};
use v3_core::contracts::Position;
use v3_core::simulation::{run_tick, seed_simulation};

#[test]
fn accounting_preserves_pre_feature_sampled_trajectories_and_actions() {
    assert_eq!(
        sampled_trajectory_digest(None),
        "733ca59ff24c7da3c0ca398424fd46e32dabaac6c175c2643759f42fd76f60d3"
    );
    assert_eq!(
        sampled_trajectory_digest(Some(0.0)),
        "a27c63e6e244728443cfa104bdcb5956f2814e11ed78bdb5900eda46ec2f2fce"
    );
}

/// The sampled-trajectory digest at the default gestation rate or `rate`.
fn sampled_trajectory_digest(gestation_rate: Option<f32>) -> String {
    let mut hash = Sha256::new();
    for seed in [7, 19] {
        let mut config = SimulationConfig::default();
        if let Some(rate) = gestation_rate {
            config.energy.lifecycle.gestation_ticks_per_founder_genome = rate;
        }
        config.world.width = 24;
        config.world.height = 24;
        config.population.initial_creatures = 24;
        config.world.food.types[0].initial_coverage = 1.0;
        config.world.food.types[0].initial_density = 1.0;
        // About three requested events per founder birth, the retired
        // two-to-four-event fixture's exposure (T11.F20).
        config.mutation.per_unit_rate = 0.03;
        let mut sim = seed_simulation(config, seed);
        for _ in 0..64 {
            run_tick(&mut sim, &mut None);
            hash.update(sim.tick.to_le_bytes());
            hash.update(sim.stats.reproduction_actions_spawned_total.to_le_bytes());
            for (id, creature) in &sim.creatures {
                hash.update(id.data().as_ffi().to_le_bytes());
                hash.update(
                    serde_json::to_vec(&(
                        creature.position,
                        creature.energy.to_bits(),
                        creature.age,
                        creature.generation,
                        &creature.genome,
                        creature.shared_memory.map(f32::to_bits),
                        creature.prev_shared_memory.map(f32::to_bits),
                        &creature.graph_runtime.node_state,
                        &creature.graph_runtime.node_outputs,
                        &creature.graph_runtime.plasticity_weights,
                        &creature.graph_runtime.eligibility_traces,
                    ))
                    .unwrap(),
                );
                if let Some(log) = sim.action_logs.get(id) {
                    hash.update(serde_json::to_vec(log.entries()).unwrap());
                }
            }
            for y in 0..24 {
                for x in 0..24 {
                    hash.update(
                        sim.world
                            .food_at_type(Position::new(x, y), OrdinaryFoodTypeId::default())
                            .to_bits()
                            .to_le_bytes(),
                    );
                }
            }
        }
        assert!(sim.stats.reproduction_actions_spawned_total > 0);
        assert!(sim.stats.mutation_events_applied_total > 0);
    }
    hex::encode(hash.finalize())
}

/// Applied-behavior guard: a mutation-on short run digested over positions,
/// energy bits, and ages only, with no genome bytes. T19.F03 held it through
/// the inert vote surface; T19.F04 moves it by construction (the founder's
/// actions now come from votes) and re-pins it after two agreeing runs;
/// T19.F05 re-pins both digests here for the 27-entry input-reference draw.
#[test]
fn mutation_on_applied_trajectory_guard_is_pinned() {
    assert_eq!(
        applied_guard_digest(None),
        "fce2603114d41a9eea3c03dbf844bee3ccb392954eff29d69cf3526d59654135"
    );
    assert_eq!(
        applied_guard_digest(Some(0.0)),
        "876245cea60b9fbf7f95f2ff435b57e2ae147c1a56c3f5add3de871a3af03673"
    );
}

/// The applied-behavior guard digest at the default gestation rate or `rate`.
fn applied_guard_digest(gestation_rate: Option<f32>) -> String {
    let mut hash = Sha256::new();
    for seed in [7, 19] {
        let mut config = SimulationConfig::default();
        if let Some(rate) = gestation_rate {
            config.energy.lifecycle.gestation_ticks_per_founder_genome = rate;
        }
        config.world.width = 24;
        config.world.height = 24;
        config.population.initial_creatures = 24;
        config.world.food.types[0].initial_coverage = 1.0;
        config.world.food.types[0].initial_density = 1.0;
        let mut sim = seed_simulation(config, seed);
        for _ in 0..64 {
            run_tick(&mut sim, &mut None);
            hash.update(sim.tick.to_le_bytes());
            hash.update(sim.stats.reproduction_actions_spawned_total.to_le_bytes());
            hash.update(sim.stats.mutation_events_applied_total.to_le_bytes());
            for (id, creature) in &sim.creatures {
                hash.update(id.data().as_ffi().to_le_bytes());
                hash.update(creature.position.x.to_le_bytes());
                hash.update(creature.position.y.to_le_bytes());
                hash.update(creature.energy.to_bits().to_le_bytes());
                hash.update(creature.age.to_le_bytes());
            }
        }
        assert!(sim.stats.reproduction_actions_spawned_total > 0);
        assert!(sim.stats.mutation_events_applied_total > 0);
    }
    hex::encode(hash.finalize())
}
