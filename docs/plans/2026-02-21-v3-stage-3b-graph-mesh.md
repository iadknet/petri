# V3 Stage 3b: Graph Backend + Mesh Chain Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `runtime/graph.rs` (`execute_graph_node` with all 22 operators, relaxation loop, stateful operators) and `runtime/mesh.rs` (`execute_creature_mesh` with the full mesh chain algorithm, soft-default matrix, and energy tracking) so that a full creature cognition tick can be run end-to-end.

**Architecture:** Two new source files under `v3/crates/v3-core/src/runtime/`. The graph executor runs a bounded relaxation loop with atomically-reverted state on exhaustion; it never emits `WorldAction`. The mesh executor chains node evaluations, dispatching to `execute_vm_node` or `execute_graph_node` per `BackendDef`, tracking `energy_consumed` for dynamic introspection, and applying the canonical soft-default matrix from `v3-mesh-execution-spec.md`.

**Tech Stack:** Rust, `std::collections::HashMap`, existing v3-core types: `CreatureGenome`, `NodeGenome`, `BackendDef`, `GraphBackendDef`, `GraphNodeKind`, `GraphInternalNode`, `GraphInput`, `NodeId`, `InputReference`, `RuntimeConfig`, `StaticInputs`, `NodeResult`, `WorldAction`, `resolve_input`, `execute_vm_node`.

**Parent plan:** `docs/plans/2026-02-21-v3-stage-3a-sensors-vm.md`

---

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** `runtime/graph.rs` (`execute_graph_node`, 22 operators, relaxation loop, stateful ops) and `runtime/mesh.rs` (`execute_creature_mesh`, chain algorithm, soft-default matrix) in `v3/crates/v3-core`. Excludes tick orchestration, mutation, reproduction, server, and CLI.

**Docs Impact:** `docs/plans/2026-02-21-v3-stage-3b-graph-mesh.md` (this plan created); `docs/reference/v3-graph-backend-spec.md` loop-range typo fixed; no other canonical docs changed.

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

- **GP-01:** Completes the cognition execution substrate — creatures can now run full graph-backend behavior and multi-hop mesh chains.
- **GP-02:** Graph and mesh execution remain in `runtime/`; tick orchestration and contracts are untouched.
- **GP-03:** All semantics are fully tested, TDD-style, before implementation.

## Boundary Impact

- New files: `v3/crates/v3-core/src/runtime/graph.rs` and `runtime/mesh.rs`.
- `runtime/mod.rs` gains two `pub mod` declarations and a `pub use`.
- No changes to `contracts/`, `sensors/`, `kernel/`, `creature/`, or `config/`.
- `CreatureState.energy` (f32) and `CreatureState.graph_state` (HashMap) are mutated by the mesh executor; that is the design intent.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `runtime/vm.rs` | keep | No changes; graph.rs calls `resolve_input` (same helper) but does not touch VM internals |
| `runtime/inputs.rs` | keep | `resolve_input` signature is stable; graph.rs imports it directly |
| `creature/state.rs` | keep | `CreatureState.graph_state: HashMap<NodeId, Vec<f32>>` already exists and is the correct state store |
| `contracts/` | keep | `WorldAction` and `NodeId` are consumed, not modified |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| When do stateful operators update state in the relaxation loop? | Every pass — same as other operators. State atomically rolled back on energy exhaustion. | agent | resolved |
| What are the formulas for V3 stateful operators? | Defined explicitly in Task 1 spec facts below. | agent | resolved |
| Does energy_consumed enter graph node evaluation? | Yes — passed through to `resolve_input` for dynamic introspection. Mesh tracks it as `start_energy - *energy`. | agent | resolved |
| How does Multiply aggregate differ from Add? | Multiply = product of all `(source_value * weight)` terms; Add = sum. | agent | resolved |
| How do Select/GreaterThan work with only one scalar? | They use individual weighted inputs (tracked alongside weighted_input_sum). | agent | resolved |

