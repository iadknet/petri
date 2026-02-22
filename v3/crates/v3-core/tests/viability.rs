//! End-to-end viability tests for the Stage 4 simulation.
//!
//! These tests confirm that the full seeding + tick loop produces viable behaviour:
//! creatures eat, move, and reproduce over multiple ticks without panicking.

use v3_core::config::SimulationConfig;
use v3_core::contracts::Position;
use v3_core::simulation::{run_tick, seed_simulation};

/// Return a compact config suitable for fast viability tests.
///
/// Uses elevated `initial_energy` and `max_energy` so that founder creatures
/// survive at least 5 ticks even in the worst case where no founder lands on a
/// food cell (so they reproduce instead of eating).  Per-tick worst-case cost is
/// roughly: 0.2 (decay) + 36 (graph, 2 passes × 18 nodes) + 2 (VM opcodes) +
/// 22 (reproduce cost + transfer) ≈ 60 energy.  500 energy gives a clear buffer
/// for ≥5 ticks without eating.  Default `initial_energy = 20.0` is calibrated
/// for the final tuned game; viability tests need a much larger value to
/// bootstrap founder creatures reliably across arbitrary seeds.
fn viability_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 30;
    cfg.world.height = 30;
    cfg.population.initial_creatures = 10;
    cfg.world.food.initial_coverage = 0.5;
    cfg.world.food.initial_density = 80;
    cfg.world.food.growth_rate = 0.1;
    // 500 energy ≫ 5 × worst-case per-tick cost (≈60) so founders survive 5
    // ticks regardless of whether they land on food cells.
    cfg.energy.lifecycle.initial_energy = 500.0;
    cfg.energy.lifecycle.max_energy = 500.0;
    cfg
}

/// Run for 20 ticks with default config — must not panic.
#[test]
fn sim_runs_20_ticks_without_panic() {
    let mut sim = seed_simulation(viability_config(), 42);
    for _ in 0..20 {
        run_tick(&mut sim);
    }
    // If we reach this point, no panic occurred.
}

/// After 5 ticks at least some creatures must still be alive.
#[test]
fn population_survives_5_ticks() {
    let mut sim = seed_simulation(viability_config(), 42);
    for _ in 0..5 {
        run_tick(&mut sim);
    }
    assert!(
        sim.creature_count() > 0,
        "all creatures died after 5 ticks — founders are too fragile"
    );
}

/// The total food count after 1 tick should differ from the initial seeded food
/// (either consumed by creatures or grown by the food-growth pass).
#[test]
fn food_gets_consumed_and_regrows() {
    let mut sim = seed_simulation(viability_config(), 42);
    let food_initial = sim.world.total_food();
    run_tick(&mut sim);
    // After one tick food must have changed (creatures eat or growth fires).
    assert_ne!(
        sim.world.total_food(),
        food_initial,
        "food total should change after one tick"
    );
}

/// All founders must start with the energy specified in the config.
#[test]
fn seeded_founders_start_with_correct_energy() {
    let cfg = viability_config();
    let expected = cfg.energy.lifecycle.initial_energy;
    let sim = seed_simulation(cfg, 42);
    for (_, creature) in &sim.creatures {
        assert!(
            (creature.energy - expected).abs() < f32::EPSILON,
            "founder energy {} != expected {}",
            creature.energy,
            expected
        );
    }
}

/// Place a founder on a cell pre-seeded with food, run one tick, and confirm
/// that food was consumed or energy changed.
#[test]
fn creatures_can_eat_food() {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;
    use v3_core::contracts::CreatureId;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;
    use v3_core::simulation::Simulation;

    let cfg = {
        let mut c = SimulationConfig::default();
        c.world.width = 10;
        c.world.height = 10;
        c.world.food.initial_coverage = 0.0;
        c.world.food.growth_rate = 0.0;
        c.population.initial_creatures = 0;
        c
    };

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let pos = Position::new(5, 5);
    // Manually place food on all cells.
    {
        let mut seed_cfg = cfg.clone();
        seed_cfg.world.food.initial_coverage = 1.0;
        seed_cfg.world.food.initial_density = 200;
        let mut rng = SmallRng::seed_from_u64(0);
        world.seed_food(&mut rng, &seed_cfg);
    }
    let food_before = world.food_at(pos);

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let creature_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            pos,
            200.0,
            0,
            [204, 61, 61],
            [1.0f32; 3],
            [true; 3],
        )
    });
    world.place_creature(pos, creature_id);

    let energy_before = creatures[creature_id].energy;

    let mut sim = Simulation::new(world, creatures, 0, cfg, 42);

    run_tick(&mut sim);

    let food_after = sim.world.food_at(pos);
    let energy_after = if sim.creatures.contains_key(creature_id) {
        sim.creatures[creature_id].energy
    } else {
        // Creature died — energy change irrelevant, but food should have changed.
        0.0
    };

    assert!(
        food_after < food_before || energy_after > energy_before,
        "after 1 tick: food {food_before}→{food_after}, energy {energy_before}→{energy_after} — no eating detected"
    );
}

/// Two simulations seeded with the same seed must produce identical creature
/// counts and total food immediately after seeding (before any ticks).
#[test]
fn deterministic_seeding_reproducible() {
    let cfg = viability_config();
    let s1 = seed_simulation(cfg.clone(), 42);
    let s2 = seed_simulation(cfg, 42);
    assert_eq!(
        s1.creature_count(),
        s2.creature_count(),
        "creature counts differ between identical seeds"
    );
    assert_eq!(
        s1.world.total_food(),
        s2.world.total_food(),
        "food totals differ between identical seeds"
    );
}
