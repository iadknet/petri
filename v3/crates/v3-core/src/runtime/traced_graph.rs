//! Traced graph execution — records per-pass trace data for the Execution Sampler.
//!
//! Uses [`RecordingTracer`] with the shared [`super::graph::execute_graph_impl`]
//! to avoid duplicating the relaxation loop. The tracer callbacks record
//! [`GraphNodeEvalTrace`] per node and [`GraphPassTrace`] per pass.

use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{GraphBackendDef, GraphNodeKind};
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
        kind: &GraphNodeKind,
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
    def: &GraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    node_idx: usize,
    graph_runtime: &mut GraphRuntimeState,
    sensors: &SensorSnapshot,
    config: &RuntimeConfig,
    side_outputs: &mut MeshSideOutputs,
) -> (NodeResult, GraphTrace) {
    let node_count = def.internal_nodes.len();

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
    );

    (result, tracer.into_trace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::creature::genome::{GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind};
    use crate::creature::state::GraphRuntimeState;
    use crate::runtime::graph::execute_graph_node;
    use crate::runtime::types::MeshSideOutputs;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn make_ss() -> SensorSnapshot {
        SensorSnapshot {
            local: StaticInputs {
                food_here: 0.0,
                neighbor_food: [0.0; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                generation: 0.0,
                age_ticks: 0.0,
            },
            perception: PerceptionSnapshot::zero(),
        }
    }

    /// Result equivalence: Constant + CustomOutput + RouterOutput.
    #[test]
    fn result_equivalence_constant_custom_router() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(2.0),
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 3.0,
                    }],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.5,
                    }],
                    plasticity: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let ss = make_ss();
        let config = default_config();

        let mut energy_a = 100.0f32;
        let mut gr_a = GraphRuntimeState::new();
        let result_a = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy_a,
            0.0,
            0,
            &mut gr_a,
            &ss,
            &config,
            &mut MeshSideOutputs::new(4),
        );

        let mut energy_b = 100.0f32;
        let mut gr_b = GraphRuntimeState::new();
        let (result_b, trace) = execute_graph_node_traced(
            &def,
            &[],
            &upstream,
            &mut energy_b,
            0.0,
            0,
            &mut gr_b,
            &ss,
            &config,
            &mut MeshSideOutputs::new(4),
        );

        assert_eq!(result_a, result_b);
        assert!(
            (energy_a - energy_b).abs() < 1e-6,
            "energy: {energy_a} vs {energy_b}"
        );
        assert_eq!(gr_a.node_state, gr_b.node_state);
        assert!(trace.converged);
        assert!(!trace.passes.is_empty());
    }

    /// DecayIntegrator state transitions captured across 2 calls.
    #[test]
    fn decay_integrator_state_captured() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::DecayIntegrator(0.5),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let ss = make_ss();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let mut energy = 1000.0f32;
        let mut gr = GraphRuntimeState::new();

        // Call 1: state 0.0 → 0.5
        let (r1, trace1) = execute_graph_node_traced(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &ss,
            &config,
            &mut MeshSideOutputs::new(4),
        );
        assert!(!r1.energy_exhausted);
        assert!((r1.output_slots[0] - 0.5).abs() < 1e-5);

        // Check trace captured state transition
        let decay_eval = &trace1.passes[0].node_evaluations[1]; // node 1 = DecayIntegrator
        assert_eq!(decay_eval.kind, "DecayIntegrator");
        assert!((decay_eval.state_before - 0.0).abs() < 1e-6);
        assert!((decay_eval.state_after - 0.5).abs() < 1e-6);

        // Call 2: state 0.5 → 0.75
        let (_r2, trace2) = execute_graph_node_traced(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &ss,
            &config,
            &mut MeshSideOutputs::new(4),
        );
        let decay_eval2 = &trace2.passes[0].node_evaluations[1];
        assert!((decay_eval2.state_before - 0.5).abs() < 1e-6);
        assert!((decay_eval2.state_after - 0.75).abs() < 1e-6);
    }

    /// Convergence status captured correctly.
    #[test]
    fn convergence_status_captured() {
        // A graph with only Constant nodes converges immediately
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let ss = make_ss();
        let config = default_config();

        let mut energy = 1000.0f32;
        let mut gr = GraphRuntimeState::new();

        let (_result, trace) = execute_graph_node_traced(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &ss,
            &config,
            &mut MeshSideOutputs::new(4),
        );

        assert!(trace.converged);
        assert!(trace.stable_passes_count >= config.graph_convergence_stable_passes);
    }

    /// Result equivalence with action-queue variants (PushAction + ExecuteActionQueue).
    #[test]
    fn result_equivalence_with_action_variants() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::PushAction(1), // Eat
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::ExecuteActionQueue,
                    inputs: vec![],
                    plasticity: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let ss = make_ss();
        let config = default_config();

        let mut energy_a = 100.0f32;
        let mut gr_a = GraphRuntimeState::new();
        let mut so_a = MeshSideOutputs::new(4);
        let result_a = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy_a,
            0.0,
            0,
            &mut gr_a,
            &ss,
            &config,
            &mut so_a,
        );

        let mut energy_b = 100.0f32;
        let mut gr_b = GraphRuntimeState::new();
        let mut so_b = MeshSideOutputs::new(4);
        let (result_b, trace) = execute_graph_node_traced(
            &def,
            &[],
            &upstream,
            &mut energy_b,
            0.0,
            0,
            &mut gr_b,
            &ss,
            &config,
            &mut so_b,
        );

        assert_eq!(result_a, result_b);
        assert!(result_a.terminal, "should be terminal");
        assert!(
            (energy_a - energy_b).abs() < 1e-6,
            "energy: {energy_a} vs {energy_b}"
        );
        assert!(trace.converged);

        // Both should have Eat queued
        let actions_a = so_a.action_queue.into_actions_or_noop();
        let actions_b = so_b.action_queue.into_actions_or_noop();
        assert_eq!(actions_a, actions_b);
        assert_eq!(
            actions_a,
            vec![crate::contracts::WorldAction::Eat],
            "both paths should queue Eat"
        );
    }
}
