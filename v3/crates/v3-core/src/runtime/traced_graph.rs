//! Traced graph execution — records per-pass trace data for the Execution Sampler.
//!
//! Uses [`RecordingTracer`] with the shared [`super::graph::execute_graph_impl`]
//! to avoid duplicating the relaxation loop. The tracer callbacks record
//! [`GraphNodeEvalTrace`] per node and [`GraphPassTrace`] per pass.

use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::graph::{execute_graph_impl, GraphTracer};
use crate::runtime::trace::{kind_label, GraphNodeEvalTrace, GraphPassTrace, GraphTrace};
use crate::runtime::types::{MeshSideOutputs, NodeResult};
use crate::sensors::perception::SensorSnapshot;

// ─── RecordingTracer ─────────────────────────────────────────────────────────

/// Tracer that records per-pass trace data for the Execution Sampler.
///
/// Captures [`GraphNodeEvalTrace`] per node and [`GraphPassTrace`] per pass,
/// assembled into a [`GraphTrace`] via [`into_trace`](RecordingTracer::into_trace).
pub(crate) struct RecordingTracer {
    passes: Vec<GraphPassTrace>,
    current_pass_evals: Vec<GraphNodeEvalTrace>,
    current_pass_cost: f32,
    current_pass_energy: f32,
    // Captured by on_finish — avoids fragile reconstruction from trace data.
    final_outputs: Vec<f32>,
    final_stable_passes: u32,
    final_converged: bool,
}

impl RecordingTracer {
    pub(crate) fn new(max_passes: u32, node_count: usize) -> Self {
        Self {
            passes: Vec::with_capacity(max_passes as usize),
            current_pass_evals: Vec::with_capacity(node_count),
            current_pass_cost: 0.0,
            current_pass_energy: 0.0,
            final_outputs: Vec::new(),
            final_stable_passes: 0,
            final_converged: false,
        }
    }

    /// Consume the tracer and build the final [`GraphTrace`].
    pub(crate) fn into_trace(self) -> GraphTrace {
        GraphTrace {
            passes: self.passes,
            converged: self.final_converged,
            stable_passes_count: self.final_stable_passes,
            final_outputs: self.final_outputs,
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

    fn on_finish(&mut self, curr_outputs: &[f32], stable_passes: u32, converged: bool) {
        self.final_outputs = curr_outputs.to_vec();
        self.final_stable_passes = stable_passes;
        self.final_converged = converged;
    }
}

// ─── Public entry point ──────────────────────────────────────────────────────

/// Execute a graph-backend mesh node with trace recording.
///
/// Identical behavior to [`super::graph::execute_graph_node`] but additionally
/// returns a [`GraphTrace`] capturing per-pass node evaluations, convergence
/// status, and final outputs.
#[allow(clippy::too_many_arguments)]
pub fn execute_graph_node_traced(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
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
            stable_passes_count: 0,
            final_outputs: Vec::new(),
        };
        return (NodeResult::halted(*upstream_slots, 0.0), trace);
    }

    let mut tracer = RecordingTracer::new(config.max_graph_relax_iters, node_count);

    let result = execute_graph_impl(
        &mut tracer,
        def,
        input_refs,
        upstream_slots,
        energy,
        energy_consumed,
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

// Old traced_graph tests removed — CGP graph tests live in runtime/cgp_graph.rs.
