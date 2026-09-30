# T21.F01 — Local Telemetry Stack and Run Identity

**Status**: Complete
**Last updated**: 2026-09-29
**Feature**: T21.F01
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

`make telemetry-up` starts one local stack from a Compose file that keeps
its data until the user retires it with `make telemetry-clean`. A run started
with `--telemetry on` in `v3-cli run` or `v3-server` appears in that stack as
log records carrying its invocation ID, run ID, seed, build revision, world,
full effective config and lifecycle status; a server reset starts a new run.
With telemetry off, on, or on with no stack listening, the run's
deterministic output is identical, and a stopped stack costs a run at most a
bounded flush at its end. F01 also delivers the telemetry-neutrality test,
the parent comparison and the overhead check.

## Non-Goals

- Exported counters, gauges, tick gauges, dashboards and Grafana provisioning
  (T21.F02); tick, phase and creature traces (T21.F03, T21.F04); run records
  for `bench`, `recruitment`, `input-opportunity` and `v3-lab` (T21.F06):
  at F01 those commands accept the flag and export nothing; the default flip
  and presets (T21.F05); template conventions (T21.F07).
- Any change to `v3-core` source: F01 reads seed, tick, status and config
  from the binaries and adds no observation seam; it only declares the empty
  `telemetry-seams` feature in `crates/v3-core/Cargo.toml` that later
  seams sit behind. `crates/v3-core/src/bin/profile_ticks.rs`,
  a harness compiled inside `v3-core` (telemetry-free under the track's
  Boundary note), takes no flag.
- Retiring one run by ID; the cleanup command takes a cutoff date only.
- Retries inside the exporter, disk buffering, and any frontend change.

## Inputs and Invariants

| Input | Where | What F01 takes from it |
| --- | --- | --- |
| Track row and notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | Stack layout with three verifications and the separate-services fallback; storage, cleanup, flag, run identity, transport, failure behavior, reference build, the three check commands, the 25% interim ceiling and the idle-stack reading. |
| Benchmark gate | `docs/workflow.md`, "Benchmark gate" | Observability exemption; check triggers; interleaved pairs, spread-chosen pair count, inconclusive rule, 5 minutes of measured time, `scripts/bench-wait`; record format. |
| Research note | `docs/strategy/run-observability-research-2026-09-29.md` | D1, D3, D4; SDK status and the blocking HTTP client; store retention and deletion facts. |
| CLI run | `crates/v3-cli/src/lib.rs` `run_simulation`, `crates/v3-cli/src/main.rs` `Cli`, `RunArgs` | NDJSON on stdout is the deterministic output; `TickSampleEvent` holds four `HashMap` fields whose key order varies per process, so byte comparison needs the canonical form below; `run_started` already carries `config_digest`; seed, ticks and `--config` are in `RunArgs`. |
| Server lifecycle | `crates/v3-server/src/app_state.rs` (`AppState::new` seeds a run at seed 0), `http/lifecycle.rs` (`startup`, `start`, `pause_sim`, `run_loop`), `state.rs` (`SimHandle.status`), `http/status.rs` (`patch_config`, `perf` timings in the status payload), `bin/server.rs` | `startup` re-seeds and is the reset; `start` can run the initial simulation without a `startup`; status moves Idle, Running, Paused; a config patch produces a new digest; the binary parses no arguments today and reads `V3_SERVER_BIND_ADDR`. |
| Lab binary | `crates/v3-lab/src/main.rs`, `cli.rs` | Gets the flag; exports nothing until T21.F06. |
| Config identity | `crates/v3-core/src/config/recipe.rs` `config_digest` | The digest every record carries; unchanged by F01. |
| Build revision | no source today | F01 adds it (table below). |
| Shell lint scope | `scripts/quality-check` | ShellCheck covers `scripts/*` only, so every F01 script lives there. |
| Host | Docker Desktop 29.6.2, Compose v5.3.1, 8 CPUs, 7.75 GiB VM memory (checked 2026-09-29) | Docker is available; port 3000 is `v3-server`'s, so Grafana is mapped elsewhere. |
| Image | `grafana/otel-lgtm:0.34.0` (Docker Hub, pushed 2026-09-25; previous tag 0.33.1) | Data under `/data`; `PROMETHEUS_EXTRA_ARGS`, `LOKI_EXTRA_ARGS`, `TEMPO_EXTRA_ARGS`; config mounts at `/otel-lgtm/{loki,tempo}-config.yaml`; container ports 3000, 4317, 4318; the collector forwards logs to Loki at `/otlp`, metrics to Prometheus, traces to Tempo. |
| Rust SDK | `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` 0.33.0 (crates.io, 2026-09-29) | `http-proto` + `reqwest-blocking-client` + `logs`; no tokio requirement. |

