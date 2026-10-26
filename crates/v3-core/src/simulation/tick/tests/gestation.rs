//! Pre-birth gestation (T03.F12): conception, the hold, delivery and loss.

use super::support::*;
use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Direction, Position, WorldAction};
use crate::creature::action_log::{ActionLog, ActionResult};
use crate::genome::OutcomeChannel;
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::tick::{run_phase_0, run_phase_2, run_tick};
use crate::simulation::{seed_simulation, Simulation};
use rand::{rngs::SmallRng, SeedableRng};

const PARENT_ENERGY: f32 = 100.0;
const REPRODUCE: WorldAction = WorldAction::Reproduce {
    direction: Direction::N,
    energy_transfer_fraction: 0.5,
};

/// One founder (80 units) at (5, 5) on free, ungated economics, gestating
/// at `rate` ticks per founder genome.
fn gestating_sim(rate: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = make_sim_with_one_creature(PARENT_ENERGY);
    let config = &mut sim.config;
    config.energy.lifecycle.gestation_ticks_per_founder_genome = rate;
    config.energy.complexity_cost.enabled = false;
    config.energy.costs.reproduce_cost = 1.0;
    // A zero effort floor isolates the replication charge (T16.F02).
    config.energy.costs.noop_cost = 0.0;
    config.energy.costs.reproduce_attempt_cost = 0.0;
    config.energy.lifecycle.min_reproduce_age = 0;
    config.energy.lifecycle.min_reproduce_energy = 0.0;
    config.mutation.per_unit_rate = 0.0;
    sim.action_logs
        .insert(id, ActionLog::new(sim.config.action_log.capacity));
    (sim, id)
}

/// Run one Phase 2 slot for `id` at `tick` and return its outcome record.
fn slot(sim: &mut Simulation, id: CreatureId, tick: u64, actions: Vec<WorldAction>) -> u32 {
    sim.tick = tick;
    let mut outcomes = OutcomeAccumulator::default();
    outcomes.snapshot_energy(id, sim.creatures[id].energy);
    run_phase_2(
        sim,
        vec![(id, decision(actions))],
        &mut outcomes,
        &mut SmallRng::seed_from_u64(tick),
    );
    let bank = outcomes.compute_signal_bank(id, 0.0).unwrap();
    bank.signals[OutcomeChannel::OffspringSuccess as usize] as u32
}

fn log_len(sim: &Simulation, id: CreatureId) -> usize {
    sim.action_logs[id].len()
}

/// After `Reproduce` at tick 0 with fraction 0.5: the charge (1.0) and the
/// litter (49.5) are paid; the litter is what a delivered child starts with.
const LITTER: f32 = (PARENT_ENERGY - 1.0) * 0.5;

fn conceive(sim: &mut Simulation, id: CreatureId) {
    let offspring = slot(sim, id, 0, vec![REPRODUCE, WorldAction::NoOp]);
    assert_eq!(offspring, 0, "no child exists at conception");
}

fn children(sim: &Simulation, parent: CreatureId) -> Vec<Position> {
    sim.creatures
        .iter()
        .filter(|(id, _)| *id != parent)
        .map(|(_, c)| c.position)
        .collect()
}

