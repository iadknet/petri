# Edge Groups and Per-Edge Learning Design Note (2026-10-13)

The user asked how creatures sense other creatures, found that the
multi-field creature sensors (`NearbyCreatureCore`, `Vitals`, `Identity`,
`AreaOccupancySummary`) never turned into behavior in the live world while the
neighbor rings did, and asked what it would take for those sensors to be
wired the way ring projections are wired: all of an input's fields added
together, as a group, with the group evolving as a unit or edge by edge. The
answer grew into a refactor of the graph genome's edge model. This note records
the design, the boundaries it draws, the options it rejected, the user's
decisions, and the feature contracts the spec owner reads. An adversarial
Codex Astra review (three rounds, 2026-10-13: 24 findings, then 7, then 3
blocking and 1 advisory with 5 partial confirmations; the round 3 edits
were not re-reviewed) shaped §3, §4, §7 and §8; the findings and
dispositions are in
[`docs/progress/readings/t25-design-review.md`](../progress/readings/t25-design-review.md).

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
  shared across a group changes every member equally. The plasticity update
  cost stays 0 for now.
- Group creation comes in two forms: every member alike, or learning
  attributes randomized per member. The geometric pattern is never
  randomized.
- The three nearby-creature families merge into one `NearbyCreature`
  family. An edge costs one genome unit wherever it lives.

## Answer

Today an edge is `{source, weight}` stored positionally in a compute node's
or sink's `inputs` list; learning is a setting on a compute node that covers
all of its input edges; the only stored group is the `RingProjection` gene,
and two other mechanisms infer groups from edge patterns on the fly. The
refactor:

1. **Edge** gains `group`, `learning` and `scope`. It stays in its container
   (input order is semantic for `GreaterThan`, `Select` and the trace
   payload), so no edge identifiers are needed; a group member is a sink
   edge whose `group` names a group record on the backend definition and
   whose source and target fit the group's shape.
2. **Group** is a record `{shape, scale, inherit_scale}`. A shape fixes the
   geometric pattern of its members; a member's genomic weight is
   `scale × pattern(source, target) + edge.weight`, one pure function every
   reader calls, with a runtime resolver that substitutes learned values.
   Ring projections become `Ring` groups with eight materialized member
   edges. A new `DirectionPair` shape fans a sensor's `(x, y)` pair into a
   kind's eight direction sinks in the fixed compass pattern.
3. **Learning** moves to the edge: learned state (learned own weight,
   eligibility trace) is indexed over both surfaces, compute inputs and sink
   inputs, so vote edges can learn; a member with scope `Group` applies its
   update to the group's learned scale, which every member feels through the
   pattern.
4. **Mutation** has an edge layer and a group layer. Attribute operators pick
   an edge, then draw whether the change applies to that edge or to every
   member of its group. Structured refinement's inferred ring groups and the
   recruitment plan's transient bundles become explicit groups.
5. **Readers** read explicit membership from the genome; wiring analyses that
   look for direction-general behavior (ring rules, drive fan-outs) keep
   their own derived definitions and compute them from members and genomic
   effective weights.

Five features, in order: T25.F01 the container API, edit records and birth
correspondence (behavior-preserving); T25.F02 edge groups replace ring
projections; T25.F03 direction-pair groups and the `NearbyCreature` family;
T25.F04 learning per edge and group with the scope draw; T25.F05 group
creation forms, detach and the catalog. §8 holds the contracts.

This recommendation is provisional against the repository's research depth
bar: the local evidence is complete (three code maps, a live probe and one
adversarial review round), the external precedent rests on one paper
verified at source and on summaries of the others (§5), and no scratch proof
has been run. The features' own gate and goal readings are the proof; no lab
run is planned (user ruling, 2026-10-10).

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
- No backward compatibility: stored genomes, hash pins and draw sequences
  constrain nothing. Expected re-pins are predeclared.
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

