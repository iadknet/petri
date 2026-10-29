//! The transaction's contract (T30.F06): the settle order, the span-0
//! conception handed to the caller, and no removal inside `resolve`.

use slotmap::SlotMap;

use super::{resolve, Conception};
use crate::config::SimulationConfig;
use crate::contracts::ActionOutcome;
use crate::contracts::{CreatureId, Direction, Position, WorldAction};
use crate::creature::action_log::ActionType;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::simulation::simulation::Simulation;

/// A simulation holding one founder creature per `(position, energy)`, with
/// births delivered at conception (gestation rate 0.0).
fn sim_with(creatures_at: &[(Position, f32)]) -> (Simulation, Vec<CreatureId>) {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 10;
    cfg.world.height = 10;
    cfg.world.food.growth_rate = 0.0;
    cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let mut ids = Vec::new();
    for &(position, energy) in creatures_at {
        let id = creatures.insert_with_key(|id| {
            crate::creature::birth::Zygote::founder(
                v3alpha1_founder_genome().with_body(crate::creature::founder::founder_body(
                    3,
                    77,
                    &crate::config::SimulationConfig::default(),
                )),
                0,
            )
            .born_at(id, position, energy)
        });
        world.place_creature(position, id);
        ids.push(id);
    }
    (Simulation::new(world, creatures, 0, cfg, 42), ids)
}

fn at() -> Position {
    Position::new(5, 5)
}

fn north() -> Position {
    Position::new(5, 4)
}

/// A blocked move the actor cannot afford: its failed effort is booked before
/// the floor nets the overdraft, so the netting lowers the failed effort with
/// the charge and the subset `failed <= charge` holds at equality; nothing of
/// the overdraft is a floor credit.
#[test]
fn settle_nets_the_overdraft_after_the_failed_effort_is_booked() {
    let (mut sim, ids) = sim_with(&[(at(), 0.001), (north(), 50.0)]);
    let actor = ids[0];

    let applied = resolve(
        &mut sim.action_inputs(),
        actor,
        &WorldAction::Move(Direction::N),
    );

    assert_eq!(applied.result.outcome, ActionOutcome::Blocked);
    assert!(applied.energy_after < 0.0, "{}", applied.energy_after);
    assert!(applied.actor_removed);
    assert_eq!(sim.creatures[actor].energy, 0.0);
    let flows = &sim.stats.energy_flows;
    let charged = flows.action_charges[ActionType::Move];
    assert!((charged - f64::from(0.001_f32)).abs() < 1e-12, "{charged}");
    assert_eq!(flows.failed_action_effort[ActionType::Move], charged);
    assert_eq!(flows.zero_floor_credit, 0.0);
}

/// At gestation span 0 the conception is returned, not spawned: the
/// population is unchanged and the parent carries no pending birth.
#[test]
fn a_span_zero_conception_is_handed_to_the_caller() {
    let (mut sim, ids) = sim_with(&[(at(), 200.0)]);
    let parent = ids[0];
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;

    let applied = resolve(
        &mut sim.action_inputs(),
        parent,
        &WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer_fraction: 0.5,
        },
    );

    assert_eq!(applied.result.outcome, ActionOutcome::Success);
    let Some(Conception { target, litter }) = applied.conceived else {
        panic!("a span-0 conception is returned");
    };
    assert_eq!(target, north());
    assert!(litter > 0.0);
    assert_eq!(sim.creatures.len(), 1);
    assert!(sim.world.creature_at(north()).is_none());
    assert!(sim.creatures[parent].pending_birth.is_none());
    assert!(!applied.actor_removed);
}

/// A span above 0 carries the litter and returns no conception.
#[test]
fn a_carried_litter_returns_no_conception() {
    let (mut sim, ids) = sim_with(&[(at(), 200.0)]);
    let parent = ids[0];
    sim.config
        .energy
        .lifecycle
        .gestation_ticks_per_founder_genome = 1.0;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;

    let applied = resolve(
        &mut sim.action_inputs(),
        parent,
        &WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer_fraction: 0.5,
        },
    );

    assert_eq!(applied.result.outcome, ActionOutcome::Success);
    assert_eq!(applied.conceived, None);
    assert!(sim.creatures[parent].pending_birth.is_some());
}

/// An exhausted victim yields nothing to take, and stays.
#[test]
fn an_exhausted_victim_is_kept() {
    let (mut sim, ids) = sim_with(&[(at(), 100.0), (north(), 0.0)]);
    let (attacker, victim) = (ids[0], ids[1]);

    let applied = resolve(
        &mut sim.action_inputs(),
        attacker,
        &WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 10.0,
        },
    );

    assert_eq!(applied.result.outcome, ActionOutcome::NothingToTake);
    assert_eq!(applied.counterpart, Some(victim));
    assert!(sim.creatures.contains_key(victim));
    assert_eq!(sim.world.creature_at(north()), Some(victim));
}

/// A killing steal reports the victim as its counterpart and leaves it in
/// place; removal is the caller's.
#[test]
fn resolve_removes_no_killed_victim() {
    let (mut sim, ids) = sim_with(&[(at(), 100.0), (north(), 1.0)]);
    let (attacker, victim) = (ids[0], ids[1]);
    sim.creatures[attacker] =
        crate::creature::birth::Zygote::reborn(&sim.creatures[attacker], |body| {
            body.with_steal_share(1.0)
        });

    let applied = resolve(
        &mut sim.action_inputs(),
        attacker,
        &WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 10.0,
        },
    );

    assert_eq!(applied.result.outcome, ActionOutcome::TransferredAndKilled);
    assert_eq!(applied.counterpart, Some(victim));
    assert!(sim.creatures.contains_key(victim));
    assert_eq!(sim.world.creature_at(north()), Some(victim));
    assert!(sim.creatures[victim].energy <= 0.0);
    assert!(!applied.actor_removed);
}

/// The failed effort is booked once, by the settle step, as the attempt's
/// effort: a rejected steal books it, an applied one does not; `resolve`
/// writes no run total.
#[test]
fn the_failed_effort_is_the_attempt_effort_of_a_failed_outcome_only() {
    let steal = WorldAction::StealEnergy {
        direction: Direction::N,
        effort: 1.0,
    };
    let (mut sim, ids) = sim_with(&[(at(), 100.0)]);
    let rejected = resolve(&mut sim.action_inputs(), ids[0], &steal);
    assert_eq!(rejected.result.outcome, ActionOutcome::NoVictim);
    assert!(rejected.result.attempt_effort > 0.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.failed_action_effort[ActionType::StealEnergy],
        rejected.result.attempt_effort
    );
    assert_eq!(
        flows.action_charges[ActionType::StealEnergy],
        rejected.result.attempt_effort
    );
    assert_eq!(sim.stats.predation_actions_attempted_total, 0);
    assert_eq!(sim.stats.predation_actions_rejected_total, 0);

    let (mut sim, ids) = sim_with(&[(at(), 100.0), (north(), 50.0)]);
    let applied = resolve(&mut sim.action_inputs(), ids[0], &steal);
    assert_eq!(applied.result.outcome, ActionOutcome::Success);
    assert!(applied.result.amount > 0.0);
    assert_eq!(applied.counterpart, Some(ids[1]));
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.failed_action_effort[ActionType::StealEnergy], 0.0);
    assert_eq!(
        flows.action_charges[ActionType::StealEnergy],
        applied.result.attempt_effort
    );
    assert_eq!(sim.stats.predation_actions_transferred_total, 0);
    assert!(sim.stats.last_tick_predation_events.is_empty());
}
