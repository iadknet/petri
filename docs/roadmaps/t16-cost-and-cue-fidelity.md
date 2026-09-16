# T16 — Cost and Cue Fidelity

**Status**: Planned
**Last updated**: 2026-09-16
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

- [ ] A failed move, eat, or reproduce costs the creature exactly what the
  attempt cost; `failed_action_penalty` and its startup ramp are gone from the
  engine, the config, the panel, and the reference specs, and the penalized-failure
  share of action energy in the goal reports is the effort cost alone.
- [ ] A reproduce attempt that fails its energy or transfer gate leaves the
  parent's energy unchanged; a successful birth pays exactly what it pays today.
- [ ] `NearbyCreatureIdentity` reports only cues a neighbor carries (tag
  identity, tag affinity, phenotype similarity); no sensor reads `lineage_id`.
- [ ] Each change is read on the gate and goal profiles with the survey's
  paired-perturbation probe alongside the T11.F14 readings, so the avoidance and
  seeking fractions of T11.F21 are taken on a substrate where the penalty is not
  the confound.

## Executable Features

- [ ] **T16.F01 — Gate Before Charge in Reproduction** — Depends on: None
  - Goal: An animal that cannot afford a litter does not conceive. The reproduce energy gate (`energy - cost >= min_reproduce_energy`) and the transfer feasibility check are evaluated before any charge is taken, so a rejected attempt costs the parent nothing and a successful birth costs what it costs today; a TDD test on the `RejectedEnergyConstraints` path asserts zero energy change. Evidence: [antipattern review](../strategy/antipattern-review-2026-09-16.md) Section 3 (109 rejections at -10.8 each against 315 births in the sampled logs).
- [ ] **T16.F02 — Effort-Priced Failed Actions** — Depends on: T16.F01
  - Goal: Bumping into a wall costs the step, not more. A blocked move, an eat on an empty cell, and a reproduce into an occupied or barrier cell charge only the action's own cost; `failed_action_penalty`, its ramp, and their config, panel, and spec surface are retired. Predeclared: the failed-action share of action energy in the goal reports falls to the effort cost, the gate and goal trajectories move (an epoch re-pin is expected and recorded), and the blocked-move fraction is read before and after so a later reading of T11.F21's avoidance fraction is attributable. Evidence: review Section 2 (7,196 of 23,460 action energy on penalized failures with 0 of 12,501 avoidances).
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
  rule; T16.F01 and T16.F03 are expected to leave the founder gate unchanged
  (the founder never fails its energy gate in the gate profile, and no founder
  node reads the identity bank), which the closure report verifies rather than
  assumes.
- Priority: not in the order of new starts until the user places it.
