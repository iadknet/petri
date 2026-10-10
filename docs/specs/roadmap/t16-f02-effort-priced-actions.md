# T16.F02 — Effort-Priced Actions

**Status**: In Progress
**Last updated**: 2026-10-10
**Feature**: T16.F02
**Track**: [T16 — Cost and Cue Fidelity](../../roadmaps/t16-cost-and-cue-fidelity.md)

## Goal

Every act costs its effort, and failing costs nothing extra. Every committed
action (`Move`, `Eat`, `Reproduce`, `StealEnergy`, `NoOp`) charges a nonzero
effort on every attempt, whatever the outcome, and none charges less than
`NoOp`: `eat_cost` defaults to 0.05 (was 0), a reproduce attempt pays a new
`reproduce_attempt_cost` before any gate while the replication cost is still
paid only on a conception in T16.F01's order, and a `StealEnergy` requesting
nothing pays at least `NoOp`'s effort. A blocked move, an eat on an empty
cell, a rejected reproduce and a steal with no victim pay that effort and
nothing more: `failed_action_penalty`, its startup ramp, and their config,
panel, telemetry and reference-spec surface are gone. The goal report shows
what failures cost (a failed-action effort flow) and how reproduce attempts
end (attempts and rejections by reason).

## Non-Goals

- No change to action validity, gate predicates beyond reading post-effort
  energy (invariant 3), result classification, target selection, the
  multi-action tick, vote selection, priority bids, the age and complexity
  multipliers, the values of `move_cost`, `noop_cost`, `reproduce_cost`
  (replication) and `steal_cost_rate`, the transfer rule, gestation, or
  mutation.
- T19.F07 (a failed step ends the plan) is not here: a failed step still
  leaves the rest of the tick's plan to run.
- No masking, reward, sensor, operator, assay or new pressure. The T12.F04
  pressure-integration rule does not apply: an action price is not an
  environmental pressure, and the goal-world recipes, which set no cost,
  are unchanged.
- No lab recalibration: `crates/v3-lab` changes only where it read the
  retired surface, plus test-fixture seeds the new prices moved off their
  path (invariant 8, Lab row); no stored calibration verdict is re-run.
- Closed specs, strategy notes, stored summaries, readings and sweep configs
  are not edited. A saved config carrying a retired key fails to resolve, with
  no alias or migration, as at T11.F20.

## Inputs and Invariants

Sources of truth: the T16.F02 row as amended 2026-10-10; track success
criteria 1, 2 and 4; the track notes "Order", "T16.F02 changes production
trajectories" and "Amended and placed, 2026-10-10";
[T16.F01](t16-f01-gate-before-charge-in-reproduction.md) (gate-then-charge
order, its invariants 1–6, and its deferral of criterion 4's probe to this
feature); the [antipattern review](../../strategy/antipattern-review-2026-09-16.md)
Section 2; the [action-choice pressure note](../../strategy/action-choice-pressure-research-2026-10-09.md)
Sections 2.1–2.3 and 4. Planning references from T11.F29, the gate epoch,
the goal-worlds epoch and the previous closure at once, are tabulated in the
[readings](../../progress/readings/t16-f02.md).

Research and options (2026-10-10):

| Question | Options and evidence | Choice |
| --- | --- | --- |
| How effort is priced | Polyworld charges activation × a behavior's energy factor, and a blocked move pays the movement and no more (Yaeger 1994); Avida charges a failed instruction its cycle and has no failure penalty; Huang and Ontañón (2022) find invalid-action penalties hurt exploration | Effort on every attempt, no outcome surcharge (the row) |
| Where "none less than `NoOp`" holds | (a) defaults only; (b) normalization raises each cost to `noop_cost`; (c) one charge-time floor `max(base, noop_cost)` on every action | (c): holds for every config, gives `StealEnergy` its zero-amount floor from the same rule, and adds no cross-field normalization; (b) would make the server's reject-never-clamp patch path reject a `noop_cost` patch for fields it did not name; (a) leaves the seven tracked recipes' explicit `eat_cost` 0.0 free |
| Effort values | The floor, or larger | The floor (0.05) for every new or raised effort: the smallest value the row admits; nothing in the record bounds a larger one (note Section 2.1) |
| Where the reproduce attempt is charged | `execute_reproduce`, or Step 0 of `apply_reproduce` | Step 0 of `apply_reproduce`, the authority on the sequence |
| Reproduce attempt flow | Its own flow, or `action_charges.reproduce` | `action_charges.reproduce`; the rejected share is `failed_action_effort.reproduce` |
| Packages | None applies | Extend the existing charge sites |

