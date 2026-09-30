# T21.F04 — Creature Cognition Windows

**Status**: In Progress
**Last updated**: 2026-09-29
**Feature**: T21.F04
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

A run started with `--telemetry on` records one selected creature at a time
for a window of consecutive ticks and exports each window as one short
trace: a `creature_window` root span and one `creature_tick` child span per
recorded tick, whose attributes carry what the creature sensed and how its
state changed, and whose span events carry every mesh hop's routing and vote
contribution, every pass's votes and commit, and every action the world
applied with its result and cost. The creature's genome is one log record
per window, referenced by hash. Every sample names the policy that selected
it (`window`, or `manual` for the server's Execution Sampler, which now
exports too). `Petri / Run` lists the run's windows and opens any of them.
With telemetry off, on, or on with no stack listening, the run's
deterministic output is identical, and the traced and untraced paths make
the same decisions.

## Non-Goals

- More than one creature recorded at a time (the track adds that only if
  F04's readings show one window at a time gives too few samples); an
  adaptive governor; collector tail sampling.
- Backend-internal detail (VM steps, graph node evaluations, slot writes,
  per-dispatch decision inputs): not exported; the server's sampler still
  serves it over HTTP.
- No change to the sampler's HTTP protocol, the frontend, the CLI's NDJSON
  or the server's payloads; no new counter; no `SimulationConfig` or
  `RuntimeConfig` field; no Tempo configuration change.
- The qualified cadence, presets and default-on (T21.F05); measurement
  command records (T21.F06); template conventions (T21.F07).

## Inputs and Invariants

| Input | Where | What F04 takes from it |
| --- | --- | --- |
| Track row and notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | Recorder (one window at a time; budget enforced during capture; two costs separated; identical decisions); applied outcomes (join from the action phase); context contract (identity, tick, governing config, digest; large payloads logged once per window by hash; absent never zero); sampling contract (hard caps per tick and per wall second; selection a pure function, never production RNG; every sample names its policy); boundary (plain data behind `telemetry-seams`); storage and the commit rule (projected size per run, per-run cap); time (one trace per window, wall-time durations, tick on every span); run identity; overhead ceiling 25%; reference build. |
| Benchmark gate | `docs/workflow.md`, "Benchmark gate" | Observability exemption and its no-behavior-change condition; check triggers; the overhead method, 300 s cap and inconclusive rule. |
| F01 spec | `docs/specs/roadmap/t21-f01-local-telemetry-stack-and-run-identity.md` | The flag and endpoint; the reference build; the queue bounds (2,048 items, 8 MiB, 4 MiB body, one batch in flight, 5 s request, 10 s flush, no retry); exact per-run counts and the stderr line; identity attributes; the three check commands; T = 6908. |
| F02 spec | `docs/specs/roadmap/t21-f02-run-metrics-and-dashboards.md` | The environment-variable pattern (read only when telemetry resolves on, invalid refuses to start, recorded on `run.started`); the encoded queue item and per-item accounting; `scripts/telemetry-dashboards-check`. |
| F03 spec | `docs/specs/roadmap/t21-f03-tick-and-phase-tracing.md` | Trace items posted alone to `/v1/traces`; the identity attributes on every span; the seam pattern (`SimStats::last_tick_phases`, `TickPhaseTimings` with the tick's start, elapsed and phase offsets); the tick trace ID (run key high 64 bits, then the tick) that F04's IDs must not collide with; the `spans`-table dashboard pattern. |
| Research note | `docs/strategy/run-observability-research-2026-09-29.md` | Facts 1 and 2 (the recorder runs one creature sequentially and stores selected actions, not outcomes); the budget arithmetic; hash-selected windows under hard caps. |
| Recorder | `crates/v3-core/src/simulation/tick.rs` `run_tick`, `run_cognition`; `runtime/trace/{domain,recording}.rs`; `runtime/traced_mesh.rs` `RecordingMeshExecution` | `run_tick(sim, &mut Option<ActiveTrace>)`; the traced creature leaves the parallel batch, runs `execute_creature_mesh_traced` and its decision is reinserted at its queue position; a `TickTrace` per tick with static inputs, hops (`MeshHopTrace`: node, pass, energy, output slots, route with per-slot gate scores, vote contribution, backend trace), passes (`MeshPassTrace`: end reason, votes, effective votes, committed action, hops), final actions, termination reason, priority bid, commit counts; `record_hop` and `record_pass` are the mode's only recording points; a trace whose creature is gone at the tick's start ends (`ticks_remaining = 0`); mesh-level traced/untraced equivalence tests exist (`traced_mesh.rs`, `traced_vm_tests.rs`), tick-level ones compare only population and IDs (`tick/tests/trace.rs`). |
| Action phase | `tick.rs` `run_phase_2`, `push_action_log`, `execute_*`; `creature/action_log.rs`; `simulation/outcomes.rs`; `tick/helpers.rs` `remove_creature_if_dead`; `simulation.rs` `remove_creature` | Every living creature has an `ActionLog` (ring of `config.action_log.capacity`, default 500) whose entries carry tick, `ActionType`, `ActionResult` (9 variants with `as_key`), direction, amount, food type, energy before and after and the priority bid; `OutcomeAccumulator` holds per-creature damage received and offspring spawned; a creature whose energy reaches zero is removed inside the action loop and its log with it; `remove_creature` resolves the `DeathCause` from `pending_death_cause`. |
| Creature state | `crates/v3-core/src/creature/state.rs`, `identity.rs` | `position`, `energy`, `age`, `generation`, `shared_memory` (16), `previous_outcome` (4), `cached_genome_size`, `identity.{lineage_id,kin_tag}`; the genome serializes (`serde`); `CreatureId` is a slotmap key whose `data().as_ffi()` is the server's `u64` creature ID and whose slotmap iteration order is deterministic for a seeded run. |
| Server sampler | `crates/v3-server/src/http/creature.rs` `start_sample`, `get_sample`; `state.rs` `SimHandle.active_trace`; `http/lifecycle.rs` `step`, `run_loop` | `start_sample` (1 to 10 ticks) sets `active_trace`; `get_sample` takes a complete one and assembles the HTTP sample; both loops pass `&mut h.active_trace` to `run_tick` and call `after_tick`; `startup` clears it. |
| Run loops and exporter | `crates/v3-cli/src/{lib,telemetry}.rs`; `crates/v3-server/src/telemetry.rs`; `crates/v3-telemetry/src/{lib,queue,trace,testing}.rs` | The CLI passes `&mut None` to `run_tick`; `RunHandle` holds identity, `started`, `started_ns` and `traces_taken`; `Signal::{Metrics,Traces}` name items on gap lines; `trace::encode` and `Attributes` are the span pattern; the in-test `Receiver` decodes `/v1/traces` into `ReceivedSpan`s (no events yet). |
| Goal-world rates | `docs/progress/features/t20-f05-structured-heritable-refinement-goal.json` `per_creature_tick` | 9.87 mesh hops, 3.53 passes, 2.41 VM steps and 8.25 graph relax iterations per creature-tick: the sizing basis below. |
| Image | `grafana/otel-lgtm:0.34.0`, Tempo 3.0.3 (probed 2026-09-29 on scratch project `petri-telemetry-f04plan`, readings file) | Span events with attributes and array-valued attributes (doubles) are stored and returned whole by `/api/v2/traces/<id>`; TraceQL reaches events (`event:name`, `event.<attr>`) and array elements (`span.<attr> = v`); a `spans` table with `select()` on the root works as in F03; search finds a trace after the ~30 s block cut. |

Invariants:

1. No `SimulationConfig` or `RuntimeConfig` field, no production RNG draw,
   no simulation default, no stored summary content and no deterministic
   output changes; the config digest and the pinned recipe digests do not
   move. The traced path's decisions, energies and world effects equal the
   untraced path's for every creature.
2. Seam, all under `#[cfg(feature = "telemetry-seams")]` and plain data
   (table below). Before a tick is recorded the tick's reserve is charged;
   the recording mode then counts and sizes each hop and pass record before
   storing it. A record that would pass a cap is not stored, `truncated` is
   set once, and for the rest of the tick every node runs through the
   untraced executors with no further hop or pass record (the mode's
   backend-trace type becomes `Option`); the tick record and its reserved
   outcome are kept and the window ends after that tick. Execution is
   unchanged either way. `run_tick` fills the traced creature's
   `TickOutcome`; the sampler's `None` budget records as today with the
   outcome added.

| Seam item | Definition |
| --- | --- |
| `TraceBudget` | `max_events: u32`, `max_bytes: u64`; on `ActiveTrace` as `budget: Option<TraceBudget>` with counters `events: u32`, `bytes: u64` and `truncated: Option<Truncation { tick: u64, reason: Events \| Bytes }>` |
| Event | one hop, pass or applied-action record |
| Bytes | a record's in-memory size: its `size_of` plus every vector's `capacity × size_of` of its element type, recursively, backend trace and preallocation included |
| Tick reserve | charged against both caps before the tick's first hop: `max_actions_per_turn` events, and the bytes of the `TickTrace` fixed part, `max_actions_per_turn` selected and applied action records and the `TickOutcome`; a tick whose reserve does not fit is not recorded and the window ends before it, truncated (`Bytes` or `Events`), so retained events and bytes never exceed the caps |
| Window end on truncation | `ticks_remaining = 0` after the truncating tick; the creature returns to the parallel batch |
| Allocation bound | transient: a hop's backend trace by `max_vm_steps` or the graph's size, the mode's hop preallocation by `max_mesh_hops`; retained: the caps |
| `TickOutcome` (on `TickTrace` as `outcome: Option<TickOutcome>`) | position and age at cognition; `typed_local_food` (the `SensorSnapshot` vectors); the tick's applied actions in order, captured where `push_action_log` builds each entry (so independent of the log's capacity and of removal): type, result, direction, amount, food type, energy before and after, priority bid, and the realized accounting from the `EnergyFlows` deltas around the action: `charge` (`action_charges`), `penalty` (`failed_action_penalty`), `reward` (`food_intake_by_type` sum, `predation_attacker_credit` and `predation_kill_bonus_credit`); damage received (`observed_damage`, lethal or not); offspring spawned (`OutcomeAccumulator`); position, energy, `shared_memory` and `previous_outcome` after the tick, absent when the creature was removed; `died` from `observed_removal`; the tick's `TickPhaseTimings` |
| `SimStats::observed_creature` | `Option<CreatureId>`, set by `run_tick` at the tick's start to the trace's creature (else `None`), with `observed_removal: Option<DeathCause>` and `observed_damage: f32`, cleared by `reset_tick_counters`; `Simulation::remove_creature` (every removal path) and the predation site each compare the affected creature with `observed_creature` and record only for it, so the work is one compare per event and the memory two fields, whatever the population; `observed_actions: Vec<AppliedAction>` stages the tick's applied actions for it (bounded by `max_actions_per_turn`, cleared per tick) so `run_phase_2` keeps its signature; a creature gone at a tick's start ends the window with that tick's `observed_removal` |
| Perception | the window's `ActiveTrace` sets `include_perception_debug`, so `TickTrace.debug_perception` holds the banks; `TickOutcome` adds `typed_area_food` (per type, 7), `typed_local_food` and the two assembly predicates sensor assembly used (`uses_typed_local_food`, `uses_extended_perception`), so the encoder omits a bank the genome never reads (assembly gives it a zero snapshot; the HTTP sampler's payload is unchanged) |

