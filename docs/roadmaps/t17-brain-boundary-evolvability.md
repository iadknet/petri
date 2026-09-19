# T17 — Brain Boundary Evolvability

**Status**: In Progress
**Last updated**: 2026-09-18
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Every value that crosses between a creature's brain and its world or body is
on the scale the brain's mutation operators already assume, so a
sensor-driven rule for how much to invest, how much to take, or when to act
is one edge away instead of a scaling construction. The
[reproduction-collapse research note](../strategy/reproduction-collapse-research-2026-09-18.md)
(Section 8) audited every boundary value: new graph thresholds, constants,
and edge weights are drawn in [−1, 1], parameter steps are ±0.1, and the
Covariance Hebbian rule is `(pre − 0.5)(post − 0.5)`, yet the parent's own
energy, age, and generation reach the brain in raw units and the reproduce
transfer and steal amounts leave it in raw energy. Afterward a creature can
evolve, in one mutation, an offspring investment that follows the food around
it or the time since it last ate, and a threshold on its own energy that a
new compute node can actually tune.

## Track Success Criteria

- [ ] Every introspective input (`EnergyCurrent`, `EnergyConsumedThisTick`,
  `AgeTicks`, `Generation`) reaches the brain as a value in [0, 1], and the
  founder's behavior is unchanged by construction (its gates re-expressed on
  the new scale, verified by the founder tests and the viability test).
- [ ] The reproduce transfer and the steal amount are fractions in [0, 1] at
  the action boundary, and a founder attempts only births that physiology
  accepts.
- [ ] The T11.F14 battery and the survey's paired-perturbation probe are read
  before and after each feature, so the change in behavior-changing births,
  silent births, and Hebbian-carrier outcomes is attributable to it.

## Executable Features

- [x] **T17.F01 — Offspring Investment as a Fraction of Parent Energy** — Depends on: T16.F01
  - Goal: A parent gives its young a share of what it has, not a fixed ration. The reproduce action's energy-transfer meta becomes a fraction in [0, 1] of the parent's post-cost energy (Polyworld's `MateEnergyFraction`, a heritable share of current energy the world bounds to [0.2, 0.8]; capital breeders provision litters from their reserves), so a sensor routed into that slot is a condition-dependent investment rule. Physiology keeps the parent's `min_reproduce_energy` gate (T16.F01) and adds a minimum litter (a child born with less than `initial_energy` is not conceived, rejected before charge). The founder emits a constant fraction and its brain gate is set so every attempt it makes is accepted: the founder's reproduce branch ends with the terminal `ExecuteActionQueue`, so a refused attempt is a tick without eating, and a founder whose gate sits below what physiology accepts pins itself at its threshold (measured: 0 births in 2,000 ticks, with and without `failed_action_penalty`). Predeclared: births-based indicators and the goal trajectories move (epoch re-pin recorded); the founder's births per creature-tick on the gate profile are read before and after.
- [ ] **T17.F02 — Unit-Scale Introspection** — Depends on: None
  - Goal: A creature feels how full, how tired, and how old it is as a share of itself, not in joules or ticks. `EnergyCurrent` and `EnergyConsumedThisTick` reach the brain as fractions of `max_energy`; `AgeTicks` as a saturating fraction of a configured reference span (T03.F07's lifespan is its later denominator); `Generation` is removed from the input key set or saturates the same way. The founder's gates are re-expressed exactly (32 → 0.16 of 200 since T17.F01; the age gate on the same span) so its behavior is identical by construction. Evidence: a new `Threshold` drawn in [−1, 1] against raw energy is always on; the founder's 30 needs ~700 ±0.1 steps to reach 100; Covariance with `pre` = 20–200 drives the weight to its clamp in one tick, which is the six Orchards Hebbian carriers; 14 of the 59 Orchards survivors had `Generation` (1–6) swapped in for energy or age as a constant gate. Predeclared: the T11.F14 changed/silent/dead readings and the goal trajectories move (epoch re-pin recorded); founder gate behavior unchanged.
- [ ] **T17.F03 — Steal Amount as a Fraction** — Depends on: T17.F01
  - Goal: A bite takes a share of the prey, not a fixed calorie count. The steal action's amount meta becomes a fraction in [0, 1] of the victim's current energy, capped by the existing config limit (Polyworld's attack depletion scales with the attacker's state and is capped). Predeclared: predation transfer per event in the goal reports is read before and after.
- [ ] **T17.F04 — Per-Type Eat Bank** — Depends on: T11.F21
  - Goal: Which food to bite is a bid per food type, as direction became a bid per direction in T11.F21. Today the eat action's type index is a rounded scalar, which a [0, 1] sensor drives correctly for exactly two food types and not for three. Trigger: a baseline world or T02/T12 feature with a third ordinary food type; until then unscheduled.

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
- Hazard to carry into every spec here: the founder's reproduce branch is
  terminal (`ExecuteActionQueue` halts the VM), so the founder forages or
  breeds per tick, never both. Any feature that can make physiology refuse a
  founder attempt must either keep the founder's brain gate at or above the
  acceptance region or change the founder to queue reproduce and forage in
  one tick. A refused attempt is free since T16.F01 but still costs the tick.
- These are mechanism features under the natural-analog rule; each goal
  line names its analog. None adds a sensor, an operator, or an assay.
- Priority: not in the order of new starts until the user places it.
