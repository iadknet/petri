# T25 — Edge Groups and Per-Edge Learning

**Status**: In Progress
**Last updated**: 2026-10-22
**Master**: [Program Roadmap](../roadmap.md)

## Goal

A graph edge is a domain object with its learning mode, and edges can belong
to a group that one gene builds and that evolves as a unit or edge by edge.
Ring projections become such groups; a sensed `(x, y)` offset can reach a
kind's eight direction votes as one silent gene; learning lives on edges and
groups with an evolvable reward source and trace decay; mutation has a group
layer and an edge layer. Design, boundaries, options and contracts:
[edge groups and learning design note](../strategy/edge-groups-and-learning-design-2026-10-13.md)
(version 3, after three Codex rounds and two rounds of nine Opus facet
reviews recorded in [the review readings](../progress/readings/t25-design-review.md)).

## Track Success Criteria

- [ ] One owner edits the graph, one captures a parent's learned state as a child's birth seeds, and one builds the running brain from them; no mutation operator keeps birth alignment by hand, and no struct literal builds an edge outside the constructors.
- [ ] No wired-sink unit remains and the founder anchor was re-pinned (F02); every edge outside a group costs one genome unit, and a group's cost follows F10's gene rule (the uniform one-unit-per-member rule was F02's outcome, superseded by F10 on 2026-10-20).
- [ ] Ring projections are edge groups: eight member edges with one shared scale, each member deletable, re-weightable and learnable on its own; every reader counts them as edges, reads membership from the genome, and no reader re-derives explicit membership (derived wiring analyses keep their own definitions over non-member edges).
- [ ] A direction-pair group born silent on a creature, food, barrier or crowd offset is retained and its scale moves off zero in a 40,000-tick goal-long run on at least one standard goal world.
- [ ] The three creature families are one `NearbyCreature` family declared once in `contracts::inputs`.
- [ ] Learning mode, reward source, trace decay and the Lamarckian flag are per-edge heritable attributes on both surfaces; vote edges can learn; no cost knob or lazy initializer remains.
- [ ] (Superseded by T25.F12 for the representation: `Scope` and `inherit_scale` give way to bundle `Learning`, member overrides and `credits`; the behavior this criterion asked for is kept.) Scope and `inherit_scale` are heritable; a group-scope update is one pattern-weighted change to the shared scale, bounded once; every attribute mutation draws its scope.
- [ ] Group creation is uniform or randomized, a member can be detached, and the catalog is one consistent set of edge and group operators with a bijection test.
- [ ] A group costs one genome unit however many members it has and however they have changed; a pair recruitment's direction members and drive facts belong to one group, so a recruitment costs one unit, not its silent edges; runtime learning never changes the size.
- [ ] Sensor wiring is a set of orthogonal mutations over one concept, the bundle (T25's group, renamed): every bundle has the same parts, cost, birth law, learning draw, member law (composition, learning, decay, seeds, detach) and edit operators, and differs only in its wiring (`Plain` into one sink, `Ring` and `DirectionPair` geometry-aware into a kind); `AddSensorLive`/`AddSensorSilent` wire every new sensor whole, one bundle per field set (a single-field sensor is a bundle of one), `AddBundleLive`/`AddBundleSilent` add a bundle to a declared sensor, through one birth policy; `RecruitNeutralInput`, its config and the four add-group operators are gone; the goal run counts creations by operator and wiring.
- [ ] Learning is one type: edges and bundles carry the same `Learning` (one `lamarckian` flag, one retention); every member's weight is `scale × pattern + modifier`; a bundle has two slots of the same `Learning` type, one for its scale ("learn as a group") and one as its members' default ("learn per edge"), either or both present; a member's policy is inherit (static modifier unless the member-default slot is present, then that slot's settings), static, or its own `Learning`, which always overrides; a member may also credit the scale; a learned scale's state survives without contributors; one learning layout, derived from the genome once at birth, resolves every member's policy and lists every learner for runtime, structure checks and the census; and the runtime updates every learnable number through one path with no scope branches.
- [ ] A learned edge weight and a learned group scale relax toward their birth values at a heritable per-edge or per-group retention, the founder stays at no decay, and the goal run reads the retention distribution beside the plasticity census.

## Executable Features

- [x] **T25.F01 — Edge Container, Constructors and Birth Seeds** — Depends on: None
  - Goal: Development separated from heredity, as a cell's machinery for building synapses is separate from what the egg carries: one owner edits the graph, one captures what a child is born with, and one builds the running brain from it, with no behavior change.
- [x] **T25.F02 — Uniform Genome Size** — Depends on: T25.F01
  - Goal: Every synapse costs the same to carry wherever it sits, as a neuron pays per synapse and not per target: an edge is one unit on both surfaces, a wired sink costs nothing of itself, a ring gene costs its eight synapses, and the founder is re-anchored.
- [x] **T25.F03 — Ring Groups Replace Ring Projections** — Depends on: T25.F02
  - Goal: One developmental program wires a whole map (Eph/ephrin gradients, central-complex columns): a ring projection is a group of eight real synapses with one shared scale, and any one synapse can later be changed or lost on its own.
- [x] **T25.F04 — Direction-Pair Groups** — Depends on: T25.F03
  - Goal: The sensory map sits in register with the motor map (superior colliculus): a creature's sensed offset to food, a barrier, a crowd or another creature can reach all eight direction votes of one kind as one silent gene, and that thing's plain facts reach the kind's drive.
- [x] **T25.F05 — The NearbyCreature Family** — Depends on: T25.F04
  - Goal: One sense of another creature, as an animal reads a neighbor's place, condition and kinship in one glance: the three creature families become one, so a slot's position, vitals and identity arrive from one declaration.
- [x] **T25.F06 — Learning Per Edge** — Depends on: T25.F05
  - Goal: Synapse-type-specific plasticity with three-factor traces: each synapse carries whether it is fixed, Hebbian or reward-modulated, which reward it listens to and how long its trace lasts, and a synapse onto a vote can learn like one onto a neuron.
- [x] **T25.F07 — Group-Scope Learning** — Depends on: T25.F06
  - Goal: A synapse built by a group program can learn for the whole group or for itself, and a change to such an attribute lands on one synapse or on every synapse its group built.
- [x] **T25.F08 — Creation Forms, Detach and the Catalog** — Depends on: T25.F07
  - Goal: A mutation in a developmental gene changes every synapse it builds, a mutation at one synapse changes only that one: group creation is uniform or randomized, a member can be detached, and the catalog is one consistent set of edge and group operators.
- [x] **T25.F09 — Learned-Weight Decay** — Depends on: T25.F08
  - Goal: Early LTP decays back to baseline: a learned synapse and a learned group scale relax toward the value they were born with at a heritable rate, so learning that is no longer reinforced fades instead of wandering.

- [x] **T25.F10 — Group Genes Cost as Genes** — Depends on: T25.F09
  - Goal: The genomic bottleneck: one developmental gene wires many synapses, so a creature pays to carry and copy the gene, not every synapse it built.
- [ ] **T25.F11 — Sensor Wiring as Orthogonal Mutations** — Depends on: T25.F10
  - Goal: One developmental program wires a sensory map however the gene that calls it was reached, and a new synapse may transmit from the start or wait silent until activity unsilences it: a sensor arrives whole, one bundle per field set, a bundle can wrap a sensor already there, and every bundle is built and edited one way, live or silent, with geometry-aware wiring as the only difference.
- [ ] **T25.F12 — Learning as One Type on Edges and Bundles** — Depends on: T25.F11
  - Goal: Synapses and the developmental programs that build them learn by one machinery, as one family of plasticity rules serves every synapse type: a bundle owns its learning and whether it learns as a group or per edge, each member inherits it, stays static, or overrides it with its own, may credit the group as well, and one learner serves every learnable number.

## Notes for AI Agents

- F11 pays known debts G1, G13 and G14's inheritance witness in
  [`docs/architecture.md`](../architecture.md), and moves the brain I/O
  vocabulary its sensor descriptor needs (vote and outcome counts,
  `OUTPUT_SLOT_COUNT`) into contracts (G2), so the descriptor depends on
  nothing above it. It owns the node's declaration edits: adding a sensor
  applies the declaration and its bundles together, and pruning or removing
  a declaration repairs its uses in the same edit
  (`mutation/input_ref/mod.rs:338,366`); unused declarations and the
  parseability gate's soft defaults stay supported (user, 2026-10-22,
  folded from T30). F12 pays G3.
