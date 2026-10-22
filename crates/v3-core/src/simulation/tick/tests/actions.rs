use super::super::run_tick;
use super::support::*;
use crate::contracts::{Direction, InputReference, NodeId, Position, WorldInputKey};
use crate::creature::action_log::{ActionResult, ActionType};
use crate::creature::founder::{v3alpha1_founder_genome, FOUNDER_GENOME_SIZE_UNITS};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::simulation::actions::{
    apply_reproduce, apply_steal_energy, BarrierReaderState, MoveBlockedCause,
    ReproductionActionResult, ReproductionInvalidTargetCause,
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
                register_count: 2,
                constants: vec![1.0],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 0,
                    },
                    VmInstruction::AddVote { sink: 1, src: 1 },
                    VmInstruction::Halt,
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
                register_count: 2,
                constants: vec![1.0],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 0,
                    },
                    VmInstruction::AddVote { sink: 9, src: 1 },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn queued_actions_stop_once_action_exhausts_creature_energy() {
    // Eat and Move N tie at one unit: pass 1 commits Eat, pass 2 Move.
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::AddVote { sink: 0, src: 0 }, // Eat
        crate::creature::genome::VmInstruction::AddVote { sink: 1, src: 0 }, // Move N
        crate::creature::genome::VmInstruction::Halt,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(1.0, genome);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.costs.eat_cost = 1.0;

    let creature = &sim.creatures[id];
    let fatal_eat_cost = sim.config.energy.action_effort(
        sim.config.energy.costs.eat_cost,
        creature.cached_complexity,
        creature.age,
    );
    sim.creatures[id].energy = fatal_eat_cost - 0.01;

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.last_tick_eat, 1, "first queued Eat should run");
    assert_eq!(
        sim.stats.last_tick_move, 0,
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
        VmInstruction::AddVote { sink: 9, src: 0 },
        VmInstruction::Halt,
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
                    VmInstruction::WriteActionParam {
                        field_idx: 0,
                        src: 0,
                    },
                    VmInstruction::AddVote { sink: 0, src: 0 },
                    VmInstruction::Halt,
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
fn failed_move_records_blocked_barrier_cause_in_tick() {
    let genome = vm_program_genome(vec![
        crate::creature::genome::VmInstruction::AddVote { sink: 1, src: 0 },
        crate::creature::genome::VmInstruction::Halt,
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
        crate::creature::genome::VmInstruction::AddVote { sink: 9, src: 0 },
        crate::creature::genome::VmInstruction::Halt,
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
        VmInstruction::AddVote { sink: 1, src: 0 },
        VmInstruction::Halt,
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
        VmInstruction::AddVote { sink: 9, src: 0 },
        VmInstruction::Halt,
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
        VmInstruction::AddVote { sink: 1, src: 0 },
        VmInstruction::Halt,
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
            VmInstruction::AddVote { sink: 1, src: 0 },
            VmInstruction::Halt,
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
        VmInstruction::AddVote { sink: 9, src: 0 },
        VmInstruction::Halt,
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
            VmInstruction::AddVote { sink: 9, src: 0 },
            VmInstruction::Halt,
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
    // Bug: the tick loop's Phase 2 action costs can drive energy deeply
    // negative before the death check fires. With a high age multiplier (10x
    // at age 500), a creature with 3.0 energy paying a 60.0 move effort ends
    // up at -57.0.
    //
    // The tick loop must floor energy at 0.0 after each action so that stored
    // energy never goes negative. We test this by queuing TWO failed moves.
    // Without the floor, the first failure drives energy to -57.0 and the second
    // failure subtracts another 60.0 from that. With the floor, the first action
    // clamps energy to 0.0 and the creature is removed before the second action
    // runs — confirming the floor + death check works correctly.
    let genome = vm_program_genome(vec![
        // Queue two Move(N) actions. Both will fail against the barrier.
        crate::creature::genome::VmInstruction::AddVote { sink: 1, src: 0 },
        crate::creature::genome::VmInstruction::AddVote { sink: 1, src: 0 },
        crate::creature::genome::VmInstruction::Halt,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(3.0, genome);

    // High age → maximum age-cost multiplier (10x).
    sim.creatures[id].age = 500;
    // Disable energy decay so it doesn't confound the test.
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.costs.move_cost = 6.0;

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
fn reproduction_resets_child_reward_credit_and_keeps_parent_trace() {
    use crate::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNodeKind, GraphEdge, GraphSource,
    };
    use crate::creature::genome::{
        HebbianRule, OutcomeChannel, PlasticityConfig, RewardModulationConfig,
    };
    for lamarckian in [false, true] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.append_compute_node(crate::creature::genome::cgp::test_support::plastic_node(
            ComputeNodeKind::Constant(1.0),
            vec![GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                0.5,
            )],
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.5,
                weight_clamp: 2.0,
                lamarckian,
                modulation: Some(RewardModulationConfig {
                    reward_source: OutcomeChannel::EnergyDelta,
                    trace_decay: 0.5,
                }),
                retention: 1.0,
            }),
        ));
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
        sim.config.mutation.per_unit_rate = 0.0;
        sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
        sim.creatures[parent]
            .graph_runtime
            .learned
            .set_slot(0, 0, 1.5, 3.0);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let _ = apply_reproduce(parent, &mut sim, Direction::N, 20.0, &mut rng);
        assert_eq!(sim.creatures.len(), 2, "reproduction must actually succeed");
        let (_, child) = sim.creatures.iter().find(|(id, _)| *id != parent).unwrap();
        // A child's credit starts at zero: no trace before its first visit
        // unless its Lamarckian slot is seeded, then a zero one (T25.F06).
        let child_traces: Vec<_> = child
            .graph_runtime
            .learned()
            .edges(0)
            .map(|e| e.trace)
            .collect();
        assert_eq!(child_traces, vec![lamarckian.then_some(0.0)]);
        assert_eq!(child.age, 0);
        let parent_traces: Vec<_> = sim.creatures[parent]
            .graph_runtime
            .learned()
            .edges(0)
            .map(|e| e.trace)
            .collect();
        assert_eq!(parent_traces, vec![Some(3.0)]);
    }
}

/// A one-node VM genome padded with `Noop`s so its `genome_size()` clears the
/// founder anchor by a known margin; `apply_reproduce` is called directly, so
/// the program never runs.
fn oversized_vm_genome() -> CreatureGenome {
    vm_program_genome(vec![VmInstruction::Noop; 300])
}

/// The litter a 1000-energy parent hands over at a fraction of 1.0: the
/// `default_offspring_energy` cap, independent of the charge (T17.F01).
const CAPPED_TRANSFER: f32 = 100.0;

/// One direct `apply_reproduce` on a fresh 1000-energy parent at reproduce
/// age, at a transfer fraction of 1.0 (so the child gets `CAPPED_TRANSFER`);
/// returns the parent's energy after the birth, the child's energy, and the
/// recorded `action_charges.reproduce`.
fn charged_reproduction(genome: CreatureGenome, rate: f32) -> (f32, f32, f32) {
    let (mut sim, parent) = make_sim_with_custom_genome(1000.0, genome);
    sim.config.energy.lifecycle.genome_replication_cost_per_unit = rate;
    sim.config.mutation.per_unit_rate = 0.0;
    // A zero effort floor isolates the replication charge (T16.F02).
    sim.config.energy.costs.noop_cost = 0.0;
    sim.config.energy.costs.reproduce_attempt_cost = 0.0;
    sim.creatures[parent].age = sim.config.energy.lifecycle.min_reproduce_age;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    assert_eq!(
        sim.config.energy.lifecycle.default_offspring_energy,
        CAPPED_TRANSFER
    );
    let result = apply_reproduce(parent, &mut sim, Direction::N, 1.0, &mut rng);
    assert_eq!(result, ReproductionActionResult::Conceived);
    let (_, child) = sim.creatures.iter().find(|(id, _)| *id != parent).unwrap();
    (
        sim.creatures[parent].energy,
        child.energy,
        sim.stats.energy_flows.action_charges.reproduce as f32,
    )
}

/// The Step 5 base charge for a fresh parent of this genome at reproduce age.
fn base_reproduce_charge(genome: &CreatureGenome) -> f32 {
    let (sim, parent) = make_sim_with_custom_genome(1000.0, genome.clone());
    sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.reproduce_cost,
        sim.creatures[parent].cached_complexity,
        sim.config.energy.lifecycle.min_reproduce_age,
    )
}

