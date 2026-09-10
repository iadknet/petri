use crate::config::RuntimeConfig;
use crate::contracts::{DynamicIntrospectionKey, InputReference, NodeId};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
};
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute::execute_graph_node;
use crate::runtime::types::{MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

fn sensors() -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

fn config(rule: HebbianRule, eta: f32, lambda: f32) -> PlasticityConfig {
    PlasticityConfig {
        rule,
        learning_rate: eta,
        weight_clamp: 2.0,
        lamarckian: false,
        modulation: Some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: lambda,
        }),
    }
}

fn graph(rule: HebbianRule, eta: f32, lambda: f32) -> CgpGraphBackendDef {
    CgpGraphBackendDef {
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
            plasticity: Some(config(rule, eta, lambda)),
        }],
        output_sinks: vec![],
        action_bank: vec![],
        execute_gate: ExecuteGate { inputs: vec![] },
    }
}

fn genome(def: &CgpGraphBackendDef) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def.clone()),
            targets: vec![],
        }],
    }
}

fn begin(state: &mut GraphRuntimeState, def: &CgpGraphBackendDef) {
    state.begin_tick(&genome(def).nodes, 0);
}

fn visit(
    def: &CgpGraphBackendDef,
    state: &mut GraphRuntimeState,
    input: f32,
    energy: &mut f32,
    runtime: &RuntimeConfig,
) -> MeshSideOutputs {
    let mut memory = [0.0; 16];
    memory[0] = input;
    let mut side = MeshSideOutputs::new(10);
    let _ = execute_graph_node(
        def,
        &[InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::EnergyCurrent,
        )],
        &[0.0; OUTPUT_SLOT_COUNT],
        energy,
        0.0,
        0,
        state,
        &sensors(),
        runtime,
        &mut side,
        &mut memory,
        &[0.0; 16],
    );
    side
}

#[test]
fn repeated_visits_replace_activity_from_decayed_base() {
    let def = graph(HebbianRule::Classic, 1.0, 0.5);
    let mut state = GraphRuntimeState::new();
    let runtime = RuntimeConfig::default();
    begin(&mut state, &def);
    visit(&def, &mut state, 2.0, &mut 100.0, &runtime);
    assert_eq!(state.eligibility_traces[0][0][0], 2.0);
    begin(&mut state, &def);
    assert_eq!(state.eligibility_traces[0][0][0], 1.0);
    for _ in 0..3 {
        visit(&def, &mut state, 3.0, &mut 100.0, &runtime);
        assert_eq!(state.eligibility_traces[0][0][0], 4.0);
    }
    visit(&def, &mut state, -2.0, &mut 100.0, &runtime);
    assert_eq!(state.eligibility_traces[0][0][0], -1.0);
}

#[test]
fn recurrent_activity_uses_frozen_self_and_higher_but_current_lower_sources() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.0);
    def.compute_nodes[0].inputs = vec![
        GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 0.5,
        },
        GraphEdge {
            source: GraphSource::ComputeNode(1),
            weight: 0.5,
        },
    ];
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(3.0),
        inputs: vec![GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 0.5,
        }],
        plasticity: Some(config(HebbianRule::Classic, 1.0, 0.0)),
    });
    let mut state = GraphRuntimeState::new();
    state.node_outputs = vec![vec![2.0, 4.0]];
    begin(&mut state, &def);
    visit(&def, &mut state, 0.0, &mut 100.0, &RuntimeConfig::default());
    assert_eq!(&*state.eligibility_traces[0][0], &[2.0, 4.0]);
    assert_eq!(&*state.eligibility_traces[0][1], &[3.0]);
}

#[test]
fn activity_uses_evaluation_energy_before_pure_hebbian_cost() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.0);
    def.compute_nodes[0].inputs[0].source = GraphSource::InputLeaf {
        ref_idx: 0,
        sub_idx: 0,
    };
    let mut pure = def.compute_nodes[0].clone();
    pure.plasticity.as_mut().unwrap().modulation = None;
    def.compute_nodes.push(pure);
    let runtime = RuntimeConfig {
        graph_node_base_cost: 1.0,
        plasticity_update_cost: 2.0,
        ..RuntimeConfig::default()
    };
    let mut state = GraphRuntimeState::new();
    begin(&mut state, &def);
    let mut energy = 10.0;
    let side = visit(&def, &mut state, 0.0, &mut energy, &runtime);
    assert_eq!(energy, 6.0);
    assert_eq!(side.work_counters.plasticity_updates, 1);
    assert_eq!(state.eligibility_traces[0][0][0], 8.0);
}

