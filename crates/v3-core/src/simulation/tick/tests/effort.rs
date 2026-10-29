//! T16.F02: every committed action pays its effort, at least `NoOp`'s, on
//! every attempt, and a failure pays that effort and nothing more.

use super::support::*;
use crate::contracts::{CreatureId, Direction, OrdinaryFoodTypeId, Position, WorldAction};
use crate::creature::action_log::ActionType;
use crate::ledger::{applied_debit, DeathCause, EnergyFlows};
use crate::simulation::Simulation;

const REPRODUCE: WorldAction = WorldAction::Reproduce {
    direction: Direction::N,
    energy_transfer_fraction: 0.5,
};

/// Which action bucket of [`EnergyFlows`] an attempt is charged to.
#[derive(Clone, Copy, Debug)]
enum Bucket {
    Move,
    Eat,
    Reproduce,
    Steal,
}

impl Bucket {
    /// The effort base of creature `id`'s attempt; a steal at effort
    /// modifier 1 has intensity `steal_share` (T27.F03).
    fn base(self, sim: &Simulation, id: CreatureId) -> f32 {
        let costs = &sim.config.energy.costs;
        match self {
            Self::Move => costs.move_cost,
            Self::Eat => costs.eat_cost,
            Self::Reproduce => costs.reproduce_attempt_cost,
            Self::Steal => {
                let creature = &sim.creatures[id];
                sim.config.exchange.effort_rate
                    * creature.genome().body().steal_share()
                    * creature.energy
            }
        }
    }

    fn add_failed(self, flows: &mut EnergyFlows, debit: f64) {
        let (charge, failed) = match self {
            Self::Move => (
                &mut flows.action_charges[ActionType::Move],
                &mut flows.failed_action_effort[ActionType::Move],
            ),
            Self::Eat => (
                &mut flows.action_charges[ActionType::Eat],
                &mut flows.failed_action_effort[ActionType::Eat],
            ),
            Self::Reproduce => (
                &mut flows.action_charges[ActionType::Reproduce],
                &mut flows.failed_action_effort[ActionType::Reproduce],
            ),
            Self::Steal => (
                &mut flows.action_charges[ActionType::StealEnergy],
                &mut flows.failed_action_effort[ActionType::StealEnergy],
            ),
        };
        *charge += debit;
        *failed += debit;
    }
}

fn effort(sim: &Simulation, id: CreatureId, base: f32) -> f32 {
    let creature = &sim.creatures[id];
    sim.config
        .energy
        .action_effort(base, creature.complexity(), creature.age)
}

fn barrier_north(energy: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = make_sim_with_one_creature(energy);
    sim.world.set_barrier(Position::new(5, 4), true);
    sim.config.energy.lifecycle.min_reproduce_age = 0;
    (sim, id)
}

fn ready_parent(energy: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = make_sim_with_one_creature(energy);
    sim.config.energy.lifecycle.min_reproduce_age = 0;
    (sim, id)
}

fn occupied_north(energy: f32) -> (Simulation, CreatureId) {
    let (sim, id, _) = make_sim_two_creatures(energy, 50.0);
    (sim, id)
}

/// A creature on the corner of a bounded world, so a move north leaves it.
fn bounded_corner(energy: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = make_sim_with_one_creature(energy);
    let mut world = crate::kernel::WorldState::new(
        sim.config.world.width,
        sim.config.world.height,
        crate::config::WorldEdgeMode::Bounded,
    );
    world.reconfigure_food(sim.config.world.food.clone());
    let corner = Position::new(0, 0);
    world.place_creature(corner, id);
    sim.world = world;
    sim.creatures[id].position = corner;
    (sim, id)
}

fn capped_parent(energy: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = ready_parent(energy);
    sim.config.population.max_creatures = 1;
    (sim, id)
}

fn young_parent(energy: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = ready_parent(energy);
    sim.config.energy.lifecycle.min_reproduce_age = 100;
    (sim, id)
}

