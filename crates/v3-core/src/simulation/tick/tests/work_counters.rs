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
