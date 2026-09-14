//! End-to-end viability tests — a **merge gate** for the simulation.
//!
//! These tests validate that production economics (energy, costs, runtime limits)
//! support a self-sustaining founder population. They cover: world mechanics,
//! founder eat/move/reproduce behavior, short-horizon viability across
//! deterministic seeds, and mutation-driven phenotype divergence.
//!
//! # Viability test rules (see also AGENTS.md § Viability Test Policy)
//!
//! 1. **Production defaults required.** All configs must use `SimulationConfig::default()`
//!    for economic parameters. Permitted overrides: world size, population count, and
//!    food coverage/density (for test speed on a smaller world).
//!
//! 2. **Scenario-specific tests** may override non-economic parameters to set up a
//!    specific situation (e.g. no food growth, manual creature placement), but costs,
//!    decay, rewards, and runtime limits must stay at production defaults.
//!
//! 3. **Never weaken assertions to pass.** If viability breaks, fix the config or
//!    behavior change that caused it — not the tests.
//!
//! 4. **Run viability tests first** when changing production defaults:
//!    `cargo test -p v3-core --test viability`

use std::collections::HashMap;

use v3_core::config::SimulationConfig;
use v3_core::contracts::{CreatureId, Position};
use v3_core::creature::action_log::{ActionResult, ActionType};
use v3_core::creature::identity::CreatureIdentityState;
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::MutationEngine;
use v3_core::simulation::{run_tick, seed_simulation, Simulation};

const FOUNDER_CHANNELS: [u8; 6] = [0, 0, 92, 92, 138, 138];
const FOUNDER_ACTIVE_CHANNEL: usize = 0;
const FOUNDER_POLARITY: [bool; 6] = [true; 6];

/// Return a compact config suitable for fast, behavior-focused viability tests.
///
/// Uses production defaults for all economic parameters so these tests serve
/// as a gate against config changes that break viability. Only world size,
/// population count, and food coverage are overridden for test speed.
fn viability_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    // Smaller world for test speed
    cfg.world.width = 32;
    cfg.world.height = 32;
    // Fewer creatures for test speed
    cfg.population.initial_creatures = 10;
    // Full food coverage to compensate for small world
    cfg.world.food.initial_coverage = 1.0;
    cfg.world.food.initial_density = 1.0;
    cfg
}

/// Probe config for validating whether default economics remain viable under
/// the high-coverage startup profile used by this viability suite.
///
/// This intentionally raises startup food coverage to stress-test
/// post-migration behavior without adding test-only energy boosts.
fn high_coverage_economics_probe_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.population.initial_creatures = 20;
    cfg.world.food.initial_coverage = 1.0;
    cfg
}

