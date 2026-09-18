# T16.F01 — Gate Before Charge in Reproduction

**Status**: In Progress
**Last updated**: 2026-09-17
**Feature**: T16.F01
**Track**: [T16 — Cost and Cue Fidelity](../../roadmaps/t16-cost-and-cue-fidelity.md)

## Goal

An animal that cannot afford a litter does not conceive. In
`apply_reproduce` (`crates/v3-core/src/simulation/actions/reproduction.rs`)
the energy gate and the transfer feasibility check are evaluated on the
parent's energy *minus* the would-be charge before anything is deducted. A
reproduce attempt rejected as `RejectedEnergyConstraints` leaves the parent's
energy and `energy_flows.action_charges.reproduce` exactly as they were; a
successful birth charges what it charges today and the parent ends at the
same `f32` value as before this feature.

## Non-Goals

- The gate condition, the charge formula (T03.F11 replication multiplier,
  age and complexity multipliers), `min_reproduce_energy`, the transfer
  clamp, and the founder's own node-0 reproduce threshold are unchanged.
- The `failed_action_penalty` that `execute_reproduce` (`tick.rs`) debits
  after every non-`Spawned` result is untouched; retiring it is T16.F02. This
  feature's "costs nothing" claim is about the reproduce charge inside
  `apply_reproduce`, and the tests are written at that boundary.
- No new sensor, operator, assay, report field, config field, or panel
  surface. `reproduction_actions_rejected_by_reason` keeps counting energy
  rejections; the `DeathCause::ActionReproduce` and `ParentalTransfer` sinks
  stay as accounting sinks even though a valid config can no longer reach
  them through a rejected attempt.
- Closed feature specs (T03.F11) are history and are not edited; the living
  reference documents are.
- The survey's paired-perturbation probe (track criterion 4) is a recorded
  deferral, not a non-goal: see the `Deferred:` bullet in Notes for AI Agents.
  The T11.F14 readings the goal summary already carries are recorded with no
  predeclared direction.
- The T12.F04 three-environment pressure-integration rule does not apply:
  this feature adds no environmental pressure, so the goal-world recipes are
  untouched.

## Inputs and Invariants