#[test]
fn gestation_conception_ends_the_queue_and_holds_the_founder_one_tick() {
    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);

    let pending = sim.creatures[id].pending_birth.expect("conceived");
    assert_eq!(pending.deliver_tick, 1);
    assert_eq!(pending.litter_energy, LITTER);
    // T26.F02 invariant 7, in gestation: the debit is the pending stock.
    assert_eq!(
        sim.stats.energy_flows.parental_transfer_debit,
        f64::from(LITTER)
    );
    assert_eq!(pending.direction, Direction::N);
    assert_eq!(sim.creatures[id].energy, PARENT_ENERGY - 1.0 - LITTER);
    assert_eq!(sim.creature_count(), 1);
    assert_eq!(sim.stats.gestations_started_total, 1);
    assert_eq!(sim.stats.gestations_delivered_total, 0);
    assert_eq!(sim.stats.reproduction_actions_spawned_total, 0);
    // The queued NoOp after the conception is discarded with no log entry.
    assert_eq!(sim.stats.last_tick_noop, 0);
    assert_eq!(log_len(&sim, id), 1);
    let entry = sim.action_logs[id].entries()[0];
    assert_eq!(entry.result, ActionResult::Success);

    // Tick 1 (T + h): delivered at the parent's slot, which commits nothing.
    let offspring = slot(&mut sim, id, 1, vec![WorldAction::NoOp]);
    assert_eq!(
        offspring, 1,
        "the delivery counts as the parent's offspring"
    );
    assert_eq!(children(&sim, id), vec![Position::new(5, 4)]);
    let child = sim.creatures.values().find(|c| c.generation == 1).unwrap();
    assert_eq!(child.energy, LITTER);
    assert!(sim.creatures[id].pending_birth.is_none());
    assert_eq!(sim.creatures[id].offspring_spawned_count, 1);
    assert_eq!(sim.stats.gestations_delivered_total, 1);
    assert_eq!(sim.stats.reproduction_actions_spawned_total, 1);
    assert_eq!(sim.stats.gestation_creature_ticks_total, 1);
    assert_eq!(
        f64::from(LITTER),
        sim.stats.energy_flows.offspring_energy_credit
    );
    // Delivered within the child's capacity: no overflow, the debit is the
    // credit exactly.
    assert_eq!(sim.stats.energy_flows.offspring_capacity_overflow, 0.0);
    assert_eq!(
        sim.stats.energy_flows.parental_transfer_debit,
        sim.stats.energy_flows.offspring_energy_credit
    );
    assert_eq!(sim.stats.last_tick_noop, 0);
    assert_eq!(log_len(&sim, id), 1, "a held tick writes no log entry");

    // Tick 2 (T + h + 1): the parent acts again.
    slot(&mut sim, id, 2, vec![WorldAction::NoOp]);
    assert_eq!(sim.stats.last_tick_noop, 1);
    assert_eq!(log_len(&sim, id), 2);
    assert_eq!(sim.stats.gestation_creature_ticks_total, 1);
}

#[test]
fn gestation_span_scales_with_rate_and_holds_every_tick_of_it() {
    // Rate 2.0 on the founder: ceil(80 * 2 / 80) = 2 held ticks.
    let (mut sim, id) = gestating_sim(2.0);
    conceive(&mut sim, id);
    assert_eq!(sim.creatures[id].pending_birth.unwrap().deliver_tick, 2);

    assert_eq!(slot(&mut sim, id, 1, vec![WorldAction::NoOp]), 0);
    assert!(children(&sim, id).is_empty());
    assert_eq!(log_len(&sim, id), 1);

    assert_eq!(slot(&mut sim, id, 2, vec![WorldAction::NoOp]), 1);
    assert_eq!(children(&sim, id).len(), 1);
    assert_eq!(sim.stats.gestation_creature_ticks_total, 2);
    assert_eq!(log_len(&sim, id), 1);
}

#[test]
fn gestation_delivers_on_the_first_valid_cell_when_the_chosen_one_closed() {
    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);
    sim.world.set_barrier(Position::new(5, 4), true);

    slot(&mut sim, id, 1, vec![]);
    // N is barred; NE is next in `Direction::ALL` order.
    assert_eq!(children(&sim, id), vec![Position::new(6, 4)]);
    assert_eq!(sim.stats.gestations_delivered_total, 1);
}

#[test]
fn gestation_with_no_valid_cell_loses_the_litter_and_still_holds_the_parent() {
    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);
    for dir in Direction::ALL {
        let cell = sim
            .world
            .resolve_neighbor(Position::new(5, 5), dir)
            .unwrap();
        sim.world.set_barrier(cell, true);
    }

    assert_eq!(slot(&mut sim, id, 1, vec![WorldAction::NoOp]), 0);
    assert!(children(&sim, id).is_empty());
    assert!(sim.creatures[id].pending_birth.is_none());
    assert_eq!(sim.stats.gestations_lost_no_cell_total, 1);
    assert_eq!(sim.stats.energy_flows.gestation_lost, f64::from(LITTER));
    // T26.F02 invariant 7, lost: the debit is the lost litter exactly.
    assert_eq!(
        sim.stats.energy_flows.parental_transfer_debit,
        sim.stats.energy_flows.gestation_lost
    );
    assert_eq!(sim.stats.gestations_delivered_total, 0);
    assert_eq!(sim.stats.gestation_creature_ticks_total, 1);
    assert_eq!(
        sim.stats.mortality.deaths_total, 0,
        "a lost birth is not a death"
    );
    assert_eq!(log_len(&sim, id), 1);
}

