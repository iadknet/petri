# T21.F01 — Local Telemetry Stack and Run Identity

**Status**: In Progress
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
bounded flush at its end. The feature also delivers the three commands later
T21 closures use in place of the skipped gates: the telemetry-neutrality test
inside `make check`, the parent comparison and the overhead check.

## Non-Goals

- Exported counters, gauges, tick gauges, dashboards and Grafana provisioning
  (T21.F02); tick, phase and creature traces (T21.F03, T21.F04); run records
  for `bench`, `recruitment`, `input-opportunity` and `v3-lab` (T21.F06):
  at F01 those commands accept the flag and export nothing; the default flip
  and presets (T21.F05); template conventions (T21.F07).
- Any change to `v3-core`: F01 reads seed, tick, status and config from the
  binaries and adds no observation seam. `crates/v3-core/src/bin/profile_ticks.rs`
  is a profiling harness compiled inside `v3-core`, which the track's
  Boundary note keeps free of any telemetry dependency, so it takes no flag.
- Retiring one run by ID; the cleanup command takes a cutoff date only.
- Retries inside the exporter, disk buffering, and any frontend change.
- Grafana dashboards: at F01 a run is found through Grafana Explore on the
  Loki data source, or through the Loki query API.

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

Options weighed before the design below: the bundled image against separate
services (decided D4, fallback kept); a `tracing` subscriber bridge against
direct SDK calls in a new crate (the note's choice: direct calls, `v3-core`
untouched); `uuid` against OS entropy through the workspace's `rand` for IDs
(no new crate for 128 random bits); a `build.rs` in the telemetry crate
against a Makefile-injected variable for the build revision (`build.rs`, so
`cargo run` and `cargo test` carry it too).

Invariants:

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
   that drops when either is full; a record body above 256 KiB is truncated
   and marked `petri.truncated=true`; one export batch of at most 512
   records in flight; request timeout 5 s; no retry; the flush that starts
   at a run's end (CLI) or at process shutdown (server) finishes within 10 s
   and abandons what remains. Counters are kept per run and released when
   that run's summary line prints.
5. Reference build: `v3-cli`, `v3-server` and `v3-lab` gain a cargo feature
   `telemetry`, on by default, that gates the optional `v3-telemetry`
   dependency, the flag and every call into it. None of the three has
   another default feature, so `--no-default-features` is the reference
   build and compiles out exactly T21's work.
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
| Volume | named volume `${PETRI_TELEMETRY_VOLUME:-petri-telemetry}` at `/data`; the two variables exist for `scripts/telemetry-verify` only |
| Prometheus | `PROMETHEUS_EXTRA_ARGS=--storage.tsdb.retention.time=100y --storage.tsdb.retention.size=0 --web.enable-admin-api` |
| Tempo | mounted `telemetry/tempo-config.yaml`: the image's file for 0.34.0 (extracted from the image) plus `compactor.compaction.block_retention: 876000h` |
| Loki | `LOKI_EXTRA_ARGS=-compactor.retention-enabled=true -compactor.delete-request-store=filesystem -compactor.delete-request-cancel-period=1m -compactor.retention-delete-delay=1m -compactor.compaction-interval=5m`; `retention_period` stays at its default `0s`, so nothing expires by age |
| Grafana | defaults of the image; login `admin`/`admin` |

`make telemetry-up` runs `docker compose up -d` and then prints each store's
disk use (`du -s` of `/data/prometheus`, `/data/tempo`, `/data/loki` inside
the container); `make telemetry-down` stops the stack and keeps the volume.

**Cleanup.** `scripts/telemetry-cleanup CUTOFF [--proceed]`, behind
`make telemetry-clean CUTOFF=YYYY-MM-DD [PROCEED=1]`, uses only the pinned
image (through `docker compose exec` and `docker compose run`). Without
`--proceed` it prints each store's disk use, the deletion interval
(everything before `CUTOFF` 00:00 UTC) and the exact targets, then exits
without deleting.