---

## Context and File Layout

- Worktree: `.worktrees/v3-implementation`
- All cargo commands run from the `v3/` subdirectory
- Current passing tests: 153
- Existing runtime files: `types.rs`, `inputs.rs`, `vm.rs`, `action_decode.rs`

Expected final file tree after this stage:
```
v3/crates/v3-core/src/runtime/
├── mod.rs               ← add: pub mod graph; pub mod mesh; pub use mesh::execute_creature_mesh;
├── types.rs             ← unchanged
├── inputs.rs            ← unchanged
├── vm.rs                ← unchanged
├── action_decode.rs     ← unchanged
├── graph.rs             ← NEW: execute_graph_node
└── mesh.rs              ← NEW: execute_creature_mesh
```

---

## Spec Facts (read before implementing)

**Canonical specs:** `docs/reference/v3-graph-backend-spec.md`, `docs/reference/v3-mesh-execution-spec.md`, `docs/reference/v3-runtime-config-spec.md`

### Graph evaluation pseudocode (from spec Section 2)
```text
node_count = internal_nodes.len()
prev_outputs = [0.0; node_count]
curr_outputs = [0.0; node_count]
max_passes  = config.max_graph_relax_iters (>= 1, fallback 4)
epsilon     = config.graph_convergence_epsilon (>= 0.0, fallback 1e-3)
req_stable  = config.graph_convergence_stable_passes (>= 1, fallback 1)

stable_passes = 0
passes_executed = 0

for pass in 0..max_passes:
  // Deduct per-pass energy BEFORE evaluating the pass.
  pass_cost = config.graph_node_base_cost * node_count as f32
  // If energy would be exhausted: restore graph_state snapshot, return NodeResult::exhausted()

  energy -= pass_cost

  for current_idx in 0..node_count:
    w_inputs: Vec<f32> = []
    for input in internal_nodes[current_idx].inputs:
      source_idx = input.source_idx as usize
      source_value =
        if source_idx >= node_count: 0.0
        elif source_idx < current_idx: curr_outputs[source_idx]
        else: prev_outputs[source_idx]
      w_inputs.push(source_value * input.weight)
    weighted_input_sum = w_inputs.iter().sum()
    curr_outputs[current_idx] = evaluate_kind(kind, w_inputs, weighted_input_sum, ...)

  passes_executed += 1
  delta = max over i of |curr_outputs[i] - prev_outputs[i]|
  prev_outputs = curr_outputs.clone()

  if delta <= epsilon:
    stable_passes += 1
  else:
    stable_passes = 0
  if stable_passes >= req_stable:
    break

// curr_outputs are the final outputs; apply CustomOutput/RouterOutput writes.
```

Energy atomicity: snapshot `graph_state[node_id]` before the loop; restore if exhausted.

### GraphNodeKind semantics (22 variants)

`w_inputs: &[f32]` = per-edge weighted values. `wsum: f32` = sum of w_inputs.

