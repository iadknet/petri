//! T14.F07 per-creature lifetime counters: incremented only at the
//! sequential Phase 2 sites and zero at birth.

use super::support::*;
use crate::contracts::{Direction, OrdinaryFoodTypeId, Position, WorldAction};
use crate::creature::action_log::{ActionType, ACTION_TYPE_COUNT};
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{ComputeCostReport, MeshOutput};
use crate::simulation::outcomes::OutcomeAccumulator;
use rand::{rngs::SmallRng, SeedableRng};

fn execute(
    sim: &mut crate::simulation::Simulation,
    id: crate::contracts::CreatureId,
    actions: Vec<WorldAction>,
) {
    let decision = MeshOutput {
        actions,
        cost_report: ComputeCostReport::default(),
        priority_bid: 0.0,
        work_counters: Default::default(),
        energy_observation: Default::default(),
        termination_reason: TerminationReason::ActionEmitted,
    };
    super::super::run_phase_2(
        sim,
        vec![(id, decision)],
        &mut OutcomeAccumulator::default(),
        &mut SmallRng::seed_from_u64(11),
    );
}

fn by_type(
    sim: &crate::simulation::Simulation,
    id: crate::contracts::CreatureId,
) -> [u64; ACTION_TYPE_COUNT as usize] {
    sim.creatures[id].lifetime_actions_attempted_by_type
}

fn slot(action: ActionType) -> [u64; ACTION_TYPE_COUNT as usize] {
    let mut expected = [0; ACTION_TYPE_COUNT as usize];
    expected[action as usize] = 1;
    expected
}

#[test]
fn every_creature_starts_with_zeroed_profile_counters() {
    let (sim, id) = make_sim_with_one_creature(8.0);
    let creature = &sim.creatures[id];
    assert_eq!(creature.lifetime_actions_attempted_by_type, [0; 5]);
    assert!(creature.lifetime_eats_applied_by_type.is_empty());
    assert_eq!(creature.lifetime_predation_kills_count, 0);
    assert_eq!(creature.lifetime_predation_hits_taken_count, 0);
}

#[test]
fn noop_counts_in_its_own_type_slot() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    execute(&mut sim, id, vec![WorldAction::NoOp]);
    assert_eq!(by_type(&sim, id), slot(ActionType::NoOp));
}

#[test]
fn a_failed_eat_counts_the_attempt_but_no_applied_eat() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    sim.config.energy.costs.failed_action_penalty = 0.0;
    execute(
        &mut sim,
        id,
        vec![WorldAction::eat(OrdinaryFoodTypeId::new(0))],
    );
    assert_eq!(by_type(&sim, id), slot(ActionType::Eat));
    assert!(sim.creatures[id].lifetime_eats_applied_by_type.is_empty());
}

#[test]
fn an_applied_eat_counts_under_the_food_type_it_named() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    let pos = sim.creatures[id].position;
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(0), 1.0);
    execute(
        &mut sim,
        id,
        vec![WorldAction::eat(OrdinaryFoodTypeId::new(0))],
    );
    assert_eq!(by_type(&sim, id), slot(ActionType::Eat));
    assert_eq!(sim.creatures[id].lifetime_eats_applied_by_type, vec![1]);
    assert_eq!(
        sim.stats.eat_actions_applied_total_by_type[&OrdinaryFoodTypeId::new(0)],
        1
    );
}

#[test]
fn a_blocked_move_counts_in_the_move_slot() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    sim.config.energy.costs.failed_action_penalty = 0.0;
    let pos = sim.creatures[id].position;
    sim.world.set_barrier(Position::new(pos.x, pos.y - 1), true);
    execute(&mut sim, id, vec![WorldAction::Move(Direction::N)]);
    assert_eq!(by_type(&sim, id), slot(ActionType::Move));
    assert_eq!(sim.creatures[id].lifetime_blocked_move_count, 1);
}

