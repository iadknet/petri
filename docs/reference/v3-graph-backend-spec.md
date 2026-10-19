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
`CopyComputeNode`'s faithful-copy rule, `InputRef.Add`'s unwired push (a
Graph-node `Add` wires the new reference into one `ActionVote` sink since
T20.F16, so on this backend it is a connection event), and the
`insert_compute_node_at`/`remove_compute_node_at` index-remap pair); the
taxonomy and per-operator contract text live in `v3-mutation-spec.md` rather
than being duplicated here. T11.F08 adds duplication placement
(`duplicate_compute_nodes_in_place`: each copy directly after its original, an
edge between duplicated nodes following the copies; its split exclusion for a
sink's direct `DynamicIntrospection(EnergyCurrent)` read went with the
learning charge at T25.F02), which is what keeps a copied edge on the
evaluation phase Section 8's ordered pass gives it. T11.F06 established the
committed-state clock and T19.F02 made it per visit.

---

## 1. Architecture Overview

The graph backend uses a three-layer CGP (Cartesian Genetic Programming) model:

1. **Implicit inputs** — sensor data and shared memory reads are addressable
   sources (`GraphSource`), not physical nodes.
2. **Compute nodes** — mutable computation layer with free topology mutations.
   Supports recurrence through the last committed outputs.
3. **Fixed structural outputs** — value, routing, memory, vote, and
   parameter sinks. Structurally immutable (always present); only edges TO
   them are evolvable.

This separation eliminates the class of bugs where topology mutations corrupt
structural invariants (sensor nodes, output nodes). Every topology mutation
produces a structurally valid genome.

---

## 2. Graph Backend Data Model

```rust
pub struct GraphBackendDef {
    compute_nodes: Vec<ComputeNode>,
    output_sinks: Vec<OutputSink>,
    /// Edge-group records by `GroupId` (T25.F03); omitted from JSON when empty.
    groups: Vec<Group>,
}

pub struct Group {
    shape: Shape,                    // Ring { ring_ref: u16 }
    kind: DirectedVoteKind,          // Move | Reproduce | StealEnergy
    offset: u8,                      // always < 8
    scale: f32,                      // one shared weight, born 0
    inherit_scale: bool,             // a parent's learned scale seeds the child (T25.F07), born false
}

pub struct Membership {
    group: GroupId,
    scope: Scope,                    // Edge | Group (T25.F07), born Edge
}

pub struct ComputeNode {
    pub kind: ComputeNodeKind,
    pub inputs: Vec<GraphEdge>,
}

pub struct OutputSink {
    pub kind: OutputSinkKind,
    pub inputs: Vec<GraphEdge>,
}

pub struct GraphEdge {
    source: GraphSource,
    own_weight: f32,                 // JSON key `own_weight`
    plasticity: Option<PlasticityConfig>, // Section 13; omitted when None
    membership: Option<Membership>,  // { group: GroupId, scope: Scope }; omitted when None
}                                    // 36 bytes (T25.F06)
```