#[test]
fn gestation_at_the_population_cap_loses_the_litter() {
    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);
    sim.config.population.max_creatures = 1;

    slot(&mut sim, id, 1, vec![]);
    assert!(children(&sim, id).is_empty());
    assert!(sim.creatures[id].pending_birth.is_none());
    assert_eq!(sim.stats.gestations_lost_population_cap_total, 1);
    assert_eq!(sim.stats.energy_flows.gestation_lost, f64::from(LITTER));
}

#[test]
fn gestation_litter_dies_with_a_parent_removed_in_phase_0() {
    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);
    sim.creatures[id].energy = 0.1;
    run_phase_0(&mut sim);

    assert!(!sim.creatures.contains_key(id));
    assert_eq!(sim.creature_count(), 0, "no child");
    assert_eq!(sim.stats.gestations_lost_parent_death_total, 1);
    assert_eq!(sim.stats.energy_flows.gestation_lost, f64::from(LITTER));
    assert_eq!(sim.stats.mortality.deaths_total, 1, "only the parent dies");
}

#[test]
fn gestation_parent_exhausted_before_its_slot_is_removed_there_without_delivery() {
    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);
    // Phase 1 charges can leave a creature at or below zero unremoved.
    sim.creatures[id].energy = -0.25;

    assert_eq!(slot(&mut sim, id, 1, vec![WorldAction::NoOp]), 0);
    assert!(!sim.creatures.contains_key(id));
    assert_eq!(sim.creature_count(), 0);
    assert_eq!(sim.stats.gestations_lost_parent_death_total, 1);
    assert_eq!(sim.stats.gestations_delivered_total, 0);
    assert_eq!(sim.stats.gestation_creature_ticks_total, 0);
}

#[test]
fn gestation_rate_zero_delivers_at_conception_and_the_queue_continues() {
    let (mut sim, id) = gestating_sim(0.0);
    let offspring = slot(&mut sim, id, 0, vec![REPRODUCE, WorldAction::NoOp]);

    assert_eq!(offspring, 1);
    assert_eq!(children(&sim, id), vec![Position::new(5, 4)]);
    assert!(sim.creatures[id].pending_birth.is_none());
    assert_eq!(sim.stats.last_tick_noop, 1);
    assert_eq!(log_len(&sim, id), 2);
    assert_eq!(sim.stats.gestations_started_total, 1);
    assert_eq!(sim.stats.gestations_delivered_total, 1);
    assert_eq!(sim.stats.gestation_creature_ticks_total, 0);
}