Invariants (options weighed are in the readings file):

1. No `SimulationConfig` or `RuntimeConfig` field, no production RNG draw,
   no simulation default and no stored summary content changes. The config
   digest and the pinned recipe digests do not move.
2. Telemetry writes nothing to stdout. The deterministic output every check
   compares is the CLI's NDJSON in canonical form: each line parsed and
   re-serialized with keys sorted and values untouched (`serde_json::Value`
   in tests, `node` through Aqua in scripts). Production output does not
   change. For the server it is the simulation data of the status, snapshot
   and creature payloads at matching ticks under `step`; the `perf` timings
   and the WebSocket frame cadence are wall-time driven and excluded.
3. IDs come from OS entropy (`rand::rngs::OsRng`), never from a simulation
   RNG.
4. Bounds: a queue of at most 2,048 records and 8 MiB of queued payload
   that drops when either is full; no record is ever truncated: the stack
   raises Loki's line limit to 4 MiB (`-distributor.max-line-size=4MB`),
   the crate's body cap is the same 4 MiB, a body over it is dropped whole,
   counted in `dropped` and named on stderr as a gap, and the verification
   measures the workload's config body (2,777 B measured) against that
   cap; one export batch of at most 512 records of one run in flight; request timeout 5 s; no retry; the flush that starts
   at a run's end (CLI) or at process shutdown (server) finishes within 10 s
   and abandons what remains. Counters are kept per run and released when
   that run's summary line prints.
5. Reference build: `v3-cli`, `v3-server` and `v3-lab` gain a cargo feature
   `telemetry`, on by default, that gates the optional `v3-telemetry`
   dependency, the flag, every call into it, and forwards
   `v3-core/telemetry-seams`, an empty feature F01 declares in `v3-core`
   for the observation seams later features add there. `v3-core` has no
   default feature (only `dhat-heap`) and none of the three binaries has
   another default feature, so `--no-default-features` on the binaries is
   the reference build and compiles out exactly T21's work, now and after
   later features.
6. One global flag per binary: `v3-cli --telemetry on|off <subcommand>`,
   `v3-server --telemetry on|off`, `v3-lab --telemetry on|off <subcommand>`.
   The flag beats `PETRI_TELEMETRY=on|off`, which beats the default `off`.
   The endpoint is `OTEL_EXPORTER_OTLP_ENDPOINT`, default
   `http://127.0.0.1:4318`.
7. Nothing the stack stores enters the repository; `telemetry/` holds source
   only, and the data volume is named, not bind-mounted into the tree.
8. Every F01 script is POSIX `sh` under `scripts/`, so `make quality-check`
   lints it.

**Stack.** `telemetry/compose.yaml` and its mounted files:

| Setting | Value |
| --- | --- |
| Service | `lgtm`, image `${PETRI_LGTM_IMAGE:-grafana/otel-lgtm:0.34.0}`, `deploy.resources.limits` cpus `2`, memory `2g` |
| Ports | `127.0.0.1:3300` to Grafana 3000; `127.0.0.1:4317` and `127.0.0.1:4318` to the collector |
| Volume | named volume `${PETRI_TELEMETRY_VOLUME:-petri-telemetry}` at `/data`; with `PETRI_LGTM_IMAGE` and `PETRI_TELEMETRY_PROJECT` (Compose project name, default `petri-telemetry`) these are Compose overrides the compose file's `name:` and volume read; `scripts/telemetry-verify` and `scripts/telemetry-overhead` set them to scratch projects and volumes, and `make telemetry-up` uses the defaults |
| Prometheus | `PROMETHEUS_EXTRA_ARGS=--storage.tsdb.retention.time=100y --storage.tsdb.retention.size=0 --web.enable-admin-api` |
| Tempo | mounted `telemetry/tempo-config.yaml`: the image's file for 0.34.0 (extracted from the image; Tempo v3.0.3 on both pinned tags, no `compactor` section) with `block_retention: 876000h` under both `backend_scheduler.provider.compaction.compaction` and `backend_worker.compaction`; `/status/config` must show both; `live_store` block settings and `compaction_window` (1h) stay at the image's defaults: the backend worker merges level-0 blocks inside one window within seconds, so a cutoff retires whole compacted blocks of up to one window, the whole-block granularity the track accepts, and the cleanup preview prints the window in force |
| Loki | `LOKI_EXTRA_ARGS=-compactor.retention-enabled=true -compactor.delete-request-store=filesystem -compactor.delete-request-cancel-period=1m -compactor.retention-delete-delay=1m -compactor.compaction-interval=5m -distributor.max-line-size=4MB -store.max-query-length=0`; `retention_period` stays at its default `0s`, so nothing expires by age |
| Grafana | defaults of the image; login `admin`/`admin` |