- F11 acceptance (2026-10-22, in-domain guard): exercise sensor and bundle
  creation through the existing Graph and InputRef catalogs on applicable
  fixtures, checking descriptor-defined fields and facts, the requested
  live or silent law and the one learning-birth policy; every relevant
  operator must apply at least once, and whole-sensor wiring is a creation
  postcondition (removal and detach may leave partial structures).
- F12 acceptance (2026-10-22, in-domain guard): exercise each learning-edit
  operator on its applicable edge, member and bundle targets, checking
  broadcast behavior and unaffected settings; runtime construction, the
  census and structural classification agree with the genome-owned
  learning layout, including bundle-only learners and scales without
  contributors.
- Decisions (user, 2026-10-13): only geometry is built in (the compass
  pattern and the `(x, y)` pairs declared in `contracts::inputs`), never a
  computation; the "bundle gene" was rejected (a gene multiplying who a
  creature is by where it is; it does not prohibit T25.F11's bundles, which
  build in no computation). Reward source and trace decay
  are evolvable per edge or group; shared credit is one pattern-weighted
  change to the shared scale; the plasticity update cost is 0 and its
  machinery is deleted; group creation randomizes learning attributes, never
  the pattern; the three creature families merge; an edge costs one genome
  unit wherever it lives, the per-wired-sink unit is dropped and the founder
  anchor re-pinned (97 → 80).