Design choice: edges are co-located with each node/sink for linear,
cache-friendly evaluation. Fixed structural outputs are stored in separate
Vecs to prevent topology mutations from affecting them. Fields are private;
the container (`cgp/container.rs`) is the only writer and keeps the birth
seeds and the group invariants aligned; `EdgeMut::set_plasticity` is the only
writer of an edge's learning attributes. A group's members are ordinary sink
edges (Section 6). Equality, the `Debug` genome identity and JSON cover every
heritable field and nothing else (`Debug` prints `groups` unconditionally);
the `cgp` genome types deny unknown JSON fields.

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
    CustomOutput(u8),        // 24 slots, indices 0-23
    RouterGate(u8),          // eight routing gate scores, indices 0-7
    WriteSlot(u8),           // 16 slots, indices 0-15: shared memory write
    ClearSlot(u8),           // 16 slots, indices 0-15: shared memory clear
    ActionVote(VoteSink),    // 30 vote sinks
    ActionParam(ActionParamField), // 3 decoded parameter fields
}
```

The full sink catalog is fixed at genome construction: 24 CustomOutput + 8
RouterGate + 16 WriteSlot + 16 ClearSlot + 30 ActionVote + 4 ActionParam = 98
sinks. Mutations can only modify edges TO sinks, not add/remove/change sink
kinds.

`VoteSink` is the fixed vote catalog: `Eat` (index 0), `Move(d)` (`1 + d`),
`Reproduce(d)` (`9 + d`), `StealEnergy(d)` (`17 + d`) for `d` in the eight
`Direction::ALL` slots, `Drive(Move)` (25), `Drive(Reproduce)` (26),
`Drive(StealEnergy)` (27), `Terminate` (28), and `Decide` (29). A drive
(T11.F30) votes for its kind without naming a direction: the pass end adds it
to the kind's best direction vote, and it is never committed itself; it
serializes as `{"Drive":"Move"}`. `VoteKind` is
`Eat`, `Move`, `Reproduce`, `StealEnergy`, in that order.
`ActionParamField` holds exactly the fields the commit decoder reads, in the
order `EatFoodType`, `EatLeaveShare`, `ReproduceTransferFraction`,
`StealEnergyAmount` (T11.F27, T02.F08); `Move` has no parameter field.

A wired `ActionVote` sink writes its weighted sum into the visit's vote
contribution, which replaces the node's earlier contribution in the pass; a
wired `ActionParam(field)` sink overwrites `action_params[field.index()]`
on the tick's four-field parameter surface. The pass end reads both
(`v3-mesh-execution-spec.md` Section 2); votes are how the graph acts, so a
graph with no wired vote sink never commits an action. `pick_random_surface`
makes one uniform draw over the compute nodes and then every sink in vector
order (T11.F27).

Serialized genomes carry the field by name (`{"ActionParam":"EatFoodType"}`).
A pre-T11.F27 genome with an old `{"ActionParam":["Eat",0]}` sink fails to
deserialize; no conversion path exists.

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

## 6. Votes and Parameters

A graph selects actions through its `ActionVote` and `ActionParam` sinks
(T19.F04).

- A kind's action count is its vote: at the pass end the kind with the
  highest effective vote (best sink's vote minus the kind's bar, its commits
  so far this tick) commits once, in its best sink's direction; a vote of
  `v` therefore commits `ceil(v)` actions over successive passes.
- Parameters are read at commit from the kind's fields: `Eat` reads
  `EatFoodType` and `EatLeaveShare` (the share of the cell's food it leaves,
  clamped to `[0, 1]`, T02.F08), `Reproduce` `ReproduceTransferFraction`,
  `StealEnergy` `StealEnergyAmount`, and `Move` nothing
  (`runtime::action_decode::decode_commit`, shared with the VM's
  `WriteActionParam`).
- `Terminate` ends a non-empty tick when its vote is at least the best
  effective vote; `Decide` ends a pass early when some kind's effective vote
  is positive, or when the queue is non-empty and `Terminate` is positive.
  Neither commits.

### Qualified zero vote connections (T20.F02)

F04 may append an ordinary `GraphEdge { source, weight: 0.0 }` to a
**present** `ActionVote` sink. All 30 identities qualify: Eat, the eight
Move, Reproduce and StealEnergy directions, the three drives, Terminate and
Decide. Both empty
and wired vote sinks qualify; their position in `output_sinks` is irrelevant.
Preserve the incumbent edge order and coefficients, compute nodes, input
references and sink identities.

The source is a declared `InputLeaf { ref_idx, sub_idx }` or a current/previous
`SharedMemory { slot, previous }` slot. For **finite resolved source values**
and equal native execution context, the appended zero preserves the complete
visit vote contribution, incumbent output slots, route scores, parameters,
shared memory, and carried temporal and plasticity state. Equality is exact
numeric equality, allowing signed zero; no tolerance conceals changed votes.
The appended zero edge is static: it carries no learning attributes.

An already active graph with unchanged compute nodes pays the same native
charge. At adequate budgets its committed actions and carried state agree
through repeated visits and ticks, including nonzero incumbent state and
plasticity. A previously dormant graph (no compute nodes and no wired sink)
starts paying one minimum visit when its first zero vote edge is appended.
Its vote remains zero, but this debit can change a later live energy read or
exhaust the creature. Whole-mesh physiology/action neutrality does not cover
that activation. Unaffordable visits retain the native exhaustion behavior in
Section 12 and commit no graph effects.

| Boundary | Consumer obligation or observed limitation |
| --- | --- |
| Destinations outside this qualification | `ClearSlot`, `WriteSlot`, `CustomOutput`, `ActionParam`, `RouterGate`, and every compute input. An empty clear/store/output/parameter sink can clear or overwrite nonzero incumbent data when wired with zero. A zero input changes `Multiply` and `Min`. Exclusion does not imply every excluded append is unsafe in every context. |
| Nonfinite sources | NaN or infinity multiplied by zero can poison a previously nonzero weighted sum before sanitization. There is no neutral guarantee and no special zero-edge sanitization. |
| Refinement | Each occurrence owns its coefficient, even duplicate source/sink pairs. Changing one zero weight to nonzero can change its named vote under an excited source without changing other coefficients. Existing `AlterGraphEdgeWeight` reaches zero edges. |
| Removal and references | `RemoveGraphEdge` deletes one occurrence and does not recreate it. A declaration with even a zero-weight consumer cannot be pruned. Once its last consumer is removed it may be pruned. Pruning an earlier unused declaration reindexes surviving consumers while preserving source meaning/channel. |
| Copy and persistence | Native node copying, genome cloning and serde preserve source meaning, channel, sink identity and independent weights. Mutation/deletion in a copy leaves its original intact. Occurrences have no persistent edge ID. |
| Addressing | `ref_idx` and `sub_idx` are `u16`; declared index 65535 is representable, proposed index 65536 is not. Shared-memory slots are 0–15. Preserve world-compound wrapping, scalar index ignoring, and decision-compound out-of-width zero behavior. `ActionQueue` reads use their raw index, including readable channels beyond mutation draw width 12. Missing/out-of-width data yielding zero is not evidence of useful refinement. |
| Admission | An absent destination or unrepresentable proposed reference is ineligible. F02 introduces no creation API or new capacity limit. F04 owns atomic admission/failure and any source declaration; truncation and incumbent overwrite cannot satisfy this contract. |

| Added structure | Added `genome_size()` units |
| --- | ---: |
| One edge on an already wired sink | 1 |
| First edge on an empty sink | 1 (a sink costs nothing of itself, T25.F02) |
| Separate input declaration | 1 more |

These units feed existing maintenance, replication and size-dependent mutation
supply accounting. Resolving and summing each new source adds host work even
though effect edges have no separate physiological per-edge charge. An extended
input declaration may also trigger perception assembly. This contract promises
neither free storage/work/reproduction nor unchanged fitness.

The runtime and property fixtures, commands and cost observations are indexed
in [T20.F02 readings](../progress/readings/t20-f02.md).

### Ring edge groups (T25.F03)

A `Ring` group `{shape: Ring {ring_ref}, kind K, offset k, scale s}` is one
record plus up to eight member sink edges: the member for ring cell `c` reads
`InputLeaf { ring_ref, c }` and sits on sink `K((c + k) mod 8)`, `c < 8`, the
ring being the neighbor ring at `input_refs[ring_ref]`, whose channel `c` is
`Direction::ALL[c]`. A member's genomic weight is
`compose(own_weight, s, pattern) = s · pattern + own_weight`, the only
formula; a `Ring` member's pattern is 1 when `ring_ref` resolves to a ring and
0 otherwise, resolved from the group's `ring_ref`, never the member's own
source, so an ablation redirect keeps the weight. A non-member's genomic
weight is its `own_weight`.

- Invariants (checked by `validate_graph` at `GenomeFile::load` and in the
  mutation gate): only sink edges are members; every `GroupId` names a record
  and `groups.len() ≤ u16::MAX`; a member fits its shape; at most one member
  per `(group, source, target)`; a group has at least one member;
  `input_refs[ring_ref]` is a ring and `offset < 8`; within a sink,
  independent edges come first in insertion order, then members sorted by
  `(GroupId, sub_idx)` (I10; a `Ring` member's channel).
- Members are summed with the sink's other edges at their genomic weights,
  sequentially in the I10 order: the runtime reads one weight per sink edge
  from the table `GraphRuntimeState::for_genome` builds at construction. A
  `Ring` group at scale `s` with corrections `c_i` is bitwise its eight
  ungrouped edges of weight `s + c_i`.
- A new group (scale 0, corrections 0) is silent. No per-group energy
  charge: members wire their sinks, so a group on a dormant graph makes the
  visit entered (Section 8) and pays one minimum visit, as the first zero
  vote edge does.
- A group record is one `genome_size()` unit and each member one edge unit,
  so a full `Ring` group is nine units (T25.F02's projection charge). Its
  ring is consumed through its members; `Prune` never deletes it.
  Structured refinement never steps a member.
- The VM has no group (T11.F18 parity exemption): a VM node reaches one by
  `SwapNodeBackend` to Graph.

### Direction-pair edge groups (T25.F04)

A `DirectionPair` group `{shape: DirectionPair {ref_idx, pair}, kind K,
offset k, scale s}` reads pair `pair` of `input_refs[ref_idx]` (a `World` key's
`direction_pairs()`, sensor spec Section 5.7): a member reads
`InputLeaf { ref_idx, x }` or `InputLeaf { ref_idx, y }` and sits on a
direction sink `K(d)`; its coordinate is `(field, d)`. Its pattern is
`compass_x((d − k) mod 8)` for the `x` field and `compass_y((d − k) mod 8)`
for the `y` field, `compass_x(d)` being `Direction::ALL[d].delta().0`
normalized (exactly 0, ±1 or ±1/√2) and `compass_y(d) = compass_x((d + 6)
mod 8)`; any other field, a direction past 7 or an unresolvable pair reads 0.
At `s > 0` and `k = 0` the group votes toward the sensed offset, at `k = 4`
away from it. The weight formula is the `Ring` group's `compose`; the pattern
resolves from the group's `ref_idx` and the member's sink, so
`edge_weight(edge, sink, input_refs)` and `genomic_weight(occurrence, …)`
take the sink.

- Creation materializes the twelve non-zero-pattern members (`x` at relative
  NE, E, SE, SW, W, NW; `y` at N, NE, SE, S, SW, NW) at own weight 0 and scale
  0: thirteen `genome_size()` units, silent. A member on a zero-pattern
  direction is valid (a loaded genome may carry one) and never created.
- Invariants as for `Ring`, keyed by `(sub_idx, direction)` instead of the
  ring channel: `input_refs[ref_idx]` declares `pair` and every member reads
  its `x` or `y` field (`validate_graph`); at most two members per sink.
- Shift and retarget move a member on `K(d)` to `K'((d + k' − k) mod 8)`,
  keeping every member's relative coordinate (holes and zero-pattern members
  included), so an edit and its inverse return an equal genome. Detach,
  removal and the last member's removal behave as for `Ring`.

---

## 7. Pass Ends

A pass ends at the chain's natural end (`NoTargets`, `MissingNode`), at the
per-pass hop cap, or when a `Decide` vote holds (`v3-mesh-execution-spec.md`
Section 3). No graph output terminates the tick.

---

## 8. Evaluation Order and Per-Visit Clock

Phase 0 calls `graph_runtime.begin_tick(age)`, also used by neighborhood
sequences, standalone multi-tick callers, and cloned observations. It records
the dispatch record's age and decays every eligibility trace once; it
snapshots nothing (T19.F02). A learning graph's visited flag is set at visit
entry, before the pass-cost check, and stays set for the creature's life
(Section 13). Internal time is counted in
visits and world time in ticks: a module's state advances once per visit,
however many visits a tick's pass makes, and the per-pass hop cap and the
per-tick hop ramp bound and price those visits. A visit is entered when the
graph has at least one compute node or a wired effect surface (an output
sink carrying at least one edge, a group member included). An entered visit evaluates all compute nodes once in index order,
including disconnected nodes; with zero compute nodes that evaluation is
empty and the effects pass still runs.

- Lower-index compute sources read the current visit's computed outputs.
- Self and higher-index sources read the last committed outputs: the last
  successful visit's, this tick or earlier.
- Stateful operators start from the last committed operator state.
- A successful visit commits candidate state and outputs, which the next
  visit reads whether it comes later this tick or in a later tick. A failed
  visit leaves the last commit standing. Effects and learning remain per
  backend call.
- Unvisited modules hold values without catch-up, fabricated inputs, or charge.
- Newborn temporal state is zero. Graphs with no compute node and no wired
  effect surface do no work: no charge, no counter, upstream slots pass through.

There is no convergence loop and no relaxation configuration; the former
fields are deleted (`v3-runtime-config-spec.md`).

---

## 9. Post-Evaluation Effects

After the ordered evaluation's pass cost is affordable (learning is free,
T25.F02), one effects pass processes all fixed structural outputs. A zero-compute
entered visit runs the same pass with `compute_count = 0`, where a `ComputeNode`
source resolves to `0.0` as any out-of-range compute source does.

### Phase 1: Value outputs

Iterate `output_sinks`. For each sink with non-empty `inputs`:
- Gather `wsum = sum(resolve_source(edge.source) * applied_weight(edge))`,
  sequentially in the sink's edge order, using current-visit `curr_outputs`
  for `ComputeNode` sources and shared memory as earlier sinks wrote it. The
  applied weight is a plastic edge's learned weight, else the genomic weight
  from the runtime weight table (Section 6). On a learning graph each plastic
  edge records the value the sum read as its pre sample and the sink's
  `sanitize_f32(wsum)` as its post sample (Section 13).
- Write to target:
  - `CustomOutput(slot)`: `output_slots[slot] = wsum` (slot < 12).
  - `RouterOutput`: emit `RouteDecision::CgpNormalized { raw_value: wsum }`.
    Mesh routing resolves that decision with
    `idx = min(floor(clamp01(raw_value) * target_count), target_count - 1)`.
  - `WriteSlot(slot)`: `shared_memory[slot % 16] = sanitize_f32(wsum)`.
  - `ClearSlot(slot)`: `shared_memory[slot % 16] = 0.0` (wsum is ignored;
    the act of having edges and firing is what clears).
  - `ActionVote(sink)`: `contribution[sink] = sanitize_f32(wsum)`.
  - `ActionParam(field)`: `action_params[field.index()] = sanitize_f32(wsum)`.

Sinks with empty `inputs` are inert — no write occurs. A visit that reaches
its effects stages its vote contribution; an exhausted visit stages nothing.

### Effect-phase trace visibility (Execution Sampler)

Graph sampler traces include post-evaluation structural-layer records in
addition to the single entered evaluation:
- `output_sinks: Vec<GraphOutputSinkTrace>` (1:1 with `output_sinks`)
  - fields: `wired`, `weighted_inputs` (each edge's value times its applied
    weight, in sink order, as the node trace carries them; empty when
    unwired), `weighted_sum`, `applied`, `applied_value`
  - built only when the tracer records; the production pass allocates
    nothing per visit

The pass records (votes, effective votes, committed action, end reason) are
on the tick trace (`v3-server-api-protocol-spec.md`).

These records are always index-aligned with structural catalogs (not sparse
"only fired" lists), so UI/debug tooling can compare wired-but-inert vs active
behavior directly.

---

## 10. Routing Gates

Each `RouterGate(slot)` sink writes its weighted sum to one of eight gate
slots. The mesh chooses the earliest maximum `gate_bias + gate_score` among
all destination IDs, with the shared soft-float policy; every existing target
is eligible, the node itself and nodes already dispatched this tick included
(T19.F02). A missing winner terminates softly. T11.F15 route addition
pairs a tied branch through a pass-through detour with one weight-1 gate edge
sampled by `random_graph_source`, including full input sub-value support.

---

## 11. Stateful Operators and `graph_state`

`GraphRuntimeState` stores committed `node_state` and `node_outputs`, indexed
by `[mesh_node_idx][compute_node_idx]`; they are the live read base of every
visit and there is no tick-start copy (T19.F02). Missing slots read zero and
storage is initialized lazily. Offspring receives fresh empty temporal
vectors, regardless of learned-weight inheritance.

Trace `passes` contains one entered evaluation, including an unaffordable
attempt. `node_evaluations` describe candidate computation; `temporal_committed`
marks whether that candidate was applied. `final_outputs` always reports the
last successful committed outputs. `max_delta` compares the candidate
outputs with the last committed outputs, not a convergence test.

---

## 12. Energy Cost

Graph node cost is charged per compute-node-per-visit evaluation
(`graph_node_base_cost` or equivalent config-driven scalar). An entered visit
pays for at least one node equivalent, so a wired zero-compute visit costs what
the same graph costs with a single dummy compute node.
Canonical owner for graph runtime cost config:
`v3-runtime-config-spec.md`.

Equivalent requested energy:

```text
graph_energy_requested =
  graph_node_base_cost * max(compute_nodes.len(), 1) * entered_visits
