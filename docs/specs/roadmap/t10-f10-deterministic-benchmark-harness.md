# T10.F10 — Deterministic Benchmark Harness and Per-Feature Report

**Status**: Planned
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
  `[11, 22, 33]`, horizon 300 ticks. Sized so the two fast tests below finish
  in under 30 seconds combined in a debug build on the recording host; if they
  do not, reduce the horizon first, then the founder count, and record the
  final values in this spec.
- Regression thresholds, applied per normalized work counter against each
  reference: flag above +10 percent, severe above +50 percent. Wall-clock per
  creature-tick: flag above +25 percent, severe above a doubling, reported only
  when the reference's host identity matches, never asserted.

## Implementation Tasks

- [ ] Add integer work counters. `MeshOutput` gains a `WorkCounters` value
      with `mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`
      (Hebbian weight updates plus reward-modulated updates), each incremented
      where that work executes in `mesh.rs`, `vm.rs`, `cgp/execute.rs`, and
      the plasticity modules, in both the plain and traced modes. `SimStats`
      gains cumulative `u64` totals for those four, plus `creature_ticks_total`
      (creatures that ran the mesh, summed per tick) and
      `actions_applied_total` (every action the action phase executed: move,
      eat, noop, reproduce, steal), accumulated sequentially after the parallel
      phase. Births are the existing `reproduction_actions_spawned_total`.
      Unit tests assert each counter from a run whose expected count is known.
- [ ] Add `crates/v3-cli/src/bench.rs` with `v3-cli bench` in `main.rs`:
      `--profile gate|sweep`, `--out <path>`, `--feature <id>` (required for
      gate), `--compare <report.json>` and `--baseline <report.json>` (each
      optional, repeatable), and for `sweep` only: `--width`, `--height`,
      `--founders`, `--seeds <comma list>`, `--ticks`, `--food-coverage`
      (default 1.0). Runs each seed with `seed_simulation` and `run_tick`,
      records per-seed and aggregate results, writes the report, prints the
      comparison, and exits 3 on a severe work regression.
- [ ] Report schema, one file per feature at
      `docs/progress/features/<tNN-fNN-slug>.json`, top-level keys
      `schema_version` (1), `feature`, `deterministic`, `environment`,
      `comparison`:
  - `deterministic`: `profile` (name and every input parameter),
    `per_seed[]` (seed, `ticks`, `creature_ticks`, `mesh_hops`, `vm_steps`,
    `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`,
    `final_population`, `extinction_tick` or `null`), `totals` (same counters
    summed), `per_creature_tick` (each work counter divided by
    `creature_ticks`, six decimals), and `goal_indicators`.
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
- [ ] Series index `docs/progress/benchmark-series.json`: `{"series":
      "gate-v1", "epoch_baseline": "<report path>", "closed": ["<report
      path>", ...]}` in closure order. This feature's own gate report is the
      epoch baseline and the first closed entry.
- [ ] Fast tests in `crates/v3-cli/tests/bench.rs`, run by `make rust-test-cli`
      and therefore `make check`: (1) run the gate profile twice and assert the
      serialized `deterministic` blocks are byte-identical; (2) run the gate
      profile once and compare against the series' epoch baseline and last
      closed report, failing on any `severe` work counter and never on
      wall-clock. Locate the repository docs from `CARGO_MANIFEST_DIR`.
- [ ] `make bench` target: `PROFILE=gate FEATURE=<tNN-fNN-slug>` runs the gate
      profile in release, writes `docs/progress/features/$(FEATURE).json`, and
      compares against the series references; `PROFILE=sweep BENCH_ARGS="..."
      OUT=<path>` runs the sweep profile. Document both forms in the `##` help
      text.
- [ ] Generate this feature's report with `make bench PROFILE=gate
      FEATURE=t10-f10-deterministic-benchmark-harness`, commit it with the
      series index, and complete Performance and Goal Impact below.

## Verification

- [ ] `cargo test -p v3-core --lib` (counter unit tests) and
      `cargo test -p v3-cli` (bench determinism and regression tests) pass.
- [ ] `make rust-check` passes (format, viability, tests, Clippy with warnings
      denied).
- [ ] `make roadmap-check` passes after the spec and track edits.
- [ ] Running `make bench PROFILE=gate FEATURE=t10-f10-deterministic-benchmark-harness`
      twice yields a byte-identical `deterministic` block (diff the block, not
      the file).
- [ ] `v3-cli bench --profile sweep --width 64 --height 64 --founders 32
      --seeds 1,2 --ticks 50 --out <scratch>` produces a report with the same
      schema as the gate report.
- [ ] Benchmark report stored at
      `docs/progress/features/t10-f10-deterministic-benchmark-harness.json`.

## Performance and Goal Impact

Predeclared cost: four integer increments inside existing loops plus two
sequential sums per tick; expected work-counter change none by construction
(counters do not change control flow), expected wall-clock change under 1
percent. This feature creates the epoch baseline, so both references are its
own report and every delta is zero. Record here at closure: the gate profile's
`per_creature_tick` values, the wall-clock per creature-tick with host
identity, and the dated readings of the three populated goal indicators.
Every other indicator remains `Undefined` because no owning measure exists
yet.

## Success Criteria

- [ ] `v3-cli bench` exists with gate and sweep profiles emitting one schema,
      and `make bench` drives both.
- [ ] The gate profile's `deterministic` block is byte-identical across two
      runs, proven by a test in `make check`.
- [ ] A severe work-counter regression against the epoch baseline or the last
      closed report fails `make check`; wall-clock never does.
- [ ] `docs/progress/features/` and `docs/progress/benchmark-series.json`
      exist with this feature's report pinned as the epoch baseline.
- [ ] Every goal indicator named above is present in the report, three
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
  be nonzero for every seed.
