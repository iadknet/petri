# Edge Groups and Per-Edge Learning Design Note (2026-10-13)

The user asked how creatures sense other creatures, found that the
multi-field creature sensors (`NearbyCreatureCore`, `Vitals`, `Identity`,
`AreaOccupancySummary`) never turned into behavior in the live world while the
neighbor rings did, and asked what it would take for those sensors to be
wired the way ring projections are wired: all of an input's fields added
together, as a group, with the group evolving as a unit or edge by edge. The
answer grew into a refactor of the graph genome's edge model. This note records
the design, the boundaries it draws, the options it rejected, the user's
decisions, and the feature contracts the spec owner reads.

This is version 2. Version 1 (commit `6b43ffca`) went through three Codex
Astra review rounds; version 2 rewrites §3, §4, §7, §8 and §9 after nine Opus
facet reviews (decomposition, boundaries, cruft, tests, data model, runtime,
mutation catalog, accounting, consumers) returned 35 blocking findings. The
findings, the decisions D1–D14 taken from them and every disposition are in
[`docs/progress/readings/t25-design-review.md`](../progress/readings/t25-design-review.md).
Where version 2 reverses a Codex disposition (edit records, D1) it says so.

**Decision (user, 2026-10-13): edges and edge groups become domain objects;
learning lives on edges and groups; mutation has a group layer and an edge
layer.** This is the first feature set of program 2 (track T25).

- Only geometry is built in: which sensor field corresponds to which
  direction. Nothing that computes a behavior is built in. The "bundle gene"
  (a gene that multiplies who-a-creature-is by where-it-is) was rejected for
  that reason.
- Every edge carries its learning mode (static, Hebbian, reward-modulated,
  with a Lamarckian flag) and, inside a group, whether its learning feeds
  only itself or the whole group.
- Reward source and trace decay are evolvable per edge or group. Credit
  shared across a group is one change to the shared scale. The plasticity
  update cost is 0, and the cost machinery goes (D9).
- Group creation comes in two forms: every member alike, or learning
  attributes randomized per member. The geometric pattern is never
  randomized.
- The three nearby-creature families merge into one `NearbyCreature`
  family. An edge costs one genome unit wherever it lives; the per-wired-sink
  unit is dropped and the founder anchor re-pinned.

## Answer

Today an edge is `{source, weight}` stored positionally in a compute node's
or sink's `inputs` list; learning is a setting on a compute node that covers
all of its input edges; the only stored group is the `RingProjection` gene,
and two other mechanisms infer groups from edge patterns on the fly. The
refactor:

1. **Edge** gains `membership` and `plasticity`. It stays in its container
   (input order is semantic for `GreaterThan`, `Select` and the trace
   payload), so no edge identifiers are needed; a member is a sink edge
   whose membership names a group record on the backend definition and whose
   source and target fit the group's shape. Its own weight becomes private:
   every reader chooses the own, genomic or applied weight.
2. **Group** is a record `{shape, kind, offset, scale, inherit_scale}`. A
   shape fixes the geometric pattern of its members; a member's genomic
   weight is `scale × pattern + own`, one pure composition every reader uses,
   and the runtime resolves the same composition over learned values. Ring
   projections become `Ring` groups with eight materialized member edges. A
   new `DirectionPair` shape fans a sensor's `(x, y)` pair into a kind's
   eight direction sinks in the fixed compass pattern.
3. **Learning** moves to the edge as today's `PlasticityConfig`, on both
   surfaces, so vote edges can learn; a member with scope `Group` applies its
   pattern-weighted update to the group's learned scale.
4. **Mutation** has an edge layer and a group layer. The nine plasticity
   operators move from compute nodes to edges one to one; attribute
   operators draw whether the change applies to the picked edge or to every
   member of its group. Ring projections' operators become group operators;
   `RecruitNeutralInput`'s ring case emits a group and gains a pair case.
   Structured refinement's inferred ring groups are deleted.
5. **Readers** read explicit membership from the genome; wiring analyses that
   look for direction-general behavior keep their own derived definitions
   over non-member edges and genomic weights.

Eight features in a linear chain: F01 container, constructors and birth
seeds (behavior-preserving); F02 uniform genome size; F03 ring groups replace
projections; F04 direction-pair groups; F05 the `NearbyCreature` family; F06
learning per edge; F07 group-scope learning; F08 creation forms, detach and
the catalog. §8 holds the contracts.

This recommendation is provisional against the repository's research depth
bar: the local evidence is complete (three code maps, a live probe, three
Codex rounds and nine facet reviews), the external precedent rests on one
paper verified at source and on summaries of the others (§5), and no scratch
proof has been run. The features' own gate and goal readings are the proof;
no lab run is planned (user ruling, 2026-10-10).

## 1. Problem, constraints, criteria

**Problem.** The live world (tick 51,351, 21,093 creatures, 1,478 sampled on
2026-10-12) shows:

- 54% of creatures wire `OccRing` to a vote and 23% carry an
  `OccRing → Move` projection at offset 0 with a negative weight. Those
  creatures are blocked on 16.3% of moves against 29.1% for the rest, the
  one creature-sensor use with behavioral evidence.
- 87% wire `NearbyCreatureCore`, but 1,025 of them wire only `present`,
  which reads 1 for 91% of creatures (someone within five cells) and so acts
  as a constant. 18% wire a `dx`, `dy` or `dist` field. No behavior was
  found.
- 121 creatures carry a whole-family `AreaOccupancySummary` bundle into one
  vote sink; none shows related behavior. A bundle into one sink cannot
  turn a position into a direction.
- Every logged `StealEnergy` requested 0.0 energy (14 of 1,478 creatures
  wire `ActionParam StealEnergyAmount`); predation is inert.

What made the rings work is that a ring's index is a direction, so one gene
maps sensor cell `d` to vote `d` with a drawn rotation. The `(dx, dy)` fields
have no such mapping and whole-family bundles do not supply one.

**Constraints.**

- Natural-analog rule: a mechanism reaches creatures through the body or
  the world; only geometry (which sensor field is which direction) may be
  built in. No feature-specific sensor, no computed behavior.
- Evolvability, not target abilities: the features improve what evolution can
  find; the creature-interaction examples are yardsticks only.
- No backward compatibility, ever: no existing world, creature, genome,
  artifact or wire shape constrains the design, and nothing is converted or
  migrated. A running world restarts from the founder on the new build;
  stored artifacts stay historical. Expected re-pins are predeclared.
- KISS: the smallest set of concepts that covers the user's decisions; no
  parallel mechanisms.
- Determinism: every draw order is documented; readers draw no RNG.
- Energy accounting: every new capacity has an applied cost, with one
  explicit exception the user granted (plasticity update cost 0).

**Criteria.** Fit with the user's decisions; separation of concerns (genome,
runtime, mutation, reproduction, analysis, transport); the number of
hand-maintained invariants removed; hot-path cost; blast radius.

## 2. What the code does today

From three read-only maps taken on 2026-10-13 (data model and runtime;
mutation engine; consumers). File references are to `crates/v3-core/src/`
unless prefixed.

