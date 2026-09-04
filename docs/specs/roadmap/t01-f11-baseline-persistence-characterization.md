# T01.F11 — Baseline Persistence Characterization

**Status**: In Progress
**Last updated**: 2026-09-04
**Feature**: T01.F11
**Track**: [T01 — Experimental Science and Causal Evaluation](../../roadmaps/t01-experimental-science-and-causal-evaluation.md)

## Goal

Four committed sweep reports, each regenerable by one `make bench` command at
production defaults, record for world sizes 128, 256, 512, and the 1600
default, with density-matched founders and three seeds over 2,000 ticks: the
extinction tick, peak and plateau population, births per 100 ticks, and mean
energy. The result reproduces or supersedes the 2026-09-03 finding that every
world at or below 512-by-512 goes extinct by tick 300 while the default world
plateaus near 5,000 creatures.

## Non-Goals

- No change to simulation behavior, production defaults, founder construction,
  economics, or tick-loop mechanics. This feature only observes.
- No choice of a standard replicate world and no persistence gate or smoke
  test (T01.F12). No throughput or profiling work (T10.F09).
- No ad hoc scripts, notebooks, or plotting; the reports and this spec are the
  deliverable.
- No sweep of food coverage, mutation rates, or any parameter other than world
  size and its density-matched founder count.
- No new goal indicators beyond sampled persistence fields; every `Undefined`
  indicator stays `Undefined`.
- No edits to the stored T10.F10 report or to the regression thresholds.

## Inputs and Invariants

- Contract: T01 roadmap "Notes for AI Agents" paragraph on T01.F11; the
  master roadmap's dated baseline bullet ("T01.F11 supersedes them"); the T10
  roadmap's T10.F10 paragraph (sweep profile exists for characterization runs
  such as this one).
- Dependency output (T10.F10, Complete): `v3-cli bench --profile sweep` in
  `crates/v3-cli/src/bench.rs` and `main.rs`; `make bench PROFILE=sweep
  BENCH_ARGS="..." OUT=<path>`; report schema version 1 with a `deterministic`
  block that must stay byte-identical on re-run; `compare_against_path`
  compares the whole `ProfileBlock` and rejects mismatches; the series index
  `docs/progress/benchmark-series.json` (epoch baseline and last closed report
  are both the T10.F10 report).
- Production defaults (`crates/v3-core/src/config/simulation.rs`): world
  1600 by 1600, 10,000 founders, `max_creatures` 100,000, initial food
  coverage 0.27 for both default food types. (Corrected 2026-09-04 during
  implementation: the planned "0.54 for the shared/primary type and 0.27 for
  the others" is not the production default. `FoodResourceConfig::default()`
  carries 0.54 on the shared block, but `default_food_types()` sets 0.27 on
  both configured types and `SimulationConfig::normalize` then overwrites the
  shared coverage with the primary type's, so production coverage is 0.27
  everywhere. The invariant the sweep needs is unchanged: `None` leaves
  `SimulationConfig::default()` untouched.)
  Density matching keeps one founder per 256 cells: 128² → 64, 256² → 256,
  512² → 1,024, 1600² → 10,000.
- The existing sweep profile forces `--food-coverage` (default 1.0) onto
  every food type, so it cannot currently run at production food coverage.
  This feature must make the sweep able to leave food coverage at production
  defaults. `ProfileParams.food_coverage` becomes `Option<f32>`; `None`
  leaves `SimulationConfig::default()` untouched and serializes
  `profile.food_coverage` as the string `default`, while `Some(x)` behaves as
  today and serializes the six-decimal string. `ProfileBlock.food_coverage`
  stays a `String`, so no report type changes. The gate profile keeps its
  predeclared `Some(1.0)` and its `ProfileBlock` must serialize exactly as
  before, so the stored T10.F10 report still matches as a reference.
- Sweep grid, predeclared here: seeds `[11, 22, 33]`, horizon 2,000 ticks,
  production food coverage, sizes and founders as above. Horizon is the low
  end of the roadmap's 2,000 to 5,000 range because the default world runs
  at roughly 3 ticks per second (2026-09-03 measurement), so three seeds
  already cost on the order of half an hour to an hour of wall-clock.
