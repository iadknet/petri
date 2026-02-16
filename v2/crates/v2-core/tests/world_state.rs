use std::collections::{HashMap, HashSet};

use v2_core::phenotype::FOUNDER_PHENOTYPE_RGB;
use v2_core::world_seed::WorldSeedConfig;
use v2_core::world_state::{tick_world, WorldCell, WorldCreature, WorldTickConfig, WorldTickOutcome};

#[test]
fn creature_tick_decay_can_remove_starved_creatures_without_food() {
    let mut creatures = vec![WorldCreature {
        id: 1,
        x: 1,
        y: 1,
        energy: 0.20,
        phenotype_rgb: FOUNDER_PHENOTYPE_RGB,
    }];
    let mut food = HashMap::new();
    let barriers = HashSet::new();
    let seed_cfg = WorldSeedConfig {
        initial_food_density: 0.0,
        food_growth_rate: 0.0,
        food_spawn_rate: 0.0,
        food_spread_threshold: 1.0,
        food_spawn_floor_density: 0.0,
    };
    let tick_cfg = WorldTickConfig {
        initial_energy: 20.0,
        energy_decay_per_tick: 0.20,
        move_cost: 0.02,
        food_energy_gain: 0.0,
        reproduce_cost: 0.12,
        min_reproduce_energy: 18.0,
        offspring_energy_fraction: 0.45,
        energy_max: 20.0,
    };

    let outcome = tick_world(
        4,
        4,
        false,
        1,
        123,
        16,
        seed_cfg,
        tick_cfg,
        &mut creatures,
        &mut food,
        &barriers,
    );

    assert!(matches!(outcome, WorldTickOutcome { deaths: 1, .. }));
    assert!(creatures.is_empty(), "starved creature should be removed");
}

#[test]
fn world_tick_can_produce_births_when_food_and_energy_are_available() {
    let mut creatures = vec![WorldCreature {
        id: 1,
        x: 1,
        y: 1,
        energy: 20.0,
        phenotype_rgb: FOUNDER_PHENOTYPE_RGB,
    }];
    let mut food = HashMap::from([(WorldCell { x: 1, y: 1 }, u8::MAX)]);
    let barriers = HashSet::new();

    let outcome = tick_world(
        4,
        4,
        false,
        1,
        1234,
        16,
        WorldSeedConfig::default(),
        WorldTickConfig::default(),
        &mut creatures,
        &mut food,
        &barriers,
    );

    assert!(outcome.births > 0, "tick should report births");
    assert!(
        outcome.action_counts.reproduce > 0,
        "tick should report reproduce action counts"
    );
    assert!(
        creatures.len() > 1,
        "creature list should include offspring after reproduction"
    );
}
