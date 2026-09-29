# Run observability: a local OpenTelemetry stack for every run

Date: 2026-09-29. Status: research and planning note; no feature executed and
no source, default or dependency changed. It adds track T21. Depth: the
repository's own telemetry seams read in source; external claims checked
against project documentation and, for the span processor, the SDK source;
budget arithmetic from one stored goal summary. No new measurement was run,
and no measured counterfactual exists yet: the first T21 feature owns it.
Reviewed adversarially by Codex Astra `high` on 2026-09-29; the findings and
their dispositions are recorded at the end.

This note starts from the user's requirements and the code. It does not
inherit the design of the T21 draft that the 2026-09-27 revert rolled back.

## Decision in brief

**Add a small telemetry crate that maps Petri's existing observation seams to
OpenTelemetry, export over OTLP to a local Docker Compose stack, and sample
detailed traces under hard caps chosen by measurement.** `v3-core` takes no
OpenTelemetry dependency. Aggregate counters stay complete; only expensive
observations are sampled. The stack is an instrument for looking at runs. It
is never closure evidence, never an indicator, and never program tracking.

## Requirements (user decisions)

Stated by the user on 2026-09-26 in the planning conversation and restated
as the brief for this note on 2026-09-29.

| # | Requirement |
| --- | --- |
| R1 | Telemetry from all runs is stored in one centralized OpenTelemetry stack (Prometheus, Grafana and companions). |
| R2 | Each application run streams many basic metrics tied to that run. |
| R3 | Sampled spans and traces cover ticks down to individual creature cognition: what a creature sensed, considered and did. |
| R4 | Telemetry must not cost much. The ceiling is about a 5% performance hit at most. Sampling rates are chosen to fit it. Amended 2026-09-29: 25% is the interim ceiling for every closure before F05 qualifies the default; 5% stays the target, and the final ceiling is set at F05 from data. |
| R5 | The stack starts from a simple Docker Compose file. |
| R6 | The stack is local. Outages are acceptable: gaps are fine and the simulation keeps running. |
| R7 | The track skips the expensive closure machinery: no benchmark profiles, no mutation gate. |
| R8 | The track can be worked concurrently with T20. |
| R9 | The feature spec template (the "PRD template") gains a telemetry section. |
| R10 | Spans carry as much context as possible: feature state, config values, and whatever else explains the sample. |

## What exists locally

| Seam | Where | What it gives |
| --- | --- | --- |
| Run events | `crates/v3-cli/src/lib.rs` (`run_simulation`) | NDJSON `run_started`, `tick_sample`, `run_completed`; `config_digest`; population, energy, structure means, reproduction and mutation counters. |
| Runtime counters | `crates/v3-core/src/simulation/stats.rs` (`SimStats`) | The T14 counters: mortality by cause, energy flows, mutation supply and outcomes, cognition counts, predation. |
| Phase timing | `crates/v3-core/src/simulation/tick.rs` (`phase_wall_clock`) | Accumulated wall time for world update, sensor assembly, cognition, actions, reward learning. |
| Creature recorder | `crates/v3-core/src/runtime/trace/` and `run_cognition` in `tick.rs` | One traced creature per tick: inputs, hops, passes, votes, selected actions, termination reason, priority bid. |
| Manual sampling | `crates/v3-server/src/http/creature.rs` (`ActiveTrace`) | The server's Execution Sampler requests a trace for one creature for N ticks. |
| Config changes | `crates/v3-server/src/http/config_patch.rs` | The server can change config during a run. |

Three facts from the code shape the design.

1. **The recorder traces one creature and runs it sequentially.**
   `run_cognition` removes the traced creature from the parallel work list,
   runs it after the others, and reinserts its decision at its queue position.
   Its cost is unmeasured. One window at a time fits the recorder as it is;
   a set of creatures at once would need a recorder change. `ActiveTrace`
   and `TickTrace` hold nested vectors with no byte limit, so a budget has
   to be enforced during capture.
2. **The recorder stores selected actions, not applied outcomes.** `TickTrace`
   carries `final_actions`. Whether an action succeeded, what it cost, and
   what it changed is decided in the action phase. The project rule that
   telemetry derives from applied behavior requires the join.
3. **The simulation is singly owned.** An exporter thread cannot read
   `Simulation`. The run loop has to publish counter snapshots to the
   exporter.
4. **Not every counter is cumulative.** `SimStats` holds per-tick values
   (`last_tick_*`) that `reset_tick_counters` clears every tick. Copied on a
   cadence they are sampled gauges. Only cumulative counters stay complete
   when read late.
