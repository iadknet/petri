use super::support::*;
use crate::contracts::Direction;
use crate::contracts::{OrdinaryFoodTypeId, WorldAction};
use crate::creature::action_log::ActionType;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::ledger::DeathCause;
use crate::runtime::types::ComputeCostReport;
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::tick::run_phase_0;
use crate::simulation::tick::steal_now;
use rand::{rngs::SmallRng, SeedableRng};

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
            sim.stats.energy_flows.action_charges[ActionType::NoOp],
            f64::from(attempts)
        );
    }
}

/// Reproduce crosses at its Step 0 attempt effort (T16.F02); the
/// replication cost lands only after the energy gates pass, so it never
/// crosses zero.
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
            WorldAction::Reproduce {
                direction: Direction::N,
                energy_transfer_fraction: 1.0,
            },
            DeathCause::ActionReproduce,
        ),
        (
            WorldAction::StealEnergy {
                direction: Direction::N,
                effort: 1.0,
            },
            DeathCause::ActionStealEnergy,
        ),
    ] {
        let (mut sim, id) = make_sim_with_one_creature(1.0);
        sim.config.energy.complexity_cost.enabled = false;
        sim.config.energy.costs.noop_cost = 2.0;
        sim.config.energy.costs.eat_cost = 2.0;
        sim.config.energy.costs.move_cost = 2.0;
        sim.config.energy.costs.reproduce_attempt_cost = 2.0;
        sim.config.energy.lifecycle.min_reproduce_age = 0;
        sim.config.exchange.effort_rate = 2.0;
        execute(&mut sim, id, action);
        assert_eq!(sim.stats.mortality.count(cause), 1, "{cause:?}");
        // The 1.0 overdraft nets out of the charge, not into the floor.
        assert_eq!(sim.stats.energy_flows.zero_floor_credit, 0.0);
        let charges = sim.stats.energy_flows.action_charges;
        let charged_bucket = match cause {
            DeathCause::ActionNoop => charges[ActionType::NoOp],
            DeathCause::ActionEat => charges[ActionType::Eat],
            DeathCause::ActionMove => charges[ActionType::Move],
            DeathCause::ActionReproduce => charges[ActionType::Reproduce],
            DeathCause::ActionStealEnergy => charges[ActionType::StealEnergy],
            _ => unreachable!(),
        };
        assert_eq!(charged_bucket, 1.0);
        assert_eq!(charges.total(), 1.0);
    }
}

#[test]
fn parental_transfer_counts_only_a_successful_birth_and_newborn_starts_clear() {
    for (starting, spawned) in [(5.0, true), (3.0, false)] {
        let (mut sim, id) = make_sim_with_one_creature(starting);
        sim.config.energy.complexity_cost.enabled = false;
        sim.config.energy.costs.reproduce_cost = 1.0;
        // A zero effort floor isolates the replication charge (T16.F02).
        sim.config.energy.costs.noop_cost = 0.0;
        sim.config.energy.costs.reproduce_attempt_cost = 0.0;
        sim.config.energy.lifecycle.min_reproduce_age = 0;
        sim.config.energy.lifecycle.min_reproduce_energy = 0.0;
        // T17.F01: the whole post-cost energy goes to the child; the litter
        // floor (4.0) accepts the 5.0 start (4.0 after the charge) and
        // refuses the 3.0 start (2.0 after the charge).
        sim.config.energy.lifecycle.initial_energy = 4.0;
        execute(
            &mut sim,
            id,
            WorldAction::Reproduce {
                direction: Direction::N,
                energy_transfer_fraction: 1.0,
            },
        );
        // T16.F01: the charge lands only on a birth; a rejected attempt is free.
        assert_eq!(
            sim.stats.energy_flows.action_charges[ActionType::Reproduce],
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
fn a_charge_rounded_away_records_no_debit() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.costs.noop_cost = f32::EPSILON;
    execute(&mut sim, id, WorldAction::NoOp);
    assert_eq!(sim.stats.energy_flows.action_charges[ActionType::NoOp], 0.0);
    assert_eq!(sim.creatures[id].energy, 100.0);
}

#[test]
fn cognition_cause_is_visible_before_victim_reduction_and_its_flows_still_commit() {
    use crate::genome::VmInstruction;
    use crate::simulation::recording::ActiveTrace;
    for traced in [false, true] {
        let (mut sim, attacker, victim) = make_sim_two_with_genomes(
            100.0,
            0.01,
            vm_raw_program_genome(vec![]),
            vm_raw_program_genome(vec![VmInstruction::Noop]),
        );
        sim.config.runtime.vm.opcode_cost_multiplier = 1.0;
        sim.config.runtime.vm.step_ramp_cost = 0.0;
        sim.config.exchange.effort_rate = 0.0;
        sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
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
            effort: 1.0,
        }];
        super::super::run_phase_2(
            &mut sim,
            decisions,
            &mut OutcomeAccumulator::default(),
            &mut SmallRng::seed_from_u64(1),
        );
        assert_eq!(sim.stats.mortality.count(DeathCause::VmCompute), 1);
        assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 0);
        // An exhausted victim yields nothing to take: no kill (T27.F03).
        assert_eq!(sim.stats.predation_kills_total, 0);
        assert_eq!(sim.stats.predation_actions_rejected_total, 1);
        assert_eq!(
            sim.stats.energy_flows.vm_compute.to_bits(),
            observed.to_bits()
        );
    }
}

