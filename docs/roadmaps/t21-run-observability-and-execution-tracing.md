# T21 — Run Observability and Execution Tracing

**Status**: Planned
**Last updated**: 2026-09-26
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Local simulation runs have a shared, searchable history of their configuration,
population outcomes and sampled execution, from tick timing to individual
creature decisions. One Docker Compose command starts the persistent telemetry
stack; normal telemetry adds at most 5% elapsed time to a fixed simulation
workload, and collection failures never hold up the simulation.

## Track Success Criteria

- [ ] One documented Compose command starts a local OpenTelemetry Collector,
  Prometheus, Tempo, Loki and Grafana with persistent volumes, explicit retention,
  pinned images and provisioned data sources and dashboards.
- [ ] Server and CLI simulation entry points can report distinct runs, linked to
  their application invocation, seed, world, effective configuration and code
  revision, including resets and individual benchmark cases.
- [ ] Every normally instrumented run attempts lifecycle and basic metric
  delivery regardless of trace sampling; short runs flush within a bounded
  shutdown deadline. Missing completion and delivery gaps remain distinguishable
  from successful completion or measured zero.
- [ ] A run dashboard connects applied population, energy, mortality,
  reproduction, mutation and cognition metrics to sampled tick/phase traces and
  bounded, consecutive creature execution windows with applied action outcomes.
- [ ] Retained spans expose rich, typed context: run/build identity, effective
  configuration and feature state at capture time, tick/phase and relevant
  creature state. Large snapshots are navigable from the trace through versioned
  context records; unavailable or truncated context is explicit.
- [ ] Expensive recording is selected before capture; independent collection
  cadences and trace budgets bound recording size, export volume and queue memory.
  Creature IDs and tick numbers never become metric labels.
- [ ] Stack outage or overload causes bounded retries and visible telemetry loss,
  with no network or disk wait in the simulation tick, no unbounded buffering and
  no change to simulation RNG, state, action ordering or outcomes.
- [ ] Short paired measurements qualify the normal configuration within 5%
  elapsed-time overhead in every declared workload, including local stack
  contention, heavy cognition and unavailable/overloaded collection; targeted
  enabled/disabled comparisons verify unchanged simulation results.
- [ ] After OpenTelemetry is implemented and qualified, the spec template
  guides future features in adding context and spans through that infrastructure.

## Executable Features

- [ ] **T21.F01 — Local Stack and Run Identity** — Depends on: None
  - Goal: A simulation run can be found after it ends, with its identity, configuration and lifecycle delivered to a persistent local stack that tolerates outages.
- [ ] **T21.F02 — Applied Run Metrics and Dashboards** — Depends on: T21.F01
  - Goal: A run's population, costs and applied outcomes can be followed through time and compared with other identified runs using existing observations.
- [ ] **T21.F03 — Sampled Tick and Phase Tracing** — Depends on: T21.F02
  - Goal: A sampled tick shows where execution time went and which configuration and feature state governed it, linked to its run and metric history.
- [ ] **T21.F04 — Bounded Creature Cognition Windows** — Depends on: T21.F03
  - Goal: A sampled creature's consecutive decisions can be followed through its inputs, internal state, effective feature settings and executed cognition to actions and their applied outcomes.
- [ ] **T21.F05 — Qualified Defaults and Operator Guide** — Depends on: T21.F04
  - Goal: Ordinary runs retain useful metrics and execution examples under the 5% overhead ceiling, with documented sampling, retention and failure behavior.
- [ ] **T21.F06 — OpenTelemetry Spec Template Guidance** — Depends on: T21.F05
  - Goal: Future feature specs describe relevant OpenTelemetry context and spans using the implemented, qualified infrastructure; the guidance lives only in the spec template.

## Notes for AI Agents

- **Decisions, 2026-09-26.** Local collection is sufficient; outages and lost
  telemetry are acceptable. The user accepts at most 5% performance overhead and
  exempts this track from expensive benchmark and mutation testing. Retain
  ordinary repository checks and short, targeted telemetry overhead checks.
  T21 and T20 may proceed independently; serialize measurements on the same host.
  Feature specs are written just in time; this draft starts no implementation.
