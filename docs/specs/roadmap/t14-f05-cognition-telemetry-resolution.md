# T14.F05 — Cognition Telemetry Resolution

**Status**: In Progress
**Last updated**: 2026-09-12
**Feature**: T14.F05
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Stored closure reports distinguish plasticity assignments that change a weight
from assignments that leave it unchanged, split Hebbian and reward-modulated
updates, count production shared-memory writes that change a slot, and show the
structural carrier census beside memory sensitivity. These readings describe
applied events and exposure; they establish neither useful learning nor memory
capability.

## Non-Goals

- Change learning rules, costs, defaults, weight clamping, memory semantics,
  controller structure, or world recipes.
- Add interventions, new sensitivity scores, carrier-conditioned sensitivity
  fractions, checkpoint cognition series, or a cognition-payoff claim.
- Redesign the UI, add telemetry services, or rewrite historical reports.

## Inputs and Invariants

The owning roadmap row and F05 note define scope. [T14.F02](t14-f02-existing-counter-transfer.md)
supplies terminal `WorldTracking` transfer, absent-as-unmeasured report fields,
and the sequential reduction of dispatch-local integers. Preserve the
[architecture](../../strategy/architecture.md): runtime observes its writes,
the tick loop accumulates applied production activity, and the CLI projects
those totals into reports.

| Source | Binding behavior |
| --- | --- |
| `runtime/plasticity/hebbian.rs::apply_hebbian_updates` | Counts each edge assignment after graph evaluation, including zero and clamped-away changes; skips modulated nodes. |
| `runtime/plasticity/reward.rs::apply_reward_modulated_updates`, `simulation/tick.rs::run_reward_learning` | Assigns modulated edge weights in Phase 2.5; adds its count to the same `SimStats::plasticity_updates_total`. |
| `runtime/cgp/execute.rs`, `runtime/types.rs`, `simulation/tick.rs::TickComputeStats` | Carries Hebbian work through mesh output and commits integers in the existing queue-order reduction. |
| `runtime/vm.rs`, `runtime/cgp/effects.rs`, `runtime/traced_vm.rs::record_slot_write` | VM store/clear instructions and wired Graph write/clear sinks apply memory writes; the existing traced VM predicate is `(old_value - new_value).abs() > f32::EPSILON`. |
| `neighborhood/companions.rs::structural_companions` | Returns four overlapping structural flags over mesh-reachable nodes: shared-memory reader, shared-memory writer, stateful compute, and plasticity. This is a structural scan, not executed-path or capability analysis. |
| `v3-cli/src/bench.rs` | `WorldTracking::with_transferred_counters` reads terminal counters; `observe_final_actions` supplies the final population for `memory_sensitivity`; the shipped fractions divide by all final creatures. |

Paths beginning `runtime/`, `simulation/`, or `neighborhood/` above are relative
to `crates/v3-core/src/`; the CLI path is relative to `crates/`.

