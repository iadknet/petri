use crate::config::RuntimeConfig;
use crate::contracts::{DynamicIntrospectionKey, InputReference, NodeId};
use crate::creature::genome::cgp::test_support::{
    plastic_node, push_sink, set_compute_inputs, set_surface_plasticity,
};
use crate::creature::genome::cgp::{
    validate_graph, BirthSeeds, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence,
    EdgeSurface, GraphEdge, GraphInvariantViolation, GraphSource, OutputSink, OutputSinkKind,
};
use crate::creature::genome::vote::VoteSink;
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
        world_reads: Default::default(),
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: 0.0,
            previous_outcome: [0.0; 4],
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

/// Hebbian learning is free (T25.F02): a plastic visit pays its pass cost
/// alone, and the pass cost is the only exhaustion it can cause.
#[test]
fn applied_graph_accounting_charges_only_the_pass_under_hebbian_learning() {
    use crate::simulation::energy_accounting::DeathCause;
    let mut def = graph(HebbianRule::Classic, 1.0, 0.5);
    edit_config(&mut def, 0, |config| config.modulation = None);
    let runtime = RuntimeConfig {
        graph_node_base_cost: 2.0,
        ..RuntimeConfig::default()
    };
    for (initial, cause, updates) in [(1.0, Some(DeathCause::GraphCompute), 0), (4.0, None, 1)] {
        let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
        begin(&mut state);
        let mut energy = initial;
        let side = visit(&def, &mut state, 1.0, &mut energy, &runtime);
        assert_eq!(side.energy_observation.graph_compute, 2.0);
        assert_eq!(side.energy_observation.pending_cause, cause);
        assert_eq!(side.work_counters.plasticity_updates, updates);
        assert_eq!(energy, initial - 2.0);
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
    CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![plastic_node(
            ComputeNodeKind::Constant(1.0),
            vec![GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                0.5,
            )],
            Some(config(rule, eta, lambda)),
        )])
        .with_output_sinks(vec![])
}

/// Rewrite the learning attributes of every input edge of compute node
/// `node` with `edit` (the former per-node config, now per edge).
fn edit_config(def: &mut CgpGraphBackendDef, node: usize, edit: impl Fn(&mut PlasticityConfig)) {
    let surface = EdgeSurface::ComputeInput(node);
    for index in 0..def.surface_edges(surface).len() {
        let occurrence = EdgeOccurrence::new(surface, index);
        let mut config = def
            .edge_at(occurrence)
            .plasticity()
            .expect("a plastic edge");
        edit(&mut config);
        def.edge_mut(occurrence).set_plasticity(Some(config));
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

fn begin(state: &mut GraphRuntimeState) {
    state.begin_tick(0);
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
    visit_with_memory(def, state, &mut memory, energy, runtime)
}

fn visit_with_memory(
    def: &CgpGraphBackendDef,
    state: &mut GraphRuntimeState,
    memory: &mut [f32; 16],
    energy: &mut f32,
    runtime: &RuntimeConfig,
) -> MeshSideOutputs {
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
        memory,
        &[0.0; 16],
    );
    side
}

/// Slot `slot`'s trace (mesh node 0), `None` while uninitialized.
fn trace_at(state: &GraphRuntimeState, slot: usize) -> Option<f32> {
    state
        .learned()
        .edges(0)
        .nth(slot)
        .and_then(|edge| edge.trace)
}

/// Slot 0's trace; the slot is initialized.
fn trace(state: &GraphRuntimeState) -> f32 {
    trace_at(state, 0).expect("an initialized reward-modulated slot")
}

/// Slot `slot`'s learned weight (mesh node 0), `None` while uninitialized.
fn learned_at(state: &GraphRuntimeState, slot: usize) -> Option<f32> {
    state
        .learned()
        .edges(0)
        .nth(slot)
        .and_then(|edge| edge.learned)
}

/// Slot 0's learned weight; the slot is initialized.
fn weight(state: &GraphRuntimeState) -> f32 {
    learned_at(state, 0).expect("an initialized slot")
}

/// The reward pass's edge assignments and changes for one signal.
fn reward_counts(state: &mut GraphRuntimeState, signal: f32) -> (u32, u32) {
    let mut signals = super::OutcomeSignalBank::default();
    signals.signals[OutcomeChannel::EnergyDelta as usize] = signal;
    let counts = state.learned.apply_reward(&signals);
    (counts.updates, counts.changes)
}

/// The reward pass's edge assignments for one signal.
fn reward(state: &mut GraphRuntimeState, signal: f32) -> u32 {
    reward_counts(state, signal).0
}

/// Every successful visit adds its activity (T19.F02); decay runs once per
/// world tick in `begin_tick`, so three visits of 3.0 on a decayed base of
/// 1.0 leave 10.0, and a fourth visit of -2.0 leaves 8.0.
#[test]
fn repeated_visits_add_activity_on_the_decayed_base() {
    let def = graph(HebbianRule::Classic, 1.0, 0.5);
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    let runtime = RuntimeConfig::default();
    begin(&mut state);
    visit(&def, &mut state, 2.0, &mut 100.0, &runtime);
    assert_eq!(trace(&state), 2.0);
    begin(&mut state);
    assert_eq!(trace(&state), 1.0);
    for expected in [4.0, 7.0, 10.0] {
        visit(&def, &mut state, 3.0, &mut 100.0, &runtime);
        assert_eq!(trace(&state), expected);
    }
    visit(&def, &mut state, -2.0, &mut 100.0, &runtime);
    assert_eq!(trace(&state), 8.0);
}

#[test]
fn recurrent_activity_uses_committed_self_and_higher_but_current_lower_sources() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.0);
    set_compute_inputs(
        &mut def,
        0,
        vec![
            GraphEdge::new(GraphSource::ComputeNode(0), 0.5),
            GraphEdge::new(GraphSource::ComputeNode(1), 0.5),
        ],
    );
    set_surface_plasticity(
        &mut def,
        EdgeSurface::ComputeInput(0),
        Some(config(HebbianRule::Classic, 1.0, 0.0)),
    );
    def.append_compute_node(plastic_node(
        ComputeNodeKind::Constant(3.0),
        vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
        Some(config(HebbianRule::Classic, 1.0, 0.0)),
    ));
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    state.node_outputs = vec![vec![2.0, 4.0]];
    begin(&mut state);
    visit(&def, &mut state, 0.0, &mut 100.0, &RuntimeConfig::default());
    let traces: Vec<_> = (0..3).map(|slot| trace_at(&state, slot)).collect();
    assert_eq!(traces, vec![Some(2.0), Some(4.0), Some(3.0)]);
}

