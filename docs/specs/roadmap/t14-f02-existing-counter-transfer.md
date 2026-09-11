# T14.F02 — Existing Counter Transfer

**Status**: In Progress
**Last updated**: 2026-09-11
**Feature**: T14.F02
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Counters the simulation already maintains reach the stored closure report
instead of being discarded when the process exits. Each goal case's
`WorldTracking` block carries the production mutation supply and its
target split, the applied-behavior integer sums from `mutation_outcome_summary`
and `mutation_value_totals_by_operator`, the predation counters, failed eats by
the food type that was requested, and the number of mesh dispatches that ended
with the creature out of energy. The one counter that is structurally constant
is removed rather than transferred: `mutation_events_applied_total_semantic_noop`
and its twin `_semantic_change` leave the runtime, the server payload and the
frontend protocol together with the one-armed `MutationSemanticCategory` enum
that produced them.

## Non-Goals

- Any new mechanism, charge, rate, default or threshold. This feature records;
  it changes nothing the simulation applies.
- Death counting and attribution, and the energy flow totals — T14.F03.
- Checkpoint population readings (T14.F04), cognition telemetry resolution
  (T14.F05), the per-clade behavioral table (T14.F07), the progress page
  (T14.F11).
- `MutationValueTotals::final_energy_sum`: excluded by the track's F02 note.
  Verified zero on every benchmark path — it is only accumulated in
  `Simulation::remove_creature` (`crates/v3-core/src/simulation/simulation.rs:86`),
  which reads `creature.energy.max(0.0)`; starvation removal happens at
  energy ≤ 0, a predation kill removes the victim only at energy ≤ 0
  (`crates/v3-core/src/simulation/actions/predation.rs:110`) and bypasses this
  funnel entirely, and creatures alive at the horizon are never removed. Only
  server-side paint-stroke eviction
  (`Simulation::apply_paint_stroke`, `crates/v3-core/src/simulation/simulation.rs:80`)
  reaches the funnel with positive energy, and no benchmark paints. A
  transferred column would be a permanent zero.
- `viability_score_sum`, `viability_score_delta_sum`,
  `mean_lifetime_energy_sum`, and the six classification counters
  (`helpful_total`, `neutral_total`, `detrimental_total` and the three
  confidence totals): the score sums are the composite observability score the
  observation contract excludes, the classification counters are bucketings of
  that same score via `classify_mutation_outcome` and go with it, and
  per-creature energy means are T14.F03's accounting. Only the
  applied-behavior integer sums of `MutationValueTotals` transfer.
- Rewriting historical reports. A report stored before these blocks existed
  reads them as absent, never as zero.
- Any change to the goal profile's indicator set, thresholds or epoch.

## Inputs and Invariants

Sources of truth: `crates/v3-core/src/simulation/stats.rs` (`SimStats`,
`MutationValueTotals`), `crates/v3-core/src/simulation/tick.rs` (the Phase 2
action execution and the queue-order reduction documented at lines 541–543),
`crates/v3-core/src/runtime/mesh.rs` (`MeshOutput`, `MeshExecutionMode`,
`TerminationReason`), `crates/v3-cli/src/bench.rs` (`WorldTracking`,
`GoalCaseObservation`), and the track's F02 note.

**Determinism is the binding constraint.** Seeded runs reproduce byte-for-byte
across processes and thread counts, and the gate profile's two-run
byte-identical check runs inside `make check`. Two consequences the
implementation is bound by:

- No `HashMap` reaches the report. `WorldTracking` uses `Vec` and fixed structs
  today for exactly this reason. Every keyed collection transferred here is
  serialized as a `BTreeMap<String, _>` keyed by the operator, result or cause
  name, or as a `Vec` indexed by food type, so iteration order is fixed.
- No counter is incremented from inside the parallel mesh phase. The
  `EnergyExhausted` total is accumulated in the existing sequential queue-order
  reduction over `MeshOutput`, where integer sums commute and order is fixed.

**Two counters are created, not transferred.** Both are values the runtime
already computes at an existing decision site and discards; the track's F02 note
names both as in scope.

