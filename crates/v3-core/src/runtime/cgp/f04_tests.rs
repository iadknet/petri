//! T13.F04 — direct graph effect activation.
//!
//! A Graph module whose sink, router gate, memory sink, action slot or execute
//! gate carries an edge from an input leaf or shared memory applies that effect
//! even with no compute node, under the nonzero-compute ordering, charge and
//! exhaustion rules. A zero-compute graph with no wired surface stays inert.

use crate::config::{MutationConfig, RuntimeConfig};
use crate::contracts::{
    Direction, InputReference, NodeId, RouteTarget, StaticIntrospectionKey, WorldAction,
};
use crate::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    OutputSinkKind, WorldActionKind,
};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute::execute_graph_node;
use crate::runtime::cgp::traced::execute_graph_node_traced;
use crate::runtime::mesh::execute_creature_mesh;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{MeshOutput, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;
use proptest::prelude::*;

/// The single input leaf every fixture reads resolves to this constant.
const LEAF_VALUE: f32 = 3.0;
const BASE_COST: f32 = 0.25;

fn sensors() -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: LEAF_VALUE,
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

fn input_refs() -> Vec<InputReference> {
    vec![InputReference::StaticIntrospection(
        StaticIntrospectionKey::AgeTicks,
    )]
}

fn config() -> RuntimeConfig {
    RuntimeConfig {
        graph_node_base_cost: BASE_COST,
        ..RuntimeConfig::default()
    }
}

/// A T11.F18 blank Graph detour: the full fixed output catalog, all unwired.
fn blank() -> CgpGraphBackendDef {
    CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default())
}

fn leaf(weight: f32) -> GraphEdge {
    GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx: 0,
        },
        weight,
    }
}

fn memory(slot: u8, weight: f32) -> GraphEdge {
    GraphEdge {
        source: GraphSource::SharedMemory {
            slot,
            previous: false,
        },
        weight,
    }
}

fn wire(def: &mut CgpGraphBackendDef, kind: OutputSinkKind, edge: GraphEdge) {
    let sink = def
        .output_sinks
        .iter_mut()
        .find(|sink| sink.kind == kind)
        .expect("fixed output catalog contains every sink kind");
    sink.inputs.push(edge);
}

struct Visit {
    result: NodeResult,
    memory: [f32; 16],
    energy: f32,
    side: MeshSideOutputs,
}

fn visit_with(
    def: &CgpGraphBackendDef,
    memory_in: [f32; 16],
    energy_in: f32,
    upstream: [f32; OUTPUT_SLOT_COUNT],
) -> Visit {
    let mut memory = memory_in;
    let mut energy = energy_in;
    let mut side = MeshSideOutputs::new(4);
    let mut state = GraphRuntimeState::new();
    state.begin_tick(&[], 0);
    let result = execute_graph_node(
        def,
        &input_refs(),
        &upstream,
        &mut energy,
        0.0,
        0,
        &mut state,
        &sensors(),
        &config(),
        &mut side,
        &mut memory,
        &[0.0; 16],
    );
    Visit {
        result,
        memory,
        energy,
        side,
    }
}

fn graph_node(id: u32, def: CgpGraphBackendDef, targets: &[(u32, u8)]) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: input_refs(),
        backend_def: BackendDef::Graph(def),
        targets: targets
            .iter()
            .map(|(target, slot)| RouteTarget {
                target_id: NodeId::new(*target),
                slot: *slot,
                gate_bias: 0.0,
            })
            .collect(),
    }
}

fn run_mesh(
    nodes: Vec<NodeGenome>,
    memory: &mut [f32; 16],
    energy: &mut f32,
    config: &RuntimeConfig,
) -> MeshOutput {
    let genome = CreatureGenome {
        entry_node_id: nodes[0].node_id,
        nodes,
    };
    let mut state = GraphRuntimeState::new();
    state.begin_tick(&genome.nodes, 0);
    execute_creature_mesh(
        &genome,
        &sensors(),
        energy,
        memory,
        &[0.0; 16],
        &mut state,
        config,
    )
}

// ── Direct effect fixtures ──────────────────────────────────────────────────

