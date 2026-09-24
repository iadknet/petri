# T17 — Brain Boundary Evolvability

**Status**: In Progress
**Last updated**: 2026-09-23
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Energy- and age-based decisions and resource transfers use scales that the
brain's mutation operators can readily tune. The
[reproduction-collapse research note](../strategy/reproduction-collapse-research-2026-09-18.md)
(Section 8) identified raw energy and age inputs and raw energy transfers as
requiring scaling constructions, while new graph thresholds, constants,
and edge weights are drawn in [−1, 1] and parameter steps are ±0.1.
T17.F01 made offspring investment a fraction of the parent's post-cost
energy; T17.F02 normalized `EnergyCurrent`, `EnergyConsumedThisTick`, and
`AgeTicks` to [0, 1] and removed `Generation` from the input keyset.
The remaining work makes the steal amount a fraction and provides per-type
Eat votes when a third ordinary food type exists. T19's `HopsThisTick`,
`CommitCounts`, and `OffspringSuccess` retain their raw-count semantics;
action votes remain graded preferences, without a blanket unit-scale rule.

## Track Success Criteria

- [x] `EnergyCurrent`, `EnergyConsumedThisTick`, and `AgeTicks` reach the brain
  in [0, 1], and `Generation` is absent from the input keyset. T17.F02's scale
  conversion preserves founder behavior by construction, verified by the
  founder tests and the viability test. T19's `HopsThisTick`, `CommitCounts`,
  and `OffspringSuccess` remain raw counts, and action votes retain their
  graded preference semantics.
- [ ] The reproduce transfer and the steal amount are fractions in [0, 1] at
  the action boundary, and a founder attempts only births that physiology
  accepts.
- [ ] The T11.F14 battery and the survey's paired-perturbation probe are read
  before and after each feature, so the change in behavior-changing births,
  silent births, and Hebbian-carrier outcomes is attributable to it.

## Executable Features

- [x] **T17.F01 — Offspring Investment as a Fraction of Parent Energy** — Depends on: T16.F01
  - Goal: A parent gives its young a share of what it has, not a fixed ration. The reproduce action's energy-transfer meta becomes a fraction in [0, 1] of the parent's post-cost energy (Polyworld's `MateEnergyFraction`, a heritable share of current energy the world bounds to [0.2, 0.8]; capital breeders provision litters from their reserves), so a sensor routed into that slot is a condition-dependent investment rule. Physiology keeps the parent's `min_reproduce_energy` gate (T16.F01) and adds a minimum litter (a child born with less than `initial_energy` is not conceived, rejected before charge). The founder emits a constant fraction and its brain gate is set so every attempt it makes is accepted: the founder's reproduce branch ends with the terminal `ExecuteActionQueue`, so a refused attempt is a tick without eating, and a founder whose gate sits below what physiology accepts pins itself at its threshold (measured: 0 births in 2,000 ticks, with and without `failed_action_penalty`). Predeclared: births-based indicators and the goal trajectories move (epoch re-pin recorded); the founder's births per creature-tick on the gate profile are read before and after.
- [x] **T17.F02 — Unit-Scale Introspection** — Depends on: None
  - Goal: A creature feels how full, how tired, and how old it is as a share of itself, not in joules or ticks. `EnergyCurrent` and `EnergyConsumedThisTick` reach the brain as fractions of `max_energy`; `AgeTicks` as a saturating fraction of a configured reference span (T03.F07's lifespan is its later denominator); `Generation` is removed from the input key set or saturates the same way. The founder's gates are re-expressed exactly (32 → 0.16 of 200 since T17.F01; the age gate on the same span) so its behavior is identical by construction. Evidence: a new `Threshold` drawn in [−1, 1] against raw energy is always on; the founder's 30 needs ~700 ±0.1 steps to reach 100; Covariance with `pre` = 20–200 drives the weight to its clamp in one tick, which is the six Orchards Hebbian carriers; 14 of the 59 Orchards survivors had `Generation` (1–6) swapped in for energy or age as a constant gate. Predeclared: the T11.F14 changed/silent/dead readings and the goal trajectories move (epoch re-pin recorded); founder gate behavior unchanged.
- [ ] **T17.F03 — Steal Amount as a Fraction** — Depends on: T17.F01, T19.F04
  - Goal: A bite takes a share of the prey, not a fixed calorie count. The steal action's amount meta becomes a fraction in [0, 1] of the victim's current energy, capped by the existing config limit (Polyworld's attack depletion scales with the attacker's state and is capped). Predeclared: predation transfer per event in the goal reports is read before and after.
- [ ] **T17.F04 — Per-Type Eat Votes** — Depends on: T11.F21, T19.F04
  - Goal: Competing food preferences: one vote per food type lets a creature select what to eat directly from its sensory inputs when worlds offer more than two ordinary food types.

## Notes for AI Agents

- Origin, 2026-09-18: created at the user's direction from the
  [reproduction-collapse research note](../strategy/reproduction-collapse-research-2026-09-18.md).
  The user framed the reproduce transfer as a refactor from a flat amount to
  an evolvable fraction of parent energy (Section 6 of the note); a side
  audit of every brain↔world boundary value (Section 8) found the same unit
  mismatch on the introspective inputs and the steal amount, and the track
  groups them.
- What this track is not: famine and total-population-collapse protection
  (a physiological reserve, a population floor, density-dependent costs, a
  maximum lifespan) is a separate decision the user has asked to discuss
  before any of it is placed; the note's Sections 5–7 hold the measurements.
  T17.F01 is the evolvability half of that note's recommendation, not the
  collapse half.
- Order: T17.F02 and T17.F01 are independent; the user chooses which goes
  first. T17.F02 re-expresses the gates of every founder profile that
  exists when it lands (V3Alpha1, the forage-first variants, and T18.F01's
  specialized profile if it is already built); T18.F02 depends on it so the
  default founder is born on the unit scale. T17.F03 reuses F01's decode. T17.F04 waits for its trigger.
- Hazard to carry into every spec here: T19 preserved the canonical founder's
  reproduce-or-forage behavior through votes and its queue-reading branch.
  Any feature that can make physiology refuse a founder attempt must either
  keep its brain gate at or above the acceptance region or explicitly change
  its plan to include foraging. A rejected attempt is free since T16.F01 but
  can still forfeit the tick's foraging opportunity; no retired terminal
  instruction is part of this contract.
- These are mechanism features under the natural-analog rule; each goal
  line names its analog. None adds a sensor, an operator, or an assay.
- Priority: not in the order of new starts until the user places it.
- Post-T19 contract, corrected 2026-09-23: the runtime has one `Eat` vote,
  with a rounded scalar food type in `action_params[Eat][0]`, and directional
  votes for Move, Reproduce and StealEnergy. T17.F03 changes the StealEnergy
  amount parameter; T17.F04 remains unfinished and would add per-type Eat
  votes. Its trigger remains a baseline world or T02/T12 feature introducing
  a third ordinary food type; until then it is unscheduled. Follow the
  [current commit decoder](../reference/v3-mesh-execution-spec.md#2-pass-loop-and-action-selection),
  not the superseded note claiming `Eat[t]` already exists.