```

If energy is exhausted during graph evaluation, node evaluation halts and mesh
execution returns `WorldAction::NoOp`. Evaluation-cost exhaustion preserves
the prior successful temporal state and outputs and emits no graph effects;
learning charges no energy (T25.F02), so it cannot exhaust a creature. Actual charge and entered work remain recorded; learned
pure Hebbian weight and reward rollback are outside this temporal transaction.
Reward-trace activity commits at this same successful boundary.

Note: fixed structural outputs (the sinks) are not
counted in the per-visit energy cost. They are evaluated once post-evaluation.

---

## 13. Plasticity

### PlasticityConfig

Per-edge learning attributes (T25.F06), on `GraphEdge.plasticity`:

```rust
pub struct PlasticityConfig {
    pub rule: HebbianRule,        // Classic, Oja, AntiHebb, Covariance
    pub learning_rate: f32,       // [0, 1]
    pub weight_clamp: f32,        // [0.01, 10]
    pub lamarckian: bool,
    pub modulation: Option<RewardModulationConfig>,
}

pub struct RewardModulationConfig {
    pub reward_source: OutcomeChannel,  // EnergyDelta, ActionSuccess, DamageDelta, OffspringSuccess
    pub trace_decay: f32,               // [0.0, 1.0]
}
```

Every edge on either surface may be plastic except an edge of a `ClearSlot`
sink (its sum is ignored); `validate_graph` rejects that case and any value
outside the ranges above (I1), which the mutation draws and perturbations
keep by construction. The laws read the stored values; no runtime clamp
normalizes them. A learning attribute, a member's `scope` and a group's
`inherit_scale` carry no genome unit. `CgpGraphBackendDef::plastic_edges()`
enumerates the plastic edges in canonical (`edge_sites`) order and is the
one definition of "carries learning".

A plastic group member (T25.F07) learns at its `scope`: an **edge learner**
(`Edge`) learns its own correction as an ungrouped edge learns its weight; a
**group learner** (`Group`) learns for its group, crediting the group's one
shared scale. A group with at least one group learner has a **scale slot**
whose **bound** is the smallest `weight_clamp` among its group learners; a
static member of such a group is a **passive member**, weighed through the
learned scale. Scope never splits a group: membership compares the
`GroupId` only.

When `modulation` is `None`: pure Hebbian learning, applied after the
effects of each committed visit. When `Some`: reward-modulated three-factor
learning, the eligibility trace updated per committed visit and the weight
in Phase 2.5.

### Learned state

`GraphRuntimeState::for_genome(genome, seeds)` is the only constructor and
builds the learned state once (`runtime::plasticity::LearnedState`): per
mesh node, the learner slots over both surfaces in canonical order, the
scale slots in `GroupId` order (genomic scale, bound, learned scale, seeded
flag), the passive members, a learned own quantity per ungrouped or edge
learner (a group learner stores none), a trace per reward-modulated learner
and a visited flag. A graph without a plastic edge allocates no learned
state and runs the static loops. One composition gives every dynamic edge's
**applied weight**:

```text
applied = own + scale × pattern
own   = the learned own quantity when the learner has one, else the genomic own weight
scale = the learned scale when the group has a scale slot, else the genomic scale
pattern = 0 for an ungrouped edge
```

The executor's weighing, `applied_weight(occurrence)` (one edge, else the
genomic weight), `applied_weights(node)` (every dynamic edge in canonical
order) and the view agree; the applied weight is never clamped. The
executor hands the learner each edge's occurrence and value and the learner
decides whether the edge is a learner, a passive member or static. Readers
use the read-only views: per plastic edge `(occurrence, genomic, learned,
applied, trace, bound, kind, scope)`, where `genomic` is the genomic own
weight (a member's correction), `learned` the initialized own quantity
(always `None` for a group learner) and `trace` present only for an
initialized learner; and per scale slot `(group, genomic, learned, bound,
inherit_scale)`, `learned` present only for an initialized scale slot.

- A learner slot is initialized when its backend has been visited (the flag
  is set at visit entry, a failed visit included) or when it was seeded at
  birth; a scale slot when its backend has been visited or its scale was
  seeded. Reward, `capture()` and the views read initialized slots only, each
  slot on its own.
- A Lamarckian ungrouped or edge learner with a birth seed starts at the
  seed clamped by the child's edge clamp, initialized; every other learner
  (a Darwinian one with a seed, and a group learner, which ignores any seed
  on its row) starts at its genomic own weight and stays uninitialized until
  its backend's first visit. A scale slot starts at `clamp(seed, ±bound)`,
  seeded, when the child group has `inherit_scale` and a seed, else at the
  genomic scale. Traces start at zero. See
  [reproduction inheritance](v3-reproduction-spec.md#graph-state).
- Birth seeds are temporary, excluded from serialization and genome
  equality, and consumed by the constructor.

### Samples, order and the Hebbian laws (per committed visit)

On a learning graph the executor records, for every plastic edge, the pre
value its evaluation used — a compute edge's ordered read (lower sources this
visit's outputs, self and higher the last committed) and a sink edge's value
as the sink sum read it (Section 9) — and the post value: the node's
committed output, or the sink's `sanitize_f32(wsum)`. No source is
re-resolved after evaluation. Evaluation reads the applied weight in force
at visit start. After the effects, each pure-Hebbian learner's change is
computed from its samples and visit-start applied weight `w` with its own
rule and rate, in canonical order: an ungrouped or edge learner applies it to
its own quantity, bounded by its own clamp; a group learner adds `pattern ×
dw` to its scale's accumulator. Then every scale with a contribution (a zero
one included) is written once, scale slots in order:

```text
Classic    dw = eta * pre * post
Oja        dw = eta * post * (pre - w * post)
AntiHebb   dw = -eta * pre * post
Covariance dw = eta * (pre - 0.5) * (post - 0.5)
own   <- clamp(own + dw, -weight_clamp, weight_clamp)        (ungrouped or edge learner)
scale <- clamp(scale + Σ pattern × dw, -bound, bound)        (once per scale per event)
```

A static member contributes nothing; a member with a negative pattern
contributes with the opposite sign of its `dw`, so the net change is the sum
over contributors. A write whose stored value lay outside the bound (only an
unseeded genomic scale's first write can) counts one `plasticity_scale_snaps`.
`plasticity_updates` counts one per contributing learner per event and
`plasticity_changes` one per contributor whose stored quantity changed (for a
group learner its scale, so a moved scale counts every contributor).

On a recurrent compute edge the pre value is the committed value the
evaluation read (T25.F06; the retired replay read this visit's output). A
failed visit changes no learned value or trace.

### Eligibility trace clock and activity

```text
trace <- trace_decay * trace                        (once per world tick, begin_tick)
trace <- trace + activity(pre, post, w)             (per committed visit)
```

Activity excludes the learning rate: Classic `pre*post`, Oja
`post*(pre-w*post)`, AntiHebb `-pre*post`, Covariance `(pre-0.5)*(post-0.5)`,
from the same samples. A module dispatched twice in one tick adds twice,
bounded by the per-pass hop cap; a skipped tick leaves only the decay. Lambda
0 forgets prior credit; lambda 1 retains it; an isolated pulse is discounted
by lambda^d after d skipped ticks. Input context includes evaluation-time
energy. Learning charges no energy or work (T25.F02).

### Reward-modulated weight update (Phase 2.5)

```text
dw = learning_rate * outcome_signal[reward_source] * trace
new_own = clamp(own + dw, -weight_clamp, weight_clamp)            (ungrouped or edge learner)
scale <- clamp(scale + Σ pattern × dw, -bound, bound)             (group learners, once per tick)
```

A modulated learner changes no weight at a visit; a modulated group learner
contributes only while its scale slot is initialized, and the scale is
written once per backend's Phase 2.5 pass, as at a visit.

Eta applies once: unit activity/reward at eta 0.5 yields trace 1 and delta
0.5. At lambda 0 and unit reward, a feed-forward edge matches its pure
Hebbian update. Phase 2.5 updates every initialized modulated edge each tick,
even on skipped visits or zero deltas, and counts each update; a seeded
modulated slot counts from birth. Reward does not clear eligibility.

---

## 14. Three-Tier Liveness Model

Clear terminology to avoid conflating structural presence with functional
contribution:

- **Structurally present**: exists in the genome by construction. All fixed
  sinks are always structurally present.
- **Wired**: has at least one edge. Only wired items write values or
  votes. Only wired items count toward `genome_size()` and
  `functional_complexity()`; `genome_size()` charges a wired sink its edges
  only, one unit each, and nothing for the sink itself (T25.F02).
- **Functionally reachable** ("live"): compute nodes backward-reachable from
  any wired sink, vote and parameter sinks included. An edge-group record is
  one `genome_size()` and one `functional_complexity()` unit; its members are
  sink edges that wire their sinks, and its ring is consumed through them
  (T25.F03).
  "Live" is reserved for behaviorally contributing items only — unwired sinks
  are structurally present but NOT live.

Dormant (unwired) sinks are NOT counted toward
`genome_size()` or `functional_complexity()`. This prevents the fixed catalog
from imposing a constant complexity tax.

---

## 15. Fixed Output Catalog Construction

`GraphBackendDef::new_with_fixed_outputs()` constructs, in this order:
- 24 `CustomOutput(0..23)` sinks (catalog indices 0..24)
- 8 `RouterGate(0..7)` sinks (24..32)
- 16 `WriteSlot(0..15)` sinks (32..48)
- 16 `ClearSlot(0..15)` sinks (48..64)
- 30 `ActionVote` sinks in `VoteSink` index order (64..94)
- 4 `ActionParam` sinks in `ActionParamField::ALL` order (94..98)

All sinks start with empty edge Vecs (inert until evolution wires them).

### Queue-size contract

The genome reads the first four queued actions:
`InputReference::ActionQueue` is a constant four slots of three sub-values,
12 in all (`mutation::compound::ACTION_QUEUE_INPUT_SLOTS`), on every config
(T19.F06). A slot past the queue's end reads `0.0`.

The number of actions a tick commits is bounded by
`runtime.max_actions_per_turn`, not by the genome.

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
| Unwired `ActionParam` sink | The parameter keeps its last write this tick (zero at tick start) |
| Graph with no wired vote sink | Votes nothing; the pass commits only what other nodes vote |
| Edge with NaN/Inf weight | `sanitize_f32()` to 0.0 |
| Sink with empty `inputs` | Inert — does not write |
| Group whose `ring_ref` is out of range or not a ring | Pattern 0: each member weighs its own correction; `validate_graph` rejects the genome (I7), so a live genome never carries one |
| Group `offset >= 8` or a member off its shape | Rejected by `validate_graph`; the mutation gate rolls the event back under `GraphInvariantViolation` |
| Re-entry in one tick (a revisit) | Starts from the previous visit's commit and commits again; bounded by the per-pass hop cap |
| Missing graph state | Lazily initialized to zeros |

Cross-runtime fallback outcomes are canonical in
`v3-mesh-execution-spec.md` (Section 4, authoritative soft-default matrix).