#[test]
fn failed_visits_preserve_decay_and_last_successful_activity() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.5);
    let mut pure = def.compute_nodes[0].clone();
    pure.plasticity.as_mut().unwrap().modulation = None;
    def.compute_nodes.push(pure);
    let runtime = RuntimeConfig {
        graph_node_base_cost: 1.0,
        plasticity_update_cost: 1.0,
        ..RuntimeConfig::default()
    };
    let mut state = GraphRuntimeState::new();
    begin(&mut state, &def);
    visit(&def, &mut state, 2.0, &mut 100.0, &runtime);
    begin(&mut state, &def);
    for energy in [1.0, 2.5] {
        visit(&def, &mut state, 9.0, &mut { energy }, &runtime);
        assert_eq!(state.eligibility_traces[0][0][0], 1.0);
    }
    visit(&def, &mut state, 3.0, &mut 100.0, &runtime);
    for energy in [1.0, 2.5] {
        visit(&def, &mut state, 9.0, &mut { energy }, &runtime);
        assert_eq!(state.eligibility_traces[0][0][0], 4.0);
    }
}

fn reward(def: &CgpGraphBackendDef, state: &mut GraphRuntimeState, signal: f32) -> (f32, u32) {
    let mut signals = super::OutcomeSignalBank::default();
    signals.signals[OutcomeChannel::EnergyDelta as usize] = signal;
    super::reward::apply_reward_modulated_updates(
        def,
        0,
        &mut state.plasticity_weights,
        &state.eligibility_traces,
        &signals,
        0.25,
    )
}

#[test]
fn exact_rules_rewards_rates_and_hebbian_equivalence() {
    let pre = 0.75;
    let post = -0.25;
    let weight = 0.5;
    for (rule, activity) in [
        (HebbianRule::Classic, pre * post),
        (HebbianRule::Oja, post * (pre - weight * post)),
        (HebbianRule::AntiHebb, -pre * post),
        (HebbianRule::Covariance, (pre - 0.5) * (post - 0.5)),
    ] {
        for eta in [0.0, 0.4, 1.0] {
            for signal in [-0.75, 0.0, 1.0] {
                let mut def = graph(rule, eta, 0.0);
                def.compute_nodes[0].kind = ComputeNodeKind::Constant(post);
                let mut state = GraphRuntimeState::new();
                begin(&mut state, &def);
                visit(&def, &mut state, pre, &mut 100.0, &RuntimeConfig::default());
                assert!((state.eligibility_traces[0][0][0] - activity).abs() < 1e-7);
                assert_eq!(reward(&def, &mut state, signal), (0.25, 1));
                let expected = weight + eta * signal * activity;
                assert!((state.plasticity_weights[0][0][0] - expected).abs() < 1e-7);
                if signal == 1.0 {
                    let mut pure = def.clone();
                    pure.compute_nodes[0]
                        .plasticity
                        .as_mut()
                        .unwrap()
                        .modulation = None;
                    let mut pure_state = GraphRuntimeState::new();
                    begin(&mut pure_state, &pure);
                    visit(
                        &pure,
                        &mut pure_state,
                        pre,
                        &mut 100.0,
                        &RuntimeConfig::default(),
                    );
                    assert_eq!(pure_state.plasticity_weights, state.plasticity_weights);
                }
            }
        }
    }
}