#[test]
fn input_leaf_writes_custom_output_slot_and_pays_one_node_equivalent() {
    let mut def = blank();
    wire(&mut def, OutputSinkKind::CustomOutput(2), leaf(2.0));
    let mut upstream = [0.0; OUTPUT_SLOT_COUNT];
    upstream[5] = 7.0;

    let visited = visit_with(&def, [0.0; 16], 100.0, upstream);

    assert_eq!(visited.result.output_slots[2], 2.0 * LEAF_VALUE);
    assert_eq!(visited.result.output_slots[5], 7.0);
    assert!(!visited.result.terminal);
    assert_eq!(visited.energy, 100.0 - BASE_COST);
    assert_eq!(
        visited.side.energy_observation.graph_compute,
        f64::from(BASE_COST)
    );
    assert_eq!(visited.side.work_counters.graph_relax_iters, 1);
}

#[test]
fn input_leaf_writes_and_clears_shared_memory_slots() {
    let mut def = blank();
    wire(&mut def, OutputSinkKind::WriteSlot(4), leaf(1.0));
    wire(&mut def, OutputSinkKind::ClearSlot(5), leaf(1.0));
    let mut memory_in = [0.0; 16];
    memory_in[5] = 9.0;

    let visited = visit_with(&def, memory_in, 100.0, [0.0; OUTPUT_SLOT_COUNT]);

    assert_eq!(visited.memory[4], LEAF_VALUE);
    assert_eq!(visited.memory[5], 0.0);
    assert_eq!(visited.side.work_counters.shared_memory_writes_changed, 2);
}

#[test]
fn shared_memory_router_gate_steers_a_two_target_route() {
    for (gate_input, taken_slot, skipped_slot) in
        [(1.0, 11usize, 10usize), (-1.0, 10usize, 11usize)]
    {
        let mut router = blank();
        wire(&mut router, OutputSinkKind::RouterGate(1), memory(0, 1.0));
        let mut first = blank();
        wire(&mut first, OutputSinkKind::WriteSlot(10), leaf(1.0));
        let mut second = blank();
        wire(&mut second, OutputSinkKind::WriteSlot(11), leaf(1.0));

        let mut memory_in = [0.0; 16];
        memory_in[0] = gate_input;
        let mut energy = 100.0;
        let _ = run_mesh(
            vec![
                graph_node(0, router, &[(1, 0), (2, 1)]),
                graph_node(1, first, &[]),
                graph_node(2, second, &[]),
            ],
            &mut memory_in,
            &mut energy,
            &config(),
        );

        assert_eq!(memory_in[taken_slot], LEAF_VALUE);
        assert_eq!(memory_in[skipped_slot], 0.0);
    }
}

#[test]
fn memory_gate_and_leaf_param_emit_move_through_a_wired_execute_gate() {
    let mut def = blank();
    def.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Move);
    def.action_bank[0].gate_inputs.push(memory(0, 1.0));
    def.action_bank[0].param_inputs.push(leaf(1.0));
    def.execute_gate.inputs.push(memory(0, 1.0));
    let mut memory_in = [0.0; 16];
    memory_in[0] = 1.0;

    let visited = visit_with(&def, memory_in, 100.0, [0.0; OUTPUT_SLOT_COUNT]);

    assert!(visited.result.terminal);
    assert_eq!(
        visited.side.action_queue.into_actions_or_noop(),
        vec![WorldAction::Move(Direction::ALL[LEAF_VALUE as usize])]
    );
    assert_eq!(visited.energy, 100.0 - BASE_COST);
}

/// Either action-slot edge list alone activates the slot's surface: a gate edge
/// with no param edge, and a param edge with no gate edge, each enter the visit
/// and pay its one node-equivalent charge.
#[test]
fn action_slot_enters_a_visit_on_a_gate_edge_or_a_param_edge_alone() {
    for (label, gate_only) in [("gate edge only", true), ("param edge only", false)] {
        let mut def = blank();
        def.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Move);
        if gate_only {
            def.action_bank[0].gate_inputs.push(leaf(1.0));
        } else {
            def.action_bank[0].param_inputs.push(leaf(1.0));
        }
        assert!(def.enters_visit(), "{label}");

        let visited = visit_with(&def, [0.0; 16], 100.0, [0.0; OUTPUT_SLOT_COUNT]);

        assert_eq!(visited.side.work_counters.graph_relax_iters, 1, "{label}");
        assert_eq!(visited.energy, 100.0 - BASE_COST, "{label}");
        assert_eq!(
            visited.side.energy_observation.graph_compute,
            f64::from(BASE_COST),
            "{label}"
        );
    }
}

