# Sensor wiring as orthogonal mutations: bundles

**Date**: 2026-10-22
**Question**: Why do live creatures still wire single sensor fields (a lone
`Nearest2.red` into `Vote Reproduce[W]`) after T25 made groups, and why do
groups made by different mutations behave differently?
**Method**: the mutation code read on `main` at 1967d95c; the T25 design
sessions audited against `main` in
[the intent audit](../progress/readings/t25-intent-audit-2026-10-22.md);
one creature inspected in the user's running world (tick 11,239).

## Finding

Three mutation routes wire a sensor into votes. Each is a composite that
bundles "add a sensor" or "add a group" with its own private wiring rules,
and each answers the four questions a group creation has to answer
differently:

| Question | `InputRefAdd` (T20.F16) | `RecruitNeutralInput` (T20.F04) | `Add*Group` ×4 (T25.F03, F04, F08) |
|---|---|---|---|
| Shape from the sensor's geometry? | no: one random field | yes | yes |
| Fact members into `Drive K`? | no | yes (T25.F10) | no (F10 non-goal) |
| Learning mode drawn at birth? | no | no: always static | yes (F07 uniform, F08 randomized) |
| Born live or silent? | live, uniform on [-1, 1] | silent | silent |
| Destination | any of the 39 vote sinks | a vote sink; non-directed sinks get a whole-family bundle | a directed kind |
| Nominal first-selection probability per node-internal event | ~5.6% (VM declarations included) | ~0.4% | ~3.4%, only on a node already declaring the family |

The shares are nominal first-selection probabilities under the enabled,
unrestricted catalog (⅓ per node-internal domain × operator weight), not
measured success rates: the engine filters and redraws when an operator has
no site. Recruitment's columns describe a geometric family on a directed
destination under `WholeFamily`; its other cases build plain edges.

Nothing owns those answers. The most frequent route makes no group; the
group-making routes disagree on facts and learning. Code:
`mutation/input_ref/mod.rs` (`apply_add`, `connect_declared_input`),
`mutation/graph/recruitment.rs` (`Admission`, `Plan::append`),
`mutation/graph/group.rs` (`add_in_form`, `add_pair_in_form`,
`draw_learning`), the container's `add_group` and `add_pair_group`.

How it drifted: three features in three weeks each touched the route they
were framed around; the group operators were modelled on the old ring
projection operator (shape draws only); learning was added to them in
F07/F08 and facts to recruitment in F10, never to the other; the design
note's contract said "recruitment emits groups", so every review verified
that sentence. The user's premise that a sensor arrives as a group (T25
design session, 2026-10-13: "when an input like Nearest1 is added, all of
its dimensions are added and connected together") was never a decision
bullet. The intent audit's T5, T6, T8, T12, W2 and W5 rows are this one
inconsistency.

## Decision (user, 2026-10-22)

The user's requirements, in their words:

- "both live and silent should be types of mutations that can happen for
  newly added groups of sensors"; asked again after the review, separate
  operators, not a draw inside one operator.
- "Shouldn't groups be treated the same everywhere?"
- "scrap T25.F11 as it is and rewrite it as a refactor to fix this mess."
- "We should have a set of mutations that can happen, what they can happen
  to, and a randomized draw for those mutations."
- After the first review: an input with several fields and no geometry
  wires all its fields into one vote, not one random field.
- "Bundles should count as 1 gene just like groups do."
- "bundles are the parent that describes any group of related sensors.
  Ring and DirectionPair are just types of bundles. All three should behave
  the same, except Ring and DirectionPair get geometry-aware wiring. That
  should be the *only* difference."
- A single-field sensor is a bundle of one: "That feels cleaner and more
  extensible."
- After the third review: a sensor arrives whole, one bundle per slot or
  pair (multiple-choice answer).

## Model

### Objects

- **Sensor**: a declared input reference.
- **Edge**: the existing graph edge, unchanged: any source the graph allows
  (a compute node, an input field, a shared-memory slot) into any edge
  surface (a compute input or a sink).
- **Bundle**: one gene that wires a sensor's related fields together. It
  replaces T25's `Group` as the one concept (the type, operators, keys and
  readers are renamed). Every bundle has the same parts: its sensor and
  **field set** (the related fields it covers), its members, one shared
  scale, an own correction per member, learning attributes (mode, scope,
  `inherit_scale`, retention), and a **wiring**. A **member** is one wired
  edge occurrence, identified by its field and its destination coordinate
  (a `DirectionPair` has 12 direction members from two fields). A member's
  weight is `scale × pattern + own`.
- **Field set**: a whole sensor, or one slot or pair of it. A
  `NearbyCreature` sensor has four field sets (one per neighbour slot, 10
  fields each), an area summary two (one per pair, `dx`, `dy` and three
  facts), every other sensor one (all its fields).
