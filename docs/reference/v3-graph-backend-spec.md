# V3 Graph Backend Spec

Reference specification for `BackendDef::Graph` execution using CGP-style
layered architecture.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-runtime-config-spec.md`

The graph backend is subject to the mesh-wide node-type evolvability contract
in `v3-mutation-spec.md`. T11.F03 implements its growth-versus-connection
taxonomy and neutral-growth semantics (`AddComputeNode`'s three forms,
`CopyComputeNode`'s faithful-copy rule, `InputRef.Add`'s unwired push, and the
`insert_compute_node_at`/`remove_compute_node_at` index-remap pair); the
taxonomy and per-operator contract text live in `v3-mutation-spec.md` rather
than being duplicated here. T11.F06 owns its one-world-tick persistent-state
clock.

---

## 1. Architecture Overview

The graph backend uses a three-layer CGP (Cartesian Genetic Programming) model:

1. **Implicit inputs** — sensor data and shared memory reads are addressable
   sources (`GraphSource`), not physical nodes.
2. **Compute nodes** — mutable computation layer with free topology mutations.
   Supports recurrence through frozen world-tick outputs.
3. **Fixed structural outputs** — value sinks, action bank, and execute gate.
   Structurally immutable (always present); only edges TO them are evolvable.

This separation eliminates the class of bugs where topology mutations corrupt
structural invariants (sensor nodes, output nodes). Every topology mutation
produces a structurally valid genome.

---

## 2. Graph Backend Data Model

```rust
pub struct GraphBackendDef {
    pub compute_nodes: Vec<ComputeNode>,
    pub output_sinks: Vec<OutputSink>,
    pub action_bank: Vec<ActionSlot>,
    pub execute_gate: ExecuteGate,
}

pub struct ComputeNode {
    pub kind: ComputeNodeKind,
    pub inputs: Vec<GraphEdge>,
    pub plasticity: Option<PlasticityConfig>,
}

pub struct OutputSink {
    pub kind: OutputSinkKind,
    pub inputs: Vec<GraphEdge>,
}

pub struct ActionSlot {
    pub behavior: ActionSlotBehavior,
    pub gate_inputs: Vec<GraphEdge>,
    pub param_inputs: Vec<GraphEdge>,
}

pub struct ExecuteGate {
    pub inputs: Vec<GraphEdge>,
}

pub struct GraphEdge {
    pub source: GraphSource,
    pub weight: f32,
}
```

Design choice: edges are co-located with each node/sink for linear,
cache-friendly evaluation. Fixed structural outputs are stored in separate
Vecs to prevent topology mutations from affecting them.

---

## 3. GraphSource (Edge Addressing)

```rust
pub enum GraphSource {
    InputLeaf { ref_idx: u16, sub_idx: u16 },
    SharedMemory { slot: u8, previous: bool },
    ComputeNode(u16),
}
```

- `InputLeaf { ref_idx, sub_idx }` — reads `input_refs[ref_idx]` with
  sub-value index `sub_idx`. Replaces the old `InputRef` node kind.
- `SharedMemory { slot, previous }` — reads `shared_memory[slot]` (current)
  or `prev_shared_memory[slot]` (previous tick). Replaces `ReadSlot`/
  `ReadSlotPrev` node kinds.
- `ComputeNode(idx)` — reads the output of `compute_nodes[idx]`.

---

## 4. ComputeNodeKind

```rust
pub enum ComputeNodeKind {
    // Arithmetic
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    WeightedSum,

    // Activation
    Sigmoid,
    Tanh,
    Relu,
    Clamp01,
    Threshold(f32),

    // Logic
    GreaterThan,
    Select,

    // Stateful
    DecayIntegrator(f32),
    Momentum(f32),
    Oscillator(f32),
    AdaptiveGain,