fn poor_parent(energy: f32) -> (Simulation, CreatureId) {
    let (mut sim, id) = ready_parent(energy);
    sim.config.energy.lifecycle.min_reproduce_energy = 1_000.0;
    (sim, id)
}

/// Invariant 2: a failed attempt stores `before − effort` in `f32`, both
/// `action_charges` and `failed_action_effort` rise by the applied debit,
/// and no other flow moves. Only Step 7 counts a litter rejection.
#[test]
fn a_failed_attempt_pays_its_effort_and_nothing_else() {
    let steal = WorldAction::StealEnergy {
        direction: Direction::N,
        effort: 1.0,
    };
    let litter_reproduce = WorldAction::Reproduce {
        direction: Direction::N,
        energy_transfer_fraction: 0.0,
    };
    type Setup = fn(f32) -> (Simulation, CreatureId);
    // (label, setup, action, bucket, litter rejections)
    let cases: [(&str, Setup, WorldAction, Bucket, u64); 11] = [
        (
            "move barrier",
            barrier_north,
            WorldAction::Move(Direction::N),
            Bucket::Move,
            0,
        ),
        (
            "move occupied",
            occupied_north,
            WorldAction::Move(Direction::N),
            Bucket::Move,
            0,
        ),
        (
            "move out of bounds",
            bounded_corner,
            WorldAction::Move(Direction::N),
            Bucket::Move,
            0,
        ),
        (
            "eat configured type",
            make_sim_with_one_creature,
            WorldAction::eat(OrdinaryFoodTypeId::default()),
            Bucket::Eat,
            0,
        ),
        (
            "eat unconfigured type",
            make_sim_with_one_creature,
            WorldAction::eat(OrdinaryFoodTypeId::new(7)),
            Bucket::Eat,
            0,
        ),
        (
            "reproduce invalid target",
            barrier_north,
            REPRODUCE,
            Bucket::Reproduce,
            0,
        ),
        (
            "reproduce population cap",
            capped_parent,
            REPRODUCE,
            Bucket::Reproduce,
            0,
        ),
        (
            "reproduce age",
            young_parent,
            REPRODUCE,
            Bucket::Reproduce,
            0,
        ),
        (
            "reproduce parent energy",
            poor_parent,
            REPRODUCE,
            Bucket::Reproduce,
            0,
        ),
        (
            "reproduce litter",
            ready_parent,
            litter_reproduce,
            Bucket::Reproduce,
            1,
        ),
        (
            "steal no victim",
            make_sim_with_one_creature,
            steal,
            Bucket::Steal,
            0,
        ),
    ];
    for (label, setup, action, bucket, litter) in cases {
        // Arrange
        let (mut sim, id) = setup(100.0);
        let before = sim.creatures[id].energy;
        let effort = effort(&sim, id, bucket.base(&sim, id));
        let mut expected = sim.stats.energy_flows.clone();
        if matches!(bucket, Bucket::Eat) {
            // An eat sizes the per-type intake slots before it reads food.
            expected
                .food_intake_by_type
                .resize(sim.config.world.food.types.len(), 0.0);
        }

        // Act
        execute(&mut sim, id, action);

        // Assert
        let after = sim.creatures[id].energy;
        assert!(effort >= 0.05, "{label}: effort {effort}");
        assert_eq!(after.to_bits(), (before - effort).to_bits(), "{label}");
        bucket.add_failed(&mut expected, applied_debit(before, after));
        assert_eq!(sim.stats.energy_flows, expected, "{label}");
        assert_eq!(
            sim.stats.reproduction_actions_rejected_litter_total, litter,
            "{label}"
        );
    }
}