#[test]
fn isolated_pulse_discounts_by_elapsed_ticks_including_zero_activity_visits() {
    for lambda in [0.0f32, 0.5, 1.0] {
        for delay in [1, 2, 4, 8, 16] {
            let def = graph(HebbianRule::Classic, 0.4, lambda);
            let mut skipped = GraphRuntimeState::new();
            begin(&mut skipped, &def);
            visit(
                &def,
                &mut skipped,
                0.75,
                &mut 100.0,
                &RuntimeConfig::default(),
            );
            let mut zero = skipped.clone();
            for _ in 0..delay {
                begin(&mut skipped, &def);
                begin(&mut zero, &def);
                visit(&def, &mut zero, 0.0, &mut 100.0, &RuntimeConfig::default());
            }
            let expected = lambda.powi(delay) * 0.75;
            assert_eq!(skipped.eligibility_traces[0][0][0], expected);
            assert_eq!(skipped.eligibility_traces, zero.eligibility_traces);
            reward(&def, &mut skipped, -0.75);
            assert!(
                (skipped.plasticity_weights[0][0][0] - (0.5 - 0.4 * 0.75 * expected)).abs() < 1e-7
            );
        }
    }
}

#[test]
fn clock_does_not_initialize_unvisited_modules_or_charge_reward_work() {
    let def = graph(HebbianRule::Classic, 1.0, 0.5);
    let mut state = GraphRuntimeState::new();
    for _ in 0..4 {
        begin(&mut state, &def);
    }
    assert!(state.eligibility_traces.is_empty());
    assert!(state.plasticity_weights.is_empty());
    assert_eq!(reward(&def, &mut state, 1.0), (0.0, 0));
    // Failed first evaluation initializes zeros but never activity.
    visit(&def, &mut state, 3.0, &mut 0.0, &RuntimeConfig::default());
    assert_eq!(state.eligibility_traces[0][0][0], 0.0);
    assert_eq!(reward(&def, &mut state, 1.0), (0.25, 1));
    assert_eq!(state.plasticity_weights[0][0][0], 0.5);
}