`make telemetry-up` runs `docker compose up -d` and then prints each store's
disk use (`du -s` of `/data/prometheus`, `/data/tempo`, `/data/loki` inside
the container); `make telemetry-down` stops the stack and keeps the volume.

**Cleanup.** `scripts/telemetry-cleanup CUTOFF [--proceed]`, behind
`make telemetry-clean CUTOFF=<cutoff> [PROCEED=1]`, uses only the pinned
image. `CUTOFF`
is a date `YYYY-MM-DD`, read as 00:00 UTC, or an RFC3339 timestamp; it
must be in the past or the script refuses. Without `--proceed` it prints
each store's disk use, the deletion interval (everything before the cutoff)
and the exact targets, then exits without deleting.

| Store | Preview | Deletion |
| --- | --- | --- |
| Prometheus | the series label sets returned by `GET /api/v1/series?match[]={__name__=~".+"}&end=<cutoff>`, with their count | `POST /api/v1/admin/tsdb/delete_series?match[]={__name__=~".+"}&end=<cutoff>`, then `POST /api/v1/admin/tsdb/clean_tombstones` |
| Loki | the streams returned by `GET /loki/api/v1/series?match[]={service_name=~".+"}&start=2000-01-01T00:00:00Z&end=<cutoff>` (the query-length limit is disabled in the stack), with their count | `POST /loki/api/v1/delete` with `query={service_name=~".+"}`, `start=2000-01-01T00:00:00Z`, `end=<cutoff>`; the script prints the request ID and the status endpoint |
| Tempo | block IDs under `/data/tempo/blocks/*/*/meta.json` whose `endTime` is before the cutoff, each with its start, end and byte size, plus the IDs of straddling blocks it keeps whole | the script stops the stack, removes the listed block directories through `docker compose run --rm` on the same image, and starts the stack again |

**Run identity and records.** `crates/v3-telemetry` owns the SDK, the
identity, the records and the exporter; the binaries call it.

| Attribute | Level | Value |
| --- | --- | --- |
| `service.name` | resource | `v3-cli`, `v3-server` or `v3-lab` |
| `service.version` | resource | `CARGO_PKG_VERSION` |
| `process.pid` | resource | the process ID |
| `petri.invocation_id` | resource | 128 bits of OS entropy as 32 lowercase hex, once per process |
| `petri.build_revision` | resource | 40-hex commit from the crate's `build.rs` (`git rev-parse HEAD`), `-dirty` appended when tracked files differ when the crate builds, `unknown` when no repository is visible; `PETRI_BUILD_REVISION` overrides it; `rerun-if-changed` on the absolute paths `git rev-parse --path-format=absolute --git-path` gives for `HEAD` and the checked-out ref (or `packed-refs`), in an ordinary checkout or a linked worktree, so a dirty flag can be stale between rebuilds |
| `petri.run_id` | record | 32 lowercase hex, drawn at each `seed_simulation` |
| `petri.seed` | record | the run seed |
| `petri.world` | record | `<width>x<height>` of the effective config |
| `petri.recipe` | record | the `--config` path when one was given; absent otherwise |
| `petri.config_digest` | record | `config_digest(&config)` |
| `petri.tick` | record | the tick at capture |
| `event.name` | record | the record kind (`run.started` and so on), because Loki 3.7 drops the OTLP `event_name` field |

