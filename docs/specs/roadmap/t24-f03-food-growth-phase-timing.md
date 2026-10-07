# T24.F03 — Food Growth Phase Timing

**Status**: In Progress
**Last updated**: 2026-10-06
**Feature**: T24.F03
**Track**: [T24 — Simulation Throughput](../../roadmaps/t24-simulation-throughput.md)

## Goal

Food growth is a timed phase of its own. `run_tick` times `World::grow_food`
apart from creature aging, energy decay and death removal, so the cumulative
phase metric, the sampled tick traces and the benchmark reports show
`food_growth` next to the five phases T21 exports, and each grid pass inside
`grow_food` is a child span of the `food_growth` span on every sampled tick.
Petri / Performance shows the new phase in its phase panels and lists the
pass spans of sampled ticks. Nothing a run computes changes: seeded output is
byte-identical to the parent build, telemetry on or off. T24.F02 reads this
phase's time per tick as its speedup instrument and the pass spans for its
decomposition.

## Non-Goals

- No change to what `grow_food` computes, in what order, or which RNG draws
  it makes; T24.F01 (spread repair) and T24.F02 (parallel regrowth) are
  separate rows.
- No per-pass cumulative metric: passes are trace material on sampled ticks
  only. No finer timing inside a pass, no per-cell or per-row span.
- No new `SimulationConfig` or `RuntimeConfig` field, no telemetry setting,
  no preset change, no change to the trace caps or the snapshot cadence.
- No frontend change: Petri / Performance is the Grafana dashboard.
- No timing of the untimed remainder (energy snapshot, queue shuffle,
  priority sort); it stays the gap between phases.
- No change to how benchmark comparison flags wall-clock or normalized work.

## Inputs and Invariants

| Input | Where | What F03 takes from it |
| --- | --- | --- |
| Track row and Notes | `docs/roadmaps/t24-simulation-throughput.md` | The goal (timed apart in the metric and the traces, passes as child spans, dashboard shows both); the ordinary gate, goal and mutation gates apply; panels grouped by `petri_phase` need no edit, the Phase spans regex, check 6 and the check script's name lists do; a new `PhaseWallClock` field reaches `phase_wall_clock_ms_per_seed`; T24.F02 measures its speedup on this phase and reads the pass spans. |
| Tick loop | `crates/v3-core/src/simulation/tick.rs` `run_tick`, `run_phase_0` | Phase 0 is `grow_food` then the creature loop (age, energy charge, shared-memory decay, `begin_tick`) then death removal; each phase is one `Instant::now()` and one `.elapsed()` added to `phase_wall_clock` and, under `telemetry-seams`, recorded by `record(index, started, elapsed)` into `phases: [PhaseTiming; 5]`. |
| Phase clock | `crates/v3-core/src/simulation/stats.rs` `PhaseWallClock` (five unconditional `Duration` fields, never reset), `PhaseTiming`, `TickPhaseTimings` (seam, `phases: [PhaseTiming; 5]`) | Where the sixth field and the pass timings sit. `crates/v3-telemetry/src/windows.rs:574` reads `seam.phases[2]` and `[3]` (cognition, actions) by index. |
| Food kernel | `crates/v3-core/src/kernel/ordinary_food/ecology.rs` `grow`, `grazing.rs` `recover`, `OccupancyDepletionLayer::recover_and_deposit`, `build_type_inhibition_weighted_sums`, `state.rs` `total_food_by_type` | The passes in the table below, their order and their gates: `grow` returns before every pass on zero cells, and after the two recovery calls when `max_density <= 0` or the catalog is empty; disabled depletion returns at once, disabled grazing counts passable cells and then returns; recovery spawns run only below the recovery floor with a positive rate. |
| Metric family | `crates/v3-telemetry/src/metrics.rs` `petri.run.phase_wall_clock` (`Sum`, unit `s`, key `petri.phase`); `metrics/tests.rs` expected families | The family gains the key `food_growth`; the name is already in `scripts/telemetry-dashboards-check` `always_names`. |
| Trace encoding | `crates/v3-telemetry/src/trace.rs` `PHASES`, `phase_attributes`, `world_update`, `encode`, `span_id`; `trace/tests.rs` `expected_keys`, six-span test | Span IDs are fixed integers in order; each phase span carries its governing config group; the tests pin names, IDs, parents and key sets. |
| Check script | `scripts/telemetry-dashboards-check` check 6 (`expected` names, `complete -eq 6`) | The span-set assertion that changes. |
| Dashboard | `telemetry/grafana/dashboards/petri-performance.json` | Phase panels query `sum by (petri_phase)`; "Phase spans of sampled ticks" selects by a name regex over the five phases. |
| Benchmark report | `crates/v3-cli/src/bench/schema.rs` `SeedPhaseWallClock`, `run.rs` (`millis(phases.*)`), `tests/bench.rs` phase-total assertion; `docs/benchmark-artifacts.md` | `environment` is kept whole in version-2 summaries, so a new field needs no keep-list edit; stored summaries lack the field, so it must deserialize as default. |
| Previous closure | `docs/progress/features/t20-f05-structured-heritable-refinement*.json`; epochs T11.F25 (gate) and T11.F27 (goal worlds) in `docs/progress/benchmark-series.json` | References for the profiles. |
| Clock cost | `std::time::Instant::now()` on Apple silicon and Linux (vDSO) is tens of nanoseconds; the gate profile spends 0.4 ms per tick in world update (track Notes) | The compute predeclaration. |

