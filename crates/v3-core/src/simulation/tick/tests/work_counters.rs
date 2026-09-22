use super::super::run_tick;
use super::support::*;
use crate::contracts::NodeId;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
};
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig, VmInstruction,
};

/// A single creature running a known 3-opcode program (Noop, PushAction,
/// ExecuteActionQueue) for one tick exercises exactly one mesh hop, three VM
/// steps, no graph work, one creature-tick, and one applied action.
#[test]
fn run_tick_accumulates_known_vm_work_counters_for_one_creature() {
    let genome = vm_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::PushAction { action_type: 0 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, genome);

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.mesh_hops_total, 1);
    assert_eq!(sim.stats.vm_steps_total, 3);
    assert_eq!(sim.stats.graph_relax_iters_total, 0);
    assert_eq!(sim.stats.plasticity_updates_total, 0);
    assert_eq!(sim.stats.creature_ticks_total, 1);
    assert_eq!(sim.stats.actions_applied_total, 1);
    assert_eq!(sim.stats.last_tick_noop, 1);
}

/// Work counters accumulate cumulatively across ticks rather than resetting.
#[test]
fn run_tick_work_counters_accumulate_across_ticks() {
    let genome = vm_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::PushAction { action_type: 0 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(1000.0, genome);

    run_tick(&mut sim, &mut None);
    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.mesh_hops_total, 2);
    assert_eq!(sim.stats.vm_steps_total, 6);
    assert_eq!(sim.stats.creature_ticks_total, 2);
    assert_eq!(sim.stats.actions_applied_total, 2);
}

/// `creature_ticks_total` and `actions_applied_total` sum across every
/// founder in a multi-creature population, not just the first.
#[test]
fn run_tick_sums_creature_ticks_and_actions_across_population() {
    let mut sim = seed_simulation_default();
    let population = sim.creatures.len() as u64;

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.creature_ticks_total, population);
    assert_eq!(
        sim.stats.actions_applied_total,
        u64::from(
            sim.stats.last_tick_move
                + sim.stats.last_tick_eat
                + sim.stats.last_tick_noop
                + sim.stats.last_tick_reproduce
                + sim.stats.last_tick_steal
        )
    );
}

fn seed_simulation_default() -> crate::simulation::Simulation {
    crate::simulation::seeding::seed_simulation(small_config(), 7)
}

/// A single creature whose genome has one CGP graph node with a
/// reward-modulated (three-factor) plastic edge accumulates exactly one
/// `plasticity_updates_total` from the Phase 2.5 reward-learning pass — the
/// path that does not flow through `MeshOutput.work_counters` (see the
/// T10.F10 spec's Notes for AI Agents on the Hebbian/reward-modulated
/// split). The node has no `modulation: None` sibling, so Phase 1's Hebbian
/// pass contributes zero, isolating the count to the reward-modulated path.
#[test]
fn run_tick_accumulates_reward_modulated_plasticity_update_in_phase_2_5() {
    let node_id = NodeId::new(0);
    let genome = CreatureGenome {
        entry_node_id: node_id,
        nodes: vec![NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Constant(1.0),
                    inputs: vec![GraphEdge {
                        source: GraphSource::SharedMemory {
                            slot: 0,
                            previous: false,
                        },
                        weight: 1.0,
                    }],
                    plasticity: Some(PlasticityConfig {
                        rule: HebbianRule::Classic,
                        learning_rate: 0.5,
                        weight_clamp: 1.0,
                        lamarckian: false,
                        modulation: Some(RewardModulationConfig {
                            reward_source: OutcomeChannel::EnergyDelta,
                            trace_decay: 0.9,
                        }),
                    }),
                }],
                output_sinks: vec![],
                action_bank: vec![],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
            targets: vec![],
        }],
    };

    let (mut sim, _id) = make_sim_with_custom_genome(100.0, genome);

    run_tick(&mut sim, &mut None);

    assert_eq!(
        sim.stats.plasticity_updates_total, 1,
        "one reward-modulated edge must contribute exactly one plasticity update, \
         entirely from the Phase 2.5 call site (Phase 1's Hebbian pass skips \
         modulated nodes)"
    );
    // graph_relax_iters_total is nonzero (Phase 1 ran the single-node graph),
    // confirming this is exercising the graph path and not a no-op mesh.
    assert!(sim.stats.graph_relax_iters_total > 0);
}