| Record | Emitted | Attributes and body |
| --- | --- | --- |
| `run.started` | after `seed_simulation`, before the first tick: CLI `run_simulation`; server `AppState` construction (the initial seed-0 run) and every `startup` | identity above; CLI adds `petri.ticks_requested` and `petri.sample_every`; body is the full effective `SimulationConfig` as compact JSON, the same document `--save-config` writes |
| `run.state` | each server status transition (`idle`, `running`, `paused`) | `petri.state`, `petri.tick` |
| `run.config` | a server config patch is accepted | the new digest; body the new full config |
| `run.ended` | CLI after `run_completed`; server at the next `startup` (`reset`) and at SIGINT/SIGTERM (`shutdown`: the run loop stops and the simulation lock is held through the flush, so `petri.tick` is where it stopped; that stop emits no `run.state`) | `petri.status` (`completed`, `reset`, `shutdown`), `petri.tick`, `petri.wall_seconds` |

An attribute that is not known is omitted, never written as zero. Loki keeps
resource attributes such as `service.name` as labels and the rest as
structured metadata, so at F01 a run is found in Grafana Explore or the Loki
API with `{service_name="v3-cli"} | petri_run_id="<id>"`.

**Failure behavior and self-report.** Export is best effort over OTLP/HTTP
with the blocking client. When telemetry is on, the binary prints one stderr
line at start
(`telemetry: on endpoint=<url> invocation=<id> run=<id>`) and one per run
(`telemetry: run=<id> exported=<n> failed=<n> dropped=<n> abandoned=<n> bytes=<n> self_time_us=<n> flush_ms=<n>`).
Every record is attributed to its run when enqueued; the run's line prints
once the run has ended and each of its records has been exported, failed,
dropped by the full queue, or abandoned by the shutdown flush, so a server
reset never blocks on the previous run's flush. `exported` is records the
receiver accepted; a partial success's rejects are `failed`, and as it
reports only a count, `bytes` (body plus attribute bytes) covers fully
accepted batches only. Every count is exact. `self_time_us` is the time
inside telemetry calls on the simulation thread and `flush_ms` the time the
end-of-run or shutdown flush took.

**Check commands.** All three are POSIX `sh` under `scripts/` and wrap every
timed run in `scripts/bench-wait`.

| Command | What it does |
| --- | --- |
| `scripts/telemetry-parent-compare BASE` | `BASE` is mandatory and has no default, because in this single-worktree track the parent of a feature after F01 is the previous feature's closing commit, not the fork point; each T21 spec names the commit it compared against. Adds a scratch worktree at `BASE` outside the calling checkout, builds both `v3-cli` release binaries into separate target directories, runs the workload below on both with `--telemetry off`, omitting the flag only when the parent's `run --help` does not list it (a pre-F01 parent), compares the canonical form of both outputs, removes the worktree, and exits non-zero on a difference. |
| `scripts/telemetry-overhead` | Builds the telemetry build and the reference build (`--no-default-features`, separate target directory); calibrates `T` when the readings file has none: starting at 1,000 ticks it rescales `T` towards a 4 s run across three reference runs and refuses with exit 1, running no pair, when the third run falls outside 3 to 5 s; every calibration run, including a voided one, is charged to the cap; measures the spread; runs the pairs of each state below, stopping and starting the stack (since T21.F02 its own scratch project and volume `petri-telemetry-overhead`, so `make telemetry-up` is not a prerequisite) with `docker compose stop` and `start` and waiting for Grafana `/api/health` plus 5 s before a timed run that follows a start; prints one table with the per-state median ratio, the verdict, the pair count, the self-timed cost per tick from the stderr line, and the measured time used. It refuses a run that would pass 300 s of measured time and reports `inconclusive`. |
| `scripts/telemetry-verify` | Runs the three stack verifications and the cleanup rehearsal under Compose project `petri-telemetry-verify` on the scratch volume `petri-telemetry-verify`, never on `petri-telemetry`; refuses to start while the `petri-telemetry` project is up (same host ports); layers `telemetry/verify/compose.yaml` with a second `-f`, which mounts `telemetry/verify/tempo-config.yaml`, the production Tempo file with `compaction_window: 1m` under `backend_scheduler.provider.compaction.compaction`, so the backend worker merges level-0 blocks only inside the same minute (the production 1 h window would merge the whole rehearsal into one straddling block); takes an RFC3339 cutoff during the run: the "before" samples in one minute, the straddling pair in a later minute with the cutoff taken between its halves, the "after" samples in a minute after that, then waits for compaction to settle; the Tempo check asserts the contract itself: the preview names exactly the whole blocks whose `endTime` is before the cutoff (a block carrying `meta.compacted.json` is never a target; Tempo retires it), the straddling block is kept whole and reported with the window in force, and after cleanup the "before" trace is gone while the straddling and "after" traces still answer, where a trace counts as found only when Tempo's `/api/v2/traces/<id>` returns spans (it answers an unknown ID with 200 and an empty trace); Loki and Prometheus get before/after pairs the same way; prints one pass/fail line per check and removes the scratch volume at the end. |

