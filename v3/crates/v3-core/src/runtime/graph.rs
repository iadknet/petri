use std::collections::HashMap;

use crate::config::RuntimeConfig;
use crate::contracts::{InputReference, NodeId};
use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphNodeKind};
use crate::runtime::inputs::resolve_input;
use crate::runtime::types::NodeResult;
use crate::sensors::static_inputs::StaticInputs;

/// Immutable context for resolving `InputRef` nodes during graph evaluation.
struct EvalCtx<'a> {
    input_refs: &'a [InputReference],
    upstream_slots: &'a [f32; 12],
    energy: f32,
    energy_consumed: f32,
    static_inputs: &'a StaticInputs,
}

/// Evaluate one internal graph node's kind, returning the scalar output.
///
/// `w_inputs` holds the per-edge weighted values (source_value * weight).
/// `wsum` is `w_inputs.iter().sum()`.
/// `state` is the node's mutable persistent scalar state (for stateful operators).
#[inline]
fn evaluate_kind(
    kind: &GraphNodeKind,
    w_inputs: &[f32],
    wsum: f32,
    ctx: &EvalCtx<'_>,
    state: &mut f32,
) -> f32 {
    match kind {
        GraphNodeKind::InputRef(u) => {
            let resolved = if (*u as usize) < ctx.input_refs.len() {
                resolve_input(
                    &ctx.input_refs[*u as usize],
                    ctx.static_inputs,
                    ctx.upstream_slots,
                    ctx.energy,
                    ctx.energy_consumed,
                )
            } else {
                0.0
            };
            resolved + wsum
        }
        GraphNodeKind::Constant(f) => *f,
        GraphNodeKind::Add => wsum,
        GraphNodeKind::Multiply => w_inputs.iter().copied().product::<f32>(),
        GraphNodeKind::Negate => -wsum,
        GraphNodeKind::Abs => wsum.abs(),
        GraphNodeKind::Min => w_inputs.iter().copied().reduce(f32::min).unwrap_or(0.0),
        GraphNodeKind::Max => w_inputs.iter().copied().reduce(f32::max).unwrap_or(0.0),
        GraphNodeKind::Threshold(t) => {
            if wsum > *t {
                1.0
            } else {
                0.0
            }
        }
        GraphNodeKind::GreaterThan => {
            let a = w_inputs.first().copied().unwrap_or(0.0);
            let b = w_inputs.get(1).copied().unwrap_or(0.0);
            if a > b {
                1.0
            } else {
                0.0
            }
        }
        GraphNodeKind::Sigmoid => 1.0 / (1.0 + (-wsum).exp()),
        GraphNodeKind::Tanh => wsum.tanh(),
        GraphNodeKind::Relu => wsum.max(0.0),
        GraphNodeKind::Select => {
            let cond = w_inputs.first().copied().unwrap_or(0.0);
            if cond >= 0.5 {
                w_inputs.get(1).copied().unwrap_or(0.0)
            } else {
                w_inputs.get(2).copied().unwrap_or(0.0)
            }
        }
        GraphNodeKind::Clamp01 => wsum.clamp(0.0, 1.0),
        GraphNodeKind::WeightedSum => wsum,
        GraphNodeKind::DecayIntegrator(a) => {
            let a_c = a.clamp(0.0, 1.0);
            *state = (1.0 - a_c) * *state + a_c * wsum;
            *state
        }
        GraphNodeKind::Momentum(b) => {
            let b_c = b.clamp(0.0, 1.0);
            *state = b_c * *state + (1.0 - b_c) * wsum;
            *state
        }
        GraphNodeKind::Oscillator(f) => {
            let f_c = f.clamp(0.0, 8.0);
            *state = (*state + f_c).fract();
            (2.0 * std::f32::consts::PI * *state).sin()
        }
        GraphNodeKind::AdaptiveGain => {
            *state = (*state + 0.01 * wsum).clamp(0.1, 2.0);
            *state * wsum
        }
        // Output writers: their deferred effect (on output_slots / route_target_idx) is
        // applied post-loop. During the loop their curr_outputs slot just holds wsum.
        GraphNodeKind::CustomOutput(_) => wsum,
        GraphNodeKind::RouterOutput => wsum,
    }
}