| Store | Preview | Deletion |
| --- | --- | --- |
| Prometheus | the series label sets returned by `GET /api/v1/series?match[]={__name__=~".+"}&end=<cutoff>`, with their count | `POST /api/v1/admin/tsdb/delete_series?match[]={__name__=~".+"}&end=<cutoff>`, then `POST /api/v1/admin/tsdb/clean_tombstones` |
| Loki | the streams returned by `GET /loki/api/v1/series?match[]={service_name=~".+"}&start=1&end=<cutoff>`, with their count | `POST /loki/api/v1/delete` with `query={service_name=~".+"}`, `start=1`, `end=<cutoff>`; the script prints the request ID and the status endpoint |
| Tempo | block IDs under `/data/tempo/blocks/*/*/meta.json` whose `endTime` is before the cutoff, each with its start, end and byte size, plus the IDs of straddling blocks it keeps whole | the script stops the stack, removes the listed block directories through `docker compose run --rm` on the same image, and starts the stack again |

**Run identity and records.** `crates/v3-telemetry` owns the SDK, the
identity, the records and the exporter; the binaries call it.

| Attribute | Level | Value |
| --- | --- | --- |
| `service.name` | resource | `v3-cli`, `v3-server` or `v3-lab` |
| `service.version` | resource | `CARGO_PKG_VERSION` |
| `process.pid` | resource | the process ID |
| `petri.invocation_id` | resource | 128 bits of OS entropy as 32 lowercase hex, once per process |
| `petri.build_revision` | resource | 40-hex commit from the crate's `build.rs` (`git rev-parse HEAD`), `-dirty` appended when tracked files differ when the crate builds, `unknown` when no repository is visible; `PETRI_BUILD_REVISION` overrides it; `rerun-if-changed` on the paths `git rev-parse --git-path HEAD` and the checked-out ref resolve to (linked worktrees have a `.git` file), so a dirty flag can be stale between rebuilds |
| `petri.run_id` | record | 32 lowercase hex, drawn at each `seed_simulation` |
| `petri.seed` | record | the run seed |
| `petri.world` | record | `<width>x<height>` of the effective config |
| `petri.recipe` | record | the `--config` path when one was given; absent otherwise |
| `petri.config_digest` | record | `config_digest(&config)` |
| `petri.tick` | record | the tick at capture |

| Record | Emitted | Attributes and body |
| --- | --- | --- |
| `run.started` | after `seed_simulation`, before the first tick: CLI `run_simulation`; server `AppState` construction (the initial seed-0 run) and every `startup` | identity above; CLI adds `petri.ticks_requested` and `petri.sample_every`; body is the full effective `SimulationConfig` as compact JSON, the same document `--save-config` writes |
| `run.state` | each server status transition (`idle`, `running`, `paused`) | `petri.state`, `petri.tick` |
| `run.config` | a server config patch is accepted | the new digest; body the new full config |
| `run.ended` | CLI after `run_completed`; server at the next `startup` (`reset`) and at SIGINT/SIGTERM (`shutdown`) | `petri.status` (`completed`, `reset`, `shutdown`), `petri.tick`, `petri.wall_seconds` |

An attribute that is not known is omitted, never written as zero. Loki keeps
resource attributes such as `service.name` as labels and the rest as
structured metadata, so a run is found with
`{service_name="v3-cli"} | petri_run_id="<id>"`.

**Failure behavior and self-report.** Export is best effort over OTLP/HTTP
with the blocking client on the SDK's own thread. When telemetry is on, the
binary prints one stderr line at start
(`telemetry: on endpoint=<url> invocation=<id> run=<id>`) and one per run
(`telemetry: run=<id> exported=<n> failed=<n> dropped=<n> abandoned=<n> bytes=<n> self_time_us=<n> flush_ms=<n>`).
Every record is attributed to its run when enqueued; the run's line prints
once the run has ended and each of its records has been exported, failed,
dropped by the full queue, or abandoned by the shutdown flush, so a server
reset never blocks on the previous run's flush. The counts are exact; the
implementer chooses the processor (the SDK's batch processor wrapped, or a
crate-owned bounded queue), but the drop and abandon counts must be exact.
`self_time_us` is the time spent inside telemetry calls on the simulation
thread and `flush_ms` the time the end-of-run or shutdown flush took.