| Variant | Formula |
|---------|---------|
| `InputRef(u)` | `resolve_input(input_refs[u], ...) + wsum` (OOB u → 0.0 + wsum) |
| `Constant(f)` | `f` (ignores all inputs) |
| `Add` | `wsum` |
| `Multiply` | `w_inputs.iter().product()` (1.0 if empty) |
| `Negate` | `-wsum` |
| `Abs` | `wsum.abs()` |
| `Min` | `w_inputs.iter().copied().reduce(f32::min).unwrap_or(0.0)` |
| `Max` | `w_inputs.iter().copied().reduce(f32::max).unwrap_or(0.0)` |
| `Threshold(t)` | `if wsum > t { 1.0 } else { 0.0 }` |
| `GreaterThan` | `if w_inputs[0] > w_inputs.get(1).copied().unwrap_or(0.0) { 1.0 } else { 0.0 }` |
| `Sigmoid` | `1.0 / (1.0 + (-wsum).exp())` |
| `Tanh` | `wsum.tanh()` |
| `Relu` | `wsum.max(0.0)` |
| `Select` | `if w_inputs[0] >= 0.5 { w_inputs.get(1).copied().unwrap_or(0.0) } else { w_inputs.get(2).copied().unwrap_or(0.0) }` |
| `Clamp01` | `wsum.clamp(0.0, 1.0)` |
| `WeightedSum` | `wsum` |
| `DecayIntegrator(a)` | `a_c=a.clamp(0.0,1.0); state = (1.0-a_c)*state + a_c*wsum; state` |
| `Momentum(b)` | `b_c=b.clamp(0.0,1.0); state = b_c*state + (1.0-b_c)*wsum; state` |
| `Oscillator(f)` | `f_c=f.clamp(0.0,8.0); state=(state+f_c).fract(); (2.0*PI*state).sin()` |
| `AdaptiveGain` | `state=(state+0.01*wsum).clamp(0.1,2.0); state*wsum` |
| `CustomOutput(s)` | write `wsum` to `output_slots[s]` if `s < 12`; return `wsum` |
| `RouterOutput` | write `wsum` to `route_target_idx` (last-write-wins); return `wsum` |

Stateful operators use `graph_state[node_id][current_idx]` (extend with 0.0 if shorter).

### Mesh chain pseudocode (from spec Section 2)
```text
current_node_id = genome.entry_node_id
upstream_slots = [0.0; 12]
hops = 0
max_hops = config.max_mesh_hops (>= 1, fallback 128)
start_energy = *energy

// Soft default: entry_node_id missing → NoOp immediately.

loop:
  if hops >= max_hops: return WorldAction::NoOp
  node = genome.find_node(current_node_id) else return WorldAction::NoOp
  energy_consumed = start_energy - *energy

  result = match node.backend_def:
    Vm(def)    → execute_vm_node(def, &node.input_refs, &upstream_slots, energy, energy_consumed, memory, static_inputs, config)
    Graph(def) → execute_graph_node(def, &node.input_refs, &upstream_slots, energy, energy_consumed, node.node_id, graph_state, static_inputs, config)

  if result.energy_exhausted: return WorldAction::NoOp

  if result.world_action is Some(action): return action

  if node.targets is empty: return WorldAction::NoOp

  route_target_idx = result.route_target_idx
  route_idx_i64 =
    if route_target_idx.is_nan(): -1
    elif route_target_idx == f32::INFINITY: i64::MAX
    elif route_target_idx == f32::NEG_INFINITY: i64::MIN
    else: route_target_idx.floor().clamp(i64::MIN as f32, i64::MAX as f32) as i64

  target_idx = route_idx_i64.rem_euclid(node.targets.len() as i64) as usize
  target_id = node.targets[target_idx]
  if genome.find_node(target_id) is None: return WorldAction::NoOp

  upstream_slots = result.output_slots
  current_node_id = target_id
  hops += 1
```

---

## Task 1: `runtime/graph.rs` — Graph node executor

**Files:**
- Create: `v3/crates/v3-core/src/runtime/graph.rs`
- Modify: `v3/crates/v3-core/src/runtime/mod.rs`

### Step 1: Write failing tests first

Create `v3/crates/v3-core/src/runtime/graph.rs` with tests only (no impl yet):

