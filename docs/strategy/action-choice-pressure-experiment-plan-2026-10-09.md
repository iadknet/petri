# Effective-action pressure: experiment plan (2026-10-09)

Date: 2026-10-09. Status: **superseded on 2026-10-10**: the user chose to
build both pressures as roadmap features without a lab run (T16.F02, amended
so every committed action costs at least its effort, then T19.F07, a failed
step ends the tick's remaining plan). This plan stays as the design record;
its readings (queued length, clean-plan share, the "do less" shortcut, the
survival guard) moved into those rows. Its goal command below is not to be
run. Earlier status: plan, parked by the user behind the evolution
framework work (the deletion-rate run and the vote-margin read), not yet
run; revised the same day after a Codex adversarial review (Section "Review
dispositions"). Question raised by the user: creatures spam actions (61% of
logged actions fail, reproduce 98%) instead of choosing the actions that
will succeed (rates from one 200 × 200 viewport of the user's live world;
the research note states the scope), and the one pressure built for this, the flat failed-action
penalty, was found to tax everyone and shape nobody. What pressure makes
choosing well pay, without blocking the complex solutions evolution has not
yet reached? The evidence is in the
[research note](action-choice-pressure-research-2026-10-09.md); this plan
turns its Section 6 into predeclared rows under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md),
in the form of the [deletion-rate plan](deletion-rate-experiment-plan-2026-10-09.md).

## Problem and constraints

- **Problem.** With a failed attempt priced at 0.02 energy and no time, a
  blind attempt every tick is the optimal program. Evolved creatures attempt
  reproduce under the energy threshold (68% of sampled attempts) and eat on
  empty cells (82% of eats fail), although the founder gates both: its
  decision graph sets `Eat = f − 2g − 2q` with `f = [food_here > 0]`, and
  its reproduce votes read `can = energy gate · age gate`, the energy gate a
  `Threshold` on `EnergyCurrent` (a fraction of `max_energy`) at 0.16, which
  is 32 of 200 (`creature/cgp_founder.rs`, `creature/founder.rs`). So two of
  the discriminations at issue are **retention** targets the founder carries
  and its descendants lost; avoiding barriers on a move is an
  **origination** target the founder never had. Two pressures carry the
  intent of the retired penalty without its defect (more than the effort,
  charged equally for failures no sensor could predict):
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
    (Wang et al. 2025). The plan break is a **new hypothesis**: its analog,
    recovery after a failed attempt (a collision, a missed strike, a refused
    courtship) occupying the rest of the tick, is the modeling choice under
    test; Avida and Tierra charge a failed instruction only its own cycle and
    are not precedents for it. The engine seam it would use is T03.F12's
    queue end after a successful conception.
  - Branch-only `proto:` and `instr:` code, default off, classified
    `policy-deviation`; the unchanged-policy arm must match `main` draw for
    draw before any prototype arm is read.
  - No production change outside a roadmap feature. A pressure that proves
    out becomes a T16 row (effort price) or a new row owned by T19 (plan
    break), each with its own natural-analog line.
  - The user's ruling on shortcuts (research note Section 5): a shortcut is
    a measurement routed to the variation tracks, but a pressure is not
    kept on in production until the authored-solution arm and the invasion
    read say the shortcut is not a barrier.
  - One heavy job on the host at a time; this plan runs after the
    vote-margin run (the deletion-rate run was stopped on 2026-10-09).
  - No Fable in any role (section "Roles").
- **Decision criteria.**
  - Over a short horizon with mutation off, does a per-attempt price make
    the founder's reproduce gate pay against a matched lesion, and at what
    price? Does the plan break make the eat gate and an authored barrier
    gate pay more than without it, and does the founder survive it?
  - With mutation on, does each pressure produce gated, clean plans at held
    plan length (attempts right), or fewer attempts, shorter plans or
    sterility (attempts less)?
  - Where a shortcut appears, can a competent rare invader carrying the
    solution grow in the shortcut's own resident world?

