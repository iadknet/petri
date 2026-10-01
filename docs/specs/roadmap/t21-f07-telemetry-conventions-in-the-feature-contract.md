# T21.F07 — Telemetry Conventions in the Feature Contract

**Status**: In Progress
**Last updated**: 2026-09-30
**Feature**: T21.F07
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

The feature spec template's Telemetry section names the conventions T21.F01
to T21.F06 delivered — signal names, attribute sets, caps, trace identity,
presets, the ceiling and the test rule — in place of the interim wording it
has carried since 2026-09-29, and `.claude/agents/roadmap-reviewer.md` tells
the final reviewer to check a diff's signals against them, so that a feature
which adds a mechanism, counter, config value or trace ships its exported
signals with it and a review catches one that does not.

## Non-Goals

- No production-code, script, dashboard or stack change; the one test-only
  edit is the admitted race fix in `crates/v3-cli/tests/telemetry.rs`
  (invariant 1, Notes Exception). No new convention. The
  template records what F01–F06 delivered and where it is wired; a rule the
  closed specs do not state is not added here.
- No edit to `docs/workflow.md`: its Review section already requires the
  Telemetry section for specs planned from 2026-09-29 (track Notes), and the
  reviewer file is the checklist that section hands to Codex.
- No retrofit of the F01–F06 specs or of specs planned before 2026-09-29.
- No checker script: the convention check is a reviewer judgement on a diff,
  as the F07 row states, not roadmap machinery.
- No change to the template's Verification or Performance paragraphs beyond
  the Telemetry section.

## Inputs and Invariants

| Input | Where | What F07 takes from it |
| --- | --- | --- |
| F07 row and Notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | The two deliverables: the template's telemetry section names the delivered conventions; the reviewer file checks a diff's signals against them. A feature that adds a mechanism or counter ships its exported signals with it. |
| Template | `docs/specs/roadmap/_feature-template.md`, Telemetry section | The signal table (`Signal`, `Kind`, `Source`, `Context carried`), the `Not applicable: <reason>` form, the five interim rules and the "Until T21.F02 closes" paragraph that F07 replaces. Not budget-checked (`_` prefix). |
| Reviewer | `.claude/agents/roadmap-reviewer.md` | "What to read", "What to check" and "Severity"; Codex reads it ignoring the front matter (workflow, Review). |
| Workflow | `docs/workflow.md`, Review "Telemetry section" paragraph; Benchmark gate, observability exemption and Record rule | What the reviewer already checks; the Record wording this spec uses. |
| F01 | spec, Inputs and Invariants: identity table, records table | Resource and record attribute sets; `event.name`; absent, never zero; the reference build (`--no-default-features`, `v3-core/telemetry-seams`); queue bounds. |
| F02 | spec, invariants 4–7 and the families table | Metric names, kinds, the attribute rule, cadence caps, the Prometheus translation; wiring in `crates/v3-telemetry/src/metrics.rs` (`run_scalars!`, `counters`, `energy`, `mutations`, `outcomes`, `tick_values`), expected families in `metrics/tests.rs`, `always_names` in `scripts/telemetry-dashboards-check`. |
| F03 | spec, invariants 3–7 and the spans table | Every-span identity row, `petri.sample_policy`, `petri.config.<path>` groups per phase in `crates/v3-telemetry/src/trace.rs` (`world_update`, `cognition`, `actions`, …), trace ID = run key high 64 bits + tick, cap 65,536 per run, `petri.tick_traces_capped`. |
| F04 | spec, invariants 3, 5, 7, 8 and the caps table | Window settings on `run.started`; trace ID `2^63 + k`; `petri.window`, `petri.creature_id`; caps 2,048 events, 4 MiB per window, 4,096 samples and 256 MiB per run; `petri.windows_capped`. |
| F05 | spec, invariants 2, 3, 5 and the preset table | Default on; `--telemetry off` in every test that spawns a binary; presets `minimal`, `phases`, `standard`, `dense` in `crates/v3-telemetry/src/preset.rs`; `petri.preset`; ceiling 1.10 in `scripts/telemetry-overhead`, 5% the target. |
| F06 | spec, invariants 3, 5 and the records table | `measurement.*` records, capture at region boundaries only, held export; `petri.total.<field>` on a seed's `run.ended`; closure benchmarks export (user decision, 2026-09-30). |
| Repository rule | `AGENTS.md` | A per-tick trace or exhaustive record is never written without a predeclared size projection. |
| OTel–Prometheus compatibility | `https://opentelemetry.io/docs/specs/otel/compatibility/prometheus_and_openmetrics/` (read 2026-09-30) | The exporter, not the OTel name, adds `_total` to monotonic sums and the unit suffix; F02's naming follows it, so the template states the rule once rather than per family. |

