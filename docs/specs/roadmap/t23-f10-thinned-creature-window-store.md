# T23.F10 — Thinned Creature Window Store

**Status**: In Progress
**Last updated**: 2026-10-01
**Feature**: T23.F10
**Track**: [T23 — Behavior and Cognition Readings](../../roadmaps/t23-behavior-and-cognition-readings.md)

## Goal

When a creature window or manual sample ends, the process sends one compact
`creature.window` log record of it to Loki, for the whole run rather than only
until T21.F04's per-run trace caps. A job in the telemetry stack thins each
run's records by age in window indexes: all of the newest 300, the even indexes
of the next 300, and every index divisible by ten beyond that. It holds each
run's stored record bytes at the 500 MB cap by thinning the oldest records below
one in ten. A long run keeps its recent windows in full and a nested sample of
its whole history.

## Non-Goals

- Reading windows from Loki in the viewer (F09), the F08 readings, any
  dashboard, panel or Prometheus series.
- Thinning Tempo traces, `creature.genome` records or any record other than
  `creature.window`. T21.F04's caps on traces and genome records stay as they
  are.
- Changing how windows are selected, their interval or their tick count. No
  new `PETRI_TELEMETRY_*` setting or preset value.
- Any change to `v3-core`, simulation behavior or deterministic output.
- Weighting retained records in a reading. The track's weights of 1, 2 or 10
  belong to the reader (F09), which computes them from `N` and the stored
  indexes. A record lost in transport is not distinguished from a thinned one.
- Committing, or writing into the repository tree, any exported record or job
  output.

## Inputs and Invariants