Effort is `adjusted_action_cost(max(base, noop_cost), complexity, age)`, the
existing multiplier (at least 1). Predeclared values:

| Action | Base | Default | Charged |
| --- | --- | --- | --- |
| `NoOp` | `energy.costs.noop_cost` | 0.05, unchanged; normalization now requires finite and > 0, else 0.05 | every `NoOp` |
| `Move` | `energy.costs.move_cost` | 0.2, unchanged | every attempt, after the move resolves (order unchanged) |
| `Eat` | `energy.costs.eat_cost` | **0.05** (was 0.0) | every attempt, after the food credit and the max-energy clamp (order unchanged) |
| `Reproduce` attempt | new `energy.costs.reproduce_attempt_cost`, finite and >= 0 else 0.05 | **0.05** | every attempt, Step 0, before Step 1 |
| `Reproduce` replication | `energy.costs.reproduce_cost` × T03.F11 multiplier | 0.1, unchanged | only on conception, Step 8 (T16.F01) |
| `StealEnergy` | `predation.steal_cost_rate` × sanitized requested amount | rate 0.05, unchanged | every attempt, before the target resolves (order unchanged) |

Invariants; invariant 8, the retirement of every surface, is the second table:

| # | Invariant |
| --- | --- |
| 1 | **Floor.** The effort is the `f32` amount subtracted from stored energy. For one creature at one moment every action's effort is at least `NoOp`'s, and `NoOp`'s is above zero: the actions share the multiplier and `f32` multiplication is monotone. With `base >= noop_cost` the effort is bit for bit today's `adjusted_action_cost(base, …)`, so `Move` and `NoOp` charges are unchanged at the defaults. The flows record the applied debit, the `f64` difference of the stored `f32` endpoints, which differs from the effort by at most half an `f32` ulp of the result (under 1.6e-5 below energy 256; `max_energy` defaults to 200) and is above zero whenever the effort exceeds that half ulp. |
| 2 | **A failure pays its effort only.** A blocked move (barrier, occupied, out of bounds), an eat that finds no food (configured type or not), a reproduce rejected on any branch (invalid target, population cap, age, energy, litter) and a `RejectedNoVictim` steal store `before − effort` for that action's effort, and nothing else is debited for the attempt. Results, failure counters, `ActionSuccess` outcomes and action-log entries are unchanged except for the energies they record. |
| 3 | **Reproduce order.** Step 0 debits the attempt effort (sink `ActionReproduce`, flow `action_charges.reproduce`); Steps 1–8 then run as T16.F01 left them on the post-effort energy, so the energy gate reads `energy − effort − cost >= min_reproduce_energy` and the litter is the fraction of `energy − effort − cost`. A rejection never pays `cost` or a transfer. A conception pays effort, `cost` and transfer as three successive `f32` subtractions: "the attempt's effort plus what it pays today" (track criterion 2). |
| 3a | **Founder margin.** T17.F01 invariant 5 holds: every founder profile's gate (`EnergyCurrent` strictly above 0.16 to 0.30 of `max_energy`, at least 2.0 above `min_reproduce_energy`) still clears the energy gate and the litter floor after the effort and the cost at the maximum age multiplier (V3Alpha1: 32 − 0.5 − 1.0 = 30.5, litter 20.33; Conservative: litter 0.35 × 58.5 = 20.475). Energy spent between the gate read and the reproduce step (cognition, earlier queued actions) is outside that margin, so the viability test `founders_only_run_has_no_energy_rejected_reproduce_attempts` is the tick-level check; a failure there is escalated, not re-pinned. |
| 4 | **Steal.** The effort is debited where the cost is today, before the target resolves; a zero or invalid requested amount pays `NoOp`'s effort. Transfer, kill and bonus rules are unchanged. |
| 5 | **Accounting.** `EnergyFlows.failed_action_penalty` and `DeathCause::FailedActionPenalty` are retired. A new `EnergyFlows.failed_action_effort` (`move`, `eat`, `reproduce`, `steal_energy`; `NoOp` cannot fail) adds the same applied debit that `action_charges` records whenever the attempt failed. It is a subset, not a sink (`failed_action_effort.k <= action_charges.k`), so mortality and every balance are unchanged by it. Definition tokens advance: `applied-energy-flows-v2` from the commit that adds the subset, `applied-mortality-v2` with the retired cause. |
| 6 | **Goal report.** Each goal case gains `action_attempts`, kept in the summary as `input_use` is: `eat_attempted_total` and `eat_failed_total` summed over every food-type key of the two `SimStats` eat maps (unconfigured types included); `reproduce_attempted_total`; `reproduce_rejected_by_reason` keyed by the four rejection results' `as_key()`; and `reproduce_rejected_litter_total` from a new `SimStats` counter of Step 7's branch (`transfer <= 0.0` or under `initial_energy`). Result classification is unchanged: both branches stay `RejectedEnergyConstraints`, and the parent-energy rejections (Step 6) are that reason's count less the litter count. Moves and steals are read from the kept `moves_*` and `predation` blocks. A report stored before this feature reads the new blocks as absent, never zero. |
| 7 | **Determinism.** No production RNG draw is added or removed; the same seed and config give the same trajectory. Trajectories leave T11.F29's at the first attempt whose price changed (an `Eat`, a reproduce attempt or a steal), in the gate and the goal profile and in every seeded test run, founders-only runs included: founders eat, so founder trajectory pins (such as `founder_only_trajectory_digest_is_pinned`, at both gestation rates) move and are re-pinned like any trajectory value (T16.F01's "a founder-only pin that moves is a defect" does not carry over). The founder genome, its digest, `FOUNDER_GENOME_SIZE_UNITS`, the founder margin (3a) and the scenario-drawn founder readings in the gate report (`steering-v1`, T11.F14 `mesh_execution`) do not change. |

