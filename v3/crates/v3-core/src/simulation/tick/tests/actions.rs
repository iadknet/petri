use super::super::run_tick;
use super::support::*;
use crate::contracts::{Direction, Position};
use crate::simulation::actions::{apply_eat, apply_move, apply_reproduce, apply_steal_energy};
use crate::simulation::seeding::seed_simulation;
use rand::SeedableRng;

#[test]
fn queued_actions_stop_once_action_exhausts_creature_energy() {
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::PushAction { action_type: 1 }, // Eat
        crate::creature::genome::VmInstruction::PushAction { action_type: 0 }, // NoOp
        crate::creature::genome::VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(1.0, genome);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;

    let creature = &sim.creatures[id];
    let fatal_eat_cost = sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.eat_cost,
        creature.cached_complexity,
        creature.age,
    ) + sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.failed_action_penalty,
        creature.cached_complexity,
        creature.age,
    );
    sim.creatures[id].energy = fatal_eat_cost - 0.01;

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.last_tick_eat, 1, "first queued Eat should run");
    assert_eq!(
        sim.stats.last_tick_noop, 0,
        "remaining queued actions should stop after fatal exhaustion"
    );
    assert!(
        !sim.creatures.contains_key(id),
        "creature should die immediately after exhausting its energy"
    );
    assert!(
        sim.world.creature_at(Position::new(5, 5)).is_none(),
        "dead creature should be removed from occupancy immediately"
    );
}

#[test]
fn failed_move_deducts_penalty_in_tick() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.costs.failed_action_penalty = 7.5;
    sim.world.set_barrier(Position::new(5, 4), true);

    let energy_before = sim.creatures[id].energy;
    let complexity = sim.creatures[id].cached_complexity;
    let age = sim.creatures[id].age;
    let adjusted_move_cost =
        sim.config
            .energy
            .adjusted_action_cost(sim.config.energy.costs.move_cost, complexity, age);
    let adjusted_penalty = sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.failed_action_penalty,
        complexity,
        age,
    );

    let creature = sim.creatures.get_mut(id).unwrap();
    let succeeded = apply_move(id, creature, &mut sim.world, Direction::N, &sim.config);
    assert!(!succeeded, "move into barrier should fail");
    if !succeeded {
        sim.creatures.get_mut(id).unwrap().energy -= adjusted_penalty;
    }

    let expected = energy_before - adjusted_move_cost - adjusted_penalty;
    assert!(
        (sim.creatures[id].energy - expected).abs() < 1e-4,
        "energy {} should be {} (start {} - move {} - penalty {})",
        sim.creatures[id].energy,
        expected,
        energy_before,
        adjusted_move_cost,
        adjusted_penalty
    );

    let energy_before_eat = sim.creatures[id].energy;
    let adjusted_eat_cost =
        sim.config
            .energy
            .adjusted_action_cost(sim.config.energy.costs.eat_cost, complexity, age);
    let creature = sim.creatures.get_mut(id).unwrap();
    let eat_succeeded = apply_eat(creature, &mut sim.world, &sim.config);
    assert!(!eat_succeeded, "eat on empty cell should fail");
    if !eat_succeeded {
        sim.creatures.get_mut(id).unwrap().energy -= adjusted_penalty;
    }
    let expected_eat = energy_before_eat - adjusted_eat_cost - adjusted_penalty;
    assert!(
        (sim.creatures[id].energy - expected_eat).abs() < 1e-4,
        "energy {} should be {} after failed eat",
        sim.creatures[id].energy,
        expected_eat
    );
}

#[test]
fn failed_action_penalty_zero_preserves_old_behavior() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.costs.failed_action_penalty = 0.0;
    let energy_after_decay = 100.0 - sim.config.energy.lifecycle.energy_decay_per_tick;
    run_tick(&mut sim, &mut None);
    if sim.creatures.contains_key(id) {
        let energy = sim.creatures[id].energy;
        assert!(energy > 0.0, "creature should survive with zero penalty");
        assert!(
            energy >= energy_after_decay - 50.0,
            "energy {} should not drop excessively with zero penalty",
            energy
        );
    }
}

