use super::support::*;
use crate::contracts::Direction;
use crate::contracts::{OrdinaryFoodTypeId, WorldAction};
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{ComputeCostReport, MeshOutput};
use crate::simulation::actions::apply_steal_energy;
use crate::simulation::energy_accounting::DeathCause;
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::tick::run_phase_0;
use rand::{rngs::SmallRng, SeedableRng};

fn decision(actions: Vec<WorldAction>) -> MeshOutput {
    MeshOutput {
        actions,
        cost_report: ComputeCostReport::default(),
        priority_bid: 0.0,
        work_counters: Default::default(),
        energy_observation: Default::default(),
        termination_reason: TerminationReason::ActionEmitted,
    }
}

fn execute(
    sim: &mut crate::simulation::Simulation,
    id: crate::contracts::CreatureId,
    action: WorldAction,
) {
    super::super::run_phase_2(
        sim,
        vec![(id, decision(vec![action]))],
        &mut OutcomeAccumulator::default(),
        &mut SmallRng::seed_from_u64(11),
    );
}

#[test]
fn noop_counts_each_applied_attempt() {
    let (mut sim, id) = make_sim_with_one_creature(8.0);
    sim.config.energy.complexity_cost.enabled = false;
    sim.config.energy.costs.noop_cost = 1.0;

    for attempts in 1u32..=2 {
        execute(&mut sim, id, WorldAction::NoOp);
        assert_eq!(
            sim.creatures[id].lifetime_action_attempted_count,
            u64::from(attempts)
        );
        assert_eq!(sim.stats.last_tick_noop, attempts);
        assert_eq!(
            sim.stats.energy_flows.action_charges.noop,
            f64::from(attempts)
        );
    }
}

/// Reproduce is absent: since T16.F01 the reproduce charge lands only after
/// the energy gates pass (`energy - cost >= transfer > 0`), so it can never
/// cross zero; `reproduce_below_cost_rejects_free_and_records_no_crossing`
/// pins that instead.
#[test]
fn every_base_action_crossing_keeps_its_cause_through_the_action_floor() {
    for (action, cause) in [
        (WorldAction::NoOp, DeathCause::ActionNoop),
        (
            WorldAction::eat(OrdinaryFoodTypeId::default()),
            DeathCause::ActionEat,
        ),
        (WorldAction::Move(Direction::N), DeathCause::ActionMove),
        (
            WorldAction::StealEnergy {
                direction: Direction::N,
                amount: 1.0,
            },
            DeathCause::ActionStealEnergy,
        ),
    ] {
        let (mut sim, id) = make_sim_with_one_creature(1.0);
        sim.config.energy.complexity_cost.enabled = false;
        sim.config.energy.costs.noop_cost = 2.0;
        sim.config.energy.costs.eat_cost = 2.0;
        sim.config.energy.costs.move_cost = 2.0;
        sim.config.energy.costs.reproduce_cost = 2.0;
        sim.config.energy.costs.failed_action_penalty = 0.0;
        sim.config.energy.lifecycle.min_reproduce_age = 0;
        sim.config.predation.steal_cost_rate = 2.0;
        execute(&mut sim, id, action);
        assert_eq!(sim.stats.mortality.count(cause), 1, "{cause:?}");
        assert_eq!(sim.stats.energy_flows.zero_floor_credit, 1.0);
        let charges = sim.stats.energy_flows.action_charges;
        let charged_bucket = match cause {
            DeathCause::ActionNoop => charges.noop,
            DeathCause::ActionEat => charges.eat,
            DeathCause::ActionMove => charges.r#move,
            DeathCause::ActionStealEnergy => charges.steal_energy,
            _ => unreachable!(),
        };
        assert_eq!(charged_bucket, 2.0);
        assert_eq!(
            charges.noop + charges.eat + charges.r#move + charges.reproduce + charges.steal_energy,
            2.0
        );
    }
}

#[test]
fn reproduce_below_cost_rejects_free_and_records_no_crossing() {
    let (mut sim, id) = make_sim_with_one_creature(1.0);
    sim.config.energy.complexity_cost.enabled = false;
    sim.config.energy.costs.reproduce_cost = 2.0;
    sim.config.energy.costs.failed_action_penalty = 0.0;
    sim.config.energy.lifecycle.min_reproduce_age = 0;
    execute(
        &mut sim,
        id,
        WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer: 1.0,
        },
    );
    assert_eq!(sim.stats.mortality.count(DeathCause::ActionReproduce), 0);
    assert_eq!(sim.stats.energy_flows.action_charges.reproduce, 0.0);
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 0.0);
    assert_eq!(sim.creatures[id].energy, 1.0);
    assert!(sim.creatures[id].pending_death_cause.is_none());
}