#[test]
fn predation_caps_and_bonus_recovery_observe_separate_applied_flows() {
    let (mut sim, attacker, victim) =
        make_sim_two_with_genomes(9.0, 2.0, v3alpha1_founder_genome(), genome_of_complexity(4));
    sim.config.exchange.effort_rate = 0.0;
    sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
    sim.config.predation.kill_complexity_bonus_multiplier = 0.5;
    sim.creatures[attacker].body.capacity = 10.0;
    assert_eq!(sim.creatures[victim].complexity(), 4);
    steal_now(attacker, &mut sim, Direction::N, 10.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.predation_victim_debit, 2.0);
    assert_eq!(flows.predation_attacker_credit, 2.0);
    assert_eq!(flows.predation_kill_bonus_credit, 2.0);
    assert_eq!(flows.maximum_energy_clamp_loss, 3.0);
    assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 1);

    // An exhausted victim yields nothing: no kill, no bonus, no flow.
    let (mut sim, attacker, victim) = make_sim_two_with_genomes(
        1.0,
        -2.0,
        v3alpha1_founder_genome(),
        genome_of_complexity(3),
    );
    sim.config.exchange.effort_rate = 0.0;
    sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
    sim.config.predation.kill_complexity_bonus_multiplier = 1.0;
    assert_eq!(sim.creatures[victim].complexity(), 3);
    steal_now(attacker, &mut sim, Direction::N, 10.0);
    assert_eq!(sim.creatures[attacker].energy, 1.0);
    assert_eq!(sim.creatures[victim].energy, -2.0);
    let flows = &sim.stats.energy_flows;
    assert_eq!(flows.predation_victim_debit, 0.0);
    assert_eq!(flows.predation_attacker_credit, 0.0);
    assert_eq!(flows.predation_kill_bonus_credit, 0.0);
    assert_eq!(sim.stats.predation_kills_total, 0);
}

