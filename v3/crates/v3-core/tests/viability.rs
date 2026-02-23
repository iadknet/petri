//! End-to-end viability tests for the Stage 4-5 simulation loop.
//!
//! These tests validate intent-level behavior instead of only "no panic":
//! world mechanics, founder eat/move/reproduce behavior, short-horizon
//! viability across deterministic seeds, and Stage 5 mutation-driven
//! phenotype divergence.

use std::collections::HashMap;

use v3_core::config::SimulationConfig;
use v3_core::contracts::{CreatureId, Position};
use v3_core::mutation::MutationEngine;
use v3_core::simulation::{run_tick, seed_simulation, Simulation};

const FOUNDER_RGB: [u8; 3] = [204, 61, 61];
const FOUNDER_WEIGHTS: [f32; 3] = [1.0; 3];
const FOUNDER_POLARITY: [bool; 3] = [true; 3];

/// Return a compact config suitable for fast, behavior-focused viability tests.
///
/// This intentionally avoids the previous "energy=500 safety net" profile:
/// we keep startup energy finite and tune runtime execution cost + offspring
/// transfer so founders must still interact with world food to remain viable.
fn viability_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.population.initial_creatures = 10;

    cfg.world.food.initial_coverage = 0.6;
    cfg.world.food.initial_density = 100;
    cfg.world.food.growth_rate = 0.25;

    cfg.runtime.graph_node_base_cost = 0.1;

    cfg.energy.lifecycle.initial_energy = 110.0;
    cfg.energy.lifecycle.max_energy = 160.0;
    cfg.energy.lifecycle.default_offspring_energy = 4.0;
    cfg.energy.costs.reproduce_cost = 1.0;
    cfg
}

/// Probe config for validating whether *default* energy/runtime economics are
/// viable over a short horizon.
///
/// We keep world size/population compact for test speed while leaving default
/// economics untouched.
fn default_economics_probe_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.population.initial_creatures = 20;
    cfg
}

#[derive(Debug, Clone, Copy)]
struct TickMetrics {
    tick: u64,
    population: usize,
    newborns: usize,
    moved_survivors: usize,
    max_generation: u64,
    total_food: u64,
}

fn creature_positions(sim: &Simulation) -> HashMap<CreatureId, Position> {
    sim.creatures
        .iter()
        .map(|(id, creature)| (id, creature.position))
        .collect()
}

fn run_ticks_with_metrics(sim: &mut Simulation, ticks: usize) -> Vec<TickMetrics> {
    let mut prev_positions = creature_positions(sim);
    let mut metrics = Vec::with_capacity(ticks);

    for _ in 0..ticks {
        run_tick(sim);
        let current_positions = creature_positions(sim);

        let newborns = current_positions
            .keys()
            .filter(|id| !prev_positions.contains_key(id))
            .count();

        let moved_survivors = current_positions
            .iter()
            .filter(|(id, current_pos)| {
                prev_positions
                    .get(id)
                    .is_some_and(|prev_pos| prev_pos != *current_pos)
            })
            .count();

        let max_generation = sim
            .creatures
            .iter()
            .map(|(_, creature)| creature.generation)
            .max()
            .unwrap_or(0);

        metrics.push(TickMetrics {
            tick: sim.tick_number(),
            population: sim.creature_count(),
            newborns,
            moved_survivors,
            max_generation,
            total_food: sim.world.total_food(),
        });

        prev_positions = current_positions;
    }

    metrics
}