**Check commands.** All three are POSIX `sh` under `scripts/` and wrap every
timed run in `scripts/bench-wait`.

| Command | What it does |
| --- | --- |
| `scripts/telemetry-parent-compare [BASE]` | Adds a scratch worktree at `BASE` (default `git merge-base HEAD main`) outside the calling checkout, builds both `v3-cli` release binaries into separate target directories, runs the workload below on both (`--telemetry off` on the changed build; the parent takes no flag), compares the canonical form of both outputs, removes the worktree, and exits non-zero on a difference. |
| `scripts/telemetry-overhead` | Builds the telemetry build and the reference build (`--no-default-features`, separate target directory); calibrates `T` when the readings file has none; measures the spread; runs the pairs of each state below, stopping and starting the stack with `docker compose stop` and `start` and waiting for Grafana `/api/health` plus 5 s before a timed run that follows a start; prints one table with the per-state median ratio, the verdict, the pair count, the self-timed cost per tick from the stderr line, and the measured time used. It refuses a run that would pass 300 s of measured time and reports `inconclusive`. |
| `scripts/telemetry-verify` | Runs the three stack verifications and the cleanup rehearsal under Compose project `petri-telemetry-verify` on the scratch volume `petri-telemetry-verify`, never on `petri-telemetry`; refuses to start while the `petri-telemetry` project is up (same host ports); mounts `telemetry/verify/tempo-config.yaml`, the production file with a short `ingester.max_block_duration`, so block boundaries fall within minutes; prints one pass/fail line per check and removes the scratch volume at the end. |

Workload for both checks: `v3-cli run --seed 7 --ticks T --sample-every T
--config telemetry/overhead-world.json`, where the recipe is the gate
profile's world (128x128, 256 founders, food coverage 1.0) and `T` is
calibrated once with three reference runs so that one run takes 3 to 5 s on
this host, recorded in the readings file and fixed for later closures.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `run.started` | log record | `seed_simulation` in `run_simulation`, server construction and the `startup` handler | resource identity, seed, world, recipe, digest, ticks requested, full config body |
| `run.state` | log record | server status transitions | state, tick |
| `run.config` | log record | accepted server config patch | new digest, full config body |
| `run.ended` | log record | CLI completion, server reset and shutdown | status, tick, wall seconds |
| exporter self-report | stderr line | the telemetry crate's own counters | exported, failed, dropped, abandoned, bytes, self time, flush time |

No counter, gauge or span is exported at F01; the runtime counters stay
unchanged for T21.F02. The records consume no production RNG and change no
execution; the `--telemetry` flag is a process-level setting and enters no
recorded configuration.

## Implementation Tasks

- [ ] `telemetry/compose.yaml`, `telemetry/tempo-config.yaml`,
      `telemetry/verify/tempo-config.yaml`, `telemetry/overhead-world.json`;
      `make telemetry-up`, `telemetry-down`, `telemetry-clean`.
- [ ] `crates/v3-telemetry`: identity, `build.rs` revision, the four
      records, bounded exporter with exact per-run counts, stderr
      self-report, unit tests for saturation and the flush bound.
- [ ] `v3-cli`: global `--telemetry`, feature `telemetry`, hooks in
      `run_simulation`; `v3-server`: `--telemetry`, feature, hooks in
      `AppState` construction, `startup`, `start`, `pause_sim`,
      `patch_config`, signal shutdown; `v3-lab`: flag and feature, inert.
- [ ] Telemetry-neutrality test in `cargo test -p v3-cli` with an in-test
      OTLP receiver on an ephemeral port, and server tests for run identity
      across `startup` and for shutdown with pending records.
- [ ] `scripts/telemetry-cleanup`, `scripts/telemetry-verify`,
      `scripts/telemetry-parent-compare`, `scripts/telemetry-overhead`.