/// Invariant 3: a rejection never reaches the replication cost; the attempt
/// effort is the only debit even when the parent cannot afford the cost.
#[test]
fn reproduce_below_cost_pays_only_the_attempt_effort() {
    // Effort-cost defaults, 2026-10-10: 1.5 so the parent survives the 1.0 attempt effort.
    let (mut sim, id) = ready_parent(1.5);
    sim.config.energy.complexity_cost.enabled = false;
    sim.config.energy.costs.reproduce_cost = 2.0;
    execute(&mut sim, id, REPRODUCE);
    assert_eq!(sim.creatures[id].energy, 1.5 - 1.0);
    let flows = &sim.stats.energy_flows;
    assert!(flows.action_charges[ActionType::Reproduce] > 0.0);
    assert_eq!(
        flows.action_charges[ActionType::Reproduce],
        flows.failed_action_effort[ActionType::Reproduce]
    );
    assert_eq!(flows.parental_transfer_debit, 0.0);
    assert_eq!(sim.stats.mortality.deaths_total, 0);
}

/// Lifecycle: an effort that exhausts the creature floors it to zero,
/// charges only the energy it had, removes it with the action's sink as the
/// cause, and charges nothing queued after it.
#[test]
fn an_exhausting_effort_removes_the_creature_with_its_sink_and_ends_the_queue() {
    for (action, cause) in [
        (WorldAction::Move(Direction::N), DeathCause::ActionMove),
        (
            WorldAction::eat(OrdinaryFoodTypeId::default()),
            DeathCause::ActionEat,
        ),
        (REPRODUCE, DeathCause::ActionReproduce),
        (
            WorldAction::StealEnergy {
                direction: Direction::N,
                effort: 1.0,
            },
            DeathCause::ActionStealEnergy,
        ),
    ] {
        let (mut sim, id) = barrier_north(0.03);
        run_actions(&mut sim, id, vec![action, WorldAction::NoOp]);
        assert!(!sim.creatures.contains_key(id), "{cause:?}");
        assert_eq!(sim.stats.mortality.count(cause), 1, "{cause:?}");
        assert_eq!(sim.stats.mortality.deaths_total, 1, "{cause:?}");
        let flows = &sim.stats.energy_flows;
        assert_eq!(flows.zero_floor_credit, 0.0, "{cause:?}");
        assert_eq!(
            flows.action_charges.total(),
            applied_debit(0.03, 0.0),
            "{cause:?}"
        );
        assert_eq!(
            sim.stats.energy_flows.action_charges[ActionType::NoOp],
            0.0,
            "{cause:?}"
        );
        assert_eq!(sim.stats.last_tick_noop, 0, "{cause:?}");
    }
}

/// Lifecycle: an attacker whose steal effort exhausts it still takes the
/// transfer: `0.2 × 50` at the founder share and modifier 1.
#[test]
fn a_steal_whose_effort_exhausts_the_attacker_still_transfers() {
    let (mut sim, attacker, victim) = make_sim_two_creatures(0.03, 50.0);
    sim.config.energy.complexity_cost.enabled = false;
    execute(
        &mut sim,
        attacker,
        WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 1.0,
        },
    );
    assert_eq!(sim.creatures[victim].energy, 40.0);
    // Effort-cost defaults, 2026-10-10: the 0.4 effort floor.
    assert_eq!(sim.creatures[attacker].energy, (0.03f32 - 0.4) + 10.0);
    assert_eq!(sim.stats.mortality.deaths_total, 0);
}

/// Telemetry: a steal whose floored effort exceeds the attacker's energy,
/// into an empty cell, charges only the energy the attacker had; the
/// overdraft is neither a charge nor a floor credit.
#[test]
fn an_overdrawn_steal_into_an_empty_cell_charges_only_the_energy_it_had() {
    let (mut sim, id) = make_sim_with_one_creature(0.25);
    sim.config.energy.complexity_cost.enabled = false;
    execute(
        &mut sim,
        id,
        WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 4.0,
        },
    );
    assert!(!sim.creatures.contains_key(id));
    assert_eq!(sim.stats.mortality.count(DeathCause::ActionStealEnergy), 1);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.action_charges[ActionType::StealEnergy], 0.25);
    assert_eq!(flows.failed_action_effort[ActionType::StealEnergy], 0.25);
    assert_eq!(flows.zero_floor_credit, 0.0);
}

