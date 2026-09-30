# T21.F02 — Run Metrics and Dashboards

**Status**: In Progress
**Last updated**: 2026-09-30
**Feature**: T21.F02
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

A run started with `--telemetry on` exports the cumulative counters `SimStats`
already keeps as complete per-run counts and its per-tick values as sampled
gauges, each snapshot carrying the tick it was taken at, through the bounded
exporter T21.F01 delivered. Two provisioned Grafana dashboards, `Petri /
Runs` and `Petri / Run`, list runs and show one run's history against wall
time and against ticks. With telemetry off, on, or on with no stack
listening, the run's deterministic output is identical.

## Non-Goals

- Tick and phase traces (T21.F03), creature windows (T21.F04), measurement
  command records (T21.F06), the qualified cadence and default-on (T21.F05),
  the template conventions (T21.F07).
- No new counter, no change to a T14 counter's definition or reset, no
  change to the CLI's NDJSON or the server's payloads.
- No Prometheus configuration change: the image's OTLP translation and
  resource-attribute promotion are used as they are.
- No alerting, no Grafana state that is not provisioned from source.

## Inputs and Invariants

| Input | Where | What F02 takes from it |
| --- | --- | --- |
| Track row and notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | Sampling contract (cumulative never sampled, `last_tick_*` are sampled gauges labelled as such, caps per wall second), run identity and cardinality (run ID the only high-cardinality metric attribute), time (metrics never carry the tick as an attribute; a tick gauge in the same copy), configuration (process-level settings only), failure behavior, storage, not-closure-evidence, ownership, boundary. |
| Benchmark gate | `docs/workflow.md`, "Benchmark gate" | Observability exemption; check triggers; interleaved pairs, spread-chosen pair count, inconclusive rule, 5 minutes of measured time, `scripts/bench-wait`; 25% ceiling; record format. |
| F01 spec | `docs/specs/roadmap/t21-f01-local-telemetry-stack-and-run-identity.md` | The flag, `PETRI_TELEMETRY`, the endpoint, the reference build (`--no-default-features`, `v3-core/telemetry-seams`), the bounded queue (2,048 items, 8 MiB, 4 MiB body, 512 per batch, 5 s request, 10 s flush, no retry), exact per-run counts and the stderr line, the identity attributes, `run.started`, the three check commands, workload T = 6908. |
| Research note | `docs/strategy/run-observability-research-2026-09-29.md` | Signals table (counters complete, per-tick sampled, censuses on their own cadence), time and cardinality rules, "plot against ticks untested". |
| Counters | `crates/v3-core/src/simulation/stats.rs` `SimStats` (`Clone`), `energy_accounting.rs` `MortalityTotals`, `EnergyFlows`, `reproductive_success.rs`, `mutation/types/mod.rs` | The field list in the table below; `reset_tick_counters` clears only `last_tick_*`; every map key enum has `as_key()`. |
| Census | `crates/v3-cli/src/lib.rs` `build_tick_sample`, `Simulation::mean_energy`, `Simulation.creatures` | Population, mean energy, mean genome size, mean mesh nodes, mean generation: the same expressions over the public fields, so a snapshot's census equals the `tick_sample` of the same tick. |
| Run loops | `crates/v3-cli/src/lib.rs` `run_simulation`; `crates/v3-server/src/http/lifecycle.rs` `run_loop`, `step`, `startup`; `crates/v3-server/src/telemetry.rs` | Where snapshots are taken; the server ends a run at `startup` (reset) and at shutdown. |
| Exporter | `crates/v3-telemetry/src/{lib,queue,export,testing}.rs` | Queue entries are boxed log records; the worker posts one run's batch at a time; `CountingClient` decodes partial success; the in-test receiver decodes `POST /v1/logs`. |
| Image | `grafana/otel-lgtm:0.34.0`: Prometheus 3.14.0, Grafana 13.2.2, collector 0.161.0 (probed 2026-09-30 on a scratch project) | `otlp.translation_strategy: UnderscoreEscapingWithSuffixes`: dots become underscores, a monotonic sum gains `_total` (not doubled), unit `s` gives `_seconds`; data-point attributes become labels; `service.name` and `service.version` are promoted to `service_name`, `service_version` and `job`, every other resource attribute lands only on `target_info`; dashboard providers are read from `/otel-lgtm/grafana/conf/provisioning/dashboards/*.yaml`, datasource UIDs are `prometheus`, `loki`, `tempo`; anonymous access is Admin. |
| Proto crate | `opentelemetry-proto` 0.33.0 | Feature `metrics` gates `collector::metrics::v1` and `metrics::v1`; `ExportMetricsServiceResponse.partial_success.rejected_data_points`. |