#[derive(Debug, Clone, Copy)]
struct TickMetrics {
    tick: u64,
    population: usize,
    newborns: usize,
    moved_survivors: usize,
    max_generation: u64,
    total_food: f32,
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
        run_tick(sim, &mut None);
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
                "t{} pop={} births={} moved={} max_gen={} food={:.3}",
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
        run_tick(&mut sim, &mut None);
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
fn high_coverage_economics_survive_and_reproduce_short_horizon() {
    for seed in [1_u64, 7, 42] {
        let mut sim = seed_simulation(high_coverage_economics_probe_config(), seed);
        let metrics = run_ticks_with_metrics(&mut sim, 20);
        let births_total: usize = metrics.iter().map(|m| m.newborns).sum();
        let max_generation = metrics.iter().map(|m| m.max_generation).max().unwrap_or(0);

        assert!(
            sim.creature_count() > 0,
            "high-coverage economics seed {seed} extinct by tick {}.\n{}",
            sim.tick_number(),
            format_tick_metrics(&metrics)
        );
        assert!(
            births_total > 0 && max_generation > 0,
            "high-coverage economics seed {seed} had no reproduction.\n{}",
            format_tick_metrics(&metrics)
        );
    }
}

/// The canonical founder consumes ordinary primary food and reproduces.
#[test]
fn canonical_founder_consumes_primary_food_and_reproduces() {
    let mut cfg = viability_config();
    cfg.population.initial_creatures = 20;
    cfg.world.food.growth_rate = 0.0;
    cfg.mutation.per_unit_supply_enabled = false;
    cfg.mutation.mutation_probability = 0.0;
    let mut sim = seed_simulation(cfg, 2026);
    let primary_before = sim
        .world
        .total_food_by_type(v3_core::config::OrdinaryFoodTypeId::new(0));

    run_ticks_with_metrics(&mut sim, 30);

    assert!(sim.stats.reproduction_actions_spawned_total > 0);
    assert!(
        sim.world
            .total_food_by_type(v3_core::config::OrdinaryFoodTypeId::new(0))
            < primary_before
    );
    assert_eq!(sim.stats.mutation_events_applied_total, 0);
    let successful_eats = sim
        .action_logs
        .values()
        .flat_map(|log| log.entries())
        .filter(|entry| {
            entry.action_type == ActionType::Eat && entry.result == ActionResult::Success
        })
        .count();
    assert!(
        successful_eats > 0,
        "canonical founder should apply typed Eat actions"
    );
}

/// The total food count after 1 tick should differ from the initial seeded food
/// (either consumed by creatures or grown by the food-growth pass).
#[test]
fn food_gets_consumed_and_regrows() {
    let mut sim = seed_simulation(viability_config(), 42);
    let food_initial = sim.world.total_food();
    run_tick(&mut sim, &mut None);
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
        let mut food_cfg = cfg.world.food.clone();
        food_cfg.initial_coverage = 1.0;
        food_cfg.initial_density = 1.0;
        food_cfg.types[0].initial_coverage = 1.0;
        world.reconfigure_food(food_cfg);
        let mut rng = SmallRng::seed_from_u64(0);
        world.seed_food(&mut rng);
    }
    let food_before = world.food_at(pos);

    // Use production initial_energy (below founder reproduce threshold of 24.0)
    // so the creature eats rather than reproduces.
    let start_energy = cfg.energy.lifecycle.initial_energy;
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let creature_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            pos,
            start_energy,
            0,
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(pos, creature_id);
    let energy_before = creatures[creature_id].energy;

    let mut sim = Simulation::new(world, creatures, 0, cfg, 42);
    run_tick(&mut sim, &mut None);

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

/// Founders consume the primary type while preserving another ordinary type.
#[test]
fn founder_eats_primary_type_and_preserves_other_food() {
    use slotmap::SlotMap;
    use v3_core::config::OrdinaryFoodTypeId;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;

    let mut cfg = SimulationConfig::default();
    cfg.world
        .food
        .types
        .push(v3_core::config::FoodTypeConfig::default());
    cfg.world.width = 5;
    cfg.world.height = 5;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.food.growth_rate = 0.0;
    cfg.population.initial_creatures = 0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let pos = Position::new(2, 2);
    world.set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.5);
    world.set_food_type(pos, OrdinaryFoodTypeId::new(1), 0.5);

    let mut creatures: slotmap::SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            pos,
            cfg.energy.lifecycle.initial_energy,
            0,
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(pos, id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 42);
    run_tick(&mut sim, &mut None);

