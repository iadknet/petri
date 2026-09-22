//! T11.F03 property and example tests: every growth operator (the three
//! add-node forms, copy node, copy subgraph, and unwired input-reference
//! add on both backends) is neutral at the moment it fires under ample
//! energy and passes, a compute-consumer split preserves every surviving
//! node's per-pass value, `random_graph_source` reaches every sub-value of a
//! compound reference, and the graph raw-field operator changes exactly one
//! field by one unit.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::config::{MutationConfig, RuntimeConfig};
use crate::contracts::{InputReference, WorldAction, WorldInputKey};
use crate::creature::genome::cgp::{
    ActionSlot, ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate,
    GraphEdge, GraphSource, OutputSink, OutputSinkKind, WorldActionKind,
};
use crate::creature::state::GraphRuntimeState;
use crate::mutation::graph::operators::{
    add_bootstrap_node, add_disconnected_node, copy_cgp_subgraph, copy_compute_node,
    raw_field_mutation, split_existing_edge,
};
use crate::mutation::types::MutationSkipReason;
use crate::runtime::cgp::effects::CgpEffectsTrace;
use crate::runtime::cgp::execute::{execute_graph_impl, GraphTracer};
use crate::runtime::cgp::execute_graph_node;
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

// ─── Shared fixtures ────────────────────────────────────────────────────────

/// A handful of hand-picked scenarios, not the full 80-scenario
/// `neighborhood::battery::Battery`: enough sensor variety to exercise every
/// wired surface without the cost of the full battery per proptest case.
pub(super) fn scenarios() -> Vec<SensorSnapshot> {
    let base = |food_here: f32, neighbor_food: [f32; 8], age: f32| SensorSnapshot {
        local: StaticInputs {
            food_here,
            neighbor_food,
            neighbor_barrier: [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: age,
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    };
    vec![
        base(0.0, [0.0; 8], 0.0),
        base(1.0, [0.2, 0.4, 0.6, 0.8, 1.0, 0.0, 0.3, 0.7], 20.0),
        base(0.5, [0.9; 8], 200.0),
    ]
}

/// Ample energy and passes per the T11.F02/T11.F03 neutrality definition:
/// identical action, output-slot, and shared-memory behavior when both
/// executions have enough energy. The base `graph_node_base_cost` is left
/// at production default so the cost is real, just affordable at this
/// energy level.
pub(super) fn ample_runtime_config() -> RuntimeConfig {
    RuntimeConfig::default()
}

/// Execute `def` against every scenario with ample energy, returning each
/// scenario's `(NodeResult, action_kinds, shared_memory_after)`.
pub(super) fn run_scenarios(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    config: &RuntimeConfig,
) -> Vec<(NodeResult, Vec<WorldAction>, [f32; 16])> {
    scenarios()
        .iter()
        .map(|sensors| {
            let mut energy = 1.0e6f32;
            let mut shared_memory = [0.0f32; 16];
            let prev_shared_memory = [0.0f32; 16];
            let mut graph_runtime = GraphRuntimeState::new();
            let mut side_outputs = MeshSideOutputs::new(8);
            let result = execute_graph_node(
                def,
                input_refs,
                &[0.0f32; OUTPUT_SLOT_COUNT],
                &mut energy,
                0.0,
                0,
                &mut graph_runtime,
                sensors,
                config,
                &mut side_outputs,
                &mut shared_memory,
                &prev_shared_memory,
            );
            let actions = side_outputs.action_queue.clone().into_actions();
            (result, actions, shared_memory)
        })
        .collect()
}

/// Assert that `child` behaves exactly as `parent` did across every
/// scenario: identical actions, output slots, route gate scores, terminal
/// and energy-exhausted flags, and shared-memory writes.
pub(super) fn assert_neutral(
    label: &str,
    parent: &CgpGraphBackendDef,
    parent_refs: &[InputReference],
    child: &CgpGraphBackendDef,
    child_refs: &[InputReference],
    config: &RuntimeConfig,
) {
    let before = run_scenarios(parent, parent_refs, config);
    let after = run_scenarios(child, child_refs, config);
    assert_eq!(before.len(), after.len());
    for (i, (b, a)) in before.iter().zip(after.iter()).enumerate() {
        let (b_result, b_actions, b_mem) = b;
        let (a_result, a_actions, a_mem) = a;
        assert_eq!(b_actions, a_actions, "{label}: scenario {i} actions differ");
        assert_eq!(
            b_result.output_slots, a_result.output_slots,
            "{label}: scenario {i} output slots differ"
        );
        assert_eq!(
            b_result.route_gates.scores, a_result.route_gates.scores,
            "{label}: scenario {i} route gates differ"
        );
        assert_eq!(
            b_result.terminal, a_result.terminal,
            "{label}: scenario {i} terminal differs"
        );
        assert_eq!(
            b_result.energy_exhausted, a_result.energy_exhausted,
            "{label}: scenario {i} energy_exhausted differs"
        );
        assert_eq!(b_mem, a_mem, "{label}: scenario {i} shared memory differs");
    }
}

/// A moderately interesting base graph: forward edges, a backward
/// self-reference, and every wired surface (sink, action gate and param,
/// execute gate), so growth operators are exercised against real structure.
pub(super) fn base_def() -> CgpGraphBackendDef {
    CgpGraphBackendDef {
        birth_weights: None,
        compute_nodes: vec![
            ComputeNode {
                kind: ComputeNodeKind::Sigmoid,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 3,
                    },
                    weight: 0.6,
                }],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::WeightedSum,
                inputs: vec![
                    GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 0.9,
                    },
                    GraphEdge {
                        source: GraphSource::SharedMemory {
                            slot: 2,
                            previous: false,
                        },
                        weight: -0.5,
                    },
                ],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::DecayIntegrator(0.4),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(2), // self-loop
                    weight: 0.3,
                }],
                plasticity: None,
            },
        ],
        output_sinks: vec![OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            }],
        }],
        action_bank: vec![ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            param_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(2),
                weight: 1.0,
            }],
            direction_bids: Vec::new(),
        }],
        execute_gate: ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            }],
        },
    }
}