- **Wiring**: the only thing that differs between bundles. It says where
  each member lands and its pattern value:

| Wiring | Sensors | Destination | Members land on | Pattern |
|---|---|---|---|---|
| `Plain` | every sensor without geometry: single-field sensors (introspection keys, `UpstreamSlot`, width-1 world keys) as a bundle of one; `ActionQueue` 12, `ActionVotes` 39, `PreviousPassVotes` 39, `CommitCounts` 5, `PreviousOutcome` | one vote sink | every field into that sink | 1 for every member |
| `Ring` | `NeighborFoodRing`, `NeighborBarrierRing`, `NeighborOccupiedRing` | a kind and an offset | cell `c` into the kind's direction `(c + offset) mod 8` | 1 for every member |
| `DirectionPair` | `NearbyCreature` (field set: one slot), the three area summaries (field set: one pair) | a kind and an offset | `dx`, `dy` into the kind's eight directions; the slot's or pair's facts into the kind's drive | the compass pattern for `dx`, `dy`; 1 for facts |

A **kind** is one directed action's nine vote sinks, eight directions and
its drive: `Move`, `Reproduce`, `StealEnergy`, `GiveEnergy`. The offset is
part of geometric wiring; `Plain` has none.

### Everything else is the same for every bundle

One law for every member of every bundle: composition (`scale × pattern +
own`, the pattern supplied by the wiring, the same for the static and the
runtime weight), learning, decay, inheritance of learned state, seeds,
detach (a detached member keeps its composed weight and composed seed),
member removal, source-changing edits and last-member removal. Every
fact-only branch in today's code is deleted, named in the spec, beginning
with `detach_at`'s fact case in `creature/genome/cgp/container.rs` (it
skips weight and seed composition) and `pattern_at`'s rejection of
non-direction members.

- **Cost**: one genome unit (T25.F10), whatever its wiring or size.
- **Birth law** (assistant proposal; the pattern is never randomized, as
  T25.F08 requires):

| Birth | Scale | Every member's own correction |
|---|---|---|
| live | uniform on [-1, 1] | 0 |
| silent | 0 | 0 |

- **Learning at birth**: the F07 uniform or F08 randomized form, then the
  `inherit_scale` coin, drawn inside the operator (assistant proposal).
- **Learning form draw** inside each creating operator: uniform or
  randomized with probability 1/2 each, as the four F08 creation operators'
  equal weights give today; then the `inherit_scale` coin.
- **Operator disposition** (Graph domain; the edge operators apply to
  bundle members as they do to group members today):