Workload for both checks: `v3-cli run --seed 7 --ticks T --sample-every T
--config telemetry/overhead-world.json`, the gate profile's world (128x128,
256 founders, food coverage 1.0); `T` is calibrated once with three
reference runs to a 3 to 5 s run, recorded in the readings file and fixed
for later closures.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `run.started` | log record | `seed_simulation` in `run_simulation`, server construction and the `startup` handler | resource identity, seed, world, recipe, digest, ticks requested, full config body |
| `run.state` | log record | server status transitions | state, tick |
| `run.config` | log record | accepted server config patch | new digest, full config body |
| `run.ended` | log record | CLI completion, server reset and shutdown | status, tick, wall seconds |
| exporter self-report | stderr line | the telemetry crate's own counters | exported, failed, dropped, abandoned, bytes, self time, flush time |

No counter, gauge or span is exported at F01. The records consume no
production RNG and change no execution; the flag is process-level and enters
no recorded configuration.

## Implementation Tasks

- [x] `telemetry/compose.yaml`, `telemetry/tempo-config.yaml`,
      `telemetry/verify/{compose,tempo-config}.yaml`, `telemetry/overhead-world.json`;
      `make telemetry-up`, `telemetry-down`, `telemetry-clean`.
- [x] `crates/v3-telemetry`: identity, `build.rs` revision, the four
      records, bounded exporter with exact per-run counts, stderr
      self-report, unit tests for saturation and the flush bound.
- [x] `v3-cli`: global `--telemetry`, feature `telemetry`, hooks in
      `run_simulation`; `v3-server`: `--telemetry`, feature, hooks in
      `AppState` construction, `startup`, `start`, `pause_sim`,
      `patch_config`, signal shutdown; `v3-lab`: flag and feature, inert.
- [x] Telemetry-neutrality test in `cargo test -p v3-cli` with an in-test
      OTLP receiver on an ephemeral port, and server tests for run identity
      across `startup`, three-state neutrality under `step`, and shutdown
      with pending records.
- [x] `scripts/telemetry-cleanup`, `scripts/telemetry-verify`,
      `scripts/telemetry-parent-compare`, `scripts/telemetry-overhead`.
- [x] Run the verifications and both checks on this host; record them in
      `docs/progress/readings/t21-f01.md`.

## Verification

- [x] Telemetry-neutrality test: `cargo test -p v3-cli` (inside `make check`,
      no Docker) -> one seeded run's canonical NDJSON is identical with
      telemetry off, on with an in-test OTLP receiver that saw `run.started`
      and `run.ended` for the run ID, and on with a closed port.
- [x] Bounds: `cargo test -p v3-telemetry` -> a queue driven past 2,048
      records or 8 MiB reports the exact dropped count; a receiver that
      accepts the connection and never answers leaves the CLI run's flush
      within 10 s with `abandoned` equal to the records still pending; a
      body above 4 MiB is dropped whole, counted and named on stderr; the
      workload run's config body size is measured and recorded.
- [x] Flag precedence: parser tests in `v3-cli`, `v3-server` and `v3-lab`
      -> nothing set is off; `PETRI_TELEMETRY=on` is on; `--telemetry off`
      beats `PETRI_TELEMETRY=on`; `--telemetry on` is on.