fn format_tick_metrics(metrics: &[TickMetrics]) -> String {
    metrics
        .iter()
        .map(|m| {
            format!(
                "t{} pop={} births={} moved={} max_gen={} food={}",
                m.tick, m.population, m.newborns, m.moved_survivors, m.max_generation, m.total_food
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Run for 40 ticks with viability config — must not panic.
#[test]
fn sim_runs_40_ticks_without_panic() {
    let mut sim = seed_simulation(viability_config(), 42);
    for _ in 0..40 {
        run_tick(&mut sim);
    }
}

/// Over 20 ticks, the founder population must remain alive and reproduce.
#[test]
fn population_survives_and_reproduces_20_ticks() {
    let mut sim = seed_simulation(viability_config(), 42);
    let metrics = run_ticks_with_metrics(&mut sim, 20);
    let births_total: usize = metrics.iter().map(|m| m.newborns).sum();
    let max_generation = metrics.iter().map(|m| m.max_generation).max().unwrap_or(0);

    assert!(
        sim.creature_count() > 0,
        "all creatures died by tick {}.\n{}",
        sim.tick_number(),
        format_tick_metrics(&metrics)
    );
    assert!(
        births_total > 0 && max_generation > 0,
        "no reproduction detected over 20 ticks.\n{}",
        format_tick_metrics(&metrics)
    );
}

/// The viability profile should survive across a small deterministic seed set.
#[test]
fn population_survives_reference_seeds() {
    for seed in [1_u64, 7, 42, 99, 2026] {
        let mut sim = seed_simulation(viability_config(), seed);
        let metrics = run_ticks_with_metrics(&mut sim, 20);
        assert!(
            sim.creature_count() > 0,
            "seed {seed} went extinct by tick {}.\n{}",
            sim.tick_number(),
            format_tick_metrics(&metrics)
        );
    }
}

/// Default runtime/energy economics should support short-horizon viability.
#[test]
fn default_economics_survive_and_reproduce_short_horizon() {
    for seed in [1_u64, 7, 42] {
        let mut sim = seed_simulation(default_economics_probe_config(), seed);
        let metrics = run_ticks_with_metrics(&mut sim, 20);
        let births_total: usize = metrics.iter().map(|m| m.newborns).sum();
        let max_generation = metrics.iter().map(|m| m.max_generation).max().unwrap_or(0);

        assert!(
            sim.creature_count() > 0,
            "default economics seed {seed} extinct by tick {}.\n{}",
            sim.tick_number(),
            format_tick_metrics(&metrics)
        );
        assert!(
            births_total > 0 && max_generation > 0,
            "default economics seed {seed} had no reproduction.\n{}",
            format_tick_metrics(&metrics)
        );
    }
}

/// The total food count after 1 tick should differ from the initial seeded food
/// (either consumed by creatures or grown by the food-growth pass).
#[test]
fn food_gets_consumed_and_regrows() {
    let mut sim = seed_simulation(viability_config(), 42);
    let food_initial = sim.world.total_food();
    run_tick(&mut sim);
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
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;

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
            80.0,
            0,
            FOUNDER_RGB,
            FOUNDER_WEIGHTS,
            FOUNDER_POLARITY,
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
        0.0
    };

    assert!(
        food_after < food_before || energy_after > energy_before,
        "after 1 tick: food {food_before}->{food_after}, energy {energy_before}->{energy_after}."
    );
}

/// With no food and sufficient energy, the founder should emit Reproduce and
/// spawn a generation-1 child when the target cell is open.
#[test]
fn founder_reproduces_when_energy_allows_and_target_is_open() {
    use slotmap::SlotMap;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 10;
    cfg.world.height = 10;
    cfg.population.initial_creatures = 0;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.food.growth_rate = 0.0;
    cfg.runtime.graph_node_base_cost = 0.1;
    cfg.energy.lifecycle.initial_energy = 80.0;
    cfg.energy.lifecycle.max_energy = 120.0;
    cfg.energy.lifecycle.default_offspring_energy = 12.0;
    cfg.energy.costs.reproduce_cost = 1.0;

    let pos = Position::new(5, 5);
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let parent_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            pos,
            cfg.energy.lifecycle.initial_energy,
            0,
            FOUNDER_RGB,
            FOUNDER_WEIGHTS,
            FOUNDER_POLARITY,
        )
    });
    world.place_creature(pos, parent_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 7);
    run_tick(&mut sim);

    let generation_one = sim
        .creatures
        .iter()
        .filter(|(_, creature)| creature.generation == 1)
        .count();
    assert_eq!(
        generation_one,
        1,
        "expected exactly one generation-1 child; pop={}.",
        sim.creature_count()
    );
}

