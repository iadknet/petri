# T21.F06 — Benchmark, Assay and Lab Run Records

**Status**: In Progress
**Last updated**: 2026-09-30
**Feature**: T21.F06
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

A measurement command started with `--telemetry on` — `v3-cli bench`,
`v3-cli recruitment`, `v3-cli input-opportunity`, `v3-lab run` and
`v3-lab why-not` — appears in the stack as one measurement with its identity,
its lifecycle and, at its end, the totals its stored summary carries; each
seeded simulation a benchmark profile runs appears as a run of its own. Every
record is captured at a region boundary and held in the bounded queue T21.F01
delivered, and the exporter takes nothing until the command's final timed
region has finished, so the stored measurements do not move. With telemetry
off, on, or on with no stack listening, the command's deterministic output is
identical.

## Non-Goals

- No record for a simulation an assay or the lab runs inside its loops or
  thread pools: those are hundreds to thousands per invocation, and the
  queue holds 2,048 items. The measurement record is their record.
- No metric snapshot, tick trace or creature window from a measurement
  command; the F02 interval cadence, F03 traces and F04 windows capture
  inside the loop and stay off here whatever the environment says.
- No change to what closure measurement runs do: `make bench` and the closure
  profiles keep the default (off) now and pass `--telemetry off` explicitly
  from T21.F05, as the track's flag note says.
- No records from `v3-cli bench-summarize`, `v3-cli world inspect` or
  `v3-lab report`: they run no simulation.
- No server change; no new dashboard beyond one panel on `petri-runs`.
- No on/off output comparison for the assays: their raw records carry wall
  time and parallel completion order (recruitment streams lineages as they
  finish), so no canonical form is defined for them here; their outputs
  cannot change because their modules are not edited (invariant 7).

## Inputs and Invariants

