use crate::config::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::hebbian;
use crate::runtime::inputs::resolve_input;
use crate::runtime::types::{sanitize_f32, NodeResult};
use crate::sensors::static_inputs::StaticInputs;

/// Immutable context for resolving `InputRef` nodes during graph evaluation.
pub(crate) struct EvalCtx<'a> {
    pub(crate) input_refs: &'a [InputReference],
    pub(crate) upstream_slots: &'a [f32; 12],
    pub(crate) energy: f32,
    pub(crate) energy_consumed: f32,
    pub(crate) static_inputs: &'a StaticInputs,
}

/// Evaluate one internal graph node's kind, returning the scalar output.
///
/// `w_inputs` holds the per-edge weighted values (source_value * weight).
/// `wsum` is `w_inputs.iter().sum()`.
/// `state` is the node's mutable persistent scalar state (for stateful operators).
#[inline]
pub(crate) fn evaluate_kind(
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

/// Collect the weighted input values for a single internal node into `buf`.
///
/// Implements Gauss-Seidel update order: sources already updated in this pass
/// (`source_idx < current_idx`) use `curr_outputs`; sources not yet updated
/// (or self-loops) use `prev_outputs`.
///
/// `buf` is cleared and filled with one entry per input edge. The caller
/// should allocate `buf` once and reuse it across nodes/passes.
#[inline]
pub(crate) fn collect_weighted_inputs(
    node: &GraphInternalNode,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(node.inputs.iter().map(|input| {
        let src = input.source_idx as usize;
        let source_value = if src >= node_count {
            0.0
        } else if src < current_idx {
            curr_outputs[src]
        } else {
            prev_outputs[src]
        };
        source_value * input.weight
    }));
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
    node_idx: usize,
    graph_runtime: &mut GraphRuntimeState,
    static_inputs: &StaticInputs,
    config: &RuntimeConfig,
) -> NodeResult {
    let node_count = def.internal_nodes.len();

    // Empty graph: no work, no energy charge.
    if node_count == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
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

    // Check if any node uses Hebbian learning and prepare weights if so.
    let use_hebbian = hebbian::has_any_hebbian(def);
    if use_hebbian {
        hebbian::ensure_hebbian_weights(def, node_idx, &mut graph_runtime.hebbian_weights);
    }

    let max_passes = config.max_graph_relax_iters;
    let epsilon = config.graph_convergence_epsilon;
    let req_stable = config.graph_convergence_stable_passes;

    let mut prev_outputs = vec![0.0f32; node_count];
    let mut curr_outputs = vec![0.0f32; node_count];
    let mut stable_passes: u32 = 0;
    let mut w_inputs_buf: Vec<f32> = Vec::new();

    for _pass in 0..max_passes {
        // Charge energy BEFORE evaluating this pass.
        let pass_cost = config.graph_node_base_cost * node_count as f32;
        *energy -= pass_cost;
        if *energy <= 0.0 {
            // Restore state snapshot.
            graph_runtime.node_state[node_idx] = state_backup;
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

            // Use learned weights for Hebbian nodes, genome weights otherwise.
            if use_hebbian && node.hebbian.is_some() {
                let learned = &graph_runtime.hebbian_weights[node_idx][current_idx];
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

            // Load node-local state; write back after evaluate_kind updates it.
            let mut node_state = graph_runtime.node_state[node_idx][current_idx];

            curr_outputs[current_idx] = sanitize_f32(evaluate_kind(
                &node.kind,
                &w_inputs_buf,
                wsum,
                &ctx,
                &mut node_state,
            ));

            // Persist any state mutation from stateful operators.
            graph_runtime.node_state[node_idx][current_idx] = node_state;
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

    // Apply Hebbian weight updates after convergence.
    if use_hebbian {
        let hebb_cost = hebbian::apply_hebbian_updates(
            def,
            node_idx,
            &mut graph_runtime.hebbian_weights,
            &curr_outputs,
            config.hebbian_update_cost,
        );
        *energy -= hebb_cost;
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
    use crate::creature::genome::{GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind};
    use crate::creature::state::GraphRuntimeState;
    use crate::sensors::static_inputs::StaticInputs;

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

    /// Single-node graph with the given kind and no inputs.
    fn single_node_graph(kind: GraphNodeKind) -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: vec![GraphInternalNode {
                kind,
                inputs: vec![],
                hebbian: None,
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
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let mut config = default_config();
        config.graph_node_base_cost = 1.0;

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 3.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::DecayIntegrator(0.5),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 1000.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        // One pass, converges immediately.
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let nid: usize = 0;

        let result1 = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut gr,
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
            &mut gr,
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
                hebbian: None,
            }],
        };
        let nid: usize = 0;
        let upstream = [0.0f32; 12];
        // Will exhaust on first pass charge.
        let mut energy = 0.5f32;
        // Pre-populate state with a known sentinel.
        let mut gr = GraphRuntimeState {
            node_state: vec![vec![42.0f32]],
            hebbian_weights: Vec::new(),
        };

        let si = make_static_inputs();
        let mut config = default_config();
        config.graph_node_base_cost = 1.0; // 1 node × 1.0 > energy (0.5) → exhausted

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut gr,
            &si,
            &config,
        );

        assert!(result.energy_exhausted);
        // State must be restored to the pre-call value.
        assert!(
            (gr.node_state[nid][0] - 42.0).abs() < 1e-6,
            "state was mutated: {}",
            gr.node_state[nid][0]
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
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(2.0),
                    inputs: vec![],
                    hebbian: None,
                },
            ],
        };
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_node_base_cost = 1.0;

        let upstream = [0.0f32; 12];
        let mut energy = 50.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();

        let _ = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &si,
            &config,
        );

        // 2 nodes * 1.0 cost = 2.0 consumed in exactly 1 pass.
        assert!((energy - 48.0).abs() < 1e-5, "energy={energy}");
    }

    // ─── Test: decay_integrator_formula_correct ───────────────────────────────
    //
    // DecayIntegrator(alpha=0.5): state = (1-alpha)*state + alpha*wsum
    //
    // Two-node graph: Constant(1.0) → DecayIntegrator(0.5) → CustomOutput(0)
    // Call 1: state=0.0 → (1-0.5)*0.0 + 0.5*1.0 = 0.5
    // Call 2: state=0.5 → (1-0.5)*0.5 + 0.5*1.0 = 0.75
    #[test]
    fn decay_integrator_formula_correct() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::DecayIntegrator(0.5),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 1000.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let nid: usize = 0;

        let r1 = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut gr,
            &si,
            &config,
        );
        assert!(
            (r1.output_slots[0] - 0.5).abs() < 1e-5,
            "first call expected 0.5, got {}",
            r1.output_slots[0]
        );

        let r2 = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut gr,
            &si,
            &config,
        );
        assert!(
            (r2.output_slots[0] - 0.75).abs() < 1e-5,
            "second call expected 0.75, got {}",
            r2.output_slots[0]
        );
    }

    // ─── Test: momentum_formula_correct ──────────────────────────────────────
    //
    // Momentum(beta=0.8): state = beta*state + (1-beta)*wsum
    //
    // Call 1: state=0.0 → 0.8*0.0 + 0.2*1.0 = 0.2
    #[test]
    fn momentum_formula_correct() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Momentum(0.8),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 1,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 1000.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let nid: usize = 0;

        let r1 = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut gr,
            &si,
            &config,
        );
        assert!(
            (r1.output_slots[0] - 0.2).abs() < 1e-5,
            "momentum first call expected 0.2, got {}",
            r1.output_slots[0]
        );
    }

    // ─── Test: oscillator_nan_safe ────────────────────────────────────────────
    //
    // With initial state = f32::INFINITY, (*state + f_c).fract() = INFINITY.fract() = NaN.
    // sanitize_f32 must turn this into 0.0 before it propagates.
    #[test]
    fn oscillator_nan_safe() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Oscillator(0.0),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 1000.0f32;
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let nid: usize = 0;
        // Pre-set state slot 0 to INFINITY to simulate the corrupted state case.
        let mut gr = GraphRuntimeState {
            node_state: vec![vec![f32::INFINITY, 0.0]],
            hebbian_weights: Vec::new(),
        };

        let r = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            nid,
            &mut gr,
            &si,
            &config,
        );

        assert!(
            !r.output_slots[0].is_nan(),
            "output must not be NaN, got {}",
            r.output_slots[0]
        );
        assert!(!r.energy_exhausted);
    }

    // ─── Test: threshold_formula_correct ─────────────────────────────────────
    //
    // Threshold(t=0.5): output = 1.0 if wsum > t, else 0.0  (strictly greater than)
    #[test]
    fn threshold_formula_correct() {
        let make_threshold_graph = |input_weight: f32| -> GraphBackendDef {
            GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(1.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::Threshold(0.5),
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: input_weight,
                        }],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(0),
                        inputs: vec![GraphInput {
                            source_idx: 1,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                ],
            }
        };

        let upstream = [0.0f32; 12];
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        // wsum = 0.6 > 0.5 → 1.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_threshold_graph(0.6),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(r.output_slots[0], 1.0, "wsum=0.6 should fire");

        // wsum = 0.5 = 0.5 (not strictly greater) → 0.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_threshold_graph(0.5),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(
            r.output_slots[0], 0.0,
            "wsum=0.5 should NOT fire (strictly >)"
        );

        // wsum = 0.4 < 0.5 → 0.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_threshold_graph(0.4),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(r.output_slots[0], 0.0, "wsum=0.4 should NOT fire");
    }

    // ─── Test: multiply_empty_inputs_is_one ──────────────────────────────────
    //
    // Multiply with no inputs: product of empty iterator = 1.0 (identity element).
    #[test]
    fn multiply_empty_inputs_is_one() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Multiply,
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 1000.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        let r = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &si,
            &config,
        );

        assert_eq!(
            r.output_slots[0], 1.0,
            "Multiply with no inputs should produce 1.0, got {}",
            r.output_slots[0]
        );
    }

    // ─── Test: greater_than_formula ───────────────────────────────────────────
    //
    // GreaterThan: output = 1.0 if w_inputs[0] > w_inputs[1], else 0.0
    #[test]
    fn greater_than_formula() {
        let make_gt_graph = |wa: f32, wb: f32| -> GraphBackendDef {
            GraphBackendDef {
                internal_nodes: vec![
                    // node 0: Constant(1.0)
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(1.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    // node 1: GreaterThan — two edges from node 0, weights wa and wb
                    GraphInternalNode {
                        kind: GraphNodeKind::GreaterThan,
                        inputs: vec![
                            GraphInput {
                                source_idx: 0,
                                weight: wa,
                            },
                            GraphInput {
                                source_idx: 0,
                                weight: wb,
                            },
                        ],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(0),
                        inputs: vec![GraphInput {
                            source_idx: 1,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                ],
            }
        };

        let upstream = [0.0f32; 12];
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        // a=2.0 > b=1.0 → 1.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_gt_graph(2.0, 1.0),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(r.output_slots[0], 1.0, "2.0 > 1.0 should produce 1.0");

        // a=1.0 == b=1.0 → 0.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_gt_graph(1.0, 1.0),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(r.output_slots[0], 0.0, "1.0 == 1.0 should produce 0.0");

        // a=0.5 < b=1.0 → 0.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_gt_graph(0.5, 1.0),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(r.output_slots[0], 0.0, "0.5 < 1.0 should produce 0.0");
    }

    // ─── Test: select_formula ─────────────────────────────────────────────────
    //
    // Select: if w_inputs[0] >= 0.5 return w_inputs[1], else return w_inputs[2].
    //
    // Graph: three Constant feeders → Select → CustomOutput(0)
    //   node 0: Constant(cond_weight)   — varied per sub-test
    //   node 1: Constant(10.0)          — "true" branch value
    //   node 2: Constant(20.0)          — "false" branch value
    //   node 3: Select with edges: [0*1.0, 1*1.0, 2*1.0]
    //   node 4: CustomOutput(0) ← node 3
    #[test]
    fn select_formula() {
        let make_select_graph = |cond: f32| -> GraphBackendDef {
            GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(cond),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(10.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(20.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::Select,
                        inputs: vec![
                            GraphInput {
                                source_idx: 0,
                                weight: 1.0,
                            },
                            GraphInput {
                                source_idx: 1,
                                weight: 1.0,
                            },
                            GraphInput {
                                source_idx: 2,
                                weight: 1.0,
                            },
                        ],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(0),
                        inputs: vec![GraphInput {
                            source_idx: 3,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                ],
            }
        };

        let upstream = [0.0f32; 12];
        let si = make_static_inputs();
        let mut config = default_config();
        config.max_graph_relax_iters = 1;
        config.graph_convergence_stable_passes = 1;

        // cond=1.0 >= 0.5 → picks w_inputs[1] = 10.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_select_graph(1.0),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(
            r.output_slots[0], 10.0,
            "cond=1.0 should select true branch (10.0), got {}",
            r.output_slots[0]
        );

        // cond=0.0 < 0.5 → picks w_inputs[2] = 20.0
        let mut energy = 1000.0f32;
        let r = execute_graph_node(
            &make_select_graph(0.0),
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut GraphRuntimeState::new(),
            &si,
            &config,
        );
        assert_eq!(
            r.output_slots[0], 20.0,
            "cond=0.0 should select false branch (20.0), got {}",
            r.output_slots[0]
        );
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
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 2.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
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

    #[test]
    fn input_ref_255_soft_defaults_to_zero() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef(255),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert_eq!(
            result.output_slots[0], 0.0,
            "out-of-range InputRef must soft-default to zero"
        );
    }

    #[test]
    fn custom_output_255_does_not_write_output_slots() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(7.0),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(255),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [3.0f32; 12];
        let mut energy = 100.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert_eq!(
            result.output_slots, upstream,
            "out-of-range CustomOutput index must leave output slots unchanged"
        );
    }

    #[test]
    fn edge_source_65535_soft_defaults_to_zero() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![GraphInput {
                        source_idx: u16::MAX,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        };
        let upstream = [0.0f32; 12];
        let mut energy = 100.0f32;
        let mut gr = GraphRuntimeState::new();
        let si = make_static_inputs();
        let config = default_config();

        let result = execute_graph_node(
            &def,
            &[],
            &upstream,
            &mut energy,
            0.0,
            0,
            &mut gr,
            &si,
            &config,
        );

        assert!(!result.energy_exhausted);
        assert_eq!(
            result.output_slots[0], 0.0,
            "out-of-range edge sources must soft-default to zero"
        );
    }
}
