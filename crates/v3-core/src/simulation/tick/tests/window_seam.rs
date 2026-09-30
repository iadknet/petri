//! The creature-window seam (T21.F04): budgeted recording, the tick outcome
//! join and traced/untraced equivalence at tick level.

use std::fmt::Write as _;

use super::super::run_tick;
use super::support::*;
use crate::config::{OrdinaryFoodTypeId, SimulationConfig};
use crate::contracts::{CreatureId, InputReference, NodeId, Position, WorldInputKey};
use crate::creature::action_log::{ActionLog, ActionResult, ActionType};
use crate::creature::founder::vm_decision_founder_genome;
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::state::CreatureState;
use crate::runtime::trace::domain::TickTrace;
use crate::runtime::trace::recording::{ActiveTrace, TraceBudget, TruncationReason};
use crate::runtime::trace::size::TickReserve;
use crate::simulation::energy_accounting::DeathCause;
use crate::simulation::seeding::seed_simulation;
use crate::simulation::Simulation;

const WINDOW_BUDGET: TraceBudget = TraceBudget {
    max_events: 2_048,
    max_bytes: 4 * 1024 * 1024,
};

fn equivalence_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 24;
    cfg.world.height = 24;
    cfg.population.initial_creatures = 24;
    cfg
}

/// Every creature rebuilt around the VM decision founder.
fn with_vm_founders(mut sim: Simulation) -> Simulation {
    let ids: Vec<CreatureId> = sim.creatures.keys().collect();
    for id in ids {
        let old = &sim.creatures[id];
        let rebuilt = CreatureState::new(
            id,
            vm_decision_founder_genome(),
            old.position,
            old.energy,
            old.generation,
            [0, 0, 92, 92, 138, 138],
            0,
            [true; 6],
            old.identity,
            old.shared_memory,
        );
        sim.creatures[id] = rebuilt;
    }
    sim
}

/// Everything a trace must not change: per-creature state including
/// `graph_runtime`, and the cumulative counters.
fn fingerprint(sim: &Simulation) -> String {
    let mut out = String::new();
    for (id, c) in &sim.creatures {
        let g = &c.graph_runtime;
        writeln!(
            out,
            "{id:?} e={:?} p={:?} a={} m={:?} o={:?} s={:?} n={:?} w={:?} t={:?}",
            c.energy,
            c.position,
            c.age,
            c.shared_memory,
            c.previous_outcome,
            g.node_state,
            g.node_outputs,
            g.plasticity_weights,
            g.eligibility_traces,
        )
        .unwrap();
    }
    let s = &sim.stats;
    writeln!(
        out,
        "hops={} vm={} relax={} passes={} decided={} applied={} ticks={} flows={:?}",
        s.mesh_hops_total,
        s.vm_steps_total,
        s.graph_relax_iters_total,
        s.passes_total,
        s.decided_passes_total,
        s.actions_applied_total,
        s.creature_ticks_total,
        s.energy_flows,
    )
    .unwrap();
    out
}

/// The untraced action log entries of `id` at `tick`.
fn logged_at(sim: &Simulation, id: CreatureId, tick: u64) -> Vec<String> {
    sim.action_logs
        .get(id)
        .map(|log| {
            log.entries()
                .iter()
                .filter(|entry| entry.tick == tick)
                .map(|entry| format!("{entry:?}"))
                .collect()
        })
        .unwrap_or_default()
}

