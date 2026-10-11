# T16 — Cost and Cue Fidelity

**Status**: In Progress
**Last updated**: 2026-10-10
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Every energy charge a creature pays follows an effort it actually spent, and
every cue a creature senses is something its neighbor actually carries. The
[antipattern review](../strategy/antipattern-review-2026-09-16.md) found three
places where Petri departs from that and from the systems it is measured
against: a failed action is charged a penalty on top of its own cost with no
selective effect (31% of the sampled population's action energy), a parent
below the reproduction threshold pays the full replication cost and spawns
nothing, and the identity bank reports founder ancestry that no body carries.
Afterward, selection acts on what creatures do and perceive, not on a tax they
cannot yet avoid or a record they cannot see.

## Track Success Criteria

- [x] Every committed action costs at least its effort, success or not, and
  no action costs less than idling (`NoOp`); a failed move, eat, reproduce or
  steal costs exactly what the attempt cost; `failed_action_penalty` and its
  startup ramp are gone from the engine, the config, the panel, and the
  reference specs, and the penalized-failure share of action energy in the
  goal reports is the effort cost alone.
- [x] A reproduce attempt that fails its energy or transfer gate pays the
  attempt's effort and never the replication cost; a successful birth pays
  the attempt's effort plus what it pays today. (Amended 2026-10-10 from
  "leaves the parent's energy unchanged" at the user's direction: every
  action has at least some cost.)
- [ ] `NearbyCreatureIdentity` reports only cues a neighbor carries (tag
  identity, tag affinity, phenotype similarity); no sensor reads `lineage_id`.
- [ ] Each change is read on the gate and goal profiles with the survey's
  paired-perturbation probe alongside the T11.F14 readings, so the avoidance and
  seeking fractions of T11.F21 are taken on a substrate where the penalty is not
  the confound.

## Executable Features

- [x] **T16.F01 — Gate Before Charge in Reproduction** — Depends on: None
  - Goal: An animal that cannot afford a litter does not conceive. The reproduce energy gate (`energy - cost >= min_reproduce_energy`) and the transfer feasibility check are evaluated before any charge is taken, so a rejected attempt costs the parent nothing and a successful birth costs what it costs today; a TDD test on the `RejectedEnergyConstraints` path asserts zero energy change. Evidence: [antipattern review](../strategy/antipattern-review-2026-09-16.md) Section 3 (109 rejections at -10.8 each against 315 births in the sampled logs).
- [x] **T16.F02 — Effort-Priced Actions** — Depends on: T16.F01
  - Goal: Every act costs its effort, and failing costs nothing extra. Analog: a muscle burns energy whether or not the act succeeds (Polyworld charges activation, not outcome), and a courtship is spent whether or not a mate results (Wang et al. 2025). Every committed action (`Move`, `Eat`, `Reproduce`, `StealEnergy`, `NoOp`) charges a nonzero effort on every attempt, whatever the outcome, and none charges less than `NoOp`: the `eat_cost` default rises from 0; a reproduce attempt charges a new attempt effort before the gates (the replication cost is still charged only on a birth, in T16.F01's order); `StealEnergy` charges at least the floor when its requested amount is zero. A blocked move, an eat on an empty cell and a rejected reproduce or steal then charge only that effort; `failed_action_penalty`, its ramp, and their config, panel, and spec surface are retired. The spec predeclares each effort value. Predeclared readings: the failed-action share of action energy in the goal reports falls to the effort cost; the blocked-move fraction and the fraction of reproduce attempts rejected on parent energy are read before and after; the gate and goal trajectories move and an epoch re-pin is expected and recorded. Evidence: review Section 2 (7,196 of 23,460 action energy on penalized failures with 0 of 12,501 avoidances); the [action-choice pressure note](../strategy/action-choice-pressure-research-2026-10-09.md) Section 2 (in one live viewport at a 0.02 penalty, reproduce was a third of logged actions and 68% of attempts fell under the energy gate the founder respects).
- [ ] **T16.F03 — Phenotypic Kin Recognition** — Depends on: None
  - Goal: Kin are recognized by cues they carry (a tag, a color, a scent), not by an ancestry record. The `lineage_match` field of `NearbyCreatureIdentity` no longer reads `lineage_id`; the identity bank keeps its width so existing genomes stay valid, and the slot reports tag identity (`kin_tag` equality) beside the existing `kin_affinity` and `phenotype_similarity`. Predeclared: the identity block of any report that carries the field is re-pinned; the sensor census and the survey's identity perturbation are read before and after. Evidence: review Section 4; precedent Riolo, Cohen and Axelrod 2001 and Polyworld's color cue.

## Notes for AI Agents

- Origin, 2026-09-16: created at the user's direction from the
  [antipattern review](../strategy/antipattern-review-2026-09-16.md), which
  measured each cost on the live survey's 400 creatures and checked each
  pattern against Avida's `avida.cfg`, Polyworld (Yaeger 1994), Huang and
  Ontañón 2022, and Riolo, Cohen and Axelrod 2001. The review's findings 4 and
  5 (multi-action ticks, priority bid) were judged designs to keep and are not
  in this track.
- Order: T16.F01 is bug-sized and first; T16.F02 depends on it only so the
  reproduce path is charged correctly before the penalty is removed from it.
  T16.F03 is independent.
- These are mechanism features under the natural-analog rule; each goal line
  names its analog. None adds a sensor, an operator, or an assay. Readings use
  the T11.F14 battery and the survey's paired-perturbation probe (the survey
  note's Appendix B), not a new instrument.
- T16.F02 changes production trajectories and closes under the epoch re-pin
  rule. T16.F01 moved the gate profile: creatures there do fail the energy
  gate (the founder's node-0 threshold is `EnergyCurrent >= 30` while the
  engine gate is `energy - cost >= 30`, and mutated descendants' thresholds
  reach it too), so the charge-first engine taxed those attempts and the
  T16.F01 gate run differs from T11.F22's (largest delta `plasticity_updates`
  −4.57%, every level ok; recorded as a predeclaration miss in its spec).
  T16.F03 is expected to leave the founder gate unchanged (no founder node
  reads the identity bank), which its closure report verifies rather than
  assumes.
- Amended and placed, 2026-10-10 (user decision): T16.F02 now prices every
  committed action at its effort with `NoOp` as the floor, adds a reproduce
  attempt effort charged before the gates, and is the first new start in the
  priority order, ahead of T19.F07 (a failed step ends the tick's remaining
  plan), which depends on it so a blind step left at the end of a plan still
  costs its effort. The two run as features without a lab run; the parked
  [experiment plan](../strategy/action-choice-pressure-experiment-plan-2026-10-09.md)
  is superseded and its readings moved into the rows. A stale spec draft from
  2026-09-19 is on branch `codex/t16-f02`; it predates T19 and this amendment.
  T16.F03 remains unplaced.
- Effort-cost defaults, 2026-10-10 (user decision, not a roadmap feature):
  after goal-profile sweeps on T16.F02's closed code, the defaults became
  `noop_cost` 0.4 (the floor), `reproduce_attempt_cost` 1.0 and `move_cost`
  1.6; `eat_cost` and `reproduce_cost` are unchanged (the floor lifts eat to
  0.4). The founder reproduce margin is now guaranteed only before
  senescence (through age 130 at these economics); older founders may waste
  attempts. This supersedes the max-age claim of T16.F02 invariant 3a.
  Sweep readings and re-pins:
  [effort-cost-defaults readings](../progress/readings/effort-cost-defaults.md).