**Data model.** `GraphEdge {source, weight}` (`creature/genome/cgp.rs:27-31`)
lives in `ComputeNode.inputs` and `OutputSink.inputs`. Edge order is
semantic: `GreaterThan` reads inputs 0 and 1, `Select` reads 0..2
(`runtime/cgp/eval.rs:17-70`). Edges have no identity; every parallel
structure is positional: `birth_weights[compute][edge]`,
`plasticity_weights[mesh][compute][edge]`, trace `weighted_inputs`, the
frontend's `cn:i:k` ids. A genome is immutable after the creature is built
(`creature/state.rs:182-186`): mutation runs on the child draft at birth,
and the only post-birth per-edge state is the runtime's.
`PlasticityConfig {rule, learning_rate, weight_clamp, lamarckian,
modulation}` sits on the compute node (`cgp.rs:104-109`;
`genome/mod.rs:180-192`) and applies to all of that node's input edges.
`RingProjection {ref_idx, kind, offset, weight}` is a separate list on the
backend definition (`cgp.rs:148-160`), evaluated as a per-sink term
(`runtime/cgp/effects.rs:57-99`), and carries a hidden `ablated_channels`
field only the input-use ablation uses. The VM backend has no edges and no
learning.

**Runtime.** A graph visit evaluates compute nodes in order with learned
weights where a node is plastic (`runtime/cgp/execute.rs:174-229`), runs
Hebbian updates before effects (`:239-292`), commits outputs, accumulates
eligibility traces (`:306-331`), then applies effects: projection terms
first, then every sink's edge sum with genome weights, and `WriteSlot` and
`ClearSlot` sinks write shared memory as the sink loop proceeds
(`effects.rs:104-245`, writes at `:192-206`), so a sink later in the catalog
can read memory an earlier sink changed. Sink edges never learn; storage has
no slot for them. Reward is applied in phase 2.5 of the tick from a
four-channel signal bank (`simulation/tick.rs:1271-1334`,
`simulation/outcomes.rs:86-111`); traces decay once per tick in phase 0.
Lamarckian inheritance copies the parent's learned weights into
`birth_weights` before mutation and rebuilds the child's runtime weights
after it from the child's post-mutation flags
(`simulation/actions/cgp_reproduction.rs:9-66`), with about twelve
hand-maintained alignment sites across mutation operators.

**Size.** `genome_size` charges one unit per mesh node, input reference,
route target, compute node, edge and projection, and one more unit per
wired sink (`genome/mod.rs:263-292`); the founder is 97 units
(`creature/founder.rs:16`). Functional complexity charges wired sinks and
live compute nodes separately (`cgp_analysis.rs:112-135`).

**Mutation.** 61 operators in four hand-mirrored enums (`mutation/types/mod.rs:178-244`;
`mutation/graph/mod.rs:18-136`). A Graph event draws one operator by weight
among those applicable, redrawing on `NoApplicableTarget`. Edge operators
address an edge as `(surface, index)` (`mutation/graph/operators.rs:219-259`);
`RetargetGraphEdge` (`:674-690`), the edge split of `AddInternalGraphNode`
(`:402-445`) and `GraphRawFieldMutation` (`:941-972`) change an edge's
source. Nine plasticity operators target a compute node
(`mutation/graph/hebbian.rs`). Five ring-projection operators act on a
projection as a unit, two of them by changing only its metadata (offset,
kind; `mutation/graph/projection.rs:79-109`). `RecruitNeutralInput`
(default `WholeFamily`) appends zero-weight edges for a whole family into one
vote sink, or one zero-weight projection when a ring lands on a directed
sink; the bundle is not remembered (`mutation/graph/recruitment.rs:38-64`).
`RefineHeritableStructure` (default on) infers groups on every event from
sink edges (`GroupKey::Ring` and `GroupKey::Slots`, the latter keyed on the
three nearby-creature families and their widths,
`mutation/graph/refinement.rs:38-83`) and adds the same `s/√n` to every
member.

**Readers.** `ring_rules.rs` detects direction-general rules through direct
edges, bus paths and projections (`:123-250`); `drives.rs` detects fan-outs
as one source repeated across a kind's direction sinks (`:57-84`);
`plasticity_census.rs` reads per-node configs and runtime storage
(`:153-178`). The lab's `readings::ablate` and the core's
`input_use::ablated` mask inputs by rewriting source references on a copy
(`neighborhood/input_use/consumers.rs:233-300`;
`crates/v3-lab/src/readings.rs:655-710`) and mask projections through the
hidden field. Three places treat projections differently from edges:
`neighborhood/opportunity/discovery.rs:123` leaves projection channels out
of the vote surface, `crates/v3-lab/src/ladder/track.rs:63-69` reports a
projection-only motor node as not voting, and the frontend's
`meshSemantics.ts` ignores projections and plasticity while the server's
annotations count them.

## 3. Domain model

Module paths are today's: the genome in `creature/genome/cgp.rs` (split into
`edge`, `group` and `container` submodules), learning state in
`runtime/plasticity`, reproduction in `simulation/actions/cgp_reproduction.rs`.
No type or module is renamed (D11).

```rust
/// A heritable connection. Stays in its container's `inputs` list.
pub struct GraphEdge {
    pub source: GraphSource,
    /// Private: readers choose `own_weight()`, `def.genomic_weight(occurrence)`
    /// or the runtime's applied weight. Inside a group it is a correction on
    /// top of the group term, born 0.
    own_weight: f32,
    /// Today's type, moved from the compute node (F06). `None` is static.
    pub plasticity: Option<PlasticityConfig>,
    /// Set only on sink edges that fit a group's shape (F03; `scope` in F07).
    pub membership: Option<Membership>,
}

pub struct Membership { pub group: GroupId, pub scope: Scope }
pub enum Scope { Edge, Group }
pub struct GroupId(u16);

pub struct Group {
    pub shape: Shape,
    pub kind: DirectedVoteKind,
    /// Rotation, always `< 8`.
    pub offset: u8,
    /// One shared weight, born 0.
    pub scale: f32,
    /// Whether a learned scale seeds the child's runtime state (F07).
    pub inherit_scale: bool,
}

pub enum Shape {
    /// Members: ring cell `c` of `input_refs[ring_ref]` into `K[(c + offset) mod 8]`; pattern 1.
    Ring { ring_ref: u16 },
    /// Members: the pair's x or y field of `input_refs[ref_idx]` into any
    /// direction sink of `K`; pattern is the compass projection (§4) rotated
    /// by `offset`. `pair` indexes the family's declared pairs (§4).
    DirectionPair { ref_idx: u16, pair: u8 },
}

/// Pure composition, the only formula: genomic and applied weights both use it.
fn compose(own: f32, scale: f32, pattern: f32) -> f32 { scale * pattern + own }

impl CgpGraphBackendDef {
    /// Genomic weight of the edge at `occurrence`; resolves the group itself.
    pub fn genomic_weight(&self, occurrence: EdgeOccurrence, input_refs: &[InputReference]) -> f32;
}
// runtime/plasticity: applied_weight(occurrence, &LearnedState) composes learned
// own weights and learned scales where they exist, genomic values otherwise.
```

**Birth seeds (D1).** Birth correspondence stays a draft-carried payload, as
`birth_weights` is today: `BirthSeeds { compute rows, sink rows, one slot per
group }`, a `#[serde(skip)]` private field of `CgpGraphBackendDef` that
exists only on the child draft between reproduction's capture and its
construction of the child's runtime state. It rolls back with the engine's
genome snapshot, travels with topology clones, and is realigned by the
container primitives and nowhere else. The custom `Debug` and `PartialEq`
impls exclude exactly this field; an exhaustive destructuring test makes any
other new field a compile error. This reverses the version 1 edit-record
design (Codex dispositions 1, 27, 28, 34, 35): the code already gives the
payload every property records would have rebuilt, and records would have
needed special cases for route operators, splits, appends and mesh-level
provenance.