/// Runs `traced` and `plain` side by side for 50 ticks, `traced` with a trace
/// whose target moves to a new creature every 8 ticks; asserts each recorded
/// tick applied what the untraced run applied, and equal state every tick.
fn assert_equivalent(mut traced: Simulation, mut plain: Simulation, budget: Option<TraceBudget>) {
    let mut trace: Option<ActiveTrace> = None;
    let mut recorded = 0;
    for tick in 0..50_u64 {
        if tick % 8 == 0 {
            let population: Vec<CreatureId> = traced.creatures.keys().collect();
            if population.is_empty() {
                break;
            }
            let target = population[(tick / 8) as usize % population.len()];
            let mut next = ActiveTrace::new(target, 8);
            next.budget = budget;
            next.include_perception_debug = true;
            trace = Some(next);
        }
        let target = trace.as_ref().map(|t| t.creature_id);
        let before = trace.as_ref().map_or(0, |t| t.ticks.len());
        run_tick(&mut traced, &mut trace);
        run_tick(&mut plain, &mut None);
        assert_eq!(fingerprint(&traced), fingerprint(&plain), "tick {tick}");
        let active = trace.as_ref().unwrap();
        if active.ticks.len() > before {
            recorded += 1;
            let record = active.ticks.last().unwrap();
            let outcome = record.outcome.as_ref().expect("an outcome per tick");
            let applied: Vec<String> = outcome
                .applied
                .iter()
                .map(|action| format!("{:?}", action.entry))
                .collect();
            assert_eq!(
                applied,
                logged_at(&plain, target.unwrap(), tick),
                "tick {tick}"
            );
        }
    }
    assert!(recorded > 0, "the run recorded ticks");
}

#[test]
fn traced_and_untraced_runs_are_identical_for_graph_founders() {
    for budget in [Some(WINDOW_BUDGET), None] {
        let traced = seed_simulation(equivalence_config(), 7);
        let plain = seed_simulation(equivalence_config(), 7);
        assert_equivalent(traced, plain, budget);
    }
}

#[test]
fn traced_and_untraced_runs_are_identical_for_vm_founders() {
    for budget in [Some(WINDOW_BUDGET), None] {
        let traced = with_vm_founders(seed_simulation(equivalence_config(), 9));
        let plain = with_vm_founders(seed_simulation(equivalence_config(), 9));
        assert_equivalent(traced, plain, budget);
    }
}

#[test]
fn a_truncating_event_cap_leaves_both_founders_identical() {
    // One reserved action event leaves two records for the mesh, so the cap
    // truncates inside the first tick's cognition.
    let mut cfg = equivalence_config();
    cfg.runtime.max_actions_per_turn = 1;
    let cap = Some(TraceBudget {
        max_events: 3,
        max_bytes: WINDOW_BUDGET.max_bytes,
    });
    assert_equivalent(
        seed_simulation(cfg.clone(), 11),
        seed_simulation(cfg.clone(), 11),
        cap,
    );
    assert_equivalent(
        with_vm_founders(seed_simulation(cfg.clone(), 11)),
        with_vm_founders(seed_simulation(cfg, 11)),
        cap,
    );
}

// ── Budget ───────────────────────────────────────────────────────────────

/// A one-creature simulation whose founder records several hops a tick.
fn budget_sim() -> (Simulation, CreatureId) {
    let (mut sim, id) = make_sim_with_one_creature(150.0);
    sim.config.runtime.max_actions_per_turn = 1;
    (sim, id)
}

fn unbudgeted_first_tick() -> TickTrace {
    let (mut probe, id) = budget_sim();
    let mut trace = Some(ActiveTrace::new(id, 1));
    run_tick(&mut probe, &mut trace);
    trace.unwrap().ticks.remove(0)
}

#[test]
fn an_event_cap_truncates_at_the_predicted_record_and_ends_the_window() {
    let (sim, id) = budget_sim();
    let reference = unbudgeted_first_tick();
    let reserve = TickReserve::for_config(&sim.config);
    assert_eq!(reserve.events, 1);
    assert!(
        reference.hops.len() + reference.passes.len() > 2,
        "the fixture records more than the cap allows"
    );

    let (mut traced, _) = budget_sim();
    let (mut plain, _) = budget_sim();
    let mut trace = Some(ActiveTrace::new(id, 4));
    trace.as_mut().unwrap().budget = Some(TraceBudget {
        max_events: 3,
        max_bytes: WINDOW_BUDGET.max_bytes,
    });
    run_tick(&mut traced, &mut trace);
    run_tick(&mut plain, &mut None);

    let active = trace.as_ref().unwrap();
    // The first two mesh records fit; the third does not and nothing after it
    // is stored.
    let kept = active.ticks[0].hops.len() + active.ticks[0].passes.len();
    assert_eq!(kept, 2);
    assert_eq!(
        active.ticks[0].hops[0].node_id, reference.hops[0].node_id,
        "the kept hop is the tick's first"
    );
    let truncation = active.truncated.expect("the window truncated");
    assert_eq!(truncation.tick, 0);
    assert_eq!(truncation.reason, TruncationReason::Events);
    assert!(active.is_complete(), "the window ends after the tick");
    assert_eq!(fingerprint(&traced), fingerprint(&plain));
    assert!(active.events <= 3);
}