Passes of `grow_food`, in execution order. The spans are contiguous regions
of `grow` that together cover every grid-scale piece of work it does, so that
the passes sum to the phase up to the per-cell bookkeeping between them; a
span covers its region whatever the code inside does on that tick. "Runs
when" is the condition under which `grow` reaches the region; a region it
does not reach emits no span.

| Pass span | Region of `grow` | Per type | Runs when |
| --- | --- | --- | --- |
| `depletion_recovery` | the `OccupancyDepletionLayer::recover_and_deposit` call | no | the grid has cells (the call returns at once when depletion is disabled, and the span shows that call) |
| `grazing_recovery` | the `GrazingLayer::recover` call | no | the grid has cells (disabled grazing still counts passable cells, and the span shows that call) |
| `inhibition_sums` | the `build_type_inhibition_weighted_sums` call and the `inhibited_cell_flags` allocation | no | `max_density > 0` and the catalog is non-empty |
| `density_total` | the `total_food_by_type` sum (`state.rs`, a full-grid sum) | yes | as above, every type |
| `growth_spread` | the `next_density` allocation and the growth and spread loop | yes | as above, every type |
| `recovery_spawns` | the recovery spawn loop | yes | the type's density ratio is below `recovery_floor_ratio` and its spawn rate is positive |
| `write_back` | the `next_density` write-back and per-type telemetry loop | yes | as `growth_spread` |
| `inhibited_count` | the final `inhibited_cell_flags` count | no | as `inhibition_sums` |

Invariants:

1. No behavior change. No `SimulationConfig` or `RuntimeConfig` field, no
   production RNG draw, no simulation default and no deterministic output
   changes: a stored summary's deterministic blocks and config digests equal
   the previous closure's, while its provenance (`generated_at`, revision)
   and every host-dependent reading (wall clock, throughput, phase timings,
   among them the new `food_growth_ms` and `world_update_ms`, which now
   measures less) may differ; the config digest and the pinned recipe
   digests do not move. `grow_food` performs the
   same reads, writes and draws in the same order; timing wraps the passes
   and never reorders or splits them.
2. Six phases, disjoint. `PhaseWallClock` gains `food_growth` (unconditional,
   never reset, like the other five) and `world_update` is redefined as
   creature aging, energy decay with shared-memory decay and `begin_tick`,
   and death removal. `run_tick` times them in the order `food_growth`,
   `world_update`, `sensor_assembly`, `cognition`, `actions`,
   `reward_learning`; the phases do not overlap, and their sum is the timed
   part of the tick as before. On the reference build (`--no-default-features`)
   the split adds one `Instant::now()` and one `.elapsed()` per tick, in the
   pattern of the other five phases, and nothing else. `world_update`
   keeps its name, so a cross-run panel that compares a run from before this
   feature with one after it compares different `world_update` contents.