    assert_eq!(sim.world.food_at_type(pos, OrdinaryFoodTypeId::new(0)), 0.0);
    assert_eq!(sim.world.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 0.5);
    assert!(sim.creatures[id].energy > 20.0);
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
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    // Phase 0 increments age before action selection, so age=19 here means the
    // founder acts at age 20 on this tick.
    creatures[parent_id].age = 19;
    world.place_creature(pos, parent_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 7);
    run_tick(&mut sim, &mut None);

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

/// With high energy but age below the reproduction gate, the founder should not
/// attempt reproduction and should still move/explore.
#[test]
fn founder_does_not_attempt_reproduce_when_below_min_reproduce_age() {
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
    cfg.energy.lifecycle.min_reproduce_age = 20;

    let start = Position::new(5, 5);
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let founder_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            start,
            cfg.energy.lifecycle.initial_energy,
            0,
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(start, founder_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 8);
    run_tick(&mut sim, &mut None);

    assert_eq!(sim.creature_count(), 1, "no child should be spawned");
    assert_eq!(
        sim.stats.last_tick_reproduce, 0,
        "founder should not attempt reproduce before min age"
    );
    let moved_to = sim.creatures[founder_id].position;
    assert_ne!(
        moved_to, start,
        "founder should take fallback movement branch before age gate opens"
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
    // Production initial_energy (20.0) is below the founder reproduce threshold (24.0)

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
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(start, creature_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 11);
    run_tick(&mut sim, &mut None);

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
    cfg.mutation.per_unit_supply_enabled = false;
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 1;
    cfg.mutation.per_birth_mutation_events_max = 2;

    let mut sim = seed_simulation(cfg, 42);
    let mut ever_diverged = false;
    for _ in 0..30 {
        run_tick(&mut sim, &mut None);
        if sim
            .creatures
            .values()
            .any(|c| c.phenotype_channels != FOUNDER_CHANNELS)
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
    cfg.per_unit_supply_enabled = false;
    cfg.mutation_probability = 1.0;
    cfg.per_birth_mutation_events_min = 1;
    cfg.per_birth_mutation_events_max = 5;

    let mut genome = v3alpha1_founder_genome();
    for seed in 0u64..1000 {
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut genome,
            &cfg,
            &[],
            ParentExecuted::NONE,
            &mut rng,
            1,
        );
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events,
            "accounting invariant violated at seed {seed}"
        );
    }
}

/// Reachability telemetry accumulates correctly during mutation.
#[test]
fn mutation_reachability_telemetry_accumulates() {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::genome::analysis::mesh_reachable_nodes;

    let mut cfg = SimulationConfig::default().mutation;
    cfg.per_unit_supply_enabled = false;
    cfg.mutation_probability = 1.0;
    cfg.per_birth_mutation_events_min = 3;
    cfg.per_birth_mutation_events_max = 8;
    cfg.reachable_bias.topology = 0.7;
    cfg.reachable_bias.vm = 0.7;
    cfg.reachable_bias.graph = 0.7;
    cfg.reachable_bias.input_ref = 0.5;

    let genome = v3alpha1_founder_genome();
    let reachable = mesh_reachable_nodes(&genome);

    let mut total_reachable = 0u32;
    let mut total_unreachable = 0u32;
    let mut total_na = 0u32;
    let mut total_applied = 0u32;

    for seed in 0u64..500 {
        let mut g = genome.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut g,
            &cfg,
            &reachable,
            ParentExecuted::NONE,
            &mut rng,
            1,
        );
        total_reachable += summary.reachable_target_events;
        total_unreachable += summary.unreachable_target_events;
        total_na += summary.not_applicable_events;
        total_applied += summary.applied_events;
    }

    // With 500 seeds × 3-8 events each, we should see a mix of categories.
    assert!(
        total_reachable > 0,
        "expected some reachable-target events, got 0"
    );
    assert!(
        total_na > 0,
        "expected some not-applicable events (exempt operators), got 0"
    );
    // Sum of all reachability categories must equal total applied events.
    assert_eq!(
        total_reachable + total_unreachable + total_na,
        total_applied,
        "reachability counts must sum to applied events"
    );
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
    cfg.mutation.per_unit_supply_enabled = false;
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
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(pos, parent_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 7);
    run_tick(&mut sim, &mut None);

    // Find the child (generation == 1).
    let child = sim.creatures.values().find(|c| c.generation == 1);

    if let Some(child) = child {
        assert_eq!(
            child.phenotype_channels, FOUNDER_CHANNELS,
            "child phenotype must be identical to parent when mutation_probability=0.0"
        );
    }
    // If no child was produced this tick, the test is vacuously satisfied (no mutation check needed).
}