- [x] Server tests: `cargo test -p v3-server` -> the initial run and two
      `startup` requests yield three distinct run IDs, `run.ended` with
      `reset` for each replaced run, `run.state` per transition, `run.config`
      after a patch, and shutdown with pending records completes within
      10 s with exact counts; and the server neutrality test: one seeded
      `startup` followed by N `step` calls yields identical canonical
      status, snapshot and creature payloads (minus `perf`) with telemetry
      off, on with the in-test receiver, and on with a closed port.
- [x] Reference build: `cargo build --release -p v3-cli -p v3-server -p v3-lab
      --no-default-features` compiles, `cargo tree -e features` on that build
      shows neither `v3-telemetry` nor `v3-core/telemetry-seams`, and
      `cargo clippy --workspace --all-targets -- -D warnings` passes with and
      without the feature.
- [x] Stack verifications (Docker, outside `make check`):
      `scripts/telemetry-verify` -> retention settings read back from
      Prometheus `/api/v1/status/flags`, Tempo `/status/config` and Loki
      `/config` and survive `docker compose restart`; a sample of each
      signal written before a stop is readable after a start; the cleanup
      rehearsal on the scratch volume, per store, deletes a sample before
      the cutoff and keeps one after it, with the Loki delay observed, and
      for Tempo removes a block that ends before the cutoff, keeps a block
      that starts after it and keeps a straddling block whole, reporting that
      granularity; data written under `0.33.1` (`PETRI_LGTM_IMAGE`) is
      readable under `0.34.0`. Transcript in
      [`docs/progress/readings/t21-f01.md`](../../progress/readings/t21-f01.md).
- [x] Bytes per run: the `bytes=` figure and the measured config body size
      of the workload run, and the growth of each store's disk use across ten
      workload runs, in the readings file.