**Conventions the template names.** One row per convention; the template
carries this table with the same wording, so the reviewer and a spec author
read one text. "Spec" means the feature spec that adds the thing.

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

Invariants:

1. The diff touches `docs/specs/roadmap/_feature-template.md`,
   `.claude/agents/roadmap-reviewer.md`, this spec, its readings file and the
   roadmap rollups, plus one test-only fix in T21's own test helper,
   `crates/v3-cli/tests/telemetry.rs` (`command_with` writes a recipe path
   unique to each call instead of the shared
   `CARGO_TARGET_TMPDIR/t21-<shape>.json`, so parallel tests no longer read
   a truncated file; no assertion changes), and nothing else: no other
   `crates/` file, no `scripts/`, `telemetry/` or `frontend/` file, no
   production code, no `SimulationConfig` or `RuntimeConfig` field, no
   production RNG draw, no stored summary and no deterministic output.
2. Every rule in the template is traceable to a row of the table above and
   so to a closed T21 spec, the track or `AGENTS.md`; the template cites the
   owning specs for the full attribute and family tables instead of copying
   them.
3. The template keeps its signal table and the `Not applicable: <reason>`
   form, and drops the "Until T21.F02 closes" paragraph and the phrase
   "Rules, from the T21 track" that introduced the interim bullets.
4. The reviewer file's extension is checklist, not policy: it says what to
   compare and the severity, and names the files to compare against; the
   front-matter block is unchanged and the file stays a single checklist
   Codex reads in one pass.
5. Severity, in the reviewer file: a counter, per-tick value, config value
   or mechanism switch the diff adds without its exported signal is P1,
   because the F07 row makes the signal part of the feature; a
   policy-sampled observation (span, event, window or body, not a
   cadence-bounded gauge) the diff adds without its policy, per-run cap and
   projected size is P1, the `AGENTS.md` trace rule; a departure that unbounds
   memory, series or storage or can collide trace IDs is wrong behavior
   and P1 under the existing rule; a departure that only causes later
   rework (a name, a description, an attribute set) is P2; a spec planned
   before 2026-09-29 is still not required to have a Telemetry section.
6. The template and the reviewer file agree: a check the reviewer performs
   is a convention the template states, in the same words for the names of
   files and attributes.

**Reviewer extension.** Under "What to read": the Telemetry section of
`docs/specs/roadmap/_feature-template.md`, the conventions. Under "What to
check", one bullet, "Telemetry", with the following checks on the diff: the
spec's Telemetry section is filled in or carries a `Not applicable` reason
the diff supports; each numeric cumulative counter or numeric `last_tick_*`
value the diff adds has its family in `metrics.rs`, its expected row in the
metrics tests and its Telemetry row, while a per-event record or seam is
trace material and a measurement record is not a metric; each config value
or mechanism switch the diff adds is listed as recorded configuration
(config body or `run.started` attribute) and, when it governs a traced
phase, sits in that phase's group in `trace.rs`; no metric attribute carries
a tick, creature, lineage, genome, seed or config value; a policy-sampled
observation the diff adds names its policy, per-run cap and projected size
while a gauge is bounded by the snapshot cadence,
and a trace kind it adds has an entropy-free ID that collides with neither
delivered kind, while an unsampled record at a region boundary needs
neither; a value not captured is absent, never zero; a test the diff adds
that spawns a binary passes `--telemetry off`, or is a telemetry fixture
that passes `on` and scrubs `PETRI_TELEMETRY*` and the endpoint; telemetry
capture, export and seams the diff adds sit behind `telemetry` and
`telemetry-seams`. Under "Severity": the P1 kinds and the P2 kind of
invariant 5.

## Telemetry