| Today | After F11 |
|---|---|
| `AlterGraphEdgeWeight`, `RetargetGraphEdge`, `RemoveGraphEdge`, `GraphRawFieldMutation` | kept; on a member they act on its own correction, its source (a source change detaches it first) and its removal, by the one member law |
| `EnableHebbian`, `DisableHebbian`, `MutateHebbianRule`, `MutateHebbianRate`, `ToggleHebbianLamarckian`, `EnableRewardModulation`, `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay`, `SetScope` | kept; scope coin as F07 |
| `EnableRetention`, `DisableRetention`, `MutateRetention` (member) and `EnableGroupRetention`, `DisableGroupRetention`, `MutateGroupRetention` | kept, renamed `…BundleRetention`; member and bundle retention stay distinct |
| `AlterGroupScale`, `RemoveGroup`, `DetachGraphEdge` | kept, renamed to bundle; every wiring |
| `RetargetGroupKind` | generalized to retarget destination: a sink for `Plain`, a kind for geometric wiring, one feasibility rule |
| `ShiftGroupOffset` | kept for wirings with an offset (geometry-aware wiring) |
| `AddRingGroup`, `AddDirectionPairGroup`, `AddRandomizedRingGroup`, `AddRandomizedDirectionPairGroup`, `RecruitNeutralInput` | removed |
| `InputRefAdd` (InputRef domain) | removed |
| — | new: `AddSensorLive`, `AddSensorSilent` (InputRef domain), `AddBundleLive`, `AddBundleSilent` (Graph domain) |
| `CopyEdgeBundle` | kept, renamed (it copies a compute node's input edges, not a bundle) so the word means one thing |
| `AddInternalGraphNode`, `RemoveInternalGraphNode`, `SwapGraphOperator`, `MutateGraphOperatorParam`, `CopyInternalNode`, `CopySubgraph`, `AddGraphEdge` | unchanged |

  No `inherit_scale` edit operator is added: it stays a birth draw, as
  today.

### Mutations

| Mutation | Applies to | Draws |
|---|---|---|
| `AddSensorLive` | any node | the sensor (the existing 30-bucket sampler); on a Graph node one bundle per field set, each with its own destination draw and the live birth law (`NearbyCreature` makes 4, an area summary 2, every other sensor 1); on a VM node a declaration only, as today |
| `AddSensorSilent` | any node | as live, born silent |
| `AddBundleLive` | a Graph node declaring a sensor with a field set whose wiring can be hosted | the sensor, one field set, the destination (and offset for geometric wiring), the live birth law |
| `AddBundleSilent` | as `AddBundleLive` | as live, born silent |
| `AddGraphEdge` | unchanged: today's source and surface domains and law | unchanged |

- Every sensor addition on a Graph node makes bundles; a single-field sensor is a `Plain`
  bundle of one (user, 2026-10-22). Its one weight is `scale + own`, so it
  carries one parameter more than a plain edge and costs one unit, as the
  edge it replaces did. `AddGraphEdge` stays the way to add a lone edge.
- **Deleted** (assistant proposal): `RecruitNeutralInput`, the
  `mutation.neutral_input_recruitment` config with its `Off`,
  `SingleChannel` and `WholeFamily` arms, and every consumer and fixture of
  them; the four `Add*Group` operators (replaced by the two `AddBundle*`);
  `InputRefAdd` (replaced by the two `AddSensor*`).
- **Recruitment's whole-family capabilities are kept through the common
  layers** (assistant proposal, by the maximum-flexibility rule; round 5
  of the review found no user intent to drop them): the 8-slot upstream
  bus and each 16-slot shared-memory bank (current and previous) become
  sensor families in the descriptor, so `AddSensor*` and `AddBundle*` can
  wire one as a `Plain` bundle in one event, live or silent, like any
  other multi-field sensor; single upstream slots stay in the sampler.
  `SingleChannel`'s `ActionQueue` channel drawn over the full `u16` range
  is decided separately: the spec owner checks whether channels at or
  above the sampler's 12 carry data, keeps the window in the descriptor if
  they do, and drops it with that evidence if they do not.

### Consequences to accept

- **Facts move under the shared scale.** Today a `DirectionPair`'s facts are
  own-weight-only members with no scale (T25.F10). As bundle members they
  get pattern 1 under the bundle's scale, like every other member, so a
  scale change moves direction and facts together, and own corrections
  separate them.
- **A live `Plain` bundle starts with every field at the same weight**
  (the scale); own-correction mutations and learning differentiate them.
- **A sensor arrives whole** (user, 2026-10-22): adding `NearbyCreature`
  makes four bundles, one per neighbour, costing four genes; an area
  summary two. Each neighbour stays its own gene, so two neighbours'
  directions never share a scale. An area summary's two pairs (gradient
  and nearest) share its three facts (width 7 = 2 × 2 + 3,
  `contracts/inputs.rs`); each pair's field set includes them, so each of
  the two bundles has its own fact occurrences into its own kind's drive
  (the kinds may differ). Assistant decision, by the maximum-flexibility
  rule: no field is withheld from a bundle.

### Ownership

The **sensor descriptor** in the input contract (`contracts/inputs.rs`)
owns every sensor's fields, field sets and geometry, including the widths
of the non-world references that live in `mutation/compound.rs` today
(`ActionQueue`, `ActionVotes`, `PreviousPassVotes`, `CommitCounts`,
`PreviousOutcome`), which move there. The descriptor owns field identity,
field sets and geometry only; it never knows which action receives a
field. Mutation selects from it, the wiring maps it to destinations and
patterns, the container materializes and validates it; no other layer
rediscovers a sensor's layout. The upstream-bus and shared-memory
families define their descriptor-to-source addressing once, in the
descriptor: a memory-bank member reads through the existing
`GraphSource::SharedMemory` source, so no second memory-read model
appears.

The vote vocabulary (the 39 vote sinks: four directed kinds of nine, plus
`Eat`, `Terminate`, `Decide`, and the action parameters) lives in
`creature/genome/vote.rs` today. It is a brain I/O contract that the VM,
the Graph, decoding and readers share; the spec owner decides whether F11
moves it beside the sensor descriptor or a later row does. Graph-only
output surfaces (the bus, memory writes) stay in the graph domain.

F11 keeps today's learning representation (per-member `PlasticityConfig`
and `Scope`, group `inherit_scale` and `retention`), applied the same way
to every wiring; T25.F12 replaces that representation
([bundle learning note](bundle-learning-ownership-2026-10-22.md)).

The seam between sampling and structure (revised after an architecture
challenge, Codex Astra, 2026-10-22):
1. **Operators** (mutation): applicability and target selection, each
   operator's `applies_to` and its draws.
2. **Birth policy** (mutation, the stochastic half): asks the graph domain
   for the admissible plans for a declared field set (which wirings,
   destinations and offsets are legal on this node), samples one, and
   draws the live or silent birth law and the learning attributes. One
   policy serves `AddSensor*` and `AddBundle*`; when no plan is
   admissible, `AddSensor*` has no site for that sensor and draws again,
   never leaving an unwired declaration on a Graph node.
