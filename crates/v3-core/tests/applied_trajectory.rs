//! Pins the pre-T14.F03 trajectory, including stochastic descendants and actions.
//! The expected digest was measured on planning commit f76c3f6 before accepting
//! the instrumentation. Telemetry and wall-clock fields are deliberately excluded.

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
        "5898914f4106b258d26d724379ac3cc50cf2f811f9b505294098e9cc137f41f7"
    );
}