```rust
/// A heritable connection. Stays in its container's `inputs` list.
pub struct Edge {
    pub source: GraphSource,
    /// Own weight. Inside a group: a correction on top of the group term,
    /// born 0.
    pub weight: f32,
    pub learning: Learning,
    /// Meaningful only inside a group; validation rejects it elsewhere.
    pub scope: Scope,
    /// Index into `CgpGraphBackendDef::groups`, or none.
    pub group: Option<u16>,
}

pub enum Learning {
    Static,
    Hebbian { rule: HebbianRule, rate: f32, clamp: f32, lamarckian: bool },
    RewardModulated {
        rule: HebbianRule,
        rate: f32,
        clamp: f32,
        source: OutcomeChannel,
        trace_decay: f32,
        lamarckian: bool,
    },
}

/// Where a member's learning update lands.
pub enum Scope { Edge, Group }

pub struct Group {
    pub shape: Shape,
    /// One shared weight, born 0.
    pub scale: f32,
    /// Whether a learned scale seeds the child's runtime state.
    pub inherit_scale: bool,
}

pub enum Shape {
    /// Members are sink edges from ring cell `c` into `K[(c + offset) mod 8]`;
    /// pattern 1.
    Ring { ring_ref: u16, kind: DirectedVoteKind, offset: u8 },
    /// Members are sink edges from the pair's x or y field into a direction
    /// sink of `K`; pattern is the compass projection (§4) rotated by `offset`.
    DirectionPair { x: InputLeafRef, y: InputLeafRef, kind: DirectedVoteKind, offset: u8 },
}

/// Genomic composition: what every structural reader uses.
fn genomic_weight(edge: &Edge, group: Option<&Group>, target: SinkIndex) -> f32 {
    match group {
        Some(g) => g.scale * g.shape.pattern(&edge.source, target) + edge.weight,
        None => edge.weight,
    }
}

/// Runtime resolution: the same composition over learned values where the
/// runtime holds them, falling back to the genomic ones.
fn applied_weight(edge, group, target, learned: &LearnedState) -> f32 { ... }
```

Decisions behind the shape of this model:

- **Edges stay in their containers.** A global edge list with stable
  identifiers was considered (§6, option D). Input order is semantic, the
  trace payload is ordered per target, and every reader walks a container; a
  global list would need that order anyway and would move every walker. A
  group index on the edge gives explicit membership without identifiers.
- **Membership is a validated coordinate, not a stored position.** A member
  is a sink edge whose `group` is set and whose `(source, target)` fits the
  shape: for `Ring`, source `InputLeaf{ring_ref, c}` into sink `K[(c + offset)
  mod 8]`; for `DirectionPair`, source `x` or `y` into any direction sink of
  `K`. The pattern is a function of that coordinate, so nothing else is
  stored per member. Validation rejects a member that does not fit. Any
  operation that changes a member's source or target outside a shape
  operator (retarget, raw-field step, split through a compute node) detaches
  it first, atomically (§4). Compute-input edges cannot be members while
  every shape targets direction sinks.
- **Members are materialized.** A `Ring` group is eight member edges plus one
  group record, not a generator term as today's projection is. That is what
  lets a mutation delete, retarget or re-weight one member while the group
  survives, and what lets one member be an exception, the HybrID pattern
  (§5). The hot path loses the separate projection term and gains one
  indexed group lookup per member edge (`Option<u16>` into a short `Vec`;
  no search).
- **Pattern zeros are not materialized at birth.** A `DirectionPair` member
  for `x` has pattern 0 toward N and S, so creation makes 12 members, not
  16. An omitted member is geometrically unnecessary at birth, not
  impossible: an ordinary `AddGraphEdge` can later add that edge, and if it
  fits the shape it may join the group through its correction.
- **Learned state stays in the runtime, as absolute values.** The genome
  carries the learning configuration; `GraphRuntimeState` carries, per
  learning edge, a learned own weight and an eligibility trace, indexed
  `[mesh][surface][edge]`, and per group a learned scale. The genome never
  absorbs learned values; a Lamarckian edge seeds the child's learned own
  weight, and a group with `inherit_scale` seeds the child's learned scale,
  both selected by the child's post-mutation flags, as today.
- **Identity invariant.** A genome's `Debug`, `PartialEq` and JSON include
  every heritable attribute (`source`, `weight`, `learning`, `scope`,
  `group`, and each group record) and exclude learned values, traces and
  birth payloads, continuing the rule today's custom impls apply to
  `birth_weights` and `ablated_channels` (`cgp.rs:191-209, 229-252`).
- **Detach** is a genomic transformation: `edge.weight :=
  genomic_weight(...)`, `group := None`, `scope := Edge`. Mutation runs only
  on the child draft at birth, so the only learned state in play is the
  parent's, and it reaches the child as seeds (§7): a detached edge whose
  own flag is Lamarckian is seeded with the parent's learned own weight plus
  the parent's learned scale times its pattern, so the applied weight the
  parent had is what the child starts from; a static member has no learned
  slot before or after, so the parent's learned scale contribution ends
  with the membership and the child starts from the genomic weight. Nothing
  learned enters the genome. **Remove group** removes the record and every
  member. **Remove edge** on a member removes only that member; a group
  whose last member is removed is removed with it.