#[test]
fn zero_carrying_rate_keeps_exposure_and_food_lower_floor_is_observed() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    let size = sim.creatures[id].genome_size();
    run_phase_0(&mut sim);
    assert_eq!(sim.stats.energy_flows.genome_carrying, 0.0);
    assert_eq!(
        sim.stats.energy_flows.genome_size_creature_ticks,
        u64::from(size)
    );

    sim.creatures[id].energy = -10.0;
    sim.creatures[id].pending_death_cause = Some(DeathCause::VmCompute);
    sim.config.energy.costs.eat_cost = 0.0;
    sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
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
    use crate::genome::VmInstruction;
    // `LoadConst`, `AddVote`, `Halt` at 0.1x cost 0.027 in `f64`, whose `f32`
    // rounds up to exactly the starting energy: the settled energy is 0.0
    // with a positive effective remainder. One action per tick, so the
    // committed `Eat` ends the tick before a second pass could charge.
    let (mut sim, id) = make_sim_with_custom_genome(
        0.027,
        vm_program_genome(vec![
            VmInstruction::AddVote { sink: 0, src: 0 },
            VmInstruction::Halt,
        ]),
    );
    sim.config.runtime.max_actions_per_turn = 1;
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.runtime.vm.opcode_cost_multiplier = 0.1;
    sim.config.runtime.vm.step_ramp_cost = 0.0;
    sim.config.energy.costs.eat_reward_per_food = 2.0;
    sim.config.energy.costs.eat_cost = 0.0;
    sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
    sim.world.set_food_type(
        sim.creatures[id].position,
        OrdinaryFoodTypeId::default(),
        1.0,
    );
    crate::simulation::run_tick(&mut sim, &mut None);
    assert_eq!(sim.creatures[id].energy, 2.0);
    assert_eq!(sim.creatures[id].pending_death_cause, None);
    assert_eq!(sim.stats.mortality.deaths_total, 0);
    assert_eq!(sim.stats.energy_flows.vm_compute, f64::from(0.027f32));
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
        output.energy_observation.mesh_ramp = flow;
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
    for total in [f.vm_compute, f.graph_compute, f.mesh_ramp, f.priority_bid] {
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
    sim.creatures[id].body.capacity = 5.0;
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
    assert_eq!(sim.stats.energy_flows.action_charges[ActionType::Eat], 1.0);
    assert_eq!(sim.creatures[id].energy, 4.0);
    assert_eq!(sim.creatures[id].pending_death_cause, None);
}

/// One Graph node whose single compute node carries plasticity; with
/// `reward` it is reward-modulated on `EnergyDelta`.
fn learning_genome(reward: bool) -> crate::genome::CreatureGenome {
    use crate::contracts::NodeId;
    use crate::contracts::OutcomeChannel;
    use crate::genome::cgp::{CgpGraphBackendDef, ComputeNodeKind, GraphEdge, GraphSource};
    use crate::genome::{
        BackendDef, CreatureGenome, HebbianRule, Learning, NodeGenome, RewardModulationConfig,
    };
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome::new(
            NodeId::new(0),
            vec![],
            BackendDef::Graph(
                CgpGraphBackendDef::new_with_fixed_outputs()
                    .with_compute_nodes(vec![crate::genome::cgp::test_support::plastic_node(
                        ComputeNodeKind::Constant(1.0),
                        vec![GraphEdge::new(
                            GraphSource::SharedMemory {
                                slot: 0,
                                previous: false,
                            },
                            0.5,
                        )],
                        Some(Learning {
                            rule: HebbianRule::Classic,
                            learning_rate: 0.5,
                            weight_clamp: 2.0,
                            lamarckian: false,
                            modulation: reward.then_some(RewardModulationConfig {
                                reward_source: OutcomeChannel::EnergyDelta,
                                trace_decay: 0.5,
                            }),
                            retention: 1.0,
                        }),
                    )])
                    .with_output_sinks(vec![]),
            ),
            vec![],
        )],
    }
}

/// Reward-modulated learning is free (T25.F02): a tick in which every other
/// cost is zero leaves the learner's energy where it was, with no pending
/// death cause, while its reward-modulated updates still run.
#[test]
fn reward_learning_charges_no_energy() {
    let (mut sim, id) = make_sim_with_custom_genome(1.0, learning_genome(true));
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.energy.costs.noop_cost = 0.0;
    sim.config.runtime.graph_node_base_cost = 0.0;
    crate::simulation::run_tick(&mut sim, &mut None);
    assert!(sim.stats.reward_modulated_updates_total > 0);
    assert_eq!(sim.creatures[id].energy, 1.0);
    assert_eq!(sim.creatures[id].pending_death_cause, None);
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 0.0);
}