Invariants:

1. No `SimulationConfig` or `RuntimeConfig` field, no production RNG draw, no
   simulation default, no stored summary content and no deterministic output
   changes; the config digest and the pinned recipe digests do not move.
   `v3-core` is not edited: the census is computed in `v3-telemetry` from
   `Simulation`'s public fields with the CLI's expressions, and the
   equality test below keeps the two in step.
2. Everything F02 adds to the binaries sits behind their `telemetry` feature;
   `--no-default-features` is still the reference build.
3. A snapshot is a bounded copy of the exported fields only (the table
   below; not `SimStats` whole), the census, the tick and the elapsed wall
   seconds, taken in one pass on the simulation thread, encoded there as one
   OTLP `ExportMetricsServiceRequest` and offered to F01's queue as one
   item attributed to its run. The queue is one FIFO of records and
   snapshots; its bounds, the body cap, the one batch in flight, the
   drop-when-full rule and the flush bounds apply to snapshot items exactly
   as to records. The worker takes a snapshot item as a batch of one and
   posts it to `<endpoint>/v1/metrics`. Accounting is per item: a snapshot
   is `exported` when the receiver accepted it with zero rejected data
   points, `failed` when the request failed or any data point was rejected
   (its bytes are then not counted), `dropped` or `abandoned` as a record
   is.
4. Cadence, as hard caps: at most one snapshot per tick and, for the
   interval snapshots, at most one per `PETRI_TELEMETRY_METRICS_INTERVAL_MS`
   of wall time (default `1000`; `10` to `3600000`; read only when telemetry
   resolves on, and then anything else refuses to start, as an invalid
   `--telemetry` does; `v3-lab` reads it once T21.F06 gives it an exporter),
   so the export rate is bounded
   by wall time whatever the tick rate. After a tick, an interval snapshot is
   taken when the interval has passed since the run's last snapshot. A
   server `run.state` transition takes one when the tick has none and at
   least 10 ms have passed since the last snapshot; the end of a run (CLI
   completion, server reset and shutdown) takes one when the tick has none,
   from the ending run's simulation before it is replaced, so the last
   exported counts are the run's final counts. "Taken" means captured,
   whether or not the queue later drops it. The setting is process-level and
   is recorded on `run.started` as `petri.metrics_interval_ms`.
5. Time: a snapshot's stamp is the later of its capture wall time and the
   previous snapshot's stamp plus one millisecond (Prometheus's
   resolution), a non-blocking clamp that never delays capture and absorbs
   a clock that steps back; cumulative sums carry the run's start wall
   time. The tick is a gauge in the same snapshot, never an attribute.
6. Attributes: every data point carries `petri.run_id`; breakdown maps add
   the one or two attributes in the table, whose values are the key's
   `as_key()` string (a food type or vector index is its decimal index), so
   names match the stored benchmark summaries. No
   creature ID, tick, lineage ID, genome hash, seed or config value is a
   metric attribute; the run's `run.started` record carries those.