#[test]
fn a_byte_cap_below_one_hop_truncates_at_the_first_hop() {
    let (sim, id) = budget_sim();
    let reference = unbudgeted_first_tick();
    let reserve = TickReserve::for_config(&sim.config);
    let first_hop = reference.hops[0].retained_bytes();

    let (mut traced, _) = budget_sim();
    let (mut plain, _) = budget_sim();
    let mut trace = Some(ActiveTrace::new(id, 4));
    trace.as_mut().unwrap().budget = Some(TraceBudget {
        max_events: WINDOW_BUDGET.max_events,
        max_bytes: reserve.bytes + first_hop - 1,
    });
    run_tick(&mut traced, &mut trace);
    run_tick(&mut plain, &mut None);

    let active = trace.as_ref().unwrap();
    assert_eq!(active.ticks.len(), 1, "the reserved tick record is kept");
    assert!(active.ticks[0].hops.is_empty());
    assert!(active.ticks[0].passes.is_empty());
    assert!(active.ticks[0].outcome.is_some());
    let truncation = active.truncated.expect("the window truncated");
    assert_eq!(truncation.reason, TruncationReason::Bytes);
    assert!(active.is_complete());
    assert!(active.bytes < reserve.bytes + first_hop);
    assert_eq!(fingerprint(&traced), fingerprint(&plain));
}

#[test]
fn a_tick_whose_reserve_does_not_fit_is_not_recorded() {
    let (sim, id) = budget_sim();
    let reserve = TickReserve::for_config(&sim.config);
    for (budget, reason) in [
        (
            TraceBudget {
                max_events: 0,
                max_bytes: WINDOW_BUDGET.max_bytes,
            },
            TruncationReason::Events,
        ),
        (
            TraceBudget {
                max_events: WINDOW_BUDGET.max_events,
                max_bytes: reserve.bytes - 1,
            },
            TruncationReason::Bytes,
        ),
    ] {
        let (mut traced, _) = budget_sim();
        let (mut plain, _) = budget_sim();
        let mut trace = Some(ActiveTrace::new(id, 4));
        trace.as_mut().unwrap().budget = Some(budget);
        run_tick(&mut traced, &mut trace);
        run_tick(&mut plain, &mut None);
        let active = trace.as_ref().unwrap();
        assert!(active.ticks.is_empty());
        assert_eq!(active.truncated.unwrap().reason, reason);
        assert_eq!(active.truncated.unwrap().tick, 0);
        assert!(active.is_complete());
        assert_eq!((active.events, active.bytes), (0, 0));
        assert_eq!(fingerprint(&traced), fingerprint(&plain));
    }
}

#[test]
fn retained_counts_match_the_records_kept() {
    let (_, id) = budget_sim();
    let (mut traced, _) = budget_sim();
    let mut trace = Some(ActiveTrace::new(id, 3));
    trace.as_mut().unwrap().budget = Some(WINDOW_BUDGET);
    for _ in 0..3 {
        run_tick(&mut traced, &mut trace);
    }
    let active = trace.unwrap();
    assert!(active.truncated.is_none());
    let events: usize = active
        .ticks
        .iter()
        .map(|t| t.hops.len() + t.passes.len() + t.outcome.as_ref().unwrap().applied.len())
        .sum();
    assert_eq!(active.events as usize, events);
    let bytes: u64 = active.ticks.iter().map(TickTrace::retained_bytes).sum();
    assert_eq!(active.bytes, bytes);
}