pub(super) fn base_input_refs() -> Vec<InputReference> {
    vec![
        InputReference::World(WorldInputKey::NeighborBarrierRing), // width 8
        InputReference::ActionQueue,
    ]
}

/// A second fixture carrying a Hebbian-plasticity compute node and a
/// `DynamicIntrospection(EnergyCurrent)` input reference wired directly onto
/// a non-compute surface (the action slot's param input), on top of the
/// same forward/backward/self-loop shape as `base_def`. Exercises the
/// growth-neutrality properties against plasticity's post-convergence
/// energy deduction and against the introspection reference kind, neither
/// of which `base_def` carries. T11.F08's split exclusion keeps the split
/// operator away from this fixture's action-param edge, which reads a live
/// introspection reference directly on a non-compute surface.
pub(super) fn plasticity_def() -> CgpGraphBackendDef {
    CgpGraphBackendDef {
        birth_weights: None,
        compute_nodes: vec![
            ComputeNode {
                kind: ComputeNodeKind::WeightedSum,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 3,
                    },
                    weight: 0.5,
                }],
                plasticity: Some(crate::creature::genome::PlasticityConfig {
                    rule: crate::creature::genome::HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: false,
                    modulation: None,
                }),
            },
            ComputeNode {
                kind: ComputeNodeKind::Sigmoid,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 0.7,
                }],
                plasticity: None,
            },
        ],
        output_sinks: vec![OutputSink {
            kind: OutputSinkKind::CustomOutput(0),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            }],
        }],
        action_bank: vec![ActionSlot {
            behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
            gate_inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            }],
            param_inputs: vec![GraphEdge {
                // Directly on a non-compute surface: T11.F08's split
                // exclusion skips this edge instead of caching its value in
                // an identity node ahead of the plasticity-cost deduction.
                source: GraphSource::InputLeaf {
                    ref_idx: 2,
                    sub_idx: 0,
                },
                weight: 1.0,
            }],
            direction_bids: Vec::new(),
        }],
        execute_gate: ExecuteGate {
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            }],
        },
    }
}

pub(super) fn plasticity_input_refs() -> Vec<InputReference> {
    vec![
        InputReference::World(WorldInputKey::NeighborBarrierRing), // width 8, ref_idx 0
        InputReference::ActionQueue,                               // ref_idx 1
        InputReference::DynamicIntrospection(
            crate::contracts::DynamicIntrospectionKey::EnergyCurrent,
        ), // ref_idx 2
    ]
}