- Decided (user, 2026-10-13): under uniform size the per-unit mutation
  rate becomes `0.485 / 80 = 0.0060625`, so the founder's supply is
  unchanged (the note's D14, option a). F02 lands the unit rule, the rate
  and the plasticity cost-chain deletion as three commits and runs one
  extra goal-long arm at rate 0.005 as the rate's counterfactual (D25).
- Decided (user, 2026-10-19): F09 adds passive forgetting after F08, from
  the [learned-weight decay research note](../strategy/weight-decay-research-2026-10-19.md)
  (Section 5 is the contract sketch, amended after its plan-design review in
  [the review readings](../progress/readings/t25-design-review.md)): once per
  world tick each stored learned quantity relaxes toward its own birth value,
  `x ← b + ρ·(x − b)`: an edge's learned own weight (a member's own
  correction, never the composed weight) by a heritable per-edge `ρ`, a
  group's learned scale by a retention stored on the group (a deliberate
  revision, user 2026-10-19, of the 2026-10-13 ruling that groups own no
  learning configuration beyond `inherit_scale`); a scope-`Group`
  member has no own decay. `ρ` lies in `[0, 1]`, `ρ = 1` is an exact no-op,
  enable and disable operators move it on and off 1 and a step mutates
  `1 − ρ` in log space. Learned state is clamped at construction so the
  birth value is the start value (user decision, option A), a predeclared
  re-pin for out-of-clamp slots. Verification: viability first, gate + goal;
  the goal run reads the `ρ` distribution descriptively and does not settle
  long-horizon selection. Decay toward the genomic weight, toward zero and an
  energy cost on the deviation are rejected there.
- Decided (user, 2026-10-20): F10 revises F02's uniform rule ("an edge
  costs one genome unit wherever it lives") after the
  [interaction features design note](../strategy/interaction-features-design-2026-10-20.md)
  §8 showed that one slot recruitment under the eight-signal design adds 28
  silent edges, each paying mutation supply, carry, replication, gestation
  and the optional size-pressure gate. Contract sketch, after the
  [T26–T28 design review](../progress/readings/t26-t28-design-review.md)
  round 2:
  - **One unit per group (user, 2026-10-20, simplifying the round-3
    template rule).** A group record costs one unit, and its members cost
    nothing more, whatever their own corrections, learning configuration,
    scope or retention. A hole costs nothing; a detached member is an
    ordinary edge at one unit. Only genomic state counts: runtime learning
    and birth seeds never change the birth-cached size. No template or
    exception tracking exists. This is an explicit economic discount, not
    compression: members still store their own weights and settings (review
    round 5 finding 3; the user kept the rule with this label).
  - **Fact members.** A pair recruitment's drive facts (T25.F04) become a
    second member role of the same gene: one edge per fact field into
    `Drive K`, born at weight 0, no shared scale. F10 owns their
    validation, relocation, deletion, detach and seed handling inside the
    graph container, so a recruitment's 12 + 16 edges cost one unit
    (historical: a `NearbyCreature` slot now has eight facts, 12 + 8).
    Ordinary slot-fact wiring outside a recruitment is unchanged.
    Superseded by T25.F11: facts become ordinary bundle members with
    pattern 1 under the shared scale, and recruitment is removed.
  - **Intended consequence.** Supply is drawn from size before targets, and
    weight mutation picks one materialized edge, so a genome dominated by
    intact n-member genes draws about n+1 times fewer events for the same
    targets. That is the genomic-bottleneck outcome. The spec characterizes
    the resulting supply and target law rather than promising preserved
    member rates.
  - **Founder.** The founder has no groups, so its 80 units and the rate
    0.0060625 should not move. The spec remeasures them and recalibrates
    only if they do.
  - **Bloat risk.** Cheap silent groups may accumulate (the program-1 bloat
    finding; duplicate silent groups are allowed), and per-member variation
    inside a group is free to carry. Deferred: T27.F02's goal-long run reads
    group count, materialized-member count and members with nonzero own
    corrections, with compute and energy, naming every feature closed in
    between.
  - **Verification.** Viability first, gate + goal. T25's goal-long
    allowance is spent by F02 and F04.