#[test]
fn parent_above_the_founder_anchor_pays_the_replication_factor_and_child_gets_the_transfer() {
    let genome = oversized_vm_genome();
    let size = genome.genome_size();
    assert!(
        size > FOUNDER_GENOME_SIZE_UNITS,
        "fixture must exceed the anchor"
    );
    let rate = 0.1;
    let factor = 1.0 + rate * (size - FOUNDER_GENOME_SIZE_UNITS) as f32;
    let base = base_reproduce_charge(&genome);
    assert!(base > 0.0);
    let transfer = CAPPED_TRANSFER;

    let (parent_after, child_energy, charged) = charged_reproduction(genome, rate);

    assert!(
        (charged - base * factor).abs() < 1e-4,
        "charge {charged} vs base {base} x factor {factor}"
    );
    let parent_delta = 1000.0 - parent_after;
    assert!(
        (parent_delta - (base * factor + transfer)).abs() < 1e-4,
        "parent delta {parent_delta}"
    );
    assert_eq!(
        child_energy, transfer,
        "nothing of the surcharge reaches the child"
    );
}

#[test]
fn rate_zero_charges_an_oversized_parent_exactly_the_base_charge() {
    let genome = oversized_vm_genome();
    let base = base_reproduce_charge(&genome);
    let (parent_after, child_energy, charged) = charged_reproduction(genome, 0.0);
    // `1.0 * base` is bit-exact, so the parent lands exactly where the
    // rate-free engine leaves it.
    assert_eq!(
        parent_after.to_bits(),
        (1000.0f32 - base - CAPPED_TRANSFER).to_bits()
    );
    // The flow ledger records the realized f32 debit at the parent's energy
    // scale, so allow one rounding step around the nominal base charge.
    assert!(
        (charged - base).abs() < 1e-4,
        "charge {charged} vs base {base}"
    );
    assert_eq!(child_energy, CAPPED_TRANSFER);
}