/// The self-edge regression (T25.F06 invariant 6, predeclared): a pure
/// Hebbian self-edge learns from the committed value its evaluation read,
/// where the retired post-evaluation replay read this visit's output.
#[test]
fn a_recurrent_hebbian_edge_learns_from_the_value_its_evaluation_read() {
    let hebbian = PlasticityConfig {
        modulation: None,
        weight_clamp: 10.0,
        ..config(HebbianRule::Classic, 0.1, 0.0)
    };
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
            Some(hebbian),
        )])
        .with_output_sinks(vec![]);
    def.push_edge(
        EdgeSurface::ComputeInput(0),
        GraphEdge::new(
            GraphSource::SharedMemory {
                slot: 0,
                previous: false,
            },
            1.0,
        ),
    );
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    state.node_outputs = vec![vec![4.0]];
    begin(&mut state);
    visit(&def, &mut state, 1.0, &mut 100.0, &RuntimeConfig::default());
    // Output 0.5 * 4 + 1 = 3; the self-edge read the committed 4.0, so
    // w = 0.5 + 0.1 * 4 * 3 (a replay of this visit's output would give
    // 0.5 + 0.1 * 3 * 3).
    assert_eq!(state.node_outputs[0], vec![3.0]);
    assert!((weight(&state) - (0.5 + 0.1 * 4.0 * 3.0)).abs() < 1e-6);
}

#[test]
fn activity_uses_evaluation_energy_beside_pure_hebbian_learning() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.0);
    def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0))
        .set_source(GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        });
    def.append_compute_node(def.compute_nodes()[0].clone());
    edit_config(&mut def, 1, |config| config.modulation = None);
    let runtime = RuntimeConfig {
        graph_node_base_cost: 1.0,
        ..RuntimeConfig::default()
    };
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    begin(&mut state);
    let mut energy = 10.0;
    let side = visit(&def, &mut state, 0.0, &mut energy, &runtime);
    // Two nodes' pass cost; the pure Hebbian update is free (T25.F02).
    assert_eq!(energy, 8.0);
    assert_eq!(side.work_counters.plasticity_updates, 1);
    // The read is the evaluation energy (8.0) as a fraction of max_energy.
    assert_eq!(trace(&state), 8.0 / 200.0);
}

#[test]
fn failed_visits_preserve_decay_and_last_successful_activity() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.5);
    def.append_compute_node(def.compute_nodes()[0].clone());
    edit_config(&mut def, 1, |config| config.modulation = None);
    let runtime = RuntimeConfig {
        graph_node_base_cost: 1.0,
        ..RuntimeConfig::default()
    };
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    begin(&mut state);
    visit(&def, &mut state, 2.0, &mut 100.0, &runtime);
    let learned = learned_at(&state, 1);
    begin(&mut state);
    // Both visits fail at the two-node pass cost, the only exhaustion point;
    // a failed visit changes no learned value or trace.
    for energy in [1.0, 2.0] {
        visit(&def, &mut state, 9.0, &mut { energy }, &runtime);
        assert_eq!(trace(&state), 1.0);
        assert_eq!(learned_at(&state, 1), learned);
    }
    visit(&def, &mut state, 3.0, &mut 100.0, &runtime);
    for energy in [1.0, 2.0] {
        visit(&def, &mut state, 9.0, &mut { energy }, &runtime);
        assert_eq!(trace(&state), 4.0);
    }
}