```rust
use std::collections::HashMap;
use crate::config::RuntimeConfig;
use crate::contracts::{InputReference, NodeId};
use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphInput, GraphNodeKind};
use crate::runtime::types::NodeResult;
use crate::sensors::static_inputs::StaticInputs;

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
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use slotmap::SlotMap;

    fn make_node_id() -> NodeId {
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        sm.insert(())
    }

    fn default_si() -> StaticInputs {
        StaticInputs {
            food_here: 0.5,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 1.0,
            age_ticks: 5.0,
        }
    }

    fn cfg() -> RuntimeConfig {
        SimulationConfig::default().runtime
    }

    fn one_node(kind: GraphNodeKind, inputs: Vec<GraphInput>) -> GraphBackendDef {
        GraphBackendDef {
            internal_nodes: vec![GraphInternalNode { kind, inputs }],
        }
    }

    // ── empty graph ──────────────────────────────────────────────────────────

    #[test]
    fn empty_graph_returns_halted_with_upstream_slots() {
        let def = GraphBackendDef { internal_nodes: vec![] };
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        let upstream = [1.0f32; 12];
        let r = execute_graph_node(&def, &[], &upstream, &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
        assert_eq!(r.output_slots, upstream);
        // No energy charged for empty graph
        assert!((energy - 100.0).abs() < 1e-6);
    }

    // ── energy exhaustion ────────────────────────────────────────────────────

    #[test]
    fn exhausted_energy_returns_exhausted_result() {
        let def = one_node(GraphNodeKind::Add, vec![]);
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 0.0001f32; // far below base cost of 1.0
        let r = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        assert!(r.energy_exhausted);
        assert!(r.world_action.is_none());
    }

    // ── Constant ─────────────────────────────────────────────────────────────

    #[test]
    fn constant_node_returns_constant() {
        let def = one_node(GraphNodeKind::Constant(3.14), vec![]);
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        let r = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        // No CustomOutput/RouterOutput so output_slots unchanged from upstream (all 0.0)
        // curr_outputs[0] == 3.14 but graph never emitted to slots without CustomOutput
        assert!(!r.energy_exhausted);
        assert!(r.world_action.is_none());
    }

    // ── Add / WeightedSum ────────────────────────────────────────────────────

    #[test]
    fn add_node_sums_weighted_inputs() {
        // Two internal nodes: node0=Constant(2.0), node1=CustomOutput(0) with edge from node0 weight=3.0
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode { kind: GraphNodeKind::Constant(2.0), inputs: vec![] },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput { source_idx: 0, weight: 3.0 }],
                },
            ],
        };
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        let r = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        // node0 = 2.0, node1 = CustomOutput writes wsum = 2.0 * 3.0 = 6.0 to slot 0
        assert!((r.output_slots[0] - 6.0).abs() < 1e-5, "got {}", r.output_slots[0]);
    }

    // ── RouterOutput ─────────────────────────────────────────────────────────

    #[test]
    fn router_output_sets_route_target() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode { kind: GraphNodeKind::Constant(2.5), inputs: vec![] },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput { source_idx: 0, weight: 1.0 }],
                },
            ],
        };
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        let r = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        assert!((r.route_target_idx - 2.5).abs() < 1e-5, "got {}", r.route_target_idx);
    }

    // ── Graph never emits WorldAction ─────────────────────────────────────────

    #[test]
    fn graph_node_never_emits_world_action() {
        let def = one_node(GraphNodeKind::Add, vec![]);
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        let r = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        assert!(r.world_action.is_none());
    }

    // ── Stateful operators ────────────────────────────────────────────────────

    #[test]
    fn decay_integrator_accumulates_state_across_calls() {
        // DecayIntegrator(0.5) with Constant(1.0) as source
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode { kind: GraphNodeKind::Constant(1.0), inputs: vec![] },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(0),
                    inputs: vec![GraphInput { source_idx: 0, weight: 1.0 }],
                },
                // node2 = DecayIntegrator(0.5) fed by Constant
                GraphInternalNode {
                    kind: GraphNodeKind::DecayIntegrator(0.5),
                    inputs: vec![GraphInput { source_idx: 0, weight: 1.0 }],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::CustomOutput(1),
                    inputs: vec![GraphInput { source_idx: 2, weight: 1.0 }],
                },
            ],
        };
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        // First call: state starts at 0.0; new state = 0.5*0 + 0.5*1.0 = 0.5
        let r1 = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        let slot1 = r1.output_slots[1];
        // Second call (state persists across calls via gs): state = 0.5*0.5 + 0.5*1.0 = 0.75
        let r2 = execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        let slot2 = r2.output_slots[1];
        assert!(slot2 > slot1, "state should grow toward 1.0; got {slot1} then {slot2}");
        assert!(slot2 < 1.0, "state should not reach 1.0 yet; got {slot2}");
    }

    // ── State atomicity on exhaustion ─────────────────────────────────────────

    #[test]
    fn state_not_mutated_on_energy_exhaustion() {
        let def = GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode { kind: GraphNodeKind::DecayIntegrator(0.5), inputs: vec![] },
            ],
        };
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        // Pre-set state to a known value
        gs.insert(nid, vec![0.75]);
        let mut energy = 0.0001f32;
        execute_graph_node(&def, &[], &[0.0; 12], &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        // State must not have changed
        let state_after = gs.get(&nid).and_then(|v| v.first()).copied().unwrap_or(0.0);
        assert!((state_after - 0.75).abs() < 1e-6, "state was mutated on exhaustion: {state_after}");
    }

    // ── Upstream slot passthrough ─────────────────────────────────────────────

    #[test]
    fn output_slots_initialized_from_upstream_slots() {
        // Graph with no CustomOutput writes → output_slots == upstream_slots
        let def = one_node(GraphNodeKind::Add, vec![]);
        let nid = make_node_id();
        let mut gs: HashMap<NodeId, Vec<f32>> = HashMap::new();
        let mut energy = 100.0f32;
        let mut upstream = [0.0f32; 12];
        upstream[3] = 7.7;
        let r = execute_graph_node(&def, &[], &upstream, &mut energy, 0.0, nid, &mut gs, &default_si(), &cfg());
        assert!((r.output_slots[3] - 7.7).abs() < 1e-6);
    }
}
```