- **Accounting contract.** The user accepted "an edge costs one unit
  wherever it lives, a group record one". Today's `genome_size` also
  charges one unit per wired sink, so eight members born onto eight unwired
  sinks would cost 17 units, nine onto already-wired ones. The
  recommendation is to drop the per-wired-sink unit so the rule is uniform:
  `genome_size` = mesh nodes + input references + route targets + compute
  nodes + edges on both surfaces + group records (VM: program length +
  constants). The founder anchor (`FOUNDER_GENOME_SIZE_UNITS`) is re-pinned
  to the founder's recomputed size, so the replication multiplier, which is
  relative to it, leaves the founder's reproduce charge unchanged. For a
  creature with `P` projections and `S` wired sinks before any other edit,
  `Δgenome_size = 8P − S`: the live world carries about 0.4 projections and
  roughly 17 wired sinks per creature, so most creatures get smaller and
  projection carriers grow by up to 8 per projection. Every reader of
  `genome_size` moves with it: carrying cost (`1e-4` per unit per tick),
  mutation supply (`Binomial(size, 0.005)`), the replication multiplier
  above the anchor, gestation (`ceil(size × rate / anchor)`), and the
  size-pressure roll at `(size / cap)²` against the 1,200-unit cap
  (`mutation/engine/mod.rs:128-130`; off by default).
  Functional complexity is a different measure and keeps its own rules: a
  wired sink counts one whether its edges are members or not, sink edges are
  not counted individually, a group record counts one in place of the
  projection's one, and a consumed reference counts one; a migrated
  projection whose eight sinks were unwired therefore raises complexity by
  up to 8, which the action-cost multiplier, the reproduce charge and the
  predation kill bonus (`kill_complexity_bonus_multiplier × victim
  complexity`, `simulation/actions/predation.rs:125-127`; default 0) read.
  F02 predeclares both shifts and its goal reading watches genome size,
  complexity, births and gestation. Dropping the wired-sink unit reaches
  beyond the first accepted decision; the user confirmed it on 2026-10-13.

## 4. Mechanisms

**Compass pattern.** For direction index `d` in `N, NE, E, SE, S, SW, W, NW`,
`px = [0, .71, 1, .71, 0, −.71, −1, −.71]` and
`py = [−1, −.71, 0, .71, 1, .71, 0, −.71]` (y grows southward). A pair group's
member for `x` into sink `K[d]` has pattern `px[(d − offset) mod 8]`; the `y`
member has `py[...]`. With a positive scale the group votes toward the sensed
position; offset 4 votes away; the sign of the scale is evolved. This is the
geometry Petri already builds in for rings (`RingProjection::channel_for`)
extended to a vector, and it is the one thing the design bakes in.

**Eligible pairs live with the sensor layout.** Which fields of a family form
an `(x, y)` offset is declared beside the family's field indices in
`sensors/perception.rs` (`NearbyCreature` slot k `(dx, dy)`;
`AreaOccupancySummary` `(centerX, centerY)` and `(nearestDx, nearestDy)`;
`AreaFoodSummary(t)` `(gradX, gradY)` and `(nearestDx, nearestDy)`;
`AreaBarrierSummary` the same two). The group shape consumes that metadata;
there is no second table to keep in step. Input references are shared by
both backends, so the merged family's layout is one contract for Graph
edges, VM `ReadInput` and every label.

**Learning on edges.** The update law for a learning edge is today's
(`runtime/plasticity/hebbian.rs:138-145`; `reward.rs:27-88`) with the edge's
own `rule`, `rate`, `clamp`, `source` and `trace_decay`. Contract:

- *Samples, not replays.* Every update uses the pre and post values the
  evaluation itself used: for a compute-input edge, the source value read in
  ordered evaluation and the node's committed output; for a sink edge, the
  source value the sink sum read, in sink order, and the sink's weighted sum
  (its vote, output or write value). Effects write shared memory as the sink
  loop proceeds, so a later re-resolve would not reproduce those values; the
  executor records the per-edge samples during evaluation and the learning
  step consumes them after effects. One convention for both Hebbian and
  reward-modulated edges.
- *Order.* Evaluation reads the applied weight in force at visit start;
  updates run after effects for every edge, so no edge learns from a value
  its own update changed within the visit. Every Δ, including Oja's
  normalization term, is computed from the recorded visit-start applied
  weight (today Oja reads the mutable learned weight,
  `runtime/plasticity/hebbian.rs:136`, `traces.rs:151`); only then are the
  contributions applied in genome order with their clamps, so a learned
  scale several members change in one visit gives the same result as the
  same samples in any other visit.