7. Names: an OTel name is `petri.run.<field>` for a cumulative value, with
   every `_total` token removed from the field name and, where the table
   says so, a `_by_<key>` or `_sum` suffix dropped because the attribute or
   the family already names it (`mutation_operator_funnel`,
   `reproductive_success.<x>`); the table is authoritative; and
   `petri.tick.<field>` for a sampled per-tick value, the `last_tick_`
   prefix dropped and the rest verbatim (`compute_total_mean` keeps its
   name); Prometheus appends `_total` to monotonic sums. `u64` counters and `Duration`s are monotonic sums; every
   `f64` sum is a non-monotonic sum (`EnergyFlows` is documented as signed
   changes), so its Prometheus name has no `_total` and dashboards use
   `delta`/`deriv`, not `rate`, on it. Sampled gauges say `sampled at the
   snapshot tick` in their description.
8. The stderr run line gains `snapshots=<taken>` at its end; `exported`,
   `failed`, `dropped`, `abandoned` and `bytes` count snapshot items with
   the records, and `self_time_us` includes the copy and the encoding.
9. Dashboards are source under `telemetry/grafana/`, provisioned read-only
   (`allowUiUpdates: false`; a UI edit is kept only as a saved copy) with
   fixed UIDs `petri-runs` and `petri-run` in folder `Petri`, and reference
   datasources by the image's UIDs. Nothing they show is closure evidence.