/// Collect the weighted input values for a single internal node.
///
/// Implements Gauss-Seidel update order: sources already updated in this pass
/// (`source_idx < current_idx`) use `curr_outputs`; sources not yet updated
/// (or self-loops) use `prev_outputs`.
#[inline]
fn collect_weighted_inputs(
    node: &GraphInternalNode,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
) -> Vec<f32> {
    node.inputs
        .iter()
        .map(|input| {
            let src = input.source_idx as usize;
            let source_value = if src >= node_count {
                0.0
            } else if src < current_idx {
                curr_outputs[src]
            } else {
                prev_outputs[src]
            };
            source_value * input.weight
        })
        .collect()
}

/// Execute a graph-backend mesh node.
///
/// Runs the relaxation loop (Gauss-Seidel style), handles stateful operators,
/// charges energy per pass, and maps `CustomOutput`/`RouterOutput` nodes to the
/// returned [`NodeResult`].
///
/// # Energy semantics
/// Energy is charged **before** each pass. If energy drops to `<= 0` the
/// function restores the pre-call graph state snapshot and returns
/// [`NodeResult::exhausted`].
///
/// # Empty graph
/// If `def.internal_nodes` is empty, no energy is charged and the function
/// returns [`NodeResult::halted`] with the original `upstream_slots`.
// The signature is mandated by the v3 spec / mesh executor calling convention.
#[allow(clippy::too_many_arguments)]
pub fn execute_graph_node(
    def: &GraphBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    node_id: NodeId,
    graph_state: &mut HashMap<NodeId, Vec<f32>>,
    static_inputs: &StaticInputs,
    config: &RuntimeConfig,
) -> NodeResult {
    let node_count = def.internal_nodes.len();

    // Empty graph: no work, no energy charge.
    if node_count == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
    }

    // Snapshot state for atomic rollback on energy exhaustion.
    let state_backup: Option<Vec<f32>> = graph_state.get(&node_id).cloned();

    // Ensure the state Vec exists and is long enough.
    let state_vec = graph_state.entry(node_id).or_default();
    if state_vec.len() < node_count {
        state_vec.resize(node_count, 0.0);
    }

    let max_passes = config.max_graph_relax_iters;
    let epsilon = config.graph_convergence_epsilon;
    let req_stable = config.graph_convergence_stable_passes;

    let mut prev_outputs = vec![0.0f32; node_count];
    let mut curr_outputs = vec![0.0f32; node_count];
    let mut stable_passes: u32 = 0;

    for _pass in 0..max_passes {
        // Charge energy BEFORE evaluating this pass.
        let pass_cost = config.graph_node_base_cost * node_count as f32;
        *energy -= pass_cost;
        if *energy <= 0.0 {
            // Restore state snapshot.
            match state_backup {
                Some(backup) => {
                    graph_state.insert(node_id, backup);
                }
                None => {
                    graph_state.remove(&node_id);
                }
            }
            return NodeResult::exhausted();
        }

        // Rebuild the eval context with the live energy value after the pass charge.
        let ctx = EvalCtx {
            input_refs,
            upstream_slots,
            energy: *energy,
            energy_consumed,
            static_inputs,
        };

        for current_idx in 0..node_count {
            let node = &def.internal_nodes[current_idx];
            let w_inputs = collect_weighted_inputs(
                node,
                current_idx,
                node_count,
                &prev_outputs,
                &curr_outputs,
            );
            let wsum: f32 = w_inputs.iter().sum();

            // Load node-local state; write back after evaluate_kind updates it.
            // The Vec was extended to `node_count` before the loop.
            let mut node_state = graph_state
                .get(&node_id)
                .expect("state vec must exist after entry() call above")[current_idx];

            curr_outputs[current_idx] =
                evaluate_kind(&node.kind, &w_inputs, wsum, &ctx, &mut node_state);

            // Persist any state mutation from stateful operators.
            graph_state
                .get_mut(&node_id)
                .expect("state vec must still exist")[current_idx] = node_state;
        }

        // Convergence check: max absolute change across all outputs.
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
        if stable_passes >= req_stable {
            break;
        }
    }

    // Build NodeResult: start from upstream_slots, then apply deferred writes.
    let mut output_slots = *upstream_slots;
    let mut route_target_idx: f32 = 0.0;

    for (i, node) in def.internal_nodes.iter().enumerate() {
        match &node.kind {
            GraphNodeKind::CustomOutput(s) => {
                if (*s as usize) < 12 {
                    output_slots[*s as usize] = curr_outputs[i];
                }
            }
            GraphNodeKind::RouterOutput => {
                route_target_idx = curr_outputs[i];
            }
            _ => {}
        }
    }

    NodeResult {
        output_slots,
        route_target_idx,
        world_action: None,
        energy_exhausted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::NodeId;
    use crate::creature::genome::{GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind};
    use crate::sensors::static_inputs::StaticInputs;
    use std::collections::HashMap;

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn make_static_inputs() -> StaticInputs {
        StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        }
    }

    fn node_id(n: u32) -> NodeId {
        NodeId::new(n)
    }

    /// Single-node graph with the given kind and no inputs.
    fn single_node_graph(kind: GraphNodeKind) -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind,
                inputs: vec![],
            }],
        }
    }

    // ─── Test 1: Empty graph returns halted with upstream_slots, no energy charged ───
    #[test]
    fn empty_graph_returns_halted_and_no_energy_charged() {
        let def = GraphBackendDef {
            internal_nodes: vec![],
        };
        let upstream = [1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut energy = 50.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(0),
            &mut graph_state,
            &si,
            &config,
        );

        // No energy consumed.
        assert!((energy - 50.0).abs() < 1e-6, "energy should be unchanged");
        // Returns halted (not exhausted).
        assert!(!result.energy_exhausted);
        assert!(result.world_action.is_none());
        // Upstream slots passed through.
        assert_eq!(result.output_slots, upstream);
        assert_eq!(result.route_target_idx, 0.0);
    }

    // ─── Test 2: Energy exhaustion returns NodeResult::exhausted() ───────────────
    #[test]
    fn energy_exhaustion_returns_exhausted() {
        // graph_node_base_cost=1.0, 1 node → first pass costs 1.0.
        // Starting energy = 0.5 → after charge = -0.5 ≤ 0 → exhausted.
        let def = single_node_graph(GraphNodeKind::Constant(99.0));
        let upstream = [0.0f32; 12];
        let mut energy = 0.5f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(1),
            &mut graph_state,
            &si,
            &config,
        );

        assert!(result.energy_exhausted);
        assert!(result.world_action.is_none());
    }

    // ─── Test 3: Constant node — output_slots unchanged (no CustomOutput write) ───
    #[test]
    fn constant_node_does_not_write_output_slots() {
        let def = single_node_graph(GraphNodeKind::Constant(42.0));
        let upstream = [7.0f32; 12];
        let mut energy = 100.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(2),
            &mut graph_state,
            &si,
            &config,
        );

        // No CustomOutput → output_slots stays = upstream.
        assert_eq!(result.output_slots, upstream);
        assert!(!result.energy_exhausted);
        assert!(result.world_action.is_none());
    }

    // ─── Test 4: Add + CustomOutput combo writes output_slots[0] = 6.0 ──────────
    //
    // Graph:
    //   node 0: Constant(2.0)   — no inputs
    //   node 1: CustomOutput(0) — one input: source=0, weight=3.0
    //
    // Pass evaluation:
    //   curr[0] = 2.0  (Constant ignores inputs)
    //   curr[1]: w_inputs = [curr[0]*3.0] = [6.0], wsum=6.0 → CustomOutput → curr[1]=6.0
    // Post-loop: output_slots[0] = curr[1] = 6.0
    #[test]
    fn add_custom_output_writes_correct_slot() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(2.0),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 3.0,
                    }],
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(3),
            &mut graph_state,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert!(
            (result.output_slots[0] - 6.0).abs() < 1e-5,
            "output_slots[0]={}",
            result.output_slots[0]
        );
    }

    // ─── Test 5: RouterOutput sets route_target_idx ───────────────────────────
    #[test]
    fn router_output_sets_route_target_idx() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(3.5),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(4),
            &mut graph_state,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert!(
            (result.route_target_idx - 3.5).abs() < 1e-5,
            "route_target_idx={}",
            result.route_target_idx
        );
    }

    // ─── Test 6: Graph never emits WorldAction ────────────────────────────────
    #[test]
    fn graph_never_emits_world_action() {
        let def = single_node_graph(GraphNodeKind::Add);
        let upstream = [1.0f32; 12];
        let mut energy = 100.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(5),
            &mut graph_state,
            &si,
            &config,
        );

        assert!(result.world_action.is_none());
    }

    // ─── Test 7: DecayIntegrator accumulates state across two calls ───────────
    //
    // alpha=0.5: state = 0.5*prev_state + 0.5*wsum
    //
    // Graph: Constant(1.0) → DecayIntegrator(0.5) with weight 1.0 → CustomOutput(0)
    // Call 1: state0=0.0 → state1 = 0.5*0.0 + 0.5*1.0 = 0.5   → output_slots[0]=0.5
    // Call 2: state0=0.5 → state1 = 0.5*0.5 + 0.5*1.0 = 0.75  → output_slots[0]=0.75
    #[test]
    fn decay_integrator_accumulates_state() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::DecayIntegrator(0.5),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 1000.0f32;
        let mut graph_state: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let si = make_static_inputs();
        // One pass, converges immediately.
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let nid = node_id(6);

        let result1 = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut graph_state,
            &si,
            &config,
        );
        assert!(!result1.energy_exhausted);
        assert!(
            (result1.output_slots[0] - 0.5).abs() < 1e-5,
            "first call output={}",
            result1.output_slots[0]
        );

        let result2 = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut graph_state,
            &si,
            &config,
        );
        assert!(!result2.energy_exhausted);
        assert!(
            (result2.output_slots[0] - 0.75).abs() < 1e-5,
            "second call output={}",
            result2.output_slots[0]
        );
    }

    // ─── Test 8: State NOT mutated on energy exhaustion (atomicity) ──────────
    #[test]
    fn state_not_mutated_on_energy_exhaustion() {
        let def = GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind: GraphNodeKind::DecayIntegrator(0.9),
                inputs: vec![],
            }],
        };
        let nid = node_id(7);
        let upstream = [0.0f32; 12];
        let mut energy = 0.5f32; // Will exhaust on first pass charge.
        let mut graph_state: HashMap<NodeId, Vec<f32>> = HashMap::new();
        // Pre-populate state with a known sentinel.
        graph_state.insert(nid, vec![42.0f32]);

        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut graph_state,
            &si,
            &config,
        );

        assert!(result.energy_exhausted);
        // State must be restored to the pre-call value.
        let state_after = graph_state.get(&nid).expect("state key must be present");
        assert!(
            (state_after[0] - 42.0).abs() < 1e-6,
            "state was mutated: {}",
            state_after[0]
        );
    }

    // ─── Test 9: Output slots initialized from upstream_slots (passthrough) ──
    #[test]
    fn output_slots_initialized_from_upstream_passthrough() {
        let upstream: [f32; 12] = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let def = single_node_graph(GraphNodeKind::Sigmoid);
        let mut energy = 100.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(8),
            &mut graph_state,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert_eq!(
            result.output_slots, upstream,
            "upstream_slots must pass through when no CustomOutput"
        );
    }

    // ─── Additional: Verify energy IS deducted on a successful run ───────────
    #[test]
    fn energy_deducted_per_pass_on_success() {
        // 2 nodes, base_cost=1.0 → 2.0 per pass.  max_passes=1 forces exactly 1 pass.
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(2.0),
                    inputs: vec![],
                },
            ],
        };
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_node_base_cost = 1.0;

        let upstream = [0.0f32; 12];
        let mut energy = 50.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();

        let _ = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(9),
            &mut graph_state,
            &si,
            &config,
        );

        // 2 nodes * 1.0 cost = 2.0 consumed in exactly 1 pass.
        assert!((energy - 48.0).abs() < 1e-5, "energy={energy}");
    }

    // ─── Additional: RouterOutput last-write-wins ─────────────────────────────
    #[test]
    fn router_output_last_write_wins() {
        // 3-node graph: Constant(7.0) feeder, two RouterOutput nodes.
        // node 0 → 7.0
        // node 1 → RouterOutput: 7.0 * 1.0 = 7.0
        // node 2 → RouterOutput: 7.0 * 2.0 = 14.0   ← last-write wins
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(7.0),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 2.0,
                    }],
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut graph_state = HashMap::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            node_id(10),
            &mut graph_state,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert!(
            (result.route_target_idx - 14.0).abs() < 1e-5,
            "route_target_idx={}",
            result.route_target_idx
        );
    }
}
