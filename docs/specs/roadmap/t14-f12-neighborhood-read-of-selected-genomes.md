# T14.F12 — Neighborhood Read of Selected Genomes

**Status**: In Progress
**Last updated**: 2026-09-14
**Feature**: T14.F12
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

At the end of every goal-profile world run, a seeded sample of fifty living
genomes each produces one hundred fresh births under the production mutation
engine, and the existing `neighborhood-v1` battery classifies every mutated
birth as silent, changed or dead. The tallies, their denominators, the sample's
identity and its depth are stored per world in a `neighborhood_read` block
beside `drift_depth`, and the pooled changed-per-all-births fraction becomes
the closure indicator for changed births at depth: read on the substrate that
selection and the structure costs actually produced, not on a drift walk that
neither can touch.

## Non-Goals

- No change to the mutation engine, battery, classifier, any rate, cost or
  default; no production RNG consumed.
- T11.F01's evolved half and the drift walk's readings stay byte-identical;
  only the drift chart's floor lines go, per the T11 amendment.
- No clade stratification, per-unit supply arm, 12,000-tick run or checkpoint
  reading. One read, at the horizon, per world.
- No floor enforced in code: floors are read at closure from the stored report,
  as the drift floor was.
- No use of the live creature's dispatch record as the executed set; the
  classifier's battery-derived stand-in keeps the reading comparable in kind.

## Inputs and Invariants

Sources of truth: the track's F12 note; the T11 track's 2026-09-14 floor
amendment; the T03.F11 note; the
[per-unit mutation supply research note](../../strategy/per-unit-mutation-supply-research-2026-09-14.md),
Sections 1 and 5.1 (`probe_a`); `crates/v3-core/src/neighborhood/{births,sample,mod}.rs`;
`crates/v3-cli/src/bench/{indicators,run,schema,comparison,artifacts,profiles}.rs`;
`docs/progress/index.html`; and the T13.F06 goal report at
`docs/progress/features/t13-f06-recruitment-and-retention-qualification-goal.json`.

**What already exists, verified in code.** `neighborhood::births::per_birth_result`
is the production classifier the drift walk and `probe_a` use: fresh copies of a
genome through `MutationEngine::apply_mutations_with_food_type_count`, seeded
`seed_offset + 9_000 + birth_index`, executed set from
`Battery::executed_indices`, zero-event births counted not evaluated, integer
`Tally` folds. T11.F01's evolved half already runs it on the terminal
population: `evolved_neighborhood_for_seed` takes 12 genomes at fixed ranks of
the id-sorted population, 200 births each, and stores `pooled_births` per world
under `cases[].mutational_neighborhood.evolved.per_seed[0]` (readings in the
table below). This feature does not replace that reading; it adds the larger,
seeded, births-only read the note asks for and makes its fraction the indicator.

**A mislabelled comparison key, repaired here.** `comparison.rs` names
`evolved_changed_per_all_births` and `evolved_dead_per_all_births` but reads
`any_events.changed_fraction` and `dead_fraction`, whose denominator is
`applied` (mutated births), while the drift walk's `changed_per_all_births`
divides by all births. Under the track's F01 criterion a reading's name is its
definition token, so the two keys are renamed `evolved_changed_per_mutated_births`
and `evolved_dead_per_mutated_births`; values do not change, the earlier
closure's report lacks the new names (`reference: null`, no delta), and no
historical report is rewritten.

**Sample.** From the id-sorted living population (the same order the evolved
half uses), a uniform sample without replacement of `min(50, n)` creatures,
drawn with `SmallRng::seed_from_u64(8_000_000 + world_seed)` and emitted as
ascending indices; a pure function of `(population_size, seed)` in
`neighborhood/sample.rs` beside `evolved_sample_ranks`. The draw algorithm is
pinned by a unit test asserting the exact ranks for one `(n, seed)` pair; a
change to the draw bumps `version`. An empty population yields no rows and
`UNDEFINED` fractions. Sample and birth counts travel on `NeighborhoodSizes`
(production 50 and 100; the tiny test fixture keeps its own small values) and
are recorded truthfully in the block, never in `ProfileBlock`. The read runs
on the goal world set only, per world, as `drift_depth` does; the plain `goal`
profile and the gate record it `Undefined`.