| Surface (invariant 8) | Retired | Replacement or kept |
| --- | --- | --- |
| Engine (`v3-core`) | `debit_failed_action`, `ActionContext.failed_action_penalty`, the four failure debits in `simulation/tick.rs`, `failed_action_penalty_for_tick`, `failed_action_penalty_ramp_active` | — |
| Config | `energy.costs.failed_action_penalty`; the whole `startup` section (`StartupConfig`, `StartupRampsConfig`, `FailedActionPenaltyRampConfig`), which holds only this ramp; `apply_startup_overrides` and its callers (`resolve_config`, the v3-cli bench profiles, the v3-lab arena); the ramp checks in `resolve_config` | A recipe or patch naming a retired key fails to resolve and names it (`deny_unknown_fields`; a test as T11.F20's); the seven tracked root `world-recipe-*.json` lose `failed_action_penalty` and `startup`, keep every other value, and still load |
| Server | `patch_touches_failed_action_penalty` and the ramp lock; `patch_touches_startup` | A patch naming a retired key is rejected as an unknown field |
| Frontend | `StartupRampsSection`, the ramp types and store fields, the `ControlBar` restart skip, the `RuntimeFieldGroup` ramp lock, the penalty field in `EnergyCostsSection` | `EnergyCostsSection` gains `reproduce_attempt_cost`, shows the `eat_cost` default 0.05 and a `Noop Cost` minimum above 0; the tooltips of the five effort fields say they are charged on every attempt and never below `Noop Cost`, and `Reproduce Cost` says it is the replication cost paid only on a conception |
| Telemetry | flow key `failed_action_penalty` and mortality cause `failed_action_penalty`; `petri.failed_action_penalty` on the actions phase and the window tick; `petri.penalty` on window action events (`AppliedAction.penalty`, `FlowMarks.penalty`, `TickOutcome.failed_action_penalty`) | Telemetry table below |
| Bench (`v3-cli`) | `EnergyFlowTracking.failed_action_penalty` | `failed_action_effort` and `action_attempts` (invariants 5–6); the progress page still renders stored summaries with and without the retired keys |
| Lab (`v3-lab`) | `Setup::new` reading the ramp's target tick; the `penalty` tally and the `penalty_charged` score field; the exploration probe's penalty overrides | The scene start tick stays 62,680 as a lab constant, so no other tick-dependent input moves; campaign `ROW_VERSION` 3 → 4 (the scene score lost `penalty_charged`), `SUMMARY_VERSION` unchanged; the ignored probe's refund rules refund `failed_action_effort.reproduce`, so its pre-feature readings are not comparable; mechanism-test fixtures that no longer calibrate under the new prices take new seeds (old and new in the readings) |
| Reference docs | the penalty, ramp and patch-lock text in `v3-runtime-config-spec.md` and `v3-server-api-protocol-spec.md` | Those two, `v3-tick-orchestration-spec.md` (per-action effort and floor) and `v3-reproduction-spec.md` Sections 5 and 6 (Step 0; the text that says a rejected parent pays nothing and that age precedes any charge) state invariants 1–4 |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.energy_flow` with `petri.flow` = `failed_action_effort.move`, `.eat`, `.reproduce`, `.steal_energy` | counter (sum), new keys in the existing family; `metrics/tests.rs`'s expected flow set updated | applied effort debits on failed attempts (invariant 5) | `run.started` body carries `energy.costs` and `predation` |
| `energy.costs.reproduce_attempt_cost` | recorded configuration | `SimulationConfig` | `run.started` body (no edit) and `petri.config.energy.costs.reproduce_attempt_cost` in the actions phase group (`trace.rs` `actions`) |
| `petri.run.reproduction_actions_rejected_litter` | counter (monotonic sum), new: a family in `metrics.rs`, its row in `metrics/tests.rs`, its Prometheus name in `scripts/telemetry-dashboards-check` `always_names` | Step 7 rejections (invariant 6) | `energy.lifecycle` in the `run.started` body |
| `petri.run.reproduction_actions_attempted`, `_rejected_by_reason` | counter, existing | the same counters, now counting effort-paying attempts | — |
| `failed_action_penalty` flow and mortality keys, `petri.failed_action_penalty`, `petri.penalty` | retired | — | series and attributes absent from this feature on, never zero |

Overhead check (the feature adds exported series): `scripts/telemetry-overhead`
with its own method (the T21 workload, pairs set by the five-run spread, the
300 s cap) on the running `petri-telemetry` stack, after benchmark-affecting
work is final. Predeclared: under the 1.10 ceiling, since the export changes
by a few data points per snapshot. An inconclusive or failing result is
reported under the blocker rule, never rounded to a pass.

## Implementation Tasks

- [x] Observation commit first, behavior-neutral and committed alone:
      `failed_action_effort` (flows token v2), the litter counter and the
      goal `action_attempts` block through `v3-core`, the bench tracking, the
      summary keep-list and telemetry, with every trajectory pin unchanged.
      Its build is the control (Verification).
- [x] TDD: failing tests for the cases below, then the engine change (the
      floor helper on every action charge, the `eat_cost` default, Step 0,
      the steal floor, the four failure debits removed).
- [x] Retire every surface in invariant 8; strip the seven tracked recipes.
- [x] Panel and frontend tests; reference documents.
- [x] Re-pin moved trajectory values with the reason in the test and old and
      new values in the readings; a fixture seed whose run no longer reaches
      the tested path (extinction, an uncalibrated tiny grid) is replaced the
      same way.

| Property | Cases |
| --- | --- |
| Floor helper (proptest) | any finite `base >= 0`, `noop_cost > 0`, complexity, age: effort at least `NoOp`'s and above 0; equal to `adjusted_action_cost(base, …)` when `base >= noop_cost` |
| Failure pays effort only | blocked move per cause; eat with no food, configured and unconfigured type; reproduce rejected on each of the five branches (the litter counter only on Step 7); steal with no victim: stored energy is `before − effort` in `f32`, `action_charges` and `failed_action_effort` each rise by `applied_debit(before, after)`, and no other flow moves at the action's own debit |
| Lifecycle | an effort that exhausts the creature: energy floored to 0 with `zero_floor_credit`, the creature removed, the action's sink as the death cause, and no later action charged; a steal whose effort exhausts the attacker still transfers as today; a gestating parent and the actions discarded after a conception are never charged |
| Success | `NoOp` and `Move` charges bit-identical at defaults; eat credits then pays effort; conception pays effort, cost, transfer in that order, litter from post-effort post-cost energy; steal requesting 0 pays the floor |
| Config | `noop_cost <= 0` or non-finite falls back to 0.05; `reproduce_attempt_cost` invalid falls back to 0.05; defaults pinned; each retired key rejected by name; the seven tracked recipes load |
| Surfaces | the server rejects a patch naming a retired key; the panel has no ramp section or penalty field and shows the new field; the lab runs with no ramp |

## Verification

- [x] `cargo test -p v3-core --test viability` exits 0 before and after the
      engine change.
- [x] Focused tests (`v3-core` actions, tick and config, `v3-server`,
      `v3-cli` bench, `v3-telemetry`, `v3-lab`, frontend): commands and
      counts in the readings, current working tree.
- [x] `make check` exits 0; tested commit in the readings. Before the
      re-pin, a failure of the series test
      `gate_profile_has_no_severe_regression_against_series_references` is
      recorded and attributed to the predeclared trajectory move, not fixed.
      After the epoch re-pin, `make check` exits 0 on the working tree over
      `3d6b5865` (readings, "Epoch re-pin").
- [x] Control: one goal run built from the observation commit,
      `make bench PROFILE=goal FEATURE=t16-f02-effort-priced-actions-control
      SUMMARY_OUT=<main>/.bench-artifacts/t16-f02-effort-priced-actions-control/summary.json`
      (raw and summary stay local; no committed summary, no series entry).
      Every deterministic field T11.F29's goal report carries, the flows
      definition token aside, is equal in it (the observation commit is
      behavior-neutral); its new blocks give the
      "before" of the failed-action share and the reproduce rejection fraction.
      Path, hash, bytes and per-world values in the readings.
- [x] Benchmark summaries at
      `docs/progress/features/t16-f02-effort-priced-actions.json` and
      `-goal.json`, local raw hash, bytes and verification time in the
      readings, series entries pointing to them, no full report staged; both
      epochs re-pinned at the point the Performance section names.
- [ ] Overhead check (Telemetry): verdict, median ratio and interval, pairs,
      measured time and telemetry's self-timed cost per tick in the readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the re-pin: summary
      line, output path, and every survivor resolved as killed, equivalent or
      deferred, listed here.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a muscle burns
energy whether or not the act succeeds (Polyworld charges activation, not
outcome; a blocked move pays the movement and no more), and a courtship is
spent whether or not a mate results (Wang et al. 2025, *Ecology and
Evolution* 15(8): e71932). It reaches creatures through the body's energy
ledger alone: no sensor, operator or reward changes.

Compute: one comparison per action charge, one subtraction per reproduce
attempt, one `f64` add per failure; nothing measurable per creature-tick by
itself. Every work counter can move with the trajectory.

References: T11.F29's gate and goal summaries are the gate epoch, the
goal-worlds epoch and the previous closure, so one reference serves each
profile; the control supplies the two "before" readings no stored report
carries. Standard thresholds: counters flag at 10%, severe at 50%; wall flags
at 25%, severe at 100%; `wall_clock: null` on a host mismatch. Existing
observation caps and accepted cap exceptions stand. The goal profile runs
once; per-world values are Orchards 11 / Canyon 22 / Confluence 33; every
indicator not in the table is reported with no direction.

**Epoch re-pin.** The row expects both trajectories to move. After the gate
and goal readings are recorded and before the mutation gate runs (its
baseline runs the series test, which compares the gate against the epochs),
`gate.epoch_baseline` and `goal_worlds.epoch_baseline` in
`docs/progress/benchmark-series.json` point to this feature's summaries and
both join `closed`. A severe flag, an extinction
or a reading under the persistence floor is reported first and decided under
the blocker rule before the re-pin; the re-pin never decides it.

| Reading | Reference (T11.F29) | Predeclaration |
| --- | --- | --- |
| Gate `deterministic` block | T11.F29 gate | Differs: founders pay the eat effort from their first `Eat`; counters read under the standard thresholds, no direction |
| Gate founder `steering-v1`, T11.F14 founder `mesh_execution` | 48 scenarios, 24 exact, 36 within-45, 0 of 48 avoided | Identical (invariant 7); a difference is investigated before the re-pin, and one not explained by the scenario inputs is a defect to fix |
| Goal `failed_action_penalty` flow; its mortality cause | 305,770.55 / 198,105.32 / 380,726.72; 5,637 / 4,978 / 9,489 deaths | Absent in every world |
| Goal per-attempt prices: `action_charges.eat` / `eat_attempted_total`, `failed_action_effort.eat` / `eat_failed_total`, `action_charges.steal_energy` / `predation.actions_attempted_total`, `failed_action_effort.reproduce` / reproduce rejections, `failed_action_effort.move` / blocked moves; a zero denominator is `Undefined` | eat and steal charges 0 | At least 0.04998 each, the last at least 0.19998: the nominal floor less half an `f32` ulp below energy 256 (invariant 1); the applied-behavior check of invariants 1–4 |
| Failed-action share: `failed_action_effort` / `action_charges` after; (`failed_action_effort` + penalty) / (`action_charges` + penalty) before, from the control | control | The effort cost alone after (track criterion 1), penalty component 0. No direction for the total: at 2,000 ticks the ramped penalty is at most 0.032 per failure while failed eats and rejected reproduces now pay 0.05; the frozen-count projection (readings) puts Orchards up, Canyon and Confluence down before reproduce rejections count |
| Action energy per creature-tick (charges plus penalty before, charges after) | 0.358951 / 0.230691 / 0.289009 | Expected to rise in every world (added eat and steal effort 0.0508 / 0.0262 / 0.0360 against a retired 0.0198 / 0.0152 / 0.0221 at frozen counts); behavior can move it, and a fall is a recorded miss |
| Blocked-move fraction, all causes; barrier-only `blocked_move_fraction` | 0.152969 / 0.425660 / 0.372254; 0 / 0.199465 / 0.111730 | Read before and after, no direction |
| Reproduce attempts rejected on parent energy (Step 6: `RejectedEnergyConstraints` less `reproduce_rejected_litter_total`) / `reproduce_attempted_total`; the litter share beside it | control | Read before and after, no direction: whether a 0.05 attempt effort retains the gate is the note's open question |
| `steering-v1` pooled exact / within-45 / avoidance fractions | 0.3776 / 0.6308 / 0; 0.3799 / 0.6201 / 0; 0.3750 / 0.6076 / 0 | Read before and after, no direction (track criterion 4, now on a substrate without the penalty) |
| T11.F14 `mesh_summary` executed / reachable / `routeVaries` / knockout | 44 / 49 / 5 / 12; 47 / 54 / 6 / 19; 30 / 38 / 0 / 6 | Read, no direction |
| Mortality by `action_eat`, `action_reproduce`, `action_steal_energy` | 0 everywhere | May become nonzero; recorded |
| Persistence: final, plateau | 3,404 / 3,098 / 5,122; 6,365.31 / 5,431.22 / 8,903.87 | Floor: no world extinct, final and plateau each at least half the reference; under it is a blocker |
| Work counters per creature-tick, `births`, wall | T11.F29 | No direction; standard thresholds; wall flat when the host matches |
| `config_digest` per case | T11.F29 | Changes (new field, new default, retired section); `inputs_changed` recorded |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t16-f02-effort-priced-actions
make bench PROFILE=goal FEATURE=t16-f02-effort-priced-actions
```

**Measured verdict (2026-10-10, revision `561c1454`; values per world in the
readings).** Gate and goal both exit 3 (`make` 2) with `severe` true against
T11.F29, so the blocker rule applies before any re-pin: gate `pass_cap_hits`
+59.1%; goal `vm_steps` +129.9% and `decided_passes` +1262.8%, with flags on
`plasticity_updates` +37.3%, `pass_cap_hits` +34.3%, `graph_relax_iters`
+12.5%, `mesh_hops` +12.3% and `actions_applied` +10.0%; wall ok (-4.3%,
-8.6%). Met: founder `steering-v1` and `mesh_execution` byte-identical;
penalty flow and mortality cause absent in every world; every per-attempt
price at least its floor (eat 0.0510 to 0.0524, steal 0.0507 to 0.0514,
rejected reproduce 0.0509 to 0.0548, blocked move 0.2017 to 0.2124); penalty
component of the failed-action share 0; action energy per creature-tick up in
all three worlds; persistence floor (no extinction; final 5,092 / 2,056 /
5,248 and plateau 4,263 / 4,233 / 5,142 against half of T11.F29's); digests
changed. Read, no direction: failed-action share down in every world
(0.187 / 0.429 / 0.398 against 0.196 / 0.459 / 0.418), blocked-move fractions,
parent-energy rejection fraction (0.257 / 0.244 / 0.248 against 0.294 / 0.234
/ 0.223) and litter share, `steering-v1` pooled, `mesh_summary`, and mortality
by `action_eat` / `action_reproduce` / `action_steal_energy` (now nonzero).
Overhead check: inconclusive before any pair (24 pairs at 7,826 ms do not fit
the 300 s cap; 37.5 s used), not rounded to a pass.

- Summaries: [gate](../../progress/features/t16-f02-effort-priced-actions.json),
  [goal](../../progress/features/t16-f02-effort-priced-actions-goal.json).
- Full readings: [`docs/progress/readings/t16-f02.md`](../../progress/readings/t16-f02.md).

## Success Criteria

- [ ] Every committed action pays at least `NoOp`'s nonzero effort on every
      attempt; a failure pays only its effort; a reproduce attempt pays its
      effort before the gates and the replication cost only on conception.
- [ ] `failed_action_penalty`, its ramp and the `startup` section are gone
      from engine, config, server, panel, telemetry, lab and reference
      documents; retired keys fail to resolve by name; the tracked recipes load.
- [ ] The goal report carries `failed_action_effort` and `action_attempts`, and
      every reading in the predeclaration table is recorded against its
      reference, the control's included.
- [ ] Both epochs re-pinned after the readings and before the mutation gate;
      the mutation gate resolved.

## Notes for AI Agents

- Decision: (user, 2026-10-10) standing instruction: the user is not
  consulted during this run; a decision that would go to the user is the
  spec owner's recommendation, adopted by the orchestrator and recorded as
  accepted without consultation.
- Decision: (spec owner, 2026-10-10) the floor is one charge-time rule,
  `max(base, noop_cost)` on every action, with `noop_cost` required positive;
  normalization does not raise other costs (Inputs, options table).
- Decision: (spec owner, 2026-10-10) the row's "before" for the failed-action
  share and the reproduce energy-rejection fraction comes from a control goal
  run of the observation-only commit, because no stored report carries
  reproduce rejections by reason or the effort of failed attempts; the
  control is local evidence, not a summary or series entry.
- Decision: (spec owner's recommendation, accepted without consultation
  under the user's standing instruction of 2026-10-10) the severes are
  accepted as the predeclared trajectory move: gate `pass_cap_hits` +59.1%
  (113 extra hits in about 260,000 creature-ticks, seeds 22 and 33), goal
  `vm_steps` +129.9% and `decided_passes` +1262.8% (87% of decided passes in
  Orchards, which ends as one lineage of 5,092; actions per creature-tick
  there +27.1%). The diff adds no VM, mesh or pass work, founder in-vitro
  readings are identical and wall fell, so the moves are the genotypes the
  new prices selected ([readings](../../progress/readings/t16-f02.md),
  "Severe attribution"). Both epochs re-pin to this feature's summaries, with
  both `closed` entries, before the mutation gate; thresholds unchanged.
- Exception: (spec owner's recommendation, accepted without consultation
  under the user's standing instruction of 2026-10-10) the overhead check is
  inconclusive before any pair (24 pairs at 7,826 ms per run exceed the 300 s
  cap; a rerun sets the same count) and the 1.10 ceiling is unmeasured for
  this feature, as at T03.F12; the diff's export change is a few data points
  per snapshot.
- Deferred: the goal summary records `dirty: true` from the gate's untracked
  summary file present at the run; the tracked tree was clean at `561c1454`,
  and no deterministic value depends on it (as at T16.F01).
