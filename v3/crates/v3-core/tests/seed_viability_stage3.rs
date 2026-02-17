use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::config::SimulationConfig;
use v3_core::creature::genome::CreatureGenome;
use v3_core::kernel::world_state::WorldState;
use v3_core::seed::seed_creatures;
use v3_core::SimulationState;

/// Config tuned for Stage 3 viability: generous food, reproduction enabled,
/// mutation forced to guarantee genome divergence.
fn stage3_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.food.growth_rate = 0.08;
    config.world.food.initial_density = 150;
    config.world.food.initial_coverage = 0.6;
    config.energy.lifecycle.initial_energy = 50;
    config.energy.lifecycle.max_energy = 150;
    config.energy.lifecycle.energy_decay_per_tick = 1;
    config.energy.lifecycle.min_reproduce_energy = 40;
    config.energy.lifecycle.default_offspring_energy = 15;
    config.energy.costs.move_cost = 1;
    config.energy.costs.eat_cost = 0;
    config.energy.costs.eat_reward_per_food = 1;
    config.energy.costs.reproduce_cost = 2;
    // Force mutation on every reproduction to guarantee genome divergence
    config.runtime.mutation.mutation_probability = 1.0;
    config.runtime.mutation.per_birth_mutation_events_min = 1;
    config.runtime.mutation.per_birth_mutation_events_max = 4;
    config.runtime.mutation.constant_jitter_magnitude = 0.1;
    config
}

#[test]
fn stage3_viability_reproduction_and_genome_divergence() {
    let config = stage3_config();
    let (width, height) = (32, 32);
    let mut state = SimulationState::new(WorldState::new(width, height, true));
    let mut rng = SmallRng::seed_from_u64(54321);

    seed_creatures(&mut state, 15, &config, &mut rng);

    let initial_pop = state.creatures.len();
    assert_eq!(initial_pop, 15, "should seed all 15 creatures");

    let founder_genome = CreatureGenome::simple_founder();

    // Verify all seed creatures have founder genome
    for (_, c) in state.creatures.iter() {
        assert_eq!(
            c.genome, founder_genome,
            "seed creatures should have founder genome"
        );
        assert_eq!(c.generation, 0);
    }

    let mut total_births: u32 = 0;

    // Run 200 ticks — enough for reproduction and mutation to occur
    for _ in 0..200 {
        let stats = state.tick(&config, &mut rng);
        total_births += stats.births;
    }

    // Assert: population survived
    assert!(
        !state.creatures.is_empty(),
        "population collapsed to 0 — ecology not viable"
    );

    // Assert: births occurred
    assert!(
        total_births > 0,
        "no births occurred during 200 ticks — reproduction not working"
    );

    // Assert: at least one generation>0 creature exists
    let has_offspring = state.creatures.iter().any(|(_, c)| c.generation > 0);
    assert!(
        has_offspring,
        "no offspring survived — all generation>0 creatures died"
    );

    // Assert: at least one creature's genome differs from the founder
    let has_diverged = state
        .creatures
        .iter()
        .any(|(_, c)| c.generation > 0 && c.genome != founder_genome);
    assert!(
        has_diverged,
        "no genome divergence found — mutation not working"
    );
}