- Definitions, predeclared here and applied per seed:
  - `peak_population` and `peak_tick`: the maximum population observed after
    any tick from seeding (tick 0, founders) through the last executed tick,
    and the first tick at which it occurred.
  - `plateau_population`: the arithmetic mean of the population over the
    final quarter of the horizon (ticks strictly after `0.75 × horizon`
    through the horizon), as a six-decimal string; `null` when the run went
    extinct before that window began. A run that goes extinct inside the
    window averages only the ticks it executed inside the window; the
    `extinction_tick` makes that visible.
  - `mean_energy`: the mean creature energy at the last executed tick,
    summed in `f64` over creatures in `SlotMap` iteration order, six-decimal
    string; `null` when the population is zero.
  - `samples`: an array of `{tick, population, mean_energy, births_total}`
    taken after `run_tick` at every executed tick that is a multiple of
    `SAMPLE_EVERY_TICKS = 100` and at the last executed tick (deduplicated;
    tick 0 is never sampled because no tick has run), where `births_total` is
    the cumulative `reproduction_actions_spawned_total` and `mean_energy`
    follows the definition above. For the 75-tick gate this is a single
    final-tick sample.
- Invariants: telemetry derives from applied simulation behavior; the new
  fields read state after `run_tick` and never alter control flow. All new
  fields live in the `deterministic` block and follow its rules (sorted keys,
  fixed six-decimal strings, no timestamps). New fields on structs that the
  reference-loading path deserializes carry `#[serde(default)]` so the stored
  T10.F10 report still loads.
- Work counters are unchanged by construction, so the expected per-creature-
  tick delta for every counter is exactly 0 percent against both references.

## Implementation Tasks

- [x] Extend `bench.rs` per-seed observation: track `peak_population`,
      `peak_tick`, `plateau_population`, `mean_energy`, and `samples` as
      defined above, placing them in `PopulationPersistenceSeed` (per-seed
      entries under `goal_indicators.population_persistence`) with
      `#[serde(default)]` on each new field. Add `SAMPLE_EVERY_TICKS` as a
      named constant. Implement the per-seed tracking as a small pure
      accumulator (fed the horizon, then one `(tick, population, mean_energy,
      births_total)` observation per executed tick) so it is unit-testable
      without running a simulation: synthetic tests assert peak and first
      peak tick, plateau `null` on extinction before the window, plateau over
      a partial window, mean energy `null` at extinction, and the exact
      sample tick set including the deduplicated final tick. One integration
      test on a tiny sweep (for example 32 by 32, 8 founders, 30 ticks)
      asserts the fields are present and byte-identical across two runs.
- [x] Make `--food-coverage` optional for `--profile sweep` per the
      `Option<f32>` rule above (drop the clap `default_value`). Reject
      `--food-coverage` for `--profile gate` instead of silently ignoring it
      (closes T10.F10 deferred finding (d)). Update the `make bench` help
      text. Tests cover: `build_config` without coverage leaves the production
      coverage values untouched; the `default` profile string round-trips through
      the profile-mismatch comparison; the stored T10.F10 report still loads
      and matches the gate profile. Factor the argument-to-profile resolution
      in `main.rs` into a function returning `Result` so the gate rejection is
      unit-tested without spawning the binary.
- [x] Run the four sweeps and commit their reports under
      `docs/progress/sweeps/t01-f11/` as `w0128.json`, `w0256.json`,
      `w0512.json`, `w1600.json`, each produced by exactly one command of the
      form `make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/wNNNN.json
      BENCH_ARGS="--width N --height N --founders F --seeds 11,22,33 --ticks
      2000 --feature t01-f11-baseline-persistence-characterization"`. Run the
      1600 sweep detached (`nohup ... &`) and poll it; it may take an hour.
      Record the exact commands and wall-clock in Verification.