/// Count identities and the energy identity of the spec's "Energy identity"
/// row over a run with births, losses on death and crowded deliveries.
/// `lower_cap_at` lowers every living body's capacity and `max_energy` to
/// [`LOWERED_CAP`] mid-run without re-clamping creatures, so later litters
/// overflow the children's capacity (T26.F02 invariant 7).
fn assert_gestation_identities(lower_cap_at: Option<u64>) {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 32;
    cfg.world.height = 32;
    cfg.world.food.types[0].initial_coverage = 1.0;
    cfg.world.food.types[0].initial_density = 1.0;
    cfg.population.initial_creatures = 40;
    cfg.population.max_creatures = 50;
    // A long span and a steep decay expose litters to parent death; the cap
    // to crowded deliveries.
    cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 8.0;
    cfg.energy.lifecycle.energy_decay_per_tick = 2.0;
    // Effort-cost defaults, 2026-10-10: the fixture keeps the earlier effort
    // costs; at floor 0.4, attempt 1.0, move 1.6 the population dies before
    // any delivery, and the identities are independent of action prices.
    cfg.energy.costs.noop_cost = 0.05;
    cfg.energy.costs.reproduce_attempt_cost = 0.05;
    cfg.energy.costs.move_cost = 0.2;
    // Re-seeded by T11.F31 (165 -> 328): the five ring projection operators
    // change every Graph-domain draw, and at seed 165 no creature holds
    // energy above the lowered cap at tick 50, the precondition of the
    // lowered-cap case. Re-seeded by T11.F30 (328 -> 433): the founder
    // heading draws shift every later draw of the run, with the same miss.
    // Re-seeded 2026-10-12 (433 -> 740) when T20.F04 whole-family recruitment
    // and T20.F05 refinement became defaults and changed every Graph draw;
    // 740 is the first seed from 433 whose tick-50 state meets the
    // precondition. Re-seeded by T25.F02 (740 -> 2733): the uniform size
    // rule lowers the carrying cost and the mutation draw count, and 2733 is
    // the first seed from 740 whose tick-50 state meets the precondition;
    // the old size rule and anchor 97 restore 740. Re-seeded by T25.F04
    // commit 1 (2733 -> 3249): `AddDirectionPairGroup` joins the Graph
    // default arm and remaps every Graph draw; 3249 is the first seed from
    // 2733 that passes both the unlowered and the lowered-cap case; removing
    // the catalog entry restores 2733. Re-seeded by T26.F02 (3249 -> 3282,
    // lowered cap 60 -> 50): the capacity chance draw on every birth shifts
    // every later draw, no seed in 3249..3600 met the precondition at 60, and
    // 3282 is the first seed from 3249 that passes both cases at 50.
    // Re-seeded by T27.F03 commit 3 (3282 -> 3485): the two share draws on
    // every birth shift every later draw, at 3282 no creature holds energy
    // above the lowered cap at tick 50, and 3485 is the first seed from 3282
    // that passes both cases.
    let mut sim = seed_simulation(cfg, 3485);
    let mut e_max = sim.creatures.values().map(|c| c.energy).fold(0.0, f32::max);
    let mut c_max = sim.config.energy.lifecycle.max_energy;
    for tick in 0..250 {
        if lower_cap_at == Some(tick) {
            sim.config.energy.lifecycle.max_energy = LOWERED_CAP;
            for creature in sim.creatures.values_mut() {
                creature.body.capacity = LOWERED_CAP;
            }
            assert!(
                sim.creatures.values().any(|c| c.energy > LOWERED_CAP),
                "creatures must hold energy above the lowered cap"
            );
            // B6: a held parent only loses energy, so one above the new cap
            // conceived above it before the lowering.
            assert!(
                sim.creatures
                    .values()
                    .any(|c| c.pending_birth.is_some() && c.energy > LOWERED_CAP),
                "a gestating parent must hold energy above the lowered cap"
            );
        }
        c_max = c_max.max(sim.config.energy.lifecycle.max_energy);
        run_tick(&mut sim, &mut None);
        e_max = sim
            .creatures
            .values()
            .map(|c| c.energy)
            .fold(e_max, f32::max);

        let s = &sim.stats;
        let pending: Vec<f32> = sim
            .creatures
            .values()
            .filter_map(|c| c.pending_birth.map(|p| p.litter_energy))
            .collect();
        assert_eq!(
            s.reproduction_actions_attempted_total,
            s.reproduction_actions_rejected_total + s.gestations_started_total
        );
        assert_eq!(
            s.gestations_started_total,
            s.gestations_delivered_total
                + s.gestations_lost_parent_death_total
                + s.gestations_lost_no_cell_total
                + s.gestations_lost_population_cap_total
                + pending.len() as u64
        );
        let flows = &s.energy_flows;
        let carried: f64 = pending.iter().copied().map(f64::from).sum();
        let residual = (flows.parental_transfer_debit
            - (flows.offspring_energy_credit
                + flows.offspring_capacity_overflow
                + flows.gestation_lost
                + carried))
            .abs();
        let bound = s.gestations_started_total as f64
            * f64::from(e_max.max(c_max))
            * f64::from(2.0f32.powi(-23));
        assert!(
            residual <= bound,
            "tick {tick}: residual {residual} exceeds {bound}"
        );
    }
    let s = &sim.stats;
    assert!(s.gestations_delivered_total > 0, "{s:?}");
    assert!(s.gestations_lost_parent_death_total > 0, "{s:?}");
    assert!(
        s.gestations_lost_no_cell_total + s.gestations_lost_population_cap_total > 0,
        "crowded deliveries must lose some litters"
    );
    assert_eq!(
        s.energy_flows.offspring_capacity_overflow > 0.0,
        lower_cap_at.is_some(),
        "only the lowered cap overflows a litter"
    );
}

