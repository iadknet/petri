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
`CopyComputeNode`'s faithful-copy rule, `InputRef.AddSensor*`'s unwired
push on a VM node (on a Graph node it wires every field set of the new
sensor as bundles since T25.F11), and the
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
    /// Bundle records by `BundleId` (T25.F03, T25.F11); omitted from JSON when empty.
    bundles: Vec<Bundle>,
}

pub struct Bundle {
    source: BundleSource,            // Sensor { ref_idx: u16, field_set: u8 } | Memory { previous: bool }
    wiring: Wiring,                  // Plain { sink } | Ring { kind, offset } | DirectionPair { kind, offset }; offset < 8
    scale: f32,                      // one shared weight; a live birth draws it, a silent birth 0
    scale_learning: Option<Learning>, // how the scale learns (T25.F12), born absent; omitted when None
    member_default: Option<Learning>, // what `Inherit` members run (T25.F12), born absent; omitted when None
}

pub struct Membership {
    bundle: BundleId,
    policy: MemberPolicy,            // Inherit | Static | Own(Learning) (T25.F12), born Static
    credits: bool,                   // the member's activity feeds the scale (T25.F12), born false
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
    role: EdgeRole,                  // Lone(Option<Learning>) | Member(Membership) (T25.F12)
}                                    // 40 bytes; JSON `learning` (lone) or `membership`, never both
```

Design choice: edges are co-located with each node/sink for linear,
cache-friendly evaluation. Fixed structural outputs are stored in separate
Vecs to prevent topology mutations from affecting them. Fields are private;
the container (`cgp/container.rs`) is the only writer and keeps the birth
seeds and the bundle invariants aligned; `EdgeMut::set_learning`,
`set_policy`, `set_credits` and `BundleMut::set_slot` are the only writers
of learning (Section 13). A bundle stores no member list: its
members are ordinary sink edges whose membership names it (Section 6).
Equality, the `Debug` genome identity and JSON cover every heritable field
and nothing else (`Debug` prints `bundles` unconditionally);
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
    ActionVote(VoteSink),    // 39 vote sinks
    ActionParam(ActionParamField), // 5 decoded parameter fields
}
```

The full sink catalog is fixed at genome construction: 24 CustomOutput + 8
RouterGate + 16 WriteSlot + 16 ClearSlot + 39 ActionVote + 5 ActionParam = 108
sinks. Mutations can only modify edges TO sinks, not add/remove/change sink
kinds.

`VoteSink` is the fixed vote catalog: `Eat`, `Directed(K, d)`, `Drive(K)`,
`Terminate` and `Decide`, for `K` a `DirectedVoteKind` (`Move`, `Reproduce`,
`StealEnergy`, `GiveEnergy`, in `DirectedVoteKind::ALL` order) and `d` in the eight
`Direction::ALL` slots. With `N` directed kinds the index rule is `Eat` 0,
`Directed(K, d)` `1 + 8·K.index() + d`, `Drive(K)` `1 + 8·N + K.index()`,
`Terminate` `VOTE_SINK_COUNT − 2` and `Decide` `VOTE_SINK_COUNT − 1`, with
`VOTE_SINK_COUNT = 3 + 9·N`: `Directed(Move, d)` (`1 + d`),
`Directed(Reproduce, d)` (`9 + d`), `Directed(StealEnergy, d)` (`17 + d`),
`Directed(GiveEnergy, d)` (`25 + d`), `Drive(Move)` (33), `Drive(Reproduce)`
(34), `Drive(StealEnergy)` (35), `Drive(GiveEnergy)` (36), `Terminate` (37),
and `Decide` (38): `N` = 4 and 39 vote sinks. A new directed kind is appended to
`DirectedVoteKind::ALL` and the rule places its sinks. A directed sink
serializes as `{"Directed":["Move",3]}`. A drive
(T11.F30) votes for its kind without naming a direction: the pass end adds it
to the kind's best direction vote, and it is never committed itself; it
serializes as `{"Drive":"Move"}`. `VoteKind` is
`Eat` followed by the directed kinds' vote kinds in `DirectedVoteKind::ALL`
order: `Eat`, `Move`, `Reproduce`, `StealEnergy`, `GiveEnergy`.
`ActionParamField` holds exactly the fields the commit decoder reads, in the
order `EatFoodType`, `EatLeaveShare`, `ReproduceTransferFraction`,
`StealEffort`, `GiveEffort` (T11.F27, T02.F08, T27.F03); `Move` has no
parameter field.

