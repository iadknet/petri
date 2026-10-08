//! Post-birth reproductive hold (T03.F12): a spawning parent commits no action
//! on the `reproductive_hold_ticks` ticks after the birth tick.

use super::support::*;
use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Direction, WorldAction};
use crate::creature::action_log::ActionLog;
use crate::creature::founder::FOUNDER_GENOME_SIZE_UNITS;
use crate::creature::genome::OutcomeChannel;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{ComputeCostReport, MeshOutput};
use crate::simulation::energy_accounting::CognitionEnergyObservation;
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::tick::{run_phase_2, run_tick};
use crate::simulation::{seed_simulation, Simulation};
use rand::{rngs::SmallRng, SeedableRng};

const PARENT_ENERGY: f32 = 100.0;

const fn reproduce(direction: Direction) -> WorldAction {
    WorldAction::Reproduce {
        direction,
        energy_transfer_fraction: 0.25,
    }
}

fn decision(actions: Vec<WorldAction>, cognition: CognitionEnergyObservation) -> MeshOutput {
    MeshOutput {
        actions,
        cost_report: ComputeCostReport::default(),
        priority_bid: 0.0,
        work_counters: Default::default(),
        energy_observation: cognition,
        termination_reason: TerminationReason::NoDecision,
        commit_counts: Default::default(),
    }
}

/// One founder (97 units) at (5, 5) on free, ungated economics, held at
/// `rate` ticks per founder genome.
fn holding_sim(rate: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = make_sim_with_one_creature(PARENT_ENERGY);
    let config = &mut sim.config;
    config
        .energy
        .lifecycle
        .reproductive_hold_ticks_per_founder_genome = rate;
    config.energy.lifecycle.genome_replication_cost_per_unit = 0.0;
    config.energy.complexity_cost.enabled = false;
    config.energy.costs.reproduce_cost = 1.0;
    config.energy.costs.noop_cost = 0.0;
    config.energy.costs.failed_action_penalty = 0.0;
    config.startup.ramps.failed_action_penalty.enabled = false;
    config.energy.lifecycle.min_reproduce_age = 0;
    config.energy.lifecycle.min_reproduce_energy = 0.0;
    config.mutation.per_unit_rate = 0.0;
    sim.action_logs
        .insert(id, ActionLog::new(sim.config.action_log.capacity));
    (sim, id)
}

/// Run one Phase 2 slot for `id` at `tick` with `cognition` recorded and
/// return the offspring signal of the slot.
fn slot_with(
    sim: &mut Simulation,
    id: CreatureId,
    tick: u64,
    actions: Vec<WorldAction>,
    cognition: CognitionEnergyObservation,
) -> f32 {
    sim.tick = tick;
    let mut outcomes = OutcomeAccumulator::default();
    if let Some(creature) = sim.creatures.get(id) {
        outcomes.snapshot_energy(id, creature.energy);
    }
    run_phase_2(
        sim,
        vec![(id, decision(actions, cognition))],
        &mut outcomes,
        &mut SmallRng::seed_from_u64(tick),
    );
    outcomes.compute_signal_bank(id, 0.0).map_or(0.0, |bank| {
        bank.signals[OutcomeChannel::OffspringSuccess as usize]
    })
}

fn slot(sim: &mut Simulation, id: CreatureId, tick: u64, actions: Vec<WorldAction>) -> f32 {
    slot_with(
        sim,
        id,
        tick,
        actions,
        CognitionEnergyObservation::default(),
    )
}

fn log_len(sim: &Simulation, id: CreatureId) -> usize {
    sim.action_logs[id].len()
}

#[test]
fn reproductive_hold_founder_spawns_at_t_and_is_held_on_t_plus_one_only() {
    let (mut sim, id) = holding_sim(1.0);

    // Tick 0 (T): the child is born now and the rest of the queue runs.
    let offspring = slot(
        &mut sim,
        id,
        0,
        vec![reproduce(Direction::N), WorldAction::NoOp],
    );
    assert_eq!(offspring, 1.0, "the child is recorded at T");
    assert_eq!(sim.creature_count(), 2);
    assert_eq!(sim.stats.reproduction_actions_spawned_total, 1);
    assert_eq!(
        sim.stats.last_tick_noop, 1,
        "the queue continues after a birth"
    );
    assert_eq!(log_len(&sim, id), 2);
    assert_eq!(sim.creatures[id].held_until_tick, 2);
    assert_eq!(sim.stats.reproductive_holds_started_total, 1);
    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 0);

    // Tick 1 (T + 1): held, commits nothing and writes no log entry.
    sim.stats.last_tick_noop = 0;
    slot(&mut sim, id, 1, vec![WorldAction::NoOp]);
    assert_eq!(sim.stats.last_tick_noop, 0);
    assert_eq!(log_len(&sim, id), 2, "a held tick writes no log entry");
    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 1);

    // Tick 2 (T + 2): acts again.
    slot(&mut sim, id, 2, vec![WorldAction::NoOp]);
    assert_eq!(sim.stats.last_tick_noop, 1);
    assert_eq!(log_len(&sim, id), 3);
    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 1);
}