#[test]
fn failed_penalty_is_a_separate_exhausting_debit_on_every_rejected_action() {
    for action in [
        WorldAction::eat(OrdinaryFoodTypeId::default()),
        WorldAction::Move(Direction::N),
        WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer: 1.0,
        },
        WorldAction::StealEnergy {
            direction: Direction::N,
            amount: 1.0,
        },
    ] {
        let (mut sim, id) = make_sim_with_one_creature(1.0);
        sim.world
            .set_barrier(crate::contracts::Position::new(5, 4), true);
        sim.config.energy.complexity_cost.enabled = false;
        sim.config.energy.costs.eat_cost = 0.0;
        sim.config.energy.costs.move_cost = 0.0;
        sim.config.energy.costs.failed_action_penalty = 2.0;
        sim.config.startup.ramps.failed_action_penalty.enabled = false;
        sim.config.predation.steal_cost_rate = 0.0;
        execute(&mut sim, id, action);
        assert_eq!(
            sim.stats.mortality.count(DeathCause::FailedActionPenalty),
            1
        );
        assert_eq!(sim.stats.energy_flows.failed_action_penalty, 2.0);
        assert_eq!(sim.stats.energy_flows.zero_floor_credit, 1.0);
    }
}

#[test]
fn parental_transfer_counts_only_a_successful_birth_and_newborn_starts_clear() {
    for (starting, spawned) in [(5.0, true), (3.0, false)] {
        let (mut sim, id) = make_sim_with_one_creature(starting);
        sim.config.energy.complexity_cost.enabled = false;
        sim.config.energy.costs.reproduce_cost = 1.0;
        sim.config.energy.costs.failed_action_penalty = 0.0;
        sim.config.energy.lifecycle.min_reproduce_age = 0;
        sim.config.energy.lifecycle.min_reproduce_energy = 0.0;
        execute(
            &mut sim,
            id,
            WorldAction::Reproduce {
                direction: Direction::N,
                energy_transfer: 4.0,
            },
        );
        // T16.F01: the charge lands only on a birth; a rejected attempt is free.
        assert_eq!(
            sim.stats.energy_flows.action_charges.reproduce,
            if spawned { 1.0 } else { 0.0 }
        );
        assert_eq!(
            sim.stats.energy_flows.parental_transfer_debit,
            if spawned { 4.0 } else { 0.0 }
        );
        assert_eq!(
            sim.stats.energy_flows.offspring_energy_credit,
            if spawned { 4.0 } else { 0.0 }
        );
        assert_eq!(
            sim.stats.mortality.count(DeathCause::ParentalTransfer),
            u64::from(spawned)
        );
        assert!(sim
            .creatures
            .values()
            .all(|c| c.pending_death_cause.is_none()));
    }
}

#[test]
fn failed_penalty_observes_the_ramped_scaled_debit_and_rounded_away_charges() {
    let (mut sim, id) = make_sim_with_one_creature(1000.0);
    sim.tick = 25;
    sim.creatures[id].age = 250;
    sim.creatures[id].cached_complexity = 200;
    sim.config.energy.complexity_cost.enabled = true;
    sim.config.energy.costs.eat_cost = 0.0;
    sim.config.energy.costs.failed_action_penalty = 99.0;
    let ramp = &mut sim.config.startup.ramps.failed_action_penalty;
    ramp.enabled = true;
    ramp.start = 2.0;
    ramp.end = 6.0;
    ramp.target_tick = 100;
    let expected_charge = sim.config.energy.adjusted_action_cost(3.0, 200, 250);
    assert!(expected_charge > 3.0);
    execute(
        &mut sim,
        id,
        WorldAction::eat(OrdinaryFoodTypeId::default()),
    );
    let after = 1000.0f32 - expected_charge;
    assert_eq!(sim.creatures[id].energy.to_bits(), after.to_bits());
    assert_eq!(
        sim.stats.energy_flows.failed_action_penalty,
        1000.0 - f64::from(after)
    );
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 0.0);

    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.costs.noop_cost = f32::EPSILON;
    execute(&mut sim, id, WorldAction::NoOp);
    assert_eq!(sim.stats.energy_flows.action_charges.noop, 0.0);
    assert_eq!(sim.creatures[id].energy, 100.0);
}