/// Both neutrality fixtures the growth properties run over: the plain
/// `base_def` and the plasticity/introspection `plasticity_def`. Not the
/// full arbitrary-graph-def space; see the spec's TDD note on fixture scope.
fn fixtures() -> [(&'static str, CgpGraphBackendDef, Vec<InputReference>); 2] {
    [
        ("base", base_def(), base_input_refs()),
        ("plasticity", plasticity_def(), plasticity_input_refs()),
    ]
}

// ─── Fire-time neutrality: the three add-node forms ────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn add_disconnected_node_is_neutral_at_fire_time(seed in any::<u64>()) {
        for (label, parent, refs) in fixtures() {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            add_disconnected_node(&mut child, &mut rng).unwrap();
            assert_neutral(
                &format!("add_disconnected_node[{label}]"),
                &parent,
                &refs,
                &child,
                &refs,
                &ample_runtime_config(),
            );
        }
    }

    #[test]
    fn add_bootstrap_node_is_neutral_at_fire_time(seed in any::<u64>()) {
        for (label, parent, refs) in fixtures() {
            let config = MutationConfig::default();
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            add_bootstrap_node(&mut child, &refs, &config, &mut rng).unwrap();
            assert_neutral(
                &format!("add_bootstrap_node[{label}]"),
                &parent,
                &refs,
                &child,
                &refs,
                &ample_runtime_config(),
            );
        }
    }

    #[test]
    fn split_existing_edge_is_neutral_at_fire_time(seed in any::<u64>()) {
        for (label, parent, refs) in fixtures() {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            if split_existing_edge(&mut child, &refs, &mut rng).is_err() {
                continue;
            }
            assert_neutral(
                &format!("split_existing_edge[{label}]"),
                &parent,
                &refs,
                &child,
                &refs,
                &ample_runtime_config(),
            );
        }
    }

    #[test]
    fn copy_compute_node_is_neutral_at_fire_time(seed in any::<u64>()) {
        for (label, parent, refs) in fixtures() {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            copy_compute_node(&mut child, &mut rng).unwrap();
            assert_neutral(
                &format!("copy_compute_node[{label}]"),
                &parent,
                &refs,
                &child,
                &refs,
                &ample_runtime_config(),
            );
        }
    }

    #[test]
    fn copy_cgp_subgraph_is_neutral_at_fire_time(seed in any::<u64>()) {
        for (label, parent, refs) in fixtures() {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            copy_cgp_subgraph(&mut child, &mut rng).unwrap();
            assert_neutral(
                &format!("copy_cgp_subgraph[{label}]"),
                &parent,
                &refs,
                &child,
                &refs,
                &ample_runtime_config(),
            );
        }
    }

    /// Pushing a new input reference (unwired) never changes graph-backend
    /// behavior: no edge references the new index, and growing
    /// `input_refs` cannot invalidate an existing edge because every
    /// existing `InputLeaf.ref_idx` is already within the old, smaller
    /// bound.
    #[test]
    fn unwired_input_ref_add_is_neutral_on_graph_node(seed in any::<u64>()) {
        use crate::mutation::sampling::random_input_reference;

        for (label, parent, parent_refs) in fixtures() {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut child_refs = parent_refs.clone();
            child_refs.push(random_input_reference(&mut rng));
            assert_neutral(
                &format!("unwired_input_ref_add_graph[{label}]"),
                &parent,
                &parent_refs,
                &parent, // def itself is untouched by InputRef::Add
                &child_refs,
                &ample_runtime_config(),
            );
        }
    }
}