- *What an update targets.* A member with scope `Edge` updates its learned
  own weight; a member with scope `Group` updates the group's learned
  scale. Each member computes its own Δ with its own configuration; a group
  may hold static members, members with different reward sources or decays,
  and a static member simply contributes nothing. Several members may
  contribute to the scale in one visit: contributions are summed in genome
  order, and the scale is clamped by each contributing member's `clamp` as
  it is applied. The group record owns no learning configuration of its
  own; `inherit_scale` is its only learning attribute.
- *Equal credit.* "Changes every member equally" is realized as one change
  to the shared scale: every member's applied weight moves by the same
  factor of its pattern, so a `Ring` group moves every member by the same
  amount, and a `DirectionPair` group moves toward or away as a unit. An
  equal additive change across a kind's eight directions would be a drive,
  which the 2026-10-11 decision keeps as a separate gene. (Codex round 1
  read "equally" as additive; the user confirmed the scale reading on
  2026-10-13.)
- *Clamp.* `clamp` bounds the learned quantity an update writes: the learned
  own weight for scope `Edge`, the learned scale for scope `Group`. Oja's
  normalization reads the applied weight.
- *Runaway.* Pure Hebbian learning on a vote edge feeds back on itself (a
  larger vote strengthens the edge). It is allowed, evolvable, bounded by the
  clamp, and selection removes it where it hurts. The user chose to make the
  mode evolvable rather than restrict vote edges to reward modulation.

**Shape operators relocate members.** `ShiftGroupOffset` moves every
surviving member one direction sink clockwise (or counter-clockwise); its
pattern travels with it, since the pattern is a function of the rotated
coordinate. `RetargetGroupKind` moves every member to the other kind's sink
of the same direction. A moved member is removed from its old sink's
`inputs` and appended to its new sink's, members in genome order; its
correction, learning configuration, scope and learned state travel with it;
holes (deleted members) stay holes; nothing is regenerated. Sink edges carry
no birth correspondence today; F04 extends the seeds to sink edges and
groups, and a relocation record carries it (§7).