#[test]
fn first_tick_repeated_visits_have_zero_base_and_ordinary_traced_parity() {
    let mut def = graph(HebbianRule::Oja, 0.4, 0.7);
    def.action_bank
        .push(crate::creature::genome::cgp::ActionSlot {
            behavior: crate::creature::genome::cgp::ActionSlotBehavior::Emit(
                crate::creature::genome::cgp::WorldActionKind::Eat,
            ),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: vec![],
        });
    let runtime = RuntimeConfig::default();
    let mut ordinary = GraphRuntimeState::new();
    let mut traced = ordinary.clone();
    for _ in 0..3 {
        begin(&mut ordinary, &def);
        begin(&mut traced, &def);
        for input in [0.75, 0.75, -0.5] {
            let mut energy = 100.0;
            let side = visit(&def, &mut ordinary, input, &mut energy, &runtime);
            let mut traced_energy = 100.0;
            let mut memory = [0.0; 16];
            memory[0] = input;
            let mut traced_side = MeshSideOutputs::new(10);
            let (_, trace) = crate::runtime::cgp::traced::execute_graph_node_traced(
                &def,
                &[InputReference::DynamicIntrospection(
                    DynamicIntrospectionKey::EnergyCurrent,
                )],
                &[0.0; OUTPUT_SLOT_COUNT],
                &mut traced_energy,
                0.0,
                0,
                &mut traced,
                &sensors(),
                &runtime,
                &mut traced_side,
                &mut memory,
                &[0.0; 16],
            );
            assert!(trace.temporal_committed);
            assert_eq!(ordinary.eligibility_traces, traced.eligibility_traces);
            assert_eq!(ordinary.plasticity_weights, traced.plasticity_weights);
            assert_eq!(ordinary.node_outputs, traced.node_outputs);
            assert_eq!(energy, traced_energy);
            assert_eq!(side.work_counters, traced_side.work_counters);
            assert_eq!(
                side.action_queue.into_actions(),
                traced_side.action_queue.into_actions()
            );
        }
        assert_eq!(
            reward(&def, &mut ordinary, 0.5),
            reward(&def, &mut traced, 0.5)
        );
        assert_eq!(ordinary.plasticity_weights, traced.plasticity_weights);
    }
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn skipped_tick_decay_is_exponential(lambda in 0.0f32..=1.0, pulse in -2.0f32..2.0, ticks in 0i32..24) {
        let def = graph(HebbianRule::Classic, 0.5, lambda);
        let mut state = GraphRuntimeState::new(); begin(&mut state, &def);
        visit(&def, &mut state, pulse, &mut 100.0, &RuntimeConfig::default());
        for _ in 0..ticks { begin(&mut state, &def); }
        let expected = lambda.powi(ticks) * pulse;
        prop_assert!((state.eligibility_traces[0][0][0] - expected).abs() < 2e-6);
    }

    #[test]
    fn identical_visit_count_and_disconnected_computation_do_not_change_credit(
        input in -2.0f32..2.0, lambda in 0.0f32..=1.0, visits in 1usize..8, extras in 0usize..8,
    ) {
        let def = graph(HebbianRule::Classic, 0.5, lambda);
        let mut bigger = def.clone();
        bigger.compute_nodes.extend((0..extras).map(|_| ComputeNode { kind: ComputeNodeKind::Oscillator(0.25), inputs: vec![], plasticity: None }));
        let mut once = GraphRuntimeState::new(); let mut repeated = GraphRuntimeState::new();
        let runtime = RuntimeConfig::default();
        let other_runtime = RuntimeConfig { max_graph_relax_iters: 1, graph_convergence_stable_passes: 99, ..runtime.clone() };
        for _ in 0..3 {
            begin(&mut once, &def); begin(&mut repeated, &bigger);
            visit(&def, &mut once, input, &mut 100.0, &runtime);
            for _ in 0..visits { visit(&bigger, &mut repeated, input, &mut 100.0, &other_runtime); }
            prop_assert_eq!(once.eligibility_traces[0][0][0], repeated.eligibility_traces[0][0][0]);
        }
    }

    #[test]
    fn normalized_reward_update_obeys_exact_clamp(
        weight in -10.0f32..10.0, pulse in -20.0f32..20.0, signal in -20.0f32..20.0,
        eta in -1.0f32..2.0, clamp in -1.0f32..12.0,
    ) {
        let mut def = graph(HebbianRule::Classic, eta, 0.0);
        def.compute_nodes[0].inputs[0].weight = weight;
        def.compute_nodes[0].plasticity.as_mut().unwrap().weight_clamp = clamp;
        let mut state = GraphRuntimeState::new(); begin(&mut state, &def);
        visit(&def, &mut state, pulse, &mut 100.0, &RuntimeConfig::default());
        reward(&def, &mut state, signal);
        let bound = clamp.clamp(0.01, 10.0);
        let actual = state.plasticity_weights[0][0][0];
        prop_assert_eq!(actual, (weight + eta.clamp(0.0, 1.0) * signal * pulse).clamp(-bound, bound));
        prop_assert!(actual >= -bound && actual <= bound);
    }
}

