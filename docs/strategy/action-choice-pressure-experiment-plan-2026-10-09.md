# Effective-action pressure: experiment plan (2026-10-09)

Date: 2026-10-09. Status: plan, parked by the user behind the evolution
framework work (the deletion-rate run and the vote-margin read), not yet
run. Question raised by the user: creatures spam actions (61% of logged
actions fail, reproduce 98%) instead of choosing the actions that will
succeed, and the one pressure built for this, the flat failed-action
penalty, was found to tax everyone and shape nobody. What pressure makes
choosing well pay, without blocking the complex solutions evolution has not
yet reached? The evidence is in the
[research note](action-choice-pressure-research-2026-10-09.md); this plan
turns its Section 6 into predeclared rows under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md),
in the form of the [deletion-rate plan](deletion-rate-experiment-plan-2026-10-09.md).

## Problem and constraints

- **Problem.** With a failed attempt priced at 0.02 energy and no time, a
  blind attempt every tick is the optimal program. The founder's energy gate
  on reproduce decayed (68% of sampled attempts under the threshold) once
  rejection became free. Two pressures carry the intent of the retired
  penalty without its defect (more than the effort, charged equally for
  failures no sensor could predict):
  - **effort-priced attempts**: a reproduce attempt costs its effort whether
    or not a litter results (Move and Eat already work this way);
  - **the plan break**: a failed step ends the tick's remaining plan, so a
    plan whose steps all succeed runs in full and a blind step inside one
    forfeits the rest. The multi-action tick stays; it is required.
- **Constraints.**
  - The multi-action tick is required (user ruling, 2026-10-09). No arm
    reduces `max_actions_per_turn` or makes the attempt the tick.
  - The not-forcing checks and a natural analog for each pressure:
    courtship and gamete investment are spent whether or not a mate results
    (Wang et al. 2025); a failed grab leaves no bite (an open-loop motor
    sequence cannot continue past a failed step). Engine precedent for the
    plan break: T03.F12 ends the parent's queue on conception.
  - Branch-only `proto:` code, default off, classified `policy-deviation`;
    the unchanged-policy arm must match `main` draw for draw.
  - No production change outside a roadmap feature. A pressure that proves
    out becomes a T16 row (effort price) or a new row owned by T19 (plan
    break), each with its own natural-analog line.
  - The user's ruling on shortcuts (research note Section 5): a shortcut is
    a measurement routed to the variation tracks, but a pressure is not
    kept on in production until the authored-solution arm and the invasion
    read say the shortcut is not a barrier.
  - One heavy job on the host at a time; this plan runs after the
    deletion-rate and vote-margin runs.
- **Decision criteria.**
  - Does a small per-attempt price retain the founder's energy gate, and at
    what price?
  - Does the plan break make a gating or ordering circuit pay more than it
    pays without the pressure, and does the founder survive it?
  - Under each pressure with mutation on, do creatures attempt less (the
    shortcut) or attempt right (gates and ordering at held plan length)?
  - Can the authored solution invade a population that has taken the
    shortcut?

## Local evidence (inspected 2026-10-09; the research note has the detail)

| Reading | Value | Source |
| --- | ---: | --- |
| Logged actions that fail | 61% | 296 live creatures, tick 31,588 |
| Reproduce attempts under the 30 gate | 68% (7,136 of 10,473) | same |
| Reproduce share of last-tick actions, September against October | 0.9% against 28% | survey 2026-09-16; status 2026-10-09 |
| Rejected-reproduce charge, September against October | about 10.8 against 0.02 | antipattern review; live config |
| Careful advantage under a 0.1 effort price (log replay, energy bound) | 0.09 energy per tick | note Section 2.2 |
| Multi-action ticks with every step succeeding | 1% (156 of 10,940) | note Section 2.2 |
| Intake earned by steps after a failure in the same tick | 47% | note Section 2.2 |
| Avoidance under the 0.45 penalty | 0 of 12,501 | survey 2026-09-16 |
| Authored sensor-to-motor edge, births over its silent twin | +20 to 43% | diagnosis E1 |