**Births.** Genome `i` in sample order uses `seed_offset = 8_000_000 + 1_000 × (i + 1)`
through the unchanged `per_birth_result`, a range disjoint from the evolved
half's (`100_000 × (i + 1)`) and the drift walk's (`7_000_000 + …`) offsets.
Seeds are fixed, so a zero-event count is a property of the seed stream, as
for the evolved half.

**Block.** `GoalCaseObservation` gains `neighborhood_read: Indicator<NeighborhoodRead>`,
`Undefined` outside the goal world set and `#[serde(default)]` so stored reports
predating it load with the reading unmeasured, never zero. Fifty flat rows per
world is the bound; no per-birth detail and no genome is stored; the block is
projected into the committed summary whole.

| Field | Content |
| --- | --- |
| `version`, `battery_version` | `neighborhood-read-v1`; the battery's own version |
| `sample_seed_formula`, `birth_seed_formula` | the two formulas above, as strings |
| `population_size`, `sample_size_requested`, `sample_size`, `birth_trials` | living creatures, 50, `min(50, n)`, 100 |
| `births` | the pooled `NeighborhoodBirths` the drift checkpoint stores: `births_total`, `zero_event_births`, `by_requested_events`, `any_events`, `by_events` |
| `silent_per_all_births`, `changed_per_all_births`, `dead_per_all_births` | computed exactly as `drift_checkpoint` computes them, over `births_total` |
| `generation_sum`, `genome_size_sum`, `total_nodes`, `reachable_nodes`, `executed_nodes` | integer sums over the sample, each with a `mean_*` `six` string |
| `genomes[]` | per sampled genome: `rank`, `creature_id`, `lineage_id`, `generation`, `genome_size`, `total_nodes`, `reachable_nodes`, `executed_nodes`, `births_total`, `zero_event_births`, `silent`, `changed`, `dead` |

**Depth beside the reading.** The population's mean generation is on T14.F04's
terminal checkpoint. The track note's "mean generation 22" is the drift walk's
checkpoint, not the population's current depth; the readings file records the
sample's mean generation beside the table below so the depth the indicator
reads is never mistaken for the walk's 2,000.

| Reading | Orchards | Canyon | Confluence | Source |
| --- | ---: | ---: | ---: | --- |
| Goal terminal mean generation | 46.77 | 48.07 | 52.73 | T13.F06 goal report, F04 checkpoint 2,000 |
| Goal terminal mean genome size / mesh nodes | 213 / 5.06 | 226 / 5.16 | 246 / 4.99 | same |
| Evolved half changed / dead per all 2,400 births | 374 / 10 | 313 / 14 | 320 / 8 | same, `evolved.per_seed[0].pooled_births` |
| 12,000-tick pair mean generation, cost / control | 177 / 301 | — | — | `docs/progress/sweeps/t03-f08/`, plains world |

**Determinism.** `per_birth_result` already folds a `rayon` map through
integer `Tally::merge` in birth order; the sample is a pure seeded draw; genomes
are read after the last tick; pooled sums are `u64` and every mean is one
integer division at format time. Nothing touches the simulation's RNG or
execution, and the block must be byte-identical across thread counts.

**Comparison and page.** Three per-case readings join `compare_cases`:
`neighborhood_read_changed_per_all_births`, `neighborhood_read_dead_per_all_births`,
`neighborhood_read_silent_per_all_births`. The environment block records
`neighborhood_read_wall_clock_ms_per_seed` and its total; nothing timed enters
`deterministic`. `docs/progress/index.html` adds one chart, "Neighborhood read:
changed per all births", per world with the first-reading reference line, and
the drift chart drops its two floor lines and the floor wording in its subtitle.

## Implementation Tasks

- [x] `v3-core`: the seeded sample-rank function in `neighborhood/sample.rs`
      with unit and property tests (empty, below, at and above the sample size;
      strictly ascending, in bounds, distinct, fixed by seed).