#[test]
fn a_none_budget_records_every_hop() {
    let (_, id) = budget_sim();
    let reference = unbudgeted_first_tick();
    let (mut traced, _) = budget_sim();
    let mut trace = Some(ActiveTrace::new(id, 1));
    trace.as_mut().unwrap().budget = Some(WINDOW_BUDGET);
    run_tick(&mut traced, &mut trace);
    let budgeted = &trace.as_ref().unwrap().ticks[0];
    assert_eq!(budgeted.hops.len(), reference.hops.len());
    assert_eq!(budgeted.passes.len(), reference.passes.len());
    assert!(!reference.hops.is_empty());
}

// ── Outcome ──────────────────────────────────────────────────────────────

/// Eat and Move N tie at one unit: pass 1 commits Eat, pass 2 Move.
fn eat_then_move_genome() -> CreatureGenome {
    vm_program_genome(vec![
        VmInstruction::AddVote { sink: 0, src: 0 },
        VmInstruction::AddVote { sink: 1, src: 0 },
        VmInstruction::Halt,
    ])
}

fn traced_tick(sim: &mut Simulation, id: CreatureId) -> Option<ActiveTrace> {
    let mut trace = Some(ActiveTrace::new(id, 1));
    run_tick(sim, &mut trace);
    trace
}

#[test]
fn the_outcome_lists_applied_actions_whatever_the_log_capacity() {
    for capacity in [None, Some(0), Some(1)] {
        let (mut sim, id) = make_sim_with_custom_genome(100.0, eat_then_move_genome());
        sim.world.set_food(Position::new(5, 5), 0.5);
        if let Some(capacity) = capacity {
            sim.action_logs.insert(id, ActionLog::new(capacity));
        }
        let trace = traced_tick(&mut sim, id).unwrap();
        let outcome = trace.ticks[0].outcome.as_ref().unwrap();
        let kinds: Vec<ActionType> = outcome
            .applied
            .iter()
            .map(|a| a.entry.action_type)
            .collect();
        assert_eq!(kinds, [ActionType::Eat, ActionType::Move], "{capacity:?}");
        assert!(outcome.after.is_some());
        assert!(outcome.died.is_none());
        assert_eq!(outcome.position, Position::new(5, 5));
        assert_eq!(
            outcome.after.as_ref().unwrap().position,
            sim.creatures[id].position
        );
        assert_eq!(
            outcome.after.as_ref().unwrap().energy,
            sim.creatures[id].energy
        );
        assert_eq!(
            outcome.after.as_ref().unwrap().previous_outcome,
            sim.creatures[id].previous_outcome
        );
        assert!(outcome.phases.is_some());
    }
}

#[test]
fn an_action_heavy_tick_records_every_applied_action() {
    // Every pass commits: a queue at the action cap.
    let (mut sim, id) = make_sim_with_custom_genome(150.0, eat_then_move_genome());
    sim.config.runtime.max_actions_per_turn = 10;
    let trace = traced_tick(&mut sim, id).unwrap();
    let record = &trace.ticks[0];
    let outcome = record.outcome.as_ref().unwrap();
    assert_eq!(outcome.applied.len(), record.final_actions.len());
}