3. Selection, in `v3-telemetry`. Window `k` (from 0) starts before the
   run's first tick and thereafter before the next tick once the interval
   has passed since the last admitted sample's start (window or manual),
   when no window or manual sample is active, the run is below its caps and the population is
   non-empty. Its creature is the `n`th creature in `sim.creatures`
   iteration order (slot order, deterministic for a seeded run),
   `n = splitmix64(seed ^ (k · 0x9E3779B97F4A7C15)) mod population`:
   a pure function of seed, window index and population, so the first
   window of a seeded run picks the same creature every time and nothing
   draws from a simulation RNG. A window ends early when the creature
   dies, the budget truncates, a manual sample starts, a server config
   patch is accepted, or the run ends (CLI completion, server reset or
   shutdown); the ticks recorded so far, zero included, are exported once
   with that end reason. Settings follow F02's rule: read only when
   telemetry resolves on, anything else refuses to start, each recorded on
   `run.started`.

| Setting | Values | Default | On `run.started` |
| --- | --- | --- | --- |
| `PETRI_TELEMETRY_CREATURE_WINDOWS` | `on`, `off` (`off` starts no window and exports no manual sample) | `on` | `petri.creature_windows` |
| `PETRI_TELEMETRY_WINDOW_TICKS` | `1` to `64` ticks per window | `8` | `petri.window_ticks` |
| `PETRI_TELEMETRY_WINDOW_INTERVAL_MS` | `10` to `3600000`, least wall time between window starts | `10000` | `petri.window_interval_ms` |