#[test]
fn exact_rules_rewards_rates_and_hebbian_equivalence() {
    let pre = 0.75;
    let post = -0.25;
    let weight_0 = 0.5;
    for (rule, activity) in [
        (HebbianRule::Classic, pre * post),
        (HebbianRule::Oja, post * (pre - weight_0 * post)),
        (HebbianRule::AntiHebb, -pre * post),
        (HebbianRule::Covariance, (pre - 0.5) * (post - 0.5)),
    ] {
        for eta in [0.0, 0.4, 1.0] {
            for signal in [-0.75, 0.0, 1.0] {
                let mut def = graph(rule, eta, 0.0);
                def.set_compute_kind(0, ComputeNodeKind::Constant(post));
                let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
                begin(&mut state);
                visit(&def, &mut state, pre, &mut 100.0, &RuntimeConfig::default());
                assert!((trace(&state) - activity).abs() < 1e-7);
                assert_eq!(reward(&mut state, signal), 1);
                let expected = weight_0 + eta * signal * activity;
                assert!((weight(&state) - expected).abs() < 1e-7);
                if signal == 1.0 {
                    let mut pure = def.clone();
                    edit_config(&mut pure, 0, |config| config.modulation = None);
                    let mut pure_state = GraphRuntimeState::for_genome(&genome(&pure), Vec::new());
                    begin(&mut pure_state);
                    visit(
                        &pure,
                        &mut pure_state,
                        pre,
                        &mut 100.0,
                        &RuntimeConfig::default(),
                    );
                    assert_eq!(weight(&pure_state), weight(&state));
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
            let mut skipped = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
            begin(&mut skipped);
            visit(
                &def,
                &mut skipped,
                0.75,
                &mut 100.0,
                &RuntimeConfig::default(),
            );
            let mut zero = skipped.clone();
            for _ in 0..delay {
                begin(&mut skipped);
                begin(&mut zero);
                visit(&def, &mut zero, 0.0, &mut 100.0, &RuntimeConfig::default());
            }
            let expected = lambda.powi(delay) * 0.75;
            assert_eq!(trace(&skipped), expected);
            assert_eq!(trace(&skipped), trace(&zero));
            reward(&mut skipped, -0.75);
            assert!((weight(&skipped) - (0.5 - 0.4 * 0.75 * expected)).abs() < 1e-7);
        }
    }
}

/// An unvisited, unseeded backend is uninitialized: the view exposes no
/// learned value or trace and reward assigns nothing. A failed first visit
/// initializes it (the visited flag is set at entry) with zero credit.
#[test]
fn clock_does_not_initialize_unvisited_modules_or_charge_reward_work() {
    let def = graph(HebbianRule::Classic, 1.0, 0.5);
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    for _ in 0..4 {
        begin(&mut state);
    }
    assert_eq!(trace_at(&state, 0), None);
    assert_eq!(learned_at(&state, 0), None);
    assert_eq!(reward(&mut state, 1.0), 0);
    // Failed first evaluation initializes zeros but never activity.
    visit(&def, &mut state, 3.0, &mut 0.0, &RuntimeConfig::default());
    assert_eq!(trace(&state), 0.0);
    assert_eq!(reward(&mut state, 1.0), 1);
    assert_eq!(weight(&state), 0.5);
}

#[test]
fn first_tick_repeated_visits_have_zero_base_and_ordinary_traced_parity() {
    let mut def = graph(HebbianRule::Oja, 0.4, 0.7);
    push_sink(
        &mut def,
        OutputSink::new(
            OutputSinkKind::ActionVote(VoteSink::Eat),
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
        ),
    );
    let runtime = RuntimeConfig::default();
    let mut ordinary = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    let mut traced = ordinary.clone();
    for _ in 0..3 {
        begin(&mut ordinary);
        begin(&mut traced);
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
            assert_eq!(ordinary.learned(), traced.learned());
            assert_eq!(ordinary.node_outputs, traced.node_outputs);
            assert_eq!(energy, traced_energy);
            assert_eq!(side.work_counters, traced_side.work_counters);
            assert_eq!(side.dispatch_effects(), traced_side.dispatch_effects());
        }
        assert_eq!(reward(&mut ordinary, 0.5), reward(&mut traced, 0.5));
        assert_eq!(ordinary.learned(), traced.learned());
    }
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn skipped_tick_decay_is_exponential(lambda in 0.0f32..=1.0, pulse in -2.0f32..2.0, ticks in 0i32..24) {
        let def = graph(HebbianRule::Classic, 0.5, lambda);
        let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new()); begin(&mut state);
        visit(&def, &mut state, pulse, &mut 100.0, &RuntimeConfig::default());
        for _ in 0..ticks { begin(&mut state); }
        let expected = lambda.powi(ticks) * pulse;
        prop_assert!((trace(&state) - expected).abs() < 2e-6);
    }

    /// Disconnected computation adds no credit, and every visit adds its
    /// activity (T19.F02): `visits` visits of the same module in one tick
    /// leave `visits` times the single-visit activity on top of the decayed
    /// base, because a `Constant` input and a fixed `Add` output make each
    /// visit's activity identical.
    #[test]
    fn disconnected_computation_does_not_change_credit_and_each_visit_adds(
        input in -2.0f32..2.0, lambda in 0.0f32..=1.0, visits in 1usize..8, extras in 0usize..8,
    ) {
        let def = graph(HebbianRule::Classic, 0.5, lambda);
        let mut bigger = def.clone();
        for _ in 0..extras { bigger.append_compute_node(ComputeNode::new(ComputeNodeKind::Oscillator(0.25), vec![])); }
        let mut once = GraphRuntimeState::for_genome(&genome(&def), Vec::new()); let mut repeated = GraphRuntimeState::for_genome(&genome(&bigger), Vec::new());
        let runtime = RuntimeConfig::default();
        let credit = |state: &GraphRuntimeState| trace_at(state, 0).unwrap_or(0.0);
        for _ in 0..3 {
            begin(&mut once); begin(&mut repeated);
            let activity = { let base = credit(&once); visit(&def, &mut once, input, &mut 100.0, &runtime); credit(&once) - base };
            let base = credit(&repeated);
            for _ in 0..visits { visit(&bigger, &mut repeated, input, &mut 100.0, &runtime); }
            let expected = base + activity * visits as f32;
            prop_assert!((credit(&repeated) - expected).abs() <= 1e-5 * (1.0 + expected.abs()),
                "{} vs {}", credit(&repeated), expected);
        }
    }

    /// The three-factor update bounds the stored weight by the edge's clamp;
    /// the parameters are in range (I1), so no law re-clamps them.
    #[test]
    fn normalized_reward_update_obeys_exact_clamp(
        weight_0 in -10.0f32..10.0, pulse in -20.0f32..20.0, signal in -20.0f32..20.0,
        eta in 0.0f32..=1.0, clamp in 0.01f32..=10.0,
    ) {
        let mut def = graph(HebbianRule::Classic, eta, 0.0);
        def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0)).set_weight(weight_0);
        edit_config(&mut def, 0, |config| config.weight_clamp = clamp);
        let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new()); begin(&mut state);
        visit(&def, &mut state, pulse, &mut 100.0, &RuntimeConfig::default());
        reward(&mut state, signal);
        let actual = weight(&state);
        prop_assert_eq!(actual, (weight_0 + eta * signal * pulse).clamp(-clamp, clamp));
        prop_assert!(actual >= -clamp && actual <= clamp);
    }
}