## Local evidence (inspected 2026-10-09; the research note has the detail)

| Reading | Value | Source |
| --- | ---: | --- |
| Logged actions that fail | 61% | 296 creatures with logs in one 200 × 200 viewport, tick 31,588 (not whole-world) |
| Reproduce attempts under the 30 gate | 68% (7,136 of 10,473) | same |
| Eat attempts on an empty cell | 82% | same |
| Reproduce share of last-tick actions, September against October | 0.9% against 28% | world status, 2026-09-16 and 2026-10-09 |
| Charge on an energy-constraint rejection, September against October | about 10.8 against 0.02 | antipattern review; live config |
| Frozen-outcome ledger difference near a 0.1 price | 0.08 to 0.09 energy per active tick (bookkeeping, not a bound) | note Section 2.2 |
| Multi-action ticks with every step succeeding | 1% (156 of 10,940) | note Section 2.2 |
| Food intake earned by steps after a failure in the same tick | 49% gross food flow, 47% credited (historical exposure, not a forecast) | note Section 2.2 |
| Barrier on the chosen direction: no longer moves there / any queue change | 0 / 24 of 12,501 | survey 2026-09-16 |
| Authored sensor-to-motor edge, births over its zero-weight twin | +20 to 43% | diagnosis E1 |

**Which baseline.** Every live number above comes from the user's world,
whose config is penalty 0.02 with the ramp off and `reproduce_cost` 0. The
goal recipes (`experiments/worlds/*.json`) set none of these, so production
is `SimulationConfig::default()`: penalty 1.0 ramped to tick 62,680 (about
0.16 at tick 10,000 and 0.32 at 20,000), charged on a gate-rejected
reproduce as on any failed action (`execute_reproduce`), and
`reproduce_cost` 0.1 charged only after the gates pass. Production therefore
already prices a rejected reproduce at a rising 0.16 to 0.32 before the age
and complexity multiplier. The plan keeps both: `default` is the production
recipe, and `live` (penalty 0.02, ramp off, `reproduce_cost` 0) is the
no-price control on which the pressures are applied.

**Instrument fit.** The opportunity assay (`crates/v3-cli/src/opportunity.rs`,
`neighborhood/opportunity/assay.rs`) runs one simulation config per
replicate with genome arms competing inside it. Both proposed fields are
global, so a pressure is a **replicate-wide treatment**: each pressure
setting is its own set of competitions, with the genotype competitors
inside. Cumulative counters (`reproduction_actions_rejected_by_reason`) need
differencing over declared windows, and action logs record executed actions
only (500 entries, no zero-action ticks, no discarded entries), so the plan
reads plan capacity from new window counters, not from logs.

## Options

| Option | What | For | Against |
| --- | --- | --- | --- |
| **A. Effort price on the reproduce attempt** | `proto:` `energy.costs.reproduce_attempt_cost: Option<f32>`, charged on every reproduce attempt before the gates; `None` is `main` | One charge, one analog, the one action whose attempt is free | Flat per attempt; `adjusted_action_cost` scales it with age (a base 0.1 is 1.0 at age 200) and complexity; reopens T16.F01's "does not conceive, costs nothing" reading (the gate still charges no replication cost, only the attempt's effort) |
| **B. Plan break** | `proto:` `runtime.plan_break_start_tick: Option<u64>`, default `None`; from that tick a failed action discards the tick's remaining queued actions with no charge and no log entry | Prices a blind step inside a plan by what the plan had left; leaves a clean plan untouched | Its historical exposure is about half of gross food intake (the Orchards shape); the cheapest adaptation is shorter plans; a trailing blind step costs nothing |
| C. A and B together | Both fields | A prices the trailing blind step B cannot; B prices the inner one A cannot | Two mechanisms in one reading unless the single arms run beside |
| D. Flat penalty restored | `failed_action_penalty` at 0.45 to 1.0 | Already measured | Refuted: a tax with 0 avoidance; charged more than the effort for failures no sensor could predict |
| E. Masking | Impossible attempts are not issued | Removes the waste | Removes the pressure with it; the knowledge lives in the environment, not the genome (Huang and Ontañón 2022) |

