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

This is version 3. Version 1 (commit `6b43ffca`) went through three Codex
Astra review rounds; version 2 (`a6f02a81`) rewrote §3, §4, §7, §8 and §9
after nine Opus facet reviews returned 35 blocking findings; version 3
rewrites the same sections after a second round of the nine facets on
version 2 returned 42 new blocking findings, most of them contract
completeness (sink order, seed shape, the ablation primitive, a closed
container API, instruments for every reading). The findings, the decisions
D1–D30 taken from them and every disposition are in
[`docs/progress/readings/t25-design-review.md`](../progress/readings/t25-design-review.md).
Where a version reverses an earlier disposition it says so.

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
  shared across a group is one pattern-weighted change to the shared scale.
  The plasticity update cost is 0, and the cost machinery goes (D9);
  learning attributes carry no genome unit.
- Group creation comes in two forms: every member alike, or learning
  attributes randomized per member. The geometric pattern is never
  randomized.
- The three nearby-creature families merge into one `NearbyCreature`
  family. An edge costs one genome unit wherever it lives; the per-wired-sink
  unit is dropped, the founder anchor re-pinned (97 → 80) and the per-unit
  mutation rate set so the founder's supply is unchanged (option a).

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
   pattern-weighted update to the group's learned scale. All runtime state
   is built once at creature construction from the genome and the birth
   seeds; nothing is initialized lazily.
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
seeds (behavior-preserving); F02 uniform genome size, the rate and the cost
chain; F03 ring groups replace projections; F04 direction-pair groups; F05
the `NearbyCreature` family; F06 learning per edge; F07 group-scope
learning; F08 creation forms, detach and the catalog. §8 holds the contracts.

This recommendation is provisional against the repository's research depth
bar: the local evidence is complete (three code maps, a live probe, three
Codex rounds and two rounds of nine facet reviews), the external precedent rests on one
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
`edge`, `group` and `container` submodules), runtime state in
`runtime/cgp` and `runtime/plasticity`, reproduction in
`simulation/actions/cgp_reproduction.rs`. No type or module is renamed
(D11); `mutation/graph/projection.rs` is deleted and `mutation/graph/group.rs`
is a new module.

```rust
/// A heritable connection. Stays in its container's `inputs` list. Every
/// field is private: readers use the getters, writers go through the
/// container (§7). `Copy`; `size_of` pinned at 16 (F03) and 36 (F06).
pub struct GraphEdge {
    source: GraphSource,
    /// Inside a group a correction on top of the group term, born 0.
    /// JSON key `own_weight`, no alias.
    own_weight: f32,
    /// Today's type, moved from the compute node (F06). `None` is static.
    /// Omitted from JSON when `None`.
    plasticity: Option<PlasticityConfig>,
    /// Set only on sink edges that fit a group's shape (F03; `scope` in
    /// F07). Omitted from JSON when `None`.
    membership: Option<Membership>,
}
impl GraphEdge {
    pub fn new(source: GraphSource, own_weight: f32) -> Self;   // static, ungrouped
    pub fn source(&self) -> GraphSource;
    pub fn own_weight(&self) -> f32;
    pub fn plasticity(&self) -> Option<&PlasticityConfig>;
    pub fn membership(&self) -> Option<Membership>;
}

pub struct Membership { pub group: GroupId, pub scope: Scope }
pub enum Scope { Edge, Group }
pub struct GroupId(u16);

/// Private fields; written only through `GroupMut` and the group module.
pub struct Group {
    shape: Shape,
    kind: DirectedVoteKind,
    /// Rotation, always `< 8`.
    offset: u8,
    /// One shared weight, born 0.
    scale: f32,
    /// Whether a learned scale seeds the child's runtime state (F07).
    inherit_scale: bool,
}

pub enum Shape {
    /// Members: ring cell `c` of `input_refs[ring_ref]` into `K[(c + offset) mod 8]`; pattern 1.
    Ring { ring_ref: u16 },
    /// Members: the pair's x or y field of `input_refs[ref_idx]` into any
    /// direction sink of `K`; pattern is the compass projection (§4) rotated
    /// by `offset`. `pair` indexes the family's declared pairs (§4).
    DirectionPair { ref_idx: u16, pair: u8 },
}

/// Edge addressing, defined here and used by mutation draws, the runtime's
/// `(sink, edge)` order, the learned-state iterator and the server payload.
pub struct EdgeOccurrence { pub surface: EdgeSurface, pub index: u16 }

/// The three edge containers are newtypes over `Vec` with `Deref<Target = [T]>`
/// and no `DerefMut`, `#[serde(transparent)]`, `Debug` delegating to the
/// inner `Vec`: every reader compiles unchanged, only writers move.

/// Pure composition, the only formula.
fn compose(own: f32, scale: f32, pattern: f32) -> f32 { scale * pattern + own }

impl CgpGraphBackendDef {
    /// Genomic weight of the edge at `occurrence`. A member's pattern comes
    /// from its group's shape reference, its own `sub_idx` and its target
    /// direction, never from its own `ref_idx`, and is 0 when the shape
    /// reference does not resolve.
    pub fn genomic_weight(&self, occurrence: EdgeOccurrence, input_refs: &[InputReference]) -> f32;
}
```

**Resolved weights live in runtime state.** From F03, `GraphRuntimeState::for_genome(&CreatureGenome, Option<BirthSeeds>)`
replaces `GraphRuntimeState::new()` as the only constructor. It builds, once
at creature construction, a table of one `f32` per sink edge filled through
`genomic_weight`, which the sink loop reads; F06 adds the learning layout
and `LearnedState` to the same struct, and `applied_weight(occurrence)`
falls back to that table. Every site that builds runtime state outside
reproduction (`neighborhood/battery.rs:334, :432`,
`neighborhood/opportunity/discovery.rs:297`, `CreatureState::new*`
(`creature/state.rs:338, :404`), seeding and the lab's `eval.rs:151`) goes
through `for_genome`; observation clones (`simulation/tick.rs:407, :475`)
carry the state.

**Birth seeds (D1).** Birth correspondence stays a draft-carried payload, as
`birth_weights` is today: `BirthSeeds { compute rows, sink rows (F06), one
slot per group (F07) }`, a `#[serde(skip)]` private field of
`CgpGraphBackendDef`, the only such field on any genome type, that exists on
the child draft between reproduction's capture and the runtime
construction. It rolls back with the engine's genome snapshot, travels with
topology clones, and is realigned by the container primitives and nowhere
else. The one hand-written `Debug`/`PartialEq` pair on the def skips exactly
this field and prints `groups` unconditionally; every other genome type
derives, and an exhaustive destructuring test makes a new field a compile
error. This reverses the version 1 edit-record design (Codex dispositions
1, 27, 28, 34, 35): the code already gives the payload every property
records would have rebuilt.