The energy-side bound says nothing about the plan break's value; only a
simulated arm can. The September-to-October contrast is confounded by a
month of features; rows P1 and P3 are its controlled form.

**Which baseline.** Every number above comes from the user's live world,
whose config is penalty 0.02 with the ramp off and `reproduce_cost` 0. The
goal recipes (`experiments/worlds/*.json`) set none of these, so the
production baseline is `SimulationConfig::default()`: penalty 1.0 ramped to
tick 62,680 (0.16 at tick 10,000, 0.32 at 20,000) and `reproduce_cost` 0.1,
charged only after the gates pass. A rejected reproduce in production
therefore already pays a rising per-attempt price of 0.16 to 0.32 over P3's
checkpoints, above the 0.1 this plan tests, and the gate decay observed live
may not reproduce there. The plan keeps both: `default` is the production
recipe (comparable with every other run), and `live` is the user's
no-price world (penalty 0.02, ramp off, `reproduce_cost` 0), the true
control for the pressures, which are applied on top of `live`.

**Instrument fit.** The opportunity assay (`crates/v3-cli/src/opportunity.rs`)
takes authored circuits per arm but its config comes from the goal case, so
a pressure value per arm is an `instr:` addition. `v3-cli run --config`
takes a saved config but no circuit. Reproduce's effort price is zero today
(T16.F01 gates before any charge; `reproduce_cost` is 0 in the live config),
so a config-only arm can only use the flat `failed_action_penalty`, which
also taxes blocked moves and failed eats; the clean arm is a `proto:` field.

## Options

| Option | What | For | Against |
| --- | --- | --- | --- |
| **A. Effort price on the reproduce attempt** | `proto:` `energy.costs.reproduce_attempt_cost: Option<f32>`, charged on every reproduce attempt before the gates; `None` is `main` | One charge, one analog, the one action whose attempt is free; log replay bounds a 0.1 price at 0.09 per tick of careful advantage | Flat per attempt, so it does not scale with opportunity; reopens T16.F01's "does not conceive, costs nothing" reading (the gate still charges no replication cost, only the attempt's effort) |
| **B. Plan break** | `proto:` `runtime.plan_break_start_tick: Option<u64>`, default `None`; from that tick a failed action discards the tick's remaining queued actions with no charge and no log entry (T03.F12's queue end, applied to failure); `Some(0)` is on from the start, a later tick is the predeclared fallback that spares the trough | Prices a blind step inside a plan by what the plan had left, so it scales with opportunity and leaves a clean plan untouched; the within-tick form of what Avida and Tierra do across a slice | Drops 47% of today's intake at once (the Orchards shape); the cheapest adaptation is shorter plans; a trailing blind step costs nothing |
| C. A and B together | Both fields | A prices the trailing blind step B cannot; B prices the inner one A cannot | Two mechanisms in one reading unless the single arms run beside |
| D. Flat penalty restored | `failed_action_penalty` at 0.45 to 1.0 | Already measured | Refuted: a tax with 0 avoidance; charged more than the effort for failures no sensor could predict |
| E. Masking | Impossible attempts are not issued | Removes the waste | Removes the pressure with it; the knowledge lives in the environment, not the genome (Huang and Ontañón 2022) |

## Recommendation: A and B as separate arms with their checks, C only as a third arm in P3

### The prototypes (`proto:`)

- **`energy.costs.reproduce_attempt_cost: Option<f32>`.** `None` is the
  default and absent from the serialized config, so recipe digests are
  unchanged. When `Some(c)`: at the top of `apply_reproduce`, before the
  target, cap, age and energy gates, debit
  `adjusted_action_cost(c, complexity, age)` and record it in a new
  `action_charges.reproduce_attempt` flow. Everything after is `main`.
  - Tests: `None` reproduces `main` draw for draw and flow for flow;
    `Some(0.0)` changes no energy; `Some(0.1)` debits exactly once per
    attempt whatever the outcome; a rejected attempt at `Some(c)` leaves
    energy lower by the charge and nothing else; serialization round-trips.
