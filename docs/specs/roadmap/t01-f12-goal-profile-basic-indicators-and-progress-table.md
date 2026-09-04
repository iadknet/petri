# T01.F12 — Goal Profile, Basic Indicators, and Progress Table

**Status**: In Progress
**Last updated**: 2026-09-04
**Feature**: T01.F12
**Track**: [T01 — Experimental Science and Causal Evaluation](../../roadmaps/t01-experimental-science-and-causal-evaluation.md)

## Goal

A fixed, minutes-scale goal benchmark reports final-population lineage diversity
and memory sensitivity without changing the simulation. A plain progress table
shows both compute comparisons and every indicator, backed by separate gate and
goal series. This run delivers committed implementation and automated review for
manual verification; integration remains pending.

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

- [ ] Add fixed `goal` CLI profile and `make bench PROFILE=goal FEATURE=...`.
      Reject conflicting profile-parameter overrides rather than ignore them;
      retain explicit thread and output selection. Keep goal outside `make check`.
- [ ] Implement the isolated final-state probe and stable lineage/memory report
      aggregation with the definitions above, reusing production sensor/runtime
      functions. Keep the simulation trajectory and counters unchanged.
- [ ] Extend report loading and the series index to separate gate and goal
      references; tests prove reference selection and profile mismatch rejection.
- [ ] Create `docs/progress.md` as one table: four closed historical feature rows
      back-filled from stored reports and one clearly labeled pending T01.F12
      row. Include date, report links, compute delta against both references,
      and every indicator. Historical goal readings are `Undefined`, since those
      profiles never ran; their gate compute deltas remain historical evidence.
- [ ] Commit implementation, generate gate and goal reports with that provenance,
      then record results, series baseline, and pending manual status. Keep the
      owning feature checkbox unchecked and this spec In Progress.

## Verification

- [ ] TDD evidence for profile resolution, lineage examples (empty, one, balanced,
      unequal), constructed memory-insensitive and memory-sensitive controllers,
      full-action differences, zero/scramble union, and no live-state/RNG changes.
      Pure aggregation/permutation invariants receive proptests; preserve any
      generated regression file. Use tiny goal fixtures for report determinism
      across repeated runs and thread counts, plus old gate-field equality.
- [ ] Run viability first if tick mechanics/defaults/founders are touched; run
      `cargo check --workspace --all-targets` after coherent Rust edits, focused
      tests, and `make roadmap-check` after document edits.
- [ ] Explicit simplification self-review for reuse, enums, existing dependencies,
      redundant computation, and scope; record changes and affected rechecks.
- [ ] `make rust-mutants` after simplification; record its summary, output path,
      complete missed/timeout survivor list and each killed/equivalent/deferred
      resolution. Only strengthen tests to kill mutants, then rerun the target.
- [ ] Store `docs/progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table.json`
      (gate) and `docs/progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table-goal.json`
      (goal) through `make bench`; record both compute references and report
      provenance. No threshold or stored baseline edits to obtain a pass.
- [ ] Run goal a second time in a separate process to scratch output, sequentially
      without concurrent builds/benchmarks, and require full deterministic-block
      equality. Record both elapsed times, births and deaths on every seed, and
      the actual lineage and memory readings, including null/zero results.
- [ ] Sol consultations received before approach and before reporting done (also
      after a repeated failure), with accepted/rejected guidance and count.
- [ ] Fresh Astra independent final review and required remediation completed;
      record finding counts and advisory deferrals.
- [ ] Orchestrator independently runs `make roadmap-check`, commits all final
      content, and runs `make check` against that exact clean commit. Record the
      tested hash in the task handoff (a commit cannot embed its own hash).
- [ ] User manual verification and integration (intentionally pending).

## Performance and Goal Impact

Predeclared compute cost: no simulation work-counter change, exactly 0% for
nonzero counters against the last closed gate report (T10.F11) and pinned gate
epoch (T10.F10); both-zero deltas remain null/ok. No measurable gate cost is
expected (under 1% excluding wall-clock noise). Gate probe work is absent.
Goal adds one final population aggregation and three cloned mesh evaluations per
survivor, outside simulation counters. Include probe time in a separately named
environment timing or the report's measured total so the observation cost is
visible; preserve the existing gate timing boundary. No gate epoch re-pin.

Record measured gate deltas and levels, threshold crossings, dated per-seed goal
readings, observation overhead, both total run times, and second-run equality
here after the runs. The first goal-v1 reading is a provisional program baseline
pending manual verification/integration, not evidence of improvement.

## Success Criteria

- [ ] The fixed goal benchmark provides truthful, reproducible lineage and memory
      sensitivity readings, while the gate trajectory stays unchanged.
- [ ] One progress table exposes every indicator, links its evidence, and clearly
      distinguishes historical undefined goals from the new pending baseline.
- [ ] Automated gates, mutation triage, benchmark reporting, and independent
      review pass on committed content in a clean feature worktree.
- [ ] Manual verification and integration are approved (intentionally pending).

## Notes for AI Agents

- Start verified: clean main at `82f0efc072f2073e510727cb15ce6dc92b54e408`;
  target unclaimed and all owning-row dependencies checked. Worktree:
  `/Users/istefanek/projects/petri/.worktrees/t01-f12`, branch `codex/t01-f12`.
  Session metadata confirms orchestrator `gpt-6-astra`, effort `high`.
- Planning readiness review: no P1/P2/P3 findings; Ready. Template, current track
  requirements, dependency outputs, empty-population handling, measurement
  isolation, gate preservation, and manual-verification override were checked.
  Implementation and measured performance remain to be verified.
- User override: implementation and automated review must finish here, but keep
  feature unchecked and spec In Progress. Manual verification and integration
  are pending. Preserve worktree and branch and stop for the user's decision.
- Usage unavailable. Advisor consultation and reviewer counts pending.