- [x] Generate this feature's gate report with `make bench PROFILE=gate
      FEATURE=t01-f11-baseline-persistence-characterization`, append it to
      `closed` in `docs/progress/benchmark-series.json`, and complete
      Performance and Goal Impact.
- [x] Update the master roadmap's dated baseline bullet in `docs/roadmap.md`
      to cite this feature's reports and measured persistence result in place
      of the 2026-09-03 `v3-cli run` persistence numbers, in one or two
      sentences. Leave its throughput sentence for T10.F09 and leave the T01
      track's T01.F11 paragraph unchanged; it is this feature's contract.

## Verification

Run on 2026-09-04 at commit `d4d5005a` (the code commit; the sweep and gate
reports all record `git_revision` `d4d5005a`), macOS arm64, 12 logical cores.

- [x] `cargo test -p v3-cli` passes with the new persistence-field and
      food-coverage tests; the two existing gate tests still pass against the
      unmodified T10.F10 report. Result: 9 lib unit tests (7 accumulator, 2
      food coverage), 6 `main.rs` resolution unit tests, 9 `tests/bench.rs`
      integration tests including the new
      `tiny_sweep_records_persistence_fields_identically_across_two_runs` and
      `default_food_coverage_round_trips_through_the_profile_comparison`, and
      7 `tests/cli.rs` tests — 31 passed, 0 failed.
      `gate_profile_has_no_severe_regression_against_series_references` is the
      end-to-end proof that the stored T10.F10 report still deserializes and
      its `ProfileBlock` still equals the gate profile's.
- [x] `make rust-check` passes (format, clippy `-D warnings`, viability 25/25,
      v3-core 1008 unit tests, and the full workspace test set; 0 failures).
- [x] `make roadmap-check` passes after the spec and master edits:
      `roadmap-check: validation passed`.
- [x] Re-running the 128 sweep command to
      `<scratch>/w0128-rerun.json` yields a `deterministic` block
      byte-identical to the committed `w0128.json` (parsed objects compared
      with sorted keys; 2,811 identical bytes).
- [x] Benchmark report stored at
      `docs/progress/features/t01-f11-baseline-persistence-characterization.json`
      and appended to `closed` in `docs/progress/benchmark-series.json`.
- [x] `make check` passes (exit 0) after that append, so the gate profile
      still compares cleanly against both series references — the T10.F10
      report and this feature's own.

Sweep commands, each run once from the repository root, with the wall-clock
each report recorded in `environment.wall_clock_ms_total` (sum over its three
seeds, release build):

```sh
make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w0128.json BENCH_ARGS="--width 128 --height 128 --founders 64 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w0256.json BENCH_ARGS="--width 256 --height 256 --founders 256 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w0512.json BENCH_ARGS="--width 512 --height 512 --founders 1024 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w1600.json BENCH_ARGS="--width 1600 --height 1600 --founders 10000 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
make bench PROFILE=gate FEATURE=t01-f11-baseline-persistence-characterization
```

Wall-clock: 128 → 0.5 s, 256 → 7.8 s, 512 → 594.3 s, 1600 → 533.5 s, gate →
0.3 s. The 1600 sweep was launched detached with `nohup` and polled; it
finished in about nine minutes, far under the hour the plan budgeted, because
the release build runs the default world at roughly 11 ticks per second
rather than the 3 ticks per second measured through `v3-cli run` on
2026-09-03. The 512 sweep is the slowest of the four: its surviving seed
plateaus at about 12,300 creatures, roughly twice the default world's plateau.

## Performance and Goal Impact

Predeclared cost: one population comparison per tick, one `O(population)`
energy sum every 100 ticks and at the end of each seed, and a few dozen
sample entries per seed in the report. Expected work-counter delta against
both references: exactly 0 percent on every counter. Expected wall-clock
delta: under 1 percent. No threshold is expected to be crossed and the epoch
baseline is not re-pinned.

Measured 2026-09-04, gate report
`docs/progress/features/t01-f11-baseline-persistence-characterization.json`.
The epoch baseline and the last closed report were the same file when it was
generated, so it carries one reference comparison, against the T10.F10 report:

| counter (per creature-tick) | current | T10.F10 | delta | level |
| --- | --- | --- | --- | --- |
| `mesh_hops` | 1.998362 | 1.998362 | 0.000000% | ok |
| `vm_steps` | 28.028231 | 28.028231 | 0.000000% | ok |
| `graph_relax_iters` | 2.998624 | 2.998624 | 0.000000% | ok |
| `plasticity_updates` | 0.000000 | 0.000000 | `null` | ok |
| `actions_applied` | 1.000000 | 1.000000 | 0.000000% | ok |
| `births` | 0.001212 | 0.001212 | 0.000000% | ok |

As predeclared, every counter is exactly 0 percent and `plasticity_updates`
is zero in both runs, so its delta is `null` at level `ok`. Simulation
behavior is unchanged. Wall-clock, which never gates, was 0.004614 ms per
creature-tick against 0.005448 (-15.31 percent, level `ok`) on the same host.
The epoch baseline is not re-pinned.

Dated goal-indicator reading (gate profile: 128 by 128, 256 founders, seeds
11/22/33, 75 ticks, forced food coverage 1.0):

- `population_persistence`: no seed goes extinct. Seed 11 peaks at 290 at
  tick 40, plateaus at 287.315789, ends at 284 with mean energy 8.321675;
  seed 22 peaks at 274 at tick 40, plateaus at 268.210526, ends at 260 with
  mean energy 7.453132; seed 33 peaks at 277 at tick 40, plateaus at
  268.736842, ends at 264 with mean energy 7.945192. Each seed carries one
  sample, the 75-tick final sample, as the 100-tick cadence predicted.
- `births_per_100_ticks`: 32.888889, unchanged from the T10.F10 report.
- `reachable_structure_size_distribution`: min 96, p25 96, median 96, p75 96,
  max 106, mean 96.012376 — unchanged from the T10.F10 report.
- Every other indicator remains `Undefined`.

Sweep results (production food coverage, seeds 11/22/33, 2,000 ticks,
density-matched founders; births per 100 ticks is derived here from the
report's per-seed `births` and `ticks`, which is why it is not a stored field;
a dash means the run was extinct so the field is `null`):

| world | founders | seed | extinction tick | peak population (tick) | plateau population | births/100 ticks | final mean energy |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 128² | 64 | 11 | 161 | 77 (43) | — | 9.32 | — |
| 128² | 64 | 22 | 175 | 76 (33) | — | 8.00 | — |
| 128² | 64 | 33 | 167 | 80 (46) | — | 10.18 | — |
| 256² | 256 | 11 | 151 | 309 (37) | — | 41.06 | — |
| 256² | 256 | 22 | none | 304 (26) | 1.000000 | 2.75 | 33.910748 |
| 256² | 256 | 33 | 158 | 309 (36) | — | 37.97 | — |
| 512² | 1024 | 11 | none | 22592 (1182) | 12336.136000 | 5851.05 | 71.751983 |
| 512² | 1024 | 22 | 256 | 1237 (35) | — | 124.61 | — |
| 512² | 1024 | 33 | 202 | 1222 (37) | — | 134.65 | — |
| 1600² | 10000 | 11 | none | 35327 (130) | 6105.724000 | 6944.45 | 31.220906 |
| 1600² | 10000 | 22 | none | 36265 (143) | 6724.372000 | 7646.65 | 41.842101 |
| 1600² | 10000 | 33 | none | 35363 (147) | 6226.670000 | 7318.35 | 33.438039 |

**The 2026-09-03 result is reproduced for the default world and superseded
for the small worlds.** The default 1600 by 1600 world persists on all three
seeds: it peaks near 35,000 to 36,000 around ticks 130 to 147, crashes to a
minimum of 4,310 to 5,003 around ticks 800 to 1,000, and recovers slowly to
6,790 to 7,291 at tick 2,000, so "plateaued near 5,000" holds as the trough
but the population is drifting upward, not flat, over the second half. The
claim that every world at or below 512 by 512 goes extinct by tick 300 does
not hold. Only 128 by 128 went extinct on every seed (ticks 161, 167, 175).
At 256 by 256 seed 22 survived all 2,000 ticks, but at a single creature from
about tick 200 onward — survival of the state, not of a population. At 512 by
512 seed 11 survived with the largest plateau measured anywhere in the sweep
(12,336): it nearly collapsed to 42 creatures near tick 200, recovered to
22,534 by tick 1,200, and then declined steadily to 9,206 at tick 2,000, so
its plateau figure is the mean of a falling tail rather than a settled level.
Persistence below the default size is therefore seed-dependent and bimodal,
not uniform extinction, and the extinctions that do occur happen earlier
(ticks 151 to 256) than the 180-to-300 band recorded on 2026-09-03. Choosing
a standard replicate world (T01.F12) has to treat the 512 and 256 survivals
as marginal: one is a single creature and the other has not settled by tick
2,000.

## Success Criteria

- [ ] Four sweep reports at production defaults, sizes 128/256/512/1600 with
      density-matched founders, seeds 11/22/33, 2,000 ticks, are committed and
      each is regenerable by the one command recorded in this spec.
- [ ] Each report records per seed the extinction tick, peak and plateau
      population, births per 100 ticks, mean energy, and 100-tick samples.
- [ ] The Performance and Goal Impact section states whether the 2026-09-03
      baseline result is reproduced or superseded, with numbers from the
      reports.
- [ ] Simulation behavior is unchanged: every work counter's per-creature-tick
      delta in the gate report is 0 percent against both references, except a
      counter that is zero in both runs, which reports a `null` delta at
      level `ok`.

## Notes for AI Agents

- The sweep runs seeds sequentially in one process. Small worlds stop at
  extinction, so their sweeps finish in seconds to minutes; the default world
  is the long one. Do not shorten the horizon below 2,000 or drop a seed to
  save time; if the 1600 sweep cannot complete on the recording host, record
  the concrete blocker here and stop.
- Keep the gate `ProfileBlock` serialization byte-for-byte unchanged, or the
  T10.F10 reference will fail the profile-mismatch check in `make check`.
- T10.F10 deferred maintainability findings (string comparison levels,
  hand-built `PerCreatureTick` lookup, counter growth path, hand-rolled
  environment helpers, repeated clap validation, inline wall-clock literals)
  stay deferred to T10.F09 except finding (d), which this feature closes.
  clap cannot express "forbidden when `--profile gate`" declaratively, so
  one explicit check in the resolution function is acceptable; do not add
  further print-and-exit branches to `run_bench`.