/// The Phase 2.5 zero floor is its own step: it floors exactly the creatures
/// the reward pass reaches (an outcome bank and a reward-modulated node) and
/// credits `zero_floor_credit`; a creature without either is untouched.
#[test]
fn reward_phase_zero_floor_reaches_only_reward_learners_with_an_outcome_bank() {
    for (label, reward, banked, floored) in [
        ("reward learner with a bank", true, true, true),
        ("plain Hebbian learner", false, true, false),
        ("newborn reward learner", true, false, false),
    ] {
        let (mut sim, id) = make_sim_with_custom_genome(1.0, learning_genome(reward));
        let mut outcomes = OutcomeAccumulator::default();
        if banked {
            outcomes.snapshot_energy(id, 1.0);
        }
        sim.creatures[id].energy = -1.5;
        super::super::run_reward_learning(&mut sim, &outcomes);
        let (energy, credit) = if floored { (0.0, 1.5) } else { (-1.5, 0.0) };
        assert_eq!(sim.creatures[id].energy, energy, "{label}");
        assert_eq!(sim.stats.energy_flows.zero_floor_credit, credit, "{label}");
    }
}

/// One Graph node with no compute node whose one `Eat` vote edge reads
/// shared memory and is reward-modulated on `EnergyDelta` (T25.F06).
fn sink_learning_genome() -> crate::genome::CreatureGenome {
    use crate::contracts::votes::VoteSink;
    use crate::contracts::OutcomeChannel;
    use crate::genome::cgp::{
        CgpGraphBackendDef, EdgeOccurrence, GraphEdge, GraphSource, OutputSinkKind,
    };
    use crate::genome::{BackendDef, HebbianRule, Learning, RewardModulationConfig};
    let mut genome = learning_genome(true);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let eat = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap();
    def.push_edge(
        eat,
        GraphEdge::new(
            GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            0.5,
        ),
    );
    def.edge_mut(EdgeOccurrence::new(eat, 0))
        .set_learning(Some(Learning {
            rule: HebbianRule::Classic,
            learning_rate: 0.5,
            weight_clamp: 2.0,
            lamarckian: false,
            modulation: Some(RewardModulationConfig {
                reward_source: OutcomeChannel::EnergyDelta,
                trace_decay: 0.5,
            }),
            retention: 1.0,
        }));
    *genome.nodes[0].backend_edit().1 = BackendDef::Graph(def);
    genome
}

/// The zero-energy floor at cost 0 reaches a creature whose only
/// reward-modulated edge is a sink edge, and its reward learning charges
/// nothing (T25.F06 invariant 6).
#[test]
fn the_zero_floor_reaches_a_reward_modulated_sink_edge_at_cost_zero() {
    let (mut sim, id) = make_sim_with_custom_genome(1.0, sink_learning_genome());
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    sim.config.energy.costs.noop_cost = 0.0;
    sim.config.runtime.graph_node_base_cost = 0.0;
    crate::simulation::run_tick(&mut sim, &mut None);
    assert!(sim.stats.reward_modulated_updates_total > 0);
    assert_eq!(sim.creatures[id].energy, 1.0);
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 0.0);

    let (mut sim, id_floor) = make_sim_with_custom_genome(1.0, sink_learning_genome());
    let mut outcomes = OutcomeAccumulator::default();
    outcomes.snapshot_energy(id_floor, 1.0);
    sim.creatures[id_floor].energy = -1.5;
    super::super::run_reward_learning(&mut sim, &outcomes);
    assert_eq!(sim.creatures[id_floor].energy, 0.0);
    assert_eq!(sim.stats.energy_flows.zero_floor_credit, 1.5);
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
        let (mut sim, id) = make_sim_with_custom_genome(initial_energy, sized_genome(4));
        sim.config.energy.lifecycle.energy_decay_per_tick = 2.0;
        sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.5;
        assert_eq!(sim.creatures[id].genome_size(), 4);

        run_phase_0(&mut sim);

        assert!(!sim.creatures.contains_key(id));
        assert_eq!(sim.stats.mortality.deaths_total, 1);
        assert_eq!(sim.stats.mortality.count(cause), 1, "{initial_energy}");
    }
}

