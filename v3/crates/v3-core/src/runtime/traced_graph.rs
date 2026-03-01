//! Traced graph execution — identical logic to [`super::graph::execute_graph_node`]
//! but records per-pass trace data for the Execution Sampler.
//!
//! **Maintenance note:** This module reuses `evaluate_kind` and `collect_weighted_inputs`
//! from `graph.rs`. Only the outer relaxation loop is duplicated with trace recording.
//! When updating graph execution semantics, apply the same changes here and verify
//! with equivalence tests.

use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::GraphBackendDef;
use crate::creature::state::GraphRuntimeState;
use crate::runtime::graph::{collect_weighted_inputs, evaluate_kind, EvalCtx};
use crate::runtime::graph_effects::apply_graph_effects;
use crate::runtime::plasticity::hebbian;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::trace::{kind_label, GraphNodeEvalTrace, GraphPassTrace, GraphTrace};
use crate::runtime::types::{sanitize_f32, MeshSideOutputs, NodeResult};
use crate::sensors::perception::SensorSnapshot;

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

    let empty_trace = || GraphTrace {
        passes: Vec::new(),
        converged: false,
        stable_passes_count: 0,
        final_outputs: Vec::new(),
    };

    if node_count == 0 {
        return (NodeResult::halted(*upstream_slots, 0.0), empty_trace());
    }

    // Ensure node_state has enough slots for this node index.
    if graph_runtime.node_state.len() <= node_idx {
        graph_runtime.node_state.resize(node_idx + 1, Vec::new());
    }

    // Snapshot state for atomic rollback on energy exhaustion.
    let state_backup: Vec<f32> = graph_runtime.node_state[node_idx].clone();

    // Ensure the state Vec for this node is long enough.
    let state_vec = &mut graph_runtime.node_state[node_idx];
    if state_vec.len() < node_count {
        state_vec.resize(node_count, 0.0);
    }

    // Check if any node uses plasticity learning and prepare weights if so.
    let use_plasticity = hebbian::has_any_hebbian(def);
    if use_plasticity {
        hebbian::ensure_hebbian_weights(def, node_idx, &mut graph_runtime.plasticity_weights);
    }

    let max_passes = config.max_graph_relax_iters;
    let epsilon = config.graph_convergence_epsilon;
    let req_stable = config.graph_convergence_stable_passes;

    let mut prev_outputs = vec![0.0f32; node_count];
    let mut curr_outputs = vec![0.0f32; node_count];
    let mut stable_passes: u32 = 0;
    let mut w_inputs_buf: Vec<f32> = Vec::new();

    // Trace recording
    let mut trace_passes: Vec<GraphPassTrace> = Vec::with_capacity(max_passes as usize);

    for pass in 0..max_passes {
        let pass_cost = config.graph_node_base_cost * node_count as f32;
        *energy -= pass_cost;
        if *energy <= 0.0 {
            graph_runtime.node_state[node_idx] = state_backup;
            let trace = GraphTrace {
                passes: trace_passes,
                converged: false,
                stable_passes_count: stable_passes,
                final_outputs: curr_outputs,
            };
            return (NodeResult::exhausted(), trace);
        }

        let ctx = EvalCtx {
            input_refs,
            resolve: ResolveCtx {
                sensors,
                upstream_slots,
                energy: *energy,
                energy_consumed,
                action_queue: &side_outputs.action_queue,
            },
        };

        let mut node_evaluations: Vec<GraphNodeEvalTrace> = Vec::with_capacity(node_count);

        for current_idx in 0..node_count {
            let node = &def.internal_nodes[current_idx];

            // Use learned weights for plasticity nodes, genome weights otherwise.
            if use_plasticity && node.plasticity.is_some() {
                let learned = &graph_runtime.plasticity_weights[node_idx][current_idx];
                hebbian::collect_weighted_inputs_hebbian(
                    node,
                    current_idx,
                    node_count,
                    &prev_outputs,
                    &curr_outputs,
                    learned,
                    &mut w_inputs_buf,
                );
            } else {
                collect_weighted_inputs(
                    node,
                    current_idx,
                    node_count,
                    &prev_outputs,
                    &curr_outputs,
                    &mut w_inputs_buf,
                );
            }
            let wsum: f32 = w_inputs_buf.iter().sum();

            let mut node_state = graph_runtime.node_state[node_idx][current_idx];
            let state_before = node_state;

            curr_outputs[current_idx] = sanitize_f32(evaluate_kind(
                &node.kind,
                &w_inputs_buf,
                wsum,
                &ctx,
                &mut node_state,
            ));

            graph_runtime.node_state[node_idx][current_idx] = node_state;

            node_evaluations.push(GraphNodeEvalTrace {
                node_index: current_idx,
                kind: kind_label(&node.kind),
                weighted_inputs: w_inputs_buf.clone(),
                weighted_sum: wsum,
                state_before,
                state_after: node_state,
                output: curr_outputs[current_idx],
            });
        }

        let delta = prev_outputs
            .iter()
            .zip(curr_outputs.iter())
            .map(|(p, c)| (c - p).abs())
            .fold(0.0f32, f32::max);

        prev_outputs.clone_from(&curr_outputs);

        if delta <= epsilon {
            stable_passes += 1;
        } else {
            stable_passes = 0;
        }

        trace_passes.push(GraphPassTrace {
            pass_index: pass,
            energy_cost: pass_cost,
            energy_after: *energy,
            node_evaluations,
            max_delta: delta,
        });

        if stable_passes >= req_stable {
            break;
        }
    }

    // Apply plasticity weight updates after convergence.
    // NOTE: If energy goes negative here, we do not roll back weight changes or
    // return exhausted. Acceptable while plasticity_update_cost defaults to 0.0.
    // When a nonzero cost is introduced, add exhaustion handling here.
    if use_plasticity {
        let plasticity_cost = hebbian::apply_hebbian_updates(
            def,
            node_idx,
            &mut graph_runtime.plasticity_weights,
            &curr_outputs,
            config.plasticity_update_cost,
        );
        *energy -= plasticity_cost;
    }

    // Build NodeResult via the shared 3-phase effect pass.
    let result = apply_graph_effects(def, &curr_outputs, upstream_slots, side_outputs);

    let converged = stable_passes >= req_stable;
    let trace = GraphTrace {
        passes: trace_passes,
        converged,
        stable_passes_count: stable_passes,
        final_outputs: curr_outputs,
    };

    (result, trace)
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