**Metric families.** Every cumulative field of `SimStats` is exported;
excluded are `mutation_outcome_baseline_by_generation_bucket` and
`mutation_outcome_baseline_global` (running statistics with an unbounded
key set, not counters) and `last_tick_predation_events` (per-event records,
T21.F04's material).

| Family (OTel name) | Prometheus name | Source | Attributes | Kind |
| --- | --- | --- | --- | --- |
| `petri.run.tick` | `petri_run_tick` | `Simulation.tick` | — | gauge |
| `petri.run.elapsed_seconds` | `petri_run_elapsed_seconds` | wall time since `run.started` | — | gauge |
| `petri.run.<field>` for the 31 `u64` scalars of `SimStats` (`reproduction_actions_*`, `mutation_events_*`, `move_actions_attempted`, `mutation_*_target`, `mesh_hops` … `actions_applied`, `predation_*`) | `petri_run_<field>_total` | the field | — | monotonic sum |
| `petri.run.deaths` | `petri_run_deaths_total` | `mortality.deaths_total` | — | monotonic sum |
| `petri.run.deaths_by_cause` | `petri_run_deaths_by_cause_total` | `mortality.by_cause` | `petri.cause` (18) | monotonic sum |
| `petri.run.reproductive_success.<creatures_observed\|offspring_spawned\|survival_ticks>` | `petri_run_reproductive_success_<x>_total` | `reproductive_success_by_cognitive_class.by_class` | `petri.cognitive_class` (4) | monotonic sum |
| `petri.run.energy_flow` | `petri_run_energy_flow` | `energy_flows` `f64` fields; `action_charges.<action>` as `action_charge.<action>`; `food_intake_by_type[i]` as `food_intake.<i>` | `petri.flow` (≤ 27) | non-monotonic sum |
| `petri.run.genome_size_creature_ticks` | `petri_run_genome_size_creature_ticks_total` | `energy_flows.genome_size_creature_ticks` | — | monotonic sum |
| `petri.run.phase_wall_clock` (unit `s`) | `petri_run_phase_wall_clock_seconds_total` | `phase_wall_clock` | `petri.phase` (5) | monotonic sum |
| `petri.run.mutation_events_<attempted\|applied>_by_domain` | `…_by_domain_total` | the two `by_domain` maps | `petri.domain` (4) | monotonic sum |
| `petri.run.mutation_events_<attempted\|applied\|skipped>_by_operator` | `…_by_operator_total` | the three `by_operator` maps | `petri.operator` (55) | monotonic sum |
| `petri.run.mutation_events_skipped_by_reason` | `…_by_reason_total` | `mutation_events_skipped_by_reason` | `petri.skip_reason` (3) | monotonic sum |
| `petri.run.mutation_operator_funnel` | `petri_run_mutation_operator_funnel_total` | `mutation_operator_funnel_total_by_operator` | `petri.operator`, `petri.stage` (5) | monotonic sum |
| `petri.run.mutation_skip_reasons_by_operator` | `…_total` | `mutation_skip_reasons_total_by_operator` | `petri.operator`, `petri.skip_reason` | monotonic sum |
| `petri.run.mutation_added_node_input_classes_by_operator` | `…_total` | `mutation_added_node_input_classes_total_by_operator` | `petri.operator`, `petri.input_class` (9) | monotonic sum |
| `petri.run.mutation_added_node_world_inputs_by_operator` | `…_total` | `mutation_added_node_world_inputs_total_by_operator` | `petri.operator`, `petri.world_input` (10) | monotonic sum |
| `petri.run.mutation_value.<field>` (20 fields) | `…_total` (`u64`) or `petri_run_mutation_value_<field>` (`f64`) | `mutation_value_totals_by_operator` | `petri.operator` | monotonic (`u64`) or non-monotonic (`f64`) sum |
| `petri.run.mutation_outcome_summary.<field>` (20 fields) | as above | `mutation_outcome_summary` | — | as above |
| `petri.run.reproduction_actions_rejected_by_reason` | `…_total` | `reproduction_actions_rejected_by_reason` | `petri.reason` (5) | monotonic sum |
| `petri.run.reproduction_actions_rejected_invalid_target_by_cause` | `…_total` | `…_invalid_target_total_by_cause` | `petri.cause` (4) | monotonic sum |
| `petri.run.<field>_by_reader_state` (6 maps) | `…_total` | the six `BarrierReaderState` maps | `petri.reader_state` (2) | monotonic sum |
| `petri.run.eat_actions_<applied\|failed>_by_type` | `…_total` | the two `by_type` maps | `petri.food_type` | monotonic sum |
| `petri.run.move_actions_blocked_by_cause` | `…_total` | `move_actions_blocked_total_by_cause` | `petri.cause` (3) | monotonic sum |
| `petri.run.predation_actions_by_result` | `…_total` | `predation_actions_by_result` | `petri.result` (3) | monotonic sum |
| `petri.tick.actions` | `petri_tick_actions` | `last_tick_{move,eat,noop,reproduce,steal}` | `petri.action` (5) | sampled gauge |
| `petri.tick.predation_kills` | `petri_tick_predation_kills` | `last_tick_predation_kills` | — | sampled gauge |
| `petri.tick.<field>` for `compute_total_{mean,min,max}`, `compute_vm_mean`, `compute_graph_mean`, `priority_bid_mean`, `priority_bidders_count`, `food_occupancy_depletion_mean`, `food_occupancy_depletion_occupied_cells`, `food_growth_suppressed_by_occupancy_depletion`, `food_cells_with_type_inhibition`, `food_growth_suppressed_by_type_inhibition` | `petri_tick_<field>` | `last_tick_<field>` | — | sampled gauge |
| `petri.tick.food_<total_density\|grazing_modifier_mean\|grazed_cell_share>` | `petri_tick_food_<x>` | the three `*_by_type` vectors | `petri.food_type` | sampled gauge |
| `petri.tick.<population\|mean_energy\|mean_genome_size\|mean_mesh_nodes\|mean_generation>` | `petri_tick_<x>` | the census | — | sampled gauge |

Only keys present in a map are exported, so the series per run are bounded
by the enum sizes: the operator-keyed families allow 55 × (3 + 5 + 3 + 9 +
10 + 20) = 2,750 series and the rest about 150, so at most about 2,900
cumulative series and 35 sampled gauges per run, typically several hundred.
Projected size: ≤ 180 B per data point (`petri.run_id` about 50 B, an
operator attribute up to about 55 B, a second attribute about 30 B, two
timestamps, the value and protobuf framing) plus about 60 B of metric
metadata per family, so ≤ 560 KB per snapshot and, at the default cadence,
≤ 34 MB per minute on the wire; Prometheus stores samples, not payloads,
about 15 MB per hour of a running run at the worst-case series count. The
measured `bytes=` figure of the workload run and each store's growth over
ten runs go in the readings file.

**Stack.** `telemetry/compose.yaml` mounts `telemetry/grafana/dashboards.yaml`
at `/otel-lgtm/grafana/conf/provisioning/dashboards/petri.yaml` and
`telemetry/grafana/dashboards/` at `/otel-lgtm/petri-dashboards`, both
read-only; `telemetry/verify/compose.yaml` is unchanged.

The data volume is declared `external: true`, so no `docker compose down
-v`, on any project, can remove it: `make telemetry-up` creates
`petri-telemetry` with `docker volume create` when it is missing, and every
scratch project (`telemetry-verify`, `telemetry-overhead`,
`telemetry-dashboards-check`) creates and removes its own named volume
explicitly (incident in the readings file).

| Dashboard | Panels (every query filtered to the run) |
| --- | --- |
| `petri-runs`, `Petri / Runs`, default range 7 days | Runs started (Loki table of `run.started`: time, `service_name`, `petri_run_id`, `petri_seed`, `petri_world`, `petri_recipe`, `petri_build_revision`, `petri_ticks_requested`, `petri_metrics_interval_ms`; each row links to `/d/petri-run?var-run_id=<id>`); Runs ended (Loki table of `run.ended`: time, run ID, `petri_status`, `petri_tick`, `petri_wall_seconds`); Runs with metrics in range (Prometheus table `max by (petri_run_id, service_name) (petri_run_tick)`, linked the same way) |
| `petri-run`, `Petri / Run`, variable `run_id` from `label_values(petri_run_tick, petri_run_id)` | Identity (Loki logs panel: the run's `run.started`, `run.state`, `run.config`, `run.ended`); against wall time (timeseries): tick and tick rate (`deriv` of `petri_run_tick`), population, mean energy, structure means, deaths by cause (rate), births (rate of `reproduction_actions_spawned`), mutation events attempted/applied/skipped (rate), energy flows by `petri_flow` (`delta`, stacked), phase wall clock share (rate per phase over the summed rate), actions per tick (sampled), compute means (sampled), food density by type (sampled); against ticks (xychart, x = `petri_run_tick`): population, mean energy, mean genome size, deaths, mutation events applied |

A panel whose series are sampled gauges says `(sampled)` in its title.

**Check command.** `scripts/telemetry-dashboards-check` (POSIX `sh`,
`scripts/bench-wait` around the run) prints one pass/fail line per row and
removes its scratch volume at the end.

| Step | What it does or asserts |
| --- | --- |
| Stack | Compose project and scratch volume `petri-telemetry-dashboards`, created and removed explicitly; refuses while `petri-telemetry` is up |
| Workload | `v3-cli run --telemetry on --seed 7 --ticks T --sample-every T --config telemetry/overhead-world.json`, T from the F01 readings line |
| Provisioning | both dashboards present in folder `Petri` via `/api/dashboards/uid/` on `127.0.0.1:3300` |
| Names | every scalar family in the table above, and every map family with a key on the workload, appears under its Prometheus name in `/api/datasources/proxy/uid/prometheus/api/v1/label/__name__/values`; the names are listed in the script |
| Panels | every panel query of both dashboards, read from the JSON with `node` (Aqua) and evaluated through `/api/ds/query` with `$run_id` substituted by the run's ID and a range from one minute before the run to now, returns data |
| Tick gauge | `petri_run_tick` for the run has at least three samples with distinct timestamps, non-decreasing, ending at T |
| Run record | the Loki `run.started` line of the run carries `petri_metrics_interval_ms` |
| Volume survives | after the script's own `down -v`, `petri-telemetry` still exists when it existed at the start |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.*` cumulative families (table above) | monotonic and non-monotonic cumulative sums | `SimStats` copied after a tick | `petri.run_id`; breakdown key; resource identity (`service.name`, `service.version` as labels, invocation ID and build revision on `target_info`) |
| `petri.run.tick`, `petri.run.elapsed_seconds` | gauges | the same copy | `petri.run_id` |
| `petri.tick.*` (table above) | sampled gauges | `last_tick_*` and the census in the same copy | `petri.run_id`; description `sampled at the snapshot tick` |
| `run.started` | log record (F01) | gains `petri.metrics_interval_ms` | the cadence in force |
| exporter self-report | stderr line (F01) | gains `snapshots=` | snapshots taken |

The config digest and the mechanism switches in force are on the run's
`run.started` record, which a dashboard reaches by run ID; metrics carry no
config value. Snapshots consume no production RNG and change no execution;
a value that was not captured is absent, never zero.

## Implementation Tasks

- [x] `crates/v3-telemetry`: `PETRI_TELEMETRY_METRICS_INTERVAL_MS` in
      `Options`; the snapshot copy and census; OTLP metrics encoding of the
      families above; snapshot items in the queue with `/v1/metrics` posting,
      partial-success accounting and `snapshots=`; `petri.metrics_interval_ms`
      on `run.started`; the in-test receiver decodes `POST /v1/metrics`.
- [x] `v3-cli`: snapshot hook in `run_simulation` after each tick and at
      completion; `v3-server`: hooks in `run_loop`, `step`, transitions,
      `startup` and shutdown.
- [x] `telemetry/grafana/dashboards.yaml`, `telemetry/grafana/dashboards/
      {petri-runs,petri-run}.json`, the compose mounts.
- [x] `scripts/telemetry-dashboards-check`.
- [x] External data volume: `external: true` in `telemetry/compose.yaml`,
      `docker volume create` in `make telemetry-up`, explicit scratch
      volumes in the three scripts (`scripts/telemetry-scratch.sh`), the
      volume-survives check.
- [x] Run the checks on this host; record them in
      `docs/progress/readings/t21-f02.md`.

## Verification

- [x] Neutrality and snapshot content: `cargo test -p v3-cli` (inside
      `make check`, no Docker) -> canonical NDJSON identical with telemetry
      off, on with the in-test receiver, and on with a closed port; with the
      interval `10`, the receiver holds at least two snapshots, ticks and
      timestamps strictly increasing by at least 1 ms, the last one at the
      tick count with the census, the six cumulative scalars and the four
      maps that `TickSampleEvent` carries equal to the final `tick_sample`;
      a run whose ticks all fall inside one interval still yields exactly
      one snapshot, the run-end one.
- [x] Encoding and bounds: `cargo test -p v3-telemetry` -> a `SimStats` with
      every map key populated encodes every family in the table with the
      predicted OTel names, kinds, attribute keys and `as_key()` values and
      no other series, so the families `tick_sample` does not carry are
      covered here; snapshot items past the queue bounds are dropped and
      counted; a partial success with rejected data points counts the
      snapshot as failed with no bytes; a snapshot and records of one run
      queued together are attributed to that run across a reset; an
      invalid interval is refused.
- [x] Server: `cargo test -p v3-server` -> a snapshot at a transition, none
      for a second transition at the same tick, one at reset from the ending
      run at the `run.ended` tick, and one at shutdown; the three-state
      payload neutrality test still passes.
- [x] Reference build: `cargo build --release -p v3-cli -p v3-server -p v3-lab
      --no-default-features` compiles, `cargo tree -e features` shows neither
      `v3-telemetry` nor `telemetry-seams`, clippy `-D warnings` passes with
      and without the feature. Passed 2026-09-30 (tree: 0 matching lines).
- [x] Dashboards (Docker, outside `make check`):
      `scripts/telemetry-dashboards-check` -> every check passes; transcript
      and the stored Prometheus names in the readings file; the two
      dashboards opened in a browser on the same run, with the tick-axis
      panels plotting, recorded as a checklist row there; the
      volume-survives line passes with `petri-telemetry` present, and
      `scripts/telemetry-verify` still exits 0 on its own scratch volume. Passed 2026-09-30.
- [x] Bytes per run: the workload run's `bytes=` and `snapshots=` figures and
      each store's growth over ten runs, in the readings file.
- [x] Parent comparison: `scripts/telemetry-parent-compare 5c14c8d3` (F01's
      closing commit) -> identical canonical output, in the readings file.
- [x] Overhead check: `scripts/telemetry-overhead` -> the table in the
      readings file, verdict in Performance and Goal Impact.
- [x] `make check` -> exit 0 (2026-09-30; runs the three test rows above).
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: `Not applicable:
      observability feature`.
- [x] Benchmark summary: `Not applicable: observability feature`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary; the reviewer checks that against the diff, and
every change to `v3-cli`, `v3-server` and `v3-telemetry` sits behind the
`telemetry` feature. Measurement feature: the natural-analog rule and the
environmental-pressure rule do not apply, and no indicator can move.

Parent comparison: runs because F02 edits `run_simulation` and the server's
run loop; `BASE` is `5c14c8d3`. Expected: identical canonical output.

Overhead check: runs because a snapshot adds work to the run loop. The
method is F01's, unchanged, with T = 6908 fixed:

| Item | Predeclared |
| --- | --- |
| Workload, reference, spread, pairs, states, reading, verdict, rerun, time cap, inconclusive | as the F01 spec's table: `--no-default-features` reference run with the stack stopped; n = 5 at spread ≤ 5%, 8 at ≤ 10%, else inconclusive; states (a) on, stack healthy, (b) on, stack stopped, (c) idle-stack block; pass at median ≤ 1.25 with ⌈0.75 n⌉ pairs ≤ 1.25; one rerun; 300 s cap; inconclusive is reported, never rounded |
| Cadence under test | the default, `PETRI_TELEMETRY_METRICS_INTERVAL_MS` unset: about 4 interval snapshots plus the final one per 3.9 s workload run |
| Expected | (a) and (b) within noise of 1.0; self time below 3 µs per tick (five snapshots of at most a few milliseconds over 6,908 ticks) |

**Measured verdict.**

| Record | Value |
| --- | --- |
| Profiles | `Not applicable: observability feature` |
| Parent comparison | method above, `BASE` `5c14c8d3`, T = 6908: identical canonical output |
| Overhead check | method above, default interval, n = 8 (spread 6.09%), T = 6908; measured time 150.9 s of 300 s, one attempt |
| (a) on, stack healthy | median ratio 1.001, 8 pairs, pass; self time 0.463 µs per tick |
| (b) on, stack stopped | median ratio 0.994, 8 pairs, pass; self time 0.371 µs per tick |
| (c) idle-stack cost | 1.016 |

- Readings: [`docs/progress/readings/t21-f02.md`](../../progress/readings/t21-f02.md).

## Success Criteria

- [ ] A `v3-cli run --telemetry on` run and a `v3-server --telemetry on` run
      appear in Prometheus with every scalar family in the table and every
      map family with a key, each series carrying `petri_run_id`, and the
      last snapshot of a run holds its final counts.
- [ ] `Petri / Runs` lists runs with identity and links to `Petri / Run`,
      which shows the selected run against wall time and against ticks, and
      `scripts/telemetry-dashboards-check` passes, including the line that
      shows a scratch project's `down -v` leaves `petri-telemetry` in place.
- [ ] The neutrality, encoding, bounds and server tests pass inside
      `make check` without Docker; the interval setting is process-level and
      recorded on `run.started`.
- [ ] The parent comparison is identical and the overhead check passes in
      states (a) and (b), with the idle-stack cost and the self-timed cost
      per tick recorded.

## Notes for AI Agents

- Decision: F02 exports every cumulative `SimStats` field and no more; a
  counter a later mechanism feature adds ships its own family under the
  naming rule above (track ownership note, applied 2026-09-30).
- Decision: keys sharing an `as_key()` string (`WorldInputKey` across food
  types) are summed into one series.
- Decision: the interval variable is read only when telemetry is on.
- Decision: the interval counts from `run.started`; a run's first
  transition snapshot needs no 10 ms gap.
- Decision: `reproductive_success.<x>` names follow the table; other
  `_sum` fields keep `_sum`.
- Decision: the check script lists the 36 map families the workload keys.
