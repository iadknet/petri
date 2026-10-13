# T25 — Edge Groups and Per-Edge Learning

**Status**: Planned
**Last updated**: 2026-10-13
**Master**: [Program Roadmap](../roadmap.md)

## Goal

A graph edge is a domain object with its learning mode, and edges can belong
to a group that one gene builds and that evolves as a unit or edge by edge.
Ring projections become such groups; a sensed `(x, y)` offset can reach a
kind's eight direction votes as one silent gene; learning lives on edges and
groups with an evolvable reward source and trace decay; mutation has a group
layer and an edge layer. Design, boundaries, options and contracts:
[edge groups and learning design note](../strategy/edge-groups-and-learning-design-2026-10-13.md).

## Track Success Criteria

- [ ] One owner edits the graph, one folds a parent's learned state into a child's birth seeds, and one builds the running brain from them; no mutation operator keeps birth alignment by hand.
- [ ] Ring projections are edge groups: eight member edges with one shared scale, each member deletable, re-weightable and learnable on its own; every reader counts them as edges, reads membership from the genome, and no reader re-derives explicit membership (derived wiring analyses such as ring rules and drive fan-outs keep their own definitions).
- [ ] A direction-pair group born silent on a creature, food, barrier or crowd offset is retained and its scale moves off zero in a 40,000-tick goal-long run on at least one standard goal world.
- [ ] Learning mode, reward source, trace decay, Lamarckian flag and scope are per-edge and per-group heritable attributes; vote edges can learn; a group-scope update changes every member equally.
- [ ] Every attribute mutation draws its scope (F04); group creation is uniform or randomized, a member can be detached and a group removed (F02, F04, F05).

## Executable Features

- [ ] **T25.F01 — Edge Container, Edit Records and Birth Seeds** — Depends on: None
  - Goal: Development separated from heredity, as a cell's machinery for building synapses is separate from what the egg carries: one owner edits the graph, one turns a parent's learned state into what a child is born with, and one builds the running brain from it, with no behavior change.
- [ ] **T25.F02 — Edge Groups Replace Ring Projections** — Depends on: T25.F01
  - Goal: One developmental program wires a whole map (Eph/ephrin gradients, central-complex columns): a ring projection is a group of eight real synapses with one shared scale, and any one synapse can later be changed or lost on its own.
- [ ] **T25.F03 — Direction-Pair Groups** — Depends on: T25.F02
  - Goal: The sensory map sits in register with the motor map (superior colliculus): a creature's sensed offset to food, a barrier, a crowd or another creature can reach all eight direction votes of one kind as one silent gene, and that thing's plain facts reach the kind's drive.
- [ ] **T25.F04 — Learning Per Edge and Group** — Depends on: T25.F02
  - Goal: Synapse-type-specific plasticity with three-factor traces: each synapse carries whether it is fixed, Hebbian or reward-modulated, which reward it listens to and how long its trace lasts; a synapse a group program built can learn for the whole group or for itself, and a change to such an attribute lands on one synapse or on every synapse its group built.
- [ ] **T25.F05 — Group Creation Forms, Detach and the Catalog** — Depends on: T25.F03, T25.F04
  - Goal: A mutation in a developmental gene changes every synapse it builds, a mutation at one synapse changes only that one: group creation is uniform or randomized, a member can be detached, and the catalog is one consistent set of edge and group operators.

## Notes for AI Agents

- Decisions (user, 2026-10-13): only geometry is built in (the compass
  pattern and the eligible `(x, y)` pairs declared on the sensor layout),
  never a computation; the bundle gene was rejected. Reward source and trace
  decay are evolvable per edge or group; shared credit changes every member
  equally (one change to the shared scale); the plasticity update cost
  stays 0; group creation randomizes learning attributes, never the pattern.
- Decided (user, 2026-10-13): F03 merges `NearbyCreatureCore`, `Vitals` and
  `Identity` into one `NearbyCreature` family; an edge costs one genome unit
  wherever it lives and a group record one; `genome_size`'s per-wired-sink
  unit is dropped so that rule is uniform (a ring group costs 9 units) and
  the founder anchor is re-pinned (user confirmation, 2026-10-13). Open:
  F05 decides the `GroupKey::Slots` refinement disposition with a reading.
- F01 is behavior-preserving infrastructure: any pin that moves under it is
  a defect. From F02 on, genome identity and every hash or draw pin move
  with each feature and are predeclared.
- Verification plans are in the design note's §8: F01 gate; F02 gate +
  goal; F03 gate + goal-long; F04 gate + goal; F05 gate + goal; mutation
  gate and slow tests run on all five.