#[test]
fn founder_reproduce_charge_is_bit_identical_with_and_without_the_replication_rate() {
    let (parent_default, child_default, charged_default) =
        charged_reproduction(v3alpha1_founder_genome(), 0.1);
    let (parent_zero, child_zero, charged_zero) =
        charged_reproduction(v3alpha1_founder_genome(), 0.0);
    let base = base_reproduce_charge(&v3alpha1_founder_genome());
    assert_eq!(parent_default.to_bits(), parent_zero.to_bits());
    assert_eq!(
        parent_default.to_bits(),
        (1000.0f32 - base - CAPPED_TRANSFER).to_bits()
    );
    assert_eq!(charged_default.to_bits(), charged_zero.to_bits());
    assert_eq!(child_default, child_zero);
}

/// A one-node VM genome that eats the named food type: write the type index
/// to the `EatFoodType` field the commit decodes, then vote one `Eat`.
fn eat_type_genome(type_idx: u16) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![f32::from(type_idx), 1.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteActionParam {
                        field_idx: 0,
                        src: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::AddVote { sink: 0, src: 1 },
                    VmInstruction::Halt,
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

/// A one-node VM genome that eats food type 0 leaving the share `leave` on
/// the cell (T02.F08): write `EatLeaveShare`, then vote one `Eat`.
fn eat_leave_genome(leave: f32) -> CreatureGenome {
    use crate::creature::genome::vote::{ActionParamField, VoteSink};
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![leave, 1.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteActionParam {
                        field_idx: ActionParamField::EatLeaveShare.index() as u8,
                        src: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Eat.index() as u8,
                        src: 1,
                    },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    }
}

/// T02.F08: an applied partial eat through `execute_eat` keeps
/// `amount × leave` on the cell, credits only the eaten amount, logs it, and
/// counts as applied and partial, not failed.
#[test]
fn partial_eat_retains_density_credits_the_eaten_amount_and_counts_partial() {
    let pos = Position::new(5, 5);
    let staple = crate::config::OrdinaryFoodTypeId::default();
    let (mut sim, id) = make_sim_with_custom_genome(20.0, eat_leave_genome(0.75));
    sim.action_logs
        .insert(id, crate::creature::action_log::ActionLog::new(8));
    sim.world.set_food_type(pos, staple, 1.0);
    let reward = sim.config.world.food.types[0]
        .energy_per_unit
        .unwrap_or(sim.config.energy.costs.eat_reward_per_food);

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.world.food_at_type(pos, staple), 0.75);
    let credited = sim.stats.energy_flows.food_intake_by_type[0];
    assert!(
        (credited - f64::from(0.25 * reward)).abs() < 1e-4,
        "credited {credited}, reward {reward}"
    );
    let entry = sim
        .action_logs
        .get(id)
        .expect("creature action log")
        .entries()
        .iter()
        .find(|entry| entry.action_type == ActionType::Eat)
        .copied()
        .expect("eat action log entry");
    assert_eq!(entry.result, ActionResult::Success);
    assert_eq!(entry.amount, 0.25);
    assert_eq!(entry.food_type, Some(staple));
    assert_eq!(
        sim.stats.eat_actions_applied_total_by_type.get(&staple),
        Some(&1)
    );
    assert_eq!(
        sim.stats.eat_actions_partial_total_by_type.get(&staple),
        Some(&1)
    );
    assert_eq!(
        sim.stats.eat_actions_failed_total_by_type.get(&staple),
        None
    );
}

/// T02.F08: an eat that clears the cell is applied but not partial; an eat
/// leaving everything is a failed eat with the cell untouched.
#[test]
fn full_eat_is_not_partial_and_leave_one_is_a_failed_eat() {
    let pos = Position::new(5, 5);
    let staple = crate::config::OrdinaryFoodTypeId::default();

    let (mut sim, _id) = make_sim_with_custom_genome(20.0, eat_leave_genome(0.0));
    sim.world.set_food_type(pos, staple, 1.0);
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.world.food_at_type(pos, staple), 0.0);
    assert_eq!(
        sim.stats.eat_actions_applied_total_by_type.get(&staple),
        Some(&1)
    );
    assert_eq!(
        sim.stats.eat_actions_partial_total_by_type.get(&staple),
        None
    );

    let (mut sim, _id) = make_sim_with_custom_genome(20.0, eat_leave_genome(1.0));
    sim.world.set_food_type(pos, staple, 1.0);
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.world.food_at_type(pos, staple), 1.0);
    assert_eq!(
        sim.stats.eat_actions_applied_total_by_type.get(&staple),
        None
    );
    assert_eq!(
        sim.stats.eat_actions_partial_total_by_type.get(&staple),
        None
    );
    assert_eq!(
        sim.stats.eat_actions_failed_total_by_type.get(&staple),
        Some(&1)
    );
}