#[test]
fn charge_penalty_and_reward_match_the_energy_flow_deltas() {
    // A rewarded eat then a move blocked by a neighbor.
    let (mut sim, a, _) = make_sim_two_creatures(100.0, 100.0);
    sim.config.energy.costs.eat_cost = 0.5;
    sim.config.energy.costs.failed_action_penalty = 7.5;
    sim.config.startup.ramps.failed_action_penalty.enabled = false;
    sim.creatures[a].genome = eat_then_move_genome();
    sim.world.set_food(Position::new(5, 5), 0.5);
    let mut trace = Some(ActiveTrace::new(a, 1));
    run_tick(&mut sim, &mut trace);
    let outcome = trace.unwrap().ticks.remove(0).outcome.unwrap();
    let eat = &outcome.applied[0];
    assert_eq!(eat.entry.action_type, ActionType::Eat);
    assert_eq!(eat.entry.result, ActionResult::Success);
    assert!(eat.reward > 0.0);
    assert!(eat.charge > 0.0);
    assert_eq!(eat.penalty, 0.0);
    let moved = &outcome.applied[1];
    assert_eq!(moved.entry.result, ActionResult::Blocked);
    assert!(moved.penalty > 0.0);
    assert_eq!(moved.reward, 0.0);
    // The deltas account for every energy change the two actions logged.
    let spent = eat.entry.energy_before - moved.entry.energy_after;
    let accounted =
        eat.charge + eat.penalty - eat.reward + moved.charge + moved.penalty - moved.reward;
    assert!(
        (f64::from(spent) - accounted).abs() < 1e-3,
        "{spent} {accounted}"
    );
}

/// Steals ten energy from the north neighbor.
fn steal_north_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![1.0, 10.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::WriteActionParam {
                        field_idx: 2,
                        src: 1,
                    },
                    VmInstruction::AddVote { sink: 17, src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn a_steal_credits_its_reward_and_the_victim_records_damage() {
    let (mut sim, attacker, victim) = make_sim_two_creatures(100.0, 80.0);
    sim.creatures[attacker].genome = steal_north_genome();
    sim.creatures[victim].genome = vm_program_genome(vec![VmInstruction::Halt]);
    let mut trace = Some(ActiveTrace::new(attacker, 1));
    run_tick(&mut sim, &mut trace);
    let outcome = trace.unwrap().ticks.remove(0).outcome.unwrap();
    let steal = &outcome.applied[0];
    assert_eq!(steal.entry.action_type, ActionType::StealEnergy);
    assert!(steal.reward > 0.0);
    assert!(steal.charge > 0.0);

    let (mut sim, attacker, victim) = make_sim_two_creatures(100.0, 80.0);
    sim.creatures[attacker].genome = steal_north_genome();
    sim.creatures[victim].genome = vm_program_genome(vec![VmInstruction::Halt]);
    let victim_before = sim.creatures[victim].energy;
    let mut trace = Some(ActiveTrace::new(victim, 1));
    run_tick(&mut sim, &mut trace);
    let outcome = trace.unwrap().ticks.remove(0).outcome.unwrap();
    assert!(outcome.damage_received > 0.0);
    assert!(outcome.damage_received < victim_before);
    assert!(outcome.died.is_none());
}

#[test]
fn a_victim_killed_before_or_after_its_turn_records_predation() {
    // Before its turn: the attacker bids higher and acts first. After: the
    // victim acts first by bidding higher itself.
    for victim_first in [false, true] {
        let (mut sim, attacker, victim) = make_sim_two_creatures(150.0, 3.0);
        sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
        sim.config.predation.steal_cost_rate = 0.0;
        sim.creatures[attacker].genome = steal_north_genome();
        sim.creatures[victim].genome = vm_program_genome(vec![
            VmInstruction::AddVote { sink: 25, src: 0 },
            VmInstruction::Halt,
        ]);
        if victim_first {
            sim.creatures[victim].genome = bidding(sim.creatures[victim].genome.clone());
        } else {
            sim.creatures[attacker].genome = bidding(sim.creatures[attacker].genome.clone());
        }
        let mut trace = Some(ActiveTrace::new(victim, 3));
        run_tick(&mut sim, &mut trace);
        assert!(
            !sim.creatures.contains_key(victim),
            "victim_first={victim_first}"
        );
        let record = &trace.as_ref().unwrap().ticks[0];
        let outcome = record.outcome.as_ref().unwrap();
        assert_eq!(
            outcome.died,
            Some(DeathCause::Predation),
            "victim_first={victim_first}"
        );
        assert!(outcome.damage_received > 0.0);
        assert!(outcome.after.is_none());
    }

    // Untraced, nothing is observed.
    let (mut sim, attacker, _) = make_sim_two_creatures(150.0, 3.0);
    sim.config.predation.steal_cost_rate = 0.0;
    sim.creatures[attacker].genome = steal_north_genome();
    run_tick(&mut sim, &mut None);
    assert!(sim.stats.observed_creature.is_none());
    assert!(sim.stats.observed_removal.is_none());
    assert_eq!(sim.stats.observed_damage, 0.0);
}

/// `genome` plus a priority bid, so it acts before an unbidding creature.
fn bidding(mut genome: CreatureGenome) -> CreatureGenome {
    let BackendDef::Vm(def) = &mut genome.nodes[0].backend_def else {
        unreachable!("fixture genomes are VM");
    };
    let at = def.program.len() - 1;
    def.program
        .insert(at, VmInstruction::SetPriorityBid { src: 0 });
    genome
}

#[test]
fn a_creature_dying_in_the_action_phase_keeps_what_it_applied() {
    let (mut sim, id) = make_sim_with_custom_genome(1.0, eat_then_move_genome());
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.startup.ramps.failed_action_penalty.enabled = false;
    let creature = &sim.creatures[id];
    let fatal = sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.eat_cost,
        creature.cached_complexity,
        creature.age,
    ) + sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.failed_action_penalty,
        creature.cached_complexity,
        creature.age,
    );
    sim.creatures[id].energy = fatal - 0.01;
    let trace = traced_tick(&mut sim, id).unwrap();
    let outcome = trace.ticks[0].outcome.as_ref().unwrap();
    assert_eq!(outcome.applied.len(), 1);
    assert_eq!(outcome.applied[0].entry.action_type, ActionType::Eat);
    assert!(outcome.died.is_some());
    assert!(outcome.after.is_none());
}