#[test]
fn phase_zero_allocates_combined_debit_and_counts_even_dead_creature_exposure() {
    let (mut sim, decay, carrying) =
        make_sim_two_with_genomes(1.0, 3.0, sized_genome(4), sized_genome(4));
    sim.config.energy.lifecycle.energy_decay_per_tick = 2.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.5;
    assert_eq!(sim.creatures[decay].genome_size(), 4);
    assert_eq!(sim.creatures[carrying].genome_size(), 4);
    run_phase_0(&mut sim);
    assert_eq!(sim.stats.mortality.count(DeathCause::LifecycleDecay), 1);
    assert_eq!(sim.stats.mortality.count(DeathCause::GenomeCarrying), 1);
    assert_eq!(sim.stats.energy_flows.lifecycle_decay, 4.0);
    assert_eq!(sim.stats.energy_flows.genome_carrying, 4.0);
    assert_eq!(sim.stats.energy_flows.genome_size_creature_ticks, 8);
}

#[test]
fn predation_leaves_an_exhausted_victim_its_own_cause_and_takes_nothing() {
    let (mut sim, attacker, victim) = make_sim_two_creatures(1.0, -2.0);
    sim.config.exchange.effort_rate = 0.0;
    sim.config.energy.costs.noop_cost = 0.0; // no T16.F02 effort floor
    sim.config.predation.kill_complexity_bonus_multiplier = 0.0;
    sim.creatures[victim].pending_death_cause = Some(DeathCause::VmCompute);
    steal_now(attacker, &mut sim, Direction::N, 10.0);
    assert_eq!(sim.stats.predation_kills_total, 0);
    assert_eq!(sim.stats.energy_flows.predation_victim_debit, 0.0);
    assert_eq!(sim.stats.energy_flows.predation_attacker_credit, 0.0);
    assert_eq!(sim.creatures[attacker].pending_death_cause, None);
    sim.remove_creature(victim);
    assert_eq!(sim.stats.mortality.count(DeathCause::VmCompute), 1);
    assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 0);
}

/// The per-creature compute total is the plain sum of all three cognition
/// cost components, including the T19.F01 hop ramp. The three values are
/// distinct powers of two, so the sum is exact in `f32` and differs from
/// every pairwise product or difference of the same terms.
#[test]
fn tick_compute_total_sums_vm_graph_and_mesh_ramp_costs() {
    // Arrange
    let mut output = decision(vec![WorldAction::NoOp]);
    output.cost_report = ComputeCostReport {
        vm_cost: 1.0,
        graph_cost: 2.0,
        mesh_ramp_cost: 4.0,
    };
    let mut compute = super::super::TickComputeStats::default();

    // Act
    compute.record(&output);

    // Assert
    assert_eq!(compute.total_sum, 7.0);
    assert_eq!(compute.total_min, 7.0);
    assert_eq!(compute.total_max, 7.0);
    assert_eq!(compute.vm_sum, 1.0);
    assert_eq!(compute.graph_sum, 2.0);
}

/// Pass and decided-pass counts (T19.F04) sum across creatures within a tick
/// and accumulate across ticks into the cumulative totals.
#[test]
fn tick_compute_passes_and_decided_passes_accumulate_across_creatures_and_ticks() {
    // Arrange
    let mut first = decision(vec![WorldAction::NoOp]);
    first.work_counters.passes = 3;
    first.work_counters.decided_passes = 2;
    let mut second = decision(vec![WorldAction::NoOp]);
    second.work_counters.passes = 4;
    second.work_counters.decided_passes = 1;
    let mut stats = crate::simulation::stats::SimStats::default();

    // Act
    for _tick in 0..2 {
        let mut compute = super::super::TickComputeStats::default();
        compute.record(&first);
        compute.record(&second);
        compute.commit(&mut stats);
    }

    // Assert
    assert_eq!(stats.passes_total, 14);
    assert_eq!(stats.decided_passes_total, 6);
}