**Group removal is one atomic operation.** `remove_group(i)` removes the
record, removes every member edge, and decrements every edge's `group` index
above `i`. The same operation runs when `reindex_input_refs_after_removal`
drops a reference a shape names (`ring_ref`, or a pair's `x` or `y`), and
`InputRef.Prune` counts shape references as consumers so it never prunes
one. A group with no members is removed. Property tests cover every edge's
link after any sequence of removals.

**Copy operators and members.** `CopyInternalNode`, `CopySubgraph`,
`CopyEdgeBundle` and `duplicate_compute_nodes_in_place` copy compute-input
edges only; sink edges and groups are untouched, so a copy never duplicates
a member. `AddGraphEdge` may land on a sink a group targets; the new edge is
not a member unless it fits the shape and the operator says so (F05 decides
whether a fitting edge joins; F02 says no).

**Ablation is an observation-only mask.** The lab's `readings::ablate`, the
core's `input_use::ablated` and the lab's `expressed()` (learning off) stop
rewriting genome copies. They pass an evaluation-context mask with three
parts: `(mesh node, input reference index, sub-value or all)` keys whose
reads return 0 in either backend (a Graph `InputLeaf` edge, a VM
`ReadInput`, a group shape reference; today's ablation rewrites VM reads
per sub-value too, `neighborhood/input_use/consumers.rs:289-299`,
`crates/v3-lab/src/readings.rs:633`); `(mesh node, surface, edge index)`
triples for masking one Graph edge; and a flag that disables learning. The genome, its membership, input order,
learned-state correspondence, size and functional complexity are untouched
by construction. `ablated_channels` is deleted.

**Recruitment emits groups.** `RecruitNeutralInput` under `WholeFamily`,
landing on a directed kind `K`:

- a ring family becomes a `Ring` group of eight born-0 members, as today's
  projection;
- a family with an eligible pair becomes one `DirectionPair` group of twelve
  born-0 members for the drawn pair, plus one born-0 edge per remaining
  scalar field of the drawn slot into `Drive K` (scalar → drive, the
  2026-10-11 rule);
- any other family keeps today's behavior.

For `NearbyCreature` the plan recruits one drawn slot, not all four; four
slots of twelve members plus drives per event is too heavy a step.

**Draw contracts.** Every operator documents its draw order in code, as
today.

- *Attribute operators* (F04: learning mode, rate, clamp, reward source,
  trace decay, Lamarckian flag, scope): pick the Graph module as every
  Graph operator does; pick one eligible edge uniformly over both surfaces
  in canonical order, eligibility per attribute as today's node-level
  predicates define it (`hebbian.rs:141-164`, moved to edges); if the edge
  is a member, `gen_bool(0.5)` chooses this edge or every member; the new
  value is drawn once and applied through the transition table below to
  every selected member. No eligible edge is `NoApplicableTarget`, redrawn
  as today.
- *Transition table.* `SetLearningMode(m)` draws `m` uniformly over the two
  modes the picked edge is not in, then draws, once and before the member
  loop, every field a transition into `m` can need: rule, rate, clamp and
  Lamarckian flag with today's `EnableHebbian` laws (`hebbian.rs:195-210`)
  and source and decay with today's `EnableRewardModulation` laws
  (`:277-291`). Then per selected member: already in `m`, unchanged; from
  `Static` into a learning mode, the member takes every drawn field it
  needs; from `Hebbian` into `RewardModulated`, the member keeps its own
  rule, rate, clamp and flag and takes the drawn source and decay; from
  `RewardModulated` into `Hebbian`, it keeps its own rule, rate, clamp and
  flag and drops source and decay; into `Static`, it drops everything.
  "The same values" means the drawn fields; a member's retained fields stay
  its own. A parameter operator draws one step before the loop and applies
  that step to each selected member's own value: `rate`, `clamp` and
  `trace_decay` use today's multiplicative-or-additive law (`hebbian.rs:246-261`),
  `clamp` bounded to `[0.01, 10]`; `source` draws one of the other three
  channels relative to the picked edge and writes it; the Lamarckian flag
  is set to the picked edge's new value. Members whose variant lacks the
  field are unchanged and consume no draw. `SetScope` writes the picked
  edge's new scope (the other value) to every selected member. The table
  is total: every (operator, variant) pair has one outcome.
- *Static creation* (F02 for `Ring`, F03 for `DirectionPair`): draw the
  shape's parameters (ring or pair, kind, offset); every member is
  `Static` with scope `Edge`; `inherit_scale` is false; no learning draw
  exists before F04. Members and scale are born 0.
- *Uniform learned creation* (F04): the shape draws, then one mode drawn
  uniformly over `Static`, `Hebbian` and `RewardModulated`, its fields
  drawn once with the laws above, one scope coin (`gen_bool(0.5)`) and one
  `inherit_scale` coin, written to every member.
- *Randomized creation* (F05): the shape draws, then per member in genome
  order an independent mode, fields and scope; one `inherit_scale` coin.
- *Shape operators* (F02): one projection-equivalent draw each, as
  `projection.rs:44-46` documents today.

**Refinement disposition.** `RefineHeritableStructure`'s `GroupKey::Ring`
inference becomes redundant and, left alone, would double-step materialized
members. F02 excludes grouped edges from inference. Its `GroupKey::Slots` key
(one field across the four ranked slots into one sink) is keyed today on the
three nearby-creature families and their widths; F03 re-keys it on the
merged family (`field = channel mod 9`, `slot = channel div 9`) so the
default-on mechanism keeps working across the merge. F05 decides, with a
reading of how often it fires, whether `Slots` stays an inferred
correspondence, becomes a shape, or is dropped. Nothing default-on is
deleted or broken silently.

**The creature-sensor family.** `Nearest1` spans three input references
today (`Core`, `Vitals`, `Identity`), so "add Nearest1 as one unit" is a
cross-family bundle. **Decision (user, 2026-10-13): merge** the three into
one `NearbyCreature` family (4 slots × 9 fields = 36 sub-values; the
sampler's 27 buckets become 25; labels, census, telemetry gauges and the
inspector follow). No backward compatibility, one family per thing sensed,
and a slot's pair and scalars arrive from one declaration. F03 owns the
merged layout as the shared input contract for both backends.

**Out of scope.** The VM backend (no edges); `Drive` sinks as group targets;
compute-input edges as members; founder rewiring (its four cardinal taxis
edges stay independent edges); the unbounded steal amount (carried finding
5 of the program 1 archive).

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
spec owner of F04 should read Gerstner 2018 and Miconi 2018 in full.

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

Each boundary owns one thing. Three concerns that today share code are
separated on purpose: *structural editing* of the genome, *birth
correspondence* (which child edge descends from which parent edge, used
once at reproduction), and *learned-state construction* (building the
child's runtime values). The win the user asked for is concrete: today
about twelve mutation sites keep the birth payload aligned by hand, three
readers infer structure the genome will now store, and one hidden field
exists for one caller. After the refactor each has one owner.

| Boundary | Responsibility | Today |
| --- | --- | --- |
| `genome::graph::edge` | `Edge`, `Learning`, `Scope`; validation (finite weight, group index in range, scope only in a group, member fits its shape) | `GraphEdge` in `cgp.rs` |
| `genome::graph::group` | `Group`, `Shape`, the compass pattern, pair metadata read from the sensor layout, `genomic_weight`, membership validation | `RingProjection` plus inference in `refinement.rs`, `ring_rules.rs`, `drives.rs` |
| `genome::graph::container` | The only API that adds, copies, removes, relocates, retargets, re-weights or detaches edges and groups; emits **edit records** (below); owns no runtime or birth state | ~12 hand-maintained alignment sites in `operators.rs` and `cgp.rs` |
| `simulation::reproduction` | Birth correspondence: captures the parent's learned values per edge and group before mutation, folds the published edit records (graph and mesh level) into **seed values** per child edge and group under the child's post-mutation flags, and hands the seeds to the runtime | `cgp_reproduction.rs` and `birth_weights` on the genome |
| `runtime::graph::learning` | A constructor from seeds; learned own weights, learned scales and traces over both surfaces; evaluation-time samples; Hebbian and reward updates; scope routing; clamps; no knowledge of mutation or edit records | `runtime/plasticity/*` keyed by compute node |
| `runtime::graph::effects` | Sink sums over applied weights; records per-edge samples; no projection special case | `effects.rs:57-99, 133-171` |
| `runtime::graph::mask` | The observation-only read and learning mask the lab and the input-use readings pass | genome rewrites and `ablated_channels` |
| `mutation::graph::edge_ops`, `group_ops` | The two layers and one scope-draw helper; every structural change goes through the container API; the catalog triplet | `operators.rs`, `hebbian.rs`, `projection.rs`, `refinement.rs`, `recruitment.rs` |
| `analysis` and `census` | Explicit membership read from the genome (plasticity carriers, sensor census "wired" through members); derived wiring analyses (ring rules through edges, bus paths and groups; drive fan-outs) keep their own definitions over members and genomic weights | Inference in three files; `ablated_channels` |
| Transport and inspector | Serde shape; the creature endpoint adds per-edge applied weights and learned values as derived fields; `types/genome.ts`; an edge-group list replaces `RingProjectionList`; the graph shows genomic and applied weights apart; a per-edge learning badge | Projections drawn as a list, raw `edge.weight` displayed, no plasticity display |

**Edit records.** Today's inheritance semantics are richer than "what moved
where": a weight step or retarget drops the edge's inherited value
(`operators.rs:692-708`), while an identity split keeps correspondence
although it changes the edge's source (`:402-445`), and topology operators
copy whole backends, remove mesh nodes and replace a copied node's backend
(`mutation/topology/structural.rs:75-236`), with inheritance across them
tested at `simulation/actions/reproduction.rs:981`. The record kinds are
therefore:

| Record | Emitted by | Inheritance rule |
| --- | --- | --- |
| `Created { occurrence }` | add edge, create group, born-connected edge, recruitment | no seed |
| `Copied { from, to }` | copy edge bundle, copy or duplicate compute node, subgraph copy, identity split (the moved edge) | seed copied from `from` |
| `Removed { occurrence }` | remove edge, remove group, reference reindex | seed dropped |
| `Relocated { from, to }` | shift offset, retarget kind | seed follows the edge |
| `Invalidated { occurrence }` | weight step, retarget source, raw-field step | seed dropped |
| `Detached { occurrence, group, pattern }` | detach, with the group's occurrence and the member's pattern value as operands | seed composed as §3 states from the member's and the group's *current* seed state: a component invalidated earlier in the batch uses its genomic fallback; nothing parental is resurrected; the operands make the composition possible when detaching the last member deletes the group |
| `ComputeInserted { index }`, `ComputeRemoved { index }` | add, split, remove or copy of a compute node | shifts every later compute-input surface; a removal also invalidates every edge sourced from the removed node, as today's dangling `ComputeNode(u16::MAX)` rule does (`cgp.rs:370-398`) |
| `NodeCopied { from, to }`, `NodeRemoved { node }`, `BackendReplaced { node }` | topology operators | mesh-level correspondence, as the existing tests require |

Two rules today's operators imply are kept: a retarget that draws the
edge's existing source changes nothing and invalidates nothing
(`operators.rs:685-689`), and a weight step always invalidates
(`:699-708`). An occurrence is `(mesh node, surface, index)` for an edge
and `(mesh node, group index)` for a group. **Addressing is sequential:**
each record names positions as they stand when it is emitted, and
reproduction replays the batch in order to carry a parent occurrence to its
final child position; a relocation of several members is an ordered
sequence of `Relocated` records, each addressed after the previous one
applied. Records are event-local: the engine snapshots the genome and
restores it on operator failure or parseability rejection
(`mutation/engine/mod.rs:647-661`), so an event's batch is published only
after the event validates and is discarded with the rollback; a regression
covers a rejected event followed by an applied one. Reproduction folds the
published batches into seeds; the runtime's constructor consumes seeds and
nothing else. The graph container owns no topology: mesh-level records come
from the topology operators through the same event batch.

The three readers that treat projections differently today (§2) get an
explicit expectation, not a before/after equality: after F02 a motor node
whose only votes come from a group is a voting node in `ladder/track.rs`, a
group's channels are on the vote surface in `discovery.rs`, and the frontend
mesh semantics count members as action writers. Readers the migration
should not change (sensor census "wired", drive fan-outs on independent
edges) are pinned separately on fixtures without groups.

**Determinism and pins.** Any field on `Edge` changes the derived `Debug`
and so every genome identity and fingerprint; any catalog change remaps every
Graph draw; the lab's `GENOME_FORMAT` bumps. The repo has done this before
(b20abdff re-pinned 21 files). Each feature predeclares its re-pins.

## 8. Contracts (placed 2026-10-13; the track rows are authoritative)

Order: F01, F02, F03, F04, F05. F03 and F04 are independent after F02; F03
runs first because it answers the question that started this work. F01 is
behavior-preserving infrastructure, cut out so that the representation
change in F02 lands on a container, correspondence and runtime seam that
already closes with the existing inheritance tests green.

- **T25.F01 — Edge Container, Edit Records and Birth Seeds.**
  - Goal: Development separated from heredity, as a cell's machinery for
    building synapses is separate from what the egg carries: one owner edits
    the graph, one turns a parent's learned state into what a child is born
    with, and one builds the running brain from it, with no behavior change.
  - Scope: `genome::graph::container` as the only edge-editing API, with
    every mutation operator and `cgp.rs` walker routed through it; the edit
    record kinds and event-local batches of §7, including the mesh-level
    records from topology operators; `simulation::reproduction` folding
    batches into seeds; a runtime constructor from seeds; `birth_weights`
    removed from the genome. Edges keep `{source, weight}`; no group, no
    per-edge learning, no accounting change.
  - Verification: gate (no trajectory change is expected: the deterministic
    outputs of the gate worlds, the reproducibility test and every
    inheritance test are identical before and after); mutation gate run;
    slow tests run. Predeclared re-pins: none; any pin that moves is a
    defect.
  - Readings: identical deterministic gate output; the existing Lamarckian
    inheritance tests (`reproduction.rs:981` and the `cgp_reproduction`
    tests) pass unchanged; a regression for a rejected event followed by an
    applied one; the count of hand-maintained alignment sites removed.
- **T25.F02 — Edge Groups Replace Ring Projections.**
  - Goal: One developmental program wires a whole map (Eph/ephrin gradients,
    central-complex columns): a ring projection is a group of eight real
    synapses with one shared scale, and any one synapse can later be changed
    or lost on its own.
  - Depends on: T25.F01.
  - Scope: `Edge {group}` and `Group {shape: Ring, scale}`; membership
    validation; `genomic_weight` and the runtime resolver (learned values
    still per node for now); atomic group removal and reindex through the
    container; migration of `RingProjection` to `Ring` groups with eight
    born members; the five projection operators become `CreateRingGroup`
    (static creation, §4), `AlterGroupScale`, `ShiftGroupOffset`,
    `RetargetGroupKind` and `RemoveGroup`, with member relocation; retarget,
    raw-field and split operators detach a member first; refinement excludes
    grouped edges; recruitment's ring path emits a group; the observation-only
    mask replaces `ablated_channels` and genome rewrites in both ablation
    paths and `expressed()`; readers (`ring_rules`, `drives`,
    `sensor_census`, annotations, `live_circuit`, lab track, `discovery.rs`)
    read members; the identity invariant; the accounting contract (§3) and
    the founder anchor re-pin; the group operators' catalog, applicability,
    weight, dispatch and telemetry entries; serde, server derived fields,
    frontend types and the edge-group list. Learning untouched.
  - Verification: gate + goal (the change alters production cost and
    operators, the workflow's `gate + goal` row); mutation gate run; slow
    tests run. Predeclared re-pins: genome identity, `[Self; 61]` and
    projection catalog pins, the recruitment hash, lab `GENOME_FORMAT`, the
    founder size anchor, genome-size and complexity readings.
  - Readings: a fixed fixture genome with a ring projection and its
    migrated group produce identical vote vectors on the same inputs; the
    three readers' post-migration expectations (§7); goal indicators,
    genome size, functional complexity, births and gestation against the
    series (`Δgenome_size = 8P − S`, §3).
- **T25.F03 — Direction-Pair Groups.**
  - Goal: The sensory map sits in register with the motor map (superior
    colliculus): a creature's sensed offset to a thing, food, a barrier, a
    crowd or another creature, can reach all eight direction votes of one
    kind as one silent gene, and its plain facts reach that kind's drive.
  - Depends on: T25.F02.
  - Scope: `Shape::DirectionPair`, the compass pattern and pair metadata on
    the sensor layout; `CreateDirectionPairGroup` (static creation);
    recruitment's pair case with one drawn slot and scalar drive edges; the
    `NearbyCreature` family merge (user decision) as the shared input
    contract, with the sampler, labels, census, telemetry gauges and
    inspector following; the `Slots` refinement key re-keyed on the merged
    layout; inspector rows; the new operator's catalog, applicability,
    weight, dispatch and telemetry entries.
  - Verification: gate + goal-long (40,000 ticks): whether pair groups are
    retained and whether their scales move off 0 is visible only past
    20,000 ticks; mutation gate run; slow tests run.
  - Readings: pair groups present at the final tick that were born at least
    N ticks earlier (retention across an observation interval, not a count
    that supply alone can raise), by family and kind, against the number
    created; the sign and offset distribution of those groups against the
    recorded creation draws; blocked-move and steal-hit fractions of
    carriers against non-carriers; harmful-per-birth not up.
- **T25.F04 — Learning Per Edge and Group.**
  - Goal: Synapse-type-specific plasticity with three-factor traces: each
    synapse carries whether it is fixed, Hebbian or reward-modulated, which
    reward it listens to and how long its trace lasts; a synapse built by a
    group program can learn for the whole group or for itself, and a change
    to such an attribute lands on one synapse or on every synapse its group
    built.
  - Depends on: T25.F02.
  - Scope: `Learning` (with `clamp`) and `Scope` on `Edge`, `inherit_scale`
    on `Group`; runtime state over both surfaces and learned scales, sparse
    (allocated only for learning edges and groups); evaluation-time samples
    and updates after effects; sink edges learn; scope routing and summed
    scale credit; seeds for sink edges and groups through F01's records;
    `PlasticityConfig` removed from compute nodes; the nine node operators
    become the edge attribute operators with the scope draw and the
    transition table (§4); uniform learned creation for both shapes; the
    replaced operators' catalog, applicability, weight, dispatch
    (`mutation/graph/mod.rs:325+`) and telemetry entries;
    `plasticity_census`, telemetry gauges and the dashboards check follow.
    Plasticity update cost stays 0.
  - Verification: gate + goal; mutation gate run; slow tests run.
    Predeclare CPU and memory for the per-tick cost of sample recording and
    learning over sink edges on learning creatures, and no cost on
    non-learning graphs (T24 ceiling).
  - Readings: learning carriers by mode, scope and surface; learned-weight
    drift by surface; sampled clamp occupancy (the share of learned values
    at their clamp at the snapshot tick) by surface; goal indicators not
    down.
- **T25.F05 — Group Creation Forms, Detach and the Catalog.**
  - Goal: A mutation in a developmental gene changes every synapse it builds;
    a mutation at one synapse changes only that one. Group creation is
    uniform or randomized, a member can be detached, and the catalog is one
    consistent set of edge and group operators.
  - Depends on: T25.F03, T25.F04.
  - Scope: randomized creation for both shapes; `DetachEdge`; whether a
    fitting `AddGraphEdge` joins a group; the `Slots` refinement disposition
    with its reading; its own operators' catalog, applicability, weight,
    dispatch and telemetry entries, and a final reconciliation of the
    catalog triplet's weights and tests across the track (each earlier
    feature integrates the operators it introduces).
  - Verification: gate + goal (operator changes, the workflow's `gate +
    goal` row); mutation gate run; slow tests run.
  - Readings: operator funnel by layer; share of attribute changes applied at
    group scope; harmful-per-birth not up.

## 9. Remaining uncertainty

- Whether evolution picks up pair groups at all; the goal-long reading of
  F03 is the test, and failure returns the question to sector rings
  (option F) or to a learning route.
- Size and complexity accounting: the uniform rule (§3) touches every
  genome's carrying cost, supply and gestation and the founder anchor;
  F02's goal reading watches the consequences.
- Vote-edge Hebbian runaway, allowed by decision; F04 reads clamp
  occupancy.
- Hot path: F02 moves projection evaluation into edge traversal with one
  indexed lookup per member; F04 adds sample recording and sink-edge
  learning. Each predeclares its own CPU and memory.
- The `Slots` refinement key is an open decision for F05.
- Whether F01's behavior-preserving cut holds exactly: any pin that moves
  under F01 is a defect, not a re-pin.

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
- Codex Astra adversarial review, rounds 1 and 2, 2026-10-13: `docs/progress/readings/t25-design-review.md`
