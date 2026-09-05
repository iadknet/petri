# T01.F12 — Goal Profile, Basic Indicators, and Progress Table

**Status**: In Progress
**Last updated**: 2026-09-04
**Feature**: T01.F12
**Track**: [T01 — Experimental Science and Causal Evaluation](../../roadmaps/t01-experimental-science-and-causal-evaluation.md)

## Goal

A fixed, minutes-scale goal benchmark reports final-population lineage diversity
and memory sensitivity without changing the simulation. A plain progress table
shows both compute comparisons and every indicator, backed by separate gate and
goal series. Implementation and automated review are complete; manual
verification and integration remain pending.

## Non-Goals

- No simulation behavior, production default, founder, or economics changes.
- No gate-profile parameter change, gate epoch re-pin, or edits to old reports
  and closed dependency specs. The current T01 track supersedes their old
  forward references about changing the gate.
- No causal memory-dependence claim, strategy/ecotype count, uncertainty protocol,
  dashboard, campaign runner, new dependency, or adjacent roadmap feature.
- No integration, remote mutation, worktree removal, or branch deletion.

## Inputs and Invariants

- Sources: the owning track's T01.F12 note; `docs/workflow.md` with the user's
  manual-verification override; [T01.F11](t01-f11-baseline-persistence-characterization.md),
  [T10.F09](t10-f09-throughput-baseline-and-profiling-budget.md), and
  [T10.F11](t10-f11-cross-process-reproducibility-of-seeded-runs.md).
- Predeclared goal-v1 parameters: world 1600 by 1600, 10,000 founders,
  seeds `[11, 22, 33]`, 2,000 ticks, production food coverage (`None`),
  production defaults otherwise. Use existing thread control; record the
  actual pool size. T10.F11's regenerated `w1600.json` persisted on all seeds,
  with births and deaths, and took 588/589 seconds for its two runs. Budget:
  about ten minutes per release run, 15 minutes before investigating a budget
  overrun. If reduction is necessary, record the measured reason and revised
  fixed parameters before re-running; never choose parameters for better readings.
- Existing implementation: `crates/v3-cli/src/bench.rs` already owns sweep
  execution, report types, comparisons, persistence, and structure size;
  `main.rs` owns CLI resolution; `Makefile` exposes `make bench`. Extend those.
  Reuse the core's actual sensor assembly and mesh/action-selection path for a
  small read-only final-state probe; do not independently recreate perception
  or tick mechanics in the CLI. Pure aggregation belongs beside the report.