/// The capacity `assert_gestation_identities` lowers every body and
/// `max_energy` to.
const LOWERED_CAP: f32 = 50.0;

#[test]
fn gestation_energy_and_count_identities_hold_every_tick() {
    assert_gestation_identities(None);
}

#[test]
fn gestation_energy_identity_holds_when_the_cap_is_lowered_mid_run() {
    assert_gestation_identities(Some(50));
}

#[test]
fn gestation_records_the_parent_age_at_delivery_by_birth_order() {
    use crate::simulation::birth_age::BirthOrder;
    let (mut sim, id) = gestating_sim(1.0);
    sim.creatures[id].age = 40;
    conceive(&mut sim, id);
    // The parent ages while it carries the litter; delivery age is recorded.
    sim.creatures[id].age = 60;
    slot(&mut sim, id, 1, vec![WorldAction::NoOp]);

    sim.creatures[id].energy = PARENT_ENERGY;
    sim.creatures[id].age = 400;
    let south = WorldAction::Reproduce {
        direction: Direction::S,
        energy_transfer_fraction: 0.5,
    };
    slot(&mut sim, id, 2, vec![south]);
    sim.creatures[id].age = 401;
    slot(&mut sim, id, 3, vec![WorldAction::NoOp]);
    assert_eq!(sim.stats.gestations_delivered_total, 2);

    let first = sim.stats.birth_parent_age.totals(BirthOrder::First);
    assert_eq!((first.count, first.sum), (1, 60));
    let later = sim.stats.birth_parent_age.totals(BirthOrder::Later);
    assert_eq!((later.count, later.sum), (1, 401));
}

/// The child's heading (T11.F30) is the step from the parent's cell to the
/// cell it is placed on: the validated `Reproduce` direction at span 0, the
/// stored direction at delivery, or the fallback cell's direction; the
/// parent's own heading is never copied.
#[test]
fn a_child_heads_away_from_its_parent_on_every_delivery_path() {
    let child_heading = |sim: &Simulation, parent: CreatureId| {
        sim.creatures
            .iter()
            .find(|(id, _)| *id != parent)
            .map(|(_, c)| c.heading)
            .expect("a child")
    };
    let reproduce = |direction| WorldAction::Reproduce {
        direction,
        energy_transfer_fraction: 0.5,
    };

    let (mut sim, id) = gestating_sim(0.0);
    sim.creatures[id].heading = Direction::W;
    slot(&mut sim, id, 0, vec![reproduce(Direction::SE)]);
    assert_eq!(children(&sim, id), vec![Position::new(6, 6)]);
    assert_eq!(child_heading(&sim, id), Direction::SE);
    assert_eq!(sim.creatures[id].heading, Direction::W);

    let (mut sim, id) = gestating_sim(1.0);
    slot(&mut sim, id, 0, vec![reproduce(Direction::E)]);
    slot(&mut sim, id, 1, vec![]);
    assert_eq!(children(&sim, id), vec![Position::new(6, 5)]);
    assert_eq!(child_heading(&sim, id), Direction::E);

    let (mut sim, id) = gestating_sim(1.0);
    conceive(&mut sim, id);
    sim.world.set_barrier(Position::new(5, 4), true);
    slot(&mut sim, id, 1, vec![]);
    assert_eq!(children(&sim, id), vec![Position::new(6, 4)]);
    assert_eq!(child_heading(&sim, id), Direction::NE);
}