/// Authored action-contingent outcomes are a test treatment, never a production sensor.
#[test]
fn constructed_controller_adapts_to_reversal_while_frozen_weights_do_not() {
    use crate::contracts::WorldAction;
    use crate::runtime::mesh::execute_creature_mesh;
    let started = std::time::Instant::now();
    let mut def = graph(HebbianRule::Classic, 0.5, 0.0);
    def.set_compute_kind(0, ComputeNodeKind::Constant(1.0));
    set_compute_inputs(&mut def, 0, Vec::new());
    def.append_compute_node(plastic_node(
        ComputeNodeKind::Threshold(0.0),
        vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
        Some(config(HebbianRule::Classic, 0.5, 0.0)),
    ));
    let edge = |index, weight| GraphEdge::new(GraphSource::ComputeNode(index), weight);
    // Eat when the learned gate fires; otherwise nothing is voted and the
    // tick is `NoOp`.
    def = def.with_output_sinks(vec![OutputSink::new(
        OutputSinkKind::ActionVote(VoteSink::Eat),
        vec![edge(1, 1.0)],
    )]);
    let genome = genome(&def);
    let run = |state: &mut GraphRuntimeState, acquisition: bool, frozen: bool| {
        let mut correct = Vec::new();
        let mut weights = Vec::new();
        let mut counts = [0usize; 2];
        for tick in 0..32 {
            state.begin_tick(tick);
            let result = execute_creature_mesh(
                &genome,
                &sensors(),
                &mut 100.0,
                &mut [0.0; 16],
                &[0.0; 16],
                state,
                &RuntimeConfig::default(),
                crate::contracts::Direction::N,
            );
            assert_eq!(result.actions.len(), 1);
            let first = matches!(result.actions[0], WorldAction::Eat { .. });
            assert!(first || matches!(result.actions[0], WorldAction::NoOp));
            counts[usize::from(!first)] += 1;
            let success = first == acquisition;
            correct.push(success);
            if !frozen {
                reward(state, if success { 1.0 } else { -1.0 });
            }
            weights.push(weight(state));
        }
        (correct, weights, counts)
    };
    let mut live = GraphRuntimeState::for_genome(&genome, Vec::new());
    let acquisition = run(&mut live, true, false);
    assert_ne!(weight(&live), 0.5);
    assert!(acquisition.0[24..].iter().filter(|&&x| x).count() >= 7);
    let mut frozen = live.clone();
    let acquired_weights = weight(&frozen);
    let reversal = run(&mut live, false, false);
    let control = run(&mut frozen, false, true);
    assert!(reversal.0[24..].iter().filter(|&&x| x).count() >= 7);
    assert!(control.0[24..].iter().filter(|&&x| x).count() <= 1);
    assert_eq!(weight(&frozen), acquired_weights);
    eprintln!("reversal acquisition counts {:?}, weights {:?}; live counts {:?}, weights {:?}; frozen counts {:?}, weights {:?}", acquisition.2, acquisition.1, reversal.2, reversal.1, control.2, control.1);
    assert!(started.elapsed().as_secs_f32() < 60.0);
}

