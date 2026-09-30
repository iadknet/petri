# T21.F03 — Tick and Phase Tracing

**Status**: Complete
**Last updated**: 2026-09-29
**Feature**: T21.F03
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

A run started with `--telemetry on` exports one short trace for each tick an
interval snapshot is taken at, and for the final tick of a CLI run: a `tick`
span with five child spans, one per phase of `run_tick`, whose durations
are the phase timings `v3-core` already measures and whose attributes carry
the run's identity, the tick, the config values governing each phase and
the world state in force. `Petri / Run` lists the run's tick traces and
opens any of them. With telemetry off, on, or on with no stack listening,
the run's deterministic output is identical, and `run_tick` behaves the
same with the seam compiled in or out.

## Non-Goals

- Creature windows and the server's manual sampler (T21.F04); measurement
  command records (T21.F06); the qualified cadence, presets and default-on
  (T21.F05); the template conventions (T21.F07).
- No new counter, no change to a T14 counter or to `phase_wall_clock`, no
  change to the CLI's NDJSON or the server's payloads, no
  `SimulationConfig` or `RuntimeConfig` field.
- No OpenTelemetry trace SDK: spans are encoded on the simulation thread
  and travel through T21.F01's queue, as T21.F02's snapshots do.
- No Tempo configuration change and no TraceQL metrics.
- No trace with a server transition, reset or shutdown snapshot, and none
  for a tick that has no snapshot; no per-creature or per-action span.

## Inputs and Invariants

| Input | Where | What F03 takes from it |
| --- | --- | --- |
| Track row and notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | Boundary (plain data behind `telemetry-seams`, no exporter state in `v3-core`); context contract (identity, tick, phase, governing config values, digest; absent never zero); sampling contract (hard caps per tick and per wall second, selection never draws production RNG, every sample names its policy); time (durations are wall time, every span carries its tick, one trace per sampled tick); run identity; configuration (process-level only); failure behavior; storage and the commit rule (projected size per run and per-run cap); overhead ceiling 25%; reference build. |
| Benchmark gate | `docs/workflow.md`, "Benchmark gate" | Observability exemption and its no-behavior-change condition; check triggers; the overhead method, 300 s cap and inconclusive rule; record format. |
| F01 spec | `docs/specs/roadmap/t21-f01-local-telemetry-stack-and-run-identity.md` | The flag and endpoint; the reference build (`--no-default-features`, `v3-core/telemetry-seams`); the queue bounds (2,048 items, 8 MiB, 4 MiB body, one batch in flight, 5 s request, 10 s flush, no retry); exact per-run counts and the stderr line; the identity attributes; the three check commands; T = 6908. |
| F02 spec | `docs/specs/roadmap/t21-f02-run-metrics-and-dashboards.md` | The snapshot cadence (`PETRI_TELEMETRY_METRICS_INTERVAL_MS`, interval, transition and run-end triggers, at most one per tick); the queue item pattern (encoded request, batch of one, per-item accounting); `petri.phase` keys `world_update`, `sensor_assembly`, `cognition`, `actions`, `reward_learning`; the dashboards and `scripts/telemetry-dashboards-check`. |
| Research note | `docs/strategy/run-observability-research-2026-09-29.md` | Signals table (cheap aggregates always, detailed tick traces sampled); context R10; trace SDK is Beta; a span costs 1.1 to 2.5 µs in the SDK's own benchmark. |
| Tick loop | `crates/v3-core/src/simulation/tick.rs` `run_tick` | Five timed phases in a fixed order, each `phase_started.elapsed()` added to `sim.stats.phase_wall_clock`; the untimed remainder (energy snapshot, queue shuffle, priority sort) is the gap between them; `sim.tick += 1` at the end. |
| Stats | `crates/v3-core/src/simulation/stats.rs` `SimStats` (`Debug, Clone, Default`, not serialized), `PhaseWallClock`, `last_tick_*` | Where the seam lives; the per-tick values the tick span carries. |
| Config | `crates/v3-core/src/config/simulation.rs` | The paths in the attribute table; `failed_action_penalty_for_tick(tick)` is the penalty in force under the startup ramp. |
| Run loops | `crates/v3-cli/src/{lib,telemetry}.rs`, `crates/v3-server/src/{telemetry.rs,http/lifecycle.rs}` | `after_tick` after every `run_tick` (CLI loop, server `run_loop` and `step`); transition, reset and shutdown snapshots. |
| Exporter | `crates/v3-telemetry/src/{lib,queue,export,metrics,testing}.rs` | `Item::Snapshot` and `MetricsClient` are the pattern; `RunHandle` holds `started` and `started_ns`; the in-test `Receiver` decodes logs and metrics. |
| Proto crate | `opentelemetry-proto` 0.33.0 | Feature `trace` gates `trace::v1` and `collector::trace::v1`; `ExportTraceServiceResponse.partial_success.rejected_spans`. |
| Image | `grafana/otel-lgtm:0.34.0`: Tempo 3.0.3, Grafana 13.2.2 (probed 2026-09-29 on scratch project `petri-telemetry-f03plan`, readings file) | OTLP/HTTP traces accepted at `/v1/traces`; `/api/v2/traces/<id>` answers at once, and an unknown ID with `200` and an empty trace; TraceQL search over span attributes (`{ span.petri.run_id = "…" }`) finds a trace only once the live store has cut its block (`max_block_duration: 30s`, `max_trace_idle: 5s`; found 30 s after ingest); Grafana `/api/ds/query` with `queryType: traceql`, `tableType: traces` returns a `Traces` frame whose `traceID` field carries a data link to the trace; the metrics-generator writes `traces_spanmetrics_{calls_total,latency_*,size_total}` per span name and service. |