5. **Benchmarks time several regions in one process.** `bench/run.rs`
   times cases and observations in sequence, so an exporter that runs
   between them competes with the next one.

## Budget arithmetic

From the stored T11.F27 goal summary
(`docs/progress/features/t11-f27-compact-action-parameter-storage-goal.json`,
`environment` block):

| Reading | Value |
| --- | --- |
| Ticks per second, total | 9.74 |
| Wall time per creature-tick | 15.8 µs |
| Mean wall time per tick | about 103 ms |
| 5% of a tick | about 5 ms |
| 5% of a creature-tick | about 0.8 µs |
| Wall flag threshold in the benchmark | 25% |
| Wall severe threshold in the benchmark | 100% |

Consequences:

- **Per-creature spans on every tick cannot fit.** The SDK's own benchmark
  reports 1.1 to 2.5 µs to create one span
  ([issue 3746](https://github.com/open-telemetry/opentelemetry-rust/issues/3746)).
  That is more than the whole 0.8 µs budget before any attribute is set.
  Creature detail must be sampled, and the decision must precede recording.
- **A percentage of the population is the wrong control.** Cost would grow
  with population and tick rate. Caps per tick and per wall second keep the
  cost fixed.
- **5% is expensive to prove.** The benchmark's configured wall flag is 25%.
  That is a threshold, not a measured noise floor, and the run-to-run spread
  of a short workload on this host is unmeasured. Each overhead check
  measures that spread first and sets its pair count from it.
- **So the ceiling is staged (user decision, 2026-09-29).** Every closure
  before F05 qualifies the default is held to a 25% interim ceiling, applied
  to total telemetry cost. Each closure also records telemetry's
  self-timed cost per tick. That reading covers only time inside telemetry
  calls on the simulation thread, so it is a steady lower bound, not the
  total. F05 measures cost against detail and the user sets the default's
  ceiling from it, with 5% as the target. The risk of a loose interim
  ceiling is an unconditional cost, paid whether or not anything is
  recorded. Comparing telemetry off and on in one binary cannot see it, so
  cost is measured against a reference build of the same commit with
  everything T21 added compiled out. Both sides then hold the same
  simulation code. A fixed earlier commit cannot be the baseline, and
  ratios from earlier closures cannot be multiplied in, because other
  tracks keep changing what a tick costs. A 20 ms addition is 20% of a
  100 ms tick and 40% of a 50 ms one.
- **The checks have a time cap.** All timed runs for one closure fit inside
  5 minutes, less than the eleven-minute goal run the track skips. F05's
  qualifying measurement is the one longer allowance, 60 minutes, taken
  once. Reaching a cap gives an inconclusive result for the user, not more
  runs.

## External findings

Checked 2026-09-29.

| Topic | Finding | Source |
| --- | --- | --- |
| Rust SDK maturity | Metrics and logs API and SDK are Stable, their OTLP exporters RC. Traces API, SDK and OTLP exporter are Beta. Latest SDK line is 0.33. MSRV 1.75; the workspace pins 1.93. | [opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust) |
| Span export | `BatchSpanProcessor` exports on a dedicated background thread. A full queue drops the span through `try_send`; it never blocks. Defaults: queue 2,048, delay 5 s, batch 512, export timeout 30 s. It logs the first drop and the total at shutdown. | [span_processor.rs](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/src/trace/span_processor.rs) |
| Transport | The gRPC exporter needs a tokio runtime. The HTTP exporter with the blocking client does not. `v3-cli` and `v3-lab` are synchronous; `v3-server` is tokio. | [BatchSpanProcessor docs](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html) |
| Sampling | A head decision is made before the work. A tail decision at the collector sees the whole trace but the application already paid to produce it. | [Sampling](https://opentelemetry.io/docs/concepts/sampling/) |
| Collector outages | Collector queues and disk storage protect data that reached the collector. Data the SDK could not deliver is lost. | [Collector resiliency](https://opentelemetry.io/docs/collector/resiliency/) |
| Prometheus and OTLP | Prometheus 3 ingests OTLP natively behind `--web.enable-otlp-receiver`. It expects cumulative temporality. Resource attributes land on `target_info` unless promoted. An out-of-order window of 30 minutes is the documented setting. | [Prometheus guide](https://prometheus.io/docs/guides/opentelemetry/) |
| Retention defaults | Prometheus keeps 15 days unless a retention flag is set, and time retention cannot be switched off, only set very long. Tempo's `block_retention` defaults to 336 hours. Loki keeps logs unless compactor retention is enabled. | [Prometheus storage](https://prometheus.io/docs/prometheus/latest/storage/), [Tempo configuration](https://grafana.com/docs/tempo/latest/configuration/), [Loki log deletion](https://grafana.com/docs/loki/latest/operations/storage/logs-deletion/) |
| Deleting on request | Prometheus deletes series in a time range through its admin API and frees disk with `clean_tombstones`. Loki accepts delete requests by stream and time window once compactor retention is enabled. Tempo has no delete API; one is proposed. | [Prometheus admin API](https://prometheus.io/docs/prometheus/latest/querying/api/), [Tempo deletion RFC](https://github.com/grafana/tempo/discussions/5775) |
| Bundled stack | `grafana/otel-lgtm` bundles the Collector, Prometheus, Tempo, Loki, Pyroscope and Grafana in one container. It persists under `/data`, takes `*_EXTRA_ARGS` per component and mounted config files, and is described as for development, demo and testing. | [docker-otel-lgtm](https://github.com/grafana/docker-otel-lgtm) |

## Options considered

### Backend stack

| Option | For | Against |
| --- | --- | --- |
| **A. `grafana/otel-lgtm` in a Compose file (recommended start)** | One service, one volume, one command. Persistence, retention arguments and dashboard mounts are supported. A container CPU and memory limit bounds contention with the simulation. | Described as not for production. Components upgrade together. No per-component limits. |
| B. Separate Collector, Prometheus, Tempo, Loki and Grafana services | Per-component retention, limits and upgrades. | Five services and their configuration files to maintain for one developer's machine. |
| C. Prometheus native OTLP with no Collector | Fewer moving parts for metrics. | Traces and logs still need a receiver, so nothing is saved overall. |
| D. A different backend family (VictoriaMetrics, ClickHouse-based) | Some are lighter on disk. | The user named Prometheus and Grafana. No requirement motivates the switch. |

The "not for production" label on option A describes availability and scale
guarantees. R6 waives both: the stack is local and outages are acceptable.
Petri speaks only OTLP to `localhost`, so moving from A to B later changes
the Compose file and nothing in Rust.

Keeping data indefinitely narrows the gap between A and B. The bundled image
now needs overrides for retention and deletion, and it makes no promise
about carrying data across its own versions. The user chose A on 2026-09-29
with a fallback. T21.F01 verifies three things on the image: the retention
overrides take effect and survive a restart, the cleanup script reaches each
store's delete path from outside the container, and stored data survives a
change of image version. If any fails, F01 uses B. The image keeps each
store's data in its own directory in the standard format, so a later move
carries the data.

### Where the OpenTelemetry code lives

| Option | For | Against |
| --- | --- | --- |
| **A new `crates/v3-telemetry` crate used by the binaries (recommended)** | `v3-core` stays dependency-free and deterministic. Core mutation builds do not grow. The mapping has one owner. | Core needs a few plain observation seams. |
| OpenTelemetry calls inside `v3-core` | Spans sit next to the code they describe. | Global SDK state inside the simulation crate. Slower core builds. Instrumentation cost even when nothing listens. |
| The `tracing` crate in core with the OpenTelemetry bridge | The Rust project recommends `tracing` for logs. | A `tracing` span per creature-tick pays a dispatch cost on the hot path, and the recorder's structured data does not fit span fields well. |

### Sampling control

| Option | For | Against |
| --- | --- | --- |
| **Hash-selected windows under hard caps (recommended)** | Cost is fixed regardless of population. Selection is a pure function of run, creature and window, so it consumes no production RNG and the same seeded run samples the same creatures. | Caps need measurement to set. |
| Fixed percentage of creature-ticks | Simple to state. | Cost grows with population and speed. |
| Collector tail sampling | Can keep the interesting traces. | The application has already paid. It cannot protect the ceiling. |
| Adaptive governor driven by self-timing | Holds the ceiling on any host. | More machinery. Keep in reserve until fixed caps are shown to fail. |

## Recommended design

**Boundary.** `v3-core` exposes plain data: counter snapshots, phase timings,
and recorder output for a requested creature. `crates/v3-telemetry`
owns the SDK, the exporters, run identity, sampling policy and the mapping to
metrics, spans and log records. The binaries wire the two together.

**Transport.** OTLP over HTTP with the blocking client on the SDK's own
threads. It works the same in the synchronous CLI and the tokio server.

**Configuration.** Telemetry settings are process-level flags and
environment variables. They never enter `SimulationConfig` or
`RuntimeConfig`, so `config_digest` and the pinned recipe digests do not
move and a run's simulation identity is the same with telemetry on or off.
One command-line flag turns telemetry on or off for an invocation. In every
binary it defaults to off until T21.F05 closes and to on afterwards (user
decision, 2026-09-29). When the default flips, tests that start a binary and
invocations that produce stored closure measurements pass the off flag
explicitly.

**Storage.** Data is kept indefinitely (user decision, 2026-09-29). The
stack configuration overrides the Prometheus and Tempo retention defaults,
and a cleanup command retires data older than a stated date on request.
Prometheus cannot switch time retention off, so indefinite means a value
set far beyond any planned use. The command shows each store's disk use and
the exact targets before deleting, and keeps everything newer than the
cutoff. Tempo has no delete API, so the command removes only whole trace
blocks that end before the cutoff, with Tempo's writers stopped. Disk
use grows without bound by design, so the stack reports it at start and
T21.F01 records measured bytes per run.

**Run identity.** Two levels. An invocation is one process. A run is one
simulation from its seed or reset to its end. A server reset or a benchmark
case starts a new run inside the same invocation. Every signal carries both
IDs.

**Signals.**

| Signal | Policy |
| --- | --- |
| Run lifecycle, identity, full effective config | Every run, never sampled. One log record at start, one at end, one per config change. |
| Cumulative counters | Complete counts. The run loop copies them to the exporter on a cadence; the exporter reads the copy. |
| Per-tick values | Sampled gauges, labelled as sampled. Accumulated before the per-tick reset only where a complete count is promised. |
| Population gauges and censuses | Each on its own cadence set by its cost. |
| Tick and phase timing | Cheap aggregates always. Detailed tick traces sampled. |
| Creature cognition | One selected creature at a time, recorded for a short run of consecutive ticks under an event and byte cap. |
| Measurement commands | Identity, lifecycle and end-of-run totals, captured at region boundaries, held in memory and exported after the final timed region has finished. |
| Manual investigation | The server's existing sampler exports too, labelled as manual. |

**Time.** Trace durations and metric timestamps are wall time. Simulation
progress is the tick. Every span and log record carries the tick. Each
counter snapshot exports a tick gauge taken in the same copy, so a metric
can be placed on the tick axis without a tick attribute.
Traces are short: one per sampled tick and one per creature window. No trace
spans a whole run.

**Cardinality.** The invocation ID is a resource attribute. The run ID is
the only high-cardinality metric attribute,
on a bounded metric set. Each run adds its own series and they are kept
until retired, so the per-run metric set stays small. Creature IDs, ticks, genome
hashes and lineage IDs appear on spans and log records only.

**Context (R10).** Each span carries the identity of its run and build, its
tick and phase, and the config values and mechanism switches that govern the
code it describes, read at capture time. The complete effective config is
logged once per run and again when the server changes it, keyed by its
digest, and spans carry that digest. Payloads too large for a span, such as
a genome, are logged once per window and referenced by hash. A field that
was not captured is marked absent, never written as zero. When the payload
is too expensive, lower the sampling rate before removing context.

**Failure behavior (R6).** Export is best effort. Queues are bounded and
drop when full. Retries and the shutdown flush are bounded in time. Dropped
counts and export failures are recorded so a gap is visible as a gap.

**Determinism.** Telemetry consumes no production RNG, selects no
survivors and changes no execution. A seeded run produces the same
`deterministic` output with telemetry on, off, and with the stack down.

## Guardrails as written rules

1. The stack is an instrument for looking at runs. Closure evidence stays in
   the stored summaries and `docs/progress.md`. No dashboard reading closes
   a feature, sets a default or becomes an indicator.
2. Nothing the stack stores is committed. The Compose file, collector and
   dashboard definitions are source and are committed. Data lives in Docker
   volumes until the user retires it with the cleanup command.
3. Every feature that adds a trace predeclares its size per run and its cap,
   as the telemetry commit rule already requires for per-tick traces.
4. The ceiling is a property of the default configuration, measured with
   the stack healthy, stopped and slow. Its target is 5% and F05 sets the
   final figure. A preset that exceeds it is opt-in and labelled, and
   telemetry is not default-on while its cost sits at the 25% interim
   ceiling.
5. Measurement commands export only after every timed region of the
   invocation has finished.

## Placement

T21 depends on no other track and no other track depends on it. Its rows
are not in the order of new starts. The user starts a T21 feature by naming
it, and it runs beside a feature from the order under the workflow's
existing concurrent-sessions rule. Its code lands in a new crate, the
binaries' wiring, the Compose directory and a few observation seams in
`v3-core`, which keeps it away from the mutation and graph code T20 edits.

Its closures skip the gate and goal benchmark profiles and the mutation
gate (R7). In their place each closure runs the track's own checks: the
telemetry-neutrality test inside `make check`, and the short overhead check
whenever the feature adds work to a run.

## Decisions

Decided by the user on 2026-09-29:

| # | Decision | Answer |
| --- | --- | --- |
| D1 | Is telemetry on by default? | A command-line flag turns it on or off. The default is off until T21.F05 closes and on afterwards. |
| D3 | How long does the stack keep data? | Indefinitely. A cleanup script retires old data ad hoc. |
| D4 | Bundled image or separate services? | Bundled image first, with separate services as the fallback if an F01 verification fails. |

Still open:

| # | Decision | Recommendation | Needed before |
| --- | --- | --- | --- |
| D2 | Which end-of-run totals do benchmark, assay and lab commands record, and do closure measurement runs export? Export from measurement commands is required by R1. | Identity, lifecycle and the totals their stored summary already carries. Closure measurement runs stay off. | T21.F06 |
| D5 | What is the default's final ceiling? | Decide from the measured cost of each level of detail, starting from 5%. | T21.F05 closes |

## Remaining uncertainty

- The cost of the recorder on many creatures at once is unmeasured.
- The Rust trace SDK is Beta. The version is pinned, and an upgrade is an
  ordinary dependency change.
- Whether the bundled image passes T21.F01's three verifications is
  untested.
- The cost of the stack itself on the development host is unmeasured.
  Docker on macOS runs in a virtual machine whose CPU share competes with
  the simulation.
- Whether Grafana can plot a run against ticks comfortably, and not only
  against wall time, is untested.

## Adversarial review record (Codex Astra `high`, 2026-09-29)

Four read-only jobs through the Codex plugin, each on a fresh thread,
reviewing the track, this note and the workflow, template and roadmap edits.

### Round 1 (job `task-mun3j60n-fjwzxj`, verdict `not-ready`)

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | "Nothing expires by age" contradicts Prometheus time retention, which cannot be disabled. | Accepted. Indefinite is defined as size retention off and time retention set far beyond planned use, stated in F01's spec. |
| 2 | blocking | Retiring trace blocks by age can delete newer data or race with writers. | Accepted. One cleanup contract for every store; Tempo removes only whole blocks ending before the cutoff, with writers stopped, verified on a scratch volume. |
| 3 | blocking | Goal command templates and the Codex mutation section still require both specialists. | Accepted. Both templates and the adapter section are conditional on the Benchmark gate's exemptions. `AGENTS.md` is left unedited at the user's direction. |
| 4 | blocking | Off/on in one binary cannot see cost or behavior change on both paths. | Accepted. A parent comparison is added. Its fixed reference commit was replaced by the merge base in round 2. |
| 5 | blocking | F03 promises dashboard reachability but does not depend on F02. | Accepted. F03 and F04 depend on F02. |
| 6 | blocking | Export from measurement commands was left open although all runs are required. | Accepted. Export is required; only which totals are recorded stays open. |
| 7 | blocking | Measurement commands had no flag behavior, and writing between timed regions still competes. | Accepted. Their default stays off at F05; F06 exports after every timed region has finished. |
| 8 | blocking | The interim ceiling was tied to feature numbers, leaving F06 uncovered. | Accepted. The ceiling follows qualification state. |
| 9 | blocking | `last_tick_*` values reset every tick, so a cadence copy is not a complete count. | Accepted. Cumulative counters and per-tick gauges are separated. |
| 10 | advisory | "No default" is ambiguous beside F05's default flip. | Accepted. The text says simulation default and names the flag's default as process-level. |
| 11 | advisory | A configured 25% threshold is not a measured noise floor; self-timing is partial. | Accepted. Both claims are corrected and each check predeclares pairs and the inconclusive result. |
| 12 | advisory | Recorder vectors grow without a byte limit. | Accepted. F04 enforces its cap during capture and marks truncation. |
| 13 | advisory | The claim that sequential cost cannot be tuned by caps is unsupported; a set of creatures was never required. | Accepted. F04 starts with one window at a time and separates unconditional from per-sample cost. |
| 14 | advisory | Metrics "carry the tick" while ticks are forbidden as attributes. | Accepted. A tick gauge is exported in each snapshot; the invocation ID is a resource attribute. |
| 15 | advisory | The template conflated observations with configuration. | Accepted. Signals and recorded configuration are separate rules. |
| 16 | advisory | Telemetry was missing from the implementer and reviewer section lists. | Accepted. Added to both, with the grandfather clause. |
| 17 | advisory | Procedure was repeated in the track. | Accepted. The track states what each check shows; the workflow holds the procedure. |

### Round 2 (job `task-mun3u6z3-r2v21x`, verdict `not-ready`)

Round 2 confirmed every round 1 fix except the three listed as advisory
below.

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Measurement commands stayed off at F05, against the decision that telemetry is on by default after F05, and F06 could close first. | Accepted. F05 depends on F06 and flips the default in every binary. Tests and closure measurement runs pass the off flag explicitly. |
| 2 | blocking | F06 promised unchanged measurements with telemetry on, but the parent comparison runs with it off, and delayed export does not exclude capture or background work. | Accepted. F06 delivers a fixture for deterministic fields, capture placement and exporter start; wall time is compared statistically. |
| 3 | blocking | Pair counts, calibration and reruns had no total cap, so the checks could become expensive. | Accepted. 15 minutes of measured time per closure, 60 for F05; reaching the cap is inconclusive. |
| 4 | blocking | Applying 25% to each ratio separately allowed 56% combined. | Accepted. One ceiling applies to total telemetry cost; separate ratios are for attribution. |
| 5 | blocking | A fixed pre-F01 baseline would include other tracks' simulation changes. | Accepted. The baseline is the merge base; the old commit is historical evidence only. |
| 6 | advisory | The roadmap note and this note still said "F01 through F04". | Accepted. Both follow qualification state. |
| 7 | advisory | F02's goal line still promised gauges as complete counts. | Accepted. Counters and gauges are stated separately. |
| 8 | advisory | The track still held check procedure. | Accepted. Triggers, method, cap and ceiling rule moved to the workflow. |

### Round 3 (job `task-mun3zw1t-ne1g1d`, verdict `not-ready`)

Round 3 confirmed every round 2 fix except the ones below.

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Multiplying ratios from earlier closures does not measure current cost once another track changes what a tick costs. | Accepted. Cost is measured against a reference build of the same commit with T21's work compiled out. Earlier ratios are attribution only. |
| 2 | advisory | A 15-minute cap is bounded but not cheap beside an eleven-minute goal run. | Accepted. Routine checks are capped at 5 minutes; F05's 60 minutes is a stated one-time allowance. |
| 3 | advisory | The track's overhead note still prescribed a calculation. | Accepted. Removed with the multiplication. |

### Round 4 (job `task-mun44rbl-f5l35p`, verdict `ready`)

A narrow round that confirmed the three round 3 fixes and raised no finding.

Codex did not check the external claims in this note in any round, and no
round ran a build, test or measurement; its sandbox was read-only with no
network access.

## Sources

- [opentelemetry-rust repository and status table](https://github.com/open-telemetry/opentelemetry-rust)
- [opentelemetry-sdk span processor source](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/src/trace/span_processor.rs)
- [BatchSpanProcessor documentation](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html)
- [Span processor benchmark figures](https://github.com/open-telemetry/opentelemetry-rust/issues/3746)
- [OpenTelemetry sampling concepts](https://opentelemetry.io/docs/concepts/sampling/)
- [OpenTelemetry Collector resiliency](https://opentelemetry.io/docs/collector/resiliency/)
- [Using Prometheus as your OpenTelemetry backend](https://prometheus.io/docs/guides/opentelemetry/)
- [grafana/docker-otel-lgtm](https://github.com/grafana/docker-otel-lgtm)
- [Prometheus storage](https://prometheus.io/docs/prometheus/latest/storage/)
- [Prometheus TSDB admin API](https://prometheus.io/docs/prometheus/latest/querying/api/)
- [Tempo configuration](https://grafana.com/docs/tempo/latest/configuration/)
- [Tempo trace deletion RFC](https://github.com/grafana/tempo/discussions/5775)
- [Loki log deletion](https://grafana.com/docs/loki/latest/operations/storage/logs-deletion/)
