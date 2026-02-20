# V3 Graph Backend Spec

Reference specification for `BackendDef::Graph` execution.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`

---

## 1. Graph Backend Data Model

```rust
pub struct GraphBackendDef {
    pub internal_nodes: Vec<GraphInternalNode>,
}

pub struct GraphInternalNode {
    pub kind: GraphNodeKind,
    pub inputs: Vec<GraphInput>,
}

pub struct GraphInput {
    pub source_idx: u16,
    pub weight: f32,
}
```

Design choice: edges are co-located with each internal node for linear,
cache-friendly evaluation.

---

## 2. Evaluation Order, Recurrence, and Convergence

Graph evaluation uses bounded relaxation passes to support internal recurrence
(including backward and self edges) without unbounded runtime.

Per node evaluation:

```text
node_count = internal_nodes.len()
prev_outputs = [0.0; node_count]
curr_outputs = [0.0; node_count]

max_graph_relax_iters =
  validated(config.max_graph_relax_iters, default=4, min=1)
graph_convergence_epsilon =
  validated(config.graph_convergence_epsilon, default=1e-3, min=0.0)
graph_convergence_stable_passes =
  validated(config.graph_convergence_stable_passes, default=1, min=1)

stable_passes = 0
passes_executed = 0

for pass in 0..max_graph_relax_iters:
  for current_idx in 0..node_count-1:
    weighted_input_sum = 0.0
    for input in internal_nodes[current_idx].inputs:
      source_idx = input.source_idx as usize
      source_value =
        if source_idx >= node_count:
          0.0
        else if source_idx < current_idx:
          curr_outputs[source_idx]   // already updated this pass
        else:
          prev_outputs[source_idx]   // self/backward/not-yet-updated

      weighted_input_sum += source_value * input.weight

    curr_outputs[current_idx] =
      evaluate_kind(internal_nodes[current_idx].kind, weighted_input_sum)

  passes_executed += 1
  delta = max_abs(curr_outputs[i] - prev_outputs[i]) over i in 0..node_count-1
  prev_outputs = curr_outputs

  if delta <= graph_convergence_epsilon:
    stable_passes += 1
  else:
    stable_passes = 0

  if stable_passes >= graph_convergence_stable_passes:
    break
```

This is "iterate until convergence or budget exhaustion." It is intentionally
bounded by `max_graph_relax_iters` to prevent infinite internal loops.

---

## 3. GraphNodeKind

```rust
pub enum GraphNodeKind {
    InputRef(u8),
    InputUpstreamSlot(u8),

    Constant(f32),
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    Threshold(f32),
    GreaterThan,
    Sigmoid,
    Tanh,
    Relu,
    Select,
    Clamp01,
    WeightedSum,

    DecayIntegrator(f32),
    Momentum(f32),
    Oscillator(f32),
    AdaptiveGain,

    CustomOutput(u8),
    RouterOutput,
}
```

`InputRef(u8)` reads through `NodeGenome.input_refs` and combines it with the
node's internal weighted aggregate:

`output = input_ref_value + weighted_input_sum`

Missing input refs read as `0.0`.

`InputUpstreamSlot(u8)` reads from routing parent output slots and combines that
base value with internal weighted aggregate:

`output = upstream_slot_value + weighted_input_sum`

Invalid slot reads yield `0.0`.

---

## 4. Stateful Operators and `graph_state`

Graph state is keyed by mesh `NodeId`:

```rust
HashMap<NodeId, Vec<f32>>
```

Stateful operators use fixed slot mapping by internal node index.
Example: internal node `i` uses state slot `i` in the owning mesh node state
vector.

Rules:
- If state vector for `NodeId` is missing: allocate zeroed vector.
- If vector is shorter than needed index: extend with zeros.
- State persists across ticks for a living creature.
- Offspring starts with fresh zeroed graph state.

---

## 5. Outputs and Routing

During graph evaluation:
- Graph `output_slots` buffer is initialized from incoming `upstream_slots`.
- `CustomOutput(slot)` writes into `output_slots[slot]` when `slot < 12`.
- Invalid custom output slot writes are ignored (slot value is unchanged).
- `RouterOutput` writes candidate route value to `route_target_idx`.
- If multiple `RouterOutput` nodes execute, last-write-wins.

Graph backend never emits `WorldAction` directly.

---

## 6. Energy Cost

Graph node cost is charged per internal-node-per-pass evaluation
(`graph_node_base_cost` or equivalent config-driven scalar).

Equivalent requested energy:

```text
graph_energy_requested =
  graph_node_base_cost * internal_nodes.len() * passes_executed
```

If energy is exhausted during graph evaluation, node evaluation halts and mesh
execution returns `WorldAction::NoOp`.

---

## 7. Backend-Local Soft Defaults

Determinism scope is canonical in `AGENTS.md`; V3 harness reproducibility
controls are specified in `v3-mesh-execution-spec.md`
(`Test-Mode Reproducibility Notes`).

Cross-runtime fallback outcomes are canonical in
`v3-mesh-execution-spec.md` (Section 4, authoritative soft-default matrix).

Backend-local soft defaults:
- Invalid edges read as `0.0`.
- Missing input refs read as `0.0`.
- Missing state lazily initialized to zeros.
- Non-converged graphs at iteration cap use the last computed pass output.