**Research decision (2026-09-12).** Extend the existing integer counters and
Serde report blocks. Reconstructing counts from traces is the credible local
alternative, but production VM execution uses a no-op trace sink and reward
learning happens after mesh execution; collecting full traces would add data
and allocation solely to recover these small totals. Direct event counters fit
the [Prometheus instrumentation guidance](https://prometheus.io/docs/practices/instrumentation/)
on counting outcomes alongside totals. Existing
[Serde field attributes](https://serde.rs/field-attrs.html) support optional
additions without fabricating historical zeros. No dependency is needed. The
remaining limitation is intentional: the inherited structural predicate is a
coarse exposure description, not proof that an intervention can affect action.

**Learning counts.** Keep `plasticity_updates_total` and its benchmark work
counter definition unchanged. Add the count of assignments whose final clamped
`f32` weight is unequal (`!=`) to its immediately prior weight at both update
sites; a nonzero proposed delta that clamps or rounds away is not a change.
Expose total assignments and changed assignments separately for Hebbian and
reward-modulated learning, with their combined changed total. Each pathway's
changed count is at most its assignment count; the two pathways sum to the
combined assignment and changed counts. Count actual edge assignments, not
configured nodes, traces, or energy charges. Preserve formula evaluation order,
costs, and the timing of weight writes.

**Memory events.** Count each executed VM store/clear and applied wired Graph
write/clear whose sanitized written value satisfies the existing changed-write
predicate against that slot's immediately prior value. Reuse the predicate
across production and traced paths; do not change its epsilon boundary. A
write later reversed by another write still counts. Unexecuted instructions,
invalid or unwired Graph sinks, same-value writes, memory decay, snapshots,
initialization, and inheritance do not count. Dispatch-local integer counts
reach `SimStats` through the existing sequential reduction; observation and
tracing on copies never increment the running simulation's totals.

**Report contract.** Add one terminal `WorldTracking.cognition` block carrying
the learning totals/splits and the cumulative changed-memory-write count.
Checkpoint samples omit it. Add a per-seed structural-companions census beside
`goal_indicators.memory_sensitivity`, populated from the same final living
population as the action observation. Include the final creature count and a
count for each of the four existing flags; a creature may appear in several
counts. Reuse `structural_companions` without changing its predicate or adding
a second reachability analysis. Record zero for an observed empty population.
The block is absent when memory sensitivity is unmeasured. Additions use
`Option` with Serde default/skip-none semantics, so historical absence remains
unmeasured. Preserve existing sensitivity values, fractions, version tokens,
and comparison inputs, including the temporal-memory block. New counts are
diagnostics accompanying the existing goal indicator, not another indicator
or a new no-regression floor. Report ordering and seeded trajectories remain
deterministic; no global counters, RNG calls, or unordered serialized maps.

## Implementation Tasks

- [ ] Add applied-change observations at both learning sites and all production
  shared-memory write sites, and carry the counts into cumulative `SimStats`.
- [ ] Transfer the terminal cognition block and final-population carrier census
  into the existing benchmark report with truthful absence semantics.
- [ ] Complete focused verification and closure evidence; update affected
  reference documentation, progress artifacts, and roadmap/spec state.

## Verification

- [ ] Learning assignments versus changes, pathway partitioning, unchanged
  clamped/rounded results and costs: `cargo test -p v3-core runtime::plasticity`
  and runtime integration tests; exact results in the
  [readings](../../progress/readings/t14-f05-cognition-telemetry-resolution.md).
- [ ] VM/Graph memory event coverage, epsilon boundary, repeated/reversed writes,
  and traced/plain parity: `cargo test -p v3-core runtime::`; counter invariants
  have property tests. Production accumulation and observational non-mutation:
  `cargo test -p v3-core simulation::tick::tests`; results in the readings.
- [ ] Terminal transfer, checkpoint omission, four overlapping carrier counts,
  empty population, historical absence, and unchanged sensitivity/comparison
  semantics: `cargo test -p v3-cli bench::tests`; results in the readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary, output path,
  and every survivor here as killed, equivalent, or explicitly deferred under
  the workflow; raw output in the readings. No run recorded yet.
- [ ] Gate report: `make bench PROFILE=gate FEATURE=t14-f05-cognition-telemetry-resolution`
  → `docs/progress/features/t14-f05-cognition-telemetry-resolution.json`.
- [ ] One goal report:
  `make bench PROFILE=goal FEATURE=t14-f05-cognition-telemetry-resolution`
  → `docs/progress/features/t14-f05-cognition-telemetry-resolution-goal.json`.
  Record the new observations, including zeros, for Orchards, Canyon, and
  Confluence in the readings.
- [ ] Required final `make check`, including existing reproducibility checks,
  passes; command result in the readings. A second goal run is not applicable
  under the workflow's one-goal-run contract.

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation-only feature; no
mechanism or environmental pressure is added. Extend instrumentation at
existing assignments with bounded comparisons and integer additions, without
per-event allocations; one final structural census adds a genome scan outside
the tick loop. Expected compute cost is small and unjustified severe regression
blocks closure; no epoch re-pin is expected or authorized.

References are the epoch and last eligible closure selected by
`docs/progress/benchmark-series.json`: gate epoch
`remove-complementary-nutrition.json`, world-goal epoch
`t12-f04-baseline-world-set-goal.json`, and currently the corresponding
T14.F03 reports as last closures. Keep normalized work thresholds at strictly
greater than +10% flagged / +50% severe; wall-clock signals at greater than
+25% / +100%. Preserve goal floors and no-regression coverage. Founder
neighborhood observation stays within 10 seconds, evolved observation within
180 seconds summed across seeds, and the existing 15-minute total goal-profile
investigation threshold remains in force.

Expected direction for lineage diversity, memory sensitivity, temporal memory
sensitivity, mutational neighborhood, drift depth, persistence, births per
100 ticks, and every existing deterministic work counter: **none**. At identical
inputs their values and trajectories stay unchanged; movement is a defect to
resolve. The new carrier census is wired beside the existing memory indicator
in the goal profile and is not a sensitivity or capability score. Newly visible
counts have no directional target; zero is a valid reading.

**Measured verdict.** Not measured.

- Reports: [gate](../../progress/features/t14-f05-cognition-telemetry-resolution.json),
  [goal](../../progress/features/t14-f05-cognition-telemetry-resolution-goal.json).
- Full readings: [t14-f05](../../progress/readings/t14-f05-cognition-telemetry-resolution.md).

## Success Criteria

- [ ] Closure reports distinguish unchanged and changed plasticity assignments
  and Hebbian versus reward-modulated learning without changing learning.
- [ ] Production changed-memory-write totals cover both backends and reflect
  individual applied events, including writes subsequently reversed.
- [ ] The memory reading carries the final-population structural census while
  its shipped fraction and historical readings retain their meaning.
- [ ] Required verification and both benchmark profiles pass, evidence is
  stored, and feature/spec completion state agrees.

## Notes for AI Agents

- Decision: The carrier census reuses `structural_companions` and is exposure,
  not capability; it does not redefine `memory-sensitivity-v1`.
- Cost: One completed user intervention: direct authorization for local T14.F05
  planning and implementation commits. Closure cost totals remain unmeasured.
