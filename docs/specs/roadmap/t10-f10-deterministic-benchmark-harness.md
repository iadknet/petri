# T10.F10 — Deterministic Benchmark Harness and Per-Feature Report

**Status**: Complete
**Last updated**: 2026-09-03
**Feature**: T10.F10
**Track**: [T10 — Evolutionary Scale and Experiment Infrastructure](../../roadmaps/t10-evolutionary-scale-and-experiment-infrastructure.md)

## Goal

One checked-in command produces a compact JSON benchmark report whose
deterministic block is byte-identical on re-run at the same commit, compares it
against two stored references, and fails a fast test on a severe unjustified
compute regression. Every later feature runs it at closure.

## Non-Goals

- No peak-memory measurement, no parallelism changes, no campaign or
  checkpoint infrastructure (T10.F01 onward).
- No wall-clock assertion anywhere; wall-clock is recorded only.
- No dashboard, database, service, or composite score.
- No new goal indicators beyond the three the T10 roadmap already names as
  populated. Every other indicator is present and `"Undefined"`.
- No change to simulation behavior, production defaults, founders, or tick-loop
  mechanics. Counters observe applied execution; they do not alter it.

## Inputs and Invariants

- Contract: T10 roadmap "Notes for AI Agents" paragraphs on T10.F10, the
  master roadmap's benchmark paragraph, and the Performance and Goal Impact
  section of `docs/specs/roadmap/_feature-template.md`.
- Existing surfaces: `crates/v3-cli/src/main.rs` (clap `Run` subcommand only),
  `crates/v3-cli/src/lib.rs` (`run_simulation`, NDJSON events),
  `crates/v3-core/src/simulation/stats.rs` (`SimStats`: reproduction, mutation,
  predation counters; per-tick `last_tick_move|eat|noop|reproduce|steal`),
  `crates/v3-core/src/runtime/types.rs` (`MeshOutput`, `ComputeCostReport` with
  two `f32` costs and no counts), `crates/v3-core/src/runtime/mesh.rs`
  (`hops` loop in `execute_creature_mesh_impl`, shared by the traced mode),
  `crates/v3-core/src/runtime/vm.rs` (`steps` loop),
  `crates/v3-core/src/runtime/cgp/execute.rs` (`for pass in 0..max_passes`
  relaxation loop; Hebbian and reward-modulated plasticity entry points),
  `crates/v3-core/src/creature/genome/analysis.rs` (`functional_complexity`),
  `crates/v3-core/src/simulation/tick.rs` (mesh phase runs under rayon
  `par_iter_mut`, so per-creature counts must be integers summed after the
  parallel phase; the action phase is sequential).
- Invariant: runtime-facing telemetry derives from applied simulation behavior
  (`AGENTS.md`). A counter increments exactly where the counted work executes.
- Invariant: the mesh phase is parallel, so the wall-clock half varies with
  core count and load. Only integer counters may fail a test.
- Determinism contract: same commit, same inputs, byte-identical
  `deterministic` block. Keys sorted (use `BTreeMap`), floats written with a
  fixed six-decimal format, no timestamp, hostname, duration, or path inside
  that block.
- Gate profile constants (predeclared here; revisable by this feature only
  with a recorded timing reason, and by T01.F12 later): production defaults
  except world 128 by 128, 256 founders, initial food coverage 1.0, seeds
  `[11, 22, 33]`, horizon **75 ticks** (revised down from the originally
  predeclared 300). Sized so the two fast tests below finish in under 30
  seconds combined in a debug build on the recording host; if they do not,
  reduce the horizon first, then the founder count, and record the final
  values in this spec.
  - Timing reason for the revision: a `cargo run` (debug) timing probe on the
    recording host (Apple M1 Pro, 8 logical cores) measured ~8.4s per
    single-threaded 300-tick seed run at 128x128/256 founders/coverage 1.0.
    Nine seed-runs are needed for `make check` (two full gate runs in test 1,
    one comparison run in test 2, three seeds each) — 300 ticks would cost
    ~76s, over budget. At 75 ticks the same host measured ~2.6-2.7s per seed
    run (~24s for all nine), leaving headroom. Population does not go extinct
    within 75 ticks for these seeds (extinction is acceptable but not
    required; see Notes for AI Agents). At 75 ticks no creature dies in any
    of the three seeds (minimum population equals the founder count, 256, in
    every seed) — the horizon is short enough that `population_persistence`
    currently only proves a population floor was held, not decline or
    recovery dynamics; T01.F12, which is expected to lengthen or otherwise
    revise the gate profile, must revisit whether this indicator is
    interpretable at its horizon.