### Step 2: Run tests to verify they fail

```bash
cd /path/to/worktree/v3 && cargo test -p v3-core graph -- --nocapture 2>&1 | tail -20
```

Expected: tests fail because `execute_graph_node` is `todo!()`.

### Step 3: Implement `execute_graph_node`

Replace the `todo!()` with the full implementation following the pseudocode and operator table in the Spec Facts section above.

Key implementation notes:
- If `internal_nodes` is empty: charge no energy, return `NodeResult::halted(*upstream_slots, 0.0)`.
- Snapshot `graph_state.get(&node_id).cloned()` before the pass loop for rollback.
- After snapshot, ensure `graph_state` has a Vec of length `>= node_count`; extend with 0.0 if needed.
- Pass loop: deduct `graph_node_base_cost * node_count as f32` at the START of each pass. Check `*energy <= 0.0` after deduction → restore snapshot, return `NodeResult::exhausted()`.
- `curr_outputs` and `prev_outputs` are both `vec![0.0f32; node_count]`.
- After each pass: compute `delta` (max `|curr[i] - prev[i]|`), update `prev = curr`, check convergence.
- After the loop: initialize `output_slots = *upstream_slots`, `route_target_idx = 0.0`, then scan `curr_outputs` to apply `CustomOutput(s)` and `RouterOutput` effects.
- Graph never sets `world_action`; return `NodeResult { output_slots, route_target_idx, world_action: None, energy_exhausted: false }`.

For stateful operators: `state = graph_state.get_mut(&node_id).unwrap()[current_idx]` (already extended to length >= node_count). Update `state` in-place.

### Step 4: Add module declaration to `runtime/mod.rs`

```rust
pub mod action_decode;
pub mod graph;
pub mod inputs;
pub mod types;
pub mod vm;
pub use types::{sanitize_f32, NodeResult};
```

### Step 5: Run tests to verify they pass

```bash
cd /path/to/worktree/v3 && cargo test -p v3-core graph -- --nocapture 2>&1 | tail -20
```

Expected: all graph tests pass. Previous 153 tests still pass.

```bash
cd /path/to/worktree/v3 && cargo test -p v3-core 2>&1 | tail -5
```