- [x] Parent comparison: `scripts/telemetry-parent-compare ae97765f` (the
      merge base with `main`, F01's parent) -> identical canonical output,
      result in the readings file; the server is covered by its three-state
      neutrality test.
- [x] Overhead check: `scripts/telemetry-overhead` -> the table in the
      readings file, verdict in Performance and Goal Impact.
- [x] `make check` -> exit 0.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: `Not applicable:
      observability feature`.
- [x] Benchmark summary: `Not applicable: observability feature`.

| Item (2026-09-29) | Result |
| --- | --- |
| Neutrality, bounds, flag precedence, server tests | pass inside `make check`: `canonical_output_is_identical_off_on_and_on_with_a_closed_port`, 15 `v3-telemetry` tests (including `records_a_partial_success_rejects_count_as_failed_for_their_run` and `the_build_script_watches_head_by_an_absolute_path`), three `telemetry_flag_beats_environment_which_beats_default_off`, `each_seeding_is_a_run_and_each_reset_ends_the_previous_one`, `simulation_payloads_are_identical_off_on_and_on_with_a_closed_port`, `shutdown_with_pending_records_ends_within_10_s_with_exact_counts`, `shutdown_stops_the_running_simulation_at_the_tick_run_ended_reports`; `telemetry-cleanup-test` passes `--proceed` only for `PROCEED=1` |
| Reference build | compiles; `cargo tree -e features` shows 0 `v3-telemetry`/`telemetry-seams` lines; clippy `-D warnings` passes with and without the feature |
| `scripts/telemetry-verify` | exit 0, all 10 checks PASS; Loki delete processed after 318 s; straddling block kept whole under the `1m0s` window |
| Bytes per run | `bytes=3377`; stored `run.started` body 2,777 B; ten runs: Loki +16 KiB, Tempo 0, Prometheus +36 KiB (+164 KiB idle over the same time) |
| `scripts/telemetry-parent-compare ae97765f` | identical, 3 canonical lines, T = 6908 |
| `make check` | exit 0 |

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary; the reviewer checks that against the diff, and
every change to `v3-cli`, `v3-server` and `v3-lab` source sits behind the
`telemetry` feature, while `v3-core` changes only in its `Cargo.toml`
feature list. Measurement feature: the natural-analog rule and the
environmental-pressure rule do not apply, and no indicator can move.

Parent comparison: runs because F01 edits `run_simulation` and the server's
run loop file. Expected: identical canonical output.

Overhead check, method fixed before any run:

| Item | Predeclared |
| --- | --- |
| Workload | the workload above; calibrating `T` (three reference runs) is charged to the cap in the closure that calibrates |
| Reference | the same commit built with `--no-default-features`, always run with the stack stopped |
| Spread | three consecutive reference runs; spread = (max − min) / median |
| Pairs | n = 5 when spread ≤ 5%, n = 8 when spread ≤ 10%, else inconclusive before any pair runs |
| States | (a) telemetry on, stack healthy: each pair is one telemetry run with the stack started and one reference run with it stopped, so the ratio includes the stack's contention; (b) telemetry on, stack stopped; (c) idle-stack cost: three reference runs with the stack started against the reference runs of (b), a block comparison |
| Reading | (a) and (b): the median over pairs of (telemetry wall time / reference wall time), pair order alternated; (c): median reference wall time with the stack started / median with it stopped |
| Verdict | pass when the median ≤ 1.25 and at least ⌈0.75 n⌉ pairs are ≤ 1.25; fail when the median > 1.25 and at least ⌈0.75 n⌉ pairs are > 1.25; otherwise inconclusive; (c) has no ceiling and is recorded as the idle-stack cost |
| Self time | `self_time_us` / `T` from the stderr line, recorded beside each ratio as a lower bound |
| Rerun | one rerun of one inconclusive state, with n fresh pairs, only when it fits the remaining cap; both results are recorded and the rerun is the verdict |
| Time cap | calibration, spread, all pairs of every state, the block of (c) and the rerun within 300 s of measured run time; the script stops at the cap and reports `inconclusive` |
| Inconclusive | reported to the user as such, never rounded to a pass or extended with more runs |
| Expected | (a) and (b) within noise of 1.0: F01 adds two records and one stderr line per run |

**Measured verdict.**

| Record | Value |
| --- | --- |
| Profiles | `Not applicable: observability feature` |
| Parent comparison | method above, `BASE` `ae97765f`, T = 6908: identical canonical output |
| Overhead check | method above, n = 8 (spread 5.40%), T = 6908; measured time 190.3 s of 300 s over four attempts (158.2 s for the record run) |
| (a) on, stack healthy | median ratio 0.996, 8/8 pairs ≤ 1.25, pass; self time 0.102 µs per tick |
| (b) on, stack stopped | median ratio 1.010, 8/8 pairs ≤ 1.25, pass; self time 0.018 µs per tick |
| (c) idle-stack cost | 1.003 |

- Readings: [`docs/progress/readings/t21-f01.md`](../../progress/readings/t21-f01.md).

## Success Criteria

- [x] `make telemetry-up` starts the stack, prints each store's disk use,
      and Grafana answers on `127.0.0.1:3300`; the three verifications pass
      on `grafana/otel-lgtm:0.34.0`, or the failed check is recorded and the
      spec has been revised to the separate-services layout before
      implementation continues.
- [x] `make telemetry-clean CUTOFF=<date or RFC3339>` previews disk use, interval and
      targets per store and deletes only with `PROCEED=1`, keeping everything
      newer.
- [x] A `v3-cli run --telemetry on` run appears in Loki with the identity
      attributes, `run.started` and `run.ended`; a `v3-server --telemetry on`
      run adds `run.state` and `run.config`, and a reset starts a new run ID.
- [x] The telemetry-neutrality and bounds tests pass inside `make check`
      without Docker, and the flag defaults to off in all three binaries.
- [x] The parent comparison is identical and the overhead check passes in
      states (a) and (b), with the idle-stack cost and the self-timed cost
      per tick recorded.

## Notes for AI Agents

- Decision: the stack layout is the bundled image (user decision D4,
  2026-09-29). A failed verification needs no user decision but is an
  escalation to the spec owner, who revises this spec to the separate
  Collector, Prometheus, Tempo, Loki and Grafana services (their versions,
  volumes, retention settings, endpoints and cleanup paths) before
  implementation continues; a failure both layouts share is a user blocker.
- Exception: the challenge loop ended `not-ready` (user decision to commit
  the plan, 2026-09-29). Rounds, models and dispositions are in the
  readings file; the round-3 fixes are unconfirmed. No fourth round (user).
- Decision: `profile_ticks` keeps no flag (user, 2026-09-29).
- Cost: `/usage` awaiting; 4 implementer passes (advisor 3, 2, 4, 2);
  5 spec-owner resumes; 3 Codex rounds, final `not-ready`; review 4 P1,
  0 P2, 0 P3, all fixed in one pass.
