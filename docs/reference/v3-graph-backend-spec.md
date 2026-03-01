# V3 Graph Backend Spec

Reference specification for `BackendDef::Graph` execution.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-runtime-config-spec.md`

---

## 1. Graph Backend Data Model

```rust
pub struct GraphBackendDef {
    pub internal_nodes: Vec<GraphInternalNode>,
}

pub struct GraphInternalNode {
    pub kind: GraphNodeKind,
    pub inputs: Vec<GraphInput>,
    pub plasticity: Option<PlasticityConfig>,
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
  for current_idx in 0..node_count:
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
  delta = max_abs(curr_outputs[i] - prev_outputs[i]) over i in 0..node_count
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
Canonical owner for graph convergence budget/config defaults:
`v3-runtime-config-spec.md`.

---

## 3. GraphNodeKind

```rust
pub enum GraphNodeKind {
    InputRef { ref_idx: u16, sub_idx: u16 },

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

`InputRef { ref_idx, sub_idx }` reads `input_refs[ref_idx]` with sub-value
index `sub_idx` and combines it with the node's internal weighted aggregate:

`output = resolve_input(input_refs[ref_idx], sub_idx, ctx) + weighted_input_sum`

For scalar inputs (all variants except `ActionQueue`), `sub_idx > 0` returns
`0.0`. For compound inputs (e.g. `ActionQueue`), `sub_idx` addresses
individual sub-values within the compound (see `v3-sensor-spec.md`).

Missing input refs read as `0.0`. Out-of-bounds `ref_idx` reads as `0.0`.

When `InputRef` resolves to `InputReference::UpstreamSlot(slot)`, the
base value is routing-parent `upstream_slots[slot]` (invalid slot -> `0.0`),
then combined with internal weighted aggregate using the same rule.

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
Canonical owner for graph runtime cost config:
`v3-runtime-config-spec.md`.

Equivalent requested energy:

```text
graph_energy_requested =
  graph_node_base_cost * internal_nodes.len() * passes_executed
```

If energy is exhausted during graph evaluation, node evaluation halts and mesh
execution returns `WorldAction::NoOp`.

---

## 7. Plasticity

### PlasticityConfig

Per-node learning configuration:

```rust
pub struct PlasticityConfig {
    pub rule: HebbianRule,        // Classic, Oja, AntiHebb, Covariance
    pub learning_rate: f32,
    pub weight_clamp: f32,
    pub lamarckian: bool,
    pub modulation: Option<RewardModulationConfig>,
}

pub struct RewardModulationConfig {
    pub reward_source: OutcomeChannel,  // EnergyDelta, ActionSuccess, DamageDelta, OffspringSuccess
    pub trace_decay: f32,               // [0.0, 1.0]
}
```

When `modulation` is `None`: pure Hebbian learning (weight updates applied
during Phase 1 post-convergence pass).

When `modulation` is `Some`: reward-modulated three-factor learning (eligibility
traces updated during Phase 1; weight updates deferred to Phase 2.5).

### Runtime state

Plasticity weights (`plasticity_weights`) and eligibility traces
(`eligibility_traces`) are stored parallel to `node_state` in
`GraphRuntimeState`. Both use `Vec<Vec<Box<[f32]>>>` layout — one `Box<[f32]>`
per node, one `f32` per input edge.

- Plasticity weights are lazy-initialized to `1.0` on first use.
- Eligibility traces are lazy-initialized to `0.0` on first use.
- Offspring start with empty traces. Plasticity weights are inherited only
  if `lamarckian` is true.

### Eligibility trace update (Phase 1, post-convergence)

After graph convergence, for each reward-modulated node:

```text
trace[edge] = decay * old_trace + hebbian_delta(pre, post, weight)
```

where `hebbian_delta` applies the node's `HebbianRule` (Classic, Oja, etc.).

### Reward-modulated weight update (Phase 2.5)

```text
dw = learning_rate * outcome_signal[reward_source] * trace[edge]
new_weight = clamp(weight + dw, -weight_clamp, weight_clamp)
```

---

## 8. Backend-Local Soft Defaults

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