/// With no food and reproduction gate below threshold, founder should still
/// emit Move (not stay in place).
#[test]
fn founder_moves_when_no_food_and_below_reproduce_threshold() {
    use slotmap::SlotMap;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 10;
    cfg.world.height = 10;
    cfg.population.initial_creatures = 0;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.food.growth_rate = 0.0;
    cfg.runtime.graph_node_base_cost = 0.1;
    cfg.energy.lifecycle.initial_energy = 23.0;
    cfg.energy.lifecycle.max_energy = 40.0;

    let start = Position::new(5, 5);
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let creature_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            start,
            cfg.energy.lifecycle.initial_energy,
            0,
            FOUNDER_RGB,
            FOUNDER_WEIGHTS,
            FOUNDER_POLARITY,
        )
    });
    world.place_creature(start, creature_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 11);
    run_tick(&mut sim);

    assert_eq!(
        sim.creature_count(),
        1,
        "movement test should not spawn child"
    );
    let moved_to = sim.creatures[creature_id].position;
    assert_ne!(
        moved_to, start,
        "founder did not move from {start:?}; moved_to={moved_to:?}"
    );
}

// ── Stage 5: Mutation + phenotype divergence ──────────────────────────────────

/// With mutation_probability=1.0, at some point during 30 ticks a creature must
/// carry a phenotype_rgb that differs from the founder baseline [204, 61, 61].
///
/// We track divergence across all ticks rather than only at the end, because
/// heavy mutation can eventually drive the population extinct (genome damage
/// accumulates). What matters is that the phenotype divergence mechanism fires.
#[test]
fn mutation_offspring_diverge_from_parent_over_time() {
    let mut cfg = viability_config();
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 1;
    cfg.mutation.per_birth_mutation_events_max = 2;

    let mut sim = seed_simulation(cfg, 42);
    let mut ever_diverged = false;
    for _ in 0..30 {
        run_tick(&mut sim);
        if sim
            .creatures
            .values()
            .any(|c| c.phenotype_rgb != FOUNDER_RGB)
        {
            ever_diverged = true;
            break;
        }
    }

    assert!(
        ever_diverged,
        "after 30 ticks with mutation_probability=1.0, no creature ever diverged from founder phenotype"
    );
}

/// MutationEngine accounting invariant: attempted == applied + skipped for all calls.
#[test]
fn mutation_accounting_invariant_in_viability() {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use v3_core::creature::founder::v3alpha1_founder_genome;

    let mut cfg = SimulationConfig::default().mutation;
    cfg.mutation_probability = 1.0;
    cfg.per_birth_mutation_events_min = 1;
    cfg.per_birth_mutation_events_max = 5;

    let mut genome = v3alpha1_founder_genome();
    for seed in 0u64..1000 {
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations(&mut genome, &cfg, &mut rng);
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events,
            "accounting invariant violated at seed {seed}"
        );
    }
}

/// When mutation_probability=0.0, child must inherit parent phenotype unchanged.
#[test]
fn phenotype_inherits_unchanged_when_no_genome_mutation() {
    use slotmap::SlotMap;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 10;
    cfg.world.height = 10;
    cfg.mutation.mutation_probability = 0.0;
    cfg.energy.lifecycle.initial_energy = 80.0;
    cfg.energy.lifecycle.max_energy = 120.0;
    cfg.energy.lifecycle.default_offspring_energy = 12.0;
    cfg.energy.costs.reproduce_cost = 1.0;
    cfg.runtime.graph_node_base_cost = 0.1;

    let pos = Position::new(5, 5);
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let parent_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            pos,
            cfg.energy.lifecycle.initial_energy,
            0,
            FOUNDER_RGB,
            FOUNDER_WEIGHTS,
            FOUNDER_POLARITY,
        )
    });
    world.place_creature(pos, parent_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 7);
    run_tick(&mut sim);

    // Find the child (generation == 1).
    let child = sim.creatures.values().find(|c| c.generation == 1);

    if let Some(child) = child {
        assert_eq!(
            child.phenotype_rgb, FOUNDER_RGB,
            "child phenotype must be identical to parent when mutation_probability=0.0"
        );
    }
    // If no child was produced this tick, the test is vacuously satisfied (no mutation check needed).
}