- [ ] Run the verifications and both checks on this host; record them in
      `docs/progress/readings/t21-f01.md`.

## Verification

- [ ] Telemetry-neutrality test: `cargo test -p v3-cli` (inside `make check`,
      no Docker) -> one seeded run's canonical NDJSON is identical with
      telemetry off, on with an in-test OTLP receiver that saw `run.started`
      and `run.ended` for the run ID, and on with a closed port.
- [ ] Bounds: `cargo test -p v3-telemetry` -> a queue driven past 2,048
      records or 8 MiB reports the exact dropped count; a receiver that
      accepts the connection and never answers leaves the CLI run's flush
      within 10 s with `abandoned` equal to the records still pending; a
      body above 256 KiB is truncated and marked.
- [ ] Server tests: `cargo test -p v3-server` -> the initial run and two
      `startup` requests yield three distinct run IDs, `run.ended` with
      `reset` for each replaced run, `run.state` per transition, `run.config`
      after a patch, and shutdown with pending records completes within
      10 s with exact counts.
- [ ] Reference build: `cargo build --release -p v3-cli -p v3-server -p v3-lab
      --no-default-features` compiles, and `cargo clippy --workspace
      --all-targets -- -D warnings` passes with and without the feature.
- [ ] Stack verifications (Docker, outside `make check`):
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
- [ ] Bytes per run: the `bytes=` figure and the measured config body size
      of the workload run, and the growth of each store's disk use across ten
      workload runs, in the readings file.
- [ ] Parent comparison: `scripts/telemetry-parent-compare` -> identical
      canonical output against the merge base with `main`, result in the
      readings file.
- [ ] Overhead check: `scripts/telemetry-overhead` -> the table in the
      readings file, verdict in Performance and Goal Impact.
- [ ] `make check` -> exit 0.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: `Not applicable:
      observability feature`.
- [ ] Benchmark summary: `Not applicable: observability feature`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary; the reviewer checks that against the diff, and
every change to `v3-cli`, `v3-server` and `v3-lab` source sits behind the
`telemetry` feature. Measurement feature: the natural-analog rule and the
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

**Measured verdict.** Not yet measured.

- Readings: [`docs/progress/readings/t21-f01.md`](../../progress/readings/t21-f01.md).

## Success Criteria

- [ ] `make telemetry-up` starts the stack, prints each store's disk use,
      and Grafana answers on `127.0.0.1:3300`; the three verifications pass
      on `grafana/otel-lgtm:0.34.0`, or the fallback layout is in place with
      the failed check recorded.
- [ ] `make telemetry-clean CUTOFF=<date>` previews disk use, interval and
      targets per store and deletes only with `PROCEED=1`, keeping everything
      newer.
- [ ] A `v3-cli run --telemetry on` run appears in Loki with the identity
      attributes, `run.started` and `run.ended`; a `v3-server --telemetry on`
      run adds `run.state` and `run.config`, and a reset starts a new run ID.
- [ ] The telemetry-neutrality and bounds tests pass inside `make check`
      without Docker, and the flag defaults to off in all three binaries.
- [ ] The parent comparison is identical and the overhead check passes in
      states (a) and (b), with the idle-stack cost and the self-timed cost
      per tick recorded.

## Notes for AI Agents

- Decision: the stack layout is the bundled image (user decision D4,
  2026-09-29); a failed verification switches F01 to separate services
  without a further user decision, and a failure both layouts share is a
  user blocker.
- Exception: plan committed without a Codex `ready` verdict (user
  decision, 2026-09-29). Round 1 (`task-mun4xhdu-i5w6ec`) was `not-ready`
  with 10 blocking findings; the spec owner fixed nine and rebutted one.
  No Codex round confirmed them: the account's usage limit (reset
  2026-10-03 11:07) failed round 2 (`task-mun55swy-2n3x72`,
  `task-mun56ruh-xd9j61`) and an Astra `low` probe
  (`task-mun584cu-r684nn`). Later Codex jobs in this track use Astra
  `low` (user request, 2026-09-29).
