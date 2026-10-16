# T25 — Edge Groups and Per-Edge Learning

**Status**: In Progress
**Last updated**: 2026-10-13
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
- [ ] An edge costs one genome unit wherever it lives and a group record one; no wired-sink unit remains, and the founder anchor is re-pinned.
- [ ] Ring projections are edge groups: eight member edges with one shared scale, each member deletable, re-weightable and learnable on its own; every reader counts them as edges, reads membership from the genome, and no reader re-derives explicit membership (derived wiring analyses keep their own definitions over non-member edges).
- [ ] A direction-pair group born silent on a creature, food, barrier or crowd offset is retained and its scale moves off zero in a 40,000-tick goal-long run on at least one standard goal world.
- [ ] The three creature families are one `NearbyCreature` family declared once in `contracts::inputs`.
- [ ] Learning mode, reward source, trace decay and the Lamarckian flag are per-edge heritable attributes on both surfaces; vote edges can learn; no cost knob or lazy initializer remains.
- [ ] Scope and `inherit_scale` are heritable; a group-scope update is one pattern-weighted change to the shared scale, bounded once; every attribute mutation draws its scope.
- [ ] Group creation is uniform or randomized, a member can be detached, and the catalog is one consistent set of edge and group operators with a bijection test.

## Executable Features

- [x] **T25.F01 — Edge Container, Constructors and Birth Seeds** — Depends on: None
  - Goal: Development separated from heredity, as a cell's machinery for building synapses is separate from what the egg carries: one owner edits the graph, one captures what a child is born with, and one builds the running brain from it, with no behavior change.
- [x] **T25.F02 — Uniform Genome Size** — Depends on: T25.F01
  - Goal: Every synapse costs the same to carry wherever it sits, as a neuron pays per synapse and not per target: an edge is one unit on both surfaces, a wired sink costs nothing of itself, a ring gene costs its eight synapses, and the founder is re-anchored.
- [x] **T25.F03 — Ring Groups Replace Ring Projections** — Depends on: T25.F02
  - Goal: One developmental program wires a whole map (Eph/ephrin gradients, central-complex columns): a ring projection is a group of eight real synapses with one shared scale, and any one synapse can later be changed or lost on its own.
- [x] **T25.F04 — Direction-Pair Groups** — Depends on: T25.F03
  - Goal: The sensory map sits in register with the motor map (superior colliculus): a creature's sensed offset to food, a barrier, a crowd or another creature can reach all eight direction votes of one kind as one silent gene, and that thing's plain facts reach the kind's drive.
- [ ] **T25.F05 — The NearbyCreature Family** — Depends on: T25.F04
  - Goal: One sense of another creature, as an animal reads a neighbor's place, condition and kinship in one glance: the three creature families become one, so a slot's position, vitals and identity arrive from one declaration.
- [ ] **T25.F06 — Learning Per Edge** — Depends on: T25.F05
  - Goal: Synapse-type-specific plasticity with three-factor traces: each synapse carries whether it is fixed, Hebbian or reward-modulated, which reward it listens to and how long its trace lasts, and a synapse onto a vote can learn like one onto a neuron.
- [ ] **T25.F07 — Group-Scope Learning** — Depends on: T25.F06
  - Goal: A synapse built by a group program can learn for the whole group or for itself, and a change to such an attribute lands on one synapse or on every synapse its group built.
- [ ] **T25.F08 — Creation Forms, Detach and the Catalog** — Depends on: T25.F07
  - Goal: A mutation in a developmental gene changes every synapse it builds, a mutation at one synapse changes only that one: group creation is uniform or randomized, a member can be detached, and the catalog is one consistent set of edge and group operators.

## Notes for AI Agents

- Decisions (user, 2026-10-13): only geometry is built in (the compass
  pattern and the `(x, y)` pairs declared in `contracts::inputs`), never a
  computation; the bundle gene was rejected. Reward source and trace decay
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
- Open: F08 decides the `GroupKey::Slots` refinement disposition from the
  refinement-by-key readings F04 and F05 record.
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
  mutation gate and slow tests run on all eight. F03 and F06 land a
  text-only commit (genome identity) before their behavior commit.