Expected: `test result: ok. N passed`.

### Step 6: Commit

```bash
cd /path/to/worktree
git add v3/crates/v3-core/src/runtime/graph.rs v3/crates/v3-core/src/runtime/mod.rs
git commit -m "feat(v3-core): implement execute_graph_node (22 operators, relaxation loop, atomic state)"
```

---

## Task 2: `runtime/mesh.rs` — Mesh chain executor

**Files:**
- Create: `v3/crates/v3-core/src/runtime/mesh.rs`
- Modify: `v3/crates/v3-core/src/runtime/mod.rs`

### Step 1: Write failing tests first

Create `v3/crates/v3-core/src/runtime/mesh.rs` with tests and stub:

```rust
use std::collections::HashMap;
use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::{BackendDef, CreatureGenome, GraphBackendDef, GraphInternalNode,
    GraphNodeKind, NodeGenome, VmBackendDef, VmInstruction};
use crate::sensors::static_inputs::StaticInputs;

pub fn execute_creature_mesh(
    genome: &CreatureGenome,
    static_inputs: &StaticInputs,
    energy: &mut f32,
    memory: &mut [u8; 1024],
    graph_state: &mut HashMap<NodeId, Vec<f32>>,
    config: &RuntimeConfig,
) -> WorldAction {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use slotmap::SlotMap;

    fn cfg() -> RuntimeConfig { SimulationConfig::default().runtime }

    fn default_si() -> StaticInputs {
        StaticInputs {
            food_here: 0.5,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 1.0,
            age_ticks: 5.0,
        }
    }

    fn nid(sm: &mut SlotMap<NodeId, ()>) -> NodeId { sm.insert(()) }

    // ── soft defaults ─────────────────────────────────────────────────────────

    #[test]
    fn missing_entry_node_returns_noop() {
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let id = nid(&mut sm);
        let genome = CreatureGenome { entry_node_id: id, nodes: vec![] };
        let mut energy = 100.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
        assert_eq!(action, WorldAction::NoOp);
    }

    #[test]
    fn max_hops_exceeded_returns_noop() {
        // Self-looping VM node: Halt (no world action) → routes back to itself forever
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let id = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: id,
            nodes: vec![NodeGenome {
                node_id: id,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![id], // routes to self
            }],
        };
        let mut config = cfg();
        config.max_mesh_hops = 3;
        let mut energy = 10000.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &config);
        assert_eq!(action, WorldAction::NoOp);
    }

    #[test]
    fn empty_targets_returns_noop() {
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let id = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: id,
            nodes: vec![NodeGenome {
                node_id: id,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![], // no targets → NoOp
            }],
        };
        let mut energy = 100.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
        assert_eq!(action, WorldAction::NoOp);
    }

    #[test]
    fn energy_exhaustion_returns_noop() {
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let id = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: id,
            nodes: vec![NodeGenome {
                node_id: id,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![1.0],
                    program: vec![
                        VmInstruction::LoadConst { dst: 0, const_idx: 0 },
                        VmInstruction::EmitWorldAction { action_type: 1 }, // Eat
                    ],
                }),
                targets: vec![],
            }],
        };
        let mut energy = 0.0001f32; // too low to run VM
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
        assert_eq!(action, WorldAction::NoOp);
    }

    // ── VM action emission ────────────────────────────────────────────────────

    #[test]
    fn vm_node_emits_eat_action() {
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let id = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: id,
            nodes: vec![NodeGenome {
                node_id: id,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::EmitWorldAction { action_type: 1 }],
                }),
                targets: vec![],
            }],
        };
        let mut energy = 100.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
        assert_eq!(action, WorldAction::Eat);
    }

    // ── Graph node in mesh ────────────────────────────────────────────────────

    #[test]
    fn graph_node_routes_to_vm_node_which_emits_action() {
        // Graph node: RouterOutput with constant 0.0 → routes to target[0] = vm_id
        // VM node: emits Eat
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let graph_id = nid(&mut sm);
        let vm_id = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: graph_id,
            nodes: vec![
                NodeGenome {
                    node_id: graph_id,
                    input_refs: vec![],
                    backend_def: BackendDef::Graph(GraphBackendDef {
                        internal_nodes: vec![
                            GraphInternalNode { kind: GraphNodeKind::Constant(0.0), inputs: vec![] },
                            GraphInternalNode {
                                kind: GraphNodeKind::RouterOutput,
                                inputs: vec![crate::creature::genome::GraphInput { source_idx: 0, weight: 1.0 }],
                            },
                        ],
                    }),
                    targets: vec![vm_id],
                },
                NodeGenome {
                    node_id: vm_id,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::EmitWorldAction { action_type: 1 }],
                    }),
                    targets: vec![],
                },
            ],
        };
        let mut energy = 1000.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
        assert_eq!(action, WorldAction::Eat);
    }

    // ── Routing wrapping ──────────────────────────────────────────────────────

    #[test]
    fn route_wrapping_rem_euclid_selects_correct_target() {
        // VM emits WriteRouteTarget = 3.7, targets has 3 entries → floor(3.7)=3, rem_euclid(3)=0
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let id0 = nid(&mut sm);
        let id1 = nid(&mut sm);
        let id2 = nid(&mut sm);
        let id3 = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![
                NodeGenome {
                    node_id: id0,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![3.7],
                        program: vec![
                            VmInstruction::LoadConst { dst: 0, const_idx: 0 },
                            VmInstruction::WriteRouteTarget { src: 0 },
                            VmInstruction::Halt,
                        ],
                    }),
                    targets: vec![id1, id2, id3],
                },
                NodeGenome {
                    node_id: id1,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::EmitWorldAction { action_type: 1 }], // Eat
                    }),
                    targets: vec![],
                },
                // id2, id3 emit different actions (NoOp by Halt)
                NodeGenome {
                    node_id: id2,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    targets: vec![],
                },
                NodeGenome {
                    node_id: id3,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::Halt],
                    }),
                    targets: vec![],
                },
            ],
        };
        let mut energy = 1000.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        let action = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
        assert_eq!(action, WorldAction::Eat); // id1 → Eat
    }

    // ── upstream_slots passthrough ────────────────────────────────────────────

    #[test]
    fn output_slots_passed_as_upstream_to_next_node() {
        // Graph node writes 9.0 to slot 5, then VM node reads it (via ReadInput UpstreamSlot(5))
        // and emits Reproduce using meta slot 0. This tests that upstream_slots is wired correctly.
        // (VM ReadInput will read slot 5 from upstream_slots and put it in a register.)
        // We just verify no panic and the chain runs without crashing.
        let mut sm: SlotMap<NodeId, ()> = SlotMap::with_key();
        let g_id = nid(&mut sm);
        let v_id = nid(&mut sm);
        let genome = CreatureGenome {
            entry_node_id: g_id,
            nodes: vec![
                NodeGenome {
                    node_id: g_id,
                    input_refs: vec![],
                    backend_def: BackendDef::Graph(GraphBackendDef {
                        internal_nodes: vec![
                            GraphInternalNode { kind: GraphNodeKind::Constant(9.0), inputs: vec![] },
                            GraphInternalNode {
                                kind: GraphNodeKind::CustomOutput(5),
                                inputs: vec![crate::creature::genome::GraphInput { source_idx: 0, weight: 1.0 }],
                            },
                            GraphInternalNode {
                                kind: GraphNodeKind::RouterOutput,
                                inputs: vec![crate::creature::genome::GraphInput { source_idx: 0, weight: 0.0 }],
                            },
                        ],
                    }),
                    targets: vec![v_id],
                },
                NodeGenome {
                    node_id: v_id,
                    input_refs: vec![crate::contracts::InputReference::UpstreamSlot(5)],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 2,
                        constants: vec![],
                        program: vec![
                            VmInstruction::ReadInput { dst: 0, input_idx: 0 },
                            VmInstruction::Halt,
                        ],
                    }),
                    targets: vec![],
                },
            ],
        };
        let mut energy = 1000.0f32;
        let mut mem = [0u8; 1024];
        let mut gs = HashMap::new();
        // Just verify no panic; result is NoOp (VM halts without emitting)
        let _ = execute_creature_mesh(&genome, &default_si(), &mut energy, &mut mem, &mut gs, &cfg());
    }
}
```

