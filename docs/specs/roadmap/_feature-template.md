# T01.F01 — <Feature title>

**Status**: Planned
**Last updated**: <YYYY-MM-DD>
**Feature**: T01.F01
**Track**: [T01 — <Track title>](../../roadmaps/t01-<track-slug>.md)

**A spec records the state of the world at closure, not the path taken to reach
it.** Write for the next process that reads this file — a later feature's Plan
step, a reviewer, you in a month — and give it only what it is bound by. Fold
each outcome into the section it changes, in the present tense. Delete text a
later pass superseded rather than annotating it; the history is in git.

There is no readiness-review log, no implementation-deviation log, and no
pass-by-pass narrative. `scripts/roadmap-check.mjs` enforces a **15 KB budget of
non-table prose** per spec: tables and fenced blocks are free, narration is not.
Measured summaries live in `docs/progress/features/<id>.json` (machine-written)
and concise readings in `docs/progress/readings/<id>.md`. Full gate/goal/sweep
reports stay in the main checkout's ignored `.bench-artifacts/<id>/`;
[`docs/benchmark-artifacts.md`](../../benchmark-artifacts.md) defines paths,
provenance and conversion. New full reports are never committed.

## Goal

State the independently observable feature outcome.

## Non-Goals

- State explicitly excluded work.

## Inputs and Invariants

List source-of-truth inputs, decision-relevant research evidence, exact
dependency outputs, and invariants. The owning roadmap row is the source of
truth for feature dependencies.

## Telemetry

State what a person looking at a run of this feature can see, or
`Not applicable: <reason>` when the feature changes nothing a run does. Use
one table; tables are free under the prose budget.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `<name>` | counter, gauge, span, span event or log record | the applied behavior it derives from | the config values and mechanism switches that govern it |

Conventions T21 delivered. `From` names the source: a closed T21 spec
([F01](t21-f01-local-telemetry-stack-and-run-identity.md),
[F02](t21-f02-run-metrics-and-dashboards.md),
[F03](t21-f03-tick-and-phase-tracing.md),
[F04](t21-f04-creature-cognition-windows.md),
[F05](t21-f05-qualified-sampling-defaults.md),
[F06](t21-f06-benchmark-assay-and-lab-run-records.md); "inv." is one of its
invariants, and its attribute and family tables are authoritative), the
[T21 track](../../roadmaps/t21-run-observability-and-execution-tracing.md),
`AGENTS.md` or `docs/workflow.md`. "Spec" means the feature spec that adds
the thing.