- **`runtime.plan_break_start_tick: Option<u64>`**, default `None`, absent
  from the serialized config when `None`. From that tick on (a deterministic
  step, no RNG, so P0 identity is clean), in the per-creature loop over
  `output.actions` in `simulation/tick.rs`: after an action whose result is not `Success` (or
  `TransferredAndKilled`), the remaining entries of that creature's queue for
  the tick are discarded with no charge, no log entry and no
  `record_action_attempt`; a new counter `plan_breaks_total` and a flow-free
  `actions_discarded_by_plan_break_total` record it. `NoOp` cannot fail.
  T03.F12's conception end is unchanged and counted apart.
  - Tests: `None` reproduces `main` draw for draw; `Some(0)` on an authored
    queue `[Move(blocked), Eat, Reproduce]` executes the move, charges it,
    discards two, logs one; a clean queue runs in full; before the start
    tick nothing is discarded; the stats identity
    `attempted == executed + discarded` holds (proptest).

Both fields are read through the serialized config, so the frozen
instrument builds without them.

### Gate P0: identity

The census probe and the opportunity assay on the prototype binary with
both fields at their defaults, against the instrument built without them:
samples and timing-free checkpoint blocks hash-equal on Canyon seeds 1022
and 2022 to 20,000; the assay's `default` arm set reproduces E1's `(a)`
and `(z)` readings. A mismatch stops the plan.

### Row P1: does a per-attempt price retain the founder's gate? (mutation off)

- **Form.** The opportunity assay's competition, E1's sizes: three goal
  worlds, 8 replicates, 1,000 ticks, 3 threads, mutation off.
- **Arms (`instr:` arm set `attempt-price`).** The founder (gated: it
  attempts reproduce only while `EnergyCurrent` clears its threshold)
  against the founder with that gate lesioned so it votes reproduce every
  pass (the lesion is authored in the arm set, the way E1 authored its
  edges), on the `live` config, under `reproduce_attempt_cost` at `None`,
  0.05, 0.1, 0.3.
- **Reading.** Births of gated over lesioned (the A/Z ratio under T20.F01's
  rule as implemented in `crates/v3-cli/src/opportunity.rs`: positive at 7 of 8 informative
  replicates with a/z > 1 and pooled ≥ 1.05).
- **Prediction.** At `None` the lesioned founder is at or above the gated
  one (free attempts are optimal); the ratio rises with the price; the
  smallest price at which the gated founder is positive on 2 of 3 worlds is
  the **retention price**. If no price up to 0.3 retains the gate, the
  effort price is too weak at this horizon and row P3's `price` arm is
  dropped.

### Row P2: does the plan break make a gate pay, and does the founder survive it? (mutation off)

- **Form.** As P1.
- **Arms (`instr:` arm set `plan-break`).** `plan_break_start_tick` `None`
  and `Some(0)`, on the `live` config, crossed with: the founder; the founder plus an authored `FoodHere(0) →
  Eat` gate (the eat vote scaled by the food-here read, so the blind eat is
  not issued); the founder plus an authored reorder that places its
  reproduce vote after its move and eat (the trailing-blind-step form).
- **Readings.** Births of each authored arm over the founder, with and
  without the break; founder population at tick 1,000 under the break over
  without it (the survival guard).