/// Telemetry: an overdrawn steal whose loot covers the overdraft charges
/// its whole effort and leaves no floor credit.
#[test]
fn an_overdrawn_steal_with_a_victim_charges_the_effort_the_loot_covers() {
    let (mut sim, attacker, victim) = make_sim_two_creatures(0.25, 300.0);
    sim.config.energy.complexity_cost.enabled = false;
    execute(
        &mut sim,
        attacker,
        WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 1.0,
        },
    );
    assert_eq!(sim.creatures[victim].energy, 240.0);
    assert_eq!(sim.creatures[attacker].energy, (0.25f32 - 0.4) + 60.0);
    let flows = &sim.stats.energy_flows;
    let after_effort = 0.25f32 - 0.4;
    assert_eq!(
        flows.predation_attacker_credit,
        applied_debit(after_effort + 60.0, after_effort)
    );
    assert_eq!(
        flows.action_charges[ActionType::StealEnergy],
        applied_debit(0.25, 0.25 - 0.4)
    );
    assert_eq!(flows.failed_action_effort[ActionType::StealEnergy], 0.0);
    assert_eq!(flows.zero_floor_credit, 0.0);
}

/// Telemetry: a deficit a creature carries into Phase 2 stays a floor
/// credit; an effort it could not pay charges nothing.
#[test]
fn an_inherited_deficit_stays_a_floor_credit_and_the_unpaid_effort_charges_nothing() {
    let (mut sim, id) = make_sim_with_one_creature(-10.0);
    sim.config.energy.complexity_cost.enabled = false;
    execute(
        &mut sim,
        id,
        WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 1.0,
        },
    );
    assert!(!sim.creatures.contains_key(id));
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.action_charges[ActionType::StealEnergy], 0.0);
    assert_eq!(flows.failed_action_effort[ActionType::StealEnergy], 0.0);
    assert!(
        (flows.zero_floor_credit - 10.0).abs() < 1e-6,
        "floor credit {}",
        flows.zero_floor_credit
    );
}

/// Lifecycle: actions discarded after a conception and a gestating parent's
/// tick charge nothing.
#[test]
fn a_gestating_parent_and_its_discarded_actions_are_never_charged() {
    let (mut sim, id) = ready_parent(100.0);
    sim.config
        .energy
        .lifecycle
        .gestation_ticks_per_founder_genome = 1.0;
    let eat = WorldAction::eat(OrdinaryFoodTypeId::default());
    run_actions(
        &mut sim,
        id,
        vec![REPRODUCE, eat, WorldAction::Move(Direction::S)],
    );
    assert!(sim.creatures[id].pending_birth.is_some());
    let charged = sim.stats.energy_flows.action_charges;
    assert_eq!(
        (charged[ActionType::Eat], charged[ActionType::Move]),
        (0.0, 0.0)
    );
    let held = sim.creatures[id].energy;

    execute(&mut sim, id, eat);
    assert_eq!(sim.creatures[id].energy, held);
    assert_eq!(sim.stats.energy_flows.action_charges, charged);
}

/// Success: `NoOp` and `Move` are charged bit for bit what they were before
/// the floor, at the defaults.
#[test]
fn noop_and_move_charges_are_unchanged_at_the_defaults() {
    for action in [WorldAction::NoOp, WorldAction::Move(Direction::S)] {
        let (mut sim, id) = make_sim_with_one_creature(100.0);
        sim.creatures[id].age = 250;
        let costs = &sim.config.energy.costs;
        let base = if action == WorldAction::NoOp {
            costs.noop_cost
        } else {
            costs.move_cost
        };
        let creature = &sim.creatures[id];
        let legacy =
            sim.config
                .energy
                .adjusted_action_cost(base, creature.complexity(), creature.age);
        execute(&mut sim, id, action);
        assert_eq!(
            sim.creatures[id].energy.to_bits(),
            (100.0f32 - legacy).to_bits(),
            "{action:?}"
        );
    }
}