#[test]
fn cognition_cause_is_visible_before_victim_reduction_and_its_flows_still_commit() {
    use crate::creature::genome::VmInstruction;
    use crate::runtime::trace::recording::ActiveTrace;
    for traced in [false, true] {
        let (mut sim, attacker, victim) = make_sim_two_creatures(100.0, 0.01);
        sim.creatures[attacker].genome = vm_program_genome(vec![]);
        sim.creatures[victim].genome = vm_program_genome(vec![VmInstruction::Noop]);
        sim.config.runtime.vm.opcode_cost_multiplier = 1.0;
        sim.config.runtime.vm.step_ramp_cost = 0.0;
        sim.config.predation.steal_cost_rate = 0.0;
        sim.config.predation.kill_complexity_bonus_multiplier = 0.0;
        let queue = [attacker, victim];
        let inputs = super::super::assemble_sensor_inputs(&sim, &queue);
        let mut trace = traced.then(|| ActiveTrace::new(victim, 1));
        let mut decisions =
            super::super::run_cognition(&mut sim, &inputs, &mut trace, traced.then_some(victim));
        assert_eq!(
            sim.creatures[victim].pending_death_cause,
            Some(DeathCause::VmCompute)
        );
        assert_eq!(
            sim.stats.energy_flows.vm_compute, 0.0,
            "cognition must not mutate global totals"
        );
        let observed = decisions[1].1.energy_observation.vm_compute;
        assert!(observed > 0.0);
        decisions[0].1.actions = vec![WorldAction::StealEnergy {
            direction: Direction::N,
            amount: 1.0,
        }];
        super::super::run_phase_2(
            &mut sim,
            decisions,
            &mut OutcomeAccumulator::default(),
            &mut SmallRng::seed_from_u64(1),
        );
        assert_eq!(sim.stats.mortality.count(DeathCause::VmCompute), 1);
        assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 0);
        assert_eq!(sim.stats.predation_kills_total, 1);
        assert_eq!(
            sim.stats.energy_flows.vm_compute.to_bits(),
            observed.to_bits()
        );
    }
}

#[test]
fn predation_caps_and_bonus_recovery_observe_separate_applied_flows() {
    let (mut sim, attacker, victim) = make_sim_two_creatures(9.0, 2.0);
    sim.config.predation.steal_cost_rate = 0.0;
    sim.config.predation.kill_complexity_bonus_multiplier = 0.5;
    sim.config.energy.lifecycle.max_energy = 10.0;
    sim.creatures[victim].cached_complexity = 4;
    apply_steal_energy(attacker, &mut sim, Direction::N, 10.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.predation_victim_debit, 2.0);
    assert_eq!(flows.predation_attacker_credit, 2.0);
    assert_eq!(flows.predation_kill_bonus_credit, 2.0);
    assert_eq!(flows.maximum_energy_clamp_loss, 3.0);
    assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 1);

    let (mut sim, attacker, victim) = make_sim_two_creatures(1.0, -2.0);
    sim.config.predation.steal_cost_rate = 0.0;
    sim.config.predation.kill_complexity_bonus_multiplier = 1.0;
    sim.creatures[victim].cached_complexity = 3;
    apply_steal_energy(attacker, &mut sim, Direction::N, 10.0);
    assert_eq!(sim.creatures[attacker].energy, 2.0);
    assert_eq!(
        sim.creatures[attacker].pending_death_cause, None,
        "bonus restores positive energy"
    );
}

#[test]
fn zero_carrying_rate_keeps_exposure_and_food_lower_floor_is_observed() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    let size = sim.creatures[id].cached_genome_size;
    run_phase_0(&mut sim);
    assert_eq!(sim.stats.energy_flows.genome_carrying, 0.0);
    assert_eq!(
        sim.stats.energy_flows.genome_size_creature_ticks,
        u64::from(size)
    );

    sim.creatures[id].energy = -10.0;
    sim.creatures[id].pending_death_cause = Some(DeathCause::VmCompute);
    sim.config.energy.costs.eat_cost = 0.0;
    sim.config.energy.costs.eat_reward_per_food = 2.0;
    sim.world.set_food_type(
        sim.creatures[id].position,
        OrdinaryFoodTypeId::default(),
        1.0,
    );
    execute(
        &mut sim,
        id,
        WorldAction::eat(OrdinaryFoodTypeId::default()),
    );
    assert_eq!(sim.stats.energy_flows.food_intake_by_type, vec![2.0]);
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 8.0);
    assert_eq!(sim.stats.mortality.count(DeathCause::VmCompute), 1);
}