3. Seam. Under `telemetry-seams` only, `TickPhaseTimings.phases` becomes
   `[PhaseTiming; 6]` in the order of invariant 2, and the seam gains the
   tick's food passes: for each pass that ran, its name from the table, its
   food type index for a per-type pass, and its offset from the tick start
   and its elapsed time, in execution order. Pass timing is captured inside
   the kernel by clock reads around each pass and carried out through
   `FoodGrowthSummary` or a seam argument, compiled out with the feature;
   `v3-core` still takes no dependency and holds no exporter state. The
   passes nest inside `food_growth`: each starts at or after the phase's
   start, ends at or before its end, and the next starts at or after the
   previous ends. Every reader that indexes `phases` (`windows.rs`) is
   updated with the order.
4. Metric. `petri.run.phase_wall_clock` exports the six keys; `food_growth`
   is `Duration` → monotonic sum in seconds like the others, with no other
   attribute. The expected-families test lists it. No per-pass family.
5. Trace. A tick trace is one `tick` span (ID 1), six phase spans (IDs 2 to
   7, order of invariant 2, parent `tick`), then one span per food pass that
   ran (IDs 8 upward in execution order, parent the `food_growth` span, ID 2).
   The trace ID, the sampling rule, the caps and the transport are T21.F03's,
   unchanged. `PHASES` lists the six names.
6. Attributes. The `food_growth` span carries the food config group that
   `world_update` carried (`world.food.shared.growth_rate`, `max_density`,
   `grazing.enabled`, `occupancy_depletion.enabled`, `world.food.annealing.enabled`,
   `world.food.types` and the per-type `growth_inhibitor`, `growth_rate`,
   `recovery_spawn_rate`, `energy_per_unit`); `world_update` keeps
   `energy.lifecycle.energy_decay_per_tick`, `genome_carry_cost_per_unit` and
   `shared_memory.decay_rate`. A pass span carries the every-span identity
   set, `petri.phase` = `food_growth`, and `petri.food_type` (decimal index)
   on a per-type pass; its governing config stays on its parent and is not
   repeated. The `creature_window` root's reuse of the cognition and actions
   groups is unchanged.
7. Bounds. The pass set is `4 + 4 × food types` spans at most per trace (12
   for the two-type default, 36 for eight types). A pass span is about 450
   bytes on the wire (the eight every-span attributes, among them a 64-hex
   digest and a run ID, `petri.food_type` on a per-type pass, name, IDs and
   timestamps), so the encoding fixtures bound the
   whole trace with every pass present, `recovery_spawns` included: the
   two-type trace under 16 KiB (T21.F03's bound, kept for the default) and
   the eight-type trace under 32 KiB (the six-span eight-type trace was
   under 16 KiB; 36 pass spans add about 16 KiB), both far under the
   queue's 4 MiB body cap. The observation is T21.F03's tick trace, with its
   policy (`interval`, `run_end`), its cap (65,536 traces per run) and its
   cadence unchanged; its projected size per run grows by about 5.3 KiB per
   trace for two types: about 19 MiB more per hour at the default 1 s
   interval and at most about 340 MiB more per run at the cap.
8. Dashboard and check. Petri / Performance's phase panels show `food_growth`
   without edit; "Phase spans of sampled ticks" includes it in its name
   regex; a new table on the `tempo` datasource, "Food growth passes of
   sampled ticks", lists the pass spans of the run (TraceQL
   `{ span.petri.run_id = "$run_id" && name =~ "<the eight pass names>" } | select(span.petri.tick, span.petri.food_type)`,
   `tableType` `spans`, limit 500) so a reader sees each pass's duration on a
   sampled tick. The dashboards stay provisioned read-only under their fixed
   UIDs. Check 6 of `scripts/telemetry-dashboards-check` expects `tick`, the
   six phases, and pass spans whose names are all in the table, with one
   `growth_spread` and one `write_back` per food type of the workload; check
   2 also requires a `petri_phase="food_growth"` series on the run.
