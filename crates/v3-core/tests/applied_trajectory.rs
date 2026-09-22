//! Pins the sampled trajectory, including stochastic descendants and actions.
//! Telemetry and wall-clock fields are deliberately excluded.
//!
//! The digest below pins the current production-default trajectory after two
//! runs agree. Intentional changes to production defaults must update it only
//! after the new trajectory has been reproduced.

use sha2::{Digest, Sha256};
use slotmap::Key;
use v3_core::config::{OrdinaryFoodTypeId, SimulationConfig};
use v3_core::contracts::Position;
use v3_core::simulation::{run_tick, seed_simulation};

#[test]
fn accounting_preserves_pre_feature_sampled_trajectories_and_actions() {
    let mut hash = Sha256::new();
    for seed in [7, 19] {
        let mut config = SimulationConfig::default();
        config.world.width = 24;
        config.world.height = 24;
        config.population.initial_creatures = 24;
        config.world.food.initial_coverage = 1.0;
        config.world.food.initial_density = 1.0;
        config.mutation.per_unit_supply_enabled = false;
        config.mutation.mutation_probability = 1.0;
        config.mutation.per_birth_mutation_events_min = 2;
        config.mutation.per_birth_mutation_events_max = 4;
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
    let digest = hex::encode(hash.finalize());
    assert_eq!(
        digest,
        "14541daa2c01ba8b8a01ade8ced9efd2176d27f155f77a9c7aa64dc325307226"
    );
}

/// Applied-behavior guard: a mutation-on short run digested over positions,
/// energy bits, and ages only, with no genome bytes. T19.F03 held it through
/// the inert vote surface; T19.F04 moves it by construction (the founder's
/// actions now come from votes) and re-pins it after two agreeing runs;
/// T19.F05 re-pins both digests here for the 27-entry input-reference draw.
#[test]
fn mutation_on_applied_trajectory_guard_is_pinned() {
    let mut hash = Sha256::new();
    for seed in [7, 19] {
        let mut config = SimulationConfig::default();
        config.world.width = 24;
        config.world.height = 24;
        config.population.initial_creatures = 24;
        config.world.food.initial_coverage = 1.0;
        config.world.food.initial_density = 1.0;
        config.mutation.mutation_probability = 1.0;
        config.mutation.per_birth_mutation_events_min = 2;
        config.mutation.per_birth_mutation_events_max = 4;
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
    let digest = hex::encode(hash.finalize());
    assert_eq!(
        digest,
        "03761998705760977cbe31da198d2afda2da189b3b6ac31c011e557bea9fb431"
    );
}