**Invariants (closed list; the container upholds them, `validate_graph`
checks them at every deserialization boundary and in the parseability gate
under a distinct skip reason, D5).**

- I1 Every own weight, scale and plasticity parameter is finite; rates and
  decays in `[0, 1]`, clamps in `[0.01, 10]`, as the mutation laws bound them.
- I2 Only a sink edge carries a membership; compute-input edges never do.
- I3 Every `GroupId` is below `groups.len()`; `groups.len() ≤ u16::MAX`.
- I4 A member fits its group's shape: `Ring`, source `InputLeaf { ring_ref, c }`
  with `c < 8` into sink `K[(c + offset) mod 8]`; `DirectionPair`, source the
  pair's x or y field into a direction sink of `K`.
- I5 At most one member per `(group, source, target)` coordinate.
- I6 A group has at least one member; removing the last member removes the
  group.
- I7 A shape is well-formed: `input_refs[ring_ref]` is a ring family; `pair`
  is below the declared pairs of `input_refs[ref_idx]`'s family; `offset < 8`.
- I8 A member's `sub_idx` is canonical (below its compound width); world
  compounds wrap today, so `(ring, 8)` would read cell 0.
- I9 Identity: `Debug`, `PartialEq` and JSON cover every heritable field and
  nothing else; `BirthSeeds` is the only `#[serde(skip)]` field on any
  genome type; genome types carry `deny_unknown_fields`.

Decisions behind the model:

- **Edges stay in their containers.** A global edge list with identifiers was
  considered (§6, option D): input order is semantic, the trace payload is
  ordered per target, every reader walks a container. A `GroupId` on the
  edge gives explicit membership without identifiers.
- **Membership is a validated coordinate** (I4), not a stored position; the
  pattern is recomputed from `(source, target direction)`, never stored. An
  operation that changes a member's source outside a shape operator
  (retarget to a different source, raw-field step, split through a compute
  node) detaches it first.
- **Members are materialized.** A `Ring` group is eight member edges plus one
  record. That is what lets a mutation delete, re-weight or detach one
  member while the group survives, the HybrID pattern (§5). The hot path
  loses the projection term and gains one indexed lookup per member edge.
- **Pattern zeros are not materialized at birth.** A pair member for `x` has
  pattern 0 toward N and S, so creation makes 12 members, not 16; an omitted
  member is geometrically unnecessary at birth, not impossible, and a later
  `AddGraphEdge` on that sink is an ordinary edge, never a member (F08
  confirms the no-join rule).
- **Learned state is runtime state**, constructed once from seeds at birth
  (D8): a learned own weight for every learning edge with scope `Edge`, a
  trace for every reward-modulated edge, and one learned scale per group
  with a scope-`Group` learner. There is no lazy initializer. A per-module
  visited bit keeps phase 2.5's "initialized modules only" semantics. The
  genome never absorbs learned values.
- **Detach** is genomic: `own := compose(own, scale, pattern)`, `membership
  := None`, and the group's seed slot composes into the edge's seed in place
  when the edge is Lamarckian; a static member keeps no learned state before
  or after. `RemoveGroup` removes the record and every member; `RemoveGraphEdge`
  on the last member removes the group (I6).
- **Accounting contract.** `genome_size` = mesh nodes
  + input references + route targets + compute nodes + edges on both
  surfaces + group records (VM: program length + constants); the
  per-wired-sink unit and the projection unit go. The founder is 97 → **80**
  (node 0: 20, node 1: 60; ForageFirst profiles 100 → 83), the anchor
  `FOUNDER_GENOME_SIZE_UNITS` re-pinned so the founder's replication
  multiplier stays 1.0 and its gestation 1 tick; the founder's carrying cost
  falls 0.0097 → 0.0080 per tick. For a creature with `P` projections and
  `S` wired sinks, `Δgenome_size = 8P − S` (`P` counted as the ring groups
  the same wiring would be under the new rule); a ring group is +8 relative
  to non-carriers. Readers that move: carrying cost (`1e-4` per unit per
  tick), mutation supply (`Binomial(size, rate)`: the per-unit rate becomes
  `0.485 / 80 = 0.0060625`, option a, user 2026-10-13, so the founder's
  supply of 0.485 events per birth and its clone fraction are unchanged,
  while an evolved genome of the same size draws up to 21% more events than
  under the old rule, less the sink units it no longer carries),
  the replication multiplier, gestation (`ceil(size × rate / 80)`, larger
  for every genome above the founder, ×1.21 at depth), the size-pressure
  roll against the 1,200 cap (off by default, not re-anchored), and the
  drift-walk and recruitment-paths instruments that draw
  `Binomial(anchor, rate)`. Functional complexity keeps its rules: a wired
  sink counts one whether its edges are members or not, sink edges are not
  counted individually, a group record counts one, a consumed reference one;
  a ring group onto previously unwired sinks raises complexity by up to 8,
  which the action-cost multiplier, the reproduce charge and the predation
  kill bonus read, all off at defaults.

## 4. Mechanisms

**Compass pattern.** The table is derived from `Direction::delta()`
(`contracts/direction.rs`), not typed by hand: for direction `d`,
`px[d] = normalized dx`, `py[d] = px[(d − 2) mod 8]`, with the diagonals at
`std::f32::consts::FRAC_1_SQRT_2` so every row has norm 1 within 1 ulp and
the axis zeros are exactly `0.0`. A pair member for `x` into `K[d]` has
pattern `px[(d − offset) mod 8]`; the `y` member `py[...]`. With a positive
scale the group votes toward the sensed position; offset 4 votes away; the
sign of the scale is evolved. The applied weight is two rounded operations
(no `mul_add`) and sums stay sequential, so results are platform-stable.

**Pair metadata lives in `contracts::inputs`** beside `compound_width` (D12):
each world family declares its width, its field offsets and its direction
pairs (`NearbyCreature` slot k `(dx, dy)`; `AreaOccupancySummary`
`(centerX, centerY)` and `(nearestDx, nearestDy)`; `AreaFoodSummary(t)`
`(gradX, gradY)` and `(nearestDx, nearestDy)`; `AreaBarrierSummary` the same
two). `sensors::perception` writes against it, the shape validates against
it, labels, census, lab and frontend read it. Shape eligibility depends on
the `WorldInputKey` variant only, never on a food type, so `InputRef.Swap`'s
kind partition preserves it.

**Learning on edges (F06, F07).** The update laws are today's
(`runtime/plasticity/hebbian.rs:138-145`, `reward.rs:27-88`) with the edge's
own configuration. Contract:

- *Layout at birth.* A per-backend learning layout (a `has_learning` flag,
  the slot list, precomputed genomic member weights) is built once at
  creature construction like `cached_complexity`; `execute_graph_impl` is
  monomorphized on a learner parameter beside `GraphTracer`, so a
  non-learning graph runs today's loops with no sample branch; phases 0 and
  2.5 iterate the slot list, never the genome.