/// The reward update bounds the stored weight by the edge's clamp.
#[test]
fn explicit_reward_clamp_bounds() {
    for (signal, expected) in [(-10.0, -2.0), (10.0, 2.0)] {
        let def = graph(HebbianRule::Classic, 1.0, 0.0);
        let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
        begin(&mut state);
        visit(&def, &mut state, 3.0, &mut 100.0, &RuntimeConfig::default());
        reward(&mut state, signal);
        assert_eq!(weight(&state), expected);
    }
}

/// I1 at the load boundary (T25.F06): a trace decay outside `[0, 1]` (the
/// former runtime-normalized −1 and 2), a learning rate outside `[0, 1]` and
/// a weight clamp outside `[0.01, 10]` are rejected.
#[test]
fn out_of_range_learning_attributes_are_rejected_at_the_load_boundary() {
    let edge = |plasticity: &str| {
        format!(
            r#"{{"source":{{"SharedMemory":{{"slot":0,"previous":false}}}},"own_weight":0.5,"plasticity":{plasticity}}}"#
        )
    };
    let node = |plasticity: &str| {
        let json = format!(
            r#"{{"compute_nodes":[{{"kind":"Add","inputs":[{}]}}],"output_sinks":[]}}"#,
            edge(plasticity)
        );
        let def: CgpGraphBackendDef = serde_json::from_str(&json).unwrap();
        NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }
    };
    let modulated = |decay: &str| {
        format!(
            r#"{{"rule":"Classic","learning_rate":1.0,"weight_clamp":2.0,"lamarckian":false,"modulation":{{"reward_source":"EnergyDelta","trace_decay":{decay}}}}}"#
        )
    };
    let hebbian = |eta: &str, clamp: &str| {
        format!(
            r#"{{"rule":"Classic","learning_rate":{eta},"weight_clamp":{clamp},"lamarckian":false}}"#
        )
    };
    for plasticity in [
        modulated("-1.0"),
        modulated("2.0"),
        hebbian("-0.5", "2.0"),
        hebbian("1.5", "2.0"),
        hebbian("0.5", "50.0"),
        hebbian("0.5", "0.0"),
    ] {
        assert_eq!(
            validate_graph(&node(&plasticity)),
            Err(GraphInvariantViolation::PlasticityOutOfRange),
            "{plasticity}"
        );
    }
    for plasticity in [
        modulated("0.0"),
        modulated("1.0"),
        hebbian("0.0", "0.01"),
        hebbian("1.0", "10.0"),
    ] {
        assert_eq!(validate_graph(&node(&plasticity)), Ok(()), "{plasticity}");
    }
}

/// `set_plasticity` debug-asserts I1.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "a config is in range (I1)")]
fn set_plasticity_debug_asserts_the_range() {
    let mut def = graph(HebbianRule::Classic, 1.0, 0.5);
    edit_config(&mut def, 0, |config| config.weight_clamp = 50.0);
}

/// A seeded Lamarckian child starts at its inherited weight, and its
/// seeded reward-modulated slot counts from birth (T25.F06 invariant 12):
/// one zero-delta assignment before its first visit.
#[test]
fn birth_aligned_weights_drive_first_execution_and_reward_without_parent_credit() {
    let mut def = graph(HebbianRule::Classic, 0.1, 0.5);
    def.set_compute_kind(0, ComputeNodeKind::Add);
    edit_config(&mut def, 0, |config| config.lamarckian = true);
    def.seed_compute_rows(vec![vec![Some(0.75)]]);
    def.duplicate_compute_nodes(&[0]);
    def.remove_compute_node(0);
    let seeds = vec![def.take_birth_seeds()];
    let mut child = GraphRuntimeState::for_genome(&genome(&def), seeds);
    assert_eq!(trace_at(&child, 0), Some(0.0));
    assert_eq!(reward_counts(&mut child, 1.0), (1, 0));
    assert_eq!(weight(&child), 0.75);
    begin(&mut child);
    visit(&def, &mut child, 2.0, &mut 100.0, &RuntimeConfig::default());
    assert_eq!(child.node_outputs[0][0], 1.5);
    assert_eq!(trace(&child), 3.0);
    reward(&mut child, 1.0);
    assert!((weight(&child) - 1.05).abs() < 1e-6);
}