#[test]
fn rounded_compute_exhaustion_preserves_the_queued_eat_recovery() {
    use crate::creature::genome::VmInstruction;
    let (mut sim, id) = make_sim_with_custom_genome(
        0.024,
        vm_program_genome(vec![VmInstruction::PushAction { action_type: 1 }]),
    );
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.runtime.vm.opcode_cost_multiplier = 0.1;
    sim.config.runtime.vm.step_ramp_cost = 0.0;
    sim.config.energy.costs.eat_reward_per_food = 2.0;
    sim.config.energy.costs.eat_cost = 0.0;
    sim.world.set_food_type(
        sim.creatures[id].position,
        OrdinaryFoodTypeId::default(),
        1.0,
    );
    crate::simulation::run_tick(&mut sim, &mut None);
    assert_eq!(sim.creatures[id].energy, 2.0);
    assert_eq!(sim.creatures[id].pending_death_cause, None);
    assert_eq!(sim.stats.mortality.deaths_total, 0);
    assert_eq!(sim.stats.energy_flows.vm_compute, f64::from(0.024f32));
    assert_eq!(sim.stats.energy_flows.food_intake_by_type, vec![2.0]);
}

#[test]
fn dispatch_float_totals_follow_priority_queue_order_without_preaggregation() {
    let mut cfg = small_config();
    cfg.population.initial_creatures = 3;
    let mut sim = crate::simulation::seed_simulation(cfg, 7);
    let ids: Vec<_> = sim.creatures.keys().collect();
    let mut decisions = Vec::new();
    for (index, (bid, flow)) in [(1.0, 1.0), (3.0, 1.0e16), (2.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        let mut output = decision(vec![]);
        output.priority_bid = bid;
        output.energy_observation.vm_compute = flow;
        output.energy_observation.graph_compute = flow;
        output.energy_observation.hebbian_learning = flow;
        output.energy_observation.priority_bid = flow;
        decisions.push((ids[index], output));
    }
    super::super::sort_by_priority_bid(&mut decisions);
    super::super::run_phase_2(
        &mut sim,
        decisions,
        &mut OutcomeAccumulator::default(),
        &mut SmallRng::seed_from_u64(1),
    );
    let f = &sim.stats.energy_flows;
    for total in [
        f.vm_compute,
        f.graph_compute,
        f.hebbian_learning,
        f.priority_bid,
    ] {
        assert_eq!(
            total.to_bits(),
            1.0e16f64.to_bits(),
            "adding small dispatches after the large one rounds each independently"
        );
    }
}

#[test]
fn observational_reexecutions_do_not_commit_energy_or_mortality() {
    let (mut sim, _) = make_sim_with_one_creature(100.0);
    crate::simulation::run_tick(&mut sim, &mut None);
    let flows = sim.stats.energy_flows.clone();
    let deaths = sim.stats.mortality.clone();
    crate::simulation::observe_final_actions(&sim);
    crate::simulation::observe_temporal_actions(&sim);
    assert_eq!(sim.stats.energy_flows, flows);
    assert_eq!(sim.stats.mortality, deaths);
}

#[test]
fn food_credit_is_observed_before_cap_and_can_recover_an_exhausted_creature() {
    let (mut sim, id) = make_sim_with_one_creature(-2.0);
    sim.creatures[id].pending_death_cause = Some(DeathCause::VmCompute);
    sim.config.energy.complexity_cost.enabled = false;
    sim.config.energy.costs.eat_cost = 1.0;
    sim.config.energy.costs.eat_reward_per_food = 10.0;
    sim.config.energy.lifecycle.max_energy = 5.0;
    sim.world.set_food_type(
        sim.creatures[id].position,
        OrdinaryFoodTypeId::default(),
        1.0,
    );
    execute(
        &mut sim,
        id,
        WorldAction::eat(OrdinaryFoodTypeId::default()),
    );
    assert_eq!(sim.stats.energy_flows.food_intake_by_type, vec![10.0]);
    assert_eq!(sim.stats.energy_flows.maximum_energy_clamp_loss, 3.0);
    assert_eq!(sim.stats.energy_flows.action_charges.eat, 1.0);
    assert_eq!(sim.creatures[id].energy, 4.0);
    assert_eq!(sim.creatures[id].pending_death_cause, None);
}

#[test]
fn reward_learning_exhaustion_is_retained_until_the_next_phase_zero() {
    use crate::contracts::NodeId;
    use crate::creature::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
    };
    use crate::creature::genome::{
        BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
        RewardModulationConfig,
    };
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            targets: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![ComputeNode {
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
                        lamarckian: false,
                        modulation: Some(RewardModulationConfig {
                            reward_source: OutcomeChannel::EnergyDelta,
                            trace_decay: 0.5,
                        }),
                    }),
                }],
                output_sinks: vec![],
                action_bank: vec![],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
        }],
    };
    let (mut sim, id) = make_sim_with_custom_genome(1.0, genome);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.energy.costs.noop_cost = 0.0;
    sim.config.runtime.graph_node_base_cost = 0.0;
    sim.config.runtime.reward_learning_cost = 2.0;
    crate::simulation::run_tick(&mut sim, &mut None);
    assert_eq!(sim.stats.energy_flows.reward_learning, 2.0);
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 1.0);
    assert_eq!(sim.creatures[id].energy, 0.0, "removal must remain delayed");
    assert_eq!(
        sim.creatures[id].pending_death_cause,
        Some(DeathCause::RewardLearning)
    );
    sim.config.energy.lifecycle.energy_decay_per_tick = 1.0;
    run_phase_0(&mut sim);
    assert_eq!(sim.stats.mortality.count(DeathCause::RewardLearning), 1);
    assert_eq!(sim.stats.mortality.count(DeathCause::LifecycleDecay), 0);
}