#[test]
fn wired_blank_detour_forwards_its_bus_and_executes_its_successor() {
    let mut detour = blank();
    wire(&mut detour, OutputSinkKind::CustomOutput(0), leaf(1.0));
    let mut successor = blank();
    successor.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Move);
    successor.action_bank[0].gate_inputs.push(leaf(1.0));
    successor.action_bank[0].param_inputs.push(leaf(1.0));
    successor.execute_gate.inputs.push(leaf(1.0));

    let mut memory_in = [0.0; 16];
    let mut energy = 100.0;
    let output = run_mesh(
        vec![
            graph_node(0, detour, &[(1, 0)]),
            graph_node(1, successor, &[]),
        ],
        &mut memory_in,
        &mut energy,
        &config(),
    );

    assert_eq!(
        output.actions,
        vec![WorldAction::Move(Direction::ALL[LEAF_VALUE as usize])]
    );
    assert_eq!(output.termination_reason, TerminationReason::ActionEmitted);
    assert_eq!(output.work_counters.mesh_hops, 2);
    assert_eq!(output.work_counters.graph_relax_iters, 2);
    assert_eq!(energy, 100.0 - 2.0 * BASE_COST);
}

#[test]
fn unwired_blank_visit_is_free_and_passes_its_bus_through() {
    let mut upstream = [0.0; OUTPUT_SLOT_COUNT];
    upstream[3] = 4.0;
    let mut memory_in = [0.0; 16];
    memory_in[0] = 2.0;

    let visited = visit_with(&blank(), memory_in, 100.0, upstream);

    assert_eq!(visited.result.output_slots, upstream);
    assert_eq!(visited.result.route_gates.scores, [0.0; 8]);
    assert!(!visited.result.terminal);
    assert_eq!(visited.memory, memory_in);
    assert_eq!(visited.energy, 100.0);
    assert_eq!(visited.side.work_counters.graph_relax_iters, 0);
    assert_eq!(visited.side.energy_observation.graph_compute, 0.0);
    assert!(visited.side.action_queue.is_empty());
}

/// One visit per tick steps the integrator once per tick; a second visit in
/// the same tick steps it again from the first visit's commit (T19.F02).
#[test]
fn stateful_module_and_blank_neighbor_each_step_once_per_visit() {
    let mut stateful = blank();
    stateful.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::DecayIntegrator(0.5),
        inputs: vec![leaf(1.0)],
        plasticity: None,
    });
    wire(
        &mut stateful,
        OutputSinkKind::WriteSlot(1),
        GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        },
    );
    let mut wired_blank = blank();
    wire(&mut wired_blank, OutputSinkKind::WriteSlot(2), leaf(1.0));

    for (blank_def, per_tick_cost, neighbor_write) in [
        (blank(), BASE_COST, 0.0),
        (wired_blank, 2.0 * BASE_COST, LEAF_VALUE),
    ] {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                graph_node(0, stateful.clone(), &[(1, 0)]),
                graph_node(1, blank_def, &[]),
            ],
        };
        let mut state = GraphRuntimeState::new();
        let mut memory_in = [0.0; 16];
        let mut energy = 100.0;
        for expected in [1.5, 2.25] {
            state.begin_tick(&genome.nodes, 0);
            let _ = execute_creature_mesh(
                &genome,
                &sensors(),
                &mut energy,
                &mut memory_in,
                &[0.0; 16],
                &mut state,
                &config(),
            );
            assert!((memory_in[1] - expected).abs() < 1e-6);
            assert_eq!(memory_in[2], neighbor_write);
        }
        assert!((energy - (100.0 - 2.0 * per_tick_cost)).abs() < 1e-6);
        // A revisit within the same tick (no `begin_tick`) advances again.
        let _ = execute_creature_mesh(
            &genome,
            &sensors(),
            &mut energy,
            &mut memory_in,
            &[0.0; 16],
            &mut state,
            &config(),
        );
        assert!((memory_in[1] - 2.625).abs() < 1e-6);
        assert!((energy - (100.0 - 3.0 * per_tick_cost)).abs() < 1e-6);
    }
}