/// A seed is clamped by the child's edge clamp at construction (T25.F06
/// invariant 7), a Darwinian edge ignores its seed, and a seedless backend
/// is uninitialized.
#[test]
fn inherited_weight_above_a_mutated_child_clamp_is_clamped() {
    for (lamarckian, seed, expected) in [
        (true, Some(3.5), Some(2.0)),
        (true, Some(-3.5), Some(-2.0)),
        (true, Some(1.25), Some(1.25)),
        (false, Some(1.25), None),
        (true, None, None),
    ] {
        let mut def = graph(HebbianRule::Classic, 0.1, 0.5);
        edit_config(&mut def, 0, |config| config.lamarckian = lamarckian);
        def.seed_compute_rows(vec![vec![seed]]);
        let seeds = vec![def.take_birth_seeds()];
        let child = GraphRuntimeState::for_genome(&genome(&def), seeds);
        assert_eq!(learned_at(&child, 0), expected, "{lamarckian} {seed:?}");
        let applied = child.learned().applied_weight(
            0,
            &def,
            &[],
            EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0),
        );
        assert_eq!(applied, expected.unwrap_or(0.5));
    }
}

/// A backend with one seeded Lamarckian edge and one unseeded edge per
/// surface exposes, updates and captures exactly the seeded slots before its
/// first visit, then every slot (T25.F06 invariants 4, 8 and 12).
#[test]
fn a_mixed_seed_backend_initializes_only_its_seeded_slot() {
    let lamarckian = PlasticityConfig {
        lamarckian: true,
        ..config(HebbianRule::Classic, 0.1, 0.5)
    };
    let memory = GraphEdge::new(
        GraphSource::SharedMemory {
            slot: 0,
            previous: false,
        },
        0.5,
    );
    let mut def =
        CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![plastic_node(
            ComputeNodeKind::Add,
            vec![memory, memory],
            Some(lamarckian),
        )]);
    let vote = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap();
    def.push_edge(vote, GraphEdge::new(GraphSource::ComputeNode(0), 1.0));
    def.push_edge(vote, GraphEdge::new(GraphSource::ComputeNode(0), 1.0));
    set_surface_plasticity(&mut def, vote, Some(lamarckian));
    let EdgeSurface::SinkInput(vote_index) = vote else {
        unreachable!()
    };
    let mut seeds = BirthSeeds::Rows {
        compute: vec![vec![Some(0.25), None]],
        sinks: def
            .output_sinks()
            .iter()
            .map(|sink| vec![None; sink.inputs.len()])
            .collect(),
        groups: Vec::new(),
    };
    let BirthSeeds::Rows { sinks, .. } = &mut seeds;
    sinks[vote_index][1] = Some(-0.75);
    let mut child = GraphRuntimeState::for_genome(&genome(&def), vec![Some(seeds)]);
    let learned: Vec<_> = (0..4).map(|slot| learned_at(&child, slot)).collect();
    assert_eq!(learned, vec![Some(0.25), None, None, Some(-0.75)]);
    let traces: Vec<_> = (0..4).map(|slot| trace_at(&child, slot)).collect();
    assert_eq!(traces, vec![Some(0.0), None, None, Some(0.0)]);
    assert_eq!(reward_counts(&mut child, 1.0), (2, 0));
    let captured = child.learned().capture(0, &def).expect("seeded slots");
    assert_eq!(captured.compute_rows(), &[vec![Some(0.25), None]]);
    assert_eq!(captured.sink_rows()[vote_index], vec![None, Some(-0.75)]);
    begin(&mut child);
    visit(&def, &mut child, 1.0, &mut 100.0, &RuntimeConfig::default());
    assert!((0..4).all(|slot| learned_at(&child, slot).is_some()));
    assert_eq!(reward_counts(&mut child, 1.0).0, 4);
    let captured = child.learned().capture(0, &def).expect("every slot");
    assert!(captured.compute_rows()[0].iter().all(Option::is_some));
    assert!(captured.sink_rows()[vote_index].iter().all(Option::is_some));
}

/// A graph without a plastic edge allocates no learned state: the layout
/// is empty, the view yields nothing and a capture seeds nothing.
#[test]
fn a_non_learning_graph_allocates_no_learned_state() {
    let def =
        CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![ComputeNode::new(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
        )]);
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    assert!(!state.learned().has_learning());
    assert_eq!(
        state.learned(),
        &crate::runtime::plasticity::LearnedState::default()
    );
    begin(&mut state);
    visit(&def, &mut state, 1.0, &mut 100.0, &RuntimeConfig::default());
    assert!(!state.learned().has_learning());
    assert_eq!(state.learned().edges(0).count(), 0);
    assert!(state.learned().capture(0, &def).is_none());
    let founder = crate::creature::founder::founder_genome(crate::config::FounderProfile::V3Alpha1);
    assert!(!GraphRuntimeState::for_genome(&founder, Vec::new())
        .learned()
        .has_learning());
}