- *Samples.* The executor records, for every learning edge, the pre value
  the evaluation used (compute inputs: the ordered read; sink inputs: the
  value the sink sum read, in sink order) and the post value (the node's
  committed output; the sink's `sanitize_f32(wsum)`). Effects write shared
  memory as the sink loop proceeds, so a later re-resolve would differ.
  Recording a sample costs 4 B per learning edge in reused scratch; the two
  `action_queue` clones the replays needed go. On recurrent compute edges
  this changes pure-Hebbian semantics (today's replay reads this visit's
  output for self and higher sources): predeclared, with a self-edge
  regression.
- *Order.* Evaluation reads the applied weight in force at visit start;
  every Δ, Oja's term included, is computed from visit-start values after
  effects; then contributions apply in canonical `(sink, edge)` order with
  their clamps. The cost chain is deleted (D9), so no visit is voided by
  learning.
- *Scope and credit (D4).* A member with scope `Edge` updates its learned own
  weight. A member with scope `Group` contributes `p_m × Δ_m` to the group's
  learned scale, its own rule, rate, source and decay applied, so a
  negative-pattern member never moves against its rule and `Ring` members
  (pattern 1) contribute Δ unchanged. Contributions in one event sum, and
  the scale is bounded once by the group's bound, the smallest clamp among
  its scope-`Group` learners. A static member contributes nothing. One
  change to the shared scale is what "every member equally" means: every
  member's applied weight moves in proportion to its pattern.
- *Eligibility.* Every wired sink edge is a learning candidate except
  `ClearSlot` edges, whose sink ignores its sum. Vote edges may run pure
  Hebbian learning (runaway allowed by decision, clamp-bounded); per-visit
  sampling is the T19.F02 rule, so credit scales with visits, which F06
  reads beside clamp occupancy.
- *Seeds.* A Lamarckian edge's learned own weight seeds the child; a group
  seeds its learned scale only when `inherit_scale` is set and the child
  group has a scope-`Group` learner; seeds are clamped at construction; the
  container's `set_weight`, `set_source` and `set_scale` clear the seed they
  touch, as a weight step does today; a detached Lamarckian edge is seeded
  with its own learned weight plus the parent's learned scale times its
  pattern.

**Shape operators relocate members in two phases.** `ShiftGroupOffset` and
`RetargetGroupKind` remove every member from its sink, then append each at
its new sink after the independent edges in `(group, channel)` order, so a
shift followed by its inverse returns an equal genome (a proptest). The
pattern travels with the coordinate; correction, plasticity, scope, learned
state and seeds travel with the edge; holes stay holes; nothing is
regenerated.

**Removal.** `RemoveGroup` removes the record and every member and renumbers
every higher `GroupId` in one container operation; `RemoveGraphEdge` on a
last member does the same. Reference reindexing never removes a group: the
only production pruner removes references no consumer addresses, and every
live group's members address its reference, so the reindex is decrement-only
with a debug assertion (D6).

**Edge operators on members.** `RetargetGraphEdge` keeps today's draws (edge,
then source); a same-source draw changes nothing; a changed source detaches
first, then clears the seed. A raw-field step always changes the source, so
it always detaches. A split detaches before rewiring. `AddGraphEdge` never
joins a group (F08 confirms). Copy operators copy compute-input edges only,
so they never copy a member.

**Recruitment emits groups.** `RecruitNeutralInput` draws the family, then
the destination; when the family is a ring and the destination a directed
sink, both arms emit a `Ring` group of eight born-0 members with `offset =
(d − channel) mod 8` under `SingleChannel` and `offset = d` under
`WholeFamily`, as today's projection cases do. When the family declares
pairs and the destination is a directed sink, `WholeFamily` draws one slot
(`NearbyCreature`) or one pair (area summaries) and emits a `DirectionPair`
group of twelve born-0 members with `offset = d` plus one born-0 edge into
`Drive K` per field of that slot in no eligible pair; `SingleChannel` keeps
today's single-edge plan. Recruitment always creates static groups.

**Ablation (D3).** Members are plain sink edges, so today's rewrite ablation
in `neighborhood/input_use/consumers.rs` and the lab's `readings::ablate`
covers them; their projection branches and `ablated_channels` are deleted.
Validation never runs on an observation copy. The lab's `expressed()` stays a
lab-side rewrite that sets every edge's plasticity to `None` and every
`inherit_scale` to false, which touches neither size, complexity,
membership nor order.

**Draw contracts.** Every operator documents its draw order in code and
pins it with a parallel-RNG tail test, as today.

- *Group operators* (F03): `AddRingGroup` draws ring, kind, offset as
  `AddRingProjection` does today; `AlterGroupScale` picks a group then steps
  with `perturb_weight`; `ShiftGroupOffset`, `RetargetGroupKind` and
  `RemoveGroup` pick a group and apply. `AddDirectionPairGroup` (F04)
  enumerates `(ref_idx ascending, pair ascending)` over references whose
  family declares pairs, draws one, then kind, then offset. Applicability:
  the enumeration is non-empty, the 24 direction sinks exist, and
  `groups.len() < u16::MAX`.
- *Plasticity operators* (F06): the nine node operators move to edges with
  their laws unchanged (`hebbian.rs:195-349`): enable, disable, rule, rate,
  Lamarckian flag, enable and disable modulation, source, decay; the edge is
  picked uniformly over both surfaces in canonical order among edges
  matching the operator's predicate; a `ClearSlot` edge never matches.
- *Scope coin* (F07): after picking the edge, if it is a member,
  `gen_bool(0.5)` chooses this edge or every member; the drawn values are
  drawn once; a parameter step under group scope draws one `u` in `[−1, 1]`
  and each member applies its own branch of today's law; a member whose
  variant lacks the field is unchanged and consumes no draw; a non-member
  draws no coin. `SetScope` picks a member, draws the coin, writes the other
  scope; no value draw.
- *Static creation* (F03, F04): shape draws only; members static, scope
  `Edge`, `inherit_scale` false, scale and corrections 0.
- *Uniform learned creation* (F07): shape draws, then one mode over
  `{static, Hebbian, reward-modulated}`, its fields with today's enable
  laws, one scope coin, then one `inherit_scale` coin after the member
  loop.
- *Randomized creation* (F08, separate operators per shape so F07's draws
  do not move): shape draws, then per member in genome order an independent
  mode, fields and scope, then one `inherit_scale` coin.
- *`DetachGraphEdge`* (F08): one member uniformly in canonical edge order;
  no coin, no value draw; detaching the last member removes the group.

**Refinement disposition (D7).** `GroupKey::Ring` and the ring arm of
`correspondence` are deleted in F03: the explicit group's step is
`AlterGroupScale`, and an inferred ring beside an explicit one would be two
mechanisms for one concept. Refinement's private `Group` and `Member` are
renamed `InferredSet` and `InferredMember`, and its weight writes go through
the container setter so they clear seeds like every other weight step. The
`Slots` key is re-keyed on the merged family in F05 (`field = channel mod 9`,
`slot = channel div 9`) and kept until F08 decides, from refinement
applications recorded by key on F04's goal-long and F05's goal runs, whether it
stays, becomes a shape, or goes.

**The creature-sensor family (F05).** The three families merge into one
`NearbyCreature` family (4 slots × 9 fields = 36 sub-values) declared in
`contracts::inputs`. The sampler keeps 27 buckets with indices 10–12 all
drawing `NearbyCreature` (D13): the family is 36 wide against 7 or 8 for
others, so three buckets is proportionate, and the draw pins hold.
Recruitment takes one slot everywhere, which keeps the 30-edge bound.

**Non-goals (D11).** The VM backend (no edges); `Drive` sinks as group
targets; compute-input edges as members; founder rewiring; type and module
renames; generating the operator enums from a table (a bijection test is
added instead); the four domain selection loops; the dangling
`ComputeNode(u16::MAX)` sentinel; tolerance of out-of-range `InputLeaf`
reads; the unbounded steal amount (program 1 carried finding 5).

