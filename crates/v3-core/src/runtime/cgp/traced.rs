use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::effects::CgpEffectsTrace;
use crate::runtime::cgp::execute::{execute_graph_impl, GraphTracer};
use crate::runtime::trace::domain::{
    kind_label, GraphActionSlotTrace, GraphExecuteGateTrace, GraphNodeEvalTrace,
    GraphOutputSinkTrace, GraphPassTrace, GraphTrace,
};
use crate::runtime::types::{MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::sensors::perception::SensorSnapshot;

pub(crate) struct RecordingTracer {
    passes: Vec<GraphPassTrace>,
    current_pass_evals: Vec<GraphNodeEvalTrace>,
    current_pass_cost: f32,
    current_pass_energy: f32,
    final_outputs: Vec<f32>,
    temporal_committed: bool,
    output_sinks: Vec<GraphOutputSinkTrace>,
    action_slots: Vec<GraphActionSlotTrace>,
    execute_gate: GraphExecuteGateTrace,
}

impl RecordingTracer {
    pub(crate) fn new(max_passes: u32, node_count: usize) -> Self {
        Self {
            passes: Vec::with_capacity(max_passes as usize),
            current_pass_evals: Vec::with_capacity(node_count),
            current_pass_cost: 0.0,
            current_pass_energy: 0.0,
            final_outputs: Vec::new(),
            temporal_committed: false,
            output_sinks: Vec::new(),
            action_slots: Vec::new(),
            execute_gate: GraphExecuteGateTrace {
                wired: false,
                weighted_sum: 0.0,
                queue_non_empty: false,
                fired: false,
            },
        }
    }

    pub(crate) fn into_trace(self) -> GraphTrace {
        GraphTrace {
            passes: self.passes,
            converged: false,
            temporal_committed: self.temporal_committed,
            stable_passes_count: 0,
            final_outputs: self.final_outputs,
            output_sinks: self.output_sinks,
            action_slots: self.action_slots,
            execute_gate: self.execute_gate,
        }
    }
}

impl GraphTracer for RecordingTracer {
    fn on_pass_start(&mut self, _pass: u32, pass_cost: f32, energy_after: f32) {
        self.current_pass_cost = pass_cost;
        self.current_pass_energy = energy_after;
        self.current_pass_evals.clear();
    }

    fn on_node_eval(
        &mut self,
        node_index: usize,
        kind: &ComputeNodeKind,
        weighted_inputs: &[f32],
        weighted_sum: f32,
        state_before: f32,
        state_after: f32,
        output: f32,
    ) {
        self.current_pass_evals.push(GraphNodeEvalTrace {
            node_index,
            kind: kind_label(kind),
            weighted_inputs: weighted_inputs.to_vec(),
            weighted_sum,
            state_before,
            state_after,
            output,
        });
    }

    fn on_pass_end(&mut self, delta: f32) {
        self.passes.push(GraphPassTrace {
            pass_index: self.passes.len() as u32,
            energy_cost: self.current_pass_cost,
            energy_after: self.current_pass_energy,
            node_evaluations: std::mem::take(&mut self.current_pass_evals),
            max_delta: delta,
        });
    }

    fn on_finish(&mut self, curr_outputs: &[f32], temporal_committed: bool) {
        self.final_outputs = curr_outputs.to_vec();
        self.temporal_committed = temporal_committed;
    }

    fn on_effects(&mut self, effects: CgpEffectsTrace) {
        self.output_sinks = effects.output_sinks;
        self.action_slots = effects.action_slots;
        self.execute_gate = effects.execute_gate;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_graph_node_traced_with_reserve(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    energy: &mut f32,
    energy_consumed: f32,
    reproductive_reserve: f32,
    node_idx: usize,
    graph_runtime: &mut GraphRuntimeState,
    sensors: &SensorSnapshot,
    config: &RuntimeConfig,
    side_outputs: &mut MeshSideOutputs,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
) -> (NodeResult, GraphTrace) {
    let node_count = def.compute_nodes.len();

    if node_count == 0 {
        let trace = GraphTrace {
            passes: Vec::new(),
            converged: false,
            temporal_committed: false,
            stable_passes_count: 0,
            final_outputs: Vec::new(),
            output_sinks: Vec::new(),
            action_slots: Vec::new(),
            execute_gate: GraphExecuteGateTrace {
                wired: false,
                weighted_sum: 0.0,
                queue_non_empty: false,
                fired: false,
            },
        };
        return (
            NodeResult::halted(
                *upstream_slots,
                crate::runtime::routing::RouteGateMap::default(),
            ),
            trace,
        );
    }

    let mut tracer = RecordingTracer::new(1, node_count);

    let result = execute_graph_impl(
        &mut tracer,
        def,
        input_refs,
        upstream_slots,
        energy,
        energy_consumed,
        reproductive_reserve,
        node_idx,
        graph_runtime,
        sensors,
        config,
        side_outputs,
        shared_memory,
        prev_shared_memory,
    );

    (result, tracer.into_trace())
}