**Invariants (closed list).** The container primitives assert the
def-local ones (I1–I3, I5, I6, I11, the Ring coordinate of I4 and
`offset < 8`) at every exit; the group module's operations, which take
`input_refs`, assert the reference-dependent ones (I4's pair role, I7, I8);
`validate_graph(&NodeGenome)`, defined in the container module beside this
list, checks all of them at the one genome deserialization boundary
(`GenomeFile::load`) and in the parseability gate after every event under
its own skip reason and counter (D5); an engine-level proptest (random
grouped genomes × random events over all four domains) asserts
`validate_graph` is `Ok` after every applied event and that the counter
stays 0.

- I1 Every own weight, scale and plasticity parameter is finite; rates and
  decays in `[0, 1]`, clamps in `[0.01, 10]`, as the mutation laws bound them.
  (Other genome floats keep today's tolerance; the round-trip generator is
  restricted to finite values.)
- I2 Only a sink edge carries a membership; compute-input edges never do.
- I3 Every `GroupId` is below `groups.len()`; `groups.len() ≤ u16::MAX`.
- I4 A member fits its group's shape: `Ring`, source `InputLeaf { ring_ref, c }`
  with `c < 8` into sink `K[(c + offset) mod 8]`; `DirectionPair`, source the
  pair's x or y field into a direction sink of `K`.
- I5 At most one member per `(group, source, target)` coordinate.
- I6 A group has at least one member; removing the last member removes the
  group.
- I7 A shape is well-formed: `input_refs[ring_ref]` is a ring family; `pair`
  is below the declared pairs of `input_refs[ref_idx]`'s family; `offset < 8`;
  a shape reference index is at most `u16::MAX`.
- I8 A member's `sub_idx` is canonical (below its compound width); world
  compounds wrap today, so `(ring, 8)` would read cell 0.
- I9 Identity: `Debug`, `PartialEq` and JSON cover every heritable field and
  nothing else; `BirthSeeds` is the only `#[serde(skip)]` field; genome
  types carry `deny_unknown_fields`.
- I10 Sink order: within each sink, independent edges come first in
  insertion order, then members sorted by `(GroupId, channel)`. `push_edge`
  inserts an independent edge before the sink's first member; detach moves
  the detached edge to the end of the independent prefix; relocation
  re-sorts members. Sink sums are sequential, so this order is both identity
  and arithmetic. (Facets 4, 5 and 7 found the same gap.)
- I11 Seed shape: when `BirthSeeds` is present it has one row per compute
  node and one per sink, each sized to that container's `inputs`, plus one
  slot per group.

Decisions behind the model:

- **Edges stay in their containers.** A global edge list with identifiers was
  considered (§6, option D): input order is semantic, the trace payload is
  ordered per target, every reader walks a container. A `GroupId` on the
  edge gives explicit membership without identifiers.
- **Membership is a validated coordinate** (I4), not a stored position; the
  pattern is recomputed from the shape reference, `sub_idx` and target
  direction, never stored. An operation that changes a member's source
  outside a shape operator (retarget to a different source, raw-field step,
  split through a compute node) detaches it first.
- **Members are materialized.** A `Ring` group is eight member edges plus one
  record. That is what lets a mutation delete, re-weight or detach one
  member while the group survives, the HybrID pattern (§5). The hot path
  loses the projection term and reads one table entry per sink edge.
- **Pattern zeros are not materialized at birth.** A pair member for `x` has
  pattern 0 toward N and S, so creation makes 12 members, not 16; an omitted
  member is geometrically unnecessary at birth, not impossible, and a later
  `AddGraphEdge` on that sink is an ordinary edge, never a member (F08
  confirms the no-join rule).
- **Learned state is runtime state**, constructed once from seeds at birth
  (D8) inside `for_genome`: a learned own weight for every learning edge
  with scope `Edge`, a trace for every reward-modulated edge, and one learned
  scale per group with a scope-`Group` learner. There is no lazy allocation.
  A per-module predicate `initialized(slot) = module visited || slot seeded`,
  with the visited flag set at visit entry before the pass-cost check and
  kept for life, gates phase 2.5, `capture()` and the census iterator's
  `learned: Option<f32>`, which keeps today's "initialized rows only"
  semantics for all three readers; F06 predeclares the one intended change
  (a genuinely learned seed is clamped by the child's clamp). The genome
  never absorbs learned values.
- **Clamp rule.** Every learning write bounds the quantity it stores: an own
  weight by its edge's clamp, a scale by the group bound (the smallest clamp
  among its scope-`Group` learners). The applied weight is never clamped.
  At-clamp counts, seed clamps and detach seeds use that same quantity and
  bound.
- **Detach** is a group-module operation `detach(def, occurrence, input_refs)`
  calling the container's `detach_at(occurrence, pattern)`: genomically,
  `own := compose(own, scale, pattern)`, `membership := None`; and whenever
  the edge or the group has a seed, `seed := compose(edge seed or own, group
  seed or scale, pattern)` clamped by the edge's clamp, with no inheritance
  flag read (only `LearnedState::from_seeds` reads the child's final flags).
  `RemoveGroup` removes the record and every member; `RemoveGraphEdge` on
  the last member removes the group (I6).
- **Accounting contract.** `genome_size` = mesh nodes + input references +
  route targets + compute nodes + edges on both surfaces + group records
  (VM: program length + constants); the per-wired-sink unit goes. **F02
  charges a surviving `RingProjection` nine units** (eight implied members
  and the record), so F02 carries the whole size change under its goal-long
  and F03's replacement of projections by groups is size-neutral, pinned
  (facet 8 over facets 1 and 4: nothing evolves for free during a 40,000-tick
  run). The founder is 97 → **80** (node 0: 20, node 1: 60; ForageFirst
  profiles 100 → 83), the anchor `FOUNDER_GENOME_SIZE_UNITS` re-pinned so the
  founder's replication multiplier stays 1.0 and its gestation 1 tick; the
  founder's carrying cost falls 0.0097 → 0.0080 per tick. For a creature with
  `P` projections and `S` wired sinks, `Δgenome_size = 8P − S`. The per-unit
  mutation rate becomes `0.485 / 80 = 0.0060625` (option a, user
  2026-10-13), bit-equal under the founder supply pin, so the founder's
  0.485 events per birth and its clone fraction are unchanged; an evolved
  genome of `x` old units draws `1.2125 (x − S + 8P) / x` of its old events,
  above 1.21 wherever `8P > S`. Readers that move: carrying cost (`1e-4` per
  unit per tick), supply, the replication multiplier, gestation
  (`ceil(size × rate / 80)`, ×1.21 at depth), the size-pressure roll against
  the 1,200 cap (off by default, not re-anchored), and the drift-walk and
  recruitment-paths instruments that draw `Binomial(anchor, rate)` (their
  means stay 0.485; their draw streams change, because one `u64` is drawn
  per unit at any rate). Functional complexity keeps its rules: a wired sink
  counts one whether its edges are members or not, sink edges are not
  counted individually, a group record counts one, and a consumed reference
  counts one through edges only (the projection reference chain goes and no
  shape chain replaces it); a ring group onto previously unwired sinks
  raises complexity by up to 8, which the action-cost multiplier, the
  reproduce charge and the predation kill bonus read, all off at defaults.
  Learning attributes, scope and `inherit_scale` carry no genome unit and no
  update cost (user, 2026-10-13).

## 4. Mechanisms

**Compass pattern.** The table is derived from `Direction::delta()`
(`contracts/direction.rs:31-38`), not typed by hand: for direction `d`,
`px[d] = normalized dx`, `py[d] = px[(d − 2) mod 8]`, with the diagonals at
`std::f32::consts::FRAC_1_SQRT_2` so every row has norm 1 within 1 ulp and
the axis zeros are exactly `0.0`. A pair member for `x` into `K[d]` has
pattern `px[(d − offset) mod 8]`; the `y` member `py[...]`. With a positive
scale the group votes toward the sensed position; offset 4 votes away; the
sign of the scale is evolved. The applied weight is two rounded operations
(no `mul_add`) and sums stay sequential, so results are platform-stable. The
test that pins the meaning, per pair-declaring family: a thing at
`Direction::delta(d)` with scale 1 and offset 0 makes `K[d]` the argmax, and
offset 4 makes `K[d + 4]` the argmax.

**Pair metadata lives in `contracts::inputs`** beside `compound_width` (D12):
each world family declares its width, its field offsets and its direction
pairs (`NearbyCreature` slot k `(dx, dy)`; `AreaOccupancySummary`
`(centerX, centerY)` and `(nearestDx, nearestDy)`; `AreaFoodSummary(t)`
`(gradX, gradY)` and `(nearestDx, nearestDy)`; `AreaBarrierSummary` the same
two). `sensors::reducers` writes against it and `PerceptionSnapshot`'s
widths derive from it; the shape validates against it; labels, census and
the lab read it; the frontend's `inputRefUtils.ts` is a hand mirror pinned
by a drift test. Shape eligibility depends on the `WorldInputKey` variant
only, never on a food type, so `InputRef.Swap`'s kind partition preserves
it (a test checks every swap alternative of a ring or pair-declaring
reference keeps the variant).

**Learning on edges (F06, F07).** The update laws are today's
(`runtime/plasticity/hebbian.rs:138-145`, `reward.rs:27-88`) with the edge's
own configuration. Contract:

- *Layout at birth.* `for_genome` builds a per-backend learning layout (a
  `has_learning` flag, the slot list, the learned-state allocation) beside
  the genomic weight table; `execute_graph_impl` and `apply_cgp_graph_effects`
  are monomorphized on a learner parameter passed explicitly (the tracer
  never reaches effects), so a non-learning graph runs today's loops with no
  sample branch; phases 0 and 2.5 iterate the slot list, never the genome;
  sample and weight buffers are runtime-state scratch, and effects' per-visit
  allocations go.
- *Samples.* The executor records, for every learning edge, the pre value
  the evaluation used (compute inputs: the ordered read; sink inputs: the
  value the sink sum read, in sink order) and the post value (the node's
  committed output; the sink's `sanitize_f32(wsum)`). Effects write shared
  memory as the sink loop proceeds, so a later re-resolve would differ.
  Recording a sample costs 4 B per learning edge; the two `action_queue`
  clones the replays needed go. On recurrent compute edges this changes
  pure-Hebbian semantics (today's replay reads this visit's output for self
  and higher sources): predeclared, with a self-edge regression.
- *Order.* Evaluation reads the applied weight in force at visit start;
  every Δ, Oja's term included, is computed from visit-start values after
  effects; then contributions apply in canonical `(sink, edge)` order with
  their clamps. The cost chain is deleted (D9): no visit is voided by
  learning, and the zero-energy floor that today sits inside the reward-cost
  block (`tick.rs:1329-1332`) becomes its own step keyed on "has a
  reward-modulated edge", pinned by a test.
- *Scope and credit (D4).* An "event" is one committed visit (Hebbian) or
  one phase-2.5 pass per backend per tick. A member with scope `Edge`
  updates its learned own weight. A member with scope `Group` contributes
  `p_m × Δ_m` to the group's learned scale, its own rule, rate, source and
  decay applied, so a negative-pattern member never moves against its rule
  and `Ring` members (pattern 1) contribute Δ unchanged. Contributions in
  one event sum in canonical order, and the scale is bounded once by the
  group bound. A static member contributes nothing. One change to the shared
  scale is what "every member equally" means: every member's applied weight
  moves in proportion to its pattern.
- *Eligibility.* Every wired sink edge is a learning candidate except
  `ClearSlot` edges, whose sink ignores its sum. Members are excluded from
  the plasticity operators' predicates in F06 and admitted in F07 together
  with scope (so no half-state in which a member learns without a scope or
  seeds). Vote edges may run pure Hebbian learning (runaway allowed by
  decision, clamp-bounded); per-visit sampling is the T19.F02 rule, so
  credit scales with visits, which F06 reads beside clamp occupancy.
- *Seeds.* A Lamarckian edge's learned own weight seeds the child; a group
  seeds its learned scale only when `inherit_scale` is set and the child
  group has a scope-`Group` learner; every seed is clamped at construction
  by the child's bound; the container's `set_weight`, `set_scale` and a
  source-changing `set_source` clear the seed they touch, as a weight step
  does today, and a same-source retarget or an unchanged raw-field move
  keeps it.

**Shape operators relocate members in two phases.** `ShiftGroupOffset` and
`RetargetGroupKind` (group-module operations over the container's
`relocate_group`) remove every member from its sink, then re-insert each at
its new sink in the I10 position, so a shift followed by its inverse returns
an equal genome (a proptest). The pattern travels with the coordinate;
correction, plasticity, scope, learned state and seeds travel with the
edge; holes stay holes; nothing is regenerated. F03 predeclares the
summation-order change on sinks carrying members.

**Removal.** `RemoveGroup` removes the record and every member and renumbers
every higher `GroupId` in one container operation; `RemoveGraphEdge` on a
last member does the same. Reference reindexing never removes a group: the
only production pruner removes references no consumer addresses, and every
live group's members address its reference, so the reindex is decrement-only
(shape references decrement with the edges) with a debug assertion (D6).

**Edge operators on members.** `RetargetGraphEdge` keeps today's draws (edge,
then source); a same-source draw changes nothing; a changed source detaches
first (through the group module), then clears the seed. A raw-field step
always changes the source, so it always detaches. A split (`split_edge`)
detaches before rewiring. `AddGraphEdge` never joins a group (F08 confirms).
Copy operators copy compute-input edges only, through `copy_edges`, which
carries seeds. Refinement skips members in every key.

**Recruitment emits groups.** `RecruitNeutralInput` draws the family, then
the destination, then, for every `WholeFamily` `NearbyCreature` recruitment
whatever the destination, one slot (`gen_range(0..4)`), and for an
area-summary family on a directed destination one pair (`gen_range(0..2)`).
A ring family on a directed sink emits, under both arms, a `Ring` group of
eight born-0 members with `offset = (d − channel) mod 8` under
`SingleChannel` and `offset = d` under `WholeFamily`, as today's projection
cases do. A pair-declaring family on a directed sink under `WholeFamily`
emits a `DirectionPair` group of twelve born-0 members with `offset = d`
plus one born-0 edge into `Drive K` per field of the drawn slot in no
eligible pair; `SingleChannel` keeps today's single-edge plan; other
destinations keep today's whole-family plain-edge plan with one slot.
Recruitment always creates static groups. `NearbyCreature` is weighted
three times in recruitment's family universe, mirroring D13 (facet 7: the
sampler and the recruitment universe are separate draws). Group creation
by any route requires the 24 direction sinks, `Drive K`, and group
capacity.

**Ablation (D3).** Members are plain sink edges, so today's rewrite ablation
in `neighborhood/input_use/consumers.rs` and the lab's `readings::ablate`
covers them; their projection branches, `ALL_RING_CHANNELS` and
`ablated_channels` are deleted. Because the containers are private and
`set_source` detaches, F01 adds one observation-only container primitive,
`redirect_leaf_reads(f: FnMut(ref_idx, sub_idx) -> Option<u16>)`, which
rewrites `InputLeaf.ref_idx` on a copy with no detach, no seed change and no
assertion; it is the only sanctioned I4 exemption, its four callers are
named in F01 (`input_use/mod.rs:249, :271`, `opportunity/assay.rs:267`, the
lab's `ablate`), and a copy never re-enters mutation or validation. A
member's genomic weight survives the rewrite bitwise because its pattern
resolves from the group's shape reference, not its own `ref_idx`, and
complexity counts references through edges only, so the lab's sentinel
scheme stays one-for-one. The lab's `expressed()` stays a lab-side rewrite
that sets every edge's plasticity to `None`.

**Draw contracts.** Every operator documents its draw order in code and
pins it with a parallel-RNG tail test, as today.

- *Group operators* (F03): `AddRingGroup` draws ring, kind, offset as
  `AddRingProjection` does today; `AlterGroupScale` picks a group then steps
  with `perturb_weight`; `ShiftGroupOffset`, `RetargetGroupKind` and
  `RemoveGroup` pick a group and apply. `AddDirectionPairGroup` (F04)
  enumerates `(ref_idx ascending, pair ascending)` over references whose
  family declares pairs, draws one, then kind, then offset.
- *Plasticity operators* (F06): the nine node operators move to edges with
  their laws unchanged (`hebbian.rs:195-349`): enable, disable, rule, rate,
  Lamarckian flag, enable and disable modulation, source, decay; the edge is
  picked uniformly over both surfaces in canonical order among edges
  matching the operator's predicate; a `ClearSlot` edge never matches; a
  member never matches until F07.
- *Scope coin* (F07): after picking the edge, if it is a member,
  `gen_bool(0.5)` chooses this edge or every member; a group-scope
  application touches exactly the members the operator's own predicate
  admits and leaves the rest unchanged with no draw; the drawn value is
  drawn once (a parameter step draws one `u` in `[−1, 1]` and each member
  applies its own branch of today's law; `ToggleHebbianLamarckian` writes
  `!picked.lamarckian` to every plastic member); a non-member draws no coin.
  `SetScope` picks a member, draws the coin, writes the other scope; no
  value draw. One tail test per case.
- *Static creation* (F03, F04): shape draws only; members static and
  ungrouped-equivalent in every attribute they have at that feature; scale
  and corrections 0.
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
mechanisms for one concept. Refinement's `GroupKey`, `groups()`,
`has_group()`, `Group` and `Member` become `SlotKey`, `inferred_sets()`,
`has_inferred_set()`, `InferredSet` and `InferredMember`, and its weight
writes go through the container setter so they clear seeds like every other
weight step. Refinement never steps a member. The `Slots` key is re-keyed on
the merged family in F05 (`field = channel mod 9`, `slot = channel div 9`)
and kept until F08 decides, from refinement applications recorded by key on
post-F05 runs, whether it stays or goes; "goes" deletes
`RefineHeritableStructure`, `structured_heritable_refinement` with its
config-digest undo step, the inferred-set code and the spec rows; a `Slots`
shape would be a later row, not an F08 outcome.

**The creature-sensor family (F05).** The three families merge into one
`NearbyCreature` family (4 slots × 9 fields = 36 sub-values) declared in
`contracts::inputs`. The sampler keeps 27 buckets with indices 10–12 all
drawing `NearbyCreature` (D13): the family is 36 wide against 7 or 8 for
others, so three buckets is proportionate, and the draw pins hold.
Recruitment takes one slot everywhere, which keeps the 30-edge bound.

**Non-goals (D11).** The VM backend (no edges); `Drive` sinks as group
targets; compute-input edges as members; founder rewiring; type and module
renames; generating the operator enums from a table (a bijection test that
also checks `complexity_effect` agreement is added instead); the four domain
selection loops; the dangling `ComputeNode(u16::MAX)` sentinel; tolerance of
out-of-range `InputLeaf` reads and recruitment's `append_end`; whole-family
plain-edge bundles into non-directed sinks (the mechanism §1 found inert);
`BirthEdgeClass` as a second "which edge was created" channel; the existing
`simulation::energy_accounting` imports in `runtime` and `creature::state`
(the dependency rule of §7 binds T25's new modules only); the unbounded
steal amount (program 1 carried finding 5).

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
| `creature::genome::cgp::edge` | `GraphEdge` with private fields and getters, `GraphEdge::new`, `Membership`, `Scope`; field-level validation (I1) | `GraphEdge` with public fields |
| `creature::genome::cgp::group` | `Group`, `Shape`, `GroupId`, the compass pattern from `Direction::delta()`, `compose`, the reference-dependent checks (I4 pair role, I7, I8); the shape operations `shift`, `retarget` and `detach(def, occurrence, input_refs)` over the container's `relocate_group` and `detach_at` | `RingProjection`; inference in `refinement.rs`, `ring_rules.rs`, `drives.rs` |
| `creature::genome::cgp::container` | The closed API over the three private `Vec` newtypes and `groups`. Reads returning shared references: `compute_nodes()`, `output_sinks()`, `groups()`, `edges()`, `edge_at(EdgeOccurrence)`, `members(GroupId)`, `genomic_weight`. Writes: `push_edge` (I10 position), `remove_edge`, `retain_edges`, `copy_edges(from, to)` carrying seeds, `split_edge(occurrence, identity_node)`, compute-node insert, append (no source remap), remove and duplicate, `set_compute_kind`, reference reindex (decrement-only), `add_group`, `remove_group`, `relocate_group(id, kind, offset)`, `detach_at(occurrence, pattern)`, `EdgeMut::{set_weight, set_source, set_plasticity, set_scope}` (`set_weight` and a source-changing `set_source` clear the seed; `set_source` debug-asserts non-membership), `GroupMut::{set_scale, set_inherit_scale}` (`set_scale` clears the group seed), `redirect_leaf_reads` (observation copies only). Defines `EdgeOccurrence` and the canonical enumeration, `validate_graph`, and the invariant list; keeps `BirthSeeds` aligned (I11); never hands out `&mut GraphEdge`; no RNG | ~12 hand-maintained alignment sites in `operators.rs` and `cgp.rs`; `pub` fields written from `input_ref/mod.rs`, `topology/routing.rs`, `refinement.rs`, the ablation paths, the founder, instruments and the lab |
| `contracts::inputs` | Family widths, field offsets and direction pairs: one layout both backends, the shape, labels, census, lab and (as a pinned mirror) the frontend read | widths in `contracts/inputs.rs`, offsets in `sensors/perception.rs` |
| `simulation::actions::cgp_reproduction` | Moves `Option<BirthSeeds>`: `LearnedState::capture()` before mutation (returns `None` per node when nothing was learned, the fast path), construction through `CreatureState::new*` after; writes no runtime field itself | `capture_birth_weights`, `build_cgp_child_plasticity_weights`, a direct `graph_runtime` write at `reproduction.rs:567` |
| `runtime::cgp` (`GraphRuntimeState::for_genome`) | The only runtime-state constructor: the genomic weight table (F03), the learning layout and `LearnedState::from_seeds` (F06); the learner parameter for `execute_graph_impl` and `apply_cgp_graph_effects` | `GraphRuntimeState::new()` plus lazy init in two places |
| `runtime::plasticity` | Learned own weights, learned scales and traces over both surfaces; samples; Hebbian and reward updates; scope routing; the clamp rule; `applied_weight(occurrence)` (the only resolver); a `pub` read-only view yielding edge items `(occurrence, genomic, learned: Option, bound, mode, scope)` and, from F07, group items `(node, GroupId, genomic scale, learned scale, bound)`; storage `pub(crate)` | three fallbacks, two evaluation paths, lazy init, `pub` storage read by the census and v3-telemetry |
| `runtime::cgp::effects` | Sink sums over the weight table; the learner records samples; no projection term; no per-visit allocation | `effects.rs:57-99, 117-118, 133-171` |
| `mutation::graph` | Edge and group operators (`group.rs` new, `projection.rs` deleted), the scope-coin helper, the nine plasticity operators on edges; every structural change through the container; a bijection test over the operator enums that also asserts `complexity_effect` agreement | `operators.rs`, `hebbian.rs`, `projection.rs`, `refinement.rs`, `recruitment.rs` |
| `mutation::engine` | One generic snapshot, validate, commit-or-rollback wrapper (F01) | four near-identical wrappers |
| `analysis` and `census` | Explicit membership from the genome (plasticity carriers and scales through the runtime view; sensor census "wired" through members); derived wiring analyses over non-member edges and genomic weights, with ring groups and pair groups reported in their own fields and blocks | inference in three files; `ablated_channels` |
| v3-lab | Rewrite ablation through `redirect_leaf_reads`, its own sentinel scheme kept; `expressed()` a lab rewrite; the comparator stays a lab solution built through `GraphEdge::new` | projection special cases; the hidden field |
| Transport and inspector | Serde shape; the `graph_weights` payload (below); `mesh_annotations` required with the genome and the frontend's inference deleted; an edge-group list; a per-edge plasticity badge; sink `weighted_inputs` in the trace | raw `edge.weight` drawn; two sources of truth in `meshSemantics.ts`; no plasticity display |

**Vocabulary.** One stem, "plastic", for every new name, matching
`PlasticityConfig`, `runtime::plasticity` and today's telemetry.

| Concept | Code | Operator keys | Telemetry | Inspector |
| --- | --- | --- | --- | --- |
| Group record | `Group`, `GroupId` | `Graph.AddRingGroup`, `Graph.AlterGroupScale`, `Graph.ShiftGroupOffset`, `Graph.RetargetGroupKind`, `Graph.RemoveGroup`, `Graph.AddDirectionPairGroup`, `Graph.AddRandomizedRingGroup`, `Graph.AddRandomizedDirectionPairGroup` | `petri.run.groups_removed`, `ring_rules.ring_group_*`, `pair_groups` block | "Edge groups" list; member edges dashed in the `group_member` tint |
| Membership | `Membership`, `Scope` | `Graph.DetachGraphEdge`, `Graph.SetScope` | `petri.run.graph_edges_detached`; `petri.tick.plastic_edges_by_scope` by `petri.plasticity_scope` | member row: target, pattern, own, genomic, applied |
| Edge learning | `GraphEdge::plasticity` | the nine existing `Graph.*Hebbian*` and `Graph.*RewardModulation*` keys, unchanged | `petri.tick.plastic_edges` by (`petri.plasticity_learning`, `petri.surface`); `plastic_edges_by_rule`; `plastic_edges_by_inheritance`; `plastic_edge_drift` and `plastic_edges_at_clamp` by `petri.surface` | plasticity badge |
| Learned scale | `LearnedState` scale slots, `Group::inherit_scale` | — | `petri.tick.learned_scales`, `learned_scale_drift`, `learned_scales_at_bound` by `petri.plasticity_inheritance` | group row: genomic scale, learned scale, `inherit_scale` |

Each family stays within the two-attribute rule and is added to
`always_names`; the panels that group by `(learning, inheritance)` today are
redrawn on the new families.

**The `graph_weights` payload** (F03, extended in F06 and F07). A top-level
field beside `genome` in `CreatureDetailResponse`, computed independently of
`include_genome`, excluded by its own token `graph_weights` (added to the
protocol's §4.11 list and the inspector's polling pattern), omitted when the
creature has neither members nor learning edges, absent never zero:
`[{node_id, scales: [{group, genomic, learned?}], edges: [{surface:
"compute" | "sink", container, edge, genomic, applied?, learned?}]}]` in
genome order, keyed to `graphFullModel.ts`'s edge ids. F03 ships genomic
values stored beside the genome; F06 adds `applied` and `learned` per edge
and keeps the field in every poll, replaced in a live slice; F07 adds
`learned` per scale. The inspector draws a member at its `genomic` value,
never at its own weight; a member-only node is an action writer through the
server's annotations, which are required whenever the genome is present.

**Dependency direction** (for T25's new modules: `edge`, `group`,
`container`, the `contracts::inputs` layout, `for_genome`, learned state):
`contracts` ← `creature::genome` ← `runtime` ← `simulation`; `sensors` reads
`contracts` and `creature`; `creature::genome` never imports `sensors` or
`runtime`. v3-lab, v3-telemetry, v3-cli and the server read v3-core through
the runtime view and the composition functions, never through storage. The
existing `energy_accounting` imports are a non-goal (§4).

**Validation homes (D5).** As §3 states: def-local invariants at container
exits, reference-dependent ones in the group module, `validate_graph` at
`GenomeFile::load` and in the gate under its own skip reason, and the
engine-level proptest.

**Determinism and pins.** F01 moves no pin; any movement under it is a
defect. The absolute mutation-off oracle is a carrying-cost-0 sibling inside
`viability.rs`'s `founder_only_trajectory`, hashing positions, energy bits,
age, generation and food only (never genome text or learned state). F02
moves both siblings once, by construction (the unit count changes the
number of Bernoulli draws per birth at any rate, and the carrying cost
changes energy bits); the restoring check is "anchor 97 restores them".
Nothing after F02 may move them. Pins that hash genome text
(`baseline_worlds.rs:142`, `recruitment_tests.rs:49`,
`applied_trajectory.rs`) move at F03 and F06 for text reasons; those
features land a text-only commit first, re-pinned with "the hash without
genome text is unchanged", before their behavior commit. Every re-pin names
its mechanism and a restoring check, one cause per commit. When a feature
changes a proptest's strategy, it deletes that test's lines from
`proptest-regressions/` and commits any new file.

## 8. Contracts (placed 2026-10-13; the track rows are authoritative)

Order: F01 → F02 → F03 → F04 → F05 → F06 → F07 → F08, a linear chain (D10).
Every feature: `make check` green at its end, the mutation gate and the slow
tests run, a "Predeclared re-pins" line, a "Reference specs" line, and a
"Tests first" list the implementer writes red before code. Operator lines
give the catalog after the feature as `ALL / default arm / [Self; N]`; the
spec owner expands each into the mutation spec's table format.

- **T25.F01 — Edge Container, Constructors and Birth Seeds.**
  - Goal: Development separated from heredity, as a cell's machinery for
    building synapses is separate from what the egg carries: one owner edits
    the graph, one captures what a child is born with, and one builds the
    running brain from it, with no behavior change.
  - Scope: the container module with the closed API of §7 over the three
    `Vec` newtypes, restricted to today's write sites (group and plasticity
    primitives arrive with their features); `GraphEdge` fields private with
    getters, `GraphEdge::new`, `ComputeNode::new` and a def builder, with
    every struct literal (256 `GraphEdge {`, 183 `ComputeNode {`, 70 def
    literals) and the ~15 local test helpers migrated across every crate and
    the frontend fixtures; `birth_weights` renamed `BirthSeeds` as a private
    container-kept field with I11 asserted; `LearnedState::capture()` and
    construction through `CreatureState::new*`; `redirect_leaf_reads` with
    its four callers; the one generic engine wrapper; `clamp_sub_idx_after_swap`
    and the dead reindex branches deleted; `connect_declared_input`, routing
    and refinement writing through the container; the drift-walk dating test
    rewritten to a constructed lineage. `BirthEdgeClass` stays (non-goal).
  - Verification: gate; mutation gate run; slow tests run. No pin moves.
  - Predeclared re-pins: none.
  - Tests first: the golden Lamarckian corpus recorded on main before any
    change, as a v3-core unit test that asserts its own coverage (at least
    one payload-carrying application per alignment site and per topology
    clone or swap, at least one rollback, one all-empty parent) and hashes
    each child genome and its built weights, then retired in F01's closing
    commit; `engine/tests.rs:1757` ported; the decrement-only reindex
    assertion; the constructed-lineage drift test; the carrying-cost-0
    sibling of `founder_only_trajectory`.
  - Oracles: the corpus; both `founder_only_trajectory` siblings; the
    inheritance tests ported scenario by scenario with identical expected
    values (`reproduction.rs:1012` as an explicit backend-replacement test,
    same-source retargets included); the existing absolute pins
    (`applied_trajectory`, `baseline_worlds`, recruitment hash, drift, slow
    tier) unchanged.
  - Reference specs: `v3-reproduction-spec.md` Lamarckian section (payload
    name).
- **T25.F02 — Uniform Genome Size.**
  - Goal: Every synapse costs the same to carry wherever it sits, as a
    neuron pays per synapse and not per target: an edge is one unit on both
    surfaces, a wired sink costs nothing of itself, a ring gene costs its
    eight synapses, and the founder is re-anchored.
  - Depends on: T25.F01.
  - Scope, in three commits: (1) the unit rule (§3), a `RingProjection` at
    nine units, `FOUNDER_GENOME_SIZE_UNITS` 97 → 80 and the drift-walk and
    recruitment-paths instrument constant; (2) the per-unit rate default
    `0.0060625`, with the frontend's `MutationSection.tsx` default and the
    committed world files that pin 0.005; (3) the plasticity cost chain
    deleted (D9: both knobs and the `hebbian_update_cost` alias, both
    `DeathCause`s, the energy-flow fields, bench, telemetry, frontend config
    and dashboard fields, the split exclusion, the zero-energy floor kept as
    its own step), inert at the default 0. Also `mean_wired_sinks` and
    `mean_projections` on `PersistenceSample` and the keep-list; the
    recruitment-paths, recruitment-transitions and neighborhood-read tokens
    bumped; a `genome_size` definition-change entry in `benchmark-series.json`;
    the T20.F02 storage-contract table and test; the ~20 doc and code sites
    citing 97 or 0.005.
  - Verification: `cargo test -p v3-core --test viability` first; gate +
    goal + goal-long (the size question is a long-run one); one extra
    goal-long arm on the F02 build at rate 0.005, so the size rule and the
    rate each have a counterfactual; mutation gate run; slow tests run.
  - Predeclared re-pins, each with mechanism and restoring check: both
    `founder_only_trajectory` siblings (`viability.rs:1113`; draw count and
    carry; anchor 97 restores); the four `applied_trajectory.rs` digests,
    `baseline_worlds.rs:142-145`, `recruitment_tests.rs:49`,
    `reproducibility.rs:47` and the slow-tier recruitment-paths rows (draw
    count; anchor 97 restores); the founder size pin (`state.rs:808-815`,
    `phase0.rs:228`, `cli.rs:223`), `reproduction.rs:682-683` and `:748`
    (unit rule); `projection_tests.rs:73-81` (nine units); the lifecycle
    storage test; `config/simulation.rs:1460` and `:871-875`, `server.rs:3818`,
    the `supply_rule` strings (`drift.rs:813-819`, `bench/run/tests.rs:223`),
    `bench/mutation_effects/tests.rs:75`, `ConfigPanel.test.tsx:69`,
    `fixtures.ts:102` (rate; 0.005 restores); the bench config digests
    (`bench_artifacts.rs:220-321`; rate and the two deleted knobs, with an
    undo link each); the energy-accounting and plasticity tests that assert
    the deleted costs; `drift_depth` and the progress chart; goal readings
    that carry genome size, carrying cost, gestation, supply and exposure.
  - Tests first: `genome_size_charges_no_wired_sink_unit` (a sink with 2
    edges goes 3 → 2), `a_ring_projection_costs_nine_units`,
    `founder_is_eighty_units` with the arithmetic in a comment, the carry-0
    sibling unchanged under the unit rule.
  - Readings: mean genome size, mean wired sinks, mean projections, births
    per creature-tick and gestation on the three goal worlds through 40,000
    ticks against the merge base (F02 opens the `goal_worlds_long` series);
    requested events per birth decomposed as `E_old = 0.005 (x̄' + S̄ − 8P̄)`,
    `E_size = 0.005 x̄'`, `E_new = 0.0060625 x̄'`, with the extra arm
    isolating the rate.
  - Reference specs: `v3-genome-spec.md` §3 (wired-sink rule, 97),
    `v3-runtime-config-spec.md` (`genome_size` definition, cost rows, the
    cost knobs), `v3-reproduction-spec.md` (97), `v3-mutation-spec.md` §4.2
    (supply) and the split-exclusion rows, `v3-graph-backend-spec.md` §6 size
    table and §12, `v3-server-api-protocol-spec.md:217`,
    `v3-startup-seeding-spec.md:170`.
- **T25.F03 — Ring Groups Replace Ring Projections.**
  - Goal: One developmental program wires a whole map (Eph/ephrin gradients,
    central-complex columns): a ring projection is a group of eight real
    synapses with one shared scale, and any one synapse can later be changed
    or lost on its own.
  - Depends on: T25.F02.
  - Scope, in a text-only commit then a behavior commit: `Membership`,
    `GroupId`, `Group`, `Shape::Ring`, `EdgeOccurrence`, `compose`,
    `genomic_weight`, I2–I10 and `validate_graph` with its gate skip reason
    and counter; the one hand-written identity pair; `RingProjection`, its
    evaluator and its list deleted outright, `Ring` groups in their place
    with no converter (size-neutral, pinned); `GraphRuntimeState::for_genome`
    with the genomic weight table and every construction site migrated; the
    five group operators in `group.rs` with two-phase relocation under I10;
    detach on retarget, raw-field and split; `GroupKey::Ring` deleted and the
    refinement renames; refinement skipping members; recruitment's ring case
    under both arms; the projection branches, `ALL_RING_CHANNELS` and
    `ablated_channels` deleted from both ablation paths; `live_barrier_use.rs`
    deleted; readers (`ring_rules` → ring-rules-v2 with `ring_group_*`
    fields, `drives` skipping members, `sensor_census`, annotations,
    `live_circuit`, `discovery.rs`, `ladder/track.rs`, `steering::votes_move`);
    `graph_edges_detached_total` and `groups_removed_total` with their
    families and check rows; `REACHABLE_STRUCTURE_VERSION` v2; the serde
    shape (`groups` replaces `projections`, `own_weight`, omitted-when-`None`
    fields, `deny_unknown_fields`) and the lab `GENOME_FORMAT` bump; the
    `graph_weights` payload with its token and a protocol-version bump (and
    a §2 history line); `mesh_annotations` required with the genome and the
    frontend's `infer*`/`collectNodeBadges` deleted; the edge-group list
    (one row per group: shape, kind, offset, scale, members present over 8
    with holes; expandable member rows; member edges dashed in a
    `group_member` tint at the server's genomic value; a per-group collapse
    defaulting open); `types/genome.ts`; petri-visuals `edgeGroups`, the
    `TINT` mirror and the min build.
  - Operators: `AddRingProjection` → `AddRingGroup` (2), `AlterRingProjectionWeight`
    → `AlterGroupScale` (4), `ShiftRingProjectionOffset` → `ShiftGroupOffset`
    (2), `RetargetRingProjectionKind` → `RetargetGroupKind` (2),
    `RemoveRingProjection` → `RemoveGroup` (2, `Decreasing`): 28 / 56 /
    `[Self; 61]`; only names and keys move, asserted.
  - Verification: viability first; gate + goal + goal-long (member erosion
    and ring-carrier retention are long-run questions, and the ring motif was
    declared goal-long); mutation gate run; slow tests run. Predeclare the
    gate's `wall_clock_ms_per_creature_tick` delta, `plasticity_updates`
    (unchanged; learning is per node still) and the AddGraphEdge position
    change under I10.
  - Predeclared re-pins: operator keys in events and telemetry (stored
    artifacts stay historical); genome identity and JSON for every graph
    genome (text commit); `GENOME_FORMAT`; `PROTOCOL_VERSION`; `size_of::<GraphEdge>()`
    16; `neutral_connection_tests/projection.rs:212` rewritten by hand to the
    new summation order; `recruitment_tests.rs:223` and its regression file;
    goal readings that carry complexity and `ring_rules`.
  - Tests first: a `Ring` group (scale s, corrections `c_i` drawn nonzero)
    equals eight ungrouped edges of weight `s + c_i` bitwise, with members in
    I10 position (`eight_edges` oracle; `projection.rs:99, 126, 155, 189`
    swapped to the group builder; strategies restricted to valid groups, the
    invalid arms moved to `validate_graph`-rejects tests); `remove_group`
    leaves no member and renumbers links; `shift_then_unshift_is_identity`
    under I10; `detach_preserves_genomic_weight_bitwise`;
    `refinement_never_steps_a_member` (a `Slots` fixture, since the ring key
    is gone); `a_node_voting_only_through_group_members_is_a_motor_node`;
    `group_member_channels_are_on_the_vote_surface`; an annotation-driven
    test that a member-only node is an action writer; the per-field
    ring-rules-v2 expectation for a `Ring` group (`direction_general` yes,
    `direction_general_edges_only` no, `ring_group_*` counts); the catalog
    bijection test with `:70`'s weight, arm, key and admission assertions
    ported to the group names; `ablating_a_grouped_genome_leaves_size_complexity_groups_memberships_and_member_weights_unchanged`
    and `masking_a_ring_reference_zeroes_every_member_read`; the engine-level
    invariant proptest with the Δ column per operator (`AddRingGroup` +9,
    `RemoveGroup` −(n + 1), `RemoveGraphEdge` on a last member −2); the
    two-way identity test and a JSON round trip; the frontend tests: an
    `EdgeGroupList` test replacing `RingProjectionList.test.tsx`, a
    `graphFullModel` test that a member at scale 0.5 and correction 0 draws
    at 0.5 from `graph_weights`, the composition source in the drift
    fingerprint and the `group_member` tint asserted. Every `RingProjection`
    test file is listed as ported, rewritten or deleted
    (`projection_tests.rs:54-201`, `neutral_connection_tests/projection.rs:246-421`,
    `mutation/graph/tests/projection.rs`).
  - Readings: the three readers' post-change expectations; ring groups per
    creature, members per surviving group and detach or remove events per
    group over 40,000 ticks against the erosion the weights predict; the
    blocked-move fraction of ring-group carriers against non-carriers; the
    size series continued from F02 (size-neutral expected).
  - Reference specs: `v3-graph-backend-spec.md` §2, §6, §8 (entered-visit
    clause), §9, §14, §16 (soft-default rows); `v3-genome-spec.md` §3, §4;
    `v3-mutation-spec.md` §4.2 step 7, §5, §6, projection and recruitment
    rows; `v3-runtime-config-spec.md:185`; `v3-server-api-protocol-spec.md`
    §2, §4.11.
- **T25.F04 — Direction-Pair Groups.**
  - Goal: The sensory map sits in register with the motor map (superior
    colliculus): a creature's sensed offset to a thing, food, a barrier, a
    crowd or another creature, can reach all eight direction votes of one
    kind as one silent gene, and its plain facts reach that kind's drive.
  - Depends on: T25.F03.
  - Scope: `Shape::DirectionPair`, the compass pattern from
    `Direction::delta()`, pair metadata in `contracts::inputs` for today's
    families with `sensors::reducers` as the writer; `AddDirectionPairGroup`;
    the five group operators extended to pairs (relocation around holes);
    recruitment's pair case with one pair or one slot and scalar drive
    edges; `drives` reporting pair groups in their own block (drives-v2);
    a `pair_groups` block (pair-groups-v1) on every persistence checkpoint
    with class counts (family, kind, offset, sign of scale, scale off 0) and
    the cumulative `AddDirectionPairGroup` applied count, in the keep-list;
    `refinement_applications_by_key_total`; the inspector rows (12 expected
    members) and petri-visuals.
  - Operators: `AddDirectionPairGroup` (2, `Increasing`): 29 / 58 /
    `[Self; 62]`.
  - Verification: gate + goal + goal-long (a new operator remaps every
    Graph draw, so the 2,000-tick reading is attributable; retention is
    visible only past 20,000 ticks); mutation gate run; slow tests run.
    Predeclare the wall-clock delta of pair-member weights in the table.
  - Predeclared re-pins: every Graph draw; the catalog pins; drives-v1 →
    v2; genome identity of every grouped genome (shape variant).
  - Tests first: `py_equals_normalized_delta_dy`; per pair-declaring family,
    `a_pair_group_at_offset_0_votes_toward_a_thing_at_each_direction` and the
    offset-4 counterpart; `pair_creation_materializes_12_members_not_16`;
    every declared pair's indices within the family width; every swap
    alternative keeps the variant; shift and retarget on a pair group with
    holes; recruitment's pair-case draw order; `node_fanouts` skips members;
    a two-slot pair-group fixture that refinement never steps.
  - Readings: the `pair_groups` block at tick T − N and T beside the
    applications between them; blocked-move and steal-hit fractions of
    carriers against non-carriers; harmful-per-birth not up; refinement
    applications by key.
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
    the sampler (27 buckets, indices 10–12, D13) and its pins; recruitment's
    family universe weighting `NearbyCreature` three times and drawing one
    slot immediately after the destination on every arm; labels,
    `inputRefUtils.ts`, the sensor census universe, the input-use catalog
    (input-use-v2), mutation-effects contexts (mutation-effects-v2), the
    `WorldReads` bit layout, span attributes, the sampler perception payload
    (protocol bump, §2 history line), telemetry gauge values and dashboards,
    `MutationAddedNodeInputClass`, annotation classes, the lab's
    `census_covers`; the `Slots` refinement key re-keyed.
  - Verification: gate + goal; mutation gate run; slow tests run.
  - Predeclared re-pins: every `InputRef.Add` and recruitment draw (the
    family universe), the sampler index tests, genome identity of every
    genome with a creature reference, `GENOME_FORMAT`, `PROTOCOL_VERSION`,
    the sensor gauge series (99 stored feature JSON files carry the old keys;
    historical).
  - Tests first: `merged_nearby_creature_equals_core_vitals_identity_per_slot`
    written while the three families still exist, kept as the layout pin;
    the `Slots` fixtures rewritten to `field = channel mod 9`; the
    recruitment draw-order test with the slot draw after the destination.
  - Readings: declaration and recruitment share of the merged family against
    the three old ones; goal indicators not down.
  - Reference specs: `v3-sensor-spec.md` §1, §3.1, §5.4–5.6;
    `v3-mutation-spec.md` sampler table; `v3-server-api-protocol-spec.md` §2.
- **T25.F06 — Learning Per Edge.**
  - Goal: Synapse-type-specific plasticity with three-factor traces: each
    synapse carries whether it is fixed, Hebbian or reward-modulated, which
    reward it listens to and how long its trace lasts, and a synapse onto a
    vote can learn like one onto a neuron.
  - Depends on: T25.F05.
  - Scope, in a text-only commit then a behavior commit: `plasticity` on the
    edge and `ComputeNode.plasticity` removed (36 production files); the
    learning layout and `LearnedState::from_seeds` in `for_genome` with the
    `initialized` predicate; samples; updates after effects with the clamp
    rule; sink edges learn (members excluded until F07); `BirthSeeds` sink
    rows under I11; `EdgeMut::set_plasticity`; the nine plasticity operators
    on edges with the `ClearSlot` and member exclusions; `plasticity_census`
    through the runtime view with scale-free rows; the telemetry families of
    §7 and the dashboards check; `companions` with "carries learning"
    defined once (any plastic edge on either surface of a reachable node);
    annotations; recruitment-paths records (v3); mutation-effects state
    records (v3); the serde shape, `types/genome.ts`, `graph_weights`
    `applied` and `learned` per edge in every poll, sink `weighted_inputs`
    in the trace payload, the plasticity badge, the two knobs removed from
    the frontend config types, fixtures and tests; the lab's `expressed()`;
    `plasticity_updates` redefined (one per plastic-edge assignment on both
    surfaces; per visit for Hebbian; per initialized modulated edge per tick)
    with a direction.
  - Operators: the nine move with their weights (18 of the arm): 29 / 58 /
    `[Self; 62]` unchanged in count; keys unchanged.
  - Verification: viability first; gate + goal; mutation gate run; slow
    tests run. Predeclare `wall_clock_ms_per_creature_tick`, the
    `plasticity_updates` shift with its mechanisms, phase 0 and 2.5 timings
    and a dhat heap check through a named harness (`profile_ticks` has no
    fixture option).
  - Predeclared re-pins: identity of every graph genome (text commit;
    `plasticity` prints on every edge); every plasticity draw; the
    AddInternalGraphNode split-site draw on learning graphs (the exclusion
    is gone); the recurrent-edge Hebbian trajectories; `applied_trajectory.rs:104-105`
    edited (it hashes the deleted storage); `GENOME_FORMAT`; `PROTOCOL_VERSION`;
    `size_of::<GraphEdge>()` 36; the mutation-effects and recruitment-paths
    tokens; the cognitive-class and `has_plasticity` series.
  - Tests first: a table over every test in `plasticity/tests.rs:140-661`
    (numeric learning expectations kept with each node's config on its
    compute-input edges; cost assertions deleted; lazy-empty assertions
    replaced by `initialized` assertions; `EnergyCurrent` sample versus
    re-resolve named); `hebbian.rs:352-630` draw tails ported onto edges;
    `plasticity_census_tests.rs` ported to the view (`:121` becomes an
    `initialized` test); `sink_edge_learns_from_the_recorded_sample_not_a_replay`
    (a `WriteSlot` earlier in sink order changes memory a later sink edge
    read); `a_non_learning_graph_allocates_no_learned_state`; the self-edge
    regression; `clear_slot_edges_are_never_eligible`;
    `members_are_not_eligible_until_scope_exists`; a model-based proptest of
    sink-row seeds (uniquely tagged seeds through random operator sequences
    against a per-surface `Vec<Option<tag>>` model, through push, remove,
    split, routing's clone, recruitment); `inherited_weight_above_a_mutated_child_clamp_is_clamped`;
    the zero-energy floor pinned at cost 0.
  - Readings: plastic edges by learning mode and surface, by rule, by
    inheritance; drift and sampled clamp occupancy by surface; visits per
    tick of learning nodes; goal indicators not down.
  - Reference specs: `v3-graph-backend-spec.md` §8 ("initialized"), §9
    (sink `weighted_inputs`, the cost sentence), §13; `v3-genome-spec.md` §5;
    `v3-reproduction-spec.md` §4 "Graph state"; `v3-mutation-spec.md`
    plasticity rows; `v3-mesh-execution-spec.md`, `v3-tick-orchestration-spec.md`,
    `v3-evolution-observability-spec.md`, `v3-cli-contract-spec.md`
    plasticity rows; `v3-server-api-protocol-spec.md` §2, §4.11.
- **T25.F07 — Group-Scope Learning.**
  - Goal: A synapse built by a group program can learn for the whole group
    or for itself, and a change to such an attribute lands on one synapse or
    on every synapse its group built.
  - Depends on: T25.F06.
  - Scope: `Scope` in `Membership`, `inherit_scale` on `Group` with
    `GroupMut::set_inherit_scale`, learned scales and group bounds, the
    pattern-weighted credit (D4), members admitted to the plasticity
    operators' predicates, the scope coin with its group-scope semantics,
    `SetScope`, uniform learned creation for both shapes, group and detach
    seeds under I11 (group slots), the runtime view's group items; the serde
    shape, `types/genome.ts`, `graph_weights` `learned` per scale, the
    edge-group list columns (scope, `inherit_scale`, learned scale), the
    by-scope and learned-scale telemetry families in `always_names` with
    panels; census scale rows (drift against the genomic scale once per
    group, at-bound against the group bound) and the counter rule (one
    `plasticity_updates` per contributing member).
  - Operators: `SetScope` (2, `Neutral`): 30 / 60 / `[Self; 63]`.
  - Verification: viability first (phase 0 and 2.5 routing change); gate +
    goal; mutation gate run; slow tests run.
  - Predeclared re-pins: every Graph draw; identity of every grouped genome;
    the creation draws of F03 and F04 (now learned); `GENOME_FORMAT`;
    `PROTOCOL_VERSION`.
  - Tests first: `oja_group_members_use_visit_start_applied_weight`;
    `group_scale_credit_is_pattern_weighted_summed_in_order_and_bounded_once`
    (the scale delta equals the bounded `Σ p_m Δ_m` from the recorded
    samples; own weights unchanged); `a_static_member_contributes_nothing`;
    `a_scale_seeds_only_with_a_group_scope_learner`; the scope-coin tails
    ("a non-member draws no coin", "a member the predicate rejects is
    untouched and consumes no draw", `EnableHebbian` on a modulated member,
    `EnableRewardModulation` on a modulated member, `Toggle` writes the
    picked value); the seed proptest extended to group slots; the detach
    seed composition with a learned scale.
  - Readings: plastic edges by scope; learned-scale drift and at-bound
    occupancy; first-update snaps of genomic scales; goal indicators not
    down.
  - Reference specs: `v3-graph-backend-spec.md` §2 (scope, `inherit_scale`),
    §13 (scope); `v3-genome-spec.md` §3, §5; `v3-reproduction-spec.md` §4
    (group and detach seeds); `v3-mutation-spec.md` operator table;
    `v3-server-api-protocol-spec.md` §2, §4.11.
- **T25.F08 — Creation Forms, Detach and the Catalog.**
  - Goal: A mutation in a developmental gene changes every synapse it builds;
    a mutation at one synapse changes only that one. Group creation is
    uniform or randomized, a member can be detached, and the catalog is one
    consistent set of edge and group operators.
  - Depends on: T25.F07.
  - Scope: `AddRandomizedRingGroup` and `AddRandomizedDirectionPairGroup`;
    `DetachGraphEdge`; the no-join rule for `AddGraphEdge` pinned; the
    `Slots` decision, keep or delete, from the post-F05 refinement-by-key
    readings, with catalog totals for both outcomes and the deletion list
    for "delete"; the layer grouping of the operator funnel derived in the
    bench reading from operator keys, not exported; the catalog tables and
    tests.
  - Operators: three new at 2 each (`Increasing`, `Increasing`, `Neutral`):
    33 / 66 / `[Self; 66]` if `Slots` stays, one fewer each if it goes.
  - Verification: gate + goal; mutation gate run; slow tests run.
  - Predeclared re-pins: every Graph draw; the catalog pins; the
    `structured_heritable_refinement` config digest if it goes.
  - Tests first: randomized creation draw order; a detached Lamarckian
    member's seed and a detached static member's; the last-member detach
    removes the group; `add_graph_edge_never_joins_a_group`.
  - Readings: operator funnel by layer; share of attribute changes applied
    at group scope; harmful-per-birth not up; the `Slots` firing rate.
  - Reference specs: `v3-mutation-spec.md` operator table and, if `Slots`
    goes, its refinement rows.

## 9. Remaining uncertainty

- Whether evolution picks up pair groups at all; F04's goal-long reading is
  the test, and failure returns the question to sector rings (option F) or
  to a learning route.
- The supply rate under uniform size (D14, option a): an evolved genome
  draws `1.2125 (x − S + 8P) / x` of its old events, where genome bloat is
  the carried finding; F02's goal-long and its rate-0.005 arm read it.
- Vote-edge Hebbian runaway, allowed by decision; F06 reads clamp occupancy
  and visits per tick.
- Hot path: F03 reads one table entry per sink edge and widens the edge to
  16 bytes, F06 to 36; F06 adds sample recording on learning graphs only.
  Each predeclares its wall-clock and counter deltas.
- The `Slots` refinement key is an open decision for F08.
- Version 3 was reviewed by the nine facets once (version 2) and not again;
  each feature's spec runs its own three-round Codex challenge against this
  note and the code, and the user decides whether a third facet round runs
  on the three facets that produced the most new findings (tests,
  boundaries, data model).

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
- Codex Astra adversarial review (three rounds) and two rounds of nine Opus facet reviews, 2026-10-13: `docs/progress/readings/t25-design-review.md`