| Counter | Site | Why it is not a read |
| --- | --- | --- |
| `eat_actions_failed_total_by_type` | the `NoFood` branch of `execute_eat`, `crates/v3-core/src/simulation/tick.rs` | only applied eats are counted today (`eat_actions_applied_total_by_type`); the failure branch sets `ActionResult::NoFood`, debits the failed-action penalty and keeps no total |
| `mesh_dispatches_energy_exhausted_total` | the untraced production mesh path | `TerminationReason::EnergyExhausted` is returned by `execute_creature_mesh_impl` and reaches only the traced single-creature path; `MeshOutput` does not carry it |

Everything else in this feature is a read of a field `SimStats` already
maintains. `MeshOutput` gains a `termination_reason: TerminationReason` field
(a `Copy` enum, no allocation) so the production path can report it.

**The constant counter.** `MutationOperator::semantic_category` is a `const fn`
with no match arm — it returns `MutationSemanticCategory::SemanticChange` for
every operator (`crates/v3-core/src/mutation/types/mod.rs:322`). So
`mutation_events_applied_total_semantic_noop` is always zero and
`_semantic_change` always equals `mutation_events_applied_total`. Both are
exposed over the server payload and typed in the frontend protocol. The pair,
`semantic_category`, and the `MutationSemanticCategory` enum are deleted
together; keeping one arm of a two-armed split would leave a duplicate total
labelled as a distinction that is not made.

**Report shape.** New blocks are `Option<T>` with `#[serde(default)]`, following
the `moves_blocked_total_by_cause` precedent, so a historical report
deserializes them as `None`. Each block is one struct with named integer
fields. They belong to the end-of-run per-case block only: `WorldTracking::observe`
leaves them absent, `with_transferred_counters` adds them at the single
end-of-run site, and they are `skip_serializing_if = "Option::is_none"` so a
checkpoint sample carries exactly the field set it carried before this feature.

## Implementation Tasks

- [x] Add `eat_actions_failed_total_by_type` to `SimStats` and increment it in
      the `NoFood` branch of `execute_eat`.
- [x] Carry `termination_reason` on `MeshOutput` and accumulate
      `mesh_dispatches_energy_exhausted_total` in the existing sequential
      queue-order reduction.
- [x] Delete `mutation_events_applied_total_semantic_noop`,
      `mutation_events_applied_total_semantic_change`, `semantic_category` and
      `MutationSemanticCategory` from `v3-core`, and their carriers in
      `v3-cli`, `v3-server` and `frontend/src/types/protocol.ts`, with the
      reference specs under `docs/reference/` updated to match.
- [x] Extend `WorldTracking` with the optional transferred blocks — mutation
      supply and target split, `mutation_outcome_summary` integer fields,
      per-operator integer value totals, predation counters and results,
      failed eats by requested type, energy-exhausted dispatches — all
      deterministically ordered, and populate them at the end-of-run per-case
      site only, leaving checkpoint samples at their previous shape.
- [x] Tests: the transferred report values equal the `SimStats` values that
      produced them; a report stored without the blocks loads with them absent;
      a failed eat of a type with no food increments only that type; a
      dispatch that runs out of energy increments the exhausted total.

## Verification

- [x] Focused tests or checks: `cargo test -p v3-core --test viability` ->
      24 passed, 0 failed; `cargo test -p v3-core` -> 1362 passed and 3 ignored across eight
      binaries, 0 failed; `cargo test -p v3-cli` -> 120 passed across four
      binaries, 0 failed; `cargo test -p v3-server` -> 143 passed across three
      binaries, 0 failed (all via `make check`, which runs the per-binary targets);
      `make check` -> exit 0 on the final code.
- [x] Byte-identical reproduction across processes and thread counts:
      `crates/v3-core/tests/reproducibility.rs` inside `make check` ->
      3 passed, 0 failed.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path, and
      every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here; `docs/workflow.md` requires it in the spec.