- Regression thresholds, applied per normalized work counter against each
  reference: flag above +10 percent, severe above +50 percent. Wall-clock per
  creature-tick: flag above +25 percent, severe above a doubling, reported only
  when the reference's host identity matches, never asserted.

## Implementation Tasks

- [x] Add integer work counters. `MeshOutput` gains a `WorkCounters` value
      with `mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`.
      As built: `plasticity_updates` on `MeshOutput.work_counters` carries
      only the Hebbian contribution (incremented in `cgp/execute.rs` inside
      Phase 1, where a `MeshOutput` still exists to attach it to); the
      reward-modulated contribution is added directly to
      `SimStats.plasticity_updates_total` at its Phase 2.5 call site in
      `tick.rs`, because reward-modulated updates apply after `decisions:
      Vec<(CreatureId, MeshOutput)>` has already been consumed and no
      `MeshOutput` remains to carry a count (see Notes for AI Agents for the
      full reasoning). The cumulative total is the sum of both paths, matching
      "Hebbian weight updates plus reward-modulated updates". `mesh_hops`,
      `vm_steps`, and `graph_relax_iters` are each incremented where that work
      executes in `mesh.rs`, `vm.rs`, and `cgp/execute.rs` respectively, in
      both the plain and traced modes. `SimStats` gains cumulative `u64`
      totals for those four, plus `creature_ticks_total`
      (creatures that ran the mesh, summed per tick) and
      `actions_applied_total` (every action the action phase executed: move,
      eat, noop, reproduce, steal), accumulated sequentially after the parallel
      phase. Births are the existing `reproduction_actions_spawned_total`.
      Unit tests assert each counter from a run whose expected count is known.
- [x] Add `crates/v3-cli/src/bench.rs` with `v3-cli bench` in `main.rs`:
      `--profile gate|sweep`, `--out <path>`, `--feature <id>` (required for
      gate), `--compare <report.json>` and `--baseline <report.json>` (each
      optional, repeatable), and for `sweep` only: `--width`, `--height`,
      `--founders`, `--seeds <comma list>`, `--ticks`, `--food-coverage`
      (default 1.0). Runs each seed with `seed_simulation` and `run_tick`,
      records per-seed and aggregate results, writes the report, prints the
      comparison, and exits 3 on a severe work regression.
- [x] Report schema, one file per feature at
      `docs/progress/features/<tNN-fNN-slug>.json`, top-level keys
      `schema_version` (1), `feature`, `deterministic`, `environment`,
      `comparison`:
  - `deterministic`: `profile` (name and every input parameter),
    `per_seed[]` (seed, `ticks`, `creature_ticks`, `mesh_hops`, `vm_steps`,
    `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`,
    `final_population`, `extinction_tick` or `null`), `totals` (same counters
    summed), `per_creature_tick` (each work counter divided by
    `creature_ticks`, six decimals), and `goal_indicators`. `graph_relax_iters`
    is incremented at the top of each relaxation pass, before that pass's
    energy check, so a pass that gets entered and aborts partway through
    (energy exhausted after its cost was deducted) is still counted — the
    counter measures passes attempted, not passes that completed.
  - `goal_indicators` carries every key below on day one. Ecosystem:
    `population_persistence` (per seed: extinction tick or `null`, minimum and
    final population), `births_per_100_ticks`,
    `reachable_structure_size_distribution` (min, p25, median, p75, max, mean
    of `functional_complexity` over the final population, pooled across
    seeds), `strategy_count`, `strategy_causal_distinctness`,
    `evolutionary_activity`, `adaptive_novelty`. Cognition:
    `memory_dependence`, `learning_dependence`, `prediction_dependence`,
    `information_integration`, `reciprocal_interaction`. The first three are
    populated; every other value is the string `"Undefined"`, never zero.
  - `environment`: `generated_at` (RFC 3339), `host` (hostname, OS, arch, CPU
    model when available, logical cores), `build_profile`, `git_revision`,
    `wall_clock_ms` per seed and total, `wall_clock_ms_per_creature_tick`.
  - `comparison`: for each reference given, its path, per-counter percent
    delta, the flag level per counter (`ok`, `flag`, `severe`), counters
    absent from the reference listed as `new`, wall-clock delta only when
    host identity matches, and an overall `severe: bool`.
- [x] Series index `docs/progress/benchmark-series.json`: `{"series":
      "gate-v1", "epoch_baseline": "<report path>", "closed": ["<report
      path>", ...]}` in closure order. This feature's own gate report is the
      epoch baseline and the first closed entry.