A wired `ActionVote` sink writes its weighted sum into the visit's vote
contribution, which replaces the node's earlier contribution in the pass; a
wired `ActionParam(field)` sink overwrites `action_params[field.index()]`
on the tick's five-field parameter surface. The pass end reads both
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
  `StealEnergy` `StealEffort` as its effort modifier `2^clamp(x, −2, 2)`
  (1 when unwired, T27.F03), and `Move` nothing
  (`runtime::action_decode::decode_commit`, shared with the VM's
  `WriteActionParam`).
- `Terminate` ends a non-empty tick when its vote is at least the best
  effective vote; `Decide` ends a pass early when some kind's effective vote
  is positive, or when the queue is non-empty and `Terminate` is positive.
  Neither commits.

### Qualified zero vote connections (T20.F02)

F04 may append an ordinary `GraphEdge { source, weight: 0.0 }` to a
**present** `ActionVote` sink. All 39 identities qualify: Eat, the eight
Move, Reproduce, StealEnergy and GiveEnergy directions, the four drives,
Terminate and Decide. Both empty
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

### Bundles (T25.F03, T25.F04, T25.F11)

A bundle is one gene that wires one field set of one source. Its `source`
names a declared sensor's field set (`Sensor { ref_idx, field_set }`, the set
from the sensor's descriptor in `contracts::inputs`) or a shared-memory bank
(`Memory { previous }`, 16 fields, no declaration). A member is a sink edge
whose `Membership` names the bundle; it reads a field of the set
(`InputLeaf { ref_idx, field }` or `SharedMemory { slot, previous }`), and
its **coordinate** is `(field, site)`, the site read from its sink: `Sink`
(the one vote sink of a `Plain` wiring), `Direction(d)` (the direction sink
`K(d)` of a geometric wiring's kind `K`) or `Drive` (`Drive K`). The wiring
owns the legal coordinates and the pattern:

| Wiring | Field set | Coordinates | Pattern |
| --- | --- | --- | --- |
| `Plain { sink }` | without geometry | every field on `Sink` | 1 |
| `Ring { kind K, offset k }` | a neighbor ring | cell `c` on `Direction((c + k) mod 8)`, `c < 8`, cell `c` being `Direction::ALL[c]` | 1 |
| `DirectionPair { kind K, offset k }` | a pair `{x, y, facts}` | `x` and `y` on the twelve directions whose compass value is non-zero; each fact on `Drive` | `compass_x((d − k) mod 8)` at `x`, `compass_y((d − k) mod 8)` at `y`, 1 at a fact |

`compass_x(d)` is `Direction::ALL[d].delta().0` normalized (±1 or ±1/√2 on
the twelve coordinates) and `compass_y(d) = compass_x((d + 6) mod 8)`; at
`s > 0` and `k = 0` a pair bundle votes toward the sensed offset, at `k = 4`
away from it.

**The one member law.** Every member of every wiring, a fact member
included, weighs `compose(own_weight, s, pattern) = s · pattern +
own_weight` under the bundle's shared scale `s`, the pattern read from the
wiring at the member's coordinate; the runtime's learned term, the sink
weight rows, the server's composed weight and a detached edge's weight read
the same pattern. Learning, decay, seeds, detach and removal are identical
for every member. A non-member's genomic weight is its `own_weight`.

- **Creation.** `NodeGenome::add_sensor` declares a sensor and builds one
  bundle per field set in one edit; `NodeGenome::add_bundle` builds one for
  a declared sensor's set or a memory bank. Both check everything before
  writing (the declaration index fits `u16`, the births cover the sensor's
  sets exactly once on a Graph node, every wiring is admissible for its
  set, the record count stays within `u16::MAX`) and write nothing on
  failure. `plans(def, set)` lists the admissible wirings: one
  `Plain { sink }` per present vote sink for a set without geometry, and
  `(kind, offset)` for every kind whose eight direction sinks (and, for a
  pair, its drive) are present. A new bundle materializes every coordinate
  of its set in canonical order (a pair's twelve compass coordinates, then
  its facts), own weight 0, `Static`, `credits` false, no seed; its scale follows
  the birth law (`v3-mutation-spec.md`).
- **Invariants.** `check_local_invariants` checks everything the record and
  the edge decide, after every bundle-touching primitive in debug builds:
  only sink edges are members; every `BundleId` names a record and
  `bundles.len() ≤ u16::MAX`; a member reads its source's kind on a site its
  wiring maps that field to as far as the record decides (a `Ring` cell on
  its offset's direction, no `Ring` or `Plain` member on a drive);
  coordinates are unique per bundle; a bundle has at least one member;
  `offset < 8`; retention in `[0, 1]`; within a sink, independent edges come
  first in insertion order, then members sorted by `(BundleId, field)` (I10).
  `validate_graph` (at `GenomeFile::load` and in the mutation gate) adds what
  needs the declarations: the source resolves to a field set the wiring
  fits (`ShapeReference`), and every member reads a field of that set on
  the site its wiring maps it to (`MemberShape`), resolved under
  `SensorContext::WIDEST` so validity never depends on the configured
  action queue.
- Members are summed with the sink's other edges at their genomic weights,
  sequentially in the I10 order: the runtime reads one weight per sink edge
  from the table `GraphRuntimeState::for_genome` builds at construction. A
  bundle at scale `s` with corrections `c_i` is bitwise its ungrouped edges
  of weight `s · pattern_i + c_i`.
- A silent bundle (scale 0, corrections 0) votes nothing. No per-bundle
  energy charge: members wire their sinks, so a bundle on a dormant graph
  makes the visit entered (Section 8) and pays one minimum visit, as the
  first zero vote edge does.
- **Edits.** Shift (`ShiftBundleOffset`, geometric wirings only) and
  retarget (`RetargetBundle`: a `Plain` bundle to another vote sink, a
  geometric one to another kind) move every member to its new site by
  coordinate, keeping holes, so an edit and its inverse return an equal
  genome. Detach turns a member into an ordinary edge carrying its composed
  weight and its seed. Removing a member leaves a hole; the record goes
  with its last member. A source-changing edit detaches first.
- A bundle record is one `genome_size()` unit and its members nothing
  (T25.F10), with or without holes; a detached member is an ordinary edge at
  one unit. `Prune` never deletes a sensor a member reads.
- The VM has no bundle (T11.F18 parity exemption): a VM node reaches one by
  `SwapNodeBackend` to Graph.

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
sink carrying at least one edge, a bundle member included). An entered visit evaluates all compute nodes once in index order,
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

## 13. Learning

### Learning

One learning type (T25.F06, T25.F12), `genome::learning::Learning`, serves
every learnable number:

```rust
pub struct Learning {
    pub rule: HebbianRule,        // Classic, Oja, AntiHebb, Covariance
    pub learning_rate: f32,       // [0, 1]
    pub weight_clamp: f32,        // [0.01, 10], the clamp of the learned value
    pub lamarckian: bool,         // a child inherits the learned value
    pub modulation: Option<RewardModulationConfig>,
    pub retention: f32,           // [0, 1], learned-value retention per tick (T25.F09)
}

pub struct RewardModulationConfig {
    pub reward_source: OutcomeChannel,  // EnergyDelta, ActionSuccess, DamageDelta, OffspringSuccess
    pub trace_decay: f32,               // [0.0, 1.0]
}

pub enum MemberPolicy { Inherit, Static, Own(Learning) }
```

It is stored on a lone edge (`learning`), on a member as its override
(`policy: Own(Learning)`), and in a bundle's two slots: `scale_learning`
("learns as a group") and `member_default` ("learns per edge"), either,
both or neither present. A **learner** is one learnable number: a lone
edge's weight, a member's **modifier** (its own weight) or a bundle's
scale. A member's **effective learning** is its override for `Own`,
`member_default` for `Inherit` (none when absent), none for `Static`; its
`credits` switch feeds the bundle's scale whatever its policy, so a member
may learn its modifier and credit at once. A **contributor** is a member
with `credits` of a bundle with `scale_learning`.

Every lone edge on either surface may learn except an edge of a `ClearSlot`
sink (its sum is ignored); members sit on vote sinks only. `validate_graph`
checks I1 on every stored learning (lone, `Own`, both slots) and rejects a
learning `ClearSlot` edge, and the mutation draws and perturbations keep the
ranges by construction. The laws read the stored values; no runtime clamp
normalizes them. No learning setting carries a genome unit. The writers are
`EdgeMut::set_learning` (a lone edge), `EdgeMut::set_policy` and
`set_credits` (a member) and `BundleMut::set_slot`; none changes a seed. A
new member is born `Static` with `credits` false.

### Learning layout

`LearningLayout::of(def, input_refs)` (genome-owned) is the one analysis of
what learns: in canonical order every **edge learner** (a lone edge with
learning, or a member with an effective learning) with its occurrence,
effective learning, source (`Lone`, `Own`, `MemberDefault`) and, for a
member, its bundle and pattern; in `BundleId` order every **scale learner**
(a bundle with `scale_learning`, whether or not it has contributors) with its
learning, its contributors (possibly none) and its members (every member,
each weighing through the learned scale). It is the only code that resolves
`Inherit` or reads a policy or `credits` to decide what learns, and its
emptiness is the one definition of "carries learning": runtime
construction, `StructuralCompanions.has_plasticity`, the mesh annotations'
stateful flag (an edge learner into a live compute node or a sink, or any
scale learner), the census carriers and the lab's masking
(`CreatureGenome::clear_learning`, which makes every lone edge static, every
member `Static` and both slots absent) read it.

When `modulation` is `None`: pure Hebbian learning, applied after the
effects of each committed visit. When `Some`: reward-modulated three-factor
learning, the eligibility trace updated per committed visit and the value
in Phase 2.5.

### Learned state

`GraphRuntimeState::for_genome(genome, seeds)` is the only constructor and
builds the learned state once per birth (`runtime::plasticity::LearnedState`)
from each Graph node's layout, after the child's mutation and before its
first visit: per learner its effective learning, value, birth value, trace
and seeded flag, the contributor and member indices, and a visited flag per
backend. `runtime::plasticity` names no member policy, `credits` or slot. A
genome whose layouts are all empty allocates no learned state and runs the
static loops. One composition gives every dynamic edge's **applied
weight** (an edge learner, or a member of a bundle with a scale learner):

```text
applied = own + scale × pattern
own   = the learned value of an edge learner, else the genomic own weight
scale = the learned scale when the bundle has a scale learner, else the genomic scale
pattern = the wiring's pattern at the member's coordinate; 0 for a lone edge
```

The executor's weighing, `applied_weight(occurrence)` (one edge, else the
genomic weight), `applied_weights(node)` (every dynamic edge in canonical
order) and the view agree; the applied weight is never clamped. The
executor hands the learner each edge's occurrence and value and the learner
decides whether the edge is dynamic. Readers use the read-only views: per
edge learner `(occurrence, genomic, birth, retention, learned, applied,
trace, bound, kind, learning, source, initialized, inherits)`, where
`genomic` is the genomic own weight (a member's modifier), `birth` its birth
value, `learned` present once initialized, `trace` present only for an
initialized modulated learner and `inherits` the effective `lamarckian`;
and per scale learner `(bundle, genomic, birth, retention, learned, bound,
lamarckian)`, `learned` present once initialized.

- A learner is initialized when its backend has been visited (the flag is
  set at visit entry, a failed visit included) or when it was seeded at
  birth. Reward, `capture()` and the views read initialized learners only,
  each on its own.
- Every learned value starts at its **birth value** (T25.F09): an edge
  learner at `clamp(seed if its learning is Lamarckian else genomic own,
  ±clamp)`, a scale learner the same with the bundle's seed, genomic scale
  and `scale_learning`. A seeded learner is initialized; every other one
  stays uninitialized until its backend's first visit. A scale learner
  without contributors keeps its value, decays, is captured and is seeded
  like any other; it only receives no credit. Traces start at zero. See
  [reproduction inheritance](v3-reproduction-spec.md#graph-state).
- Birth seeds are temporary, excluded from serialization and genome
  equality, and consumed by the constructor.

### Samples, order and the one update law (per committed visit)

On a learning graph the executor records, for every dynamic edge, the pre
value its evaluation used — a compute edge's ordered read (lower sources this
visit's outputs, self and higher the last committed) and a sink edge's value
as the sink sum read it (Section 9) — and the post value: the node's
committed output, or the sink's `sanitize_f32(wsum)`. No source is
re-resolved after evaluation. Evaluation reads the applied weight in force
at visit start. After the effects, from visit-start state, each learner
takes two steps (T25.F12 invariant 6): aggregation, then one scalar update.
An edge learner evaluates its learning's rule on its own `pre`, `post` and
visit-start applied weight `w`; a scale learner evaluates `scale_learning`'s
rule on each contributor's `pre`, `post` and visit-start applied weight and
sums the results times each contributor's pattern, unnormalized:

```text
Classic    dw = eta * pre * post
Oja        dw = eta * post * (pre - w * post)
AntiHebb   dw = -eta * pre * post
Covariance dw = eta * (pre - 0.5) * (post - 0.5)
own   <- clamp(own + dw, -weight_clamp, weight_clamp)                 (edge learner)
scale <- clamp(scale + Σ pattern × dw, -weight_clamp, weight_clamp)   (scale learner, once)
```

A non-crediting member contributes nothing; a member with a negative
pattern contributes with the opposite sign of its `dw`; the birth clamp
keeps every stored value inside its clamp, so no write snaps.
`plasticity_updates` counts learner updates (an edge learner per commit, a
scale learner per commit with a contributor, a reward step per initialized
modulated learner) and `plasticity_changes` those whose stored value moved;
a scale counts once, not per contributor.

On a recurrent compute edge the pre value is the committed value the
evaluation read (T25.F06; the retired replay read this visit's output). A
failed visit changes no learned value or trace.

### Eligibility trace clock and activity

```text
trace <- trace_decay * trace                        (once per world tick, begin_tick)
x     <- b + retention * (x - b)                    (once per world tick, begin_tick, after the traces)
trace <- trace + activity                           (per committed visit; Σ pattern × activity for a scale)
```

Each learner has one trace. The second line is the learned-value decay
(T25.F09): every initialized learner relaxes toward its birth value `b` by
its learning's `retention`; a retention of exactly 1 is skipped (an exact
no-op) and an uninitialized learner holds `b`. The stored value stays
absolute, so a decayed value stops within `ulp(x) / (2(1 − retention))` of
`b`. Learning in the same tick acts on the decayed value.

Activity excludes the learning rate: Classic `pre*post`, Oja
`post*(pre-w*post)`, AntiHebb `-pre*post`, Covariance `(pre-0.5)*(post-0.5)`,
from the same samples. A module dispatched twice in one tick adds twice,
bounded by the per-pass hop cap; a skipped tick leaves only the decay. Lambda
0 forgets prior credit; lambda 1 retains it; an isolated pulse is discounted
by lambda^d after d skipped ticks. Input context includes evaluation-time
energy. Learning charges no energy or work (T25.F02).

### Reward-modulated update (Phase 2.5)

```text
x <- clamp(x + learning_rate * outcome_signal[reward_source] * trace, -weight_clamp, weight_clamp)
```

The reward step is `three_factor` per initialized modulated learner, an
edge's or a scale's, at its own source, rate and clamp. A modulated learner
changes no value at a visit.

Eta applies once: unit activity/reward at eta 0.5 yields trace 1 and delta
0.5. At lambda 0 and unit reward, a feed-forward edge matches its pure
Hebbian update. Phase 2.5 updates every initialized modulated learner each
tick, even on skipped visits or zero deltas, and counts each update; a
seeded modulated learner counts from birth. Reward does not clear
eligibility.

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
  any wired sink, vote and parameter sinks included. A bundle record is
  one `genome_size()` and one `functional_complexity()` unit; its members are
  sink edges that wire their sinks at no `genome_size()` unit (T25.F10), and
  its source is consumed through them.
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
- 39 `ActionVote` sinks in `VoteSink` index order (64..103)
- 5 `ActionParam` sinks in `ActionParamField::ALL` order (103..108)

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
| Bundle whose source does not resolve to a field set its wiring fits | Rejected by `validate_graph` (`ShapeReference`), so a live genome never carries one |
| Bundle `offset >= 8` or a member off its wiring's coordinates | Rejected by `validate_graph` (`OffsetOutOfRange`, `MemberShape`); the mutation gate rolls the event back under `GraphInvariantViolation` |
| Re-entry in one tick (a revisit) | Starts from the previous visit's commit and commits again; bounded by the per-pass hop cap |
| Missing graph state | Lazily initialized to zeros |

Cross-runtime fallback outcomes are canonical in
`v3-mesh-execution-spec.md` (Section 4, authoritative soft-default matrix).
