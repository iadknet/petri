# V3 Genome Spec

Reference specification for the V3 creature mesh genome structure.

Status: Active

Related references:
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-sensor-spec.md`

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
- `input_refs` is shared indirection for VM and Graph backends. An entry
  keeps its kind (mesh read class and compound width) for the node's life and
  its copies': mutation replaces it only with another member of the same
  kind, and deletes it only while no edge or `ReadInput` addresses it
  (T11.F22).
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
`GraphBackendDef` is defined in `v3-graph-backend-spec.md`. The graph backend
uses a CGP-style layered model with implicit inputs, mutable compute nodes,
and fixed structural outputs (the sinks). The fixed catalog holds 106 sinks: 24 `CustomOutput`, 8
`RouterGate`, 16 `WriteSlot`, 16 `ClearSlot`, 39 `ActionVote` (four kind drives included, T11.F30, T27.F03), and 3
`ActionParam`, one per decoded parameter field (T11.F27, T02.F08, T27.F03); the vote and parameter sinks are how a graph acts
(`v3-mesh-execution-spec.md` Section 2) and are drawable by mutation.
`genome_size` charges one unit per compute edge and per sink edge with no
membership and nothing for a sink itself, so unwired sinks add nothing to it
(T25.F02), and one unit per edge-group record with nothing for its members of
either role (T25.F03, T25.F10: a gene costs one unit, an explicit economic
discount); the founder carries no group and is 80 units.
A graph edge serializes as `{source, own_weight, plasticity?, membership?}`
(T25.F06: learning attributes live on the edge; a compute node is `{kind,
inputs}`, and a genome with a node-level `plasticity` key fails to
deserialize, with no conversion path) and a graph
backend with edge groups carries `groups: [{shape: {Ring: {ring_ref}}, kind,
offset, scale, inherit_scale, retention}]` (T25.F03, T25.F07, T25.F09), each member's
`membership` is `{group, scope}` with `scope` `"Edge"` or `"Group"` (T25.F07;
both keys always written, so a genome from before T25.F07 fails to
deserialize, with no conversion path); every `plasticity` config and every
group carries `retention` in `[0, 1]`, always written (T25.F09, genome file
format 6); the `cgp` genome types deny unknown fields, so a
genome from before T25.F03 (`weight`, `projections`) fails to deserialize,
with no conversion path. A genome from before T25.F05 that names
`NearbyCreatureCore`, `NearbyCreatureVitals` or `NearbyCreatureIdentity`
fails to deserialize too; the one `NearbyCreature` reference (36 channels)
replaces them. The lab's genome files are `genome_format` 4 (T25.F06);
format 3 and earlier files are refused.
A serialized genome from before T11.F27 that carries an old parameter sink
(`{"ActionParam":["Eat",0]}`) or an old `WriteActionParam { slot_idx, .. }`
fails to deserialize; there is no conversion path. A pre-T11.F27 genome with
neither deserializes unchanged.
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
- Graph backend fixed output catalog is structurally intact (correct sink
  count and sink kinds match the canonical catalog).
- Every Graph backend passes `validate_graph` (T25.F03, T25.F06): the
  edge-group invariants of `v3-graph-backend-spec.md` ("Ring edge groups"),
  every learning attribute in range, and no plastic `ClearSlot` edge or group
  member (Section 13 there), checked in the mutation gate after every event
  and at the lab's `GenomeFile::load`.

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

Within each mesh node, operator state, committed outputs and scratch
buffers are indexed by compute node index. Learned state (T25.F06) is one
slot per plastic edge over both surfaces in canonical order — a sink edge can
learn — and, since T25.F07, one scale slot per edge group with a
group-scope learner, in `GroupId` order, built once at birth
(`v3-graph-backend-spec.md` Section 13). A member's `scope` and a group's
`inherit_scale` and `retention` and an edge's `retention` are genome fields;
the learned scale is runtime state. Fixed
structural outputs (the sinks) carry no operator state.

---

## 6. Output Slot Contract

- Every node yields `output_slots: [f32; 12]` in `NodeResult`.
- On the first hop, incoming `upstream_slots` is `[0.0; 12]`.
- On subsequent hops, incoming `upstream_slots` is the prior node's
  `NodeResult.output_slots`.
- Per node evaluation, `output_slots` is initialized from incoming
  `upstream_slots`; unwritten slots pass through unchanged.
- Slots are runtime dataflow values, not persisted in genome.