4. Wiring. The CLI loop and both server loops call
   `Telemetry::before_tick(run, sim, slot)`, which starts a window into the
   binary's `Option<ActiveTrace>` slot when due, pass the slot to
   `run_tick`, then call `after_tick`, which exports a window that ended.
   The server keeps its manual sampler: while `SimHandle.active_trace` is
   `Some` the loops pass it instead (table below).

| Sample rule | Definition |
| --- | --- |
| Start metadata | sample index, start tick, start instant, digest and genome hash, captured when a sample starts while the creature exists; the root carries that digest plus `petri.config_changed` when a patch landed before export; each tick span carries the digest in force when its tick ran (`after_tick` appends it per recorded tick) |
| Start | `start_sample` ends an active window (`manual`); the sample is admitted for export only when the window interval has passed since the last admitted sample's start (automatic or manual), else it serves HTTP as today, is not exported and `run.ended` counts it in `petri.samples_skipped`; an admitted sample captures the start metadata and emits the genome record; a sample it replaces is exported first as `replaced` |
| Export | exactly once, under the originating run: at `get_sample` hand-over (`complete`), or when replaced, cleared by reset or pending at shutdown (`replaced`, `run_end`), whichever comes first; `petri.sample_policy` = `manual`, the same encoder, the event cap applied at encoding (dropped events mark it truncated) |
| Caps | an admitted sample takes the run's next sample index and counts against the per-run caps like a window; one creature is recorded per tick either way; windows resume after it |