/// Learned-state equality is the learned slots: two births of one genome
/// are equal, and a visit that moves a weight makes them differ.
#[test]
fn learned_states_differ_once_a_visit_moves_a_learned_weight() {
    let def = graph(HebbianRule::Classic, 1.0, 0.5);
    let untouched = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    let mut visited = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    assert_eq!(untouched.learned(), visited.learned());
    begin(&mut visited);
    visit(
        &def,
        &mut visited,
        1.0,
        &mut 100.0,
        &RuntimeConfig::default(),
    );
    assert!(learned_at(&visited, 0).is_some());
    assert_ne!(untouched.learned(), visited.learned());
    assert_ne!(
        visited.learned(),
        &crate::runtime::plasticity::LearnedState::default()
    );
}

/// A pure Hebbian sink edge with `learning_rate` `eta` and clamp 10.
fn hebbian_sink(eta: f32) -> PlasticityConfig {
    PlasticityConfig {
        rule: HebbianRule::Classic,
        learning_rate: eta,
        weight_clamp: 10.0,
        lamarckian: false,
        modulation: None,
    }
}

/// Push `edge` onto `def`'s sink `kind` with `config`.
fn push_plastic_sink_edge(
    def: &mut CgpGraphBackendDef,
    kind: OutputSinkKind,
    edge: GraphEdge,
    config: Option<PlasticityConfig>,
) {
    let surface = def.sink_surface(kind).unwrap();
    def.push_edge(surface, edge);
    let index = def.surface_edges(surface).len() - 1;
    def.edge_mut(EdgeOccurrence::new(surface, index))
        .set_plasticity(config);
}

fn memory_edge(slot: u8, weight: f32) -> GraphEdge {
    GraphEdge::new(
        GraphSource::SharedMemory {
            slot,
            previous: false,
        },
        weight,
    )
}

/// Invariant 5: a sink edge learns from the value the sink sum read, with
/// shared memory as earlier sinks wrote it. `CustomOutput(0)` reads slot 1
/// before `WriteSlot(1)` overwrites it with slot 2; the `Eat` vote, later in
/// sink order, reads the written value. A replay after the effects would
/// read the written value on both edges.
#[test]
fn sink_edge_learns_from_the_recorded_sample_not_a_replay() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::CustomOutput(0),
        memory_edge(1, 1.0),
        Some(hebbian_sink(0.5)),
    );
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::WriteSlot(1),
        memory_edge(2, 1.0),
        None,
    );
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        memory_edge(1, 1.0),
        Some(hebbian_sink(0.1)),
    );
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    begin(&mut state);
    let mut memory = [0.0; 16];
    memory[1] = 2.0;
    memory[2] = 5.0;
    let side = visit_with_memory(
        &def,
        &mut state,
        &mut memory,
        &mut 100.0,
        &RuntimeConfig::default(),
    );
    assert_eq!(memory[1], 5.0);
    assert_eq!(side.work_counters.plasticity_updates, 2);
    // CustomOutput(0): pre 2 (before the write), post 2: 1 + 0.5 * 2 * 2.
    assert_eq!(learned_at(&state, 0), Some(3.0));
    // Eat: pre 5 (after the write), post 5: 1 + 0.1 * 5 * 5.
    assert_eq!(learned_at(&state, 1), Some(3.5));
}

/// The goal: a plastic `ActionVote` edge moves its learned weight after a
/// visit and the next visit's vote contribution follows it.
#[test]
fn a_vote_edge_learns_and_its_vote_follows() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        memory_edge(0, 0.5),
        Some(hebbian_sink(0.25)),
    );
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    let runtime = RuntimeConfig::default();
    begin(&mut state);
    let first = visit(&def, &mut state, 2.0, &mut 100.0, &runtime);
    let eat = VoteSink::Eat.index();
    assert_eq!(first.dispatch_effects().0[eat], 1.0);
    // pre 2, post 1: 0.5 + 0.25 * 2 * 1.
    assert_eq!(learned_at(&state, 0), Some(1.0));
    let second = visit(&def, &mut state, 2.0, &mut 100.0, &runtime);
    assert_eq!(second.dispatch_effects().0[eat], 2.0);
    assert_eq!(second.work_counters.plasticity_updates, 1);
}

