# T10.F09 — Throughput Baseline and Profiling Budget

**Status**: In Progress
**Last updated**: 2026-09-04
**Feature**: T10.F09
**Track**: [T10 — Evolutionary Scale and Experiment Infrastructure](../../roadmaps/t10-evolutionary-scale-and-experiment-infrastructure.md)

## Goal

Eight committed sweep reports, each regenerable by one `make bench` command,
record wall-clock per tick and per creature-tick at every T01.F11 sweep point
on one thread and on all eight logical cores; the tick loop reports where its
wall-clock goes by phase; the spec records a per-core budget in ticks and
births per wall-clock hour and decides, by a rule predeclared below, whether a
core-parallelism feature must precede any campaign.

## Non-Goals

- No parallelism implementation and no optimization of any hot path; this
  feature measures. If the rule below says parallelism is required, it adds
  one dependency-linked feature row, not code.
- No change to simulation behavior, production defaults, founders, or the
  order and content of tick phases. Timers observe phases; they do not move
  work between them.
- No choice of the standard replicate world and no persistence gate
  (T01.F12). No campaign orchestration and no `v3-cli run` changes (T10.F04).
- No peak-memory measurement and no new goal indicators; every `Undefined`
  indicator stays `Undefined`.
- No replacement of the hand-rolled RFC 3339, hostname, CPU-model, and git
  helpers in `bench.rs`: no existing workspace dependency provides them, so
  that T10.F10 deferral stays deferred. The counter growth path (adding a
  work counter touches six places) also stays deferred to the first feature
  that adds a work counter; this feature adds none.

## Inputs and Invariants

- Contract: the T10 roadmap "Notes for AI Agents" paragraph on T10.F09; the
  master roadmap's throughput sentence in its dated baseline bullet ("T10.F09
  owns re-measuring it") and its scale reference of roughly 190 steps per
  second with 60,000 agents, about 11.4 million agent-steps per second on a
  GPU; the T10.F10 spec note that correcting the stale "core crate does not
  use `rayon`" sentence belongs to T10.F09; the T01.F11 spec note deferring
  the T10.F10 maintainability findings to T10.F09.
- Dependency output (T01.F11, Complete): the sweep grid, reused here
  unchanged, seeds `[11, 22, 33]`, 2,000 ticks, production food coverage,
  density-matched founders: 128² → 64, 256² → 256, 512² → 1,024,
  1600² → 10,000. Its four reports under `docs/progress/sweeps/t01-f11/` ran
  on the rayon default pool (8 threads) at commit `d4d5005a` and recorded
  wall-clock per creature-tick of 0.024926 ms (128²), 0.096047 ms (256²),
  0.031235 ms (512²), and 0.009512 ms (1600²), and 11.1 to 11.4 ticks per
  second for the three 1600² seeds. The 256² seed 22 run held one creature
  for about 1,800 ticks at roughly 3.3 ms per tick, which is the per-tick
  world-size cost with no creature work in it.
- Existing code: `crates/v3-core/src/simulation/tick.rs` runs, in order,
  Phase 0 (`run_phase_0`: food growth, aging, decay, death removal), the
  turn-queue build, Phase 1a (`assemble_sensor_inputs`, sequential), Phase 1b
  (`run_cognition`, `par_iter_mut` on the rayon global pool), priority sort,
  Phase 2 (`run_phase_2`, sequential actions), and Phase 2.5
  (`run_reward_learning`). `SimStats` derives no serde traits and is copied
  field by field into server telemetry. `crates/v3-cli/src/bench.rs` records
  wall-clock only in `environment` (`wall_clock_ms_per_seed`,
  `wall_clock_ms_total`, `wall_clock_ms_per_creature_tick`). `v3-cli` does not
  depend on `rayon`; the workspace pins `rayon = "1.10"`. Comparison levels
  are the strings `ok`, `flag`, `severe`, `new`, and the wall-clock thresholds
  (25 percent flag, doubling severe) are inline literals next to the named
  work-counter constants.
- Recording host: Apple M1 Pro, 8 logical cores (6 performance, 2
  efficiency), macOS, release build (`[profile.release] debug = true`, so
  `/usr/bin/sample` symbolicates it). `perf`, `samply`, and `cargo flamegraph`
  are not installed and are not added.
- Invariants: the `deterministic` block is unchanged byte for byte, so every
  report this feature produces has a `deterministic` object equal to the
  T01.F11 report at the same sweep point, on either thread count; thread
  count and every timing live only in `environment`; wall-clock is never
  asserted; work counters change by exactly 0 percent; production code is
  never edited to kill a mutant; shell automation is POSIX `sh`.
