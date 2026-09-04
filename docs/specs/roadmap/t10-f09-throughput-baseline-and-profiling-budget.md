# T10.F09 — Throughput Baseline and Profiling Budget

**Status**: Complete
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
  never edited to kill a mutant; shell automation is POSIX `sh`. (Corrected
  2026-09-04 during implementation: the equality half of the first invariant
  was false of the code before this feature. Seeded runs diverge between
  processes at 512² and above; see the determinism finding under Performance
  and Goal Impact. The serialization of the block is unchanged, which the
  gate references prove; it is the simulation that is not reproducible.)
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
  hours, an overnight run. If required, add a row `Sequential Tick-Phase
  Parallelism — Depends on: T10.F09` to the T10 track under the next free ID
  (T10.F12, since T10.F11 was assigned to reproducibility on 2026-09-04),
  make T10.F07 depend on it, and write no code for it here. Independently of that
  outcome, record for T10.F04 which of `8 / c` and `S` is larger: it decides
  whether campaigns run one replicate per core on one thread or replicates
  in sequence on all cores.

## Implementation Tasks

- [x] Phase timing in `v3-core`. Add `PhaseWallClock` (five cumulative
      `std::time::Duration` fields named as in the schema above) to
      `SimStats`, accumulated in `run_tick` around the five phase calls with
      `Instant`; cumulative, never reset by `reset_tick_counters`. TDD: a
      unit test under `simulation/tick/tests/` asserts each field is
      non-decreasing across two `run_tick` calls and that the tick counters
      reset while these do not. Run `cargo test -p v3-core --test viability`
      first, since `tick.rs` changes.
- [x] Bench extensions in `crates/v3-cli`. Add `rayon` to the crate's
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
- [x] Pay down two T10.F10 maintainability deferrals touched by this work:
      comparison levels become a serde-renamed enum, and the wall-clock flag
      and severe thresholds become named constants beside the work-counter
      ones. The serialized `comparison` and `deterministic` JSON must not
      change; the existing gate tests against the stored references prove it.
- [x] Commit the code, then run the eight sweeps from the repository root,
      the long ones detached and polled. Each command has the
      form below with `N`, `F`, `T`, and the output name substituted:

      ```sh
      make bench PROFILE=sweep OUT=docs/progress/sweeps/t10-f09/wNNNN-tT.json BENCH_ARGS="--width N --height N --founders F --seeds 11,22,33 --ticks 2000 --threads T --feature t10-f09-throughput-baseline-and-profiling-budget"
      ```

      Then check every report's parsed `deterministic` object equals the
      T01.F11 report at the same sweep point (`python3` with `json.load`),
      and record the result. All eight ran, in sequence so no two competed
      for cores, between 19:19 and 19:47 UTC on 2026-09-04 at commit
      `f78486b2`; the equality check ran and its result — four equal, four
      not — is recorded under Verification and in the determinism finding.
- [x] Run the concurrency probe and record the lone and mean concurrent
      `wall_clock_ms_total`, `c`, and the exact loop.
- [x] Profile one one-thread 1600² seed-11 run of 200 ticks: after
      `cargo build --release -p v3-cli`, launch `target/release/v3-cli bench`
      directly in the background so the pid is the benchmark's, then run
      `/usr/bin/sample <pid> 30 -file <path>` while it executes. Keep the raw
      output under `~/.local/share/petri-tools/profiles/t10-f09/` and record
      the path and the ten heaviest frames by sample count, with their share,
      in Performance and Goal Impact.
- [x] Complete Performance and Goal Impact: the per-core budget table, the
      phase-share table, the per-tick cost decomposition (Phase 0
      milliseconds per tick and nanoseconds per cell; the other phases in
      microseconds per creature-tick, both from the one-thread reports), `S`,
      `c`, `T`, and the decision with its numbers.
- [x] Roadmap edits. In the T10 track's T10.F09 paragraph replace the
      sentence carrying the 2026-09-03 rates and the claim that the core
      crate does not use `rayon` with one sentence giving the measured one-
      and eight-thread 1600² ticks per second and stating that Phase 1b
      already runs on the rayon global pool while every other phase is
      sequential, and append one sentence of T10.F04 guidance from the
      decision. If the rule requires parallelism, add the parallelism row
      under the next free ID and the T10.F07 dependency. In `docs/roadmap.md` replace the "about 3 ticks
      per second single-threaded (2026-09-03; T10.F09 owns re-measuring it)"
      sentence with the measured one-thread and eight-thread 1600² figures
      citing `docs/progress/sweeps/t10-f09/`. Leave the T10 track success
      criterion on the standard replicate world unchecked: that world is
      chosen by T01.F12, which cites this budget.