/// Success: an eat credits the food and the clamp, then pays its effort.
#[test]
fn a_successful_eat_credits_then_pays_its_effort() {
    let (mut sim, id) = make_sim_with_one_creature(198.0);
    let eat = WorldAction::eat(OrdinaryFoodTypeId::default());
    sim.world
        .set_food_type(Position::new(5, 5), OrdinaryFoodTypeId::default(), 1.0);
    let effort = effort(&sim, id, sim.config.energy.costs.eat_cost);
    execute(&mut sim, id, eat);
    let max = sim.creatures[id].energy_cap();
    assert_eq!(sim.creatures[id].energy.to_bits(), (max - effort).to_bits());
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.action_charges[ActionType::Eat],
        applied_debit(max, max - effort)
    );
    assert_eq!(flows.failed_action_effort[ActionType::Eat], 0.0);
}

/// Success: a conception pays the attempt effort, then the replication cost,
/// then the litter, as three successive `f32` subtractions; the litter is
/// the fraction of the post-effort, post-cost energy.
#[test]
fn a_conception_pays_effort_then_cost_then_litter() {
    let (mut sim, id) = ready_parent(100.0);
    let energy = &sim.config.energy;
    let effort = effort(&sim, id, energy.costs.reproduce_attempt_cost);
    let creature = &sim.creatures[id];
    let cost = energy.adjusted_action_cost(
        energy.costs.reproduce_cost,
        creature.complexity(),
        creature.age,
    );
    let after_effort = 100.0f32 - effort;
    let after_cost = after_effort - cost;
    let litter = (0.5 * after_cost).min(energy.lifecycle.default_offspring_energy);
    execute(&mut sim, id, REPRODUCE);
    assert_eq!(
        sim.creatures[id].energy.to_bits(),
        (after_cost - litter).to_bits()
    );
    let flows = &sim.stats.energy_flows;
    assert_eq!(
        flows.action_charges[ActionType::Reproduce],
        applied_debit(100.0, after_effort) + applied_debit(after_effort, after_cost)
    );
    assert_eq!(flows.failed_action_effort[ActionType::Reproduce], 0.0);
    assert_eq!(flows.offspring_energy_credit, f64::from(litter));
}

/// T27.F03: a steal's effort base is `effort_rate × intensity × attacker
/// energy`, charged exactly above the `NoOp` floor and at the floor below
/// it or at rate 0; the transfer is `intensity × victim energy`.
#[test]
fn a_steal_pays_its_charge_base_above_the_floor_and_the_floor_below_it() {
    // (rate, attacker energy, effort modifier, expected base)
    for (rate, energy, effort, base) in [
        (0.05, 100.0, 1.0, 0.05 * 0.2 * 100.0),
        (0.05, 100.0, 4.0, 0.05 * 0.8 * 100.0),
        (0.05, 10.0, 1.0, 0.4),
        (0.0, 100.0, 1.0, 0.4),
    ] {
        let (mut sim, attacker, _victim) = make_sim_two_creatures(energy, 50.0);
        sim.config.exchange.effort_rate = rate;
        let creature = &sim.creatures[attacker];
        let charged = sim.config.energy.adjusted_action_cost(
            f32::max(base, sim.config.energy.costs.noop_cost),
            creature.complexity(),
            creature.age,
        );
        execute(
            &mut sim,
            attacker,
            WorldAction::StealEnergy {
                direction: Direction::N,
                effort,
            },
        );
        let stolen = (0.2 * effort).min(1.0) * 50.0;
        assert_eq!(
            sim.creatures[attacker].energy.to_bits(),
            ((energy - charged) + stolen).to_bits(),
            "rate {rate} energy {energy} effort {effort}"
        );
    }
}