5. Identity without entropy. `k` is one per-run sample index, allocated
   at start and shared by windows and manual samples. The trace ID is the
   run key's high 64 bits followed by `2^63 + k` big-endian, and a tick
   trace is not captured for a tick at or above `2^63`, so the two never
   collide; sample `k` of a run is the first 16 hex digits of its run ID
   followed by `8` and `k` in 15 hex digits. Span IDs: root `1`; the
   `i`th recorded tick (from 0) `(i + 1) << 8`. Timestamps are offsets
   from the run's `started`: the root spans the sample's start instant to
   the last recorded tick's end, to the ending tick's end when the
   creature died before recording, or to the start instant itself when no
   tick ran; a tick span is its tick's seam start and elapsed; hop and
   pass events are stamped at the tick's cognition phase start, action
   events at its actions phase start.
6. Transport and accounting as F03: one `ExportTraceServiceRequest` per
   window encoded on the simulation thread, offered as `Signal::Windows`
   (item name `window`) and posted alone to `/v1/traces`; `exported`,
   `failed`, `dropped`, `abandoned` and `bytes` count it with the rest;
   the stderr line gains `windows=<taken> recorded=<creature-ticks>`;
   `self_time_us` includes selection, encoding and the genome record,
   which is emitted once when the sample starts, through F01's log path
   (table below).
7. Context: the tables below. Every span carries F03's identity row plus
   `petri.sample_policy`, `petri.window` and `petri.creature_id`; a value
   that was not captured is omitted. `petri.tick` on a tick span is the
   tick count after that tick, F03's convention, so the tick trace of the
   same tick has the ID composed from the run ID and that value.
8. Caps (table below), so cost does not grow with population or tick rate.
   Per-run bytes count every encoded window request and genome body at
   encoding, whatever the exporter does with them; a body over the queue's
   4 MiB cap is dropped whole, counted and named on stderr and counts as
   its 4 MiB reservation, so a sample adds at most 8 MiB; the event cap
   keeps a window under about 1.5 MB encoded.

| Cap | Value | When reached |
| --- | --- | --- |
| Events per window | 2,048 | truncated (invariant 2) |
| In-memory record bytes per window | 4 MiB | truncated (invariant 2) |
| Samples per run (windows and manual) | 4,096 | no sample starts; `run.ended` carries `petri.windows_capped` = `count` |
| Encoded bytes per run (windows and genome records) | 256 MiB; a sample is admitted only while the count plus two 4 MiB bodies (its genome record, its window request) fits, so the last sample cannot cross it | as above, `bytes` |
| Window starts per wall interval | 1 | the next start waits for the interval |
| Creatures recorded per tick | 1 | by construction |

9. Dashboard: `telemetry/grafana/dashboards/petri-run.json` gains a row
   `Creature windows` with a `spans` table on the `tempo` datasource,
   `{ span.petri.run_id = "$run_id" && name = "creature_window" } | select(span.petri.window, span.petri.creature_id, span.petri.sample_policy, span.petri.ticks_recorded, span.petri.end_reason, span.petri.truncated)`,
   limit 500, one row per window whose span link opens the trace; still
   provisioned read-only under the fixed UID.

**Spans, events and records.** Arrays are OTLP array values in the index
order of the source; `VoteVector` arrays are in `VoteSink` order (27),
commit counts and effective votes in `VoteKind` order (4).