| Input | Where | What F10 takes from it |
| --- | --- | --- |
| Track row and Notes | `docs/roadmaps/t23-behavior-and-cognition-readings.md`: F10 row; Notes "Window store" (user decisions 2026-10-01), "Window prototype", "Verification", "Ownership"; D6 and D7 in "Open user decisions" | Loki for window details; keep levels and nested bands; one delete request per band; a POSIX `sh` loop as an extra Compose service; about 4 KB a record; readings weight kept records (the reader's concern, F09); per-run cap 500 MB, thinning the oldest band below the 10% floor at the cap; F09 reads retained windows from Loki once F10 lands, which removes its dependence on T21.F04's lifetime window budget |
| Exemption and track checks | `docs/workflow.md`, "Benchmark gate" | No gate or goal profile and no mutation gate while no simulation behavior changes; the neutrality test, parent comparison and overhead check stand in; the 10% ceiling |
| Windows | `crates/v3-telemetry/src/windows.rs` (`Samples::fits_caps`, `start_sample`, `export`, `after_tick`, `start_manual`, `end_samples`, `encode`, `action_label`); T21.F04 spec invariant 3 | Sample index `k` is shared by windows and manual samples and is allocated at start. Only one window and one manual sample are open at a time. `MAX_SAMPLES_PER_RUN` = 4,096 and `MAX_SAMPLE_BYTES_PER_RUN` = 256 MiB currently stop admission. A sample emits its `creature.genome` record at start and its trace at end |
| Records | `crates/v3-telemetry/src/lib.rs` `emit_with`, `emit_record`; `queue.rs` run line | Log records carry run identity attributes and go through the bounded queue (2,048 items, 8 MiB, drop when full). The stderr run line lists per-run counts |
| Prototype shape | `telemetry/reports/creature-windows.html` `parseWindow` | The fields the viewer and readings use from a window: rings, food underfoot, selected actions, applied results paired by index, energy, and hops as node, backend and vote magnitude |
| Stack | `telemetry/compose.yaml`; `scripts/telemetry-cleanup`; `scripts/telemetry-scratch.sh`; `scripts/telemetry-cleanup-test` | Pinned `grafana/otel-lgtm:0.34.0` (Loki 3.7.8, with `sh`, `curl`, `awk`, `sed` and GNU `date`, but no `jq`). Loki retention and the delete API are on (`filter-and-delete`, 1 m cancel period). Only `service_name` is a stream label; record attributes are structured metadata (`petri_run_id`, `petri_window`, …). Scratch stacks; a Docker-free stub-test pattern |
| Loki deletion | Loki 3.7 `pkg/compactor/deletion/delete_request.go`, `pkg/logql/syntax/ast.go` (readings file) | A delete query is a full log pipeline. Label filters on structured metadata count as filters and are evaluated per line, numeric comparisons included. Filtered requests split into slices of at most 24 h. Queries hide pending deletions |

Invariants:

1. Exemption. No `SimulationConfig` or `RuntimeConfig` field, no production RNG
   draw, no simulation default, no stored summary content and no
   deterministic-output change. The record is built in `v3-telemetry` from the
   ended sample's `ActiveTrace` and `SampleMeta`, at the points where the sample
   is exported now, behind the binaries' `telemetry` feature. `v3-core` is not
   changed.
2. Admission. T21.F04's selection, interval, single-open-sample and population
   conditions are unchanged. The two per-run caps no longer stop admission.
   They stop the trace and the `creature.genome` record of later samples, and
   `run.ended` `petri.windows_capped` still names the first cap met. A manual
   sample is skipped only inside the interval. The sample index keeps counting
   for the whole run. This changes T21.F04 invariant 3's "below its caps"
   admission condition, as the user approved on 2026-10-01 (Notes).
3. Record. Every sample that ends, whatever its end reason and including one
   with zero ticks, emits exactly one `creature.window` record through the
   existing log path, with the attributes in the Telemetry table and the body
   below. Values keep their f32 value in shortest round-trip form, and a value
   that was not captured is omitted. Hops are the only unbounded part: a
   record keeps at most 2,048 hops, and past that it cuts them and sets
   `truncated`, which matters for a manual sample because it records with no
   budget. Selected actions and applied results are never cut, since each tick
   holds at most `max_actions_per_turn` of them. Ticks are at most 64 for a
   window and 10 for a manual sample. A body over the queue's 4 MiB cap is dropped and counted, as any
   item is. At the default 8 ticks a record is about 4 KB, and the size is
   measured at closure.
4. Keep level. The record for index `i` carries `petri.keep` = 2 if `i` is
   divisible by 10, else 1 if `i` is even, else 0.
5. Keep set. Let `N` be the highest index among a run's records that Loki's
   queries return. A record `i` is retained if `N - i < 300`, or if
   `N - i < 600` and its keep level is at least 1, or if its keep level is 2,
   unless the cap rule (invariant 7) removed it. As `N` grows, the bands and the
   cap only remove records, so the store never needs back a record it deleted.
6. Job. The job keeps no state outside its process and runs one pass every
   `PETRI_WINDOW_STORE_PASS_S` seconds (default 600). A pass lists two kinds of
   run. The first is every run with `creature.window` records timestamped no
   earlier than 3 h before the last successful pass started, or every run there
   is on the job's first pass. The 3 h margin exceeds a record's worst delay
   from its timestamp to Loki. A record is stamped when it is queued (`lib.rs`
   `emit_record`). It then waits for at most one request already in flight
   (taken off the queue) and 2,047 queued items ahead of it, which can include
   encoded metrics and traces posted singly, before its own request. Each
   request is one item or batch, one in flight, with a 5 s timeout and no
   retry, so the delay is at most 2,049 × 5 s = 10,245 s. A full queue drops an item; it does not delay
   it. Windows come only from `v3-cli run` and the server,
   whose exporters are never held (`start_held` serves measurement commands,
   which record no windows). The second is every run that a previous pass left
   unsettled. A run is settled once a pass finds nothing to delete and its
   bytes at or under the cap, or its cap logged as unreachable. For each
   of those runs it reads `N` and, for each band (300–599, 600 and older), the
   records still returned by queries that the keep set no longer retains.
   Queries hide records under accepted delete requests, so what a band still
   returns is exactly the work not yet requested. Late or out-of-order arrivals
   are therefore picked up by a later pass, and a partial or crashed pass leaves
   nothing to roll back. For each band with such records, the pass sends one
   delete request. Its label filters select the service,
   `event_name="creature.window"`, `petri_run_id`, the band's index range and
   the keep levels below the band's. Its time range spans the selected records'
   timestamps, rounded outward. The filters decide what is deleted, never the
   time range. No request selects another event, another run, or a retained
   record. Discovery and counting use aggregate (metric) queries, so Loki's
   5,000-entry query limit never truncates a run.
7. Cap. A run's stored bytes are the `bytes_over_time` that Loki's queries
   report for its `creature.window` records. When they exceed
   `PETRI_WINDOW_STORE_CAP_BYTES` (default 500,000,000), the pass sends one cap
   request. That request deletes, at age 600 or more, the records whose index is
   not divisible by `10^t`, for the lowest tier `t >= 2` that still has such
   records. It covers the shortest prefix from the oldest record whose bytes
   cover the excess, at the granularity of the job's byte query. If the whole
   tier is not enough, the next tier waits for the next pass. Records at age
   under 600 are never cap-thinned, and index 0 survives every tier. When those
   records alone exceed the cap, the job thins everything else and logs that
   the cap cannot be reached. Between passes, and while a request waits to be
   applied, a run may exceed the cap by what it wrote since the last pass.
8. Failure. A failed Loki call ends that run's pass and is logged. The next
   pass recomputes everything from what queries return. A request that was
   accepted but whose response was lost is hidden from the next pass's query
   like any accepted one. A duplicate request that slips through is harmless.
9. Isolation. Tests and the F10 stack verification never write to or delete
   from the `petri-telemetry` stack or volume, and they run beside it: the
   verification's scratch stack publishes no host ports and is reached through
   `docker compose exec`. Closure does not restart the user's running stack;
   the job starts at the next `make telemetry-up`.

| Body field (`creature.window`, compact JSON) | Content |
| --- | --- |
| `v` | schema version, `1` |
| `window`, `policy`, `creature`, `ticks_requested` | sample index, `window` or `manual`, creature ID, requested ticks |
| `lineage`, `generation`, `genome_size`, `genome_hash`, `age_start` | the sample's start attributes (`SampleMeta.root`) |
| `end`, `died`, `truncated`, `config_changed` | end reason; death cause when present; `{reason, tick}` when truncated; `true` when a patch landed |
| `ticks[]` | one entry per recorded tick, in order |
| `ticks[].t`, `e0`, `e1`, `here` | tick count after the tick (as on the tick span), energy at start, energy at end (absent when removed), food underfoot |
| `ticks[].food`, `food_by_type`, `barrier`, `occupied` | the sensed rings: summed food, per-type food rings when the genome reads typed local food, barrier, occupancy |
| `ticks[].sel`, `res` | selected actions (`action_label`); applied results in order (`as_key`), where entry `i` pairs with `sel[i]` and an action with no result was not applied |
| `ticks[].hops` | per hop in order: `[node, "v" or "g", sum of the absolute vote contributions]` |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `creature.window` | log record, one per ended sample | the ended sample's recording and start metadata | run identity attributes, `petri.tick` (tick count at the end), `petri.window`, `petri.keep`, `petri.sample_policy`, `petri.creature_id`; body as above |
| `window_records=` on the stderr run line | count | `creature.window` records offered | per run, beside `windows=` (traces) |
| `petri.windows_capped` on `run.ended` | existing attribute | the first T21 cap met | now means the cap that stopped traces and genome records, not windows |
| Job pass line | job stdout (`docker compose logs`) | each pass | per run touched: `N`, the requests issued by band, stored bytes, any error |
| Recorded configuration | `telemetry/compose.yaml` | the job service | `PETRI_WINDOW_STORE_PASS_S`, `PETRI_WINDOW_STORE_CAP_BYTES`. These are stack settings, not run settings; the band sizes are fixed constants |

## Implementation Tasks

- [ ] Producer: admission past the caps (invariant 2); building and emitting
      the record (invariants 3 and 4); the `window_records=` count; tests.
- [ ] Job: `scripts/telemetry-window-store`, a POSIX `sh` script with a
      `--once` mode; a `window-store` service in `telemetry/compose.yaml` that
      runs the stack's lgtm image (`PETRI_LGTM_IMAGE`) with the script mounted
      read-only, reaches Loki over the Compose network and has small resource
      limits; the compose header updated; a Docker-free stub test
      `scripts/telemetry-window-store-test` run by `quality-check`.
- [ ] Scratch-stack verification of the job on a portless scratch stack
      (invariant 9), recorded in the readings file.
- [ ] Track checks, `make check`, spec and readings updated.

## Verification

- [ ] `cargo test -p v3-telemetry`, `-p v3-cli`, `-p v3-server` (inside `make
      check`) -> the rows below.
- [ ] `scripts/telemetry-window-store-test` (inside `make check`) -> the job
      rows below.
- [ ] Scratch stack (Docker, outside `make check`) -> the stack rows below,
      in the readings file.
- [ ] Reference build: `cargo build --release -p v3-cli -p v3-server
      --no-default-features` compiles and `cargo tree` shows no `v3-telemetry`.
- [ ] Parent comparison: `scripts/telemetry-parent-compare <merge base>` ->
      identical canonical output.
- [ ] Overhead check: `scripts/telemetry-overhead` -> verdict against 1.10.
- [ ] Bytes: record body sizes (median and maximum) and the run line counts
      from one default CLI run, in the readings file.
- [ ] `make check` -> exit 0.
- [ ] Mutation gate: `Not applicable: observability feature`.
- [ ] Benchmark summary: `Not applicable: observability feature`.

| Rows | What is asserted |
| --- | --- |
| Record | a synthetic window's record holds every body field above with exact values, its attributes and keep levels for indexes 0, 1, 2, 10 and 15; omitted rings and `e1` stay absent; a zero-tick sample emits a record; a manual sample's record has policy `manual`; a manual sample with more than 2,048 hops keeps every selected action and applied result, cuts hops and sets `truncated` |
| Caps | with the caps lowered for the test, the samples past the count cap and past the byte cap are still admitted and emit records, with no trace, no genome record and `petri.windows_capped` set; a manual sample past a cap is not skipped |
| CLI | neutrality: canonical NDJSON identical with telemetry off, on, and on with a closed port; the receiver's `creature.window` count equals `window_records=`; `PETRI_TELEMETRY_CREATURE_WINDOWS=off` gives none |
| Job (stubbed Loki) | no request while `N < 300`; a sparse history (indexes 1 and 1,000 only) deletes index 1; one request per band with run, event, index-range and keep-level filters and a time range from the selected records; no request when the queries return nothing outside the keep set; a settled run with no newer arrival is not listed after the first pass, while an unsettled one is, even with no new arrival; a record timestamped 2.9 h before the last pass started is found; a failed call sends no further request for that run; over the cap, one request thins the oldest records at age 600 or more to 1 in 100, then 1 in 1,000 once the first tier is exhausted, never touching age under 600 or index 0; an unreachable cap is logged |
| Stack | on a portless scratch stack beside the running production stack: one run of 6,000 synthetic records (past the 5,000-entry query limit) and one of 1,300 with index gaps and interleaved timestamps, plus non-window records, sent over OTLP; after the passes and Loki's filtering, each run's queried indexes equal the keep set and nothing else changed; a further pass issues no request; records sent in two batches, the second holding a lower index, end in the keep set; with a small cap, an idle finished run needing two tiers thins nested to 1 in 100 and then 1 in 1,000 over successive passes, and its queried bytes end at or under the cap; pass time and request count recorded |

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior (invariant 1).
The parent comparison (`scripts/telemetry-parent-compare`, T21 workload,
telemetry off) is expected byte-identical.

Overhead check method: `scripts/telemetry-overhead` in routine mode, as T21
defines it. That means the T21 workload at preset `standard`, the script's
interleaved pairs with their count set by its five-run spread measurement,
and every stack state it runs. It passes when the 90% interval's upper bound
of the median ratio is below 1.10, fails when the lower bound is at or above
1.10, and is otherwise inconclusive. It runs inside the 5-minute cap, and an
inconclusive result is reported to the user, not rerun.

Expected: a pass with no change detectable against the parent. The added
work per window is one compact JSON encoding of about 4 KB, once every 10 s by
default. The routine check measures a short run that stays inside the trace
caps, so its coverage is the pre-cap path. Past the caps, the parent recorded
no windows, so a long run gains window work there that the check does not
measure. Per window, that work is a subset of the pre-cap work: recording and
the record, with no trace encoding and no genome record. Its share of a run
still depends on wall-time cadence and simulation throughput.
No indicator, goal-profile reading or epoch moves. The job runs outside every
process and timed region.

**Measured verdict.** Pending.

- Full readings: [`docs/progress/readings/t23-f10.md`](../../progress/readings/t23-f10.md).

## Success Criteria

- [ ] Every ended sample emits one `creature.window` record, including samples
      after T21.F04's trace caps.
- [ ] On a scratch stack, the job leaves each run's stored records equal to the
      nested keep set, with at most one request per band per pass and none
      once nothing is left to delete.
- [ ] The 1-in-10 floor holds until the per-run byte cap binds; then the
      oldest records thin further, nested. For a
      reachable cap, the run's stored bytes converge to at or under the cap
      within the passes the tiers need, with or without new arrivals. An
      unreachable cap is logged.
- [ ] The neutrality test, parent comparison and overhead check pass, and
      `make check` exits 0.

## Notes for AI Agents

- Decision: run substitution (2026-10-01): Fable credits are exhausted, so the
  spec owner runs as Opus 5.5. Wherever the contract has a role consult its
  Fable advisor, the orchestrator runs a fresh read-only Codex consult instead.
- Decision: run substitution (2026-10-01, the user's standing decision of
  2026-09-29): every Codex challenge round and review uses `gpt-6.1-sol` at
  `high` through `codex exec -s read-only`, not the plugin broker.
- Decision: user ruling (2026-10-01): a fourth, confirm-only Codex challenge
  round is authorized (T22.F04 precedent), limited to the fixes made after
  round 3 and the two edits below.
- Decision: user ruling (2026-10-01): D7 wins over the floor. The 1-in-10 floor
  holds until the per-run byte cap binds; then the oldest records thin further.
  The track success criterion was reworded to match.
- Decision: user ruling (2026-10-01): T21.F04's caps rule changes. Past the
  per-run caps, windows still send the compact Loki record, with no Tempo
  trace and no genome record.