/// `InputRef::Add` on a VM node inserts no instruction (the auto-inserted
/// `ReadInput` is removed), so the VM program is byte-identical and its
/// execution is unaffected by the wider `input_refs` for the same reason as
/// the graph case: no instruction can reference an index that did not
/// exist before the push.
#[test]
fn unwired_input_ref_add_is_neutral_on_vm_node() {
    use crate::creature::genome::{VmBackendDef, VmInstruction};
    use crate::runtime::vm::execute_vm_node;

    let def = VmBackendDef {
        register_count: 2,
        constants: vec![],
        program: vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 3,
            },
            VmInstruction::Halt,
        ],
    };
    let parent_refs = base_input_refs();
    let mut child_refs = parent_refs.clone();
    child_refs.push(InputReference::UpstreamSlot(0));
    let config = ample_runtime_config();

    for sensors in scenarios() {
        let mut energy_before = 1.0e6f32;
        let mut mem_before = [0.0f32; 16];
        let prev_mem = [0.0f32; 16];
        let mut side_before = MeshSideOutputs::new(8);
        let result_before = execute_vm_node(
            &def,
            &parent_refs,
            &[0.0f32; OUTPUT_SLOT_COUNT],
            &mut energy_before,
            0.0,
            &mut mem_before,
            &prev_mem,
            &sensors,
            &config,
            &mut side_before,
        );

        let mut energy_after = 1.0e6f32;
        let mut mem_after = [0.0f32; 16];
        let mut side_after = MeshSideOutputs::new(8);
        let result_after = execute_vm_node(
            &def,
            &child_refs,
            &[0.0f32; OUTPUT_SLOT_COUNT],
            &mut energy_after,
            0.0,
            &mut mem_after,
            &prev_mem,
            &sensors,
            &config,
            &mut side_after,
        );

        assert_eq!(result_before.output_slots, result_after.output_slots);
        assert_eq!(result_before.terminal, result_after.terminal);
        assert_eq!(mem_before, mem_after);
        assert_eq!(
            side_before.action_queue.len(),
            side_after.action_queue.len()
        );
    }
}

// ─── Split preserves per-pass values ────────────────────────────────────────

/// Records every `(pass, node_index) -> output` value evaluated during
/// relaxation, so a split's insert-with-remap can be checked against the
/// exact per-pass trajectory rather than just the converged result.
#[derive(Default)]
struct RecordingTracer {
    pass: u32,
    // (pass, node_index) -> output
    records: Vec<(u32, usize, f32)>,
}

impl GraphTracer for RecordingTracer {
    fn on_pass_start(&mut self, pass: u32, _pass_cost: f32, _energy_after: f32) {
        self.pass = pass;
    }
    fn on_node_eval(
        &mut self,
        node_index: usize,
        _kind: &ComputeNodeKind,
        _weighted_inputs: &[f32],
        _weighted_sum: f32,
        _state_before: f32,
        _state_after: f32,
        output: f32,
    ) {
        self.records.push((self.pass, node_index, output));
    }
    fn on_pass_end(&mut self, _delta: f32) {}
    fn on_finish(&mut self, _curr_outputs: &[f32], _temporal_committed: bool) {}
    fn on_effects(&mut self, _effects: CgpEffectsTrace) {}
}

impl RecordingTracer {
    fn value_at(&self, pass: u32, node_index: usize) -> Option<f32> {
        self.records
            .iter()
            .find(|(p, n, _)| *p == pass && *n == node_index)
            .map(|(_, _, v)| *v)
    }
}