| Span or event | Attributes and source |
| --- | --- |
| every span | F03's identity row (`petri.run_id`, `petri.seed`, `petri.world`, `petri.recipe`, `petri.config_digest`: the start digest on the root, the tick's on a tick span, resource attributes); `petri.tick`; `petri.phase` = `cognition` (the code the hops and passes describe; `actions` on `action` events); `petri.sample_policy` = `window` or `manual`; `petri.window` = `k`; `petri.creature_id` (decimal string of the ffi ID) |
| `creature_window` (root) | `petri.tick` = first recorded tick (the ending tick when none), `petri.tick_end` = last; `petri.lineage_id`, `petri.kin_tag`, `petri.generation`, `petri.age_start`, `petri.genome_size`, `petri.genome_hash`, `petri.genome_dropped` when dropped (at start); `petri.ticks_requested`, `petri.ticks_recorded`, `petri.end_reason` = `complete`, `died`, `truncated`, `manual`, `replaced`, `config_change`, `run_end`; `petri.died` (cause) when the creature died in the window; `petri.truncated` (bool), `petri.truncated_reason`, `petri.truncated_tick` when set; `petri.events`; `petri.config_changed` when set; `petri.samples_skipped` is on `run.ended`, not here; F03's `cognition` and `actions` config groups (`petri.config.runtime.*` including `vm.*`, `petri.config.energy.*`, `population`, `mutation`, `predation`), read at start |
| `creature_tick` (child) | `petri.tick`; sensed: `petri.food_here`, `petri.neighbor_food`, `petri.neighbor_barrier`, `petri.neighbor_occupied` (8), `petri.age_ticks`, `petri.previous_outcome` (4) from `static_inputs`; `petri.food_here_by_type` (per type) and `petri.neighbor_food.<i>` (8 per type) from `typed_local_food`; `petri.area_food`, `petri.area_barrier`, `petri.area_occupancy` (7), `petri.nearby_core` (16), `petri.nearby_vitals` (8), `petri.nearby_identity` (12) from `debug_perception` and `petri.area_food.<i>` (7 per type) from `typed_area_food`, all omitted when extended perception was not assembled for the genome; the typed-food attributes likewise when typed local food was not; `petri.position` (`[x, y]`), `petri.age`, `petri.energy_start` (`TickTrace.energy_before`), `petri.energy_after_cognition`, `petri.failed_action_penalty` (the tick's); decided: `petri.actions_selected` (array of action strings), `petri.termination_reason`, `petri.priority_bid`, `petri.commit_counts`, `petri.hops`, `petri.passes`; outcome: `petri.actions_applied`, `petri.damage_received`, `petri.offspring_spawned`, `petri.died` (cause) when removed; state after: `petri.position_end`, `petri.energy_end`, `petri.shared_memory` (16), `petri.previous_outcome_end` (4), omitted when removed |
| event `hop` | `petri.hop`, `petri.pass`, `petri.node`, `petri.backend` = `vm` or `graph`, `petri.energy_before`, `petri.energy_after`, `petri.output_slots`, `petri.vote_contribution`; when the hop routed: `petri.route.slots`, `petri.route.targets`, `petri.route.scores` (effective) as aligned arrays in the recorded order and `petri.route.selected` (index into them); omitted when it did not |
| event `pass` | `petri.pass`, `petri.end_reason`, `petri.hops`, `petri.votes`, `petri.effective_votes`, `petri.committed` (action string) when any |
| event `action` | `petri.index`, `petri.action` (`ActionType::as_key`), `petri.result` (`ActionResult::as_key`), `petri.direction` (0 to 7; omitted for none), `petri.amount`, `petri.food_type`, `petri.energy_before`, `petri.energy_after`, `petri.charge`, `petri.penalty`, `petri.reward` |
| record `creature.genome` | identity, `petri.tick`, `petri.window`, `petri.creature_id`, `petri.genome_hash`, `petri.genome_size`; body the genome's compact JSON with keys sorted recursively as `config_digest` does; the hash is `sha256:<hex>` of that text and is always on the root; a body over the 4 MiB cap is dropped whole, counted and named as F01 says, with `petri.genome_dropped` = true on the root (`mutation.genome_size_cap`, 1,200 units by default, bounds it; bytes per unit measured in the readings file) |

Projected size, at the goal-world rates: a tick span about 2.5 KB (the
sensed arrays about 1 KB), a hop event about 500 B, a pass event about
600 B, an action event about 200 B, so about 9.5 KB per recorded
creature-tick and about 80 KB per default window plus a genome record,
estimated under 64 KB for a founder genome until measured at closure; at
the default cadence about 360 windows and about 50 MB per hour; the
workload run: one window and one genome record, about 150 KB. The measured
`bytes=`, `windows=` and `recorded=` figures, the genome body size and
Tempo's and Loki's growth over ten runs go in the readings file.

**Check command.** `scripts/telemetry-dashboards-check` gains the rows
below and keeps its scratch project.

| Step | What it does or asserts |
| --- | --- |
| Panel query | the `Creature windows` query is polled like `Tick traces` and its row count recorded (at least one row) |
| Trace content | the first row's trace from `/api/v2/traces/<id>` has one `creature_window` span and as many `creature_tick` spans as its `petri.ticks_recorded`, each with `petri.run_id`, `petri.tick`, `petri.sample_policy` = `window`, at least one `hop` event and one `pass` event across them; the ID composed from the run ID and `petri.window` answers with the same spans |
| Join | the trace ID composed from the run ID and the first tick span's `petri.tick` answers with a tick trace when that tick had a snapshot, else the row records that it had none |
| Genome record | the Loki `creature.genome` line of the run carries the root's `petri.genome_hash` |
| Run record | the Loki `run.started` line carries `petri_creature_windows`, `petri_window_ticks`, `petri_window_interval_ms` |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `creature_window` span | span, sampled by window | the recorder's window for the selected creature | identity, window, creature identity, genome hash, end reason, truncation, cognition config |
| `creature_tick` span | child span per recorded tick | `TickTrace` and its `TickOutcome` | identity, tick, sensed inputs, selected actions, applied actions, damage, death, state after |
| `hop`, `pass`, `action` events | span events | `MeshHopTrace`, `MeshPassTrace`, the tick's `ActionLog` entries | routing, votes, applied result and cost |
| `creature.genome` | log record per window | the creature's genome at window start | identity, window, creature, hash, size |
| `run.started`, `run.ended` | log records (F01) | gain the three settings; `petri.windows_capped` when a cap was reached | the sampling configuration and cap |
| exporter self-report | stderr line (F01) | gains `windows=` | windows taken |

Windows consume no production RNG, change no decision and name their
policy; a value that was not captured is absent, never zero. Nothing here
is closure evidence.

## Implementation Tasks

- [x] `cargo test -p v3-core --test viability` first, then `v3-core`:
      `TraceBudget`, `Truncation`, `TickOutcome`, the budget checks in
      `RecordingMeshExecution` and the outcome join in `run_tick`, all
      behind `telemetry-seams`; the tick-level equivalence, budget and
      outcome tests under the feature (the existing `make check` row).
- [x] `crates/v3-telemetry`: the three settings in `Options`; window
      selection, `before_tick`/`after_tick` and the manual hand-over API;
      `windows.rs` encoding (spans, events, arrays, genome record);
      `Signal::Windows`, `windows=`, per-run caps, `petri.windows_capped`;
      the in-test receiver decodes span events.
- [x] `v3-cli` and `v3-server`: the window slot around `run_tick`; the
      server's manual precedence and export at `get_sample`.
- [x] `telemetry/grafana/dashboards/petri-run.json`: the `Creature windows`
      row; `scripts/telemetry-dashboards-check`: the rows above.
- [ ] Run the checks on this host; record them in
      `docs/progress/readings/t21-f04.md`.

## Verification

- [ ] Viability: `cargo test -p v3-core --test viability` -> passes, run
      before any other test of the tick-loop change.
- [ ] Seam: `cargo test -p v3-core --features telemetry-seams --lib` (a
      `make check` row) -> the seam rows of the table; `cargo test -p
      v3-core --lib` still passes without the feature.
- [ ] Neutrality and window content: `cargo test -p v3-cli` (inside `make
      check`, no Docker) -> the CLI rows.
- [ ] Encoding and bounds: `cargo test -p v3-telemetry` -> the encoding
      rows.
- [ ] Server: `cargo test -p v3-server` -> the server rows.
- [ ] Reference build: `cargo build --release -p v3-cli -p v3-server -p
      v3-lab --no-default-features` compiles, `cargo tree -p v3-cli -p
      v3-server -p v3-lab --no-default-features -e features` shows neither
      `v3-telemetry` nor `telemetry-seams`, and clippy `-D warnings` passes
      with and without the feature.
- [ ] Dashboards (Docker, outside `make check`):
      `scripts/telemetry-dashboards-check` -> every row passes, including
      the five above; the `Creature windows` panel and one window opened
      from it, with its events visible, recorded as a checklist row in the
      readings file.
- [ ] Bytes per run: the workload run's `bytes=` and `windows=`, the genome
      record's body size, and Tempo's and Loki's growth over ten runs, in
      the readings file.
- [ ] Parent comparison: `scripts/telemetry-parent-compare 2d640fe9` (F03's
      closing commit) -> identical canonical output, in the readings file.
- [ ] Overhead check: `scripts/telemetry-overhead` -> the table in the
      readings file, verdict in Performance and Goal Impact; then the
      per-creature-tick reading.
- [ ] `make check` -> exit 0.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: `Not applicable:
      observability feature`.
- [ ] Benchmark summary: `Not applicable: observability feature`.

| Test rows | What is asserted |
| --- | --- |
| Seam: equivalence | two seeded simulations (VM and graph founders) run 50 ticks, one with a budgeted trace whose target moves to a new creature every 8 ticks and one with `None`; at every tick the traced creature's decisions equal the untraced run's and at the end tick, creature keys, energies, positions, ages, shared memory, `previous_outcome`, `graph_runtime` state and cumulative counters are equal; the same with an event cap of 3 under `max_actions_per_turn` = 1 (the tick reserve holds `max_actions_per_turn` events, so the default 10 never fits a cap of 3) and with a `None` budget |
| Seam: outcome | a tick's outcome lists the actions `push_action_log` built in order, with an `action_log.capacity` of 0 and of 1 and an action-heavy tick; charge, penalty and reward match the `EnergyFlows` deltas for a rewarded eat, a failed move and a steal; a creature that dies in the action phase keeps the actions applied before its death with `died` set; a traced victim killed before and after its own turn has `died` = `Predation` and its `damage_received`, while an untraced tick records no removal or damage; a creature that dies in phase 0 ends the window with zero records and the cause; with two food types the outcome's `typed_area_food` and `typed_local_food` hold distinct per-type values |
| Seam: budget | an event cap of 3 (`max_actions_per_turn` = 1) and a byte cap below one hop's size each truncate at the predicted tick and hop, the rest of that tick records nothing, the window ends after it and the simulation state equals the untraced run's; a tick whose reserve does not fit either cap is not recorded and the window ends before it; a `None` budget records every hop |
| CLI: neutrality | canonical NDJSON identical with telemetry off, on with the in-test receiver, and on with a closed port, windows on |
| CLI: content | the receiver holds a window trace with a `creature_window` root, as many `creature_tick` children as `petri.ticks_recorded`, the IDs of invariant 5, `hop`, `pass` and `action` events, the sensed arrays (`petri.food_here_by_type` and `petri.neighbor_food.<i>` for each of two food types; the area banks absent for the founder, which never reads them), and a `creature.genome` record with the root's hash; two `on` runs of the same seed pick the same first creature; a run shorter than the window exports it with `run_end`; `recorded=` equals the sum of `petri.ticks_recorded`; `PETRI_TELEMETRY_CREATURE_WINDOWS=off` yields no window and `windows=0`; an invalid setting is refused |
| Encoding | a synthetic window encodes the predicted trace and span IDs, parents, timestamps, names, attribute keys, arrays and the config group values, and a zero-tick window its root alone spanning its start instant; alternating window and manual samples get distinct IDs; a manual sample past 2,048 events is cut at encoding and marked truncated; the 4,097th sample, and a sample when more than 248 MiB are counted (one at exactly 248 MiB is), are not admitted and `run.ended` says which cap, with genome bodies counted; a genome body over 4 MiB is dropped and the root marked; a window item over the body cap is dropped and counted (the queue-bounds drop is the `Item::Encoded` path F03's test covers); the oversized-genome case runs with `Limits.max_body_bytes` lowered; a failed or rejected post is `failed`; a tick trace is not captured at tick `2^63` |
| Server | a window exports under `step`; `start_sample` ends an active window as `manual` and the completed manual sample is exported once, with policy `manual` and its start digest, when `get_sample` hands it over, then windows resume; a second `start_sample` exports the first as `replaced`; a `start_sample` inside the interval serves HTTP, exports nothing and counts in `petri.samples_skipped`; a config patch ends a window as `config_change` and a manual sample spanning one carries `petri.config_changed` on the root and the new digest on the later tick spans; reset and shutdown export a pending window and an unfetched manual sample as `run_end` under the old run; the payload neutrality test still passes |

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary; the reviewer checks that against the diff, and
that every `v3-core` change sits behind `telemetry-seams` and every
`v3-cli`, `v3-server` and `v3-telemetry` change behind `telemetry`.
Measurement feature: the natural-analog rule and the environmental-pressure
rule do not apply, and no indicator can move.

Parent comparison: runs because F04 edits `run_tick`, `run_cognition` and
the action phase; `BASE` is `2d640fe9`. Expected: identical canonical
output.

Overhead check: runs because the seam adds work to the traced tick and a
window adds selection, capture and encoding. The method is F01's,
unchanged, with T = 6908 fixed:

| Item | Predeclared |
| --- | --- |
| Workload, reference, spread, pairs, states, reading, verdict, rerun, time cap, inconclusive | as the F01 spec's table: `--no-default-features` reference run with the stack stopped; n = 5 at spread ≤ 5%, 8 at ≤ 10%, else inconclusive; states (a) on, stack healthy, (b) on, stack stopped, (c) idle-stack block; pass at median ≤ 1.25 with ⌈0.75 n⌉ pairs ≤ 1.25; one rerun; 300 s cap; inconclusive is reported, never rounded |
| Configuration under test | the defaults: windows on, 8 ticks, 10 s interval, tick traces on, metrics interval unset; one window (8 recorded creature-ticks) and one genome record per 3.9 s workload run |
| Attribution block | readings, not gates, taken after the overhead check with the stack stopped and charged to the same 300 s cap, only if at least 40 s of it remain, else reported as not taken: (i) two interleaved pairs of telemetry on with `PETRI_TELEMETRY_CREATURE_WINDOWS=off` against the reference build, the median ratio being the unconditional cost of the seams and F02/F03 work with nothing recorded; (ii) two interleaved pairs of windows back-to-back (`PETRI_TELEMETRY_WINDOW_INTERVAL_MS=10`, `PETRI_TELEMETRY_WINDOW_TICKS=64`) against windows off, both telemetry on; the reading is the median of each pair's (wall difference / the `recorded=` count of the windows run), so partial, dead or truncated windows are counted as recorded |
| Expected | (a) and (b) within noise of 1.0; self time below 6 µs per tick (F03's 0.4 µs plus one window encode and one genome record, each under 10 ms, over 6,908 ticks); (i) within noise of 1.0; (ii) between 50 µs and 1 ms per recorded creature-tick (one creature's cognition run serially after the batch, plus trace allocation) |

**Measured verdict.**

| Record | Value |
| --- | --- |
| Profiles | `Not applicable: observability feature` |
| Parent comparison | pending |
| Overhead check | pending |
| Self-timed cost per tick | pending |
| Attribution block (i), (ii) | pending |

- Readings: [`docs/progress/readings/t21-f04.md`](../../progress/readings/t21-f04.md).

## Success Criteria

- [ ] With windows on, the stack healthy, the queue unsaturated and the
      per-run caps not reached, a `v3-cli run --telemetry on` run of at
      least one tick with a living population and a `v3-server --telemetry
      on` run store in Tempo
      one `creature_window` trace per window, with a `creature_tick` span
      per recorded tick carrying the attributes and events of the table and
      fetchable by the ID composed from the run ID and the window index,
      and in Loki one `creature.genome` record per window; a server manual
      sample is stored with policy `manual`; with the stack stopped or the
      queue full the run keeps its speed and the missing windows are
      counted.
- [ ] `Petri / Run` lists the run's windows and opens one, and
      `scripts/telemetry-dashboards-check` passes with its new rows.
- [ ] The viability, seam, neutrality, encoding, bounds and server tests
      pass inside `make check` without Docker; `v3-core` changes only
      behind `telemetry-seams`, and the reference build compiles them out.
- [ ] The parent comparison is identical and the overhead check, the
      self-timed cost and the per-creature-tick reading are recorded with
      the verdicts above.

## Notes for AI Agents

- Decision: a window's trace ID is the run key's high 64 bits followed by
  `2^63 + k`; tick traces keep the low 64 bits below `2^63`, so the two
  kinds never collide and no ID is drawn.
- Exception: outside `telemetry-seams`, because a `cfg` cannot split a
  signature: `push_action_log` and `execute_*` return the entry they
  build; `RecordingMeshExecution` uses `Option<BackendTrace>` and returns
  `RecordedMesh`. No work is added to the untraced tick.
- Decision: the server's manual sampler exports its samples labelled
  `manual`, takes precedence over an automatic window and shares the
  window interval as its admission; a manually targeted creature is never
  read as a representative one.
- Exception: plan committed after round 3 `not-ready` (user, 2026-09-29).
  No new blocking finding; the two upheld items were fixed as Codex
  proposed and are unconfirmed; details in the readings file.
