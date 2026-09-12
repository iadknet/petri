# T14.F04 — Population Readings on the Persistence Checkpoints

**Status**: In Progress
**Last updated**: 2026-09-11
**Feature**: T14.F04
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Every persistence checkpoint the benchmark already samples carries the
population readings the report previously took only at the horizon: mean genome
size, mean mesh node count and mean generation of the living population, and the
surviving founder clade count with its Shannon entropy. A stored report shows
when a structural or lineage change took hold, not only where the run ended.

## Non-Goals

- Any new mechanism, charge, rate, default, threshold or RNG draw. This feature
  reads what the simulation already applies.
- New checkpoints, a new cadence, or any change to `SAMPLE_EVERY_TICKS` and the
  existing sampled-tick rule.
- The clade size distribution, per-clade rows, ecotype descriptors and the
  ancestry graph — T04.F01 and T14.F07.
- New goal indicators, thresholds, comparison-block entries, indicator version
  or definition tokens, and the no-regression rule's coverage set. These are
  checkpoint readings inside the stored `population_persistence` block, not
  indicators; T14.F01's token rule applies to indicators and is untouched here.
- Progress-page presentation of the new series — T14.F11.
- Sensor usage, the clade persistence timeline and the occupancy grid, which
  reuse these checkpoints — T14.F08, T14.F09, T14.F10.
- Rewriting historical reports. A report stored before these fields existed
  reads them as absent, never as zero.
- Death counting, energy flow totals (T14.F03) and cognition telemetry
  resolution (T14.F05).

## Inputs and Invariants

Sources of truth: `crates/v3-cli/src/bench.rs` (`PersistenceSample`,
`PersistenceAccumulator`, `run_one_seed`, `lineage_diversity`,
`SAMPLE_EVERY_TICKS`), `crates/v3-cli/src/lib.rs` (`structure_means`,
`build_tick_sample`), and the track's F04 note.

**Reuse, do not re-measure.** The track's F04 note binds this feature to the
existing checkpoints and the existing accessors. `structure_means` already
returns the three means and is already called by `build_tick_sample` on the
`v3-cli run` path; `bench.rs` never calls it. `lineage_diversity` already
computes the surviving founder clade count and the Shannon entropy for the
terminal goal reading. Both are shared rather than reimplemented, and their
existing callers keep their current output byte-for-byte.

**Placement.** The new fields go on `PersistenceSample`, beside
`mean_energy` and `births_total`, and not inside `WorldTracking`. T14.F02's
invariant that a checkpoint sample carries no transferred counter block is a
`WorldTracking` property; it and its test stay as they are.

**Determinism is the binding constraint.** Seeded runs reproduce byte-for-byte
across processes and thread counts, and the gate profile's two-run
byte-identical check inside `make check` now exercises these fields:

- The three means sum `u64` totals over the living population before one
  division, so no floating-point accumulation order reaches the value.
- Clade counting is keyed in a `BTreeMap<u32, _>`; no `HashMap` iteration order
  reaches the report, per T14.F02's constraint.
- Every value is read after `run_tick` on an already-sampled tick, from state
  the tick produced. No production RNG is consumed, no survivor is selected, and
  no execution path changes.

**Empty population.** `mean_energy` is already `None` at extinction. The three
means and the entropy follow the same rule rather than reporting zero: the
extinction sample, which the accumulator always takes, carries them as absent or
`UNDEFINED` exactly as the terminal reading does for an empty population. Clade
count is `0` there, which is a true count, not an unmeasured value.

**Historical reports.** Every new field is optional with `#[serde(default)]`, so
the stored reports the comparison chain and the progress page read continue to
deserialize with these readings absent.

## Implementation Tasks

- [x] Share `structure_means` and the clade-count/entropy computation between
      their existing callers and the checkpoint sample, without changing either
      caller's current output.
- [x] Carry the five readings on `PersistenceSample`, evaluated only on sampled
      ticks and only from post-tick state, with the empty-population and
      historical-report rules above.