- [x] Generate this feature's gate report with `make bench PROFILE=gate
      FEATURE=t10-f09-throughput-baseline-and-profiling-budget`, append it to
      `closed` in `docs/progress/benchmark-series.json`, and complete the
      gate half of Performance and Goal Impact.

## Verification

Recorded on the recording host (Apple M1 Pro, 8 logical cores, macOS) as each
command ran. Every report under `docs/progress/sweeps/t10-f09/` names
`git_revision` `f78486b2`, the commit that added the phase timers and
`--threads`; the only later change to `crates/` is test code, so the
production code that produced those numbers is the code at the closing
commit.

- [x] `cargo test -p v3-core --test viability` passes before other checks.
      Run first because `tick.rs` changed: 25 passed, 0 failed (1.64 s).
- [x] `cargo test -p v3-core --lib` passes with the phase-timing test:
      1013 passed, 0 failed (18.76 s), including the two new
      `simulation::tick::tests::phase_timing` tests.
- [x] `cargo test -p v3-cli` passes with the threads, throughput proptest,
      and thread-independence tests, and the two stored gate references still
      load and match: 14 + 8 + 14 + 7 + 0 passed, 0 failed (lib unit tests,
      bin unit tests, `tests/bench.rs`, `tests/cli.rs`, doc-tests), counted
      after the mutation-survivor tests were added. The
      `gate_profile_has_no_severe_regression_against_series_references` test
      is the end-to-end proof that the stored T10.F10 and T01.F11 reports
      still deserialize under the extended `environment` schema and the
      `comparison.level` enum.
- [x] `make rust-check` (format, viability, every test subset, Clippy with
      warnings denied) and `make roadmap-check` (`validation passed`) both
      exit 0.
- [x] Every `docs/progress/sweeps/t10-f09/*.json` `deterministic` object was
      compared with its T01.F11 counterpart and the result recorded. Equality
      holds at 128² and 256² only; the item as originally written ("equals")
      is not achievable at this commit, and the user reassigned the fix to
      T10.F11 on 2026-09-04 (see Notes for AI Agents). Checked with `python3`
      (`json.load`, whole-object equality)
      against `docs/progress/sweeps/t01-f11/<point>.json`:
      `w0128-t1` True, `w0128-t8` True, `w0256-t1` True, `w0256-t8` True,
      `w0512-t1` False (seed 11), `w0512-t8` False (seed 11), `w1600-t1`
      False (seeds 11, 22, 33), `w1600-t8` False (seeds 11, 22, 33). The
      simulation is not reproducible across processes on long, large
      trajectories; see the determinism finding in Performance and Goal
      Impact for the evidence that it predates this feature and for the two
      candidate sources.
- [x] `make rust-mutants` summary line, output path, and full survivor list,
      each survivor killed, equivalent, or deferred. Diff against
      `5421b17e`; output in
      `~/.local/share/petri-tools/mutants/t10-f09/mutants.out`.
      First run: `59 mutants tested in 8m: 16 missed, 30 caught, 13
      unviable`. After adding the tests below, second run:
      `59 mutants tested in 5m: 1 missed, 45 caught, 13 unviable`. No
      timeouts in either run, and no production code was changed to kill a
      mutant. Full survivor list from the first run:

      | survivor | resolution |
      | --- | --- |
      | `main.rs:210:5` replace `run_bench` with `()` | killed — `bench_subcommand_writes_a_report_and_rejects_zero_threads` runs the built binary and requires the `--out` report to exist |
      | `bench.rs:387:9` replace `ComparisonLevel::fmt` with `Ok(Default::default())` | killed — `comparison_level_displays_as_its_serialized_string` |
      | `bench.rs:524:28` replace `*` with `/` in `millis` | killed — `millis_converts_a_duration_to_fractional_milliseconds` |
      | `bench.rs:840:29` replace `/` with `%` in `build_environment` | killed — `build_environment_derives_the_per_creature_tick_cost_and_throughput` |
      | `bench.rs:840:29` replace `/` with `*` in `build_environment` | killed — same test |
      | `bench.rs:935:9` delete match arm `None` in `counter_level` | **equivalent** — with the arm deleted, `None` fails both `Some(d) if …` guards and falls to `_ => ComparisonLevel::Ok`, the value the deleted arm returned |
      | `bench.rs:937:20` replace match guard `d > FLAG_PERCENT` with `true` | killed — `counter_levels_are_strict_at_their_thresholds` |
      | `bench.rs:937:20` replace match guard `d > FLAG_PERCENT` with `false` | killed — same test |
      | `bench.rs:936:22` replace `>` with `>=` in `counter_level` | killed — same test, exactly +50 percent stays `flag` |
      | `bench.rs:937:22` replace `>` with `==` in `counter_level` | killed — same test |
      | `bench.rs:937:22` replace `>` with `<` in `counter_level` | killed — same test |
      | `bench.rs:937:22` replace `>` with `>=` in `counter_level` | killed — same test, exactly +10 percent stays `ok` |
      | `bench.rs:1010:30` replace `>` with `>=` in `compare_against` | killed — `wall_clock_levels_are_strict_at_their_thresholds`, exactly +100 percent stays `flag` |
      | `bench.rs:1012:25` replace `>` with `==` in `compare_against` | killed — same test |
      | `bench.rs:1012:25` replace `>` with `<` in `compare_against` | killed — same test |
      | `bench.rs:1012:25` replace `>` with `>=` in `compare_against` | killed — same test, exactly +25 percent stays `ok` |
- [x] Benchmark report stored at
      `docs/progress/features/t10-f09-throughput-baseline-and-profiling-budget.json`,
      generated by `make bench PROFILE=gate
      FEATURE=t10-f09-throughput-baseline-and-profiling-budget`:
      `severe=false` against both references, every counter `level=ok`.
- [x] `make check` passes (exit 0), re-run after the mutation-survivor tests
      were added and the simplification pass was applied.

## Performance and Goal Impact

Predeclared cost: at most ten `Instant::now()` reads and five `Duration`
additions per tick, on the order of a microsecond against ticks that cost
milliseconds.
Expected work-counter delta against both references: exactly 0 percent on
every counter, `plasticity_updates` `null` at level `ok`. Expected wall-clock
delta: under 1 percent. No threshold is expected to be crossed and the epoch
baseline is not re-pinned.

### Gate report (measured)

Stored at
`docs/progress/features/t10-f09-throughput-baseline-and-profiling-budget.json`,
generated 2026-09-04 on the recording host at `f7c4fcf5` (a documentation
commit; production code under `crates/` is identical at `f78486b2`,
`f7c4fcf5`, and the closing commit, as the Verification preamble records) and
appended to `closed` in `docs/progress/benchmark-series.json`. Both reference
comparisons are `severe=false`, and every work counter's per-creature-tick
delta is exactly as predeclared:

| counter (per creature-tick) | current | T10.F10 | T01.F11 | delta | level |
| --- | --- | --- | --- | --- | --- |
| mesh_hops | 1.998362 | 1.998362 | 1.998362 | 0.000000% | ok |
| vm_steps | 28.028231 | 28.028231 | 28.028231 | 0.000000% | ok |
| graph_relax_iters | 2.998624 | 2.998624 | 2.998624 | 0.000000% | ok |
| plasticity_updates | 0.000000 | 0.000000 | 0.000000 | `null` | ok |
| actions_applied | 1.000000 | 1.000000 | 1.000000 | 0.000000% | ok |
| births | 0.001212 | 0.001212 | 0.001212 | 0.000000% | ok |

Wall-clock per creature-tick is 0.004391 ms, `-19.39` percent against the
T10.F10 report and `-4.82` percent against T01.F11's, both level `ok`: the
phase timers cost less than the run-to-run noise. No threshold was crossed
and the epoch baseline is not re-pinned.

### Per-core budget (measured, `--threads 1`)

Totals over the three seeds of each sweep report under
`docs/progress/sweeps/t10-f09/`. The 128² and 256² rows are the ticks of
worlds that go extinct or collapse to a handful of creatures, so their
ticks-per-hour is a per-tick world cost rather than an evolutionary rate; the
512² and 1600² rows are the budget a campaign should plan against.

| sweep point | ticks/s | creature-ticks/s | ticks/hour | births/hour | ms/creature-tick |
| --- | --- | --- | --- | --- | --- |
| 128², 64 founders | 1055.96 | 42,709 | 3,801,443 | 347,647 | 0.023415 |
| 256², 256 founders | 299.48 | 10,596 | 1,078,114 | 82,645 | 0.094375 |
| 512², 1,024 founders | 9.56 | 76,647 | 34,402 | 1,716,791 | 0.013047 |
| 1600², 10,000 founders | 9.53 | 93,456 | 34,300 | 2,744,283 | 0.010700 |

The same grid on all eight logical cores (`--threads 8`):

| sweep point | ticks/s | creature-ticks/s | ticks/hour | births/hour | ms/creature-tick |
| --- | --- | --- | --- | --- | --- |
| 128², 64 founders | 1010.45 | 40,868 | 3,637,631 | 332,666 | 0.024469 |
| 256², 256 founders | 299.51 | 10,597 | 1,078,235 | 82,654 | 0.094364 |
| 512², 1,024 founders | 20.84 | 165,547 | 75,019 | 3,731,824 | 0.006041 |
| 1600², 10,000 founders | 10.54 | 107,118 | 37,930 | 3,193,156 | 0.009335 |

Eight cores buy 10.6 percent at 1600² and a factor of 2.2 at 512²: only
Phase 1b is parallel, and at 1600² the sequential Phase 0 dominates. The
1600² one-thread rate of 9.53 ticks per second supersedes the 2026-09-03
figure of about 3 ticks per second in `docs/roadmap.md`. Against the master
roadmap's scale reference (about 190 steps per second with 60,000 agents,
roughly 11.4 million agent-steps per second on a GPU), this host delivers
93,456 creature-ticks per second on one core and 107,118 on eight: two orders
of magnitude below that reference, and the gap is not closed by core
parallelism on this machine.

### Phase shares of wall-clock

Cumulative `environment.phase_wall_clock_ms_per_seed` summed over the three
seeds, as a share of that report's `wall_clock_ms_total`. `other` is the
untimed remainder: the turn-queue build, the priority sort, seeding, and the
per-seed persistence sampling.

| sweep point | threads | world_update | sensor_assembly | cognition | actions | reward_learning | other |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 128² | 1 | 89.63% | 0.83% | 6.15% | 0.46% | 0.06% | 2.87% |
| 128² | 8 | 86.29% | 0.86% | 9.04% | 0.71% | 0.11% | 2.99% |
| 256² | 1 | 97.54% | 0.23% | 1.47% | 0.12% | 0.02% | 0.62% |
| 256² | 8 | 97.56% | 0.24% | 1.38% | 0.16% | 0.03% | 0.63% |
| 512² | 1 | 11.20% | 10.61% | 74.13% | 2.59% | 0.65% | 0.82% |
| 512² | 8 | 24.14% | 16.72% | 50.25% | 5.72% | 1.52% | 1.66% |
| 1600² | 1 | 55.89% | 8.42% | 30.69% | 3.30% | 0.74% | 0.96% |
| 1600² | 8 | 61.27% | 9.74% | 22.79% | 4.09% | 0.95% | 1.15% |

### Per-tick cost decomposition (one thread)

Phase 0 is a whole-world pass whose cost scales with cells, so it is reported
per tick and per cell; the creature phases are reported per creature-tick.

| sweep point | Phase 0 ms/tick | Phase 0 ns/cell | sensor µs/ct | cognition µs/ct | actions µs/ct | reward µs/ct |
| --- | --- | --- | --- | --- | --- | --- |
| 128² | 0.8488 | 51.81 | 0.194 | 1.440 | 0.108 | 0.014 |
| 256² | 3.2571 | 49.70 | 0.214 | 1.387 | 0.112 | 0.019 |
| 512² | 11.7150 | 44.69 | 1.384 | 9.672 | 0.338 | 0.085 |
| 1600² | 58.6590 | 22.91 | 0.901 | 3.284 | 0.353 | 0.079 |

Phase 0 costs 23 to 52 nanoseconds per cell regardless of world size, so the
1600² world pays roughly 59 milliseconds of sequential world update on every
tick before any creature runs. That single number is why eight cores buy only
10 percent at 1600².

### Sampled hot frames

`/usr/bin/sample` over a 30-second window opened 5 seconds into a one-thread
1600², 10,000-founder, seed-11, 200-tick run (the population climbs toward its
peak across that window). Raw output:
`~/.local/share/petri-tools/profiles/t10-f09/one-thread-1600-seed11.txt`,
kept outside the repository. The worker thread collected 22,260 samples; the
main thread's 22,260 `__psynch_cvwait` samples are the `install` caller
blocked on the pool and are excluded. Shares are of the worker thread's
samples, sorted by top of stack:

| # | frame | samples | share |
| --- | --- | --- | --- |
| 1 | `simulation::tick::run_phase_0` | 6,409 | 28.79% |
| 2 | `runtime::vm::execute_vm_node_with_reserve` | 4,059 | 18.23% |
| 3 | `runtime::cgp::execute::execute_graph_node_with_reserve` | 2,086 | 9.37% |
| 4 | `runtime::cgp::effects::apply_cgp_graph_effects` | 1,515 | 6.81% |
| 5 | `runtime::inputs::resolve_input` | 959 | 4.31% |
| 6 | `sensors::typed_food::assemble_typed_food_local_snapshot` | 939 | 4.22% |
| 7 | `_platform_memmove` (libsystem_platform) | 849 | 3.81% |
| 8 | `kernel::world::WorldState::resolve_neighbor` | 694 | 3.12% |
| 9 | `kernel::ordinary_food::ecology::build_type_inhibition_weighted_sums` | 678 | 3.05% |
| 10 | `_platform_memset` (libsystem_platform) | 504 | 2.26% |

The sampled picture agrees with the phase timers: Phase 0 and the food ecology
helpers it calls are the largest single consumer, and mesh execution (frames 2
to 5) is most of the rest.

### Concurrency probe

Predeclared configuration: 1600², 10,000 founders, seed 11, 200 ticks, one
thread, launched from `target/release/v3-cli` directly so eight processes do
not serialize on the cargo lock. The exact loop, run from the repository root:

```sh
i=1
while [ "$i" -le 8 ]; do
  target/release/v3-cli bench --profile sweep --width 1600 --height 1600 \
    --founders 10000 --seeds 11 --ticks 200 --threads 1 \
    --feature t10-f09-concurrency-probe --out "$OUT/concurrent-$i.json" &
  i=$((i + 1))
done
wait
```

- Lone run: `wall_clock_ms_total` 38,833.8 ms, 5,457,813 creature-ticks, peak
  resident set 1.157 GB (`/usr/bin/time -l`).
- Eight concurrent copies: 351,931.2, 356,724.9, 358,280.6, 359,282.8,
  359,286.0, 360,384.6, 360,429.7, and 360,524.8 ms; mean 358,355.6 ms.
- `c` = 358,355.6 / 38,833.8 = **9.228**. Normalized per creature-tick (the
  copies did not execute identical work; see the determinism finding below)
  `c` = 0.065820 / 0.007115 = **9.251**, the same number.
- Replicate-level throughput is `8 / c` = **0.867** lone-run equivalents:
  eight concurrent one-thread replicates deliver *less* aggregate work than
  one replicate alone. Eight copies at 1.157 GB each need 9.3 GB of the host's
  16 GB, and the M1 Pro has 6 performance plus 2 efficiency cores, so memory
  pressure, memory bandwidth, and the efficiency cores all count against the
  measured `c`; it is a property of this host, not of the code.

### Parallelism decision (predeclared rule)

- `S` = 1600² one-thread `wall_clock_ms_total` / eight-thread
  `wall_clock_ms_total` = 629,672.9 / 569,455.4 = **1.106**. Caveat: the two
  runs did not execute byte-identical work (see the determinism finding), so
  read `S` beside their totals — 58.9 million creature-ticks at one thread
  against 61.0 million at eight; per creature-tick the ratio is
  0.010700 / 0.009335 = 1.146, the same conclusion.
- `T` = the slowest one-thread 1600² seed (seed 11, 227,585.6 ms for 2,000
  ticks) extrapolated linearly to 5,000 ticks = 568,964 ms = **0.158 hours**
  (9 minutes 29 seconds).
- Rule: a core-parallelism feature is required if and only if `T` exceeds
  eight hours. `T` is 0.158 hours, so **core parallelism is not required
  before a campaign** and no parallelism row is added. The T10.F11 ID was
  assigned to reproducibility on 2026-09-04; a parallelism feature, if ever
  required, takes the next free ID.
- For T10.F04: `S` (1.106) is larger than `8 / c` (0.867), so campaigns on
  this host should run replicates **in sequence on all cores**, not one
  replicate per core. The margin is small and the mechanism is clear —
  concurrent replicates contend for memory bandwidth and capacity while the
  eight-thread speedup is capped by the sequential Phase 0 — so a campaign
  that shrinks the world, and with it Phase 0, should re-measure both numbers
  before choosing.

### Determinism finding (P1, pre-existing, not introduced here)

Four of the eight sweep reports do not have a `deterministic` object equal to
their T01.F11 counterpart, and the invariant this feature predeclared does not
hold of the code at all:

- 128² and 256²: equal to T01.F11 at both thread counts, and the one- and
  eight-thread reports are equal to each other.
- 512²: seeds 22 and 33 equal; seed 11 differs from T01.F11 *and* differs
  between this feature's own one- and eight-thread runs (creature-ticks
  19,515,311 at one thread against 19,327,549 at eight; `vm_steps` 42.8
  billion against 63.8 billion).
- 1600²: all three seeds differ from T01.F11 and between the two thread counts
  (seed 11 creature-ticks 22,297,939 at one thread against 18,061,235 at
  eight; final populations 9,007 against 6,924).

Thread count is not the variable. The nine concurrency-probe processes above
all ran `--threads 1` on the same binary and the same inputs and produced nine
different creature-tick totals (lone 5,457,813; copies 5,422,185 to
5,465,094). Running the same 200-tick 1600² seed-11 profile twice from the
code at `5421b17e`, the commit this feature branched from and before any
change here, also produced two different results (creature-ticks 5,452,034
against 5,458,625; `vm_steps` 1.87 billion against 1.20 billion), so the
nondeterminism predates this feature. The phase timers only read `Instant` and
cannot affect simulation state.

Two concrete candidate sources, found by reading and not fixed here (this
feature's Non-Goals forbid touching simulation behavior): both index an
`rng.gen_range` draw into a `Vec` built by iterating a `HashMap`, whose order
varies per process with `RandomState`.

- `crates/v3-core/src/mutation/vm/operators.rs`: `groups.into_iter()` builds
  `eligible`, then `eligible[rng.gen_range(0..eligible.len())]` picks the slot
  group to co-mutate.
- `crates/v3-core/src/mutation/topology/structural.rs`: `id_map.values()`
  builds `new_ids`, then `new_ids[rng.gen_range(0..new_ids.len())]` picks the
  backlink target.

Why the gate profile never sees it: 75 ticks at 128² with 256 founders is too
short for those operators to be drawn and then compound, so the T10.F10
determinism test and the stored gate references stay byte-identical run to
run, as this feature's own gate report shows.

## Success Criteria

- [x] Eight sweep reports at the T01.F11 grid, one and eight threads, are
      committed, each produced by the one command form above, and each was
      compared with the T01.F11 report at that sweep point with the result
      recorded: equal at 128² and 256², different at 512² seed 11 and every
      1600² seed. As originally written this criterion required equality
      everywhere; it was amended on 2026-09-04 at the user's direction
      because the simulation is not reproducible across processes at that
      scale, a pre-existing defect now owned by T10.F11. The wall-clock,
      phase, and throughput figures this feature exists to record do not
      depend on that equality.
- [x] Every bench report records `threads`, per-phase wall-clock, and
      throughput in `environment`, and the per-core budget in ticks and
      births per wall-clock hour at every sweep point is written in this spec
      and cited by the master roadmap.
- [x] The tick profile is recorded: phase shares at one and eight threads for
      every sweep point and the ten heaviest sampled frames.
- [x] The parallelism decision is recorded with `S`, `c`, and `T`, and the
      T10 track carries a parallelism row exactly when the rule requires it:
      `T` = 0.158 hours does not exceed eight hours, so no parallelism row is
      added (the T10.F11 ID the rule originally named now belongs to
      reproducibility).
- [x] Every work counter's per-creature-tick delta in the gate report is 0
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
- Determinism finding disposition (2026-09-04): the orchestrator reproduced
  the divergence independently (two 100-tick 1600² seed-11 runs in separate
  processes: 2,087,946 against 2,087,974 creature-ticks) and put the closure
  path to the user, who chose to treat reproducibility as a core feature.
  The T10 track gained `T10.F11 — Cross-Process Reproducibility of Seeded
  Runs` (depends on T10.F09), T01.F12 now depends on it, and the master
  first-slice order places it directly after this feature, so it is the next
  feature implemented. The eight sweep reports here stay as measured at
  `f78486b2`; T10.F11 regenerates the T01.F11 reports, not these. The T10
  track's determinism-contract paragraph carries the dated caveat.
- Wording deviations recorded by the implementer: `docs/roadmap.md` carries
  one extra sentence flagging the nondeterminism beyond the mandated
  throughput replacement, and the sweep task text says "detached and polled"
  rather than "detached with `nohup`", matching what ran. The simplify pass
  ran inline in one pass (the fan-out variant was unavailable to the
  subagent); it reused the `millis` helper for `wall_clock_ms`, moved
  per-seed throughput into `run_one_seed`, and built `tiny_report()` once in
  the threshold test. `Option<NonZeroUsize>` for `--threads` was considered
  and skipped because the spec predeclares the `0` rejection in
  `resolve_bench_profile` with a unit test.
- **P1, program level, for the orchestrator to own:** the T10.F10 determinism
  contract ("the same commit and the same inputs must produce a byte-identical
  deterministic block") does not hold across processes on long, large
  trajectories. Measured here: nine `--threads 1` processes on identical
  inputs produced nine different counter sets, and the same divergence
  reproduces at `5421b17e`, before this feature's code. Consequences: T01.F11's
  stored 512² and 1600² reports are not regenerable byte for byte and their
  per-seed counts are one draw, not a fixed fact; the same is true of this
  feature's four large reports; the ecological conclusions (persistence,
  peak, trough, final population band) survive because the trajectories stay
  in the same band. The fix is a feature of its own, and it should re-pin
  every affected stored report in the same commit. Candidate sources named in
  Performance and Goal Impact: `rng.gen_range` indexed into a `Vec` built by
  iterating a `HashMap` in `mutation/vm/operators.rs` and
  `mutation/topology/structural.rs`; a fix should also sweep for the same
  pattern elsewhere rather than patching those two sites.
- The 128² and 256² sweep points spend 87 to 98 percent of their wall-clock in
  Phase 0 because their populations collapse: at those points the sweep
  measures the empty-world cost, which is the same thing T01.F11 saw when one
  256² seed held a single creature at about 3.3 ms per tick.
- The concurrency probe's `c` is host-bound: 8 copies × 1.157 GB against 16 GB
  of RAM, on 6 performance plus 2 efficiency cores. Re-measure it, not just
  reuse the number, on any other machine or with a smaller world.
- Deferred review findings (recorded 2026-09-04, advisory, not fixed here):
  (1) `--threads` is parsed as `Option<usize>` and checked for zero by hand
  where clap could take `Option<NonZeroUsize>`; the predeclared unit test pins
  the current error string, so revisiting it means loosening
  `bench_subcommand_writes_a_report_and_rejects_zero_threads`. (2) Adding a
  sixth timed phase touches `PhaseWallClock`, two lines of `run_tick`,
  `SeedPhaseWallClock` and its mapping in `bench.rs`, the `phases()` helper in
  `phase_timing.rs`, and one sum in the thread-independence test; the
  smallest refactor when a phase is added is a `PhaseWallClock` accessor that
  yields the phases in order for the mapping and both tests. (3) The
  thread-independence test in `crates/v3-cli/tests/bench.rs` asserts
  byte-identical `deterministic` blocks for two in-process 32², 30-tick runs
  and shares the T10.F10 determinism test's exposure to the pre-existing
  nondeterminism if those operators are ever reached at that size; T10.F11
  owns it. (4) TDD ordering cannot be confirmed from history because code
  and tests landed in one commit. (5) The scratch reports behind the "code at
  `5421b17e`" reproduction carry `git_revision f78486b2` because the harness
  records the checked-out revision at run time, not the compiled one; the
  orchestrator's own two-process reproduction at `f5fef9a5` is the
  independent confirmation.
- Per-feature cost record (closed 2026-09-04 through the Fable 5.1
  orchestrator, Opus 5 implementer with a Fable advisor, and Fable 5.1
  reviewer): implementer self-reported advisor consults 3 (before the
  approach, on the determinism divergence, before reporting done); reviewer
  findings 0 P1, 1 P2, 6 P3; the P2 (spec passages naming the parallelism
  row `T10.F11` after that ID went to reproducibility) and two document P3s
  were fixed by the orchestrator in the closing commit, with no implementer
  remediation pass. Implementer subagent usage about 307k tokens over one
  run of 225 tool uses and 87 minutes, dominated by the eight sweeps and the
  one-thread probe; reviewer about 130k tokens over 48 tool uses. Session
  `/usage` totals were not collected at closure.
