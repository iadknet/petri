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
  coverage 0.54 for the shared/primary food type and 0.27 for the others.
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

- [ ] Extend `bench.rs` per-seed observation: track `peak_population`,
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
- [ ] Make `--food-coverage` optional for `--profile sweep` per the
      `Option<f32>` rule above (drop the clap `default_value`). Reject
      `--food-coverage` for `--profile gate` instead of silently ignoring it
      (closes T10.F10 deferred finding (d)). Update the `make bench` help
      text. Tests cover: `build_config` without coverage leaves the production
      0.54 / 0.27 values; the `default` profile string round-trips through
      the profile-mismatch comparison; the stored T10.F10 report still loads
      and matches the gate profile. Factor the argument-to-profile resolution
      in `main.rs` into a function returning `Result` so the gate rejection is
      unit-tested without spawning the binary.
- [ ] Run the four sweeps and commit their reports under
      `docs/progress/sweeps/t01-f11/` as `w0128.json`, `w0256.json`,
      `w0512.json`, `w1600.json`, each produced by exactly one command of the
      form `make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/wNNNN.json
      BENCH_ARGS="--width N --height N --founders F --seeds 11,22,33 --ticks
      2000 --feature t01-f11-baseline-persistence-characterization"`. Run the
      1600 sweep detached (`nohup ... &`) and poll it; it may take an hour.
      Record the exact commands and wall-clock in Verification.
- [ ] Generate this feature's gate report with `make bench PROFILE=gate
      FEATURE=t01-f11-baseline-persistence-characterization`, append it to
      `closed` in `docs/progress/benchmark-series.json`, and complete
      Performance and Goal Impact.
- [ ] Update the master roadmap's dated baseline bullet in `docs/roadmap.md`
      to cite this feature's reports and measured persistence result in place
      of the 2026-09-03 `v3-cli run` persistence numbers, in one or two
      sentences. Leave its throughput sentence for T10.F09 and leave the T01
      track's T01.F11 paragraph unchanged; it is this feature's contract.

## Verification

- [ ] `cargo test -p v3-cli` passes with the new persistence-field and
      food-coverage tests; the two existing gate tests still pass against the
      unmodified T10.F10 report.
- [ ] `make rust-check` passes.
- [ ] `make roadmap-check` passes after the spec, track, and master edits.
- [ ] Re-running the 128 sweep command to a scratch path yields a
      `deterministic` block byte-identical to the committed `w0128.json`
      (compare the parsed `deterministic` objects, as T10.F10 did).
- [ ] Benchmark report stored at
      `docs/progress/features/t01-f11-baseline-persistence-characterization.json`.

## Performance and Goal Impact

Predeclared cost: one population comparison per tick, one `O(population)`
energy sum every 100 ticks and at the end of each seed, and a few dozen
sample entries per seed in the report. Expected work-counter delta against
both references: exactly 0 percent on every counter. Expected wall-clock
delta: under 1 percent. No threshold is expected to be crossed and the epoch
baseline is not re-pinned.

To be completed at closure: the gate report's per-counter deltas against the
T10.F10 report (previous closed feature and epoch baseline), the dated reading
of `population_persistence` (now including peak, plateau, mean energy, and
samples), `births_per_100_ticks`, and
`reachable_structure_size_distribution`, and a compact table of the four
sweep reports: per world size, per seed, extinction tick, peak population,
plateau population, births per 100 ticks over the whole run, and final mean
energy, with the sentence stating whether the 2026-09-03 result is
reproduced or superseded.

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