#[test]
fn traced_zero_compute_visit_reports_its_wired_sink_and_gate() {
    let mut def = blank();
    wire(&mut def, OutputSinkKind::WriteSlot(4), leaf(1.0));
    def.execute_gate.inputs.push(leaf(1.0));

    let mut memory_in = [0.0; 16];
    let mut energy = 100.0;
    let mut side = MeshSideOutputs::new(4);
    let mut state = GraphRuntimeState::new();
    state.begin_tick(&[], 0);
    let (result, trace) = execute_graph_node_traced(
        &def,
        &input_refs(),
        &[0.0; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0,
        &mut state,
        &sensors(),
        &config(),
        &mut side,
        &mut memory_in,
        &[0.0; 16],
    );

    assert_eq!(memory_in[4], LEAF_VALUE);
    assert!(!result.terminal, "the action queue is empty");
    assert!(trace.temporal_committed);
    assert!(trace.final_outputs.is_empty());
    assert_eq!(trace.passes.len(), 1);
    assert_eq!(trace.passes[0].energy_cost, BASE_COST);
    assert!(trace.passes[0].node_evaluations.is_empty());
    let wired: Vec<_> = trace
        .output_sinks
        .iter()
        .filter(|sink| sink.wired)
        .collect();
    assert_eq!(wired.len(), 1);
    assert_eq!(wired[0].weighted_sum, LEAF_VALUE);
    assert!(wired[0].applied);
    assert!(trace.execute_gate.wired);
    assert_eq!(trace.execute_gate.weighted_sum, LEAF_VALUE);
    assert!(!trace.execute_gate.fired);
}

#[test]
fn wired_detour_chain_terminates_on_the_hop_budget() {
    let chain: Vec<NodeGenome> = (0..4)
        .map(|id| {
            let mut def = blank();
            wire(&mut def, OutputSinkKind::WriteSlot(id as u8), leaf(1.0));
            graph_node(id, def, &[(id + 1, 0)])
        })
        .collect();
    let runtime = RuntimeConfig {
        max_mesh_hops: 2,
        ..config()
    };

    let mut memory_in = [0.0; 16];
    let mut energy = 100.0;
    let output = run_mesh(chain, &mut memory_in, &mut energy, &runtime);

    assert_eq!(output.termination_reason, TerminationReason::MaxHopsReached);
    assert_eq!(output.work_counters.mesh_hops, 2);
    assert_eq!(output.work_counters.graph_relax_iters, 2);
    assert_eq!(energy, 100.0 - 2.0 * BASE_COST);
    assert_eq!(memory_in[0], LEAF_VALUE);
    assert_eq!(memory_in[1], LEAF_VALUE);
    assert_eq!(memory_in[2], 0.0);
}

// ── Properties ──────────────────────────────────────────────────────────────

/// Any wired surface built from the drawn parameters, on a zero-compute def.
fn wired_def(surface: u8, source: u8, slot: u8, weight: f32) -> CgpGraphBackendDef {
    let edge = GraphEdge {
        source: match source {
            0 => GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx: 0,
            },
            1 => GraphSource::SharedMemory {
                slot: slot % 16,
                previous: false,
            },
            // `slot % 2` draws both the dummy node's own index and an
            // out-of-range one; both resolve to 0.0 in either arm.
            _ => GraphSource::ComputeNode(u16::from(slot % 2)),
        },
        weight,
    };
    let mut def = blank();
    match surface {
        0 => wire(&mut def, OutputSinkKind::CustomOutput(slot % 24), edge),
        1 => wire(&mut def, OutputSinkKind::RouterGate(slot % 8), edge),
        2 => wire(&mut def, OutputSinkKind::WriteSlot(slot % 16), edge),
        3 => wire(&mut def, OutputSinkKind::ClearSlot(slot % 16), edge),
        4 => {
            def.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Move);
            def.action_bank[0].gate_inputs.push(edge);
            def.action_bank[0].param_inputs.push(leaf(1.0));
        }
        5 => {
            def.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Eat);
            def.action_bank[0].param_inputs.push(edge);
            def.action_bank[0].gate_inputs.push(leaf(1.0));
        }
        _ => def.execute_gate.inputs.push(edge),
    }
    def
}