## 5. Evidence

**Local (complete).**

- Live creature-sensor use, 2026-10-12: §1. Only the ring-shaped input became
  behavior; whole-family bundles into one sink did not.
- Barrier awareness probe
  ([note](barrier-awareness-probe-2026-10-12.md)): barrier avoidance
  evolved through ring projections, not through single edges or VM reads.
- Factored motor output note
  ([note](factored-motor-output-research-2026-10-11.md), §4–§5): the
  founder's four-direction taxis eroded to a median of one direction before
  projections; one gene for all eight directions is the best-supported
  encoding, with per-direction exceptions (HyperNEAT and HybrID, Clune et al.
  2011; Huizinga et al. 2014: shared encodings struggle to prune one
  connection, so keep per-direction edges). Read in full on 2026-10-11.
- Three code maps and one Codex Astra review round, 2026-10-13: §2, §7 and
  the readings file.

**External.** Research date 2026-10-13.

- A 2D vector is held as a sinusoidal pattern over a ring of neurons, "with
  the sinusoid's amplitude representing the vector's length and its phase
  representing the vector's angle" (Lyu, Abbott & Maimon 2022, Nature
  601:92, [author manuscript](https://pmc.ncbi.nlm.nih.gov/articles/PMC11104186);
  verified at source). The compass pattern is that encoding on eight cells.