/// Invariant 5: a sink edge reading `EnergyCurrent` samples the evaluation
/// energy the sink sum read (after the pass cost), as a fraction of
/// `max_energy`.
#[test]
fn a_sink_energy_current_edge_samples_its_evaluation_read() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::ActionParam(crate::creature::genome::vote::ActionParamField::ALL[0]),
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            },
            1.0,
        ),
        Some(hebbian_sink(1.0)),
    );
    let runtime = RuntimeConfig {
        graph_node_base_cost: 20.0,
        ..RuntimeConfig::default()
    };
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    begin(&mut state);
    let mut energy = 100.0;
    visit(&def, &mut state, 0.0, &mut energy, &runtime);
    assert_eq!(energy, 80.0);
    let pre = 80.0f32 / 200.0;
    assert_eq!(learned_at(&state, 0), Some(1.0 + pre * pre));
}

/// The sink trace carries each edge's weighted input as the node trace does
/// (T25.F06 invariant 14), at the applied weight.
#[test]
fn the_sink_trace_carries_weighted_inputs_at_the_applied_weight() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        memory_edge(0, 0.5),
        Some(hebbian_sink(0.25)),
    );
    push_plastic_sink_edge(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        memory_edge(0, 2.0),
        None,
    );
    let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
    begin(&mut state);
    visit(&def, &mut state, 2.0, &mut 100.0, &RuntimeConfig::default());
    let learned = learned_at(&state, 0).unwrap();
    let mut memory = [0.0; 16];
    memory[0] = 2.0;
    let (_, trace) = crate::runtime::cgp::traced::execute_graph_node_traced(
        &def,
        &[],
        &[0.0; OUTPUT_SLOT_COUNT],
        &mut 100.0,
        0.0,
        0,
        &mut state,
        &sensors(),
        &RuntimeConfig::default(),
        &mut MeshSideOutputs::new(10),
        &mut memory,
        &[0.0; 16],
    );
    let EdgeSurface::SinkInput(eat) = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap()
    else {
        unreachable!()
    };
    assert_eq!(
        trace.output_sinks[eat].weighted_inputs,
        vec![2.0 * learned, 4.0]
    );
    assert!(trace.output_sinks[0].weighted_inputs.is_empty());
}

#[test]
fn learning_counts_assignments_separately_from_final_weight_changes() {
    // Zero activity, a clamped-away delta, and a rounded-away delta still cost
    // one assignment. Only the last case changes the stored f32.
    for (weight_0, eta, input, expected) in [
        (0.5, 0.5, 0.0, 0.5),
        (2.0, 0.5, 1.0, 2.0),
        (1.0, f32::MIN_POSITIVE, 1.0, 1.0),
        (0.5, 0.5, 1.0, 1.0),
    ] {
        for modulated in [false, true] {
            let mut def = graph(HebbianRule::Classic, eta, 0.0);
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0))
                .set_weight(weight_0);
            if !modulated {
                edit_config(&mut def, 0, |config| config.modulation = None);
            }
            let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
            begin(&mut state);
            let runtime = RuntimeConfig {
                graph_node_base_cost: 0.0,
                ..RuntimeConfig::default()
            };
            let mut energy = 100.0;
            let side = visit(&def, &mut state, input, &mut energy, &runtime);
            // Learning is free (T25.F02): neither form spends energy.
            assert_eq!(energy, 100.0);
            let (assignments, changes) = if modulated {
                assert_eq!(side.work_counters.plasticity_updates, 0);
                assert_eq!(side.work_counters.plasticity_changes, 0);
                reward_counts(&mut state, 1.0)
            } else {
                (
                    side.work_counters.plasticity_updates,
                    side.work_counters.plasticity_changes,
                )
            };
            assert_eq!(weight(&state), expected);
            assert_eq!((assignments, changes), (1, u32::from(expected != weight_0)));
        }
    }
}

proptest! {
    #[test]
    fn learning_change_counts_match_each_final_assignment(
        weights in prop::collection::vec(-2.0f32..2.0, 1..12),
        eta in 0.0f32..1.0, input in -2.0f32..2.0, signal in -2.0f32..2.0,
        modulated in any::<bool>(),
    ) {
        let mut def = graph(HebbianRule::Classic, eta, 0.0);
        let edge = def.compute_nodes()[0].inputs[0];
        let plasticity = edge.plasticity();
        set_compute_inputs(&mut def, 0, weights.iter().map(|&weight| GraphEdge::new(edge.source(), weight)).collect());
        set_surface_plasticity(&mut def, EdgeSurface::ComputeInput(0), plasticity);
        if !modulated { edit_config(&mut def, 0, |config| config.modulation = None); }
        let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
        begin(&mut state);
        let side = visit(&def, &mut state, input, &mut 100.0, &RuntimeConfig::default());
        let (assignments, changes) = if modulated {
            reward_counts(&mut state, signal)
        } else {
            (side.work_counters.plasticity_updates, side.work_counters.plasticity_changes)
        };
        let learned: Vec<f32> = state.learned().edges(0).filter_map(|edge| edge.learned).collect();
        let expected_changes = weights.iter().zip(&learned).filter(|(old, new)| old != new).count() as u32;
        prop_assert_eq!(assignments, weights.len() as u32);
        prop_assert_eq!(changes, expected_changes);
        prop_assert!(changes <= assignments);
    }
}