/// A dispatch that cannot pay for its first opcode stops with
/// `TerminationReason::EnergyExhausted`, and the tick's queue-order reduction
/// counts that termination and no other.
#[test]
fn run_tick_counts_only_dispatches_that_ran_out_of_energy() {
    let genome = vm_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::PushAction { action_type: 0 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, id) = make_sim_with_custom_genome(100.0, genome);
    sim.config.energy.lifecycle.energy_decay_per_tick = 0.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;

    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats.mesh_dispatches_energy_exhausted_total, 0,
        "a dispatch that emitted its action is not an exhausted one"
    );

    // Alive, but with less energy than the first opcode's charge.
    sim.creatures[id].energy = 1e-9;
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats.mesh_dispatches_energy_exhausted_total, 1,
        "the dispatch that ran out of energy is counted once"
    );
}

/// A pass that reaches `max_mesh_hops` is counted once per creature-tick in
/// `pass_cap_hits_total` (T19.F02); a chain that ends before the cap adds nothing.
#[test]
fn run_tick_counts_passes_that_reach_the_hop_cap() {
    let mut genome = vm_program_genome(vec![VmInstruction::Halt]);
    genome.nodes.push(NodeGenome {
        node_id: NodeId::new(1),
        ..genome.nodes[0].clone()
    });
    genome.nodes[0].targets = vec![crate::contracts::RouteTarget {
        target_id: NodeId::new(1),
        slot: 0,
        gate_bias: 0.0,
    }];
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, genome);
    sim.config.runtime.max_mesh_hops = 1;

    run_tick(&mut sim, &mut None);
    assert_eq!(sim.stats.pass_cap_hits_total, 1);
    assert_eq!(sim.stats.mesh_hops_total, 1);

    sim.config.runtime.max_mesh_hops = 2;
    run_tick(&mut sim, &mut None);
    assert_eq!(
        sim.stats.pass_cap_hits_total, 1,
        "a chain that ends at its last target adds no cap hit"
    );
    assert_eq!(sim.stats.mesh_hops_total, 3);
}

fn cognition_totals(stats: &crate::simulation::stats::SimStats) -> [u64; 7] {
    [
        stats.plasticity_updates_total,
        stats.plasticity_changes_total,
        stats.hebbian_updates_total,
        stats.hebbian_changes_total,
        stats.reward_modulated_updates_total,
        stats.reward_modulated_changes_total,
        stats.shared_memory_writes_changed_total,
    ]
}

fn mixed_learning_genome(hebbian_edges: usize, reward_edges: usize) -> CreatureGenome {
    use crate::creature::genome::cgp::{OutputSink, OutputSinkKind};
    let edge = GraphEdge {
        source: GraphSource::ComputeNode(0),
        weight: 0.5,
    };
    let node = |edges, modulated: bool| ComputeNode {
        kind: ComputeNodeKind::Constant(1.0),
        inputs: vec![edge; edges],
        plasticity: Some(PlasticityConfig {
            rule: HebbianRule::Classic,
            learning_rate: 0.5,
            weight_clamp: 2.0,
            lamarckian: false,
            modulation: modulated.then_some(RewardModulationConfig {
                reward_source: OutcomeChannel::ActionSuccess,
                trace_decay: 0.0,
            }),
        }),
    };
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            targets: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![
                    ComputeNode {
                        kind: ComputeNodeKind::Constant(1.0),
                        inputs: vec![],
                        plasticity: None,
                    },
                    node(hebbian_edges, false),
                    node(reward_edges, true),
                ],
                output_sinks: vec![
                    OutputSink {
                        kind: OutputSinkKind::WriteSlot(0),
                        inputs: vec![edge],
                    },
                    OutputSink {
                        kind: OutputSinkKind::ClearSlot(0),
                        inputs: vec![edge],
                    },
                ],
                action_bank: vec![],
                execute_gate: ExecuteGate { inputs: vec![] },
            }),
        }],
    }
}