Sources of truth: the T16.F01 row and the track's "Order" and "expected to
leave the founder gate unchanged" notes; the
[antipattern review](../../strategy/antipattern-review-2026-09-16.md)
Section 3 (109 `RejectedEnergyConstraints` at about −10.8 energy each against
315 births in the live survey's 400-creature sample, where a rejected attempt
cost half the median creature's energy); `apply_reproduce` Steps 1–8;
`docs/reference/v3-reproduction-spec.md` Sections 5–6 and
`docs/reference/v3-runtime-config-spec.md` "Genome replication cost", both of
which currently document the charge-before-gate order.

Current order (Steps 5–8): charge `cost = adjusted_action_cost(reproduce_cost,
cached_complexity, age) × genome_replication_cost_multiplier(rate,
cached_genome_size)` and record it through `observe_energy` into
`action_charges.reproduce`; reject if `energy < min_reproduce_energy`;
compute `transfer = min(clamp_non_negative_finite(request),
default_offspring_energy)` and reject if `transfer <= 0.0 || energy <
transfer`; deduct `transfer` into `parental_transfer_debit`. Both rejections
return after the charge landed. The age gate at Step 4 is already ordered
before the charge and pinned by
`apply_reproduce_age_rejection_does_not_charge_reproduce_cost`
(`actions/mod.rs`); that test is the local precedent to mirror.

Options considered (research proportional to a bug-sized change): (a)
evaluate both gates on `energy - cost` before deducting, then deduct cost and
transfer in today's order — the review's recommendation and Avida's divide
(validity first, a failed divide spends nothing); (b) charge then refund on
rejection — leaves a spurious `observe_energy` crossing and flow noise; (c)
an advisory precheck in `execute_reproduce` — the reproduction spec's
ownership boundary says authoritative acceptance is emitted at action
application, so the gate must stay inside `apply_reproduce`. (a) is chosen.

Invariants:

1. Acceptance predicate unchanged. With `after_cost = energy - cost` computed
   in `f32`, the attempt is rejected as `RejectedEnergyConstraints` iff
   `after_cost < min_reproduce_energy`, or `transfer <= 0.0`, or
   `after_cost < transfer`. Every attempt accepted today is accepted after,
   and vice versa, including the `transfer <= 0.0` branch.
2. Rejection is free at the `apply_reproduce` boundary: parent energy,
   `action_charges.reproduce`, `parental_transfer_debit`, and
   `pending_death_cause` are unchanged; `reproduction_actions_rejected_total`
   and the `RejectedEnergyConstraints` reason count still increment.
3. Birth arithmetic is bitwise stable: `cost` is the same product expression
   as today, and on acceptance the parent pays `cost` then `transfer` as two
   successive `f32` subtractions in that order, so its post-birth energy, the
   two flow debits, and the offspring's initial energy are the values the
   pre-feature engine produces. The T03.F11 multiplier
   pins and the founder's charge are therefore unchanged.
4. Rejected attempts no longer contribute to `action_charges.reproduce`, so
   in any run that flow accumulates only over births.
5. Determinism: same seed, same config, same trajectory. Trajectories diverge
   from the previous closure only at the first energy-gate rejection; a run
   with none is bitwise identical to T11.F22's.
6. The founder's node-0 threshold is `EnergyCurrent >= 30.0` while the engine
   gate is `energy - cost >= 30.0`, so a founder with energy in
   `[30, 30 + cost)` attempts and is rejected at Step 6. Whether that window
   is hit in the gate profile is verified by the benchmark (below), not
   assumed.

## Implementation Tasks

- [x] TDD first: failing tests on both `RejectedEnergyConstraints` branches
      (below `min_reproduce_energy`; transfer infeasible or non-positive)
      asserting zero parent energy change and zero
      `action_charges.reproduce` change, then the reorder in
      `apply_reproduce`. The reorder is inline (no predicate extracted:
      the gate reads config plus three creature fields, and a predicate
      compared against itself would be tautological), so no property
      test. A pure gate predicate, if extracted, gets a
      property test for invariants 1 and 3.
- [x] Update `docs/reference/v3-reproduction-spec.md` Sections 5 and 6
      (gate evaluated on `energy - cost` and transfer feasibility before any
      charge; "If the energy gate fails" clause beside the age clause) and the
      "Charged before the `min_reproduce_energy` and transfer gates" bullet
      plus the "Reproduction transfer sequencing" list in
      `docs/reference/v3-runtime-config-spec.md`. Step 5 stays the charge
      computation and step 8 the payment, so the step numbers the config
      spec cites and the `too_many_lines` reason string in `reproduction.rs`
      remain accurate.
- [x] Re-pin any evolved-trajectory test value that moves, with the reason in
      the test and the old and new values in the readings file; a founder-only
      pin that moves is a defect, not a re-pin. The `applied_trajectory`
      digest moved (evolved, re-pinned); two tests that pinned the
      charge-before-gate order itself were updated to the free-rejection
      contract; no founder-only pin moved (readings file, "Pins that moved").

## Verification

- [x] Focused tests in `actions/mod.rs` and `actions/reproduction.rs`
      (names in the readings file): both rejection branches free, the
      accepted path bitwise equal to the pre-feature values, the reason
      counter still incremented; `cargo test -p v3-core` -> exit 0.
      Self-review pass (uncommitted worktree, 2026-09-17): `cargo test -p
      v3-core --test viability` first -> 26 passed; `cargo test -p v3-core`
      -> 1560 lib + 89 integration passed, 0 failed, 3 ignored; `cargo check
      --workspace --all-targets` clean; `cargo clippy --workspace
      --all-targets -- -D warnings` clean; `cargo fmt --all -- --check`
      clean.
- [ ] `cargo test -p v3-core --test viability` first, then `make check` ->
      exit 0 on the final feature commit (hash in the readings file).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred; the
      survivor list stays here.
- [ ] `make bench PROFILE=gate FEATURE=t16-f01-gate-before-charge-in-reproduction`
      and one `PROFILE=goal` run: exit statuses, `severe`, summaries at
      `docs/progress/features/<feature>.json` and `<feature>-goal.json`,
      series entries, raw hash/byte counts in the readings file, no full
      report staged.
- [ ] Gate identity check: the gate summary's `comparison.references[]`
      entry whose `path` is the T11.F22 gate summary shows `percent_delta`
      `0.000000` on all six counters, and `deterministic.per_seed[*]`
      `final_population` and `births` match; the six deltas and the three
      per-seed pairs go in the readings file (predeclaration below).
- [ ] Orchards trajectory reading (before/after table, predeclaration below)
      recorded in the readings file.
- [x] `make roadmap-check` on the document edits -> "validation passed",
      exit 0 (2026-09-17, after the Task 2 reference-document edits).
- [ ] `make check-docs` on the document edits.

Readings file: `docs/progress/readings/t16-f01.md`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: conception is
gated on body condition; an animal below the threshold does not ovulate and
pays nothing for the litter it does not carry (Avida's divide checks validity
before it spends). It reaches creatures through the body's energy ledger
alone: no sensor, no operator, no reward.

Expected compute cost: none measurable — the same arithmetic in a different
order, one fewer subtraction on a rejected attempt.

References and thresholds. Gate: the epoch baseline
`docs/progress/features/remove-complementary-nutrition.json` and the latest
closure `t11-f22-meaning-stable-input-references.json`. Goal (goal-worlds-v1):
the epoch baseline `t11-f19-per-unit-mutation-supply-goal.json` and
`t11-f22-meaning-stable-input-references-goal.json`. Standard +10%/+50% work
and +25%/+100% wall flags; no cap change, severe allowance, or epoch re-pin is
predeclared. A severe result or an extinction in any goal world is a user
decision under the blocker rule.

**Gate profile — a predeclaration, not a reading.** The track note expects
the founder gate unchanged. The predeclared direction is *identical*: every
deterministic counter (`vm_steps`, `mesh_hops`, `graph_relax_iters`,
`plasticity_updates`, `actions_applied`, `births`) and per-seed
`final_population` equal T11.F22's gate summary bit for bit
(`percent_delta` 0.000000 on the previous-closure reference; T11.F22's gate
was measured at `a249acfa`, and only a test file changed between it and this
worktree's base `61ca4009`). That identity is the closure report's
verification that no founder crossed invariant 6's window in 75 ticks × 3
seeds. Any nonzero delta means a founder did fail the
energy gate; that outcome is read under the standard thresholds, reported as
a predeclaration miss, and the track note is corrected — it is not a
blocker by itself. Wall-clock moves are host noise and flag-only.

**Goal profile.** Every evolved counter may move from the first energy-gate
rejection onward. The one direction predeclared: `energy_flows.action_charges
.reproduce / births` falls in every world, because rejected attempts stop
feeding the flow (T11.F22 read 31,373.63 / 152,093 = 0.2063 in Orchards,
238,504.81 / 196,811 = 1.2119 in Canyon country, 190,730.72 / 261,980 =
0.7281 in Confluence). A rise is a predeclaration miss to be explained by the
age and genome-size multipliers of the birthing population.

**Orchards reading — explicitly not a gate and not an acceptance criterion.**
The user's hypothesis is that the pre-gate reproduce charge drives the
Orchards goal-profile population collapse. Before: T11.F22's goal summary
(`population_persistence.per_seed[0]`), population 96,680 at tick 100, 1,399
at 200, 127 at 300, 74 at 400, 30 at 1,000, 10 from tick 1,700 to 2,000;
peak 100,000 at tick 66, minimum 10, plateau 10.32, final 10. After: this
feature's goal run, same fields. The readings file carries the two
trajectories side by side (every 100-tick sample, peak, minimum, plateau,
final) with Canyon country and Confluence alongside. The spec predicts no
direction for it: in the T11.F22 Orchards run the reproduce flow that
rejected attempts could have contributed is at most about 16,000 energy
(31,374 total less at least 0.1 per birth) against 4,565,149 of lifecycle
decay and 1,854,021 of move charges, so the hypothesis is tested by the
reading rather than assumed by the predeclaration. Whatever it shows is
recorded in the measured verdict and the readings file; it does not gate
closure.

| Indicator | Predeclared direction |
| --- | --- |
| Gate: six work counters, per-seed `final_population` | identical to T11.F22 (verification of the track note) |
| Goal: `action_charges.reproduce / births`, per world | falls |
| Goal: `births` per creature-tick, `final_population`, `plateau_population`, per world | no predeclared direction; Orchards trajectory recorded as a reading |
| Goal: `vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied` | no predeclared direction; standard thresholds |
| Goal: `failed_action_penalty` flow | unchanged mechanism; moves only with the trajectory |
| Drift depth, lineage diversity, sensor census, recruitment, T11.F14 readings | no predeclared direction, recorded |

**Measured verdict.** Pending.

- Summaries: `docs/progress/features/t16-f01-gate-before-charge-in-reproduction.json`
  and `...-goal.json`.
- Full readings: [`docs/progress/readings/t16-f01.md`](../../progress/readings/t16-f01.md).

## Success Criteria

- [ ] A reproduce attempt rejected at the energy gate or the transfer check
      leaves the parent's energy and `action_charges.reproduce` unchanged,
      proven by tests on both branches.
- [ ] A successful birth costs the parent bit for bit what it cost before
      this feature; the T03.F11 pins and the founder's charge are unchanged.
- [ ] The reference documents state the new order; the code comment and
      spec agree.
- [ ] The gate identity check and the Orchards before/after reading are
      recorded in the readings file, whatever they show.

## Notes for AI Agents

- Decision: the Orchards goal-profile population trajectory is recorded
  before and after as a reading at the user's direction; it is not a gate
  and not an acceptance criterion of this feature.
- Decision: the failed-action penalty on a rejected reproduce stays in place
  until T16.F02; this feature's zero-cost claim is scoped to the
  `apply_reproduce` charge.
- Deferred: track criterion 4's paired-perturbation probe is not taken on this
  feature because it changes no input, sensor, or decision, only the ledger
  after a decision, so there is nothing to perturb; the probe genomes live in
  session scratchpads, not the repo, and the probe is taken at T16.F02/F03 on
  the substrate this feature produces. Surfaced to the user as a recorded
  deferral by the orchestrator.