- Each connection has a fixed weight and a plastic component scaled by its
  own plasticity coefficient, so a connection can be fixed, plastic or mixed
  (Miconi, Clune & Stanley 2018, [arXiv:1804.02464](https://arxiv.org/abs/1804.02464);
  summary). That is `Learning::Static` beside a learning edge.
- Evolving a learning rule per synapse instead of weights (Floreano & Urzelai
  2000, [EPFL record](https://infoscience.epfl.ch/record/63912); Najarro &
  Risi 2020, [arXiv:2007.02686](https://arxiv.org/abs/2007.02686); summaries).
- Three-factor rules: a pre/post coincidence flags a synapse with an
  eligibility trace, and a later reward or novelty signal converts it to a
  weight change (Gerstner et al. 2018, Front. Neural Circuits 12:53,
  [PMC6079224](https://pmc.ncbi.nlm.nih.gov/articles/PMC6079224); summary).
  Petri already has this shape; the refactor gives each edge its own decay
  and source.
- Plasticity is set largely by synapse type, a group, rather than per
  synapse (Larsen & Sjöström 2015, Curr. Opin. Neurobiol. 35:127,
  [PMC5280068](https://pmc.ncbi.nlm.nih.gov/articles/PMC5280068); summary).
  That is a group's uniform creation with per-member exceptions.
- Review of evolved plastic networks (Soltoggio, Stanley & Risi 2018,
  [arXiv:1703.10371](https://arxiv.org/abs/1703.10371); summary).

Labeling: "summary" means the claim rests on an abstract or a search summary,
not a full read. The depth bar asks for full reads before a commitment; the
spec owner of F06 should read Gerstner 2018 and Miconi 2018 in full.

## 6. Options

| Option | Design | Verdict |
| --- | --- | --- |
| A. Materialized members and group records | §3 | **Chosen.** Per-edge exceptions and deletion, one composition function, readers read membership |
| B. Generator groups | Today's projection generalized: a group record computes its terms, no member edges | Simplest hot path and size accounting, but no per-member deletion or exception; rejected by the user's "both" |
| C. Groups only, learning stays per node | Add groups, keep `PlasticityConfig` on compute nodes | Does not meet the per-edge learning decision; vote edges still cannot learn |
| D. Global edge list with identifiers | Edges leave their containers; every reader keyed by id | Moves every walker and the trace order for no behavior gain; defer until a reader needs stable identity |
| E. Bundle gene | One gene computing strength × direction | Rejected 2026-10-13: bakes in a computation |
| F. Turn every directional sense into a ring | Sector rings per 45° slice | Rejected 2026-10-13: not what the user meant; sector asymmetry (13 vs 17 cells) remains an open question for a later shape |

## 7. Boundaries and decomposition

Each boundary owns one thing, at today's module paths. Three concerns that
share code today are separated on purpose: *structural editing* of the
genome, *birth seeds* (which child edge or group descends from which parent
one, used once at reproduction) and *learned-state construction*. The win
the user asked for is concrete: today about twelve mutation sites keep the
birth payload aligned by hand, three readers infer structure the genome will
store, one hidden field exists for one caller, and three copies of
"learned value if present, else genomic" exist. After the refactor each has
one owner.

| Boundary (path) | Responsibility | Today |
| --- | --- | --- |
| `creature::genome::cgp::edge` | `GraphEdge`, `Membership`, `Scope`; field-level validation (I1) | `GraphEdge` |
| `creature::genome::cgp::group` | `Group`, `Shape`, `GroupId`, the compass pattern from `Direction::delta()`, `compose`, `genomic_weight`, the coordinate checks (I4–I8) | `RingProjection`; inference in `refinement.rs`, `ring_rules.rs`, `drives.rs` |
| `creature::genome::cgp::container` | The only writer of `inputs`, `compute_nodes`, `output_sinks` and `groups` (private fields): `push_edge`, `remove_edge`, `retain_edges`, `move_edge`, compute-node insert, append, remove and duplicate, reference reindex, `add_group`, `remove_group`, `detach`, an `EdgeMut` handle whose `set_weight` and `set_source` clear the seed (`set_source_keeping_seed` for the split only), `set_scale`; keeps `BirthSeeds` aligned; no RNG, no shape semantics (shift and retarget sit in `group` on top of `move_edge`) | ~12 hand-maintained alignment sites in `operators.rs` and `cgp.rs`; `pub` fields written from `input_ref/mod.rs`, `topology/routing.rs`, `refinement.rs`, the founder, instruments and the lab |
| `contracts::inputs` | Family widths, field offsets and direction pairs: one layout both backends, the shape, labels, census, lab and frontend read | widths in `contracts/inputs.rs`, offsets in `sensors/perception.rs` |
| `simulation::actions::cgp_reproduction` | Captures `BirthSeeds` before mutation and hands them to the runtime constructor after, under the child's post-mutation flags; keeps the "parent learned nothing" fast path | `capture_birth_weights`, `build_cgp_child_plasticity_weights` |
| `runtime::plasticity` | `LearnedState::from_seeds` (the only initializer), `capture()` for reproduction, the learning layout, `applied_weight` (the only resolver), samples, Hebbian and reward updates, scope routing, group bounds, a read-only iterator `(occurrence, genomic, learned, bound, mode, scope)` for census and telemetry; storage `pub(crate)` | three fallbacks, two evaluation paths, lazy init, `pub` storage read by the census and v3-telemetry |
| `runtime::cgp::effects` | Sink sums over applied weights through a sample sink on the tracer seam, no-op for non-learning graphs; no projection term | `effects.rs:57-99, 133-171` |
| `mutation::graph` | Edge and group operators, the scope-coin helper, the nine plasticity operators on edges; every structural change through the container; a bijection test over the operator enums | `operators.rs`, `hebbian.rs`, `projection.rs`, `refinement.rs`, `recruitment.rs` |
| `mutation::engine` | One generic snapshot, validate, commit-or-rollback wrapper (F01) | four near-identical wrappers |
| `analysis` and `census` | Explicit membership from the genome (plasticity carriers through the runtime iterator; sensor census "wired" through members); derived wiring analyses over non-member edges and genomic weights, with pair groups reported in their own block | inference in three files; `ablated_channels` |
| v3-lab | Rewrite ablation over plain edges; `expressed()` a lab rewrite; the comparator stays a lab solution built through the plain-edge constructor | projection special cases; the hidden field |
| Transport and inspector | Serde shape; a top-level `graph_weights` field beside `genome` (own `exclude` token, keyed by node id, sparse: members and learning edges only, genomic and applied values, absent never zero); `mesh_annotations` required with the genome and the frontend's inference deleted; an edge-group list; a per-edge learning badge; sink `weighted_inputs` in the trace | raw `edge.weight` drawn; two sources of truth in `meshSemantics.ts`; no plasticity display |

Dependency direction: `contracts` ← `creature::genome` ← `runtime` ←
`simulation`; `sensors` reads `contracts` and `creature`; `creature::genome`
never imports `sensors` or `runtime`. v3-lab, v3-telemetry, v3-cli and the
server read v3-core through the iterator and the composition functions, never
through storage.

**Validation homes (D5).** The container primitives uphold I1–I9 with debug
assertions and are covered by proptests over random operation sequences.
`validate_graph(&NodeGenome)` runs at every deserialization boundary (server,
lab, bench artifacts) and in the parseability gate after every event under
its own skip reason and counter, so a container defect surfaces as a named
count, never as a silent rollback.

**The three readers that treat projections differently today** get explicit
post-change expectations in F03: a motor node whose only votes come from
members is a voting node in `ladder/track.rs`; member channels are on the
vote surface in `discovery.rs`; the frontend counts members as action
writers through the server's annotations. Readers the change must not
affect are pinned on group-free fixtures.

**Determinism and pins.** F01 moves no pin; any movement under it is a
defect. From F02 on every feature predeclares its re-pins, one cause per
commit, each re-pin naming its mechanism and a restoring check; F01 adds an
absolute mutation-off digest that stays byte-identical through every later
feature except F02 with the carrying cost set to 0.

## 8. Contracts (placed 2026-10-13; the track rows are authoritative)

Order: F01 → F02 → F03 → F04 → F05 → F06 → F07 → F08, a linear chain (D10).
Every feature: `make check` green at its end, the mutation gate and the slow
tests run, a "Predeclared re-pins" line, a "Reference specs" line, and a
"Tests first" list the implementer writes red before code.

- **T25.F01 — Edge Container, Constructors and Birth Seeds.**
  - Goal: Development separated from heredity, as a cell's machinery for
    building synapses is separate from what the egg carries: one owner edits
    the graph, one captures what a child is born with, and one builds the
    running brain from it, with no behavior change.
  - Scope: the container module with private fields and the primitives of
    §7; `GraphEdge::new(source, weight)`, `ComputeNode::new(kind, inputs)` and
    a def builder, with every struct literal (256 `GraphEdge {`, 183
    `ComputeNode {`, 70 def literals) and the ~15 local test helpers migrated
    to them across every crate and the frontend fixtures; `birth_weights`
    renamed `BirthSeeds` as a private container-kept field; the one generic
    engine wrapper; `clamp_sub_idx_after_swap` and the dead reindex branches
    deleted; `connect_declared_input`, routing and refinement writing
    through the container; the drift-walk dating test rewritten to a
    constructed lineage. `BirthEdgeClass` stays as it is (non-goal).
  - Verification: gate; mutation gate run; slow tests run. No pin moves.
  - Predeclared re-pins: none.
  - Oracles: a seeded golden corpus recorded on main before any change
    (Lamarckian plastic genomes with tagged learned values, mutated through
    `apply_mutations_on_units` at a high rate over all four domains,
    rollbacks included; a hash of each child genome and its built weights),
    added to `make check`; an absolute mutation-off trajectory digest; the
    inheritance tests ported scenario by scenario with identical expected
    values (`reproduction.rs:1012` as an explicit backend-replacement test);
    the existing absolute pins (`applied_trajectory`, `baseline_worlds`,
    recruitment hash, drift, slow tier) unchanged.
  - Reference specs: `v3-reproduction-spec.md` Lamarckian section (payload
    name).
- **T25.F02 — Uniform Genome Size.**
  - Goal: Every synapse costs the same to carry wherever it sits, as a
    neuron pays per synapse and not per target: an edge is one unit on both
    surfaces, a wired sink costs nothing of itself, and the founder is
    re-anchored.
  - Depends on: T25.F01.
  - Scope: the accounting contract (§3); `FOUNDER_GENOME_SIZE_UNITS` 97 → 80
    and the instrument constant of the drift walk and recruitment paths;
    the per-unit rate `0.0060625` (option a) as the new default;
    `REACHABLE_STRUCTURE_VERSION` v2 and the
    drift-depth version; a `genome_size` definition-change entry in
    `benchmark-series.json`; the T20.F02 storage-contract table and test;
    the ~15 doc sites citing 97.
  - Verification: `cargo test -p v3-core --test viability` first; gate +
    goal + goal-long (the size question is a long-run one); mutation gate
    run; slow tests run.
  - Predeclared re-pins: the founder anchor pin (`state.rs:808-815`,
    `phase0.rs`; the supply pin at `config/simulation.rs:1362-1369` keeps
    0.485), the bench config digests (`bench_artifacts.rs:220-321`: a
    default config value changes), the lifecycle storage test,
    `drift_depth` and the progress chart, goal readings that carry genome
    size, carrying cost, gestation, supply and exposure.
  - Tests first: `genome_size_charges_no_wired_sink_unit`,
    `a_ring_group_costs_nine_units_on_wired_or_unwired_sinks` (as a plain
    eight-edge fixture until F03), `founder_is_eighty_units` with the
    arithmetic in a comment.
  - Readings: mean genome size, mean wired sinks per creature, births per
    creature-tick and gestation on the three goal worlds through 40,000
    ticks against the last `goal_worlds_long` point; mutation events per
    birth at depth against the old rule, named as a cause.
  - Reference specs: `v3-genome-spec.md` §3 (wired-sink rule, 97),
    `v3-runtime-config-spec.md` (`genome_size` definition, cost rows),
    `v3-reproduction-spec.md` (97), `v3-mutation-spec.md` §4.2 (supply),
    `v3-graph-backend-spec.md` §6 size table.
- **T25.F03 — Ring Groups Replace Ring Projections.**
  - Goal: One developmental program wires a whole map (Eph/ephrin gradients,
    central-complex columns): a ring projection is a group of eight real
    synapses with one shared scale, and any one synapse can later be changed
    or lost on its own.
  - Depends on: T25.F02.
  - Scope: `Membership`, `GroupId`, `Group`, `Shape::Ring`, `compose`,
    `genomic_weight`, `own_weight` private, I2–I9 and `validate_graph` with
    its gate skip reason; derived `Debug`/`PartialEq` apart from the seeds
    field; `RingProjection`, its evaluator and its list deleted outright,
    `Ring` groups in their place with no converter and no serde alias (a
    running world restarts from the founder, which has no projections); the
    five group operators (table below) with two-phase relocation; detach on retarget,
    raw-field and split; `GroupKey::Ring` deleted and refinement's structs
    renamed; recruitment's ring case under both arms; the projection
    branches of both ablation paths and `ablated_channels` deleted;
    `live_barrier_use.rs` deleted; readers (`ring_rules` → ring-rules-v2 with
    `ring_group_*` fields, `drives` skipping members, `sensor_census`,
    annotations, `companions`, `live_circuit`, `discovery.rs`, `ladder/track.rs`,
    `steering::votes_move`); the serde shape (`groups` replaces
    `projections`, `deny_unknown_fields`) and the lab `GENOME_FORMAT` bump;
    the server's `graph_weights` field with its `exclude` token and a
    protocol-version bump; `mesh_annotations` required with the genome and
    the frontend's `infer*`/`collectNodeBadges` deleted; the edge-group list
    (one row per group: shape, kind, offset, scale, members present over
    expected with holes; expandable member rows; member edges dashed in a
    group tint with opacity from the server's genomic weight; a per-group
    collapse); `types/genome.ts`; petri-visuals `edgeGroups` and the min
    build.
  - Operators: `AddRingProjection` → `AddRingGroup` (2), `AlterRingProjectionWeight`
    → `AlterGroupScale` (4), `ShiftRingProjectionOffset` → `ShiftGroupOffset`
    (2), `RetargetRingProjectionKind` → `RetargetGroupKind` (2),
    `RemoveRingProjection` → `RemoveGroup` (2, `Decreasing`); `ALL` stays 28,
    the default arm 56, `[Self; 61]`, slots and weights unchanged; only names
    and keys move.
  - Verification: gate + goal (operators and the +8-per-group cost change);
    mutation gate run; slow tests run. Predeclare the gate's
    `wall_clock_ms_per_creature_tick` delta.
  - Predeclared re-pins: operator keys in events and telemetry (stored
    artifacts stay historical, no migration); genome identity and JSON;
    `GENOME_FORMAT`; `PROTOCOL_VERSION`; `neutral_connection_tests/projection.rs:212`
    rewritten by hand to the new summation order; goal readings that carry
    genome size (+8 per group), complexity and `ring_rules`.
  - Tests first: a `Ring` group (scale w, corrections 0) equals eight
    ungrouped edges of weight w bitwise, with members appended after
    incumbents (`eight_edges` oracle, `projection.rs:99,126,155,189` swapped
    to the group builder); validation rejects an off-ring or out-of-range
    member; `remove_group` leaves no member and renumbers links;
    `shift_then_unshift_is_identity`; `detach_preserves_genomic_weight_bitwise`;
    `refinement_never_steps_a_grouped_member` (RNG tail equal to the genome
    without members); `a_node_voting_only_through_group_members_is_a_motor_node`;
    `group_member_channels_are_on_the_vote_surface`; the ring-rules and
    drives census on a `Ring` group equals the census on eight ungrouped
    edges of weight `s + c_i`; the catalog bijection test; `ablate` on a
    group-free genome unchanged and on a ring reference zeroes every member
    read; proptests I1–I9 after random container operations and
    `Δgenome_size` per operation equal to its declared change; the
    identity destructuring test; serde and `Debug` round trip over random
    valid genomes.
  - Readings: the three readers' post-change expectations; members per
    surviving group and detach or remove events per group over the goal run
    against the erosion rate the weights predict; goal indicators not down.
  - Reference specs: `v3-graph-backend-spec.md` §2, §6 (ring projections →
    groups), §9, §14; `v3-genome-spec.md` §3, §4; `v3-mutation-spec.md`
    projection and recruitment rows; `v3-server-api-protocol-spec.md` §4.11.
- **T25.F04 — Direction-Pair Groups.**
  - Goal: The sensory map sits in register with the motor map (superior
    colliculus): a creature's sensed offset to a thing, food, a barrier, a
    crowd or another creature, can reach all eight direction votes of one
    kind as one silent gene, and its plain facts reach that kind's drive.
  - Depends on: T25.F03.
  - Scope: `Shape::DirectionPair`, the compass pattern from
    `Direction::delta()`, pair metadata in `contracts::inputs` for today's
    families; `AddDirectionPairGroup`; the five group operators extended to
    pairs (relocation around holes); recruitment's pair case with one slot
    or one pair and scalar drive edges; `drives` reporting pair groups in
    their own block (drives-v2); the inspector rows and petri-visuals.
  - Operators: `AddDirectionPairGroup` (2, `Increasing`): `ALL` 29, arm 58,
    `[Self; 62]`.
  - Verification: gate + goal-long (retention is visible only past 20,000
    ticks); mutation gate run; slow tests run.
  - Predeclared re-pins: every Graph draw (a new operator); the catalog pins;
    drives-v1 → v2; refinement applications recorded by key for F08.
  - Tests first: the compass table's row norms and exact zeros;
    `pair_creation_materializes_12_members_not_16`; every declared pair's
    indices within the family width; shift and retarget on a pair group
    with holes; recruitment's pair case draw order; `node_fanouts` skips
    members.
  - Readings: a versioned bench block of pair-group class counts (family,
    kind, offset, sign of scale, scale off 0) at tick T−N and at T beside
    the `AddDirectionPairGroup` applications between them; blocked-move and
    steal-hit fractions of carriers against non-carriers; harmful-per-birth
    not up; refinement applications by key.
  - Reference specs: `v3-graph-backend-spec.md` §6 (shapes),
    `v3-mutation-spec.md` operator table, `v3-sensor-spec.md` (pair
    declarations).
- **T25.F05 — The NearbyCreature Family.**
  - Goal: One sense of another creature, as an animal reads a neighbor's
    place, condition and kinship in one glance: the three creature families
    become one, so a slot's position, vitals and identity arrive from one
    declaration.
  - Depends on: T25.F04.
  - Scope: the merged layout in `contracts::inputs` (4 slots × 9 fields);
    the sampler (27 buckets, indices 10–12, D13) and its pins; labels,
    `inputRefUtils.ts`, the sensor census universe, the input-use catalog
    (input-use-v2), mutation-effects contexts (mutation-effects-v2), the
    `WorldReads` bit layout, span attributes, the sampler perception payload
    (protocol bump), telemetry gauge values and dashboards, `MutationAddedNodeInputClass`,
    annotation classes, the lab's `census_covers`; the `Slots` refinement
    key re-keyed; recruitment taking one slot everywhere.
  - Verification: gate + goal; mutation gate run; slow tests run.
  - Predeclared re-pins: every `InputRef.Add` and recruitment draw (the
    family universe), the sampler index tests, genome identity of every
    genome with a creature reference, `PROTOCOL_VERSION`, the sensor gauge
    series (99 stored feature JSON files carry the old keys; historical).
  - Tests first: `merged_nearby_creature_equals_core_vitals_identity_per_slot`
    written while the three families still exist, kept as the layout pin;
    the `Slots` fixtures rewritten to `field = channel mod 9`.
  - Readings: declaration share of the merged family against the three old
    ones; goal indicators not down.
  - Reference specs: `v3-sensor-spec.md` §5.4–5.6, `v3-mutation-spec.md`
    sampler table.
- **T25.F06 — Learning Per Edge.**
  - Goal: Synapse-type-specific plasticity with three-factor traces: each
    synapse carries whether it is fixed, Hebbian or reward-modulated, which
    reward it listens to and how long its trace lasts, and a synapse onto a
    vote can learn like one onto a neuron.
  - Depends on: T25.F05.
  - Scope: `plasticity: Option<PlasticityConfig>` on the edge and
    `ComputeNode.plasticity` removed (36 production files); the learning
    layout at birth and the monomorphized non-learning path; samples;
    updates after effects; the cost chain deleted (D9); `LearnedState::from_seeds`
    over both surfaces with the visited bit, `capture()`, the iterator;
    `BirthSeeds` sink rows; the nine plasticity operators on edges with the
    `ClearSlot` exclusion and the split exclusion redefined as "the graph
    has a learning edge"; `plasticity_census` through the iterator; the
    telemetry families (`learning_edges` by mode and surface, rule its own
    family) and the dashboards check; `companions` with "carries learning"
    defined once; annotations; recruitment-paths records (v2);
    mutation-effects state records; the serde shape, `types/genome.ts`,
    `graph_weights` extended with learned values, sink `weighted_inputs` in
    the trace payload, the learning badge, the config tooltip; the lab's
    `expressed()`.
  - Operators: the nine move with their weights (18 of the arm): `ALL` 29,
    arm 58, `[Self; 62]` unchanged in count; keys renamed.
  - Verification: viability first; gate + goal; mutation gate run; slow
    tests run. Predeclare `wall_clock_ms_per_creature_tick`, the
    `plasticity_updates` shift, phase 0 and 2.5 timings and a dhat heap
    check on a fixture with N learning sink edges.
  - Predeclared re-pins: identity of every plastic genome; every plasticity
    draw; the recurrent-edge Hebbian trajectories; `plasticity/tests.rs:35`
    and `:212` rewritten by hand to "no charge, effects stand"; the
    cognitive-class and `has_plasticity` series; `GENOME_FORMAT`;
    `PROTOCOL_VERSION`.
  - Tests first: `sink_edge_learns_from_the_recorded_sample_not_a_replay`
    (a `WriteSlot` earlier in sink order changes memory a later sink edge
    read); `a_non_learning_graph_allocates_no_learned_state`; the
    self-edge regression; the behavior oracle keeping
    `plasticity/tests.rs:140-661` expectations with each node's config on
    its compute-input edges; `clear_slot_edges_are_never_eligible`.
  - Readings: learning edges by mode and surface; drift and sampled clamp
    occupancy by surface; visits per tick of learning nodes; goal indicators
    not down.
  - Reference specs: `v3-graph-backend-spec.md` §12, §13; `v3-genome-spec.md`
    §5; `v3-mutation-spec.md` plasticity rows and the split exclusion;
    `v3-mesh-execution-spec.md`, `v3-tick-orchestration-spec.md`,
    `v3-evolution-observability-spec.md`, `v3-cli-contract-spec.md`
    plasticity rows; `v3-runtime-config-spec.md` cost knobs.
- **T25.F07 — Group-Scope Learning.**
  - Goal: A synapse built by a group program can learn for the whole group
    or for itself, and a change to such an attribute lands on one synapse or
    on every synapse its group built.
  - Depends on: T25.F06.
  - Scope: `Scope` in `Membership`, `inherit_scale`, learned scales and
    bounds, the pattern-weighted credit (D4), the scope coin on every
    attribute operator, `SetScope`, uniform learned creation for both
    shapes, group and detach seeds, census rows for scales, the counter
    rule for group contributions.
  - Operators: `SetScope` (2, `Neutral`): `ALL` 30, arm 60, `[Self; 63]`.
  - Verification: gate + goal; mutation gate run; slow tests run.
  - Predeclared re-pins: every Graph draw; identity of every grouped
    genome; the creation draws of F03 and F04 (now learned).
  - Tests first: `oja_group_members_use_visit_start_applied_weight`;
    `group_scale_credit_is_pattern_weighted_and_bounded_once`;
    `a_static_member_contributes_nothing`; `a_scale_seeds_only_with_a_group_scope_learner`;
    the scope-coin draw-order tails ("a non-member draws no coin", "a member
    lacking the field consumes no draw"); proptest: a scale change `Δs`
    moves each member's applied weight by `Δs × pattern`.
  - Readings: learning edges by scope; learned-scale drift and clamp
    occupancy; goal indicators not down.
  - Reference specs: `v3-graph-backend-spec.md` §13 (scope), `v3-mutation-spec.md`
    operator table.
- **T25.F08 — Creation Forms, Detach and the Catalog.**
  - Goal: A mutation in a developmental gene changes every synapse it builds;
    a mutation at one synapse changes only that one. Group creation is
    uniform or randomized, a member can be detached, and the catalog is one
    consistent set of edge and group operators.
  - Depends on: T25.F07.
  - Scope: `AddRandomizedRingGroup` and `AddRandomizedDirectionPairGroup`;
    `DetachGraphEdge`; the no-join rule for `AddGraphEdge` pinned; the
    `Slots` disposition from the recorded readings; `GraphOperator::layer()`
    for the funnel reading; the catalog tables and tests.
  - Operators: three new at 2 each (`Increasing`, `Increasing`, `Neutral`):
    `ALL` 33, arm 66, `[Self; 66]`.
  - Verification: gate + goal; mutation gate run; slow tests run.
  - Predeclared re-pins: every Graph draw; the catalog pins.
  - Tests first: randomized creation draw order; a detached Lamarckian
    member's seed and a detached static member's; the last-member detach
    removes the group; `add_graph_edge_never_joins_a_group`.
  - Readings: operator funnel by layer; share of attribute changes applied
    at group scope; harmful-per-birth not up; the `Slots` firing rate.
  - Reference specs: `v3-mutation-spec.md` operator table.

## 9. Remaining uncertainty

- Whether evolution picks up pair groups at all; F04's goal-long reading is
  the test, and failure returns the question to sector rings (option F) or
  to a learning route.
- The supply rate under uniform size (D14, option a): an evolved genome
  draws up to 21% more mutation events at equal size, where genome bloat is
  the carried finding; F02's goal-long reading watches genome size, births
  and the event rate.
- Vote-edge Hebbian runaway, allowed by decision; F06 reads clamp occupancy
  and visits per tick.
- Hot path: F03 moves projection evaluation into edge traversal and widens
  the edge struct (12 → ~32 bytes once F06 lands; predeclared per feature);
  F06 adds sample recording on learning graphs only.
- The `Slots` refinement key is an open decision for F08.
- The version 2 edits were not re-reviewed by Codex or the facet reviewers;
  each feature's spec runs its own three-round Codex challenge against this
  note and the code.

## Sources

- Lyu, Abbott & Maimon 2022, Nature 601:92, doi:10.1038/s41586-021-04067-0 (quote verified at source)
- Miconi, Clune & Stanley 2018, ICML, arXiv:1804.02464 (summary)
- Floreano & Urzelai 2000, Neural Networks 13:431 (abstract and draft)
- Najarro & Risi 2020, NeurIPS, arXiv:2007.02686 (summary)
- Gerstner, Lehmann, Liakoni, Corneil & Brea 2018, Front. Neural Circuits 12:53 (summary)
- Larsen & Sjöström 2015, Curr. Opin. Neurobiol. 35:127 (summary)
- Soltoggio, Stanley & Risi 2018, Neural Networks 108:48 (summary)
- Clune, Stanley, Pennock & Ofria 2011; Huizinga, Mouret & Clune 2014; Stanley, D'Ambrosio & Gauci 2009: read in full for the factored motor output note
- Live probe 2026-10-12: 1,478 creature dumps from `GET /v3/simulation/creature/:id` across all 64 tiles of the running world at tick 51,351 (scratch scripts, not committed)
- Codex Astra adversarial review (three rounds) and nine Opus facet reviews, 2026-10-13: `docs/progress/readings/t25-design-review.md`