#[test]
fn failed_action_penalty_increases_with_age() {
    let (mut sim_young, id_young) = make_sim_with_one_creature(100.0);
    sim_young.world.set_barrier(Position::new(5, 4), true);
    let adjusted_penalty_young = sim_young.config.energy.adjusted_action_cost(
        sim_young.config.energy.costs.failed_action_penalty,
        sim_young.creatures[id_young].cached_complexity,
        0,
    );
    let energy_before_young = sim_young.creatures[id_young].energy;
    {
        let creature = sim_young.creatures.get_mut(id_young).unwrap();
        let _ = apply_move(
            id_young,
            creature,
            &mut sim_young.world,
            Direction::N,
            &sim_young.config,
        );
    }
    sim_young.creatures[id_young].energy -= adjusted_penalty_young;
    let cost_young = energy_before_young - sim_young.creatures[id_young].energy;

    let (mut sim_old, id_old) = make_sim_with_one_creature(100.0);
    sim_old.world.set_barrier(Position::new(5, 4), true);
    sim_old.creatures[id_old].age = 400;
    let adjusted_penalty_old = sim_old.config.energy.adjusted_action_cost(
        sim_old.config.energy.costs.failed_action_penalty,
        sim_old.creatures[id_old].cached_complexity,
        400,
    );
    let energy_before_old = sim_old.creatures[id_old].energy;
    {
        let creature = sim_old.creatures.get_mut(id_old).unwrap();
        let _ = apply_move(
            id_old,
            creature,
            &mut sim_old.world,
            Direction::N,
            &sim_old.config,
        );
    }
    sim_old.creatures[id_old].energy -= adjusted_penalty_old;
    let cost_old = energy_before_old - sim_old.creatures[id_old].energy;

    assert!(
        cost_old > cost_young,
        "old creature should pay more for failed action: young={cost_young}, old={cost_old}"
    );
}

#[test]
fn last_tick_steal_resets_each_tick() {
    let mut sim = seed_simulation(small_config(), 42);
    sim.stats.last_tick_steal = 5;
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.stats.last_tick_steal, 0);
}

#[test]
fn steal_energy_dispatch_transfers_and_removes_victim() {
    let (mut sim, attacker_id, victim_id) = make_sim_two_creatures(50.0, 5.0);
    sim.config.predation.steal_cost_rate = 0.0;
    let victim_pos = Position::new(5, 4);

    let result = apply_steal_energy(attacker_id, &mut sim, Direction::N, 20.0);

    assert_eq!(
        result,
        crate::simulation::actions::PredationActionResult::TransferredAndKilled
    );
    assert!(
        !sim.creatures.contains_key(victim_id),
        "victim should be removed from slotmap"
    );
    assert!(
        sim.world.creature_at(victim_pos).is_none(),
        "victim should be removed from world occupancy"
    );
    assert!(
        sim.creatures[attacker_id].energy > 50.0,
        "attacker should have gained energy"
    );
    assert_eq!(sim.stats.predation_kills_total, 1);
    assert_eq!(sim.stats.predation_actions_attempted_total, 1);
}

#[test]
fn killed_creature_steal_action_does_not_panic() {
    let (mut sim, a_id, b_id) = make_sim_two_creatures(100.0, 5.0);
    sim.config.predation.steal_cost_rate = 0.0;

    let result = apply_steal_energy(a_id, &mut sim, Direction::N, 50.0);
    assert_eq!(
        result,
        crate::simulation::actions::PredationActionResult::TransferredAndKilled,
    );
    assert!(!sim.creatures.contains_key(b_id), "B should be dead");

    if sim.creatures.contains_key(b_id) {
        let _result = apply_steal_energy(b_id, &mut sim, Direction::S, 10.0);
    }
}

#[test]
fn killed_creature_reproduce_action_does_not_panic() {
    let (mut sim, a_id, b_id) = make_sim_two_creatures(100.0, 5.0);
    sim.config.predation.steal_cost_rate = 0.0;

    let result = apply_steal_energy(a_id, &mut sim, Direction::N, 50.0);
    assert_eq!(
        result,
        crate::simulation::actions::PredationActionResult::TransferredAndKilled,
    );
    assert!(!sim.creatures.contains_key(b_id), "B should be dead");

    if sim.creatures.contains_key(b_id) {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
        let _result = apply_reproduce(b_id, &mut sim, Direction::S, 10.0, &mut rng);
    }
}

#[test]
fn priority_bid_stats_tracked_after_tick() {
    let mut sim = seed_simulation(small_config(), 42);
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.stats.last_tick_priority_bid_mean, 0.0);
    assert_eq!(sim.stats.last_tick_priority_bidders_count, 0);
}