#[test]
fn a_creature_dying_in_phase_zero_ends_the_window_with_its_cause() {
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
    let mut trace = Some(ActiveTrace::new(id, 5));
    trace.as_mut().unwrap().budget = Some(WINDOW_BUDGET);
    run_tick(&mut sim, &mut trace);
    let active = trace.unwrap();
    assert!(active.ticks.is_empty());
    assert!(active.is_complete());
    assert_eq!(active.removed, Some(DeathCause::LifecycleDecay));
}

fn typed_food_reader_genome() -> CreatureGenome {
    let fruit = OrdinaryFoodTypeId::new(1);
    let grass = OrdinaryFoodTypeId::new(0);
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![
                InputReference::World(WorldInputKey::food_here(grass)),
                InputReference::World(WorldInputKey::food_here(fruit)),
                InputReference::World(WorldInputKey::area_food_summary(fruit)),
            ],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn the_outcome_holds_distinct_per_type_food_banks() {
    let (mut sim, id) = make_sim_with_custom_genome(100.0, typed_food_reader_genome());
    let mut second = sim.config.world.food.types[0].clone();
    second.name = "Fruit".to_string();
    second.initial_coverage = 0.0;
    sim.config.world.food.types.push(second);
    sim.world.reconfigure_food(sim.config.world.food.clone());
    let here = Position::new(5, 5);
    sim.world
        .set_food_type(here, OrdinaryFoodTypeId::new(0), 0.25);
    sim.world
        .set_food_type(here, OrdinaryFoodTypeId::new(1), 0.75);
    sim.world
        .set_food_type(Position::new(6, 5), OrdinaryFoodTypeId::new(1), 0.5);
    let trace = traced_tick(&mut sim, id).unwrap();
    let outcome = trace.ticks[0].outcome.as_ref().unwrap();
    let local = &outcome.typed_local_food.food_here_by_type;
    assert_eq!(local.len(), 2);
    assert_ne!(local[0], local[1]);
    assert_eq!(outcome.typed_area_food.len(), 2);
    assert_ne!(outcome.typed_area_food[0], outcome.typed_area_food[1]);
}