#[test]
fn move_attempts_count_blocked_and_successful_moves_alike() {
    let genome = vm_program_genome(vec![
        VmInstruction::AddVote { sink: 1, src: 0 },
        VmInstruction::Halt,
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

#[test]
fn neutral_recruitment_costs_follow_applied_genome_growth() {
    use crate::config::{NeutralInputRecruitment as Arm, OrdinaryFoodTypeId};
    use crate::creature::genome::vote::VoteSink;
    use crate::mutation::graph::recruitment::{recruit_source, SourceFamily};
    use crate::simulation::tick::phase_0_energy_charge;
    let config = crate::config::SimulationConfig::default();
    let parent = v3alpha1_founder_genome();
    let families = [
        SourceFamily::Input(InputReference::World(WorldInputKey::area_food_summary(
            OrdinaryFoodTypeId::new(0),
        ))),
        SourceFamily::Input(InputReference::World(WorldInputKey::food_here(
            OrdinaryFoodTypeId::new(1),
        ))),
        SourceFamily::Input(InputReference::World(WorldInputKey::neighbor_food_ring(
            OrdinaryFoodTypeId::new(0),
        ))),
        SourceFamily::Memory(false),
        SourceFamily::Memory(true),
    ];
    for arm in [Arm::SingleChannel, Arm::WholeFamily] {
        for family in &families {
            let mut child = parent.clone();
            recruit_source(&mut child.nodes[0], family, arm, 0, VoteSink::Eat).unwrap();
            let growth = child.genome_size() - parent.genome_size();
            assert!(growth > 0);
            let lifecycle = &config.energy.lifecycle;
            let carrying = phase_0_energy_charge(
                0.0,
                lifecycle.genome_carry_cost_per_unit,
                child.genome_size(),
            );
            let base = base_reproduce_charge(&child);
            let (_, transferred, charge) =
                charged_reproduction(child.clone(), lifecycle.genome_replication_cost_per_unit);
            assert_eq!(transferred, CAPPED_TRANSFER);
            let nominal = base * (1.0 + lifecycle.genome_replication_cost_per_unit * growth as f32);
            // Telemetry records the applied f32 debit at this fixture's 1000
            // energy, including subtraction rounding, not the nominal charge.
            assert_eq!(charge, 1000.0f32 - (1000.0f32 - nominal));
            println!("RECRUIT_COST {arm:?} {family:?}: size={} growth={growth} carry={carrying:.6} reproduction_charge={charge:.6} base={base:.6} next_supply={:.6}", child.genome_size(), config.mutation.per_unit_rate * f64::from(child.genome_size()));
        }
    }
}