#[test]
fn removals_count_once_and_distinguish_external_from_unattributed() {
    let (mut sim, alive, dead) = make_sim_two_creatures(8.0, -2.0);
    sim.remove_creature(alive);
    sim.remove_creature(alive);
    sim.remove_creature(dead);
    assert_eq!(sim.stats.mortality.deaths_total, 2);
    assert_eq!(sim.stats.mortality.count(DeathCause::ExternalRemoval), 1);
    assert_eq!(sim.stats.mortality.count(DeathCause::Unattributed), 1);
    assert_eq!(sim.stats.energy_flows.external_removal_loss, 8.0);
}

#[test]
fn phase_zero_assigns_each_creatures_decay_first_cause_at_the_zero_boundary() {
    for (initial_energy, cause) in [
        (1.0, DeathCause::LifecycleDecay),
        (2.0, DeathCause::LifecycleDecay),
        (3.0, DeathCause::GenomeCarrying),
    ] {
        let (mut sim, id) = make_sim_with_one_creature(initial_energy);
        sim.config.energy.lifecycle.energy_decay_per_tick = 2.0;
        sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.5;
        sim.creatures[id].cached_genome_size = 4;

        run_phase_0(&mut sim);

        assert!(!sim.creatures.contains_key(id));
        assert_eq!(sim.stats.mortality.deaths_total, 1);
        assert_eq!(sim.stats.mortality.count(cause), 1, "{initial_energy}");
    }
}

#[test]
fn phase_zero_allocates_combined_debit_and_counts_even_dead_creature_exposure() {
    let (mut sim, decay, carrying) = make_sim_two_creatures(1.0, 3.0);
    sim.config.energy.lifecycle.energy_decay_per_tick = 2.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.5;
    sim.creatures[decay].cached_genome_size = 4;
    sim.creatures[carrying].cached_genome_size = 4;
    run_phase_0(&mut sim);
    assert_eq!(sim.stats.mortality.count(DeathCause::LifecycleDecay), 1);
    assert_eq!(sim.stats.mortality.count(DeathCause::GenomeCarrying), 1);
    assert_eq!(sim.stats.energy_flows.lifecycle_decay, 4.0);
    assert_eq!(sim.stats.energy_flows.genome_carrying, 4.0);
    assert_eq!(sim.stats.energy_flows.genome_size_creature_ticks, 8);
}

#[test]
fn predation_retains_exhausted_victim_cause_and_signed_reverse_transfer() {
    let (mut sim, attacker, victim) = make_sim_two_creatures(1.0, -2.0);
    sim.config.predation.steal_cost_rate = 0.0;
    sim.config.predation.kill_complexity_bonus_multiplier = 0.0;
    sim.creatures[victim].pending_death_cause = Some(DeathCause::VmCompute);
    apply_steal_energy(attacker, &mut sim, Direction::N, 10.0);
    assert_eq!(sim.stats.mortality.count(DeathCause::VmCompute), 1);
    assert_eq!(sim.stats.predation_kills_total, 1);
    assert_eq!(sim.stats.energy_flows.predation_victim_debit, -2.0);
    assert_eq!(sim.stats.energy_flows.predation_attacker_credit, -2.0);
    assert_eq!(
        sim.creatures[attacker].pending_death_cause,
        Some(DeathCause::Predation)
    );
    sim.remove_creature(attacker);
    assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 1);
}