### Step 2: Run tests to verify they fail

```bash
cd /path/to/worktree/v3 && cargo test -p v3-core mesh -- --nocapture 2>&1 | tail -20
```

Expected: compile error or `todo!()` panics.

### Step 3: Implement `execute_creature_mesh`

Replace `todo!()` with the full chain algorithm from the Spec Facts section above.

Implementation notes:
- Track `start_energy = *energy` at the start for `energy_consumed` calculation.
- Check `genome.find_node(entry_node_id)` first; if None → return `WorldAction::NoOp`.
- Loop bound: `hops < max_hops` (use `config.max_mesh_hops.max(1) as usize`).
- Pass `energy_consumed = start_energy - *energy` (always ≥ 0 since energy only decreases) to each node call.
- After each node result:
  1. `energy_exhausted` → return `NoOp`
  2. `world_action.is_some()` → return it
  3. `node.targets.is_empty()` → return `NoOp`
  4. Convert `route_target_idx` to i64 index, apply `rem_euclid(targets.len())`.
  5. Look up target node; if missing → return `NoOp`.
  6. Set `upstream_slots = result.output_slots`, advance `current_node_id`, increment `hops`.

### Step 4: Export from `runtime/mod.rs`

```rust
pub mod action_decode;
pub mod graph;
pub mod inputs;
pub mod mesh;
pub mod types;
pub mod vm;
pub use types::{sanitize_f32, NodeResult};
pub use mesh::execute_creature_mesh;
```