proptest! {
    /// Dummy neutrality: appending one edgeless `Constant(0.0)` compute node
    /// changes nothing a visit produces or charges.
    #[test]
    fn wired_zero_compute_visit_matches_the_same_def_with_a_dummy_node(
        surface in 0..7u8, source in 0..3u8, slot in 0..24u8,
        weight in -4.0f32..4.0, memory_value in -4.0f32..4.0,
        energy in 0.1f32..4.0,
    ) {
        let def = wired_def(surface, source, slot, weight);
        let mut with_dummy = def.clone();
        with_dummy.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(0.0),
            inputs: vec![],
            plasticity: None,
        });
        let mut memory_in = [0.0; 16];
        memory_in[(slot % 16) as usize] = memory_value;
        let upstream = [1.0; OUTPUT_SLOT_COUNT];

        let plain = visit_with(&def, memory_in, energy, upstream);
        let dummied = visit_with(&with_dummy, memory_in, energy, upstream);

        prop_assert_eq!(plain.result.output_slots, dummied.result.output_slots);
        prop_assert_eq!(plain.result.route_gates, dummied.result.route_gates);
        prop_assert_eq!(plain.result.terminal, dummied.result.terminal);
        prop_assert_eq!(plain.result.energy_exhausted, dummied.result.energy_exhausted);
        prop_assert_eq!(plain.memory, dummied.memory);
        prop_assert_eq!(plain.energy, dummied.energy);
        prop_assert_eq!(
            plain.side.action_queue.into_actions_or_noop(),
            dummied.side.action_queue.into_actions_or_noop()
        );
        prop_assert_eq!(plain.side.work_counters, dummied.side.work_counters);
        prop_assert_eq!(
            plain.side.energy_observation.graph_compute,
            dummied.side.energy_observation.graph_compute
        );
    }

    /// Blank identity: an unwired zero-compute visit is inert.
    #[test]
    fn unwired_zero_compute_visit_charges_counts_and_writes_nothing(
        memory_value in -4.0f32..4.0, energy in 0.0f32..100.0, upstream_value in -4.0f32..4.0,
    ) {
        let memory_in = [memory_value; 16];
        let upstream = [upstream_value; OUTPUT_SLOT_COUNT];

        let visited = visit_with(&blank(), memory_in, energy, upstream);

        prop_assert_eq!(visited.result.output_slots, upstream);
        prop_assert_eq!(visited.result.route_gates.scores, [0.0; 8]);
        prop_assert!(!visited.result.terminal);
        prop_assert_eq!(visited.memory, memory_in);
        prop_assert_eq!(visited.energy, energy);
        prop_assert_eq!(visited.side.work_counters.graph_relax_iters, 0);
        prop_assert_eq!(visited.side.energy_observation.graph_compute, 0.0);
        prop_assert!(visited.side.action_queue.is_empty());
    }

    /// Exhaustion: energy at most the entry charge exhausts before any effect.
    #[test]
    fn unaffordable_wired_zero_compute_visit_exhausts_without_effects(
        surface in 0..7u8, source in 0..3u8, slot in 0..24u8,
        weight in -4.0f32..4.0, energy in -1.0f32..BASE_COST,
    ) {
        let def = wired_def(surface, source, slot, weight);
        let mut memory_in = [0.0; 16];
        memory_in[(slot % 16) as usize] = 2.0;

        let visited = visit_with(&def, memory_in, energy, [0.0; OUTPUT_SLOT_COUNT]);

        prop_assert!(visited.result.energy_exhausted);
        prop_assert_eq!(visited.memory, memory_in);
        prop_assert!(visited.side.action_queue.is_empty());
        prop_assert_eq!(visited.energy, energy - BASE_COST);
        prop_assert_eq!(visited.side.work_counters.graph_relax_iters, 1);
        prop_assert_eq!(
            visited.side.energy_observation.graph_compute,
            f64::from(energy) - f64::from(energy - BASE_COST)
        );
    }
}