- [x] Tests: the readings appear on every checkpoint and match a directly
      computed value, both through the accumulator and through the real
      `run_one_seed` loop, where the horizon checkpoint equals an independently
      re-run terminal state and differs from tick 100's; the extinction sample
      reports absence rather than zero; a stored report predating the fields
      still loads. The `v3-cli run` tick sample and the terminal goal reading
      are pinned by pre-existing tests (`crates/v3-cli/tests/cli.rs:212` and
      `:223`, `crates/v3-cli/src/bench.rs`'s `lineage_diversity` tests) plus one
      cross-check in `population_readings_average_the_living_population_and_count_its_clades`
      asserting the checkpoint entropy equals `lineage_diversity`'s.

## Verification

- [ ] `make check` -> exit status recorded here, run once on the final feature
      code, with the tested commit named.
- [x] Focused tests at commit `75fa7e6c`: `cargo test -p v3-cli` -> ok,
      84 + 11 + 20 + 11 passed, 0 failed. Covers the new checkpoint-reading
      tests (`every_checkpoint_carries_the_population_readings_of_its_own_tick`,
      `the_real_run_path_reads_every_checkpoint_from_its_own_post_tick_state`,
      `the_extinction_checkpoint_reports_absent_means_and_a_zero_clade_count`,
      `population_readings_average_the_living_population_and_count_its_clades`,
      `population_readings_of_an_empty_population_are_zero_means_and_undefined_entropy`,
      `persistence_sample_readings_survive_a_json_round_trip`), the unchanged
      `checkpoint_tracking_omits_every_transferred_block`, the
      `lineage_diversity` tests and the `v3-cli run` tick-sample tests.
      `cargo clippy -p v3-cli --all-targets` -> clean.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path, and
      every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here; `docs/workflow.md` requires it in the spec.
- [ ] Checkpoint samples in the stored goal report carry all five readings at
      every checkpoint of all three world cases, and the `WorldTracking` key set
      of a checkpoint sample is unchanged from T14.F02's stored report.
- [ ] Benchmark reports stored at
      `docs/progress/features/t14-f04-population-readings-on-the-persistence-checkpoints.json`
      and its `-goal` companion.

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement feature; the track's
observation contract exempts it from the natural-analog rule and it adds no
mechanism. References: the gate and goal profiles compare against the epoch
baselines the series index names, under the existing thresholds.

Expected compute cost: negligible. The readings are `O(population)` passes taken
only on already-sampled ticks — the same cadence and the same population scan
the existing `mean_energy` sample already pays, at most 21 sampled ticks per
seed. No epoch re-pin is expected and none is authorized here. Predeclared
direction for every goal indicator: **none** — this feature changes nothing the
simulation applies, so lineage diversity, memory sensitivity, temporal memory
sensitivity, mutational neighborhood, drift depth, population persistence and
births per 100 ticks are all expected to be unchanged, and any movement in them
is a defect rather than a result. The stored reports grow by five values per
checkpoint sample. A severe compute regression would be a blocker, not a cost to
justify.

Goal impact: the horizon readings the report already trusts become a trajectory,
so a later closure can say when a change took hold. T14.F08, T14.F09 and T14.F10
read their own series off these same checkpoints.

**Measured verdict.** One line per profile: exit status, the `severe` flag,
whether any threshold was crossed, and whether the epoch was re-pinned.

- Reports: [gate](../../progress/features/t14-f04-population-readings-on-the-persistence-checkpoints.json),
  [goal](../../progress/features/t14-f04-population-readings-on-the-persistence-checkpoints-goal.json).
- Full readings: [`docs/progress/readings/t14-f04.md`](../../progress/readings/t14-f04.md).

## Deviations

Authorized by the user in this feature's goal command, for this feature only,
because the Fable 5.1 budget is exhausted. No model configuration reaches
`main`: the merged range touches no file under `.claude/`.

- The orchestrator is Opus 5 at effort `medium` in place of Fable 5.1 at effort
  `medium`; the contract's model check passes on that basis.
- `roadmap-reviewer` is spawned with the Agent tool's `model` parameter set to
  `opus`, overriding the agent definition's `fable` frontmatter.
  `.claude/agents/roadmap-reviewer.md` is not edited.
- The implementer's advisor is Opus 5, set by a worktree-local
  `.claude/settings.local.json` containing `{"advisorModel": "opus"}`. That path
  is ignored by git and cannot be committed; the tracked
  `.claude/settings.json` is not edited.

## Success Criteria

- [ ] Every checkpoint sample of a stored benchmark report carries mean genome
      size, mean mesh nodes, mean generation, surviving founder clade count and
      Shannon entropy, on all three goal world cases.
- [ ] The readings are reproducible byte-for-byte across processes and thread
      counts, consume no production RNG and change no execution.
- [ ] Reports stored before this feature still load with the readings absent,
      and the existing horizon readings and `v3-cli run` tick sample are
      unchanged.

## Notes for AI Agents

- Decision: Checkpoint readings under `population_persistence` are not goal
  indicators: they carry no version token, no threshold and no entry in the
  comparison block the no-regression rule reads. T14.F01's token rule applies to
  indicators.
- Deferred: A sixth checkpoint reading costs edits in five places today —
  `PersistenceSample`, `PopulationReadings`, the `alive.then(|| six(...))`
  mapping in `PersistenceAccumulator::observe`, and two test helpers. The
  proposed shape for T14.F08's planner: make `PopulationReadings` itself the
  serialized type with `Option` fields, `#[serde(default)]` and
  `#[serde(flatten)]` into `PersistenceSample`, plus an `absent()` constructor
  for the extinction sample; the caveat is that two flattened fields in one
  struct requires neither flattened type to deny unknown fields. Not taken here
  because the scheduled extensions are structured blocks with their own types
  (T14.F08's per-`WorldInputKey` counts, T14.F10's occupancy grid), not a sixth
  flat scalar, and a flatten change to a determinism-critical stored-report
  struct is not worth making inside a closure path.
- Exception: The three model substitutions in this spec's Deviations section
  were authorized by the user on 2026-09-11 for T14.F04 only, because the
  Fable 5.1 budget is exhausted. They are not a precedent for later features and
  reach no file on `main`.