3. **Graph domain** (deterministic, no RNG): the wiring definitions, plan
   enumeration and legality, and the container that validates and applies
   the chosen plan atomically, keeping seed alignment as T25.F01 set it.
   Wiring belongs here, with the graph's developmental representation, not
   in the input contract.

The graph is the aggregate root: it stores every edge on its surfaces; a
bundle is a record, and a member is an edge whose membership links it to
that record, identified by its occurrence (surface and position). The
bundle never stores its members, so there is one structural owner. The
container keeps atomic edits and payload alignment together; pure wiring
functions (coordinates, pattern, destinations) and, in T25.F12, learning
resolution are extracted from it rather than spread across operators.

The `Wiring` enum (`Plain | Ring | DirectionPair`) is closed on purpose:
a finite, serializable extension point whose behavior lives in one
module; a trait or plugin hierarchy would add machinery without removing
the need to define mutation, persistence and validation per wiring.

The **wiring** owns the legal member coordinates, the pattern and the
destination mapping, and its destination requirements; creation,
retargeting, relocation, validation and duplicate detection all consume
that one definition. Today's scattered rules are retired, named in the
spec: `can_host_group`'s blanket requirement that every directed kind's
direction sinks exist (container.rs), the separate fact-hosting rule, and
the shape branches in validation and duplicate detection.

**Routes outside fresh mutation** keep their contracts, stated so they are
visible rather than exceptions: topology node and slice copies duplicate
existing bundles with no birth draws; `SwapNodeBackend` replaces a copy's
backend with a blank one and keeps its declarations unwired, which a later
`AddBundle*` or `AddGraphEdge` can wire; a VM node's declaration stays
declaration-only (VM nodes have no edges); the authored founder keeps its
hand-wired graph; test and lab fixtures build through the container.

Natural analogs: one developmental program wires a sensory map however the
gene that calls it was reached (retinotectal map formation, as T25.F03 and
F04 use); silent synapses beside transmitting ones (Isaac, Nicoll and
Malenka 1995; Liao, Hessler and Malinow 1995).

## Shape of the work

The final seams come first: the sensor descriptor, the wiring plan, the
graph edit transaction and the birth policy are defined in their final
places before any behavior moves, so the preservation stage does not
shape the abstraction (assistant proposal, after the architecture
challenge). Temporary old semantics stay internal to those seams and are
deleted at the stated transition. The whole-creature redesign (body,
inheritance, birth transaction) is not a prerequisite of this row.

Three commits (assistant proposal, after T25.F01, F03 and F06):
1. Extract the shared birth policy with every caller keeping its current
   answers and its exact draw order and consumption, including the birth
   offset; every pin holds, any moved pin is a defect.
2. Rename `Group` to `Bundle`, `Shape` to the wiring and `CopyEdgeBundle`,
   text only: genome identity digests, operator keys, telemetry and reader
   names move; every trajectory, population and draw-stream pin holds.
3. Switch to the model above and predeclare the re-pins: operator catalog,
   every Graph and InputRef draw, genome identity, the recruitment-paths
   readings, the runtime-config and mutation reference specs, both epochs.
   Commit 1's temporary per-caller answers are deleted here; none survives
   as a switch or config.

## Settled by earlier decisions

- A bundle costs one genome unit (T25.F10 for groups; the user extended it
  to every bundle on 2026-10-22).
- Only geometry is built in: the compass and ring patterns; `Plain` wiring
  has pattern 1 everywhere, so it builds in no computation; creation
  randomizes learning, never the pattern.
- No backward compatibility: genomes and pins move, predeclared.

## Open

- For the spec owner: operator weights after the change, stated rather than
  inherited (the nominal share of node-internal events that add a sensor, a
  bundle and an edge, live and silent, before and after; default: each
  live/silent pair splits its predecessors' weight evenly).
- For the spec owner, readings: creation events counted from applied
  outcomes by operator, wiring and learning form, replacing `pair_groups`'
  count of the retired `GraphAddDirectionPairGroup`; the inheritance
  witness in `simulation/actions/reproduction.rs`, which infers inherited
  pair groups by excluding two named creation operators, moves to applied
  outcomes or is retired, so a live newborn bundle never counts as
  inherited; censuses cover all three wirings, with ring and pair
  breakdowns beside the total; surviving bundles by
  birth form need a stored tag or are not read; scale reported as both the
  genomic and the learned runtime value. Bloat (bundles now arrive at the
  declaration rate) is read beside them; T27.F02's goal-long reads the
  long-horizon counts.
- Readings the audit asks for that this row does not own: `sensor_read`
  beside `sensor_wired` in the bench census (gap 3), and a by-family pair
  read in T27.F02's goal-long (gap 4).