## Recommendation: A and B as separate arms with their checks, C as a third arm in P3

### The prototypes (`proto:`)

- **`energy.costs.reproduce_attempt_cost: Option<f32>`.** `None` is the
  default and absent from the serialized config, so recipe digests are
  unchanged. When `Some(c)`: at the top of `apply_reproduce`, before the
  target, cap, age, energy and transfer gates, debit
  `adjusted_action_cost(c, complexity, age)` and record it in a new
  `action_charges.reproduce_attempt` flow. Everything after is `main`.
  - Tests: `None` reproduces `main` draw for draw and flow for flow;
    `Some(0.0)` changes no energy; `Some(0.1)` debits exactly once per
    attempt whatever the outcome; a rejected attempt leaves energy lower by
    the charge and nothing else; serialization omits the field at `None`
    and round-trips at `Some`.
- **`runtime.plan_break_start_tick: Option<u64>`**, default `None`, absent
  from the serialized config when `None`. From that tick on (a
  deterministic step, no RNG), in the per-creature loop over
  `output.actions` in `simulation/tick.rs`: after an action whose result is
  not `Success` or `TransferredAndKilled`, the remaining entries of that
  creature's queue for the tick are discarded with no charge, no log entry
  and no `record_action_attempt`. `NoOp` cannot fail. T03.F12's conception
  end is unchanged.
  - Tests: `None` reproduces `main` draw for draw; `Some(0)` on an authored
    queue `[Move(blocked), Eat, Reproduce]` executes and charges the move,
    discards two, logs one, records one Phase 2 attempt; a clean queue runs
    in full; before the start tick nothing is discarded; the identity
    `queued = executed + discarded_by_all_causes` holds (proptest), with
    discards counted by cause (conception, death, gestation, plan break).

Both fields are read through the serialized config, so the frozen
instrument builds without them.

### The instruments (`instr:`)

- **Window counters** on `SimStats`, accumulated over declared windows
  (`[9,000, 10,000)` and `[19,000, 20,000)` in P3), per creature-tick:
  queued plan length (from `MeshOutput.actions`), executed length,
  zero-action ticks, complete-plan successes (ticks with two or more queued
  actions where every executed action succeeded and nothing was discarded),
  discards by cause, reproduce attempts, and reproduce rejections split by
  cause: invalid target, age, **parent energy** (the post-cost threshold,
  `reproduction.rs` Step 6) and **transfer** (Step 7), plus births.
- **Arm sets in the opportunity assay**, each competition run under one
  replicate-wide config:
  - `reproduce-lesion`: the founder against the founder whose four
    `Reproduce(d)` sinks read the age gate (CN1) in place of `can` (CN2),
    every other edge unchanged, so the energy gate still inhibits Eat and
    Move exactly as in the founder.
  - `eat-lesion`: the founder against the founder whose Eat drive edge
    reads the constant bias in place of `food_here`, every other edge
    unchanged (a blind eater with the same reproduce gate and inhibitions).
  - E1's existing `one-edge` arm (c), `NeighborBarrierRing[E] → Move(E)` at
    −0.5, against its zero-weight twin (same references, nodes and size),
    reused unchanged.
  - Competence fixtures, E1's form: each lesioned or authored arm's
    committed actions equal the founder's on every battery scenario except
    the one the change targets, and the targeted difference is shown on an
    authored context; violations are counted and stop the arm.
- **A per-arm pressure value** is not needed: the pressure is the
  replicate's config; the existing arm sets' execution and sampling path is
  left unchanged and is gate P0's identity target.

### Gate P0: identity