9. Benchmark report. `SeedPhaseWallClock` gains `food_growth_ms` with
   `#[serde(default)]`, so earlier summaries and reports load; `run.rs` fills
   it; the phase-total assertion in `tests/bench.rs` includes it; version-2
   summaries carry it inside `environment` with no keep-list change.
   `food_growth_ms / ticks` on a goal summary is the instrument T24.F02's
   predeclaration names.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.phase_wall_clock{petri.phase="food_growth"}` | counter (monotonic sum, seconds) | `PhaseWallClock.food_growth`, the wall time of `grow_food` per tick | `petri.run_id`; `world_update` now excludes food growth |
| `food_growth` phase span | span, child of `tick` | the seam's `food_growth` timing, first in the tick's order | the every-span set, `petri.phase`, the food config group of invariant 6 |
| `world_update` phase span | span, child of `tick` (existing, narrowed) | the seam's `world_update` timing | the every-span set, `petri.phase`, the lifecycle group of invariant 6 |
| `depletion_recovery`, `grazing_recovery`, `inhibition_sums`, `density_total`, `growth_spread`, `recovery_spawns`, `write_back`, `inhibited_count` | spans, children of `food_growth`, on sampled ticks only | the seam's food pass timings | the every-span set, `petri.phase` = `food_growth`, `petri.food_type` on per-type passes |
| `food_growth_ms` in `environment.phase_wall_clock_ms_per_seed` | benchmark report and summary field | `PhaseWallClock.food_growth` at the end of a seed run | the report's provenance |

Recorded configuration: none added. No mechanism switch or config value is
introduced; the food values a pass depends on are on the `food_growth` span.

## Implementation Tasks

- [ ] `v3-core`: add `PhaseWallClock.food_growth`, split phase 0's timing in
      `run_tick` (food growth first, then the creature loop and death removal
      as `world_update`), extend the seam to six phases and the food pass
      timings, and record the passes inside `grow` behind `telemetry-seams`;
      update the phase-timing and seam tests and the doc comments.
- [ ] `v3-telemetry`: export the sixth metric key; encode six phase spans and
      the pass spans with the IDs, parents, names and attribute groups of
      invariants 5 and 6; fix `windows.rs` indexes; update the expected
      families and trace tests.
- [ ] `v3-cli`: `SeedPhaseWallClock.food_growth_ms` (`serde(default)`), filled
      in `run.rs`; the phase-total assertion; the recipe digest pins must not
      move (no config field is added).
- [ ] Dashboard and check: the Phase spans regex, the pass table, check 6 and
      check 2 of `scripts/telemetry-dashboards-check`; the script's header
      comment.
- [ ] Docs: `docs/reference/v3-tick-orchestration-spec.md` needs no rule
      change; the T21 conventions table in `_feature-template.md` names the
      `world_update` group in `phase_attributes` and is updated to name
      `food_growth` beside it.

## Verification

- [ ] Phase clock: `cargo test -p v3-core phase_timing` and
      `cargo test -p v3-core --features telemetry-seams phase_seam` (the
      seam tests compile only with the feature; a zero-test run does not
      satisfy this item) -> every one of six phases accumulates and survives
      the counter reset; the seam's six phases match the clock's increments,
      are ordered and non-overlapping; the pass spans nest inside
      `food_growth` in order with the table's names, the per-type passes
      once per type, `recovery_spawns` only when the floor condition holds
      (a fixture below the floor and one above), the depletion and grazing
      spans present whether or not those mechanisms are enabled, no pass on
      a zero-cell grid, and only the two recovery spans when `max_density`
      is zero or the catalog is empty.
- [ ] No behavior change: `cargo test -p v3-core --test reproducibility` and
      the telemetry-neutrality test inside `make check` -> pass;
      `scripts/telemetry-parent-compare 9aa0c5a5` -> identical canonical
      NDJSON against the merge base.
- [ ] Telemetry: `cargo test -p v3-telemetry` -> the expected families carry
      `food_growth`; the trace test pins `tick`, six phases and every pass
      span with IDs 1 to 7 and 8 upward, parents, names and key sets; with
      every pass present and realistic identity lengths, the two-type
      encoding stays under 16 KiB and the eight-type encoding under 32 KiB.
- [ ] Benchmark report: `cargo test -p v3-cli --test bench` -> `food_growth_ms`
      present and in the phase total; the profile runs below load T20.F05's
      summaries, which lack the field.
- [ ] Dashboard: `scripts/telemetry-dashboards-check` -> checks 1, 2, 3 and 6
      pass with the new span set and the `food_growth` series. It needs the
      production stack down (`make telemetry-down`); if the user keeps the
      stack running, the same reads are made against it after one
      `v3-cli run --telemetry on` and recorded in the readings file.
- [ ] `make check` -> exit 0.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t24-f03-food-growth-phase-timing.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, no new full report staged.

Transcripts and tables: [readings](../../progress/readings/t24-f03.md).

## Performance and Goal Impact

**Predeclaration — written before the run.** This is a measurement feature:
no mechanism, no natural analog, no environmental pressure, no indicator.

| Item | Predeclared expectation and acceptance |
| --- | --- |
| References | Previous closure T20.F05 for gate and goal; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| Trajectories | None move. Every deterministic value, persistence, diversity, cognition, neighborhood, mutation-effect and input-use reading equals T20.F05; the effective-config digests equal T20.F05's. A moved trajectory is a defect, not a cost to accept. |
| Compute | Estimate, not a measurement: on the telemetry build the profiles run, each tick gains two clock reads for the phase split and, under the seam, two clock reads and one small store per food pass (at most 2 × (4 + 4 × types), 24 for the two-type default), tens of nanoseconds each, plus the seam's per-tick overwrite: on the order of 1 µs per tick against 0.4 ms of world update per gate tick and 41–68 ms per goal-profile tick (track Notes). Trace encoding is not on the profiles' timed path: `bench` captures at region boundaries only. The normalized-work flags (+10% / +50%) cannot move, since no counter changes. A wall flag (+25% / +100%) is not accepted on this estimate: it is investigated as the gate requires (rerun under `scripts/bench-wait` on an idle host, host and build attribution in the readings), and a severe compute verdict that survives investigation is a blocker, not a cost this spec predeclares. No epoch re-pin is sought or authorized. |
| New fields | `food_growth_ms` appears in both summaries' `environment.phase_wall_clock_ms_per_seed`, and `world_update_ms` now measures aging, decay and death removal only. Both are host-dependent wall time: they are never compared for equality with T20.F05; within each run the six phases sum to a positive part of `wall_clock_ms_total` (the `tests/bench.rs` assertion), and the readings report `food_growth_ms` per tick as the instrument T24.F02 inherits. Neither is a comparison input. |
| Observation caps | Unchanged: founder neighborhood 10 s; evolved neighborhood 180 s; read 10 s; mutation effects 60 s; input use 60 s; goal end-to-end 900 s. |
| Overhead check | Runs, because the feature adds work to a run (template conventions, "Ceiling"): `scripts/telemetry-overhead` in routine mode on the T21 workload (`v3-cli run --seed 7 --ticks 6908 --sample-every 6908 --config telemetry/overhead-world.json`, preset `standard`), states a (stack healthy) and b (stack stopped), pair count from the five-run spread rule (16, 24 or 32), under the 300 s cap; pass when the 90% upper bound of the median ratio is below 1.10 in both states, fail when the lower bound is at or above it, else inconclusive and reported as such. Expected: both states well below 1.10, since the seam adds about 1 µs per tick to a ~0.5 ms tick. The script refuses while the production stack holds its ports; stopping that stack is the user's decision and a refusal is reported under the blocker rule, not waived. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t24-f03-food-growth-phase-timing
make bench PROFILE=goal FEATURE=t24-f03-food-growth-phase-timing
```

**Measured verdict.** Not yet run.

- Summaries: [gate](../../progress/features/t24-f03-food-growth-phase-timing.json),
  [goal](../../progress/features/t24-f03-food-growth-phase-timing-goal.json).
- Full readings: [`docs/progress/readings/t24-f03.md`](../../progress/readings/t24-f03.md).

## Success Criteria

- [ ] A run with telemetry on exports `petri.run.phase_wall_clock` with six
      phase keys, `food_growth` among them, and `world_update` no longer
      includes food growth.
- [ ] Every sampled tick trace has six phase spans under `tick` and, under
      `food_growth`, one span per food pass that ran, named from the pass
      table, with `petri.food_type` on per-type passes.
- [ ] Petri / Performance shows `food_growth` in its phase panels and lists
      the pass spans of sampled ticks; `scripts/telemetry-dashboards-check`
      passes with the new span set.
- [ ] Benchmark reports and summaries carry `food_growth_ms` per seed.
- [ ] Seeded output is byte-identical to the merge base with telemetry off
      and on, and the gate and goal deterministic readings equal T20.F05.

## Notes for AI Agents