    // Constant
    Constant(f32),
}
```

17 variants (down from 30 in the mixed-node model). Only computation-relevant
kinds — no input/output/shared-memory kinds.

### Node class taxonomy

| Class | Kinds | Visual role |
|-------|-------|-------------|
| Arithmetic | Add, Multiply, Negate, Abs, Min, Max, WeightedSum | Pure math |
| Activation | Sigmoid, Tanh, Relu, Clamp01, Threshold | Nonlinear transforms |
| Logic | GreaterThan, Select | Decision/gating |
| Stateful | DecayIntegrator, Momentum, Oscillator, AdaptiveGain | Per-tick memory |
| Constant | Constant(f32) | Fixed value source |

---

## 5. OutputSinkKind

```rust
pub enum OutputSinkKind {
    CustomOutput(u8),   // 12 slots, indices 0-11
    RouterGate(u8),     // eight routing gate scores, indices 0-7
    WriteSlot(u8),      // 16 slots, indices 0-15: shared memory write
    ClearSlot(u8),      // 16 slots, indices 0-15: shared memory clear
}
```

The full sink catalog is fixed at genome construction: 12 CustomOutput + 8
RouterGate + 16 WriteSlot + 16 ClearSlot = 52 sinks. Mutations can only
modify edges TO sinks, not add/remove/change sink kinds.

### Inert-when-unwired rule

A sink with empty `inputs` does NOT write its target. It preserves the
upstream/default value:
- CustomOutput with no edges: `output_slots[slot]` retains incoming
  `upstream_slots[slot]`.
- WriteSlot/ClearSlot with no edges: `shared_memory[slot]` is unchanged.
- RouterGate with no edges: that slot retains its zero gate score.

Only sinks with at least one edge compute their weighted-sum and write
the result.

---

## 6. ActionSlotBehavior and WorldActionKind

```rust
pub enum ActionSlotBehavior {
    Pop,
    Emit(WorldActionKind),
}