The census probe and the opportunity assay on the prototype binary with
both fields at their defaults, against the instrument built without them:
samples and timing-free checkpoint blocks hash-equal on Canyon seeds 1022
and 2022 to 20,000; the assay's default arm set reproduces E1's `(a)` and
`(z)` readings. A mismatch stops the plan.

### Row P1: short-horizon opportunity price (mutation off)

- **Form.** One set of competitions per price, E1's sizes: three goal
  worlds, 8 replicates, 1,000 ticks, 3 threads, mutation off, on the `live`
  config with `reproduce_attempt_cost` at `None`, 0.05, 0.1 and 0.3: 96
  competitions, arm set `reproduce-lesion`.
- **Reading.** Births of the gated founder over the lesioned founder under
  T20.F01's rule as implemented in `crates/v3-cli/src/opportunity.rs`,
  including its exposure gate (here: lesioned arms making attempts under
  the threshold in at least 5% of sampled ticks) and informativeness gate
  (a + z ≥ 20): positive at 7 of 8 informative replicates with ratio > 1
  and pooled ≥ 1.05.
- **What it is.** An opportunity reading, not retention: the T22 ladder
  defines retention over descendant lineages under mutation, which P3
  reads. The founder's threshold (32) sits above the engine's (30), so the
  lesioned arm can breed between 30 and 32 sooner; that is a real advantage
  of attempting and is reported, not corrected.
- **Prediction.** At `None` the lesioned founder is at or above the gated
  one; the ratio rises with the price.
- **Disposition.** Positive on 2 of 3 worlds at some price: P3's price is
  the smallest such price. Negative or inconclusive at every price: P3's
  price is 0.3, labelled as the production ramp's level at 20,000, and the
  arm still runs; P1 never drops a P3 arm, because a short-horizon loss
  does not reject a pressure whose effect is on retention.

### Row P2: does the plan break make a gate pay, and does the founder survive it? (mutation off)

- **Form.** As P1, on the `live` config, with `plan_break_start_tick`
  `None` and `Some(0)`: arm set `eat-lesion` (48 competitions) and E1's
  barrier arm with its twin (48 competitions).
- **Readings.** Founder over eat-lesioned founder, and barrier edge over
  its twin, under each break setting, read separately (no equality is
  assumed without the break); founder-arm population at tick 1,000 under
  the break over without it (the survival guard).
- **Prediction.** Each gate's advantage is larger with the break than
  without it.