/// Authored action-contingent outcomes are a test treatment, never a production sensor.
#[test]
fn constructed_controller_adapts_to_reversal_while_frozen_weights_do_not() {
    use crate::contracts::WorldAction;
    use crate::creature::genome::cgp::{ActionSlot, ActionSlotBehavior, WorldActionKind};
    use crate::runtime::mesh::execute_creature_mesh;
    let started = std::time::Instant::now();
    let mut def = graph(HebbianRule::Classic, 0.5, 0.0);
    def.compute_nodes[0].kind = ComputeNodeKind::Constant(1.0);
    def.compute_nodes[0].inputs.clear();
    def.compute_nodes[0].plasticity = None;
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Threshold(0.0),
        inputs: vec![GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 0.5,
        }],
        plasticity: Some(config(HebbianRule::Classic, 0.5, 0.0)),
    });
    let edge = |index, weight| GraphEdge {
        source: GraphSource::ComputeNode(index),
        weight,
    };
    def.action_bank = vec![
        ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
            gate_inputs: vec![edge(1, 1.0)],
            param_inputs: vec![],
        },
        ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::NoOp),
            gate_inputs: vec![edge(0, 1.0), edge(1, -1.0)],
            param_inputs: vec![],
        },
    ];
    def.execute_gate.inputs = vec![edge(0, 1.0)];
    let genome = genome(&def);
    let run = |state: &mut GraphRuntimeState, acquisition: bool, frozen: bool| {
        let mut correct = Vec::new();
        let mut weights = Vec::new();
        let mut counts = [0usize; 2];
        for tick in 0..32 {
            state.begin_tick(&genome.nodes, tick);
            let result = execute_creature_mesh(
                &genome,
                &sensors(),
                &mut 100.0,
                &mut [0.0; 16],
                &[0.0; 16],
                state,
                &RuntimeConfig::default(),
            );
            assert_eq!(result.actions.len(), 1);
            let first = matches!(result.actions[0], WorldAction::Eat { .. });
            assert!(first || matches!(result.actions[0], WorldAction::NoOp));
            counts[usize::from(!first)] += 1;
            let success = first == acquisition;
            correct.push(success);
            if !frozen {
                reward(&def, state, if success { 1.0 } else { -1.0 });
            }
            weights.push(state.plasticity_weights[0][1][0]);
        }
        (correct, weights, counts)
    };
    let mut live = GraphRuntimeState::new();
    let acquisition = run(&mut live, true, false);
    assert_ne!(live.plasticity_weights[0][1][0], 0.5);
    assert!(acquisition.0[24..].iter().filter(|&&x| x).count() >= 7);
    let mut frozen = live.clone();
    let acquired_weights = frozen.plasticity_weights.clone();
    let reversal = run(&mut live, false, false);
    let control = run(&mut frozen, false, true);
    assert!(reversal.0[24..].iter().filter(|&&x| x).count() >= 7);
    assert!(control.0[24..].iter().filter(|&&x| x).count() <= 1);
    assert_eq!(frozen.plasticity_weights, acquired_weights);
    eprintln!("reversal acquisition counts {:?}, weights {:?}; live counts {:?}, weights {:?}; frozen counts {:?}, weights {:?}", acquisition.2, acquisition.1, reversal.2, reversal.1, control.2, control.1);
    assert!(started.elapsed().as_secs_f32() < 60.0);
}

#[test]
fn explicit_reward_clamp_bounds_and_decay_normalization() {
    for (lambda, expected) in [(-1.0, 0.0), (2.0, 3.0)] {
        let def = graph(HebbianRule::Classic, 1.0, lambda);
        let mut state = GraphRuntimeState::new();
        begin(&mut state, &def);
        visit(&def, &mut state, 3.0, &mut 100.0, &RuntimeConfig::default());
        begin(&mut state, &def);
        assert_eq!(state.eligibility_traces[0][0][0], expected);
    }
    for (signal, expected) in [(-10.0, -2.0), (10.0, 2.0)] {
        let def = graph(HebbianRule::Classic, 1.0, 0.0);
        let mut state = GraphRuntimeState::new();
        begin(&mut state, &def);
        visit(&def, &mut state, 3.0, &mut 100.0, &RuntimeConfig::default());
        reward(&def, &mut state, signal);
        assert_eq!(state.plasticity_weights[0][0][0], expected);
    }
}

#[test]
fn birth_aligned_weights_drive_first_execution_and_reward_without_parent_credit() {
    use crate::simulation::actions::cgp_reproduction::{
        build_cgp_child_plasticity_weights, capture_birth_weights,
    };
    let mut def = graph(HebbianRule::Classic, 0.1, 0.5);
    def.compute_nodes[0].kind = ComputeNodeKind::Add;
    def.compute_nodes[0].plasticity.as_mut().unwrap().lamarckian = true;
    capture_birth_weights(&mut def, &[Box::new([0.75])]);
    def.duplicate_compute_nodes_in_place(&[0]);
    def.remove_compute_node_at(0);
    let mut child = GraphRuntimeState::new();
    child.plasticity_weights = vec![build_cgp_child_plasticity_weights(&mut def)];
    assert!(child.eligibility_traces.is_empty());
    assert_eq!(reward(&def, &mut child, 1.0), (0.0, 0));
    begin(&mut child, &def);
    visit(&def, &mut child, 2.0, &mut 100.0, &RuntimeConfig::default());
    assert_eq!(child.node_outputs[0][0], 1.5);
    assert_eq!(child.eligibility_traces[0][0][0], 3.0);
    reward(&def, &mut child, 1.0);
    assert!((child.plasticity_weights[0][0][0] - 1.05).abs() < 1e-6);
}