pub enum WorldActionKind {
    Eat,
    Move,
    Reproduce,
    StealEnergy,
    NoOp,
}
```

- `Pop` — queue-program-control: removes the last queued action.
- `Emit(kind)` — world-action-payload: decodes a world action from `kind` +
  the slot's `param_inputs` and pushes to the queue.

Behavior is evolvable via raw field mutation (not edge-computed).

### Action parameter decoding

When an `Emit(kind)` slot fires, parameters are decoded from `param_inputs`
weighted sums:

| `WorldActionKind` | `param[0]` | `param[1]` | Notes |
|---|---|---|---|
| `Eat` | — | — | No params |
| `Move` | direction index (0-7) | — | `round().clamp(0, 7)` |
| `Reproduce` | direction index (0-7) | offspring energy | Non-negative |
| `StealEnergy` | direction index (0-7) | steal amount | Non-negative |
| `NoOp` | — | — | Real action with costs |

Direction decoding matches the VM `PushAction` convention:
`meta[0].round().clamp(0.0, 7.0)` maps to `Direction::ALL`.

---

## 7. ExecuteGate

```rust
pub struct ExecuteGate {
    pub inputs: Vec<GraphEdge>,
}
```

Separate gated output controlling mesh termination. The gate fires when
`wsum(inputs) > 0.0` AND the action queue is non-empty. When fired, the mesh
hop terminates and returns the accumulated action queue for execution.

If inputs are empty, `wsum = 0.0` — gate doesn't fire. Terminal behavior is
evolvable only through edge mutations on the execute gate.

---

## 8. Evaluation Order and World-Tick Clock

Phase 0 calls `graph_runtime.begin_tick(&genome.nodes)`, also used by neighborhood
sequences, standalone multi-tick callers, and cloned observations. It snapshots
committed operator state and outputs and decays initialized eligibility. Mesh entry and
module visits do not advance this clock. Each nonempty graph visit evaluates
all compute nodes once in index order, including disconnected nodes.

- Lower-index compute sources read the current visit's computed outputs.
- Self and higher-index sources read frozen tick-start outputs.
- Stateful operators start from frozen tick-start operator state.
- A successful visit commits candidate state and outputs. Production mesh
  dispatch visits each node at most once per tick. Direct backend harness
  re-entry still recomputes from the frozen base; its last successful call
  supplies next tick's state. Effects and learning remain per backend call.
- Unvisited modules hold values without catch-up, fabricated inputs, or charge.
- Newborn temporal state is zero. Empty graphs do no work.

The former relaxation and convergence config fields remain accepted and
validated but are ignored by evaluation and allocation; see
`v3-runtime-config-spec.md`. There is no convergence loop.

---

## 9. Post-Evaluation Effects

After the ordered evaluation and its plasticity cost are affordable, a
three-phase effects pass processes all fixed structural outputs.

### Phase 1: Value outputs

Iterate `output_sinks`. For each sink with non-empty `inputs`:
- Gather `wsum = sum(resolve_source(edge.source) * edge.weight)` using
  current-visit `curr_outputs` for `ComputeNode` sources.
- Write to target:
  - `CustomOutput(slot)`: `output_slots[slot] = wsum` (slot < 12).
  - `RouterOutput`: emit `RouteDecision::CgpNormalized { raw_value: wsum }`.
    Mesh routing resolves that decision with
    `idx = min(floor(clamp01(raw_value) * target_count), target_count - 1)`.
  - `WriteSlot(slot)`: `shared_memory[slot % 16] = sanitize_f32(wsum)`.
  - `ClearSlot(slot)`: `shared_memory[slot % 16] = 0.0` (wsum is ignored;
    the act of having edges and firing is what clears).

Sinks with empty `inputs` are inert — no write occurs.

### Phase 2: Action bank scan

Iterate `action_bank` in order (index 0 to N-1). For each slot:
1. Gather `gate_wsum = sum(resolve_source(e.source) * e.weight)` from
   `gate_inputs`.
2. If `gate_wsum > 0.0` (slot fires):
   - If `behavior` is `Pop`: remove last queued item. No-op if queue empty.
   - If `behavior` is `Emit(kind)`: gather `param_wsum[i]` from
     `param_inputs`, decode world action from `kind` + params, push to queue.
3. If `gate_inputs` is empty: `gate_wsum = 0.0`, slot doesn't fire.

### Phase 3: Execute gate

Gather `gate_wsum` from `execute_gate.inputs`. If `gate_wsum > 0.0` AND
queue is non-empty: mark hop as terminal.

If `execute_gate.inputs` is empty: `gate_wsum = 0.0`, hop doesn't terminate.

### Effect-phase trace visibility (Execution Sampler)

Graph sampler traces include post-evaluation structural-layer records in
addition to the single entered evaluation:
- `output_sinks: Vec<GraphOutputSinkTrace>` (1:1 with `output_sinks`)
  - fields: `wired`, `weighted_sum`, `applied`, `applied_value`
- `action_slots: Vec<GraphActionSlotTrace>` (1:1 with `action_bank`)
  - fields: `wired`, `gate_weighted_sum`, `fired`, `param_values`,
    `queue_len_before`, `queue_len_after`, `emitted_action`
- `execute_gate: GraphExecuteGateTrace`
  - fields: `wired`, `weighted_sum`, `queue_non_empty`, `fired`

These records are always index-aligned with structural catalogs (not sparse
"only fired" lists), so UI/debug tooling can compare wired-but-inert vs active
behavior directly.

---

## 10. Routing Gates

Each `RouterGate(slot)` sink writes its weighted sum to one of eight gate
slots. The mesh chooses the earliest maximum `gate_bias + gate_score` among
unvisited destination IDs, with the shared soft-float policy. A visited top
choice falls through; missing winners terminate softly. T11.F15 route addition
pairs a tied branch through a pass-through detour with one weight-1 gate edge
sampled by `random_graph_source`, including full input sub-value support.

---

## 11. Stateful Operators and `graph_state`

`GraphRuntimeState` stores committed `node_state` and `node_outputs`, indexed
by `[mesh_node_idx][compute_node_idx]`, and frozen tick-start copies of both.
Missing slots read zero and storage is initialized lazily. Offspring receives
fresh empty temporal vectors, regardless of learned-weight inheritance.

Trace `passes` contains one entered evaluation, including an unaffordable
attempt. `node_evaluations` describe candidate computation; `temporal_committed`
marks whether that candidate was applied. `final_outputs` always reports the
last successful committed outputs. Legacy `converged` and
`stable_passes_count` are always false and zero. `max_delta` compares the
candidate outputs with the frozen tick-start outputs, not a convergence test.

---

## 12. Energy Cost

Graph node cost is charged per compute-node-per-visit evaluation
(`graph_node_base_cost` or equivalent config-driven scalar).
Canonical owner for graph runtime cost config:
`v3-runtime-config-spec.md`.

Equivalent requested energy:

```text
graph_energy_requested =
  graph_node_base_cost * compute_nodes.len() * entered_visits