#[test]
fn reproductive_hold_two_founder_genome_is_held_two_ticks() {
    let (mut sim, id) = holding_sim(1.0);
    sim.creatures[id].cached_genome_size = 2 * FOUNDER_GENOME_SIZE_UNITS;

    slot(&mut sim, id, 0, vec![reproduce(Direction::N)]);
    assert_eq!(sim.creatures[id].held_until_tick, 3);

    for tick in [1, 2] {
        slot(&mut sim, id, tick, vec![WorldAction::NoOp]);
        assert_eq!(log_len(&sim, id), 1, "held on tick {tick}");
    }
    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 2);

    slot(&mut sim, id, 3, vec![WorldAction::NoOp]);
    assert_eq!(log_len(&sim, id), 2, "acts again on T + h + 1");
}

#[test]
fn reproductive_hold_later_spawn_in_the_same_queue_resets_the_deadline() {
    let (mut sim, id) = holding_sim(1.0);
    // Let the second, smaller litter pass the `initial_energy` floor.
    sim.config.energy.lifecycle.initial_energy = 1.0;

    slot(
        &mut sim,
        id,
        0,
        vec![reproduce(Direction::N), reproduce(Direction::S)],
    );
    assert_eq!(sim.stats.reproduction_actions_spawned_total, 2);
    assert_eq!(sim.stats.reproductive_holds_started_total, 2);
    assert_eq!(
        sim.creatures[id].held_until_tick, 2,
        "the second birth overwrites the deadline, it does not extend it"
    );
}

#[test]
fn reproductive_hold_charges_cognition_while_held() {
    let (mut sim, id) = holding_sim(1.0);
    slot(&mut sim, id, 0, vec![reproduce(Direction::N)]);
    let before = sim.stats.energy_flows.vm_compute;

    let cognition = CognitionEnergyObservation {
        vm_compute: 0.75,
        ..CognitionEnergyObservation::default()
    };
    slot_with(&mut sim, id, 1, vec![WorldAction::NoOp], cognition);

    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 1);
    assert_eq!(sim.stats.energy_flows.vm_compute, before + 0.75);
    assert_eq!(log_len(&sim, id), 1);
}

#[test]
fn reproductive_hold_removes_a_dead_held_parent_at_its_slot() {
    let (mut sim, id) = holding_sim(1.0);
    slot(&mut sim, id, 0, vec![reproduce(Direction::N)]);
    // Phase 1 charges can leave a creature at or below zero unremoved.
    sim.creatures[id].energy = -0.25;
    sim.stats.last_tick_noop = 0;

    slot(&mut sim, id, 1, vec![WorldAction::NoOp]);
    assert_eq!(sim.stats.last_tick_noop, 0, "removed before any action");
    assert!(!sim.creatures.contains_key(id));
    assert_eq!(sim.creature_count(), 1, "the child lives");
    assert_eq!(sim.stats.mortality.deaths_total, 1);
    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 0);
}

#[test]
fn reproductive_hold_rate_zero_sets_no_hold() {
    let (mut sim, id) = holding_sim(0.0);

    slot(&mut sim, id, 0, vec![reproduce(Direction::N)]);
    assert_eq!(sim.stats.reproduction_actions_spawned_total, 1);
    assert_eq!(sim.creatures[id].held_until_tick, 0);
    assert_eq!(sim.stats.reproductive_holds_started_total, 0);

    slot(&mut sim, id, 1, vec![WorldAction::NoOp]);
    assert_eq!(log_len(&sim, id), 2, "acts on T + 1");
    assert_eq!(sim.stats.reproductive_hold_creature_ticks_total, 0);
}

/// Every spawning birth starts a hold at any rate above zero; none at 0.0.
fn hold_counts_over_a_run(rate: f32) -> (u64, u64, u64) {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.world.food.types[0].initial_coverage = 1.0;
    cfg.population.initial_creatures = 40;
    cfg.energy
        .lifecycle
        .reproductive_hold_ticks_per_founder_genome = rate;
    let mut sim = seed_simulation(cfg, 7);
    for _ in 0..80 {
        run_tick(&mut sim, &mut None);
        assert_eq!(
            sim.stats.reproduction_actions_attempted_total,
            sim.stats.reproduction_actions_spawned_total
                + sim.stats.reproduction_actions_rejected_total
        );
    }
    (
        sim.stats.reproduction_actions_spawned_total,
        sim.stats.reproductive_holds_started_total,
        sim.stats.reproductive_hold_creature_ticks_total,
    )
}

#[test]
fn reproductive_hold_count_identity_holds_at_every_rate() {
    for rate in [0.5, 1.0, 3.0] {
        let (spawned, started, held) = hold_counts_over_a_run(rate);
        assert!(spawned > 0, "rate {rate}: the run must spawn");
        assert_eq!(started, spawned, "rate {rate}");
        assert!(held > 0, "rate {rate}: some parent must be held");
    }
    let (spawned, started, held) = hold_counts_over_a_run(0.0);
    assert!(spawned > 0);
    assert_eq!((started, held), (0, 0));
}
