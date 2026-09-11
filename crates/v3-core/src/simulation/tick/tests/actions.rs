use super::super::run_tick;
use super::support::*;
use crate::contracts::{Direction, InputReference, NodeId, Position, WorldInputKey};
use crate::creature::action_log::{ActionResult, ActionType};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::simulation::actions::{
    apply_move, apply_reproduce, apply_steal_energy, apply_typed_eat, BarrierReaderState,
    MoveBlockedCause, ReproductionInvalidTargetCause,
};
use crate::simulation::seeding::seed_simulation;
use rand::SeedableRng;
use std::collections::HashSet;

fn move_north_with_barrier_reader_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                    VmInstruction::PushAction { action_type: 2 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        }],
    }
}

fn reproduce_north_with_barrier_reader_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                    VmInstruction::PushAction { action_type: 3 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn queued_actions_stop_once_action_exhausts_creature_energy() {
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::PushAction { action_type: 1 }, // Eat
        crate::creature::genome::VmInstruction::PushAction { action_type: 0 }, // NoOp
        crate::creature::genome::VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(1.0, genome);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.startup.ramps.failed_action_penalty.enabled = false;

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
fn zero_transfer_reproduction_records_energy_constraint_in_action_log() {
    let genome = vm_program_genome(vec![
        VmInstruction::PushAction { action_type: 3 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(80.0, genome);
    sim.config.energy.lifecycle.min_reproduce_age = 0;
    sim.action_logs
        .insert(id, crate::creature::action_log::ActionLog::new(8));

    run_tick(&mut sim, &mut None);

    let entry = sim
        .action_logs
        .get(id)
        .expect("creature action log")
        .entries()
        .back()
        .expect("reproduce action log entry");
    assert_eq!(entry.action_type, ActionType::Reproduce);
    assert_eq!(entry.result, ActionResult::EnergyConstraints);
    assert_eq!(sim.stats.last_tick_reproduce, 1);
}

#[test]
fn tick_action_log_records_applied_food_type_amount_and_result() {
    // Arrange
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![1.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteWorldActionMeta {
                        slot_idx: 0,
                        src: 0,
                    },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        }],
    };
    let (mut sim, id) = make_sim_with_custom_genome(10.0, genome);
    sim.config
        .world
        .food
        .types
        .push(crate::config::FoodTypeConfig::default());
    sim.config.energy.costs.eat_reward_per_food = 3.0;
    sim.world.reconfigure_food(sim.config.world.food.clone());
    let pos = sim.creatures[id].position;
    sim.world
        .set_food_type(pos, crate::config::OrdinaryFoodTypeId::new(1), 0.35);
    sim.action_logs
        .insert(id, crate::creature::action_log::ActionLog::new(8));

    // Act
    run_tick(&mut sim, &mut None);

    // Assert
    let entry = sim
        .action_logs
        .get(id)
        .expect("creature action log")
        .entries()
        .back()
        .expect("eat action log entry");
    assert_eq!(entry.action_type, ActionType::Eat);
    assert_eq!(
        entry.food_type,
        Some(crate::config::OrdinaryFoodTypeId::new(1))
    );
    assert!((entry.amount - 0.35).abs() < 1e-6);
    assert_eq!(entry.result, ActionResult::Success);
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
    let eat_succeeded = apply_typed_eat(
        creature,
        &mut sim.world,
        &sim.config,
        crate::config::OrdinaryFoodTypeId::default(),
    );
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
fn failed_action_penalty_ramp_uses_effective_tick_value() {
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::PushAction { action_type: 2 },
        crate::creature::genome::VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(100.0, genome);
    sim.world.set_barrier(Position::new(5, 4), true);
    sim.config.energy.costs.move_cost = 0.0;
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.energy.age_cost.enabled = false;
    sim.config.energy.complexity_cost.enabled = false;

    sim.config.startup.ramps.failed_action_penalty.enabled = true;
    sim.config.startup.ramps.failed_action_penalty.start = 5.0;
    sim.config.startup.ramps.failed_action_penalty.end = 30.0;
    sim.config.startup.ramps.failed_action_penalty.target_tick = 2;
    sim.config.apply_startup_overrides();

    let e0 = sim.creatures[id].energy;
    run_tick(&mut sim, &mut None);
    let e1 = sim.creatures[id].energy;
    assert!(
        (e0 - e1 - 5.0).abs() < 1e-4,
        "tick 0 penalty should be ramp start"
    );

    run_tick(&mut sim, &mut None);
    let e2 = sim.creatures[id].energy;
    assert!(
        (e1 - e2 - 17.5).abs() < 1e-4,
        "tick 1 penalty should be midpoint interpolation"
    );

    run_tick(&mut sim, &mut None);
    let e3 = sim.creatures[id].energy;
    assert!(
        (e2 - e3 - 30.0).abs() < 1e-4,
        "tick 2 penalty should match ramp end/runtime"
    );
}

#[test]
fn failed_move_records_blocked_barrier_cause_in_tick() {
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::PushAction { action_type: 2 },
        crate::creature::genome::VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, genome);
    sim.world.set_barrier(Position::new(5, 4), true);

    run_tick(&mut sim, &mut None);

    assert_eq!(
        sim.stats
            .move_actions_blocked_total_by_cause
            .get(&MoveBlockedCause::Barrier)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn reproduce_invalid_target_records_barrier_cause_in_tick() {
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::PushAction { action_type: 3 },
        crate::creature::genome::VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, genome);
    sim.world.set_barrier(Position::new(5, 4), true);

    run_tick(&mut sim, &mut None);

    assert_eq!(
        sim.stats
            .reproduction_actions_rejected_invalid_target_total_by_cause
            .get(&ReproductionInvalidTargetCause::Barrier)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn move_barrier_neighbor_counters_split_by_barrier_reader_state() {
    let no_reader_genome = vm_program_genome(vec![
        VmInstruction::PushAction { action_type: 2 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut no_reader_sim, _) = make_sim_with_custom_genome(100.0, no_reader_genome);
    no_reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut no_reader_sim, &mut None);
    assert_eq!(
        no_reader_sim
            .stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        no_reader_sim
            .stats
            .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );

    let (mut reader_sim, _) =
        make_sim_with_custom_genome(100.0, move_north_with_barrier_reader_genome());
    reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut reader_sim, &mut None);
    assert_eq!(
        reader_sim
            .stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::HasBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        reader_sim
            .stats
            .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::HasBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn reproduction_barrier_neighbor_counters_split_by_barrier_reader_state() {
    let no_reader_genome = vm_program_genome(vec![
        VmInstruction::PushAction { action_type: 3 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut no_reader_sim, _) = make_sim_with_custom_genome(100.0, no_reader_genome);
    no_reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut no_reader_sim, &mut None);
    assert_eq!(
        no_reader_sim
            .stats
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        no_reader_sim
            .stats
            .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );

    let (mut reader_sim, _) =
        make_sim_with_custom_genome(100.0, reproduce_north_with_barrier_reader_genome());
    reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut reader_sim, &mut None);
    assert_eq!(
        reader_sim
            .stats
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::HasBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        reader_sim
            .stats
            .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
            .get(&BarrierReaderState::HasBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn move_blocked_avoidable_counters_track_alternative_targets_by_reader_state() {
    let no_reader_genome = vm_program_genome(vec![
        VmInstruction::PushAction { action_type: 2 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut no_reader_sim, _) = make_sim_with_custom_genome(100.0, no_reader_genome);
    no_reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut no_reader_sim, &mut None);
    assert_eq!(
        no_reader_sim
            .stats
            .move_actions_blocked_avoidable_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );

    let (mut no_reader_unavoidable_sim, _) = make_sim_with_custom_genome(
        100.0,
        vm_program_genome(vec![
            VmInstruction::PushAction { action_type: 2 },
            VmInstruction::ExecuteActionQueue,
        ]),
    );
    let center = Position::new(5, 5);
    for dir in Direction::ALL {
        if let Some(neighbor) = no_reader_unavoidable_sim
            .world
            .resolve_neighbor(center, dir)
        {
            no_reader_unavoidable_sim.world.set_barrier(neighbor, true);
        }
    }
    run_tick(&mut no_reader_unavoidable_sim, &mut None);
    assert_eq!(
        no_reader_unavoidable_sim
            .stats
            .move_actions_blocked_avoidable_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        0
    );

    let (mut reader_sim, _) =
        make_sim_with_custom_genome(100.0, move_north_with_barrier_reader_genome());
    reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut reader_sim, &mut None);
    assert_eq!(
        reader_sim
            .stats
            .move_actions_blocked_avoidable_total_by_reader_state
            .get(&BarrierReaderState::HasBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn reproduction_invalid_target_avoidable_counters_track_alternative_targets_by_reader_state() {
    let no_reader_genome = vm_program_genome(vec![
        VmInstruction::PushAction { action_type: 3 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut no_reader_sim, _) = make_sim_with_custom_genome(100.0, no_reader_genome);
    no_reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut no_reader_sim, &mut None);
    assert_eq!(
        no_reader_sim
            .stats
            .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );

    let (mut no_reader_unavoidable_sim, _) = make_sim_with_custom_genome(
        100.0,
        vm_program_genome(vec![
            VmInstruction::PushAction { action_type: 3 },
            VmInstruction::ExecuteActionQueue,
        ]),
    );
    let center = Position::new(5, 5);
    for dir in Direction::ALL {
        if let Some(neighbor) = no_reader_unavoidable_sim
            .world
            .resolve_neighbor(center, dir)
        {
            no_reader_unavoidable_sim.world.set_barrier(neighbor, true);
        }
    }
    run_tick(&mut no_reader_unavoidable_sim, &mut None);
    assert_eq!(
        no_reader_unavoidable_sim
            .stats
            .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
            .get(&BarrierReaderState::NoBarrierReader)
            .copied()
            .unwrap_or(0),
        0
    );

    let (mut reader_sim, _) =
        make_sim_with_custom_genome(100.0, reproduce_north_with_barrier_reader_genome());
    reader_sim.world.set_barrier(Position::new(5, 4), true);
    run_tick(&mut reader_sim, &mut None);
    assert_eq!(
        reader_sim
            .stats
            .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
            .get(&BarrierReaderState::HasBarrierReader)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn reproduction_invalid_target_helper_classifies_out_of_bounds_and_contention() {
    let world = crate::kernel::WorldState::new(5, 5, crate::config::WorldEdgeMode::Bounded);
    let successful_spawn_targets = HashSet::new();

    assert_eq!(
        super::super::classify_reproduction_invalid_target_cause(
            &world,
            None,
            &successful_spawn_targets
        ),
        ReproductionInvalidTargetCause::OutOfBounds
    );

    let mut successful_spawn_targets = HashSet::new();
    successful_spawn_targets.insert(Position::new(1, 1));
    assert_eq!(
        super::super::classify_reproduction_invalid_target_cause(
            &world,
            Some(Position::new(1, 1)),
            &successful_spawn_targets
        ),
        ReproductionInvalidTargetCause::Contention
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

#[test]
fn energy_floor_at_zero_after_action_execution() {
    // Bug: the tick loop's Phase 2 action costs (move_cost + failed_action_penalty)
    // can drive energy deeply negative before the death check fires. With a high
    // age multiplier (10x at age 500), a creature with 3.0 energy paying 60.0 in
    // costs ends up at -57.0.
    //
    // The tick loop must floor energy at 0.0 after each action so that stored
    // energy never goes negative. We test this by queuing TWO failed moves.
    // Without the floor, the first failure drives energy to -57.0 and the second
    // failure subtracts another 60.0 from that. With the floor, the first action
    // clamps energy to 0.0 and the creature is removed before the second action
    // runs — confirming the floor + death check works correctly.
    let genome = vm_program_genome(vec![
        // Queue two Move(N) actions. Both will fail against the barrier.
        crate::creature::genome::VmInstruction::PushAction { action_type: 2 },
        crate::creature::genome::VmInstruction::PushAction { action_type: 2 },
        crate::creature::genome::VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(3.0, genome);

    // High age → maximum age-cost multiplier (10x).
    sim.creatures[id].age = 500;
    // Disable energy decay so it doesn't confound the test.
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.startup.ramps.failed_action_penalty.enabled = false;

    // Place barrier directly north of creature at (5,5) so moves fail.
    sim.world.set_barrier(Position::new(5, 4), true);

    run_tick(&mut sim, &mut None);

    // Creature should be dead after the first failed move exhausted its energy.
    assert!(
        !sim.creatures.contains_key(id),
        "creature should die after action costs exhaust energy",
    );
    // Only ONE move should have been attempted — the floor clamp ensures the
    // creature is removed before processing the second action.
    assert_eq!(
        sim.stats.last_tick_move, 1,
        "only one move should execute before creature dies; second should be prevented",
    );
    // Verify no surviving creature has negative energy (global invariant).
    for (cid, creature) in &sim.creatures {
        assert!(
            creature.energy >= 0.0,
            "creature {cid:?} has negative energy {}",
            creature.energy,
        );
    }
}

#[test]
fn reproduction_resets_reward_credit_including_frozen_tick_base() {
    use crate::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    };
    use crate::creature::genome::{
        HebbianRule, OutcomeChannel, PlasticityConfig, RewardModulationConfig,
    };
    for lamarckian in [false, true] {
        let mut def =
            CgpGraphBackendDef::new_with_fixed_outputs(&crate::config::MutationConfig::default());
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.5,
                weight_clamp: 2.0,
                lamarckian,
                modulation: Some(RewardModulationConfig {
                    reward_source: OutcomeChannel::EnergyDelta,
                    trace_decay: 0.5,
                }),
            }),
        });
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(def),
                targets: vec![],
            }],
        };
        let (mut sim, parent) = make_sim_with_custom_genome(1000.0, genome);
        sim.config.mutation.mutation_probability = 0.0;
        sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
        sim.creatures[parent].graph_runtime.eligibility_traces = vec![vec![Box::new([3.0])]];
        sim.creatures[parent]
            .graph_runtime
            .tick_start_eligibility_traces = vec![vec![Box::new([1.0])]];
        sim.creatures[parent].graph_runtime.plasticity_weights = vec![vec![Box::new([1.5])]];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let _ = apply_reproduce(parent, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(sim.creatures.len(), 2, "reproduction must actually succeed");
        let (_, child) = sim.creatures.iter().find(|(id, _)| *id != parent).unwrap();
        assert!(child.graph_runtime.eligibility_traces.is_empty());
        assert!(child.graph_runtime.tick_start_eligibility_traces.is_empty());
        assert_eq!(child.age, 0);
        assert_eq!(
            sim.creatures[parent].graph_runtime.eligibility_traces[0][0][0],
            3.0
        );
    }
}

/// A one-node VM genome that eats the named food type: load the type index
/// into the world-action meta slot the Eat decoder reads, then push and run.
fn eat_type_genome(type_idx: u16) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![f32::from(type_idx)],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteWorldActionMeta {
                        slot_idx: 0,
                        src: 0,
                    },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        }],
    }
}

/// Give an existing simulation a second food type, keeping the world's food
/// catalog and the config in step.
fn add_second_food_type(sim: &mut crate::simulation::Simulation) {
    let mut second = sim.config.world.food.types[0].clone();
    second.name = "Fruit".to_string();
    second.initial_coverage = 0.0;
    sim.config.world.food.types.push(second);
    sim.world.reconfigure_food(sim.config.world.food.clone());
}

#[test]
fn typed_eat_counters_rise_only_for_the_type_an_applied_eat_consumed() {
    let pos = Position::new(5, 5);
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, eat_type_genome(1));
    add_second_food_type(&mut sim);
    let fruit = crate::config::OrdinaryFoodTypeId::new(1);

    // No fruit on the cell: the Eat applies no food and counts nothing.
    run_tick(&mut sim, &mut None);
    assert!(
        sim.stats.eat_actions_applied_total_by_type.is_empty(),
        "an Eat that consumed nothing must not be counted: {:?}",
        sim.stats.eat_actions_applied_total_by_type
    );

    sim.world.set_food_type(pos, fruit, 1.0);
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats
            .eat_actions_applied_total_by_type
            .get(&fruit)
            .copied(),
        Some(1),
        "a successful typed Eat is counted against the type it named"
    );
    assert_eq!(
        sim.stats
            .eat_actions_applied_total_by_type
            .get(&crate::config::OrdinaryFoodTypeId::default())
            .copied(),
        None,
        "the untouched type stays absent"
    );

    // The cell is now bare again, so the next tick's Eat adds nothing.
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats
            .eat_actions_applied_total_by_type
            .get(&fruit)
            .copied(),
        Some(1)
    );
}

#[test]
fn failed_typed_eat_counters_rise_only_for_the_type_the_action_named() {
    let pos = Position::new(5, 5);
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, eat_type_genome(1));
    add_second_food_type(&mut sim);
    let staple = crate::config::OrdinaryFoodTypeId::default();
    let fruit = crate::config::OrdinaryFoodTypeId::new(1);

    // No fruit on the cell: the Eat fails and is counted against fruit alone,
    // even though the cell holds no staple either.
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats
            .eat_actions_failed_total_by_type
            .get(&fruit)
            .copied(),
        Some(1),
        "a typed Eat that found nothing is counted against the type it named"
    );
    assert_eq!(
        sim.stats
            .eat_actions_failed_total_by_type
            .get(&staple)
            .copied(),
        None,
        "the type the action did not name stays absent"
    );

    // With fruit on the cell the Eat applies, so the failure total holds.
    sim.world.set_food_type(pos, fruit, 1.0);
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats
            .eat_actions_failed_total_by_type
            .get(&fruit)
            .copied(),
        Some(1),
        "an applied Eat is not a failed one"
    );
    assert_eq!(
        sim.stats
            .eat_actions_applied_total_by_type
            .get(&fruit)
            .copied(),
        Some(1)
    );
}

#[test]
fn move_attempts_count_blocked_and_successful_moves_alike() {
    let genome = vm_program_genome(vec![
        VmInstruction::PushAction { action_type: 2 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(1000.0, genome);
    sim.world.set_barrier(Position::new(5, 4), true);

    run_tick(&mut sim, &mut None);
    assert_eq!(sim.stats.move_actions_attempted_total, 1);
    assert_eq!(
        sim.stats
            .move_actions_blocked_total_by_cause
            .get(&MoveBlockedCause::Barrier)
            .copied(),
        Some(1)
    );

    sim.world.set_barrier(Position::new(5, 4), false);
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats.move_actions_attempted_total, 2,
        "a successful move is an attempt too"
    );
    assert_eq!(
        sim.stats
            .move_actions_blocked_total_by_cause
            .get(&MoveBlockedCause::Barrier)
            .copied(),
        Some(1),
        "the successful move must not add a blocked count"
    );
}

#[test]
fn per_type_standing_density_matches_the_applied_growth_summary() {
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, vm_program_genome(vec![]));
    add_second_food_type(&mut sim);
    sim.config.world.food.growth_rate = 0.0;
    sim.world.reconfigure_food(sim.config.world.food.clone());
    sim.world.set_food_type(
        Position::new(2, 2),
        crate::config::OrdinaryFoodTypeId::new(0),
        0.5,
    );
    sim.world.set_food_type(
        Position::new(3, 3),
        crate::config::OrdinaryFoodTypeId::new(1),
        0.25,
    );

    run_tick(&mut sim, &mut None);

    let expected: Vec<f32> = (0..2)
        .map(|idx| {
            sim.world
                .total_food_by_type(crate::config::OrdinaryFoodTypeId::new(idx))
        })
        .collect();
    assert_eq!(sim.stats.last_tick_food_total_density_by_type, expected);
}