- [x] `v3-cli`: the `NeighborhoodRead` schema with `Undefined` default, its
      per-seed reader in `indicators.rs` reusing `per_birth_result`,
      `structural_companions` and `Battery::executed_indices`, the timed call in
      `run_one_seed` under the goal world set, sizes on `NeighborhoodSizes`, the
      block on the case projection in `artifacts.rs`, the three new and two
      renamed comparison keys, and the environment wall-clock fields.
- [x] `docs/progress/index.html`: the new chart and the drift-chart floor
      removal.
- [x] Tests: `Undefined` outside the goal world set and defined per world on
      it; pooled tallies equal the sum of the per-genome rows with
      `births_total == sample_size × birth_trials`; fractions divide by
      `births_total`; a report without the block loads unmeasured; a reduced
      world-set fixture is byte-identical across runs and thread counts (the
      `reduced_drift_deterministic_output_matches_across_thread_counts_and_seed_counts`
      precedent); the renamed keys carry the old values and the new keys read
      the new block.

## Verification

- [ ] `make check` -> exit 0 on the final feature code; commit named here.
      Uncommitted post-simplify worktree on fd38f20c: `make check` exit 0
      (2026-09-14).
- [x] Focused tests: `cargo test -p v3-core -p v3-cli` -> exit 0, with
      `cargo clippy -p v3-core -p v3-cli --all-targets` and
      `cargo fmt --all -- --check` clean; test names in the readings file.
      Post-simplify worktree: `cargo test -p v3-core -p v3-cli` exit 0
      (1,453 v3-core unit, 105 v3-cli unit, every integration binary ok, 0
      failed); `cargo clippy --workspace --all-targets -- -D warnings` clean;
      `cargo fmt --all -- --check` clean. New tests: `neighborhood::sample::read_tests::{empty_population_or_zero_sample_reads_nothing,
      population_at_or_below_sample_size_takes_every_rank,
      draw_is_pinned_for_one_population_and_seed,
      ranks_are_ascending_distinct_in_bounds_and_seed_fixed}` (v3-core);
      `bench::indicators::tests::{neighborhood_read_samples_seeded_ranks_and_pools_its_rows_over_all_births,
      neighborhood_read_of_an_empty_population_has_no_rows_and_undefined_fractions}`,
      `bench::run::tests::world_set_neighborhood_read_is_defined_per_world_and_byte_identical_across_thread_counts`,
      `bench::comparison::tests::evolved_and_neighborhood_read_keys_name_their_denominators`
      (v3-cli); `world_neighborhood_read_is_projected_whole` (`tests/bench_artifacts.rs`).
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`; one survivor, killed by a
      strengthened test, no second fresh run (test-only change).

      | Field | Value |
      | --- | --- |
      | Summary | `41 mutants tested in 7m: 1 missed, 31 caught, 9 unviable` |
      | Run mode | `fresh` (diff against 930ce9be; 2026-09-14) |
      | Output | `~/.local/share/petri-tools/mutants/t14-f12/mutants.out` |
      | Survivor | `crates/v3-cli/src/bench/indicators.rs:679:24: replace += with *= in neighborhood_read_for_seed` |
      | Resolution | killed: `bench::indicators::tests::neighborhood_read_samples_seeded_ranks_and_pools_its_rows_over_all_births` now assigns distinct positive generations before the read (founders were all 0, so a product matched the sum); `MUTANTS_ITERATE=1` pass: 1 tested, 1 caught |
- [x] The stored goal report carries `neighborhood_read` on all three worlds
      with `sample_size == 50`, `birth_trials == 100`, `births_total == 5000`,
      the per-genome rows summing to the pooled tally, and the three fractions
      dividing by 5,000; jq transcript and the per-world table in the readings
      file, with the sample's and the population's mean generation beside it.
      Confirmed: sample 47.0/48.5/52.7 vs population 46.77/48.07/52.73.
- [x] The comparison block lists the three `neighborhood_read_*` keys per world
      with `reference: null`, and `evolved_*_per_mutated_births` with the same
      values the previous closure stored under the old names. Yes.
- [x] The predeclared direction of none, checked as T14.F04 did: with the
      `neighborhood_read` keys deleted, the stored goal report's
      `deterministic.goal_indicators` is byte-identical to T13.F06's (no
      simulation change between them on main); jq diff in the readings file.
      Zero-length diff.
- [x] Benchmark summary stored at `docs/progress/features/<id>.json` and its
      `-goal` companion, local raw hash/byte count and verification time
      checked, series entry points to the summary, the goal summary's byte
      growth against T13.F06's recorded, and no new full report staged.
      Confirmed; series entries added.

Transcripts and per-world tables go to
[`docs/progress/readings/t14-f12.md`](../../progress/readings/t14-f12.md).

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement feature under the
track's observation contract: no natural analog, no mechanism, no change to any
rate, cost or default. References: the gate and goal profiles compare against
the epoch baselines the series index names, under the existing thresholds. No
epoch re-pin is expected and none is authorized here.

Expected compute cost: negligible and goal-only; the gate profile is untouched.
The read is 5,000 births per world on genomes of about five mesh nodes; the
evolved half's 12 genomes with about 1,300 battery trials each (54 operators
by 20 trials plus 200 births) ran in 161 / 160 / 190 ms per world at T13.F06,
so the new read is expected under one second per world. Predeclared cap: 10 seconds summed across the three worlds
(`environment.neighborhood_read_wall_clock_ms_total`), inside the unchanged
15-minute goal total and beside the untouched evolved-half 180-second and
founder 10-second caps. Exceeding it is resolved before closure, never absorbed.

Predeclared direction for every existing goal indicator and counter: **none**;
all are expected byte-identical to a run without this feature; any movement is
a defect, not a result.

Expected range of the new reading, a sanity check and not a threshold: pooled
`changed_per_all_births` per world between the drift walk's depth-22 reading
(0.104) and its depth-0 founder reading (0.217), because the evolved half's
12-genome read on the same populations gives 0.156 / 0.130 / 0.133. A reading
outside that range is explained in the readings file, not rejected.

**Floor.** The first reading of `neighborhood_read.changed_per_all_births` on
each world is that world's floor, strict not-below, from the next closure on;
the closing commit records the three values as a `Decision:` bullet below and
the T14 track cites this spec as the standing floor. Dead per all births is
stored and compared but carries no floor. T03.F11 reads this floor: a
replication rate that lowers the fraction below it, or shrinks the executed
core while the fraction holds, is rejected there.

**Decision.** First-reading floors for `neighborhood_read.changed_per_all_births`,
standing from the next closure on: Orchards in grassland (11) = 0.146400,
Canyon country (22) = 0.130000, Confluence (33) = 0.143400.

**Measured verdict.** Gate and goal exit 0, `severe=false`, both predeclared
caps and the direction-of-none check hold (zero-length diff vs T13.F06);
`neighborhood_read` defined per world as predeclared. Sample vs population
mean generation: 47.0/46.77, 48.5/48.07, 52.7/52.73. Full details in the
[readings](../../progress/readings/t14-f12.md).

- Summaries: [gate](../../progress/features/t14-f12-neighborhood-read-of-selected-genomes.json),
  [goal](../../progress/features/t14-f12-neighborhood-read-of-selected-genomes-goal.json).

## Success Criteria

- [ ] Every goal-profile world stores a `neighborhood_read` block with fixed
      sample and birth seeds, 50 genomes, 100 births each, per-all-births
      fractions with their denominators, per-genome rows and the sample's depth.
- [ ] The three `neighborhood_read_*` readings are visible to the comparison
      chain, the first reading is recorded as the per-world floor, and the
      evolved-half keys name the denominator they divide by.
- [ ] Every pre-existing indicator, counter and stored reading is byte-identical
      to a run without this feature, and the block reproduces across thread
      counts.
- [ ] The progress page draws the reading with its first-reading reference and
      the drift chart draws no floor lines.

## Notes for AI Agents

- Decision: T14.F12 was scheduled by the user on 2026-09-14 out of the order of
  new starts, together with the T11 floor amendment that makes this reading the
  closure indicator for changed births at depth.
- Deferred: `neighborhood_read_wall_clock_ms_total` serializes as `-0.0` on the gate summary (empty f64 sum, inherited from the evolved total); the sample.rs read tests sit in a second test module after the existing one.