- Research (2026-09-04): extending the existing harness versus a separate assay
  runner were considered; the existing harness already supplies seeds, threads,
  comparison references, and fixed numeric formatting, and the roadmap mandates
  it. For scrambling, a fixed permutation and the existing `rand` seeded shuffle
  are both sufficient; choose and record one algorithm, without a new library.
  The [rand SliceRandom API](https://docs.rs/rand/0.8.5/rand/seq/trait.SliceRandom.html)
  accepts an explicit RNG. Fixed permutation is the smallest option unless the
  existing probe code makes seeded shuffle simpler. Either must preserve the
  memory multiset and leave the live RNG untouched.
- Probe each final living creature on one sensor snapshot of the final world,
  held identical across intact, zeroed, and scrambled memory variants. Start
  each evaluation from a fresh clone of the same creature including learned
  runtime, with matching copies of any RNG consumed in cognition/action choice.
  No world action is applied; live memory, plasticity, creature state, counters,
  and RNG state remain unchanged. Compare the complete chosen `WorldAction`
  (including payload/direction), not trace, priority, or attempted action success.
- Gate and ordinary sweep output keep the new indicators `Undefined`; only goal
  runs perform the probes. A small configurable test fixture may exercise the
  same goal observation path without running the full fixed profile in tests.
  New keys may extend the report, but gate parameters and all prior deterministic
  fields must match the existing gate references. Stored reports remain readable.
- Preserve the gate series label, epoch reference, and four closed references.
  Add a distinct goal-v1 series with the first goal report as baseline. The
  current feature is pending manual verification: do not append it to a `closed`
  list. Its goal baseline is explicitly provisional until integration.

### Indicator definitions (goal-v1)

- `lineage_diversity`: per seed, surviving founder-clade count and Shannon
  entropy `-sum(p * ln(p))` of final abundance by `lineage_id`, in nats.
  Aggregate in stable lineage order; report six decimal places. Empty population:
  count 0, entropy `Undefined`; one lineage: entropy 0. This detects collapse
  and trends; it measures neither ecotypes nor distinct strategies.
- `memory_sensitivity`: per seed, final creature count, number whose intact
  chosen action differs from zeroed, number differing from scrambled, and number
  differing from either; divide each by final creature count and format to six
  decimals. The primary fraction is the union (either), counting each creature
  once. Empty population has zero counts and `Undefined` fractions. Record the
  scramble algorithm and snapshot timing. This is local action sensitivity to
  shared memory, not adaptive benefit, causal memory dependence, learning,
  uncertainty, or general cognition. Founders should read near zero; no value is
  a pass/fail target and a rise alone proves no program-level success.
- Existing `population_persistence`, `births_per_100_ticks`, and
  `reachable_structure_size_distribution` retain their definitions and units.
  Persistence is survival over a bounded horizon, births measure applied
  reproduction, and reachable size is structural complexity rather than ability.
- `strategy_count`, `strategy_causal_distinctness`, `evolutionary_activity`,
  `adaptive_novelty`, `memory_dependence`, `learning_dependence`,
  `prediction_dependence`, `information_integration`, and
  `reciprocal_interaction` remain `Undefined`, owned by deferred features.
  Never imply gate readings are goal readings. Definition changes start a named
  new series with an explanation in `docs/progress.md`.

## Implementation Tasks

- [x] Add fixed `goal` CLI profile and `make bench PROFILE=goal FEATURE=...`.
      Reject conflicting profile-parameter overrides rather than ignore them;
      retain explicit thread and output selection. Keep goal outside `make check`.
- [x] Implement the isolated final-state probe and stable lineage/memory report
      aggregation with the definitions above, reusing production sensor/runtime
      functions. Keep the simulation trajectory and counters unchanged.
- [x] Extend report loading and the series index to separate gate and goal
      references; tests prove reference selection and profile mismatch rejection.
- [x] Create `docs/progress.md` as one table: four closed historical feature rows
      back-filled from stored reports and one clearly labeled pending T01.F12
      row. Include date, report links, compute delta against both references,
      and every indicator. Historical goal readings are `Undefined`, since those
      profiles never ran; their gate compute deltas remain historical evidence.
- [x] Commit implementation, generate gate and goal reports with that provenance,
      then record results, series baseline, and pending manual status. Keep the
      owning feature checkbox unchecked and this spec In Progress.

## Verification

- [x] TDD evidence for profile resolution, lineage examples (empty, one, balanced,
      unequal), constructed memory-insensitive and memory-sensitive controllers,
      full-action differences, zero/scramble union, and no live-state/RNG changes.
      Pure aggregation/permutation invariants receive proptests; preserve any
      generated regression file. Use tiny goal fixtures for report determinism
      across repeated runs and thread counts, plus old gate-field equality.
- [x] Run viability first if tick mechanics/defaults/founders are touched; run
      `cargo check --workspace --all-targets` after coherent Rust edits, focused
      tests, and `make roadmap-check` after document edits.
- [x] Explicit simplification self-review for reuse, enums, existing dependencies,
      redundant computation, and scope; record changes and affected rechecks.
- [x] `make rust-mutants` after simplification; record its summary, output path,
      complete missed/timeout survivor list and each killed/equivalent/deferred
      resolution. Only strengthen tests to kill mutants, then rerun the target.
- [x] Store `docs/progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table.json`
      (gate) and `docs/progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table-goal.json`
      (goal) through `make bench`; record both compute references and report
      provenance. No threshold or stored baseline edits to obtain a pass.
- [x] Run goal a second time in a separate process to scratch output, sequentially
      without concurrent builds/benchmarks, and require full deterministic-block
      equality. Record both elapsed times, births and deaths on every seed, and
      the actual lineage and memory readings, including null/zero results.
- [x] Sol consultations received before approach and before reporting done (also
      after a repeated failure), with accepted/rejected guidance and count.
- [x] Fresh Astra independent final review and required remediation completed;
      record finding counts and advisory deferrals.
- [x] Orchestrator independently ran `make roadmap-check` and `make check` on
      clean `888fadfcc3f29849cfb0560121e9ea1571e3f628`. Root will repeat the
      full check after this status-only commit and record that tested hash in the
      task handoff (a commit cannot embed its own hash).
- [ ] User manual verification and integration (intentionally pending).

TDD record: `cargo test -p v3-core final_action_observation_uses_full_actions_and_leaves_simulation_unchanged`
was red before `GraphRuntimeState::Clone` and `observe_final_actions` existed;
it was green after the observer implementation. The goal resolver test was
likewise red before the `Goal` profile and `goal_profile_params`, then green.
The final focused checks were `cargo test -p v3-core final_action_observation`
(3 passed), `cargo test -p v3-cli --test bench sweep_output_and_reference_selection_stay_separate_from_goal`
(1 passed), and `cargo test -p v3-cli --lib memory_sensitivity_union_and_fractions_match_generated_differences`
(1 passed). Final suites were `cargo test -p v3-core` (1019 passed) and
`cargo test -p v3-cli` (18 library, 9 binary, 17 benchmark integration, and 7
CLI integration tests passed). `cargo check --workspace --all-targets` passed
again after the Clippy correction. `make roadmap-check` passed, and the
orchestrator's `make check` passed with exit 0 on
`888fadfcc3f29849cfb0560121e9ea1571e3f628`
([log](/private/tmp/t01-f12-check-888fadfc.log)). Tick mechanics, defaults,
and founder behavior were untouched, so viability-first was not applicable.

Mutation record: the initial `make rust-mutants` run used
`/Users/istefanek/.local/share/petri-tools/mutants/t01-f12/mutants.out` and
reported 54 total: 33 caught, 18 unviable, 3 missed, and 0 timeout. Its full
missed list was `crates/v3-cli/src/main.rs:247:32: replace == with != in run_bench`,
`crates/v3-cli/src/main.rs:268:42: replace && with || in run_bench`, and
`crates/v3-cli/src/main.rs:268:58: replace == with != in run_bench`. Each was
killed by the test-only `sweep_output_and_reference_selection_stay_separate_from_goal`
coverage; no production code changed. The rerun ended 2026-09-05T00:55:13Z
with 54 total, 36 caught, 18 unviable, 0 missed, and 0 timeout. The final
`missed.txt` and `timeout.txt` are empty; no survivor was equivalent or deferred.
After the behavior-equivalent Clippy correction to the empty-population fraction,
the final rerun ended 2026-09-05T01:22:06Z with 52 total, 34 caught, 18
unviable, 0 missed, and 0 timeout. Its complete missed and timeout lists are
empty; no survivor was equivalent or deferred.

Fresh Astra review initially found 0 P1, 2 P2, and 2 P3 items. This remediation
preserves each historical report's stored comparison references, labels historical
gate measurements separately from goal-v1 indicators, repairs local links,
corrects the Make help, and formats Rust. The follow-up review verified all four
items resolved and found 0 P1, 0 P2, and 0 P3 remaining.

## Performance and Goal Impact

Predeclared compute cost: no simulation work-counter change, exactly 0% for
nonzero counters against the last closed gate report (T10.F11) and pinned gate
epoch (T10.F10); both-zero deltas remain null/ok. No measurable gate cost is
expected (under 1% excluding wall-clock noise). Gate probe work is absent.
Goal adds one final population aggregation and three cloned mesh evaluations per
survivor, outside simulation counters. Include probe time in a separately named
environment timing or the report's measured total so the observation cost is
visible; preserve the existing gate timing boundary. No gate epoch re-pin.

The first goal-v1 reading below is a provisional program baseline pending manual
verification/integration, not evidence of improvement.

Measured on 2026-09-04/05 with committed producer revision
`e94569b8ebed8b28dd7bce1db5d35e134ecdc3f4` on Apple M1 Pro, 8 logical cores:

- Gate report: [evidence](../../progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table.json).
  The five nonzero normalized work counters were exactly 0.000000% against
  T10.F10 and T10.F11; plasticity was zero on both sides and therefore
  `null`/ok. Both comparisons were non-severe. Gate wall time was
  0.004590127 ms per creature tick: -15.742295% versus T10.F10 and +0.295879%
  versus T10.F11, both ok. The gate report's new lineage and memory indicators
  are `Undefined`; the remaining deterministic object equals T10.F11 after
  removing only those two new keys.
- Goal report: [stored evidence](../../progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table-goal.json);
  sequential repeat: `/private/tmp/t01-f12-goal-final-second.json`. Fixed
  inputs were 1600 by 1600, 10,000 founders, seeds 11/22/33, 2,000 ticks, and
  production food coverage. The stored/repeat wall totals were 583820.333 ms
  and 583464.009 ms; final observation overhead was 1302.799 ms and 1298.477
  ms, respectively. Parsed deterministic objects were exactly equal using
  Node `util.isDeepStrictEqual` (not a serialization hash). Both readings had
  births/final population/deaths of 134117/5291/138826 (seed 11),
  179125/10997/178128 (seed 22), and 166410/8130/168280 (seed 33), with no
  extinction. The earlier, sleep-interrupted bd2ad9b4 repeat is retained only
  as superseded evidence; its maintenance-sleep intervals are not attributed
  to probe cost.
- Goal lineage count/entropy was 142/2.780730, 144/2.947968, and 140/2.668014.
  Memory sensitivity was zeroed/scrambled/either = 0/0/0 and fraction
  0.000000 for every seed. The probe uses the complete selected action queue
  (`Vec<WorldAction>`, including order, length, variant, direction, and payload),
  after the final executed tick and before observation actions; it compares
  intact memory with zeroed memory and fixed `rotate_left(1)` scrambling of the
  16 shared-memory slots. It does not assess applied-action success, priority,
  traces, or adaptive benefit.

Simplification review found no new abstraction, dependency, enum, or framework
need. It reduced the six duplicated goal-override branches to one local flag
loop while preserving each exact error, and reduced memory-sensitivity counting
from three traversals to one fold. The focused CLI/core tests and workspace
check passed before the regenerated reports. The observer remains isolated from
`run_tick`, production defaults, and founder behavior, so viability-first did
not apply.

Advisor record: 2 Sol consultations, both accepted and none rejected. Before
approach, Sol recommended the narrow `&Simulation` observer, sorted IDs,
one sensor assembly, fresh local mutable state, `GraphRuntimeState::clone`,
fixed rotation, and complete queue comparison. Before reporting, Sol confirmed
that complete ordered `Vec<WorldAction>` equality and union-once counting are
the minimal correct semantics; no correction was needed.

## Success Criteria

- [x] The fixed goal benchmark provides truthful, reproducible lineage and memory
      sensitivity readings, while the gate trajectory stays unchanged.
- [x] One progress table exposes every indicator, links its evidence, and clearly
      distinguishes historical undefined goals from the new pending baseline.
- [x] Automated gates, mutation triage, benchmark reporting, and independent
      review passed on clean `888fadfcc3f29849cfb0560121e9ea1571e3f628`;
      root will repeat `make check` after this status-only commit.
- [ ] Manual verification and integration are approved (intentionally pending).

## Notes for AI Agents

- Start verified: clean main at `82f0efc072f2073e510727cb15ce6dc92b54e408`;
  target unclaimed and all owning-row dependencies checked. Worktree:
  `/Users/istefanek/projects/petri/.worktrees/t01-f12`, branch `codex/t01-f12`.
  Session metadata confirms orchestrator `gpt-6-astra`, effort `high`.
- Main subsequently advanced externally to
  `941c498e92928ec0386be4542ad0ea687fb580e3`. This feature remains isolated on
  its recorded merge base; later integration requires a separate reconciliation
  decision.
- Planning readiness review: no P1/P2/P3 findings; Ready. Template, current track
  requirements, dependency outputs, empty-population handling, measurement
  isolation, gate preservation, and manual-verification override were checked.
  Implementation and measured performance are recorded. The orchestrator's
  `make check` passed on `888fadfcc3f29849cfb0560121e9ea1571e3f628`; root will
  repeat it after this status-only commit.
- User override: implementation and automated review must finish here, but keep
  feature unchecked and spec In Progress. Manual verification and integration
  are pending. Preserve worktree and branch and stop for the user's decision.
- Manual verification and integration steps (intentionally pending): inspect
  [docs/progress.md](../../progress.md) to confirm the T01.F12 row remains
  pending and all historical lineage/memory values are `Undefined`. To repeat
  the fixed benchmark without replacing committed evidence, run
  `caffeinate -i make bench PROFILE=goal FEATURE=t01-f12-goal-profile-basic-indicators-and-progress-table OUT=/private/tmp/t01-f12-manual-goal.json`
  from this worktree (about ten minutes), then compare the full parsed
  `deterministic` object with the committed goal report. Expect final
  populations 5291/10997/8130, lineage counts 142/144/140, entropies
  2.780730/2.947968/2.668014, and zero memory sensitivity for seeds 11/22/33.
- Usage unavailable. Two advisor consultations completed; the fresh Astra review
  found no remaining findings.