#[test]
fn production_learning_splits_accumulate_and_observations_leave_them_unchanged() {
    use crate::runtime::trace::recording::ActiveTrace;
    for traced in [false, true] {
        let (mut sim, id) = make_sim_with_custom_genome(1000.0, mixed_learning_genome(1, 2));
        let mut trace = traced.then(|| ActiveTrace::new(id, 4));
        for tick in 1..=4 {
            run_tick(&mut sim, &mut trace);
            // Each path reaches weight 2 on tick 3; tick 4 still assigns and costs
            // work, but every proposed increment clamps away.
            let changed_ticks = tick.min(3);
            assert_eq!(
                cognition_totals(&sim.stats),
                [
                    tick * 3,
                    changed_ticks * 3,
                    tick,
                    changed_ticks,
                    tick * 2,
                    changed_ticks * 2,
                    tick * 2,
                ]
            );
            let before = cognition_totals(&sim.stats);
            let weights = sim.creatures[id].graph_runtime.plasticity_weights.clone();
            let memory = sim.creatures[id].shared_memory;
            crate::simulation::observe_final_actions(&sim);
            crate::simulation::observe_temporal_actions(&sim);
            assert_eq!(cognition_totals(&sim.stats), before);
            assert_eq!(sim.creatures[id].graph_runtime.plasticity_weights, weights);
            assert_eq!(sim.creatures[id].shared_memory, memory);
        }
    }
}

#[test]
fn production_memory_events_accumulate_without_counting_decay_or_snapshots() {
    for traced in [false, true] {
        let genome = vm_program_genome(vec![
            VmInstruction::LoadSlotPrev {
                dst: 0,
                slot_idx: 1,
            },
            VmInstruction::StoreSlotImm {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::ClearSlot { slot_idx: 0 },
        ]);
        let (mut sim, id) = make_sim_with_custom_genome(1000.0, genome);
        sim.creatures[id].shared_memory[1] = 1.0;
        sim.config.shared_memory.decay_rate = 0.1;
        let mut trace = traced.then(|| crate::runtime::trace::recording::ActiveTrace::new(id, 2));
        for tick in 1..=2 {
            run_tick(&mut sim, &mut trace);
            assert_eq!(sim.stats.shared_memory_writes_changed_total, tick * 2);
        }
        assert!(sim.creatures[id].shared_memory[1] < 1.0);
    }
    let (mut sim, id) =
        make_sim_with_custom_genome(1000.0, vm_program_genome(vec![VmInstruction::Halt]));
    sim.creatures[id].shared_memory = [1.0; 16];
    sim.config.shared_memory.decay_rate = 0.1;
    run_tick(&mut sim, &mut None);
    assert!(sim.creatures[id].shared_memory[0] < 1.0);
    assert_eq!(sim.stats.shared_memory_writes_changed_total, 0);
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn production_learning_pathways_partition_assignments_and_changes(
        hebbian_edges in 0usize..8, reward_edges in 0usize..8, ticks in 1u64..7,
    ) {
        let (mut sim, _) = make_sim_with_custom_genome(1000.0, mixed_learning_genome(hebbian_edges, reward_edges));
        for _ in 0..ticks { run_tick(&mut sim, &mut None); }
        let stats = &sim.stats;
        prop_assert_eq!(stats.hebbian_updates_total, ticks * hebbian_edges as u64);
        prop_assert_eq!(stats.reward_modulated_updates_total, ticks * reward_edges as u64);
        prop_assert_eq!(stats.plasticity_updates_total, stats.hebbian_updates_total + stats.reward_modulated_updates_total);
        prop_assert_eq!(stats.plasticity_changes_total, stats.hebbian_changes_total + stats.reward_modulated_changes_total);
        prop_assert_eq!(stats.hebbian_changes_total, ticks.min(3) * hebbian_edges as u64);
        prop_assert_eq!(stats.reward_modulated_changes_total, ticks.min(3) * reward_edges as u64);
        prop_assert!(stats.hebbian_changes_total <= stats.hebbian_updates_total);
        prop_assert!(stats.reward_modulated_changes_total <= stats.reward_modulated_updates_total);
    }
}