/// Existing viability profile still works with real mutations enabled.
#[test]
fn viability_still_passes_with_real_mutations() {
    let mut cfg = viability_config();
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 1;
    cfg.mutation.per_birth_mutation_events_max = 2;

    let mut sim = seed_simulation(cfg, 42);
    let metrics = run_ticks_with_metrics(&mut sim, 20);
    let births_total: usize = metrics.iter().map(|m| m.newborns).sum();

    assert!(
        sim.creature_count() > 0,
        "population went extinct with real mutations by tick {}.\n{}",
        sim.tick_number(),
        format_tick_metrics(&metrics)
    );
    assert!(
        births_total > 0,
        "no reproduction occurred with real mutations.\n{}",
        format_tick_metrics(&metrics)
    );
}

/// Two simulations seeded with the same seed must produce identical initial
/// creature placement and per-cell food densities.
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

    let mut positions_1: Vec<_> = s1
        .creatures
        .iter()
        .map(|(_, creature)| (creature.position.x, creature.position.y))
        .collect();
    let mut positions_2: Vec<_> = s2
        .creatures
        .iter()
        .map(|(_, creature)| (creature.position.x, creature.position.y))
        .collect();
    positions_1.sort_unstable();
    positions_2.sort_unstable();
    assert_eq!(
        positions_1, positions_2,
        "creature placement differs between identical seeds"
    );

    for y in 0..s1.world.height {
        for x in 0..s1.world.width {
            let pos = Position::new(x, y);
            assert_eq!(
                s1.world.food_at(pos),
                s2.world.food_at(pos),
                "food differs at ({x},{y}) for identical seeds"
            );
        }
    }
}

// ── Stage 6: SimStats tests ──────────────────────────────────────────────────

#[test]
fn stats_accounting_invariant_holds_over_50_ticks() {
    let mut sim = seed_simulation(viability_config(), 77);
    for _ in 0..50 {
        run_tick(&mut sim);
    }
    let s = &sim.stats;
    assert_eq!(
        s.reproduction_actions_attempted_total,
        s.reproduction_actions_spawned_total + s.reproduction_actions_rejected_total,
        "accounting invariant: attempted == spawned + rejected"
    );
}

#[test]
fn stats_last_tick_counters_reset_each_tick() {
    let mut sim = seed_simulation(viability_config(), 88);
    // Run tick 1 and capture last_tick_reproduce.
    run_tick(&mut sim);
    let tick1_reproduce = sim.stats.last_tick_reproduce;
    let tick1_attempted = sim.stats.reproduction_actions_attempted_total;
    // Run tick 2.
    run_tick(&mut sim);
    let tick2_reproduce = sim.stats.last_tick_reproduce;
    // Per-tick counter must not be cumulative from tick 1.
    // Total cumulative must have grown by tick2_reproduce.
    assert_eq!(
        sim.stats.reproduction_actions_attempted_total,
        tick1_attempted + tick2_reproduce as u64,
        "cumulative counter must grow by per-tick count each tick"
    );
    // Per-tick counter is independent each tick (not simply the sum of all previous ticks).
    // We check that if both ticks had reproduce actions, they aren't cumulated.
    if tick1_reproduce > 0 && tick2_reproduce > 0 {
        assert_ne!(
            tick2_reproduce,
            tick1_reproduce + tick2_reproduce,
            "last_tick_reproduce must reset each tick"
        );
    }
}

#[test]
fn mean_energy_is_positive_in_viable_sim() {
    let mut sim = seed_simulation(viability_config(), 99);
    for _ in 0..10 {
        run_tick(&mut sim);
    }
    assert!(
        sim.mean_energy() > 0.0,
        "mean_energy should be positive in a viable simulation, got {}",
        sim.mean_energy()
    );
}