fn record(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    config: &RuntimeConfig,
    sensors: &SensorSnapshot,
) -> RecordingTracer {
    let mut tracer = RecordingTracer::default();
    let mut energy = 1.0e6f32;
    let mut shared_memory = [0.0f32; 16];
    let prev_shared_memory = [0.0f32; 16];
    let mut graph_runtime = GraphRuntimeState::new();
    let mut side_outputs = MeshSideOutputs::new(8);
    let _: NodeResult = execute_graph_impl(
        &mut tracer,
        def,
        input_refs,
        &[0.0f32; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0,
        &mut graph_runtime,
        sensors,
        config,
        &mut side_outputs,
        &mut shared_memory,
        &prev_shared_memory,
    );
    tracer
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// For every seed that produces a compute-consumer split (as opposed to
    /// a sink/action/execute-gate append), every original node's value on
    /// the visit survives at its shifted index: nodes below the insertion
    /// point are untouched, and nodes at or above it shift by one, reading
    /// their own inputs exactly as before (the new identity node adds one
    /// extra index nobody else depended on until this split targeted it).
    #[test]
    fn split_preserves_every_surviving_nodes_per_pass_value(seed in any::<u64>()) {
        // No node in the base graph is `Add`, so the split's inserted
        // identity node is unambiguous: the only `Add`-kind node afterward.
        let parent = base_def();
        prop_assert!(
            parent.compute_nodes.iter().all(|n| n.kind != ComputeNodeKind::Add),
            "base graph must not use Add so the split's new node is unambiguous"
        );
        let refs = base_input_refs();
        let mut child = parent.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        if split_existing_edge(&mut child, &refs, &mut rng).is_err() {
            return Ok(());
        }
        // Only compute-consumer splits insert-and-shift; sink/action/execute
        // splits append, leaving every original index unchanged.
        let inserted_idx = child
            .compute_nodes
            .iter()
            .position(|n| n.kind == ComputeNodeKind::Add);
        let Some(inserted_idx) = inserted_idx else {
            return Ok(());
        };
        let is_append = inserted_idx == parent.compute_nodes.len();
        let mapped = |old_idx: usize| -> usize {
            if is_append || old_idx < inserted_idx {
                old_idx
            } else {
                old_idx + 1
            }
        };

        let config = RuntimeConfig::default();
        for sensors in scenarios() {
            let parent_trace = record(&parent, &refs, &config, &sensors);
            let child_trace = record(&child, &refs, &config, &sensors);
            for old_idx in 0..parent.compute_nodes.len() {
                let expected = parent_trace.value_at(0, old_idx);
                let actual = child_trace.value_at(0, mapped(old_idx));
                prop_assert!(expected.is_some(), "seed {seed}: node {old_idx} was evaluated");
                prop_assert_eq!(
                    expected,
                    actual,
                    "seed {}, node {} (mapped {}): {:?} vs {:?}",
                    seed,
                    old_idx,
                    mapped(old_idx),
                    expected,
                    actual
                );
            }
        }
    }
}

// ─── sub_idx reach ──────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Over every seed, a drawn `InputLeaf.sub_idx` stays within its
    /// reference's actual width (never the pre-T11.F03 defect of only ever
    /// drawing index 0). This asserts range only; the reach claim — that
    /// every `sub_idx` in that range is actually drawn over enough seeds —
    /// is the seeded example test
    /// `random_graph_source_reaches_every_sub_value_of_a_compound_reference`
    /// in `operators.rs`.
    #[test]
    fn random_graph_source_sub_idx_is_within_reference_width(seed in any::<u64>()) {
        use crate::mutation::compound::sub_value_count;
        use crate::mutation::graph::operators::random_graph_source;

        let refs = vec![
            InputReference::World(WorldInputKey::NeighborBarrierRing),
            InputReference::ActionQueue,
        ];
        let config = MutationConfig::default();
        let mut rng = SmallRng::seed_from_u64(seed);
        if let GraphSource::InputLeaf { ref_idx, sub_idx } =
            random_graph_source(3, &refs, &config, &mut rng)
        {
            let width = sub_value_count(&refs[ref_idx as usize], &config);
            prop_assert!(
                sub_idx < width,
                "seed {}: sub_idx {} >= width {}",
                seed,
                sub_idx,
                width
            );
        }
    }
}

// ─── Raw-field mutation: exactly one field, never the variant ─────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Over `base_def`'s compute-node input edges (not the sink/action/
    /// execute-gate surfaces), `GraphRawFieldMutation` never replaces a
    /// `GraphSource` variant, and at most one edge changes at all. This
    /// covers "at most one edge, same variant" only; that the changed
    /// field moves by exactly one unit is example-tested separately in
    /// `operators.rs` (`raw_field_mutation_compute_node_index_moves_by_one_unit_inward`,
    /// `raw_field_mutation_shared_memory_slot_wraps_modulo_16`, and
    /// `raw_field_mutation_never_changes_more_than_one_field`).
    #[test]
    fn raw_field_mutation_edge_never_changes_variant(seed in any::<u64>()) {
        let refs = base_input_refs();
        let mut def = base_def();
        let before: Vec<GraphSource> = def
            .compute_nodes
            .iter()
            .flat_map(|n| n.inputs.iter().map(|e| e.source))
            .collect();
        let mut rng = SmallRng::seed_from_u64(seed);
        if raw_field_mutation(&mut def, &refs, &MutationConfig::default(), &mut rng)
            == Err(MutationSkipReason::NoApplicableTarget)
        {
            return Ok(());
        }
        let after: Vec<GraphSource> = def
            .compute_nodes
            .iter()
            .flat_map(|n| n.inputs.iter().map(|e| e.source))
            .collect();
        let diffs: usize = before
            .iter()
            .zip(after.iter())
            .filter(|(b, a)| b != a)
            .count();
        prop_assert!(diffs <= 1, "seed {}: more than one edge changed", seed);
        for (b, a) in before.iter().zip(after.iter()) {
            prop_assert_eq!(
                std::mem::discriminant(b),
                std::mem::discriminant(a),
                "seed {}: source variant must never be replaced",
                seed
            );
        }
    }
}
