use super::*;
use crate::config::RuntimeConfig;
use crate::creature::genome::{GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::types::MeshSideOutputs;
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;

fn default_config() -> RuntimeConfig {
    RuntimeConfig::default()
}

fn make_sensor_snapshot() -> SensorSnapshot {
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

/// Single-node graph with the given kind and no inputs.
fn single_node_graph(kind: GraphNodeKind) -> GraphBackendDef {
    GraphBackendDef {
        internal_nodes: vec![GraphInternalNode {
            kind,
            inputs: vec![],
            plasticity: None,
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
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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

    // No energy consumed.
    assert!((energy - 50.0).abs() < 1e-6, "energy should be unchanged");
    // Returns halted (not exhausted).
    assert!(!result.energy_exhausted);
    assert!(!result.terminal);
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
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
    );

    assert!(result.energy_exhausted);
    assert!(result.terminal);
}

// ─── Test 3: Constant node — output_slots unchanged (no CustomOutput write) ───
#[test]
fn constant_node_does_not_write_output_slots() {
    let def = single_node_graph(GraphNodeKind::Constant(42.0));
    let upstream = [7.0f32; 12];
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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

    // No CustomOutput → output_slots stays = upstream.
    assert_eq!(result.output_slots, upstream);
    assert!(!result.energy_exhausted);
    assert!(!result.terminal);
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
        ],
    };
    let upstream = [0.0f32; 12];
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::RouterOutput,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            },
        ],
    };
    let upstream = [0.0f32; 12];
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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

    assert!(!result.terminal);
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
    let mut energy = 1000.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
            plasticity: None,
        }],
    };
    let nid: usize = 0;
    let upstream = [0.0f32; 12];
    // Will exhaust on first pass charge.
    let mut energy = 0.5f32;
    // Pre-populate state with a known sentinel.
    let mut gr = GraphRuntimeState {
        node_state: vec![vec![42.0f32]],
        plasticity_weights: Vec::new(),
        scratch_prev: Vec::new(),
        scratch_curr: Vec::new(),
        scratch_backup: Vec::new(),
        scratch_w_inputs: Vec::new(),
    };

    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Constant(2.0),
                inputs: vec![],
                plasticity: None,
            },
        ],
    };
    let mut config = default_config();
    config.max_graph_relax_iters = 1;
    config.graph_node_base_cost = 1.0;

    let upstream = [0.0f32; 12];
    let mut energy = 50.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();

    let _ = execute_graph_node(
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
    let mut energy = 1000.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::Momentum(0.8),
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
    let mut energy = 1000.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
    let mut energy = 1000.0f32;
    let ss = make_sensor_snapshot();
    let mut config = default_config();
    config.max_graph_relax_iters = 1;
    config.graph_convergence_stable_passes = 1;

    let nid: usize = 0;
    // Pre-set state slot 0 to INFINITY to simulate the corrupted state case.
    let mut gr = GraphRuntimeState {
        node_state: vec![vec![f32::INFINITY, 0.0]],
        plasticity_weights: Vec::new(),
        scratch_prev: Vec::new(),
        scratch_curr: Vec::new(),
        scratch_backup: Vec::new(),
        scratch_w_inputs: Vec::new(),
    };

    let r = execute_graph_node(
        &def,
        &[],
        &upstream,
        &mut energy,
        0.0,
        nid,
        &mut gr,
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Threshold(0.5),
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: input_weight,
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
        }
    };

    let upstream = [0.0f32; 12];
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
    let mut energy = 1000.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
                    plasticity: None,
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
        }
    };

    let upstream = [0.0f32; 12];
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(10.0),
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(20.0),
                    inputs: vec![],
                    plasticity: None,
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
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput {
                        source_idx: 3,
                        weight: 1.0,
                    }],
                    plasticity: None,
                },
            ],
        }
    };

    let upstream = [0.0f32; 12];
    let ss = make_sensor_snapshot();
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
        &ss,
        &config,
        &mut MeshSideOutputs::new(4),
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
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::RouterOutput,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::RouterOutput,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 2.0,
                }],
                plasticity: None,
            },
        ],
    };
    let upstream = [0.0f32; 12];
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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
                kind: GraphNodeKind::InputRef {
                    ref_idx: 255,
                    sub_idx: 0,
                },
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
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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
                plasticity: None,
            },
            GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(255),
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
                plasticity: None,
            },
        ],
    };
    let upstream = [3.0f32; 12];
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();

    let result = execute_graph_node(
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

    assert!(!result.energy_exhausted);
    assert_eq!(
        result.output_slots[0], 0.0,
        "out-of-range edge sources must soft-default to zero"
    );
}

// ─── Graph action-queue tests ────────────────────────────────────────────

/// PushAction + ExecuteActionQueue → terminal with action queued.
#[test]
fn graph_push_action_and_terminate() {
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
    let mut energy = 100.0f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let config = default_config();
    let mut so = MeshSideOutputs::new(4);

    let result = execute_graph_node(
        &def,
        &[],
        &upstream,
        &mut energy,
        0.0,
        0,
        &mut gr,
        &ss,
        &config,
        &mut so,
    );

    assert!(result.terminal);
    assert!(!result.energy_exhausted);
    let actions = so.action_queue.into_actions_or_noop();
    assert_eq!(
        actions,
        vec![crate::contracts::WorldAction::Eat],
        "graph should queue Eat via PushAction(1)"
    );
}

/// Energy exhaustion prevents effects from being applied.
#[test]
fn energy_exhaustion_prevents_effects() {
    let def = GraphBackendDef {
        internal_nodes: vec![GraphInternalNode {
            kind: GraphNodeKind::PushAction(1), // Eat
            inputs: vec![],
            plasticity: None,
        }],
    };
    let upstream = [0.0f32; 12];
    let mut energy = 0.5f32;
    let mut gr = GraphRuntimeState::new();
    let ss = make_sensor_snapshot();
    let mut config = default_config();
    config.graph_node_base_cost = 1.0; // 1 node × 1.0 > energy (0.5) → exhausted

    let mut so = MeshSideOutputs::new(4);

    let result = execute_graph_node(
        &def,
        &[],
        &upstream,
        &mut energy,
        0.0,
        0,
        &mut gr,
        &ss,
        &config,
        &mut so,
    );

    assert!(result.energy_exhausted);
    // Action queue should be empty since energy exhaustion prevents effect application
    assert!(
        so.action_queue.into_actions_or_noop() == vec![crate::contracts::WorldAction::NoOp],
        "exhausted graph must not push actions"
    );
}