- **Prediction.** The gate arm's advantage is larger under the break than
  without it; the reorder arm is at or above the founder under the break
  and equal without it. Survival guard: founder population under the break
  at least 0.5 × without it on every world; below that the break runs in
  P3 only from tick 2,000 (`Some(2000)`, after Canyon's trough), predeclared
  here as the fallback.

### Row P3: attempt less or attempt right? (mutation on)

- **Design.** Canyon goal recipe, 20,000 ticks, the census probe with
  checkpoints at 10,000 and 20,000 and the vote-delta reading; seeds 1022,
  2022, 3022, 4022; 3 h kill per run; one job at a time.
- **Arms.** `default` (the production recipe, both fields off; if gate P0
  identity holds against the deletion-rate run's `default` artifacts on the
  same seeds and checkpoints, those four runs are reused, not rerun);
  `live` (penalty 0.02, ramp off, `reproduce_cost` 0, both fields off: the
  no-price control); `price` (`live` plus the retention price from P1, or
  0.1 if P1 is inconclusive); `break` (`live` plus `plan_break_start_tick`
  `Some(0)`, or `Some(2000)` if P2's guard failed); `both`. Comparisons in
  the rule are against `live`; `default` is read beside it, and its own G
  against `live` says whether the production ramp retains the gate.
- **Readings per checkpoint, per seed.**

| Reading | Reads | Shortcut sign |
| --- | --- | --- |
| G: fraction of reproduce attempts rejected for energy (`reproduction_actions_rejected_by_reason`) | the gate | a fall with births per creature-tick below 0.5 × `live` is sterility, the run 2 shortcut |
| A: mean actions per tick and the plan-length distribution (the selected cohort's action logs at the checkpoint, capacity 500 entries, about the last 200 ticks; world-level mean A is `last_tick_actions` over living creatures from stats) | plan capacity | mean A below 0.8 × `live` is the shrink shortcut |
| K: fraction of multi-action ticks with every step succeeding (same logs as A) | plan cleanliness | gamed by shrinkage; read only beside A |
| F: failed share of actions and of energy spend | the spam | — |
| I: food intake per creature-tick; window-mean population and its lowest sample | survival | population below 0.5 × `live` fails the arm |
| P: Σ `family_executed` + `family_causal` over the target rows (B1's measure); vote-delta channels that change an action | sensor use | — |
| S: genome size, mean and per generation | cost | — |

- **Rule (draft, for the user to fix in the goal command).** An arm
  **attempts right** on a seed when, at both checkpoints: G rises against
  `live` (`price`) or K rises (`break`), with A ≥ 0.8 × `live`'s, births
  per creature-tick ≥ 0.5 × `live`'s, and population ≥ 0.5 × `live`'s.
  An arm **attempts less** when G or K improves and A or births fall below
  those floors. Neither is a validation of the pressure as a feature; the
  outcome is a routed reading:
  - attempts right on 3 of 4 seeds: the pressure is a candidate feature
    after gate P4;
  - attempts less on 3 of 4 seeds: the shortcut is the measurement; the
    circuit it names (the energy gate, or plan ordering) goes to T11/T20
    with the counts, and the pressure stays a lab arm;
  - mixed: recorded, no action.

### Gate P4: can the authored solution invade the shortcut? (mutation off)

For each arm that attempted less on any seed: the opportunity assay's
competition at E1's sizes between that seed's 20,000-tick selected cohort
(the shortcut-adapted genomes, dumped by the census probe) and the same
genomes carrying the authored circuit appended in E1's form (a reference at
the next free index plus edges, never an edit of evolved nodes): for the
price shortcut, one compute node that thresholds `EnergyCurrent` at 30 and
pushes a large negative vote onto every `Reproduce` sink while below it, so
the attempt is suppressed rather than the drive removed (appended votes add
to whatever the genome already votes); for the break shortcut, P2's
`FoodHere(0) → Eat` gate or reorder edge. All under the pressure, with the
twin's competence fixtures as in E1. **Invades** when the authored twin is
positive by E1's rule on 2 of 3 worlds. A pressure whose authored solution
cannot invade its own shortcut population is a barrier and changes shape
before any feature row is written; one that invades is safe to keep on in
the lab baseline while the variation tracks work.

## Cost

- **Code.** Two small `proto:` fields with tests; two `instr:` arm sets
  (the lesioned founder, the gate and reorder edges) and the per-arm
  pressure value in the assay; the cohort dump exists on the deletion-rate
  branch. About one session.
- **Runs.** P0 two identity runs; P1 is one eight-arm assay set and P2 one
  six-arm set (E1's eight-arm sets took 2,050 s and 1,781 s at 3 threads,
  so about 35 min each); P3 sixteen new runs at 11 to 15 min (`live`,
  `price`, `break`, `both` on four seeds; `default` reused from the
  deletion-rate run when P0 holds, else four more): about 4 h; P4 one set
  per shortcut arm and seed, about 35 min each.
- **Session.** One goal session of about 150 to 200 turns.

## Remaining uncertainty

- P1's lesion must leave the founder's forage program intact; the arm set's
  competence fixtures (E1's form) check that its non-reproduce actions
  equal the founder's on the battery.
- The plan break's 47% intake footprint may be lethal at Canyon's trough
  before any adaptation; the P2 guard and the ramp are the only timing
  controls, and a killed or extinct P3 arm counts as attempting less.
- A is flat per attempt. If P1 retains the gate only at a price that also
  suppresses legitimate attempts near the threshold, the right form is a
  price proportional to the requested transfer, which is a different proto.
- Nothing here advances the variation question; it settles whether these
  pressures are safe to keep on once variation can supply the circuits.

## Goal command (parked)

Run after the deletion-rate and vote-margin runs have landed. Open a new
session in the main checkout on a clean `main`, model Opus 5.5, with the
advisor enabled on Opus 5.5 and Codex for reviews. Paste:

```
/goal An effective-action pressure run is complete on main. Read docs/strategy/action-choice-pressure-experiment-plan-2026-10-09.md in full (its prototypes, gates P0 and P4, rows P1 to P3 and the draft rule are the contract's form), then docs/strategy/action-choice-pressure-research-2026-10-09.md Sections 2, 5 and 6, then docs/strategy/deletion-rate-run-2026-10.md once it has landed, otherwise docs/strategy/output-wiring-run-2026-10.md (gates, frozen-instrument pattern, Advice table). Confirm Opus 5.5 with the advisor enabled on Opus 5.5 and Codex for reviews, a clean main, no other heavy job on the host, and create the worktree with EnterWorktree named action-pressure. Take the census probe, the cohort dump and the diagnosis probe as file state from branch worktree-deletion-rate (instr and lab commits only, no proto). Build the two proto fields as the plan states them, the attempt-price and plan-break arm sets and the per-arm pressure value in the opportunity assay as instr, reading both fields through the serialized config so the instrument builds without them. Predeclare and commit P0 to P4 before any of them runs: E1's assay sizes for P1, P2 and P4; Canyon goal recipe, seeds 1022, 2022, 3022 and 4022, 20,000 ticks, checkpoints 10,000 and 20,000, arms default (reused from the deletion-rate run when P0 identity holds), live, price, break and both, 3 h kill per run, one job at a time for P3. Run P0, then P1 and P2, then P3 with the price P1 found and plan_break_start_tick 2000 if P2's guard failed, then P4 for every arm that attempted less. The rule is the plan's draft rule as written. Consult the advisor at every rule 8 point and record it; send the predeclaration to Codex before any row runs and the note's reading before closing. Write docs/strategy/action-choice-pressure-run-2026-10.md with the gates, the rows, the Advice table and a plan forward that says, per pressure, candidate feature (and which track), lab arm only with the circuit routed to T11/T20, or barrier (change of shape). Land docs only; keep proto and instr on the branch. Done means the note is on main, make check-docs exited 0 on main, the worktree removed and its branch kept, main clean. Stop after 200 turns.
```