#[test]
fn a_rejected_reproduce_counts_in_the_reproduce_slot() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    sim.config.energy.costs.failed_action_penalty = 0.0;
    sim.config.energy.lifecycle.min_reproduce_age = 1_000;
    execute(
        &mut sim,
        id,
        vec![WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer_fraction: 1.0,
        }],
    );
    assert_eq!(by_type(&sim, id), slot(ActionType::Reproduce));
}

#[test]
fn a_rejected_steal_counts_the_attempt_and_no_kill_or_hit() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    sim.config.energy.costs.failed_action_penalty = 0.0;
    execute(
        &mut sim,
        id,
        vec![WorldAction::StealEnergy {
            direction: Direction::N,
            amount: 1.0,
        }],
    );
    assert_eq!(by_type(&sim, id), slot(ActionType::StealEnergy));
    assert_eq!(sim.creatures[id].lifetime_predation_kills_count, 0);
    assert_eq!(sim.creatures[id].lifetime_predation_hits_taken_count, 0);
}

#[test]
fn a_mixed_sequence_keeps_the_by_type_sum_equal_to_the_attempt_count() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.costs.failed_action_penalty = 0.0;
    sim.config.energy.lifecycle.min_reproduce_age = 1_000;
    let pos = sim.creatures[id].position;
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(0), 1.0);
    execute(
        &mut sim,
        id,
        vec![
            WorldAction::NoOp,
            WorldAction::eat(OrdinaryFoodTypeId::new(0)),
            WorldAction::eat(OrdinaryFoodTypeId::new(0)),
            WorldAction::Move(Direction::N),
            WorldAction::Move(Direction::S),
            WorldAction::Reproduce {
                direction: Direction::N,
                energy_transfer_fraction: 1.0,
            },
            WorldAction::StealEnergy {
                direction: Direction::E,
                amount: 1.0,
            },
            WorldAction::NoOp,
        ],
    );
    let creature = &sim.creatures[id];
    assert_eq!(creature.lifetime_actions_attempted_by_type, [2, 2, 2, 1, 1]);
    assert_eq!(
        creature
            .lifetime_actions_attempted_by_type
            .iter()
            .sum::<u64>(),
        creature.lifetime_action_attempted_count
    );
    assert_eq!(creature.lifetime_eats_applied_by_type, vec![1]);
}

#[test]
fn a_child_starts_at_zero_while_its_parent_keeps_its_counters() {
    use crate::simulation::actions::{apply_reproduce, ReproductionActionResult};
    let (mut sim, parent) = make_sim_with_one_creature(1000.0);
    sim.config.mutation.mutation_probability = 0.0;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    let pos = sim.creatures[parent].position;
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(0), 1.0);
    execute(
        &mut sim,
        parent,
        vec![WorldAction::eat(OrdinaryFoodTypeId::new(0))],
    );
    sim.creatures[parent].lifetime_predation_kills_count = 3;
    sim.creatures[parent].lifetime_predation_hits_taken_count = 2;

    let mut rng = SmallRng::seed_from_u64(42);
    let result = apply_reproduce(parent, &mut sim, Direction::N, 10.0, &mut rng);
    assert_eq!(result, ReproductionActionResult::Spawned);
    let (_, child) = sim.creatures.iter().find(|(id, _)| *id != parent).unwrap();

    assert_eq!(child.lifetime_actions_attempted_by_type, [0; 5]);
    assert!(child.lifetime_eats_applied_by_type.is_empty());
    assert_eq!(child.lifetime_predation_kills_count, 0);
    assert_eq!(child.lifetime_predation_hits_taken_count, 0);
    let parent = &sim.creatures[parent];
    assert_eq!(
        parent.lifetime_actions_attempted_by_type,
        slot(ActionType::Eat)
    );
    assert_eq!(parent.lifetime_eats_applied_by_type, vec![1]);
    assert_eq!(parent.lifetime_predation_kills_count, 3);
    assert_eq!(parent.lifetime_predation_hits_taken_count, 2);
}