Invariants:

1. No `SimulationConfig` or `RuntimeConfig` field, no production RNG draw,
   no simulation default, no stored summary content and no deterministic
   output changes; the config digest and the pinned recipe digests do not
   move.
2. Seam. Under `#[cfg(feature = "telemetry-seams")]` only, `SimStats` gains
   `last_tick_phases: Option<TickPhaseTimings>`, plain data: the tick count
   after the tick, the population before phase 0, the rayon threads in
   effect (`rayon::current_num_threads()`, read inside `run_tick` so an
   installed pool is seen), the tick's start `Instant` and whole elapsed
   time, and for each of the five phases its offset from the tick start
   and its elapsed time, the same `Duration` `run_tick` adds to
   `phase_wall_clock`. `run_tick` overwrites it every tick; the tick gains
   only the stores, a length read, the thread-count read and one clock
   read at its end. `v3-core` takes no dependency and holds no exporter
   state; `v3-telemetry` depends on `v3-core` with `telemetry-seams`, so
   `cargo test -p v3-telemetry` builds the seam and the binaries'
   `--no-default-features` build compiles it out with the crate.
3. Selection. A tick trace is taken with an interval snapshot, in
   `after_tick` right after the `run_tick` that produced the seam and
   under the same simulation lock, and with the CLI's completion snapshot,
   which follows the last tick with nothing between; both read the config
   that governed the tick. A server transition, reset or shutdown snapshot
   takes none: the server accepts a config patch while paused
   (`http/status.rs` `patch_config`), so the seam's tick may have run
   under an earlier config. A trace so obeys the snapshot's caps (at most
   one per tick, one per interval, one at completion) and adds no cadence;
   a run of zero ticks has none. Traces are on when telemetry is on and
   off with `PETRI_TELEMETRY_TICK_TRACES=off` (`on`/`off`, read only when
   telemetry resolves on, anything else refuses to start), so T21.F05 can
   price them apart from snapshots; the setting is on `run.started` as
   `petri.tick_traces`. The decision is a pure function of the run's clock
   and tick; nothing draws from a simulation RNG.
4. Identity without entropy. The trace ID is the run key's high 64 bits
   followed by the tick as a big-endian `u64`, so the trace of tick N of a
   run is the first 16 hex digits of its run ID followed by N in 16 hex
   digits; span IDs are `1` for `tick` and `2` to `6` for the phases in
   order. Timestamps are `started_ns` plus monotonic offsets from the run's
   `started` `Instant`, so a clock step never reorders a trace.
5. Transport and accounting. A trace is encoded on the simulation thread
   as one `ExportTraceServiceRequest` under F01's resource and offered to
   the queue as one item attributed to its run; the worker posts it alone
   to `<endpoint>/v1/traces`. The queue bounds, body cap, drop-when-full
   and flush rules apply as to snapshots; a trace is `exported` with zero
   rejected spans, `failed` on a failed request, any rejected span or an
   undecodable success (no bytes counted), `dropped` or `abandoned` as a
   record is. The stderr run line gains `traces=<taken>`; `self_time_us`
   includes the capture and the encoding.