| Input | Where | What F06 takes from it |
| --- | --- | --- |
| Track row and notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | F06 row; Measurement commands, Telemetry flag, Ownership (T22 owns `crates/v3-lab`, F06 adds only run records), Boundary (`v3-lab`'s library takes no SDK type), Run identity, Failure behavior, Storage. |
| Benchmark gate | `docs/workflow.md`, "Benchmark gate" | Observability exemption and its no-behavior-change condition; check triggers; record format. |
| Research note | `docs/strategy/run-observability-research-2026-09-29.md` | D2 (below); "benchmarks time several regions in one process". |
| F01 spec | Goal, Inputs and Invariants, Performance | Flag and `PETRI_TELEMETRY`; the reference build; the queue bounds (2,048 items, 8 MiB, 4 MiB body, 512 per batch, 5 s request, 10 s flush, no retry); per-run accounting and stderr line; identity attributes; `run.started` and `run.ended`; the check commands; T = 6908. |
| F02 spec | Goal, Inputs and Invariants, Performance | The `petri-runs` dashboard and `scripts/telemetry-dashboards-check`; `Options::from_env` refuses an invalid interval when on; snapshot ≈ 195 KB (readings), why none is taken here. |
| Bench | `crates/v3-cli/src/bench/run.rs` `run_one_seed` (the seed's `start … wall_clock_ms`, then its separately timed observations), `run_deterministic` (founder neighborhood, drift depth, the seed loop, `timed_recruitment_paths` last), `build_report_with_threads`; `main.rs` `run_bench_result` (report written, then `artifacts::convert`, exit 3 on severe); `schema.rs` `PerSeed`, `Totals` | Region boundaries; the seed loop calls `run_tick(&mut sim, &mut None)` with no telemetry hook; the summary's totals. |
| Assays | `crates/v3-cli/src/recruitment.rs` `run`, `RunSummary`; `opportunity.rs` `run`, `RunSummary`; both time `execute` inside `run` and return `Outcome { raw, summary, incomplete, bytes, … }`; exit 3 when incomplete | Totals and the summary paths. |
| Lab | `crates/v3-lab/src/cli.rs` `execute` (prints the report, returns the exit code), `run.rs` `run_in_pool` (`started … wall_seconds`, writes `summary.json`), `summary.rs` `Summary { timing, exit_code, incomplete, provenance }`; `main.rs` resolves the switch and exports nothing | Where the lab's one timed region sits; the plain data the binary needs. |
| Telemetry crate | `crates/v3-telemetry/src/lib.rs` `Telemetry::start`, `start_otlp(options, held)`, `begin_run`, `end_run`, `shutdown`; `queue.rs` `State.held`, `take_batch` (waits while held), `release` (`cfg(test)`); `testing.rs` `Receiver`, `ReceivedRecord` | The held worker exists as a test hook; F06 makes it the measurement mode. |
| CLI test | `crates/v3-cli/tests/telemetry.rs` | Spawn pattern, env scrubbing, canonical comparison; the fixture lives here. |

**D2 (user decision, 2026-09-29, applied by the goal of 2026-09-30).**
Measurement commands record identity, lifecycle and the totals their stored
summary already carries. Closure measurement runs stay off.

Invariants:

1. No `SimulationConfig` or `RuntimeConfig` field, no production RNG draw,
   no simulation default, no stored summary content and no deterministic
   output changes; `v3-core` is not edited. Every call into `v3-telemetry`
   and every capture in `v3-cli` and `v3-lab` sits behind their `telemetry`
   feature. Unconditional and behavior-preserving: `main.rs` command
   functions return exit codes and `run_bench_result` is split so one exit
   point can end the measurement; `run_one_seed` builds `PerSeed` before its
   observation closure and takes the recipe path; `v3-lab`'s library
   returns plain data only (`Measurement`, `Executed`, `execute_with_root`;
   strings, numbers, paths) that the binary maps to records.
2. Held exporter. A measurement command starts telemetry held:
   `Telemetry::start_held` builds the exporter as `start` does, but the
   worker takes no batch until `Telemetry::release`. Exporter activity means
   taking a batch, encoding on the worker and sending a request; none
   happens before release. The worker thread and the HTTP client exist from
   start and idle (parked on the queue's condvar; the client has sent
   nothing). Release happens once, after the command's library entry
   (`bench::build_report_with_threads` plus the report and summary writes in
   `run_bench_result`, `recruitment::run`, `opportunity::run`,
   `v3_lab::cli::execute`) has returned, so after every timed region and
   after the summary is written; then `measurement.ended` is captured, the
   exporter is released and `shutdown` runs the bounded flush (10 s) before
   the process exits with the command's code. `std::process::exit` never
   precedes that flush on a path that captured `measurement.started`; a panic
   is not covered.
3. Capture points are region boundaries only: `measurement.started` before
   the library entry is called; per-seed `run.started` in `run_one_seed`
   before its `Instant::now()`, and `run.ended` after `wall_clock_ms` is
   computed and before the observation timer starts; `measurement.ended`
   after the entry returns. Nothing is captured inside `run_tick` loops, the
   founder-neighborhood, drift-depth, observation or recruitment-paths
   regions, or the assays' and lab's pools. Capture cost sits outside every
   `Instant` interval the stored report carries.
4. Records go through F01's queue, bounds, drop-when-full rule and per-run
   accounting unchanged; a measurement is registered as a run for
   accounting, so it gets its own stderr line at release; in held mode a
   run's `flush_ms` counts from release (or from its end, when later), so a
   seed run that ended early does not report the held period as flush.
   Memory held: about
   3 KB per seed run (the config body, 2,777 B measured, plus two records)
   and about 1 KB per measurement; gate and goal hold 3 seed runs (≈ 10 KB),
   and a sweep of more than 1,023 seeds crosses the 2,048-item bound and
   drops later records, counted in `dropped`.
5. Run identity holds: seed runs draw a run ID at `run.started`; a
   measurement draws one ID that its two records carry as `petri.run_id`,
   and every record shares the process's `petri.invocation_id`, which is how
   seed runs are joined to their measurement. No creature ID, lineage ID or
   genome hash appears anywhere; ticks appear on log records only; no
   metric is exported.
6. The settings read on start are F01's and F02's (`PETRI_TELEMETRY`,
   `OTEL_EXPORTER_OTLP_ENDPOINT`, `PETRI_TELEMETRY_METRICS_INTERVAL_MS`,
   tick-trace and window settings): an invalid value refuses to start, as
   in `run`; a valid one changes nothing here.
7. Telemetry writes nothing to stdout. The deterministic output the
   fixture compares is `bench`'s: the `deterministic` block of the raw
   report and of the summary, keys sorted; the lab's is `summary.json`
   minus its wall-derived `timing` block. For the assays no comparison is
   defined (Non-Goals): F06 edits `recruitment.rs` and `opportunity.rs`
   not at all and `v3-lab`'s library only in what `execute` returns, so
   the reviewer checks by diff that nothing they write can differ with the
   switch.
8. Wall-clock readings inside the stored reports are never promised
   identical between telemetry on and off; they are compared statistically
   under the overhead check on the bench workload (Performance section).

**Records.** All are log records through F01's `emit` path with the resource
attributes of F01. An attribute that does not apply is omitted, never zero.

| Record | Emitted | Attributes and body |
| --- | --- | --- |
| `measurement.started` | after argument resolution and before the library entry, on every command above | `petri.run_id` (the measurement's), `petri.tick` 0 (on both measurement records), `petri.command` (`bench`, `recruitment`, `input-opportunity`, `run`, `why-not`), `petri.feature` (`--feature` where the command has one; the lab has none), `petri.profile` (`gate`, `goal`, `sweep`) or `petri.assay` (the lab's assay name), `petri.pilot` (assays), `petri.seed` (the lab's `--seed`) or `petri.seeds` (bench, comma-joined), `petri.config_digest` (`config_digest` of the one effective config: bench gate and sweep, recruitment's task config; absent for the goal world set, the opportunity worlds and the lab, whose summaries carry their own), `petri.threads` when explicit, `petri.tick` 0; no body |
| `measurement.ended` | after the entry returns, on every exit path after `measurement.started`, including exit 3 and the error exit 1 | `petri.run_id`, `petri.exit_code`, `petri.wall_seconds` (since `measurement.started`), `petri.incomplete` (assays, lab: the summary's field), `petri.severe` (bench), `petri.stop_reason` (assays, when set), `petri.horizon` and `petri.gate_favorable` (input-opportunity), `petri.summary_path` (bench summary; assay summary; the lab's `summary.json`), `petri.raw_sha256` and `petri.raw_bytes` (bench and assays: the summary's raw identity); body: the totals block as compact key-sorted JSON — bench: the eleven `Totals` fields, each the sum over the stored summary's `deterministic.per_seed` rows (the v2 keep-list drops the raw report's `deterministic.totals` block and keeps every `per_seed` row, so the sums are what the summary carries; they equal the raw report's `deterministic.totals`, which the code may read); recruitment `{expected_proposals, lineage_count, proposal_count}`; input-opportunity `{worlds: [{case (the case name), replicates_requested, replicates, ticks, births_total}], verdicts: [{family, world, verdict, sampled, exposed, applied}]}`, where `replicates`, `ticks` and `births_total` are the count and sums of the world's `ReplicateRow`s and each verdict row is the summary's `VerdictRow` cut to those fields (bounded: three worlds, one row per family and world); lab `timing` plus `exit_code` and `incomplete`, each copied from `summary.json` as written (`incomplete` is `null` or a cause string there; the boolean lives in `petri.incomplete`); absent on an error exit |
| `run.started` (F01) | `run_one_seed`, before the seed's timer | F01's attributes, its cadence attributes included although nothing is sampled here; `petri.ticks_requested` = the profile's ticks; `petri.recipe` = the goal case's recipe path on the world set (checked by diff, not by a test: it needs a goal world-set run); body the effective config as in F01 |
| `run.ended` (F01) | `run_one_seed`, after `wall_clock_ms` | `petri.status` `completed`, `petri.tick` where the seed stopped (extinction included), `petri.wall_seconds`, and the seed's `PerSeed` totals as `petri.total.<field>`: `ticks`, `creature_ticks`, `mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`, `pass_cap_hits`, `passes`, `decided_passes`, `final_population`, `extinction_tick` (omitted when none) |

**Dashboard.** `petri-runs` gains one Loki table, `Measurements`, one row
per `measurement.*` record (a started and an ended row share
`petri_run_id`): time, `event_name`, `service_name`, `petri_command`,
`petri_profile` or `petri_assay`, `petri_feature`, `petri_run_id`, and on
the ended row the exit code, wall seconds and `petri_summary_path`; nothing
on it is closure evidence.

**Lab boundary.** `v3_lab::cli::execute` returns the exit code and, for `run`
and `why-not`, a plain `Measurement` value (command, assay, seed, summary
path, timing, exit code, incomplete) built by the library from `RunOutcome`;
`main.rs` holds the only calls into `v3_telemetry`. `report` returns none.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `measurement.started`, `measurement.ended` | log records | the command's invocation, the totals of its stored summary | command, profile or assay, feature, seeds, config digest, exit code, incomplete, summary path and raw identity (table above) |
| `run.started`, `run.ended` per benchmark seed | log records (F01's, extended) | `run_one_seed`; the summary's `PerSeed` totals | F01 identity; ticks requested; recipe; `petri.total.*` |

No signal is sampled; no metric, span or window is emitted. F06 adds no
mechanism switch and no config value; the held mode is not configuration.

## Implementation Tasks

- [x] `v3-telemetry`: `Telemetry::start_held`, `Telemetry::release`;
      `release` leaves `cfg(test)`; `begin_measurement`/`end_measurement`
      (or equivalent) emitting the two records above under per-run
      accounting; `end_run` accepts the seed totals; `testing.rs`
      `ReceivedRecord` gains `time_unix_nano` and `Receiver` the arrival time
      of its first request.
- [x] `v3-cli`: `main.rs` starts held for `bench`, `recruitment` and
      `input-opportunity`, captures the two measurement records, releases
      and flushes on every exit path, then exits with the command's code;
      the flag's doc no longer says only `run` exports; `bench/run.rs`
      captures the seed records at the two boundaries through the
      `telemetry` module's installed handle, behind the feature.
- [x] `v3-lab`: `cli::execute` returns the plain `Measurement`; `main.rs`
      starts held, captures, releases, flushes, exits.
- [x] Fixture and the assay and exit-path rows in
      `crates/v3-cli/tests/telemetry.rs`; the lab command row in
      `crates/v3-lab/tests/lab_run.rs` (Verification table).
- [x] `telemetry/grafana/dashboards/petri-runs.json`: the `Measurements`
      panel; `scripts/telemetry-dashboards-check` runs the tiny sweep with
      telemetry on and asserts the panel query returns its row.
- [x] `scripts/telemetry-overhead`: `PETRI_OVERHEAD_WORKLOAD=bench` runs the
      bench workload of the Performance section instead of `run`, sums the
      `self_time_us` of the invocation's stderr lines, and reads the raw
      report's `environment.wall_clock_ms_total` beside the process time.
- [ ] Readings file: bytes per measurement, the checks below.

## Verification

- [x] Fixture and neutrality: `cargo test -p v3-cli` (inside `make check`,
      no Docker) -> the CLI rows below: the three `measurement::` tests
      (a `#[path]` submodule of `telemetry.rs`) pass (2026-09-30).
- [x] Held exporter and records: `cargo test -p v3-telemetry` -> the crate
      rows: 59 passed (2026-09-30).
- [x] Lab: `cargo test -p v3-lab` -> the lab row: both lab tests pass
      (2026-09-30).
- [x] Reference build: `cargo build --release -p v3-cli -p v3-server -p
      v3-lab --no-default-features` compiles; `cargo tree … -e features`
      shows neither `v3-telemetry` nor `telemetry-seams`; clippy
      `-D warnings` passes with and without the feature (2026-09-30).
- [ ] Dashboards (Docker, outside `make check`):
      `scripts/telemetry-dashboards-check` -> every row passes, the
      `Measurements` row included.
- [ ] Bytes: the tiny sweep's and one gate run's `bytes=` lines (measurement
      and seed runs) in the readings file.
- [ ] Parent comparison: `scripts/telemetry-parent-compare 1c594616` ->
      identical canonical output, in the readings file.
- [ ] Overhead check: `PETRI_OVERHEAD_WORKLOAD=bench scripts/telemetry-overhead`
      -> the table in the readings file, verdict in Performance and Goal
      Impact.
- [x] `make check` -> exit 0 (2026-09-30).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: `Not applicable:
      observability feature`.
- [ ] Benchmark summary: `Not applicable: observability feature`.

| Test rows | What is asserted |
| --- | --- |
| CLI: fixture workload | `v3-cli --telemetry <switch> bench --profile sweep --width 32 --height 32 --founders 16 --seeds 11,22 --ticks 20 --feature t21-f06-fixture --out <tmp> --summary-out <tmp>`, once off, once on against the in-test receiver with `PETRI_TELEMETRY_METRICS_INTERVAL_MS=10`, tick traces and windows on, and once on against a closed port |
| CLI: deterministic fields | the raw report's and the summary's `deterministic` blocks (keys sorted) are byte-identical across the three runs; stdout is identical; the two `on` runs exit 0 |
| CLI: boundaries only | the receiver holds exactly six records: `measurement.started`, `run.started` and `run.ended` for seeds 11 and 22, `measurement.ended`, in that order of timestamps; no snapshot, trace or window arrived and both stderr run lines say `snapshots=0`; for each seed `run.ended` − `run.started` ≥ that seed's `wall_clock_ms` in the raw report; `measurement.started` ≤ the first `run.started` and `measurement.ended` ≥ the last `run.ended`; `run.ended` carries every `petri.total.*` of the table with the raw report's `per_seed` values; `measurement.ended` carries `petri.exit_code` 0, `petri.summary_path` equal to `--summary-out`, `petri.raw_sha256` and `petri.raw_bytes` equal to the summary's raw identity, and a body equal to the `Totals` fields summed over the written summary's `deterministic.per_seed` rows (and to the raw report's `deterministic.totals`) |
| CLI: exporter after the last region | the receiver's first request arrived (its own clock, `SystemTime` at accept) at or after the `measurement.ended` record's timestamp, which was set after the summary was written; the same holds with the tight interval and traces on |
| CLI: exit paths | a sweep whose `--out` names an existing directory (argument resolution accepts it; `write_json` fails after the report is built, whereas a parent that is a file fails in `output_paths` before `measurement.started` and writes no record) exits 1 with `measurement.started` and a `measurement.ended` carrying `petri.exit_code` 1 and no body; the fixture sweep run again with `--compare` naming a copy of its own raw report whose positive `per_creature_tick` work counters are divided by ten (a severe level is a positive delta of the current run over the reference) exits 3 with `petri.exit_code` 3, `petri.severe` true and the totals body; an invalid `PETRI_TELEMETRY_METRICS_INTERVAL_MS` with `--telemetry on` refuses to start before any record |
| CLI: assays | `v3-cli --telemetry on recruitment --feature t21-f06-fixture --pilot --threads 1 --wall-cap-secs 0 --out <tmp> --summary-out <tmp>` in a scratch git checkout (the arguments of `recruitment_cli_passes_every_option_into_the_written_summary`) exits 3 with exactly `measurement.started` and `measurement.ended`, `petri.command` `recruitment`, `petri.exit_code` 3, `petri.incomplete` true, `petri.stop_reason` set, `petri.summary_path` and the raw identity equal to the written summary's, the body equal to the summary's three counts, and the first request at or after `measurement.ended`; `v3-cli --telemetry on input-opportunity --feature t21-f06-fixture --pilot --out <unwritable>` exits 1 with `petri.exit_code` 1 and no body, having run no replicate; each command's stderr shows one `telemetry: run=…` line for the measurement, printed after the command's own output |
| Lab: command | `v3-lab --telemetry on run` with `lab_run.rs`'s `tiny` sizes, the crate directory as cwd and `--out` naming a fresh directory under the checkout's `.bench-artifacts/lab/` (the only place `resolve_out` admits; removed by the test), against the receiver: exit code as without telemetry; exactly `measurement.started` and `measurement.ended` with `petri.command` `run`, `petri.assay`, `petri.seed`, `petri.exit_code` equal to the process exit, `petri.summary_path` naming the written `summary.json` and a body equal to that file's `timing`, `exit_code` and `incomplete`; the first request at or after `measurement.ended`; `summary.json` byte-identical to the same run with `--telemetry off` once `timing` is removed from both |
| Crate: held | a held telemetry with records queued and an accepting receiver: the receiver has zero requests until `release`, then all of them; `shutdown` after release flushes within the bound; per-run lines print once per run and once for the measurement |
| Crate: records | `measurement.started`/`ended` encode the attributes of the table with absent ones omitted; `run.ended` with totals encodes `petri.total.*`; `ReceivedRecord.time_unix_nano` decodes |
| Lab | `execute` on a `run` and on a `why-not` (through a root-taking variant the test calls with `LabRoot::without_git`, as the library tests call `run`; the `tiny` sizes) returns a `Measurement` whose `command` names the subcommand and whose fields equal the written `summary.json` (`timing`, `exit_code`, `incomplete`, path); `report` returns none; the library's only `v3_telemetry` item stays `Switch` (F01's flag type), no `Telemetry`, handle or SDK type, checked by grep in the review |

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary content; `v3-core` is not edited; every call into
`v3-telemetry` and every capture sits behind the `telemetry` feature, and the
unconditional restructuring invariant 1 names changes no output; the reviewer
checks that and the capture placement of invariant 3 against the diff. Measurement
feature: the natural-analog rule and the environmental-pressure rule do not
apply, and no indicator can move.

Parent comparison: not triggered by its rule (no `v3-core` edit, nothing
inside a tick loop or timed region) but run because it costs seconds and the
diff touches `v3-cli`; `BASE` is `1c594616`, T = 6908. Its workload is
`v3-cli run`, which F06 does not touch, so its expected result is identical
canonical output and it says nothing about `bench`; the fixture's off/on
identity and the existing `gate_profile_deterministic_block_is_byte_identical_across_two_runs`
cover the bench path.

Overhead check: runs because F06 adds work to a measurement invocation (two
records per measurement, two per benchmark seed, a held worker and an idle
client), and the track compares the stored wall-clock readings
statistically. `v3-cli run` is untouched, so the check's workload is the
bench path, with F01's method otherwise unchanged and T = 6908 fixed:

| Item | Predeclared |
| --- | --- |
| Workload | `v3-cli bench --profile sweep --config telemetry/overhead-world.json --seeds 7 --ticks 6908 --out <scratch> --summary-out <scratch>`: the same world and tick count as the `run` workload, one seed, no observation region; the telemetry side adds `--telemetry on` |
| Reference, spread, pairs, states, verdict, rerun, time cap, inconclusive | as the F01 spec's table: `--no-default-features` reference run with the stack stopped; n = 5 at spread ≤ 5%, 8 at ≤ 10%, else inconclusive; states (a) on, stack healthy, (b) on, stack stopped, (c) idle-stack block; pass at median ≤ 1.25 with ⌈0.75 n⌉ pairs ≤ 1.25; one rerun; 300 s cap; inconclusive is reported, never rounded |
| Reading | the process wall time of the invocation (the quantity the ceiling governs, flush included); beside it, per pair, the ratio of the raw reports' `environment.wall_clock_ms_total` — the stored measurement the F06 row protects — with its median recorded and no ceiling of its own |
| Self time | the sum of `self_time_us` over the invocation's stderr lines, per tick of the seed run |
| Expected | process ratio within noise of 1.0 in (a) and (b): about 4 KB of records and one request per state; stored-reading median ratio within the pair spread of 1.0, since every capture sits outside the seed's `Instant` interval |

**Measured verdict.**

| Record | Value |
| --- | --- |
| Profiles | `Not applicable: observability feature` |
| Parent comparison | pending |
| Overhead check | pending |

- Readings: [`docs/progress/readings/t21-f06.md`](../../progress/readings/t21-f06.md).

## Success Criteria

- [ ] A `v3-cli bench --profile gate --telemetry on` run against the local
      stack shows on `petri-runs` as one row of `Measurements` and three seed
      runs in `Runs started` and `Runs ended` under the same invocation ID;
      the measurement's `petri_summary_path` names the written summary and
      its body equals the `Totals` fields summed over that summary's
      `per_seed` rows; the same command with
      `--telemetry off` writes a byte-identical `deterministic` block.
- [ ] The receiver's first request in the fixture arrives after the
      `measurement.ended` timestamp, and no interval snapshot, trace or
      window is captured however the environment is set.
- [ ] `v3-lab run --quick --telemetry on` and one assay pilot
      (`v3-cli input-opportunity --pilot` or `recruitment --pilot`, the
      shorter, named in the readings file) each show one measurement whose
      body is the totals of its own summary; the other assay is covered by
      its record test.
- [ ] `make check` exits 0 with the fixture inside it; the reference build
      compiles without `v3-telemetry`.

## Notes for AI Agents

- Decision: D2 (user, 2026-09-29, applied 2026-09-30): measurement commands
  record identity, lifecycle and the totals their stored summary already
  carries; closure measurement runs stay off.
- Exception: plan committed after round 2 `not-ready` (user, 2026-09-30).
  Round 3 failed on the Codex usage limit; the three round-2 corrections
  are unconfirmed; details in the readings file.