- Report schema additions, all under `environment`, all `#[serde(default)]`
  so stored reports still load:
  - `threads`: the rayon thread count in effect during the run, read with
    `rayon::current_num_threads()` from inside the pool that ran it.
  - `phase_wall_clock_ms_per_seed[]`: per seed, cumulative milliseconds in
    `world_update` (Phase 0), `sensor_assembly` (Phase 1a), `cognition`
    (Phase 1b), `actions` (Phase 2), and `reward_learning` (Phase 2.5),
    summed over the seed's executed ticks. The queue build and priority sort
    are not timed; they are the remainder against `wall_clock_ms`.
  - `throughput`: per seed and total, `ticks_per_second`,
    `creature_ticks_per_second`, `ticks_per_hour`, and `births_per_hour`,
    each derived from that report's own counters and wall-clock. The
    per-core budget is this block read from a `--threads 1` report.
- Thread control: `v3-cli bench --threads <n>` (both profiles, `n ≥ 1`) runs
  the profile inside `rayon::ThreadPool::install` on a pool of `n` threads;
  without the flag the global pool is used as today. `install` rather than
  `build_global`, so the fast tests can run one profile on one thread and
  again on the default pool in one process.
- Measurement grid, predeclared: the four T01.F11 sweep points, each at
  `--threads 1` and `--threads 8`, reports at
  `docs/progress/sweeps/t10-f09/wNNNN-t1.json` and `wNNNN-t8.json`. The
  horizon and seeds are not shortened to save time; the one-thread 512² and
  1600² runs may each take an hour.
- Concurrency probe, predeclared: 1600², 10,000 founders, seed 11, 200 ticks
  (the peak-population window, the heaviest ticks of the sweep), one thread.
  Run once alone, then run eight copies concurrently from one POSIX `sh` loop
  writing to scratch paths. The slowdown `c` is the mean `wall_clock_ms_total`
  of the eight concurrent runs divided by the lone run's. Replicate-level
  throughput on this host is `8 / c` lone-run equivalents.
- Decision rule, predeclared. Let `S` be the 1600² sweep's
  `wall_clock_ms_total` at one thread divided by the same at eight threads,
  and let `T` be the projected one-thread wall-clock for 5,000 ticks of the
  1600² world (the top of the T01.F11 horizon range), extrapolated linearly
  from the one-thread 2,000-tick sweep's slowest seed. A core-parallelism
  feature is required before any campaign if and only if `T` exceeds eight
  hours, an overnight run. If required, add the row `T10.F11 — Sequential
  Tick-Phase Parallelism — Depends on: T10.F09` to the T10 track, make
  T10.F07 depend on it, and write no code for it here. Independently of that
  outcome, record for T10.F04 which of `8 / c` and `S` is larger: it decides
  whether campaigns run one replicate per core on one thread or replicates
  in sequence on all cores.

## Implementation Tasks

- [ ] Phase timing in `v3-core`. Add `PhaseWallClock` (five cumulative
      `std::time::Duration` fields named as in the schema above) to
      `SimStats`, accumulated in `run_tick` around the five phase calls with
      `Instant`; cumulative, never reset by `reset_tick_counters`. TDD: a
      unit test under `simulation/tick/tests/` asserts each field is
      non-decreasing across two `run_tick` calls and that the tick counters
      reset while these do not. Run `cargo test -p v3-core --test viability`
      first, since `tick.rs` changes.
- [ ] Bench extensions in `crates/v3-cli`. Add `rayon` to the crate's
      dependencies. Add `--threads` per the thread-control rule, rejecting
      `0` in `resolve_bench_profile` (unit test). Record `threads`,
      `phase_wall_clock_ms_per_seed`, and `throughput` in `environment`.
      Compute `throughput` in one pure function; a proptest asserts, for any
      positive wall-clock and any counters, that each rate multiplied by the
      elapsed time recovers its counter within floating-point tolerance and
      that a zero wall-clock yields zero rates rather than infinity. One
      integration test in `crates/v3-cli/tests/bench.rs` runs a tiny sweep on
      a one-thread pool and on the default pool and asserts the
      `deterministic_block_json` strings are byte-identical and the
      `environment.threads` values differ as expected.
- [ ] Pay down two T10.F10 maintainability deferrals touched by this work:
      comparison levels become a serde-renamed enum, and the wall-clock flag
      and severe thresholds become named constants beside the work-counter
      ones. The serialized `comparison` and `deterministic` JSON must not
      change; the existing gate tests against the stored references prove it.
- [ ] Commit the code, then run the eight sweeps from the repository root,
      the long ones detached with `nohup` and polled. Each command has the
      form below with `N`, `F`, `T`, and the output name substituted:

      ```sh
      make bench PROFILE=sweep OUT=docs/progress/sweeps/t10-f09/wNNNN-tT.json BENCH_ARGS="--width N --height N --founders F --seeds 11,22,33 --ticks 2000 --threads T --feature t10-f09-throughput-baseline-and-profiling-budget"
      ```

      Then check every report's parsed `deterministic` object equals the
      T01.F11 report at the same sweep point (`python3` with `json.load`),
      and record the result.
