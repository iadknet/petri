#[test]
fn energy_drain_returns_success_when_sufficient() {
    use v3_core::creature::state::Energy;

    let mut energy = Energy::new(10);

    assert!(energy.drain(5));
    assert_eq!(energy.value(), 5);

    assert!(!energy.drain(10)); // Insufficient
    assert_eq!(energy.value(), 5); // Unchanged
}

#[test]
fn energy_charge_caps_at_max() {
    use v3_core::creature::state::Energy;

    let mut energy = Energy::new(5);
    energy.charge(100, 50);
    assert_eq!(energy.value(), 50); // Capped at max
}

#[test]
fn energy_is_alive_when_nonzero() {
    use v3_core::creature::state::Energy;

    let alive = Energy::new(1);
    assert!(alive.is_alive());

    let dead = Energy::new(0);
    assert!(!dead.is_alive());
}

#[test]
fn energy_drain_saturating_always_deducts() {
    use v3_core::creature::state::Energy;

    let mut energy = Energy::new(5);
    energy.drain_saturating(3);
    assert_eq!(energy.value(), 2);

    // Saturates to 0 when draining more than available
    energy.drain_saturating(10);
    assert_eq!(energy.value(), 0);
    assert!(!energy.is_alive());

    // Draining from 0 stays at 0
    energy.drain_saturating(1);
    assert_eq!(energy.value(), 0);
}

#[test]
fn creature_state_new_initializes_correctly() {
    use v3_core::creature::genome::CreatureGenome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;

    let genome = CreatureGenome::simple_founder();
    let creature = CreatureState::new(Position { x: 10, y: 20 }, 15, 0, [255, 128, 64], genome);

    assert_eq!(creature.position, Position { x: 10, y: 20 });
    assert_eq!(creature.energy.value(), 15);
    assert_eq!(creature.age, 0);
    assert_eq!(creature.generation, 0);
    assert_eq!(creature.phenotype_r, 255);
    assert_eq!(creature.phenotype_g, 128);
    assert_eq!(creature.phenotype_b, 64);
    assert_eq!(creature.genome, CreatureGenome::simple_founder());
    assert_eq!(creature.memory.len(), 1024);
    assert!(creature.memory.iter().all(|&byte| byte == 0));
}

#[test]
fn create_offspring_inherits_memory_phenotype_and_generation() {
    use rand::SeedableRng;
    use v3_core::config::SimulationConfig;
    use v3_core::creature::genome::CreatureGenome;
    use v3_core::creature::reproduction::create_offspring;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;

    let mut parent = CreatureState::new(
        Position { x: 2, y: 3 },
        40,
        7,
        [10, 20, 30],
        CreatureGenome::simple_founder(),
    );
    parent.memory[0] = 11;
    parent.memory[512] = 99;
    parent.memory[1023] = 250;

    let mut config = SimulationConfig::default();
    config.runtime.mutation.mutation_probability = 0.0; // Disable mutation for clone test
    let mut rng = rand::rngs::SmallRng::seed_from_u64(7);

    let child = create_offspring(&parent, Position { x: 4, y: 5 }, 12, &config, &mut rng);

    assert_eq!(child.position, Position { x: 4, y: 5 });
    assert_eq!(child.energy.value(), 12);
    assert_eq!(child.generation, parent.generation + 1);
    assert_eq!(child.phenotype_r, parent.phenotype_r);
    assert_eq!(child.phenotype_g, parent.phenotype_g);
    assert_eq!(child.phenotype_b, parent.phenotype_b);
    assert_eq!(child.genome, parent.genome);
    assert_eq!(child.memory, parent.memory);
}

#[test]
fn mutation_with_high_probability_changes_genome() {
    use rand::SeedableRng;
    use v3_core::config::runtime::mutation::MutationConfig;
    use v3_core::creature::genome::CreatureGenome;
    use v3_core::creature::mutation::mutate_genome;

    let original = CreatureGenome::simple_founder();
    let mut mutated = original.clone();
    let config = MutationConfig {
        mutation_probability: 1.0,
        per_birth_mutation_events_min: 1,
        per_birth_mutation_events_max: 4,
        constant_jitter_magnitude: 0.1,
    };
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

    mutate_genome(&mut mutated, &config, &mut rng);

    assert_ne!(mutated, original, "mutation should change the genome");
    assert_eq!(
        mutated.constants.len(),
        original.constants.len(),
        "mutation should not change number of constants"
    );
}

#[test]
fn mutation_with_zero_probability_preserves_genome() {
    use rand::SeedableRng;
    use v3_core::config::runtime::mutation::MutationConfig;
    use v3_core::creature::genome::CreatureGenome;
    use v3_core::creature::mutation::mutate_genome;

    let original = CreatureGenome::simple_founder();
    let mut unchanged = original.clone();
    let config = MutationConfig {
        mutation_probability: 0.0,
        per_birth_mutation_events_min: 1,
        per_birth_mutation_events_max: 4,
        constant_jitter_magnitude: 0.5,
    };
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

    mutate_genome(&mut unchanged, &config, &mut rng);

    assert_eq!(unchanged, original, "zero probability should not mutate");
}

#[test]
fn offspring_genome_diverges_when_mutation_enabled() {
    use rand::SeedableRng;
    use v3_core::config::SimulationConfig;
    use v3_core::creature::genome::CreatureGenome;
    use v3_core::creature::reproduction::create_offspring;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::types::Position;

    let parent = CreatureState::new(
        Position { x: 2, y: 3 },
        40,
        0,
        [204, 61, 61],
        CreatureGenome::simple_founder(),
    );

    let mut config = SimulationConfig::default();
    config.runtime.mutation.mutation_probability = 1.0; // Force mutation
    let mut rng = rand::rngs::SmallRng::seed_from_u64(99);

    let child = create_offspring(&parent, Position { x: 4, y: 5 }, 12, &config, &mut rng);

    assert_ne!(
        child.genome, parent.genome,
        "offspring genome should differ when mutation is forced"
    );
    assert_eq!(child.memory, parent.memory, "memory should still be copied");
    assert_eq!(child.generation, 1);
}