Not applicable: documentation and agent-definition change; a run does nothing
different, no signal is added or renamed, and no configuration is recorded.

## Implementation Tasks

- [x] Replace the template's Telemetry section: keep the signal table and
      the `Not applicable` form, add the conventions table above with the
      `From` column pointing at the owning specs, and remove the interim
      paragraph and bullets (invariant 3).
- [x] Extend `.claude/agents/roadmap-reviewer.md` as the Reviewer extension
      paragraph states (invariants 4–6).
- [x] Record in `docs/progress/readings/t21-f07.md` the grep results that
      tie each file and attribute name the template cites to the code
      (Verification), beside the challenge-loop table.
- [x] Run `make check-docs`; at closure the orchestrator runs `make check`
      on the final code (the user's goal requires it at each closure).

## Verification

- [x] `make check-docs` -> exit 0 on the working tree over `09ddc2f3`
      (`roadmap-check`, `policy-check`, `quality-check`).
- [ ] `make check` -> exit 0 on the final code at closure, recorded as the
      tested commit (user's goal; the suite is unchanged in what it asserts,
      the telemetry-neutrality test included). `command_with` in
      `crates/v3-cli/tests/telemetry.rs` writes each call's recipe to
      `t21-<shape>-<pid>-<seq>.json` and removes it after the run, so
      parallel tests no longer read a recipe another test is rewriting (the
      `EOF while parsing` failure of
      `an_invalid_tick_trace_switch_refuses_to_start_a_run_with_telemetry_on`).
      On the fixed helper `cargo test -p v3-cli --test telemetry` passes
      13/13 in each of 10 consecutive runs. `make check` on the working tree
      over `5bff240e` exits 0 (35 `test result: ok` lines, no failures); the
      closure run on the tested commit remains.
- [x] Template–code agreement: every file path, function name, script
      variable, attribute and setting name the template's conventions cite
      resolves by `grep` in the worktree. The `file:line` table is in
      `docs/progress/readings/t21-f07.md`.
- [x] Template–source agreement: the readings file lists each convention row
      against the closed-spec sentence, track note or `AGENTS.md` rule it
      restates, with that source's condition or exception (invariant 2).
- [x] Template–reviewer agreement: the readings file maps each check in the
      reviewer's Telemetry bullet to a template row (invariant 6).
- [x] `git diff --stat 3195f536` plus `git status --porcelain` list only the
      files invariant 1 allows (readings file, Diff scope).
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: Not applicable:
      observability feature.
- [x] Benchmark summary: Not applicable: observability feature.

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG, writes
no stored summary and makes no production-code, script or stack change; its
one crate edit is the admitted test-only race fix in
`crates/v3-cli/tests/telemetry.rs` (invariant 1, Notes Exception);
the reviewer checks that against the diff. Documentation feature: the
natural-analog rule and the environmental-pressure rule do not apply, and no
indicator can move.

Parent comparison not applicable: the diff touches no `v3-core` file and no
code that runs inside the tick loop or a timed region, so the two builds it
would compare are the same binaries.

Overhead check not applicable: the diff adds no work to a run with telemetry
on or off; the ceiling in force (1.10, F05) is unaffected and no timed run is
made.

**Measured verdict.** None: no profile, comparison or overhead run applies.
The tested commit of `make check` is recorded in Verification.

## Success Criteria

- [ ] The template's Telemetry section names the delivered conventions in
      the table above, cites the owning specs, and no longer carries the
      interim wording.
- [ ] `.claude/agents/roadmap-reviewer.md` tells the reviewer to read the
      template's conventions and check a diff's signals against them, with
      the P1 and P2 severities of invariant 5.
- [ ] `make check-docs` and, at closure, `make check` exit 0; the diff is
      within invariant 1.

## Notes for AI Agents

- Decision: closure benchmark runs export telemetry and still write their
  stored reports (user, 2026-09-30, recorded at T21.F05); the template's
  "Default and tests" row carries it.
- Exception: F07 carries one test-only fix outside its documentation scope,
  the per-call recipe path in `crates/v3-cli/tests/telemetry.rs`, because
  the user's goal requires `make check` to exit 0 on the final code and the
  race is in T21's own test helper (spec owner ruling, 2026-09-30); it
  changes no production code, behavior or assertion, so the observability
  exemption holds.