- [ ] Run the concurrency probe and record the lone and mean concurrent
      `wall_clock_ms_total`, `c`, and the exact loop.
- [ ] Profile one one-thread 1600² seed-11 run of 200 ticks: after
      `cargo build --release -p v3-cli`, launch `target/release/v3-cli bench`
      directly in the background so the pid is the benchmark's, then run
      `/usr/bin/sample <pid> 30 -file <path>` while it executes. Keep the raw
      output under `~/.local/share/petri-tools/profiles/t10-f09/` and record
      the path and the ten heaviest frames by sample count, with their share,
      in Performance and Goal Impact.
- [ ] Complete Performance and Goal Impact: the per-core budget table, the
      phase-share table, the per-tick cost decomposition (Phase 0
      milliseconds per tick and nanoseconds per cell; the other phases in
      microseconds per creature-tick, both from the one-thread reports), `S`,
      `c`, `T`, and the decision with its numbers.
- [ ] Roadmap edits. In the T10 track's T10.F09 paragraph replace the
      sentence carrying the 2026-09-03 rates and the claim that the core
      crate does not use `rayon` with one sentence giving the measured one-
      and eight-thread 1600² ticks per second and stating that Phase 1b
      already runs on the rayon global pool while every other phase is
      sequential, and append one sentence of T10.F04 guidance from the
      decision. If the rule requires parallelism, add the T10.F11 row and
      the T10.F07 dependency. In `docs/roadmap.md` replace the "about 3 ticks
      per second single-threaded (2026-09-03; T10.F09 owns re-measuring it)"
      sentence with the measured one-thread and eight-thread 1600² figures
      citing `docs/progress/sweeps/t10-f09/`. Leave the T10 track success
      criterion on the standard replicate world unchecked: that world is
      chosen by T01.F12, which cites this budget.
- [ ] Generate this feature's gate report with `make bench PROFILE=gate
      FEATURE=t10-f09-throughput-baseline-and-profiling-budget`, append it to
      `closed` in `docs/progress/benchmark-series.json`, and complete the
      gate half of Performance and Goal Impact.

## Verification

- [ ] `cargo test -p v3-core --test viability` passes before other checks.
- [ ] `cargo test -p v3-core --lib` passes with the phase-timing test.
- [ ] `cargo test -p v3-cli` passes with the threads, throughput proptest,
      and thread-independence tests, and the two stored gate references still
      load and match.
- [ ] `make rust-check` and `make roadmap-check` pass.
- [ ] Every `docs/progress/sweeps/t10-f09/*.json` `deterministic` object
      equals its T01.F11 counterpart.
- [ ] `make rust-mutants` summary line, output path, and full survivor list,
      each survivor killed, equivalent, or deferred.
- [ ] Benchmark report stored at
      `docs/progress/features/t10-f09-throughput-baseline-and-profiling-budget.json`.
- [ ] `make check` passes (exit 0) at the closing commit.

## Performance and Goal Impact

Predeclared cost: at most ten `Instant::now()` reads and five `Duration`
additions per tick, on the order of a microsecond against ticks that cost
milliseconds.
Expected work-counter delta against both references: exactly 0 percent on
every counter, `plasticity_updates` `null` at level `ok`. Expected wall-clock
delta: under 1 percent. No threshold is expected to be crossed and the epoch
baseline is not re-pinned.

Measured results, the per-core budget, the phase and hot-frame profile, `S`,
`c`, `T`, and the decision are recorded here at closure.

## Success Criteria

- [ ] Eight sweep reports at the T01.F11 grid, one and eight threads, are
      committed, each regenerable by the one command form above, each with a
      `deterministic` object equal to the T01.F11 report at that sweep point.
- [ ] Every bench report records `threads`, per-phase wall-clock, and
      throughput in `environment`, and the per-core budget in ticks and
      births per wall-clock hour at every sweep point is written in this spec
      and cited by the master roadmap.
- [ ] The tick profile is recorded: phase shares at one and eight threads for
      every sweep point and the ten heaviest sampled frames.
- [ ] The parallelism decision is recorded with `S`, `c`, and `T`, and the
      T10 track carries the T10.F11 row exactly when the rule requires it.
- [ ] Every work counter's per-creature-tick delta in the gate report is 0
      percent against both references, except a counter that is zero in both
      runs, which reports a `null` delta at level `ok`.

## Notes for AI Agents

- Run the long sweeps detached and poll them; do not shorten the horizon or
  drop a seed. If the one-thread grid cannot complete on the recording host,
  record the concrete blocker here and stop.
- `rayon::ThreadPool::install` scopes `par_iter_mut` to the installed pool;
  `build_global` can only succeed once per process and would break the
  in-process tests.
- Timings are wall-clock: they belong in `environment`, never in
  `deterministic`, and no test asserts their magnitude.