```

If energy is exhausted during graph evaluation, node evaluation halts and mesh
execution returns `WorldAction::NoOp`. Evaluation-cost and plasticity-cost
exhaustion preserve the prior successful temporal state and outputs and emit
no graph effects. Actual charge and entered work remain recorded; learned
pure Hebbian weight and reward rollback are outside this temporal transaction.
Reward-trace activity commits at this same successful boundary.

Note: fixed structural outputs (sinks, action bank, execute gate) are not
counted in the per-visit energy cost. They are evaluated once post-evaluation.

---

## 13. Plasticity

### PlasticityConfig

Per-node learning configuration (applies to compute nodes only):

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
during Phase 1 post-evaluation pass).

When `modulation` is `Some`: reward-modulated three-factor learning (eligibility
traces updated during Phase 1; weight updates deferred to Phase 2.5).

### Runtime state

Plasticity weights (`plasticity_weights`) and eligibility traces
(`eligibility_traces`) are stored parallel to `node_state` in
`GraphRuntimeState`. Both use `Vec<Vec<Box<[f32]>>>` layout — one `Box<[f32]>`
per compute node, one `f32` per input edge.

- Plasticity weights are lazy-initialized from each genome edge weight on first use.
- Eligibility traces are lazy-initialized to `0.0` on first use.
- Offspring start with empty traces. Plasticity weights are inherited only
  if `lamarckian` is true.

### Eligibility trace clock and activity (Phases 0 and 1)

At world-tick start, every initialized reward-modulated edge decays once,
including modules never visited this tick. The decayed base is frozen:

```text
base_t = clamp(trace_decay, 0, 1) * trace_(t-1)
trace_t = base_t + activity(pre, post, weight)
```

Each successful visit replaces its activity contribution from that base;
repeated visits do not add decay or duplicate credit. A skipped tick leaves
only the base. Lambda 0 forgets prior credit; lambda 1 retains it. An isolated
pulse is discounted by lambda^d after d skipped ticks. Clock bookkeeping
never initializes unvisited weights/traces or adds energy/work charges.

Activity excludes learning rate: Classic `pre*post`, Oja
`post*(pre-weight*post)`, AntiHebb `-pre*post`, Covariance
`(pre-0.5)*(post-0.5)`. It uses the unweighted source actually evaluated,
the node's resulting output and effective evaluation weight. Lower-index
sources use current-visit outputs; self/higher sources use frozen tick-start
outputs. Input context includes evaluation-time energy, before later pure
Hebbian costs. Pure Hebbian updates retain their existing semantics.

Commit activity only after evaluation and plasticity costs are affordable,
before graph effects. A failed first visit leaves only the decayed base;
a failed revisit preserves the last successful contribution. Elapsed-time
decay is never rolled back. Existing charges and work counters still apply.

### Reward-modulated weight update (Phase 2.5)

```text
dw = learning_rate * outcome_signal[reward_source] * trace[edge]
new_weight = clamp(weight + dw, -weight_clamp, weight_clamp)
```

Learning rate is clamped to [0,1] and weight-clamp magnitude to [0.01,10].
Eta applies once: unit activity/reward at eta 0.5 yields trace 1 and delta
0.5. At lambda 0 and unit reward, a feed-forward edge matches its pure
Hebbian update. Phase 2.5 updates every initialized modulated edge each tick,
even on skipped visits or zero deltas, retaining configured cost/work
semantics. Reward does not clear eligibility.

---

## 14. Three-Tier Liveness Model

Clear terminology to avoid conflating structural presence with functional
contribution:

- **Structurally present**: exists in the genome by construction. All fixed
  sinks, all action slots, and the execute gate are always structurally present.
- **Wired**: has at least one edge. Only wired items write values / fire
  actions. Only wired items count toward `genome_size()` and
  `functional_complexity()`.
- **Functionally reachable** ("live"): compute nodes backward-reachable from
  any wired sink, wired action slot gate/param, or wired execute gate inputs.
  "Live" is reserved for behaviorally contributing items only — unwired sinks
  are structurally present but NOT live.

Dormant (unwired) sinks and action slots are NOT counted toward
`genome_size()` or `functional_complexity()`. This prevents the fixed catalog
from imposing a constant complexity tax.

---

## 15. Fixed Output Catalog Construction

`GraphBackendDef::new_with_fixed_outputs(config)` constructs:
- 12 `CustomOutput(0..11)` sinks
- 1 `RouterOutput` sink
- 16 `WriteSlot(0..15)` sinks
- 16 `ClearSlot(0..15)` sinks
- `action_bank` of `config.action_queue_cap` empty `ActionSlot`s
- Empty `ExecuteGate`

All sinks and action slots start with empty edge Vecs (inert until evolution
wires them).

### Queue-size contract

`action_bank.len()` is derived from `MutationConfig::action_queue_cap`
(default 4). This gives a single source of truth for queue shape:
- `action_bank.len() == config.action_queue_cap` at genome creation time.
- `InputReference::ActionQueue` width = `action_queue_cap * 3`.
- Invariant: `action_queue_cap <= max_actions_per_turn`.
- Normalization order: `max_actions_per_turn` is normalized first, then
  `action_queue_cap` is clamped to
  `1..=min(21845, max_actions_per_turn)`.

Config-change policy: frozen at creation. `action_bank.len()` is set when the
genome is created and never changes. If `action_queue_cap` changes
mid-simulation, existing creatures keep their original bank size.

---

## 16. Soft-Default Matrix

Two-layer validation: mutation-time (bound values at creation) and runtime
(silent 0.0 fallback).

| Situation | Behavior |
|-----------|----------|
| `ComputeNode(idx)` where `idx >= compute_nodes.len()` | Resolve to 0.0 |
| `InputLeaf { ref_idx }` where `ref_idx >= input_refs.len()` | Resolve to 0.0 |
| `InputLeaf { sub_idx }` where `sub_idx >= sub_value_count` | Resolve to 0.0 |
| `SharedMemory { slot }` where `slot >= 16` | Bound at mutation time to 0-15; defensive fallback: 0.0 |
| `CustomOutput(s)` where `s >= 12` | Constructor invariant; defensive fallback: 0.0 |
| `Pop` on empty queue | Silent no-op |
| ActionSlot with empty `gate_inputs` | gate wsum = 0.0, slot doesn't fire |
| ActionSlot with empty `param_inputs` | all params = 0.0 |
| ExecuteGate with empty `inputs` | wsum = 0.0, hop doesn't terminate |
| Edge with NaN/Inf weight | `sanitize_f32()` to 0.0 |
| Sink with empty `inputs` | Inert — does not write |
| `Emit(NoOp)` that fires | Enqueues `WorldAction::NoOp` (real action with costs) |
| Direct backend re-entry in one tick | Recompute from frozen tick-start state; production mesh disallows redispatch |
| Missing graph state | Lazily initialized to zeros |

Cross-runtime fallback outcomes are canonical in
`v3-mesh-execution-spec.md` (Section 4, authoritative soft-default matrix).
