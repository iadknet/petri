# V3 Genome Spec

Reference specification for the V3 creature mesh genome structure.

Status: Active

Related references:
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`

---

## 1. Top-Level Genome

```rust
pub struct CreatureGenome {
    pub entry_node_id: NodeId,
    pub nodes: Vec<NodeGenome>,
}
```

`entry_node_id` is the first node id attempted by mesh execution each tick.

---

## 2. Node Genome

```rust
pub struct NodeGenome {
    pub node_id: NodeId,
    pub input_refs: Vec<InputReference>,
    pub backend_def: BackendDef,
    pub targets: Vec<NodeId>,
}
```

Semantics:
- `input_refs` is shared indirection for VM and Graph backends.
- `targets` are candidate route destinations chosen at runtime.
- `targets` may include dangling ids (junk DNA); runtime handles safely.

---

## 3. Backend Definitions

```rust
pub enum BackendDef {
    Vm(VmBackendDef),
    Graph(GraphBackendDef),
}
```

`VmBackendDef` is defined in `v3-vm-isa-spec.md`.
`GraphBackendDef` is defined in `v3-graph-backend-spec.md`.
Topology examples are documented in `v3-genome-topology-examples.md`.

---

## 4. Validation Scope

Genome validation enforces parseability, not full logical viability.

This section is the authoritative parseability invariant set for V3 genome
validation and mutation `ParseabilityGate`.

Required invariants:
- `nodes` is non-empty.
- `node_id` values are unique.
- backend payloads remain decodable.

Deliberately not required at validation time:
- `entry_node_id` must resolve.
- Every `targets` id must resolve.
- Graph nodes must have non-empty targets.

These are handled by mesh runtime soft defaults to preserve evolutionary
freedom; the authoritative chain-level fallback matrix is in
`v3-mesh-execution-spec.md` (Section 4).

---

## 5. Runtime-Adjacent State Mapping

`CreatureState` stores graph-local runtime state separately from genome data:

```rust
pub struct CreatureState {
    pub graph_state: HashMap<NodeId, Vec<f32>>,
    // ...
}
```

The key is mesh `NodeId` (not global flat offsets). This avoids state/index
misalignment across topology mutations.

---

## 6. Output Slot Contract

- Every node yields `output_slots: [f32; 12]` in `NodeResult`.
- On the first hop, incoming `upstream_slots` is `[0.0; 12]`.
- On subsequent hops, incoming `upstream_slots` is the prior node's
  `NodeResult.output_slots`.
- Per node evaluation, `output_slots` is initialized from incoming
  `upstream_slots`; unwritten slots pass through unchanged.
- Slots are runtime dataflow values, not persisted in genome.
