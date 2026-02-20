# V3 Graph Backend Spec

Reference specification for `BackendDef::Graph` execution.

Status: Active

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

## 2. Evaluation Order and Input Semantics

Internal nodes are evaluated in array order (`0..N-1`).

For an internal node at `current_idx`:
- Each input reads source output from `source_idx`.
- Contribution is `source_value * weight`.
- Invalid `source_idx` (`>= current_idx` or out of bounds) contributes `0.0`.

This prevents crashes from mutations and forbids backward edge dependency during
single-pass evaluation.

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

`InputRef(u8)` reads through `NodeGenome.input_refs`.

`InputUpstreamSlot(u8)` reads from routing parent output slots. Invalid slot
reads yield `0.0`.

---

## 4. Stateful Operators and `graph_state`

Graph state is keyed by mesh `NodeId`:

```rust
HashMap<NodeId, Vec<f32>>
```

Stateful operators use deterministic slot mapping by internal node index.
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
- `CustomOutput(slot)` writes into `output_slots[slot]` when `slot < 12`.
- Invalid custom output slot writes are ignored.
- `RouterOutput` writes candidate route value to `route_target_idx`.
- If multiple `RouterOutput` nodes execute, last-write-wins.

Graph backend never emits `WorldAction` directly.

---

## 6. Energy Cost

Graph node cost is charged per internal node evaluation (`graph_node_base_cost`
or equivalent config-driven scalar).

If energy is exhausted during graph evaluation, node evaluation halts and mesh
execution returns `WorldAction::NoOp`.

---

## 7. Determinism and Soft Defaults

Determinism requirements:
- Stable internal node order.
- Deterministic numeric sanitation rules.
- Deterministic last-write-wins behavior.

Soft defaults:
- Invalid edges read as `0.0`.
- Missing input refs read as `0.0`.
- Missing state lazily initialized to zeros.