/// T16.F02 invariant 5: `failed_action_effort.k` adds the very debit
/// `action_charges.k` records whenever the attempt failed.
#[test]
fn failed_action_effort_is_the_action_charge_of_each_failed_attempt() {
    let failing = [
        WorldAction::Move(Direction::N),
        WorldAction::eat(OrdinaryFoodTypeId::default()),
        WorldAction::eat(OrdinaryFoodTypeId::new(7)),
        WorldAction::Reproduce {
            direction: Direction::N,
            energy_transfer_fraction: 0.5,
        },
        WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 1.0,
        },
    ];
    for action in failing {
        // Arrange
        let (mut sim, id) = make_sim_with_one_creature(100.0);
        sim.world
            .set_barrier(crate::contracts::Position::new(5, 4), true);
        sim.config.energy.costs.eat_cost = 0.5;
        sim.config.energy.lifecycle.min_reproduce_age = 0;
        sim.config.exchange.effort_rate = 0.5;

        // Act
        execute(&mut sim, id, action);

        // Assert
        let charges = sim.stats.energy_flows.action_charges;
        assert_eq!(charges[ActionType::NoOp], 0.0, "{action:?}");
        let failed = sim.stats.energy_flows.failed_action_effort;
        for kind in ActionType::ALL {
            assert_eq!(failed[kind], charges[kind], "{action:?} {kind:?}");
        }
        assert!(charges.total() > 0.0, "{action:?} must have paid an effort");
        assert!(sim.creatures.contains_key(id), "{action:?}");
    }
}

#[test]
fn successful_attempts_add_no_failed_action_effort() {
    let (mut sim, id) = make_sim_with_one_creature(100.0);
    sim.config.energy.costs.eat_cost = 0.5;
    sim.world.set_food_type(
        crate::contracts::Position::new(5, 5),
        OrdinaryFoodTypeId::default(),
        1.0,
    );
    execute(
        &mut sim,
        id,
        WorldAction::eat(OrdinaryFoodTypeId::default()),
    );
    execute(&mut sim, id, WorldAction::NoOp);
    execute(&mut sim, id, WorldAction::Move(Direction::S));
    assert!(sim.stats.energy_flows.action_charges[ActionType::Eat] > 0.0);
    assert!(sim.stats.energy_flows.action_charges[ActionType::Move] > 0.0);
    assert_eq!(
        sim.stats.energy_flows.failed_action_effort,
        Default::default()
    );

    let (mut sim, attacker, _victim) = make_sim_two_creatures(100.0, 50.0);
    sim.config.exchange.effort_rate = 0.5;
    execute(
        &mut sim,
        attacker,
        WorldAction::StealEnergy {
            direction: Direction::N,
            effort: 1.0,
        },
    );
    assert!(sim.stats.energy_flows.action_charges[ActionType::StealEnergy] > 0.0);
    assert_eq!(
        sim.stats.energy_flows.failed_action_effort,
        Default::default()
    );
}

/// T16.F02 invariant 6: only Step 7's branch (a litter of zero or under
/// `initial_energy`) is a litter rejection, and it stays an
/// `EnergyConstraints` rejection like Step 6's.
#[test]
fn litter_rejections_are_counted_apart_from_other_rejections() {
    use crate::simulation::actions::ReproductionActionResult;
    let reproduce = |direction, energy_transfer_fraction| WorldAction::Reproduce {
        direction,
        energy_transfer_fraction,
    };
    // (parent energy, action, litter rejections expected)
    let cases = [
        (100.0, reproduce(Direction::S, 0.0), 1),
        (100.0, reproduce(Direction::S, 0.01), 1),
        (0.5, reproduce(Direction::S, 1.0), 0),
        (100.0, reproduce(Direction::N, 0.5), 0),
    ];
    for (energy, action, litter) in cases {
        let (mut sim, id) = make_sim_with_one_creature(energy);
        sim.world
            .set_barrier(crate::contracts::Position::new(5, 4), true);
        sim.config.energy.lifecycle.min_reproduce_age = 0;
        sim.config.energy.lifecycle.min_reproduce_energy = 1.0;
        execute(&mut sim, id, action);
        assert_eq!(
            sim.stats.reproduction_actions_rejected_total, 1,
            "{action:?}"
        );
        assert_eq!(
            sim.stats.reproduction_actions_rejected_litter_total, litter,
            "{action:?}"
        );
        if litter == 1 {
            assert_eq!(
                sim.stats.reproduction_actions_rejected_by_reason
                    [&ReproductionActionResult::RejectedEnergyConstraints],
                1
            );
        }
    }
}