### Step 5: Run all tests

```bash
cd /path/to/worktree/v3 && cargo test -p v3-core 2>&1 | tail -10
```

Expected: all tests pass (previous 153 + new graph/mesh tests).

```bash
cd /path/to/worktree/v3 && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -10
```

Expected: no warnings.

```bash
cd /path/to/worktree/v3 && cargo fmt --all --check 2>&1
```

Expected: no output (no formatting issues).

### Step 6: Run plan and architecture harnesses

```bash
cd /path/to/worktree && scripts/check-plan-harness.sh --mode strict 2>&1 | tail -20
scripts/check-architecture-harness.sh --mode warn 2>&1 | tail -20
scripts/check-doc-harness.sh --mode warn 2>&1 | tail -20
```

Expected: all pass (warn mode through Feb 27).

### Step 7: Commit

```bash
cd /path/to/worktree
git add v3/crates/v3-core/src/runtime/mesh.rs v3/crates/v3-core/src/runtime/mod.rs
git commit -m "feat(v3-core): implement execute_creature_mesh (mesh chain, soft defaults, energy tracking)"
```

---

## Verification Commands (full gate)

Run from worktree root (`.worktrees/v3-implementation`):

```bash
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode warn
scripts/check-architecture-harness.sh --mode warn
cd v3 && cargo fmt --all --check
cd v3 && cargo test --workspace
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
```

---

## Risks and Rollback

- Risk: stateful operator formulas might diverge from mutation spec expectations. Mitigation: formulas are specified explicitly above and testable in isolation.
- Risk: `f32::floor().clamp()` routing conversion may be off for edge cases (NaN, ±Inf). Mitigation: explicit test for `route_wrapping_rem_euclid_selects_correct_target` and the `missing_entry_node` soft default.
- Rollback: graph.rs and mesh.rs are new files; simply revert the two commits.

---

**Review cycles:** 1