- [x] Fast tests in `crates/v3-cli/tests/bench.rs`, run by `make rust-test-cli`
      and therefore `make check`: (1) run the gate profile twice and assert the
      serialized `deterministic` blocks are byte-identical; (2) run the gate
      profile once and compare against the series' epoch baseline and last
      closed report, failing on any `severe` work counter and never on
      wall-clock. Locate the repository docs from `CARGO_MANIFEST_DIR`.
- [x] `make bench` target: `PROFILE=gate FEATURE=<tNN-fNN-slug>` runs the gate
      profile in release, writes `docs/progress/features/$(FEATURE).json`, and
      compares against the series references; `PROFILE=sweep BENCH_ARGS="..."
      OUT=<path>` runs the sweep profile. Document both forms in the `##` help
      text.
- [x] Generate this feature's report with `make bench PROFILE=gate
      FEATURE=t10-f10-deterministic-benchmark-harness`, commit it with the
      series index, and complete Performance and Goal Impact below.

## Verification

- [x] `cargo test -p v3-core --lib` (counter unit tests) and
      `cargo test -p v3-cli` (bench determinism and regression tests) pass.
      Ran: `cargo test -p v3-core --lib` → 1008 passed, 0 failed (includes
      `run_tick_accumulates_reward_modulated_plasticity_update_in_phase_2_5`,
      which pins the Phase 2.5 reward-modulated contribution to
      `plasticity_updates_total` — verified red first by temporarily
      commenting out that increment site, then green after restoring it).
      Ran: `cargo test -p v3-cli --test bench` → 7 tests passed, 13.32s total
      (all run in parallel; well under the 30s budget). The two required
      tests (`gate_profile_deterministic_block_is_byte_identical_across_two_
      runs`, `gate_profile_has_no_severe_regression_against_series_
      references`) run the gate profile; five more synthetic tests exercise
      `compare_against`/`compare_against_path` directly on a tiny report,
      independent of a real regression existing:
      `compare_against_flags_severe_work_counter_regression`,
      `compare_against_labels_missing_reference_counter_as_new`,
      `compare_against_reports_severe_wall_clock_without_marking_comparison_severe`,
      `compare_against_treats_reference_zero_current_positive_as_severe`,
      `compare_against_path_errors_on_profile_mismatch`.
- [x] `make rust-check` passes (format, viability, tests, Clippy with warnings
      denied). Ran: `make rust-check` → format check, viability gate
      (`cargo test -p v3-core --test viability`), full test suite, and Clippy
      (`-D warnings`) all passed.
- [x] `make roadmap-check` passes after the spec and track edits. Ran:
      `make roadmap-check` → passed.
- [x] Running `make bench PROFILE=gate FEATURE=t10-f10-deterministic-benchmark-harness`
      twice yields a byte-identical `deterministic` block (diff the block, not
      the file). This item was verified as a **parsed-object comparison**, not
      a raw byte diff of either the whole file or the isolated `deterministic`
      substring: ran `make bench` twice and compared the two files' parsed
      `deterministic` objects with Python's `json.load` equality (`environment
      .generated_at`/`wall_clock_ms` legitimately differ run to run and were
      excluded by only comparing the `deterministic` key) → identical. Parsed-
      object equality is equivalent to a byte diff of the serialized block
      here because every float inside `deterministic` is a fixed six-decimal
      string (not a number with variable-width formatting) and every object's
      keys are sorted (`serde_json::Value`'s default `BTreeMap`-backed `Map`,
      per the Notes for AI Agents), so two structurally-equal parsed objects
      always re-serialize to the same bytes. The in-process fast test
      (`gate_profile_deterministic_block_is_byte_identical_across_two_runs` in
      `crates/v3-cli/tests/bench.rs`) additionally asserts on the serialized
      string directly, which is the true byte-identity check that runs in
      `make check`.
- [x] `v3-cli bench --profile sweep --width 64 --height 64 --founders 32
      --seeds 1,2 --ticks 50 --out <scratch>` produces a report with the same
      schema as the gate report. Ran it against a scratch path; output
      matched the gate report's top-level and `deterministic` key set.
- [x] Benchmark report stored at
      `docs/progress/features/t10-f10-deterministic-benchmark-harness.json`.

## Performance and Goal Impact

Predeclared cost: four integer increments inside existing loops plus two
sequential sums per tick; expected work-counter change none by construction
(counters do not change control flow), expected wall-clock change under 1
percent. This feature creates the epoch baseline, so the series index
(`docs/progress/benchmark-series.json`) names this feature's own report as
both `epoch_baseline` and the sole `closed` entry; `default_gate_references`
de-duplicates an epoch baseline that equals the last closed report to a
single reference, so the committed report's `comparison.references` array
has exactly one entry (self-comparison), not two. In that self-comparison,
five of the six work counters (`mesh_hops`, `vm_steps`, `graph_relax_iters`,
`actions_applied`, `births`) show `percent_delta: "0.000000"` and
`level: "ok"`; `plasticity_updates` — 0.000000 in both the current run and
the reference — has `percent_delta: null` and `level: "ok"`, because a
reference value of exactly zero makes the percent-delta ratio undefined
rather than zero (see `crates/v3-cli/src/bench.rs::percent_delta`, and the
P2 fix below for the case where the current value is nonzero instead).

Gate profile `per_creature_tick`, from the committed report
(`docs/progress/features/t10-f10-deterministic-benchmark-harness.json`,
`git_revision` `c06c4bd415b28809d46a5a0eb6b4cad7c3c82e1d` — the commit that
introduced this harness; the report was regenerated once at that commit so
`git_revision` identifies code where `v3-cli bench` actually exists, per the
note above that the closing commit's hash cannot appear inside
`deterministic` itself — release build, host Isaacs-MacBook-Pro-2.local /
macOS / aarch64 / Apple M1 Pro / 8 logical cores):

| Counter | Value |
| --- | --- |
| `mesh_hops` | 1.998362 |
| `vm_steps` | 28.028231 |
| `graph_relax_iters` | 2.998624 |
| `plasticity_updates` | 0.000000 — the v3alpha1 founder used by the gate profile does build a CGP graph backend (`build_cgp_founder_graph_with_thresholds_and_reserve` in `creature/cgp_founder.rs`), so `graph_relax_iters` above is nonzero; every one of its compute nodes carries `plasticity: None`, so no Hebbian or reward-modulated update ever fires for this genome |
| `actions_applied` | 1.000000 |
| `births` | 0.001212 |

Wall-clock (recorded only, never asserted), from the same committed report:
`wall_clock_ms_per_creature_tick` = 0.005448 ms (0.005447724313076532
unrounded), on the host above.

Dated readings of the three populated goal indicators (2026-09-03, same
committed report):
- `population_persistence`: no seed went extinct inside the 75-tick horizon;
  per seed (11/22/33) minimum population 256/256/256, final population
  284/260/264.
- `births_per_100_ticks`: 32.888889.
- `reachable_structure_size_distribution` (pooled `functional_complexity`
  over final populations across all three seeds): min 96, p25 96, median 96,
  p75 96, max 106, mean 96.012376.

Every other indicator remains `Undefined` because no owning measure exists
yet.

## Success Criteria

- [x] `v3-cli bench` exists with gate and sweep profiles emitting one schema,
      and `make bench` drives both.
- [x] The gate profile's `deterministic` block is byte-identical across two
      runs, proven by a test in `make check`.
- [x] A severe work-counter regression against the epoch baseline or the last
      closed report fails `make check`; wall-clock never does.
- [x] `docs/progress/features/` and `docs/progress/benchmark-series.json`
      exist with this feature's report pinned as the epoch baseline.
- [x] Every goal indicator named above is present in the report, three
      populated and the rest `"Undefined"`.

## Notes for AI Agents

- The T10 track note saying the core crate does not use `rayon` is stale:
  `tick.rs` runs the mesh phase with `par_iter_mut`. Do not fix that note in
  this feature; it belongs to T10.F09. It is why integer counters are summed
  after the parallel phase and why wall-clock is host-tagged and unasserted.
- Do not put `git_revision` or `generated_at` inside `deterministic`; the
  report is committed, so the closing commit's hash cannot appear in a block
  that must be reproducible.
- Serialize `deterministic` from `BTreeMap`-backed structs or a value tree
  with sorted keys; round every float to six decimals before serialization so
  the byte comparison does not depend on `f32` formatting.
- Small worlds at production defaults went extinct by tick 180 to 300 on
  2026-09-03; an extinction inside the gate horizon is acceptable for a
  benchmark and is recorded as `extinction_tick`, but the counters must still
  be nonzero for every seed. At the revised 75-tick gate horizon none of the
  three seeds went extinct.
- Deviation from the literal spec text: `plasticity_updates` is defined as
  "Hebbian weight updates plus reward-modulated updates" on `MeshOutput`, but
  reward-modulated updates apply in Phase 2.5
  (`apply_reward_modulated_updates` in `tick.rs`), which runs after
  `decisions: Vec<(CreatureId, MeshOutput)>` has already been consumed by
  Phase 2 — there is no `MeshOutput` left to attach a reward-modulated count
  to. The closest faithful implementation: the Hebbian contribution flows
  through `MeshOutput.work_counters.plasticity_updates` (summed sequentially
  in Phase 2, same as the other three work counters), and the
  reward-modulated contribution is added directly to
  `SimStats.plasticity_updates_total` at its Phase 2.5 call site. Both
  additions are sequential (not inside the parallel mesh phase), so the
  cumulative total is still deterministic; the sum still matches the spec's
  definition ("Hebbian plus reward-modulated"). The founder genome used by
  the gate profile has every CGP compute node's `plasticity` set to `None`
  (see the corrected `plasticity_updates` row in Performance and Goal Impact
  above), so this split is untested by the gate profile itself
  (`plasticity_updates` reads 0 for both paths). Two unit tests in
  `runtime/cgp/execute.rs::work_counter_tests` exercise the Hebbian path
  directly; `run_tick_accumulates_reward_modulated_plasticity_update_in_
  phase_2_5` in `simulation/tick/tests/work_counters.rs` exercises the
  reward-modulated path directly, through `run_tick` end to end (verified
  red first by stubbing out the increment, then green).
- Implementation choice, not a deviation: every float inside `deterministic`
  (`per_creature_tick`, `profile.food_coverage`,
  `goal_indicators.births_per_100_ticks`, `...mean`) is serialized as a JSON
  *string* fixed to six decimals (e.g. `"1.998362"`), not a JSON number.
  `serde_json`'s default `f64`/`f32` formatting (via `ryu`) prints the
  shortest round-tripping representation, which drops trailing zeros (`0.5`,
  not `0.500000`) and would not satisfy "fixed six-decimal format" literally.
  A formatted string is unambiguous and trivially byte-identical.
- Remediation-pass fix: `compare_against` treats a counter whose reference
  value is exactly `0.0` and whose current value is `> 0.0` as `severe`
  (division-by-zero — the ratio is unbounded), rather than the previous
  behavior of `level: "ok"` with a `null` delta. Without this, the first
  feature to introduce nonzero work for a counter that every prior report
  recorded as zero (e.g. the first founder genome with a plastic CGP node)
  could never trip a regression, no matter how large the new cost. A
  justified cost still re-pins the epoch baseline in the closing commit, per
  the roadmap's existing rule; this only changes what counts as "justified"
  by making the transition visible instead of silently `ok`. Covered by
  `compare_against_treats_reference_zero_current_positive_as_severe` in
  `crates/v3-cli/tests/bench.rs`.
- Remediation-pass fix: `compare_against_path` hard-fails (`Err`, not a
  silent skip) when a reference report's `deterministic.profile` differs
  from the current run's profile (world size, founders, seeds, ticks, or
  food coverage) — a work-counter comparison across different profiles is
  meaningless. This is a live failure mode, not a hypothetical one: the gate
  horizon changed once already during this feature's own implementation
  (300 to 75 ticks). Covered by `compare_against_path_errors_on_profile_
  mismatch` in `crates/v3-cli/tests/bench.rs`.
- Deferred review findings (recorded, not fixed in this feature): (a)
  `actions_applied_total` in `SimStats` sums `last_tick_move + last_tick_eat
  + last_tick_noop + last_tick_reproduce + last_tick_steal`, each of which
  counts an action the action phase *attempted* to apply (including blocked
  moves and rejected reproduction/steal outcomes), not actions that
  succeeded; the name overstates what is counted. (b) The byte-identity fast
  test (`gate_profile_deterministic_block_is_byte_identical_across_two_runs`)
  runs both builds in the same `cargo test` process, so it proves
  within-process reproducibility only — it does not, by itself, prove
  cross-process or cross-invocation byte-identity; the `make bench` /
  parsed-object double-run check in Verification is the closest evidence for
  the latter, but it is a manual step, not a `make check` gate. (c)
  `default_gate_references` returns an empty reference list (not an error)
  when `docs/progress/benchmark-series.json` does not exist yet; this is
  correct for this feature's own first run (there is no epoch to compare
  against) but should become a hard error once an epoch is pinned, so a
  later feature cannot silently skip its own regression comparison by
  running before the series index is restored (e.g. in a broken worktree).
  (d) `v3-cli bench --profile gate` accepts `--food-coverage` (it is a
  clap argument on `BenchArgs` shared with `--profile sweep`) but silently
  ignores it — the gate profile always uses its own predeclared 1.0 coverage
  regardless of the flag. A future pass should either reject
  `--food-coverage` for `--profile gate` or document the override explicitly.