- Decided (user, 2026-10-22): F11 is a refactor of sensor wiring, after
  the [intent audit](../progress/readings/t25-intent-audit-2026-10-22.md)
  found three composite routes answering the group-creation questions
  differently and the most frequent route making no group, the gap T25's
  own problem statement was about. User requirements: live and silent are
  separate mutation types; groups are treated the same everywhere; "a set
  of mutations that can happen, what they can happen to, and a randomized
  draw"; a multi-field input without geometry wires all its fields into one
  vote, as one gene like any group; bundles are the parent concept, Ring
  and DirectionPair are bundles with geometry-aware wiring, and that is the
  only difference. The proposed model (bundles, four new operators, one
  birth policy, recruitment and its config deleted), its assistant-proposed parts
  and the Codex review's changes are in the
  [sensor wiring note](../strategy/group-creation-unification-2026-10-22.md).
  Verification: viability first, gate + goal; slow tests run; no goal-long
  (T25's allowance is spent).
- Decided (user, 2026-10-27, "B"): in F11 the `ActionQueue` input width
  follows `max_actions_per_turn` (`3 ×` the normalized value, bounded by the
  `u16` address space) instead of staying a constant, reversing T19.F06's
  constant-width premise, after the Codex rounds upheld that a static width
  leaves queue slots unaddressable; the queue size reaches every width
  reader through a `SensorContext`, and no config cap is added.
- Decided (user, 2026-10-22): F12 follows F11, after the user asked whether
  sensors, sinks, edges and bundles keep clean layers around learning. The
  [bundle learning note](../strategy/bundle-learning-ownership-2026-10-22.md)
  found learning owned only by edges, a bundle scale borrowing its rule and
  clamp from members, paired flags (`lamarckian` / `inherit_scale`, two
  retentions) and scope branches across runtime, census and mutation. The
  user's model (2026-10-22): every member's weight is the bundle's weight
  plus its own evolvable modifier; a bundle learns as a group (its scale)
  or per edge (members without their own learning learn their modifier
  with the bundle's settings); an edge's own learning always overrides; a
  member may credit the bundle and learn itself at once; and, as a general
  rule, "bake in as few assumptions and constraints as possible and allow
  evolution maximum flexibility, while also maintaining clean abstraction
  layers". Both targets at once, an explicit static member policy and
  scale state surviving without contributors are assistant proposals under
  that rule. F12 supersedes member-owned scale-learning configuration, not
  only the names `Scope` and `inherit_scale`. Details, update law, detach
  table, commit shape and acceptance cases are in the note. Verification:
  viability first, gate + goal; slow tests run.
- Decided (F08, 2026-10-19): `Slots` deleted on nine empty post-F05
  refinement-by-key readings (F05, F06, F07 goal); `RefineHeritableStructure`,
  `mutation.structured_heritable_refinement` and the inferred-set code are
  gone; a `Slots` group shape would be a new row.
- The chain is linear by design (D10): a later feature edits the same
  operator catalog and draws as the one before it, so a rebase during a
  goal-long run would invalidate the reading.
- F01 is behavior-preserving: any pin that moves under it is a defect. From
  F02 on every feature predeclares its re-pins, one cause per commit; the
  note's §8 carries each feature's re-pin line, reference-spec line and the
  tests the implementer writes first.
- Verification plans (note §8): F01 gate; F02 gate + goal + goal-long plus
  the rate arm, viability first; F03 gate + goal, viability first
  (goal-long dropped, user 2026-10-15: at most once or twice per track);
  F04 gate + goal + goal-long, carrying F03's deferred 40,000-tick ring
  readings; F05 gate + goal; F06 gate + goal,
  viability first; F07 gate + goal, viability first; F08 gate + goal;
  F09 gate + goal, viability first; mutation gate and slow tests run on all nine. F03 and F06 land a
  text-only commit (genome identity) before their behavior commit.