| Convention | Rule | From |
| --- | --- | --- |
| Metric names | `petri.run.<field>` for a cumulative value (`u64` and `Duration`: monotonic sum; `f64`: non-monotonic sum), with every `_total` token removed from the field name and a `_by_<key>` or `_sum` suffix dropped where F02's family table, which is authoritative, says so; `petri.tick.<field>` for a sampled per-tick value (gauge, description `sampled at the snapshot tick`), the `last_tick_` prefix dropped and the rest verbatim (`compute_total_mean` keeps its name). No unit suffix in the OTel name: Prometheus turns dots to underscores and appends `_total` to a monotonic sum and `_seconds` to unit `s`. | F02 inv. 7 |
| Metric attributes | `petri.run_id` on every data point; a breakdown adds at most two attributes whose values are the key's `as_key()` string, a food type's or vector index's decimal index, a composed flow name (`petri.flow`) or a fixed literal set (`petri.stage`, `petri.phase`, `petri.action`), so series are bounded by each family's finite key set; F02's family table is authoritative for each family's keys. No tick, creature ID, lineage ID, genome hash, seed or config value on a metric; `service.name` and `service.version` are labels, other resource attributes are on `target_info`. | F02 inv. 6 and Telemetry table |
| Wiring a counter | A numeric cumulative counter added to `SimStats` or its sub-structs is exported by the same feature: a family in `crates/v3-telemetry/src/metrics.rs`, its row in the expected families of `crates/v3-telemetry/src/metrics/tests.rs`, its Prometheus name in `scripts/telemetry-dashboards-check` (`always_names` when every run has it, `keyed_names` when the check workload gives its map a key), and a row in the spec's Telemetry table. A numeric `last_tick_*` scalar or per-type vector is a `petri.tick.*` gauge. Per-event records and seams (`last_tick_predation_events`, `last_tick_phases`) are trace material, not metrics; a running statistic with an unbounded key set is not exported; a measurement command exports records, not metrics. | F02, F03 inv. 2, F06 inv. 5 |
| Record attributes | Resource: `service.name`, `service.version`, `process.pid`, `petri.invocation_id`, `petri.build_revision`. Simulation-run records (`run.*`, `creature.genome`): `petri.run_id`, `petri.seed`, `petri.world`, `petri.recipe` (when given), `petri.config_digest`, `petri.tick`, `event.name`; `run.started` carries the telemetry settings in force (`petri.preset`, `petri.metrics_interval_ms`, `petri.tick_traces`, `petri.creature_windows`, `petri.window_ticks`, `petri.window_interval_ms`) and the full effective config as its body, so a config field reaches it with no edit; `run.ended` carries `petri.status`, `petri.tick`, `petri.wall_seconds`, the cap attributes, and `petri.total.<field>` for a benchmark seed. Measurement records: both carry the resource set, the measurement's own `petri.run_id`, `petri.tick` 0 and `event.name`; `measurement.started` adds `petri.command`, `petri.feature`, `petri.profile` or `petri.assay`, the seeds and `petri.config_digest` when one config applies; `measurement.ended` adds `petri.exit_code`, `petri.wall_seconds`, `petri.summary_path`, the raw identity and the totals body; an attribute that does not apply is omitted (F06's records table is authoritative). | F01, F02–F06 |
| Span attributes | Every span: `petri.run_id`, `petri.seed`, `petri.world`, `petri.recipe`, `petri.config_digest`, the resource attributes, `petri.tick`, `petri.phase` and `petri.sample_policy` (`interval`, `run_end`, `window`, `manual`); a window span adds `petri.window` and `petri.creature_id`. On a tick span `petri.tick` is the count after the tick and the digest the one its tick ran under; the `creature_window` root carries its first recorded tick (the ending tick when none), `petri.tick_end`, the start digest and `petri.config_changed` when a patch landed. A phase span carries the config values that govern it as `petri.config.<dotted config path>`. A mechanism switch or config value a feature adds is recorded configuration: a `SimulationConfig` or `RuntimeConfig` field reaches the `run.started` body with no edit, a telemetry setting is a `run.started` attribute, and a value that governs a traced phase also goes in that phase's group in `crates/v3-telemetry/src/trace.rs` (`phase_attributes` over `PHASES`: the `world_update`, `cognition` and `actions` functions, `sensor_assembly` and `reward_learning` inline), which the `creature_window` root reuses for cognition and actions; an initialization-only value and a measurement command, which emits no span, need no phase group. Tempo indexes attributes as `span.<name>`. | F03 inv. 6, F04, F05, F06 |
| Absent, never zero | A value that was not captured is omitted on records, spans and metrics; a map key not present exports no series. | F01–F04 |
| Sampling and caps | Cumulative counts are complete and never sampled. Per-tick values are sampled gauges, labelled as such and bounded by the snapshot cadence: at most one snapshot per tick and one per `PETRI_TELEMETRY_METRICS_INTERVAL_MS` (default 1,000 ms), plus the transition and run-end snapshots. Policy sampling applies to policy-sampled observations (spans, window events, window and genome bodies); the boundary records are unsampled and exempt: `run.started` and `run.config` with their full config bodies, `run.state`, `run.ended`, `measurement.started`, and `measurement.ended` with its totals body. A policy-sampled observation (not a cadence-bounded gauge) names its policy, its per-run cap and its projected size per run in the spec before any run, and its taken count on the stderr run line; reaching a cap stops capture and is recorded on `run.ended` (`petri.tick_traces_capped`, `petri.windows_capped`). Caps in force: 65,536 tick traces per run; 4,096 samples per run, 2,048 events and 4 MiB of retained record memory per window, 256 MiB of encoded window and genome bytes per run; queue 2,048 items and 8 MiB, 4 MiB per body, 512 records per batch, one batch in flight, 5 s per request, no retry, drop when full, 10 s flush. | F01, F02 inv. 4, F03 inv. 7, F04 inv. 8, F06, AGENTS.md |
| Trace identity | A trace ID is the run key's high 64 bits followed by a 64-bit discriminator with no entropy: the tick for a tick trace (not captured at or above `2^63`), `2^63 + k` for sample `k`, so the two never collide; a trace kind a feature adds states its discriminator and that it collides with neither. Span IDs are fixed small integers; timestamps are offsets from the run's start `Instant`. | F03 inv. 4, F04 inv. 5 |
| Determinism | Signals derive from applied behavior, draw no production RNG and change no execution; no telemetry setting enters the config digest; the deterministic output each command defines (the CLI's canonical NDJSON, `bench`'s `deterministic` blocks, the lab's `summary.json` minus `timing`) is identical with telemetry on, off and with the stack stopped, and wall-clock readings are never promised identical. Telemetry capture, export and observation seams sit behind the binaries' `telemetry` feature and, in `v3-core`, behind `telemetry-seams`, so `--no-default-features` stays the reference build; a behavior-preserving restructuring that gives a capture point a place to sit may be unconditional. | F01 inv. 2 and 5, F06 inv. 1 and 7–8, workflow |
| Settings and presets | A sampling setting is a `PETRI_TELEMETRY_*` variable read only when telemetry resolves on, refuses to start when invalid and is recorded on `run.started`; `PETRI_TELEMETRY_PRESET` (`minimal`, `phases`, `standard` the default, `dense`, in `crates/v3-telemetry/src/preset.rs`) names the base value of every sampling setting, so a setting a feature adds gets one per preset; an explicit variable overrides the preset for that setting; presets change no cap. | F04 inv. 3, F05 inv. 3 |
| Ceiling | The ceiling in force is 10% (ratio 1.10 in `scripts/telemetry-overhead`), 5% the target for the default; a feature that adds work to a run, telemetry on or off, runs the overhead check under the workflow's method; `dense` is opt-in and expected over the ceiling. | F05 inv. 5, workflow |
| Default and tests | Telemetry is on by default in every binary. Every test that spawns a binary passes `--telemetry off`; a telemetry fixture passes `on` and scrubs `PETRI_TELEMETRY*` and the endpoint from the spawned environment. Closure benchmarks export and still write their stored reports. A benchmark, assay or lab command captures only at region boundaries and exports after its last timed region. | F05 inv. 2, F06 inv. 2–3 |
| Not closure evidence | Nothing the stack stores is an indicator, a default or closure evidence; `Not applicable: <reason>` is the form for a feature that changes nothing a run does. | track |

## Implementation Tasks

- [ ] Implement the feature.

## Verification

Each item names *what* is checked and *where the result lives*: a command, a test
name, a report path, or a link. Do not prescribe test design in prose — the
implementer chooses the design and records what it actually ran.

- [ ] Focused tests or checks: `<command>` -> `<result>`.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path, and
      every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here; `docs/workflow.md` requires it in the spec.
      A T21 feature under the workflow's observability exemption records
      `Not applicable: observability feature`.
- [ ] Benchmark summary stored at `docs/progress/features/<id>.json`, local raw
      hash/byte count and verification time checked, series entry points to the
      summary, and no new full report staged; or
      `Not applicable: <reason>`.

Keep this section under about 3 KB excluding the survivor list. Pass-by-pass
logs, transcripts, and raw test output go to `docs/progress/readings/<id>.md`,
linked from the item that produced them.

## Performance and Goal Impact

**Predeclaration — written before the run.** This is the scientific contract: it
prevents post-hoc rationalization, so it is never edited after measuring. For a
mechanism feature, name its natural analog and how it reaches creatures through
the world or body. Predeclare and justify any expected compute cost, the
references the gate and goal profiles are compared against, the thresholds that
apply, and the expected direction — or the explicit absence of one — for every
indicator this feature can move. A justified cost re-pins the epoch baseline in
this feature's closing commit. If this feature introduces a diversity or
cognition measure, wire its indicator into the goal profile here or state that it
remains `Undefined` and why.

**Measured verdict.** One line per profile: CLI and observed outer-process exit
statuses with their sources, the `severe` flag,
whether any threshold was crossed, and whether the epoch was re-pinned.

- Summaries: [gate](../../progress/features/<id>.json),
  [goal](../../progress/features/<id>-goal.json).
- Full readings: [`docs/progress/readings/<id>.md`](../../progress/readings/<id>.md).

Raw paths, hashes, raw/summary byte counts, comparison tables and concise
per-seed/neighborhood readings belong in the readings
file, not here. A user decision that accepts a measured cost, grants an
exception, or re-pins a baseline stays in this section verbatim: it is a
contract, not evidence.

The benchmark Verification item above is `Not applicable` only for a feature that
closes before T10.F10 is checked or that cannot change simulation cost, including
a lab feature under the workflow's Benchmark gate exemption (record
`Not applicable: lab feature` with the diff scope). A T21 feature under the
observability exemption records `Not applicable: observability feature`, then
its parent comparison and overhead check: the predeclared method, the
measured ratio in each stack state against the ceiling in force, telemetry's
self-timed cost per tick, and the reviewed no-behavior-change claim.

## Success Criteria

- [ ] The feature outcome is observable and complete.

## Notes for AI Agents

Only what a later feature is bound by. Every line is a bullet starting with one
of four labels — the checker rejects anything else, including prose paragraphs:

- `Decision:` a user decision later work must honour.
- `Exception:` an accepted exception, with what it applies to.
- `Deferred:` a deferred review finding or mutation survivor.
- `Cost:` this feature's closure cost record.

Keep execution policy in `docs/workflow.md`.