- [x] Benchmark report stored at
      `docs/progress/features/t14-f02-existing-counter-transfer.json` and
      `...-goal.json`. Both were regenerated on the post-review code, because
      remediation changed what a report contains and a stored report must be
      what the shipped code produces; the once-per-closure rule of 2026-09-05
      governs the second goal run for the `deterministic` block, not this.
- [x] Second goal run for the `deterministic` block: `Not applicable` — the
      goal profile runs once per closure and cross-process reproducibility is
      covered by `reproducibility.rs` inside `make check` (user decision,
      2026-09-05).

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement feature; the
track's observation contract exempts it from the natural-analog rule and it
adds no mechanism. References: the gate and goal profiles compare against the
epoch baselines the series index names, under the existing thresholds.

Expected compute cost: negligible and at most one integer increment per failed
eat and one per creature-tick for the termination reason, both outside any
allocation. No epoch re-pin is expected and none is authorized here. Predeclared
direction for every goal indicator: **none** — this feature changes nothing the
simulation applies, so lineage diversity, memory sensitivity, temporal memory
sensitivity, mutational neighborhood, drift depth, population persistence and
births per 100 ticks are all expected to be unchanged, and any movement in them
is a defect rather than a result. A severe compute regression would be a
blocker, not a cost to justify.

Goal impact: the failed-eat and energy-exhausted totals, together with the
transferred mutation supply and target split, are what make T03.F10's
lethality claim and T11.F17's targeting delivery re-readable at any later
closure rather than only in the session that measured them.

**Measured verdict.** Gate: exit 0, `severe=false` against both references,
no threshold crossing, no epoch re-pin — every metric reads 0.000000 % against
the T14.F01 gate report. Goal: exit 0, `severe=false` against both references,
no epoch re-pin, and every metric and every per-case indicator reading is
identical to T14.F01's stored goal report (0.000000 %), so the predeclared
direction of **none** holds; the one `flag` level, `plasticity_updates`
+40.886836 % against T12.F04, is the value T14.F01 already stored against that
same reference and is not moved by this feature. Caps: evolved neighborhood
579.48 ms of 180,000, founder neighborhood 130.82 ms of 10,000, goal profile
503.9 s of the 15-minute budget. All six transferred blocks are present and
non-null in the end-of-run block of all three world cases and absent from every
checkpoint sample, whose key set matches T14.F01's stored report; the goal
report is 74,587 lines and 3.11 MB.

- Reports: [gate](../../progress/features/t14-f02-existing-counter-transfer.json),
  [goal](../../progress/features/t14-f02-existing-counter-transfer-goal.json).
- Full readings: [`docs/progress/readings/t14-f02.md`](../../progress/readings/t14-f02.md).

## Success Criteria

- [ ] Every counter the track's F02 note names is readable in the stored goal
      report, per case, except the two it excludes.
- [ ] No transferred value is structurally constant, and no `HashMap` iteration
      order reaches the report.
- [ ] `mutation_events_applied_total_semantic_noop` and its twin no longer
      exist anywhere in the repository.
- [ ] A report stored before this feature loads with the new blocks absent.
- [ ] Byte-for-byte reproducibility across processes and thread counts is
      preserved.

## Notes for AI Agents

- Decision: the constant-counter pair is removed rather than repaired.
  `mutation_events_applied_total_semantic_noop`,
  `mutation_events_applied_total_semantic_change`, `semantic_category` and
  `MutationSemanticCategory` are deleted together; giving the predicate a real
  runtime silence test would duplicate T11's neighborhood battery, which owns
  that question.
- Exception: the orchestrator for this feature ran as Opus 5 at effort
  `medium` in place of `docs/workflow.md`'s Fable 5.1, authorized by the user
  in the goal command on 2026-09-11. Implementer and reviewer models are
  unchanged.
- Decision: `final_energy_sum` is never transferred to a report. It is zero on
  every benchmark path, and only server-side paint-stroke eviction can make it
  non-zero; a later feature that wants energy at death defines it in T14.F03's
  accounting rather than reviving this field.
- Deferred: P3 review finding, the trace-domain `TerminationReason` should be
  defined in `crates/v3-core/src/runtime/types.rs` and re-exported from
  `trace::domain` rather than imported across that layer boundary.