- **Ownership and T20 independence.** T21 owns delivery, run correlation,
  dashboards and sampled execution observation. T14 retains report integrity and
  applied counters; T15 retains benchmark evidence storage; mechanism tracks
  retain their measures and behavior. T21 has no dependency on unfinished T20
  work and adds none to T20. Use existing runtime observation boundaries; keep
  exporters, queues and backend configuration outside simulation semantics.
  Extend observation hooks narrowly where required, without refactoring the
  evolving Graph/VM recruitment or learning mechanisms. T20-specific signals can
  use these facilities when available and otherwise retain their existing
  report/test surfaces. Telemetry is observation and has no natural-analog
  requirement. Historical benchmark artifacts and the static progress page
  remain authoritative for their existing purposes.
- **Existing inputs.** `crates/v3-cli/src/lib.rs` already emits start, sampled
  metrics and completion events. `SimStats` and T14 supply applied counters.
  `crates/v3-core/src/runtime/trace/domain.rs` supplies structured execution
  samples. The selected creature currently runs sequentially after parallel
  cognition in `crates/v3-core/src/simulation/tick.rs`; do not assume its recording
  cost is negligible. Its selected actions need joining to actual application
  outcomes. Reuse these instruments before adding counters or another recorder.
- **Research, 2026-09-26.** Options considered: extend existing Petri observations
  with standard OTLP exporters and separately configured Compose services
  (selected); use the bundled [Grafana otel-lgtm image](https://grafana.com/docs/opentelemetry/docker-lgtm/)
  (credible for a prototype, officially intended for development/demo/testing);
  or build custom ingestion/storage (unnecessary given existing components).
  Separate services cost more configuration but expose retention, resource limits
  and upgrades for ongoing local history. [Tempo supports local Compose deployment](https://grafana.com/docs/tempo/latest/set-up-for-tracing/setup-tempo/deploy/locally/),
  [Prometheus accepts OTLP metrics](https://prometheus.io/docs/guides/opentelemetry/),
  and Rust has a [tracing bridge](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/)
  and [batch exporter support](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html).
  F01 verifies compatible dependency/image versions and a tiny end-to-end export;
  no performance claim is inferred from library availability.
- **F01 contract.** Distinguish invocation/session identity from simulation-run
  identity, including reset, restore and multi-case invocations. Keep lifecycle
  records outside trace sampling, record effective configuration changes, and
  represent interrupted/unknown endings honestly. Default ordinary runs to the
  documented local endpoint with an explicit telemetry-off mode; a missing
  stack is harmless. Benchmark commands support instrumentation, but established
  scientific gate/goal measurements explicitly disable export and tracing to
  preserve their comparison conditions. Test fixtures need not create archived
  runs. Bind exposed services to loopback. Retention is finite and documented;
  volume persistence does not promise permanent or lossless history.
- **Rich context contract, F01/F03/F04.** Prefer fewer richly explained samples
  over more spans that lack the context needed to interpret them. F01 stores
  the full effective simulation configuration, including resolved defaults and
  overrides, and its identity/version at run start and on effective change.
  Record the revision/build, available build features, world/recipe and seed,
  thread count, runtime/backend modes and telemetry policy. Distinguish a
  mechanism being available in the build, enabled by configuration, and actually
  exercised in the recorded execution. Reuse the existing configuration model;
  no separate feature-flag framework is required.
  F03 puts run/invocation identity, context/schema version, effective config
  revision, tick, phase, sampling policy and relevant feature names, enabled
  states and actual parameter values directly on spans as typed attributes.
  Keep searchable decision-relevant values on the span; a config hash alone is
  insufficient. Store large repeated configuration/state descriptions in
  correlated context records and provide trace-view navigation to them. Match
  their retention to the traces that use them. If delivery or retention removes
  a referenced record, show missing context rather than substituting current
  values. Child spans and parallel work carry the necessary correlation and
  scoped context explicitly; parent attributes are not assumed to be inherited.
  Bind context to the observed execution before background export, so a later
  configuration change cannot relabel earlier spans.
- **Creature context, F04.** Include creature and lineage identity, generation,
  age, position, energy, genome identity and backend where applicable. Capture
  the inputs actually read, relevant memory/state before and after execution,
  executed nodes/passes, routing scores, votes, selected action parameters,
  realized costs, applied outcomes and termination/rejection reasons. Include
  effective learning, mutation, cost or other mechanism settings when they
  explain the operation, and retain the recorded structural description needed
  to interpret node/genome identifiers within the sample's bounds. Each feature
  spec names its actual fields and existing sources. Events describe ordered
  changes; spans describe operations with measured durations. Do not invent
  timings for existing recordings that contain only execution order.
  Full-population or full-world copies per span are outside this contract.
- **Context cost and completeness.** Cache immutable context, serialize large
  snapshots once per version, and capture expensive mutable state only after
  sample admission. Reserve room for the interpretation-critical fields before
  optional detail; mark omitted fields/events and their reason. Qualify the 5%
  budget with the actual rich payload enabled, including allocation, copying,
  serialization and export. Tune sample frequency before stripping essential
  context. Verify configuration changes, parallel correlation, feature
  disabled-versus-unused states, missing context records and size-limit behavior
  through focused fixtures and a stored trace inspection. The standard
  [OpenTelemetry span attributes/events API](https://opentelemetry.io/docs/specs/otel/trace/api/)
  fits this model; its [SDK collection limits](https://opentelemetry.io/docs/specs/otel/trace/sdk/)
  require explicit sizing and dropped-detail reporting. Repeating every full
  snapshot on every span adds avoidable cost, while identifier-only spans fail
  the user's rich-context requirement; use typed local values plus correlated
  snapshots. Research checked 2026-09-26; F01 verifies limits across the selected
  Rust SDK, Collector and backends rather than assuming unlimited payloads.
- **F02 contract.** Reuse cumulative applied counters and export them
  periodically without sampling away counted events. Collect gauges and costly
  population censuses at separately budgeted cadences. Declare units and counter
  reset behavior. Put run IDs on an explicit, bounded metric set; keep full
  configuration, creature/genome identity and tick detail in records or traces.
  [Prometheus label cardinality guidance](https://prometheus.io/docs/practices/instrumentation/)
  motivates this bound. Record simulation ticks alongside wall time; cross-run
  charts state which axis and configuration they compare.
- **F03/F04 contract.** Use short tick traces and creature windows correlated by
  run identity, rather than one trace spanning a whole run. Select expensive
  capture before constructing snapshots, strings or detailed trace records;
  [head sampling](https://opentelemetry.io/docs/concepts/sampling/) can save source
  work, while downstream tail sampling cannot undo capture already performed.
  Retain complete selected windows within explicit size limits, recording
  truncation and the sampling policy. Bound window length, events, bytes and
  admission rate independently of population and tick speed. Label background
  samples separately from manual targeted recordings; sampled examples do not
  establish population frequencies. Describe executed inputs, state changes,
  routing and outcomes rather than inventing a creature's explanation. Manual
  detail stays bounded and is identified separately from qualified defaults.
- **Failure contract.** Bounded background queues, finite retries and bounded
  shutdown flushes may drop data. Show queue pressure and lost records through
  local diagnostics and exported health counters when delivery resumes. Do not
  build durable application spooling or retrospective replay. Collector
  [queue resilience](https://opentelemetry.io/docs/collector/resiliency/) does not
  guarantee delivery of data that never reached it.
- **F05 contract.** Choose actual rates from short measurements, not assumed
  percentages. The ceiling is `elapsed_enabled / elapsed_disabled <= 1.05` for
  the same fixed work and build; enabled includes the local stack's contention,
  disabled has telemetry off and that stack stopped. Every earlier runtime
  feature checks its incremental cost against the same total budget. F05 checks
  the combined defaults on small, representative-population and cognition-heavy
  workloads, with healthy and failed collection, and records host, revision,
  configuration, paired timings and variability. Predeclare repetition and a
  short wall-time cap in the spec. Noisy evidence is inconclusive, never a pass;
  evidence applies only to the declared workloads. Target margin below 5% and
  keep the disabled path cheap. This is a bounded overhead check, not an
  ecological, neighborhood or evolutionary qualification campaign.
- **F06 contract.** Only after F05 is complete, update
  [the flat feature template](../specs/roadmap/_feature-template.md) to describe
  adding relevant OpenTelemetry spans and context through the implemented
  infrastructure. Keep this guidance solely in the template, not in workflows
  or agent instructions. It must not require substitute logs, artifact dumps,
  new observation systems, extra experiments or recording before OpenTelemetry
  exists. Leave unrelated feature evidence requirements unchanged; no completed
  spec backfill. This roadmap entry does not activate the guidance or modify
  the template now.