6. Context. Every span carries the attributes of the "every span" row
   below; a phase span adds its governing config values read from
   `sim.config` at capture, so a server config patch shows on the next
   trace with the new digest. A value that was not captured is omitted.
7. Bounds. The attribute set is fixed except a few attributes per food
   type, so capture and encoding work is bounded by the config; the
   encoding test asserts an eight-type config encodes under 16 KiB, and an
   encoded trace over the queue's 4 MiB body cap is dropped whole, counted
   and named on stderr as F01's rule says. Per-run cap: at most 65,536
   traces per run, checked before capture; once reached no trace is
   captured, `traces=` stops at the cap and `run.ended` carries
   `petri.tick_traces_capped` = 65536. Below the cap the rate is the
   snapshot cadence's: at the default interval at most 3,600 traces per
   hour, sized below.
8. Dashboard. `telemetry/grafana/dashboards/petri-run.json` gains a row
   `Tick traces` with a table panel on the `tempo` datasource: TraceQL
   `{ span.petri.run_id = "$run_id" && name = "tick" } | select(span.petri.tick, span.petri.sample_policy)`,
   `queryType` `traceql`, `tableType` `spans`, limit 500, the dashboard's
   range, one row per trace showing its start, duration, tick and policy,
   whose span ID link opens the trace (the `traces` table type hides
   selected attributes in a nested field). The dashboards are still
   provisioned read-only under their fixed UIDs.

**Spans and attributes.** Names are OTel dotted names; Tempo indexes them
as `span.<name>`.