- **Survival guard.** Founder-arm population under the break at least
  0.5 × without it on every world. Below that, P3's `break` and `both` arms
  use `Some(2000)` (after Canyon's trough), predeclared here.

### Row P3: attempts right or attempts less? (mutation on)

- **Design.** Canyon goal recipe, 20,000 ticks, the census probe with
  checkpoints at 10,000 and 20,000, the vote-delta reading, and the window
  counters over `[9,000, 10,000)` and `[19,000, 20,000)`; seeds 1022, 2022,
  3022, 4022; 3 h kill per run; one job at a time.
- **Arms.** `default` (production recipe, both fields off; run here, since
  the deletion-rate run's `default` artifacts predate T11.F29's production
  change and carry no window counters); `live` (penalty 0.02, ramp off, `reproduce_cost` 0, both
  fields off); `price` (`live` plus P1's price); `break` (`live` plus
  `Some(0)`, or `Some(2000)` if P2's guard failed); `both` (`live` plus the
  same price and the same break setting).
- **Readings per window, per seed** (comparisons against `live`; `default`
  read beside it, its own G saying whether the production ramp retains the
  gate):

| Reading | Definition | Better is |
| --- | --- | --- |
| G | parent-energy rejections ÷ reproduce attempts in the window | lower |
| E | of the selected cohort, the fraction whose committed reproduce turns off when `EnergyCurrent` is swept below 0.15 on the battery (the survey's energy sweep) | higher |
| K | complete-plan successes ÷ ticks with two or more queued actions | higher |
| Q | mean queued plan length per creature-tick | held (plan capacity) |
| B | births per creature-tick | held |
| N | window-mean population and its lowest sample | held |
| F, I, P, S | failed share of actions and of spend; intake; sensor use (B1's measure); genome size | reported |

- **Per seed and window, one label** (thresholds are exploratory, chosen to
  tolerate stated losses; they are not safety claims):
  - `improved`: for `price`, G at most 0.8 × `live`'s and E above `live`'s;
    for `break`, K at least 1.2 × `live`'s; for `both`, both.
  - `attempts-right`: improved, with Q ≥ 0.8 ×, B ≥ 0.5 × and N ≥ 0.5 ×
    `live`'s.
  - `attempts-less`: improved, N ≥ 0.5 × `live`'s, and Q < 0.8 × or
    B < 0.5 × `live`'s.
  - `no-change`: not improved, N ≥ 0.5 × `live`'s.
  - `population-failure`: N < 0.5 × `live`'s, or extinct.
  - `censored`: the run was killed before the window.
  - A window with zero reproduce attempts has G undefined; it is
    `attempts-less` if B < 0.5 × `live`'s (sterility), else `no-change`.
- **Per seed:** the same label in both windows, else `mixed`.
- **Per arm:**
  - `attempts-right` on 3 of 4 seeds and the matching P1 or P2 reading
    positive: a candidate feature (T16 row for the price, T19-owned row for
    the break).
  - `attempts-less` on 3 of 4 seeds: the shortcut is the measurement; gate
    P4 runs on those seeds; the named circuit goes to T11/T20 with counts.
  - `population-failure` on 2 or more seeds: the pressure is too harsh at
    this form; a later start tick or a smaller price is the recorded next
    form.
  - Otherwise `mixed`, recorded, no action.

### Gate P4: can a competent rare invader grow in the shortcut's world? (mutation on, resident state preserved)

Runs per (arm, seed) labelled `attempts-less` with a living population at
20,000.

- **Fork.** The resident state at tick 20,000 is reproduced by running the
  same seed and arm to 20,000 in two processes (determinism is a project
  invariant; a branch-only `Clone` on `Simulation` is the alternative if it
  is cheaper). At 20,000 a seeded draw picks 5% of living residents; in one
  process they are replaced by **invaders**, in the other by **controls**;
  both continue under the same pressure for 5,000 ticks.
- **Invaders and controls by shortcut.**
  - Sterility (B fell): invader the founder (a complete, viable
    reproduction drive with its gate and transfer parameter); control the
    `reproduce-lesion` founder (the same drive, ungated). The question is
    whether a gated reproducer outgrows a blind one in the resident world.
  - Shorter plans or blind steps (Q fell, or K improved by shrinkage):
    invader the replaced resident's own genome with E1's barrier edges
    appended (`NeighborBarrierRing[d] → Move(d)` at −0.5 for each cardinal
    d, a reference at the next free index plus edges on its most-executed
    Graph node with an `ActionVote` surface); control the same genome with
    the identical appended references and edges at weight zero.
- **Competence before reading.** Each invader is executed on the battery:
  the appended node must execute before commit and the gate must change the
  targeted action on an authored context. Residents whose genome cannot
  take the append are left in place and counted. If fewer than half of the
  drawn residents carry a competent invader, P4 for that (arm, seed) is
  `instrument-inconclusive`.
- **Reading.** Living descendants of the replaced creatures at 25,000
  (by lineage of the replaced slots), invader process over control process.
  **Invades** at a ratio of 1.2 or more; **fails** below 1.0; between is
  inconclusive.
- **Disposition.** A pressure whose competent invader fails on every seed
  where P4 ran is a barrier and changes shape before any feature row; one
  whose invader invades on most is safe to keep on in the lab baseline
  while the variation tracks work. `instrument-inconclusive` never counts
  as barrier.

## Cost

- **Code.** Two small `proto:` fields with tests; the window counters, the
  rejection split, the two lesion arm sets with competence fixtures, the
  energy-sweep reading, the P4 fork-and-replace driver. About one and a
  half sessions.
- **Runs** (projections from E1's eight-arm sets at 2,050 s and 1,781 s at
  3 threads; a pilot fixes them):
  - P0: two identity runs.
  - P1: 96 competitions, about 2 to 2.3 h.
  - P2: 96 competitions, about 2 to 2.3 h.
  - P3: 20 runs at 11 to 15 min, about 4 to 5 h.
  - P4: two processes per (arm, seed) to 25,000 ticks, about 30 min each;
    up to 12 pairs, so up to 12 h in the worst case, typically far fewer.
- **Session.** One goal session of about 200 turns, likely split at P3.

## Remaining uncertainty

- The lesions are founder-specific; evolved genomes lose these gates in
  other ways, which P3 and P4 read and P1 and P2 do not.
- The plan break's exposure (about half of historical gross intake) may be lethal at Canyon's trough
  before any adaptation; the P2 guard and the start-tick fallback are the
  only timing controls.
- A is flat per attempt. If the gate is retained only at a price that also
  suppresses legitimate attempts near the threshold, the right form is a
  price proportional to the requested transfer, which is a different proto.
- P4's 5% replacement and 5,000-tick horizon are exploratory choices; a
  null there is weak evidence of a barrier, which is why only a failure on
  every seed counts.
- Nothing here advances the variation question; it settles whether these
  pressures are safe to keep on once variation can supply the circuits.

## Roles (this run's rule 8; no Fable)

Fable credits are exhausted (user, 2026-10-09). No Claude role in this run
is Fable. Rule 8 of the diagnosis plan says every role is Fable and that a
run stops rather than substitute Opus; that clause does not apply here, and
neither do the Fable roles in the output-wiring and born-connected run
notes. Nothing in them is a reason to stop or to wait for Fable.

- **Session.** Opus 5.5 (`claude-opus-5-5`) at effort `xhigh`. It is the
  implementer: it writes the instruments, runs the gates and rows, reads the
  results and writes the note. No subagent is needed; if one is spawned, it
  runs on Opus (`model: "opus"`), never Fable.
- **Advisor.** Opus, enabled with `/advisor opus` before the goal is pasted
  (the project settings default `advisorModel` to `fable`). Consulted through
  the `advisor` tool at every one of these points, each recorded with its
  dispositions in the note's Advice table:
  - before the first gate, on the approach to the whole run;
  - after each gate's or row's reading is drafted and before it is recorded;
  - whenever a gate fails, a row is `incomplete`, a result contradicts its
    prediction, or an approach has not converged after two attempts;
  - before the plan forward is drafted, and again after;
  - before closing.
  The advisor advises; it does not write code, run rows or edit the note. If
  the advisor tool is unavailable or returns a credit error, the run records
  it, sends the same brief to Codex as an advice job, and continues.
- **Reviews.** Codex `gpt-6.1-sol` at `high`, read-only, through
  `codex exec`: (a) the predeclaration before any heavy row runs; (b) the
  note's reading before closing. A Codex usage limit marks the review
  `pending`; the run continues, and the note recommends nothing until review
  (b) has completed.

## Review dispositions (Codex adversarial review, 2026-10-09)

| # | Finding | Disposition |
| --- | --- | --- |
| 1 | P3 rewarded a rising energy-rejection fraction; energy rejections mix the parent threshold and the transfer check; `both` had no predicate | Adopted: G must fall; rejections split by cause; E (energy sweep) added as behavioral evidence; `both` requires both |
| 2 | The founder already gates Eat on `food_here`; the reorder arm is not neutral without the break | Adopted: the eat gate is a retention target with a matched `eat-lesion`; the reorder arm is dropped; the barrier gate uses E1's arm and zero-weight twin; each read under each break setting separately |
| 3 | P4's reproduce circuit was mis-scaled (`EnergyCurrent` is a fraction), wrong polarity, and cannot restore a removed drive | Adopted: the sterility invader is the founder itself (a complete gated drive) against the lesioned founder; the plan invader appends E1's barrier edges with a zero-weight control; competence and `instrument-inconclusive` rules added |
| 4 | P4 measured fresh-world competition, not invasion | Adopted: the resident state is reproduced to 20,000, 5% of residents are replaced, and descendant growth is read against a matched control |
| 5 | Pressures are global configs; one arm set cannot carry them; the competition count was understated | Adopted: replicate-wide treatments; 96 and 96 competitions budgeted |
| 6 | A, K and G had no common window; action logs miss discards and zero-action ticks; the invariant was wrong | Adopted: window counters over declared windows; cumulative counters differenced; `queued = executed + discarded_by_all_causes` |
| 7 | The lesion was unspecified (the gate also inhibits forage); P1 is opportunity, not retention | Adopted: the exact lesion (Reproduce sinks read CN1, all else unchanged); P1 renamed and never drops a P3 arm; the 30-against-32 asymmetry reported |
| 8 | Contradictory and non-exhaustive dispositions | Adopted: P1's three outcomes fixed; labels made exhaustive (`no-change`, `population-failure`, `censored`, zero-denominator rule); P4 requires a living shortcut cohort; floors labelled exploratory |

## Goal command (parked)

Run after the vote-margin run has landed. Open a new session in the main
checkout on a clean, current `main`, with no other heavy job on the host.
Set the model to Opus 5.5 at effort `xhigh`, run `/advisor opus` (the
project default is a Fable advisor, and Fable credits are exhausted), and
confirm the startup notice shows an Opus advisor. Then paste:

```
/goal An effective-action pressure run is complete on main. Read docs/strategy/action-choice-pressure-experiment-plan-2026-10-09.md in full (its prototypes, instruments, gates P0 and P4, rows P1 to P3, labels, dispositions and review dispositions are the contract's form), then docs/strategy/action-choice-pressure-research-2026-10-09.md Sections 2, 5 and 6, then docs/strategy/output-wiring-run-2026-10.md (gates, frozen-instrument pattern, Advice table). No Fable in any role: confirm the session is Opus 5.5 at effort xhigh with an Opus advisor and Codex for reviews, as the plan's Roles section states (it replaces rule 8; the Fable roles in the diagnosis plan and the output-wiring and born-connected notes do not apply, and are never a reason to stop). Confirm a clean main and no other heavy job on the host, and create the worktree with EnterWorktree named action-pressure. Take the census probe, the cohort dump and the diagnosis probe as file state from branch worktree-deletion-rate (instr and lab commits only, no proto). Build the two proto fields and the instruments exactly as the plan states them, reading both fields through the serialized config so the instrument builds without them. Predeclare and commit P0 to P4 before any of them runs: replicate-wide pressure treatments at E1's assay sizes for P1 and P2; Canyon goal recipe, seeds 1022, 2022, 3022 and 4022, 20,000 ticks, checkpoints 10,000 and 20,000, windows [9,000, 10,000) and [19,000, 20,000), arms default, live, price, break and both, 3 h kill per run, one job at a time for P3; P4 per (arm, seed) labelled attempts-less with a living population. Run P0, then P1 and P2, then P3 with P1's disposition and P2's guard applied, then P4. The labels and dispositions are the plan's as written. Consult the Opus advisor at every point the Roles section lists and record it in the Advice table; send the predeclaration to Codex before any row runs and the note's reading before closing. Write docs/strategy/action-choice-pressure-run-2026-10.md with the gates, the rows, the Advice table and a plan forward that says, per pressure, candidate feature (and which track), lab arm only with the circuit routed to T11/T20, too harsh at this form, or barrier (change of shape). Land docs only; keep proto and instr on the branch. Done means the note is on main, make check-docs exited 0 on main, the worktree removed and its branch kept, main clean. Stop after 200 turns.
```