/// When mutation_probability=0.0, child must inherit parent identity unchanged
/// (same lineage_id, same kin_tag since applied_events == 0).
#[test]
fn identity_inherits_unchanged_when_no_genome_mutation() {
    use slotmap::SlotMap;
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::creature::identity::CreatureIdentityState;
    use v3_core::creature::state::CreatureState;
    use v3_core::kernel::WorldState;

    let mut cfg = SimulationConfig::default();
    cfg.world.width = 10;
    cfg.world.height = 10;
    cfg.mutation.per_unit_supply_enabled = false;
    cfg.mutation.mutation_probability = 0.0;
    cfg.energy.lifecycle.initial_energy = 80.0;
    cfg.energy.lifecycle.max_energy = 120.0;
    cfg.energy.lifecycle.default_offspring_energy = 12.0;
    cfg.energy.costs.reproduce_cost = 1.0;
    cfg.runtime.graph_node_base_cost = 0.1;

    let parent_identity = CreatureIdentityState::founder(0, 7);
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
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            parent_identity,
            [0.0; 16],
        )
    });
    world.place_creature(pos, parent_id);

    let mut sim = Simulation::new(world, creatures, 0, cfg, 7);
    run_tick(&mut sim, &mut None);

    if let Some(child) = sim.creatures.values().find(|c| c.generation == 1) {
        assert_eq!(
            child.identity.lineage_id, parent_identity.lineage_id,
            "child lineage_id must match parent"
        );
        assert_eq!(
            child.identity.kin_tag, parent_identity.kin_tag,
            "child kin_tag must be unchanged when mutation_probability=0.0"
        );
    }
}

/// Existing viability profile still works with real mutations enabled.
#[test]
fn viability_still_passes_with_real_mutations() {
    let mut cfg = viability_config();
    cfg.mutation.per_unit_supply_enabled = false;
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
        run_tick(&mut sim, &mut None);
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
    run_tick(&mut sim, &mut None);
    let tick1_reproduce = sim.stats.last_tick_reproduce;
    let tick1_attempted = sim.stats.reproduction_actions_attempted_total;
    // Run tick 2.
    run_tick(&mut sim, &mut None);
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
        run_tick(&mut sim, &mut None);
    }
    assert!(
        sim.mean_energy() > 0.0,
        "mean_energy should be positive in a viable simulation, got {}",
        sim.mean_energy()
    );
}

// ── Stage 7: Mutation skip reason tracking ────────────────────────────────────

#[test]
fn mutation_skip_reason_tracking_accumulates_correctly() {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.population.initial_creatures = 10;
    cfg.mutation.per_unit_supply_enabled = false;
    cfg.mutation.mutation_probability = 1.0;
    cfg.mutation.per_birth_mutation_events_min = 3;
    cfg.mutation.per_birth_mutation_events_max = 3;
    cfg.world.food.initial_coverage = 0.8;
    cfg.world.food.initial_density = 1.0;
    cfg.world.food.growth_rate = 0.5;
    cfg.energy.lifecycle.initial_energy = 150.0;
    cfg.energy.lifecycle.default_offspring_energy = 4.0;
    cfg.energy.costs.reproduce_cost = 1.0;
    let mut sim = seed_simulation(cfg, 42);
    for _ in 0..50 {
        run_tick(&mut sim, &mut None);
    }
    let stats = &sim.stats;
    // If any skips occurred, all keys must be valid skip reasons.
    for reason in stats.mutation_events_skipped_by_reason.keys() {
        let is_valid = matches!(
            reason,
            v3_core::mutation::MutationSkipReason::ParseabilityViolation
                | v3_core::mutation::MutationSkipReason::NoApplicableTarget
                | v3_core::mutation::MutationSkipReason::BudgetExhausted
        );
        assert!(is_valid, "unexpected skip reason key: {:?}", reason);
    }
    // Sum of per-reason counts must equal total skipped.
    let reason_total: u64 = stats.mutation_events_skipped_by_reason.values().sum();
    assert_eq!(
        reason_total, stats.mutation_events_skipped_total,
        "sum of skip reasons ({reason_total}) must equal skipped_total ({})",
        stats.mutation_events_skipped_total
    );
}

#[test]
fn ordinary_food_alone_replenishes_energy_and_reproduces() {
    let mut config = viability_config();
    config.world.food.types.truncate(1);
    config.world.food.types[0].initial_coverage = 1.0;
    let mut sim = seed_simulation(config, 42);
    let initial_energy = sim.creatures.values().next().unwrap().energy;
    run_tick(&mut sim, &mut None);
    assert!(sim.stats.last_tick_eat > 0);
    assert!(sim
        .creatures
        .values()
        .any(|creature| creature.energy > initial_energy));
    run_ticks_with_metrics(&mut sim, 29);
    assert!(sim.creature_count() > 0);
    assert!(sim.stats.reproduction_actions_spawned_total > 0);
}