| Span | Start and end | Attributes and source |
| --- | --- | --- |
| every span | — | `petri.run_id`, `petri.seed`, `petri.world`, `petri.recipe` (when given), `petri.config_digest` (the digest in force at capture) from `RunHandle`; `petri.tick` (the tick count after the traced tick, the snapshot's tick gauge value; the tick ran under `sim.tick` = `petri.tick` − 1); `petri.sample_policy` = `interval` or `run_end`; resource attributes from F01 (`service.name`, `service.version`, `process.pid`, `petri.invocation_id`, `petri.build_revision`) |
| `tick` (root) | seam's tick start; start plus the tick's elapsed time | `petri.phase` = `tick`; `petri.population_start` and `petri.threads` (seam), `petri.population` (`creatures.len()` at capture), `petri.actions.{move,eat,noop,reproduce,steal}`, `petri.predation_kills`, `petri.compute_total_{mean,min,max}`, `petri.compute_{vm,graph}_mean`, `petri.priority_bid_mean`, `petri.priority_bidders_count` (the `last_tick_*` values of the traced tick), `petri.food_total_density.<i>` per food type (`last_tick_food_total_density_by_type`: after phase 0's growth, before actions) |
| `world_update` | start plus the phase offset; plus its elapsed | `petri.phase`; `petri.config.` + `energy.lifecycle.energy_decay_per_tick`, `energy.lifecycle.genome_carry_cost_per_unit`, `shared_memory.decay_rate`, `world.food.shared.growth_rate`, `world.food.shared.max_density`, `world.food.shared.grazing.enabled`, `world.food.shared.occupancy_depletion.enabled`, `world.food.annealing.enabled`, `world.food.types` (count) and per type `world.food.types.<i>.growth_inhibitor` (its own value), `growth_rate` and `recovery_spawn_rate` (the type's value, else the shared one) and `energy_per_unit` (the type's value, else `energy.costs.eat_reward_per_food`) |
| `sensor_assembly` | as above | `petri.phase`; `petri.config.runtime.perception.vision_radius`, `petri.config.world.edge_mode` |
| `cognition` | as above | `petri.phase`; `petri.config.runtime.` + `max_mesh_hops`, `max_vm_steps`, `graph_node_base_cost`, `plasticity_update_cost`, `hop_ramp_allowance`, `hop_ramp_cost`, `max_actions_per_turn`, `vm.opcode_cost_multiplier`, `vm.step_ramp_allowance`, `vm.step_ramp_cost` |
| `actions` | as above | `petri.phase`; `petri.failed_action_penalty` (`failed_action_penalty_for_tick` for the tick that ran); `petri.config.` + `energy.costs.{move_cost,eat_cost,eat_reward_per_food,noop_cost,reproduce_cost}`, `energy.complexity_cost.{enabled,threshold,scaling_factor}`, `energy.age_cost.{enabled,grace_ticks,age_cap,max_multiplier}`, `energy.lifecycle.{min_reproduce_energy,min_reproduce_age,genome_replication_cost_per_unit}`, `population.max_creatures`, `mutation.{per_unit_rate,genome_size_cap,genome_size_pressure_enabled,mesh_layer_probability,executed_bias}`, `predation.{steal_cost_rate,kill_complexity_bonus_multiplier}` |
| `reward_learning` | as above | `petri.phase`; `petri.config.runtime.reward_learning_cost` |

Projected size: the seven identity attributes on each of six spans plus
the rows above give about 110 attributes and four more per food type, at
40 to 60 B each with span framing about 5 to 6 KB per trace; so about
22 MB per hour at the default cadence and about 25 KB for the workload
run. The measured `bytes=` and `traces=` of the workload run and Tempo's
growth over ten runs go in the readings file.

**Check command.** `scripts/telemetry-dashboards-check` (T21.F02) gains the
rows below and keeps its scratch project.

| Step | What it does or asserts |
| --- | --- |
| Panel queries | the query builder handles a Tempo target (`query`, `queryType`, `tableType`, `limit` with `$run_id` substituted) beside a Prometheus `expr`; the `Tick traces` query is polled through `/api/ds/query` until its frame has rows, up to 120 s after the run (the live store's block cut), and the row count is recorded |
| Trace content | for the first row's trace ID, `/api/datasources/proxy/uid/tempo/api/v2/traces/<id>` returns six spans, `tick` and the five phase names, each carrying `petri.run_id`, `petri.tick` and `petri.sample_policy`; a trace ID composed from the run ID and that tick answers with the same spans |
| Snapshot join | the trace's `petri.tick` is one of the run's `petri_run_tick` sample values |
| Run record | the Loki `run.started` line of the run carries `petri_tick_traces` |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `tick` span | span, sampled at the snapshot tick | the seam's tick timing and the tick's `last_tick_*` values, read after `run_tick` | identity, tick, sample policy, population before and after, threads, per-tick actions and compute, food density by type |
| `world_update`, `sensor_assembly`, `cognition`, `actions`, `reward_learning` spans | child spans | the seam's phase offsets and elapsed times, the same `Duration`s `phase_wall_clock` accumulates | identity, tick, `petri.phase`, the governing config values (table above), the penalty in force |
| `run.started`, `run.ended` | log records (F01) | gain `petri.tick_traces`; `petri.tick_traces_capped` when the cap was reached | whether tick traces are on; the per-run cap |
| exporter self-report | stderr line (F01) | gains `traces=` | traces taken |
| `traces_spanmetrics_*` | Prometheus series the image's metrics-generator derives | every exported span | span name, service; not something Petri emits or reads |

Traces consume no production RNG, change no execution and are marked with
the policy that selected them; a value that was not captured is absent,
never zero. Nothing here is closure evidence.

## Implementation Tasks

- [x] `v3-core`: `TickPhaseTimings` and `SimStats::last_tick_phases` behind
      `telemetry-seams`; `run_tick` records them; a unit test under the
      feature checks the fields against `phase_wall_clock` and the order,
      run by a `Makefile` row inside `make check`.
- [x] `crates/v3-telemetry`: `v3-core` with `telemetry-seams`;
      `opentelemetry-proto` feature `trace`; `PETRI_TELEMETRY_TICK_TRACES`
      in `Options`; trace capture and encoding (`trace.rs`); `Item::Trace`,
      `TracesClient` for `/v1/traces`, partial-success and undecodable
      accounting, `traces=`; `petri.tick_traces` on `run.started`; the
      in-test receiver decodes `POST /v1/traces`.
- [x] `v3-cli` and `v3-server`: `after_tick` and the CLI's completion take
      the trace with the snapshot they already take; the server's
      transition, reset and shutdown snapshots do not.
- [x] `telemetry/grafana/dashboards/petri-run.json`: the `Tick traces` row.
- [x] `scripts/telemetry-dashboards-check`: the rows above.
- [x] Run the checks on this host; record them in
      `docs/progress/readings/t21-f03.md`.

## Verification

- [x] Seam: `cargo test -p v3-core --features telemetry-seams --lib`
      (a `make check` row) -> after
      a tick, `last_tick_phases` holds the tick count and five phases in
      order, each elapsed equal to that tick's `phase_wall_clock`
      increment, each phase's offset plus elapsed at most the next phase's
      offset, and the last phase's end within the tick's elapsed;
      `cargo test -p v3-core --lib` still passes without the feature.
- [x] Neutrality and trace content: `cargo test -p v3-cli` (inside
      `make check`, no Docker) -> canonical NDJSON identical with telemetry
      off, on with the in-test receiver, and on with a closed port; with
      the interval `10`, every trace the receiver holds has six spans with
      the names, IDs and attributes of the table (`petri.phase` on every
      span), a `petri.tick` equal to
      a received snapshot's tick, and no two traces share a tick; a run
      shorter than the interval yields exactly one trace, the completion
      one, and a run of zero ticks none; with
      `PETRI_TELEMETRY_TICK_TRACES=off` snapshots arrive and no trace does.
- [x] Encoding and bounds: `cargo test -p v3-telemetry` -> a synthetic
      seam and config encode the six spans with the predicted trace and
      span IDs, timestamps, names and attribute keys, one shared trace ID,
      an empty parent on the root and the root's span ID as every phase's
      parent; an eight-type config
      encodes under 16 KiB; trace items past the queue bounds are dropped
      and counted; a partial success with rejected spans or an undecodable
      `200 OK` fails the trace with no bytes; the 65,536th trace is the
      last captured and `run.ended` carries `petri.tick_traces_capped`;
      a trace abandoned by the flush is counted for its run; an invalid
      `PETRI_TELEMETRY_TICK_TRACES` is refused.
- [x] Server: `cargo test -p v3-server` -> a trace accompanies an interval
      snapshot after `step`; an unsampled tick followed by a config patch
      and a resume yields a transition snapshot and no trace, and the next
      interval trace carries the new digest; reset and shutdown snapshots
      have no trace; the three-state payload neutrality test still passes.
- [x] Reference build: `cargo build --release -p v3-cli -p v3-server -p v3-lab
      --no-default-features` compiles, `cargo tree -p v3-cli -p v3-server
      -p v3-lab --no-default-features -e features` shows neither
      `v3-telemetry` nor `telemetry-seams`, and clippy `-D warnings`
      passes with and without the feature.
- [x] Dashboards (Docker, outside `make check`):
      `scripts/telemetry-dashboards-check` -> every row passes, including
      the four above; the `Tick traces` panel and one trace opened from it
      in a browser, recorded as a checklist row in the readings file.
- [x] Bytes per run: the workload run's `bytes=`, `snapshots=` and
      `traces=` figures and Tempo's growth over ten runs, in the readings
      file.
- [x] Parent comparison: `scripts/telemetry-parent-compare 6b027f2a` (F02's
      closing commit) -> identical canonical output, in the readings file.
- [x] Overhead check: `scripts/telemetry-overhead` -> the table in the
      readings file, verdict in Performance and Goal Impact.
- [x] `make check` -> exit 0.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: `Not applicable:
      observability feature`.
- [x] Benchmark summary: `Not applicable: observability feature`.

| Item | Result (2026-09-29; transcripts in the readings file) |
| --- | --- |
| Seam, neutrality, encoding, server | pass inside `make check` (both `v3-core --lib` rows); the server test runs every tick as a paused `step` with a 1 s interval, 5/5 repeat runs pass |
| Server free-running loop | `run_loop_emits_interval_traces_at_snapshot_ticks` (100 ms interval, polled to a 30 s deadline): the loop's interval trace sits at a snapshot tick of its run under the run's digest; 5/5 pass, and it fails with `run_loop`'s `after_tick` removed |
| Reference build | build exit 0; `cargo tree` 0 matches; clippy `-D warnings` exit 0 without the feature and in `make check` with it |
| Dashboards | every row PASS, `Tick traces` 4 rows; browser: panel lists the run's four traces, the span link opens the six-span trace |
| Bytes per run | `bytes=` 801,625–805,701, `snapshots=4 traces=4`; about 6.2 KB per trace; Tempo +380 KiB over ten runs |
| Parent comparison | `identical: 3 canonical lines, T=6908` |
| Overhead check | inconclusive: spread 11.05% then 11.93% (n = 0) on a loaded host |
| `make check` | exit 0 |

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary; the reviewer checks that against the diff, and
that every `v3-core` change sits behind `telemetry-seams` and every
`v3-cli`, `v3-server` and `v3-telemetry` change behind `telemetry`.
Measurement feature: the natural-analog rule and the environmental-pressure
rule do not apply, and no indicator can move.

Parent comparison: runs because F03 edits `run_tick`; `BASE` is
`6b027f2a`. Expected: identical canonical output.

Overhead check: runs because the seam adds work to every tick and a trace
adds capture and encoding to every snapshot tick. The method is F01's,
unchanged, with T = 6908 fixed:

| Item | Predeclared |
| --- | --- |
| Workload, reference, spread, pairs, states, reading, verdict, rerun, time cap, inconclusive | as the F01 spec's table: `--no-default-features` reference run with the stack stopped; n = 5 at spread ≤ 5%, 8 at ≤ 10%, else inconclusive; states (a) on, stack healthy, (b) on, stack stopped, (c) idle-stack block; pass at median ≤ 1.25 with ⌈0.75 n⌉ pairs ≤ 1.25; one rerun; 300 s cap; inconclusive is reported, never rounded |
| Configuration under test | the defaults: metrics interval unset, tick traces on; about 3 interval traces plus the completion one per 3.9 s workload run |
| Expected | (a) and (b) within noise of 1.0; self time below 3.5 µs per tick (F02's 0.46 µs plus five traces of under 1 ms over 6,908 ticks); the seam's unconditional cost is inside the ratio's noise |

**Measured verdict.**

| Record | Value |
| --- | --- |
| Profiles | `Not applicable: observability feature` |
| Parent comparison | identical canonical output against `6b027f2a`, T = 6908 |
| Overhead check | inconclusive: F01's method, T = 6908; both attempts (the second the one rerun) stopped at the spread measurement, 11.05% and 11.93% > 10%, so n = 0 and states (a) and (b) inconclusive, (c) without a reading; 24.6 s of the 300 s cap; host load average 6.6 to 8.8 |
| Self-timed cost per tick | 0.38–0.41 µs (ten traces-on workload runs, `self_time_us` 2,615–2,809) |

- Readings: [`docs/progress/readings/t21-f03.md`](../../progress/readings/t21-f03.md).

## Success Criteria

- [x] With traces on, the stack healthy, the queue unsaturated and the
      per-run cap not reached, a `v3-cli run --telemetry on` run of at
      least one tick and a `v3-server --telemetry on` run store in Tempo
      one trace per interval snapshot tick (and the CLI's completion
      tick), each with the `tick` span and five phase spans carrying the
      attributes of the table, fetchable by the ID composed from the run
      ID and the tick; with the stack stopped or the queue full the run
      keeps its speed and the missing traces are counted.
- [x] `Petri / Run` lists the run's tick traces and opens one, and
      `scripts/telemetry-dashboards-check` passes with its new rows.
- [x] The seam, neutrality, encoding, bounds and server tests pass inside
      `make check` without Docker; `v3-core` changes only behind
      `telemetry-seams`, and the reference build compiles it out.
- [x] The parent comparison is identical and the overhead check is
      recorded: inconclusive on a loaded host, accepted by the user (see
      Notes), with the self-timed cost per tick recorded.

## Notes for AI Agents

- Decision: tick traces ride the snapshot cadence; a separate trace
  cadence is T21.F05's to add if its measurements need one.
- Decision: trace IDs are composed from the run key and the tick, never
  drawn; a later trace kind (T21.F04) takes its own ID scheme that cannot
  collide with a tick's.
- Exception: the overhead check was inconclusive on both permitted
  attempts (spread 11.05% and 11.93%, n = 0, load average 6.6–8.8 from
  other containers); self time 0.38–0.41 µs per tick. The user accepted
  it on 2026-09-29; T21.F05 measures cost for the default.
- Cost: `/usage` awaiting; 3 passes (advisor 2, 2, 2); 0 spec-owner
  resumes; 3 Codex rounds, `ready`; review 1 P2, fixed.
