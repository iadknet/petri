# T14.F01 — Closure Comparison and Indicator Coverage Integrity

**Status**: Complete
**Last updated**: 2026-09-11
**Feature**: T14.F01
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

A closure's verdict means what it says. A benchmark report never compares
against its own output path: when the series offers no earlier closure, the
report records why the reference is absent instead of comparing a value to
itself. Every wired goal indicator is visible to the no-regression rule: the
three `temporal_memory_sensitivity` fractions sit in the per-case comparison
readings beside `memory_different_from_either_fraction`. The three indicators
that carried no definition token — `lineage_diversity`, `memory_sensitivity`
and `reachable_structure_size_distribution` — carry one, so a later definition
change is visible in the stored report.

## Non-Goals

- Rewriting any historical report. `t12-f04-baseline-world-set-goal.json`
  keeps its self-comparison as measured; a reading a report did not measure
  stays "not measured".
- Transferring runtime counters (T14.F02), death and energy accounting
  (T14.F03), or the progress page redesign (T14.F11).
- Any change to `crates/v3-core`, a rate, a default, a charge, or a threshold.
- Changing how the series index is written or maintained.

## Inputs and Invariants

Sources of truth: `crates/v3-cli/src/bench.rs` (report schema, `case_readings`,
`apply_comparisons`, `default_gate_references`, `default_goal_references`),
`crates/v3-cli/src/main.rs` (the `bench` subcommand, the only place that knows
the output path), `docs/progress/benchmark-series.json`, and the
[telemetry gap audit](../../strategy/telemetry-gap-audit-2026-09-10.md) §3.3–3.4.

**Defect 1 — self-reference.** The evidence: in
`docs/progress/features/t12-f04-baseline-world-set-goal.json`,
`comparison.references[0].path` is that report's own filename and all 32
readings compare 8818 to 8818. Cause: `goal_worlds.epoch_baseline` in the
series index named the report before it was written.

- `default_gate_references` and `default_goal_references` return a
  `ReferenceSelection { paths: Vec<PathBuf>, absence: Option<String> }`
  instead of a bare path list, so the cause of an empty selection travels
  with it; explicit `--baseline`/`--compare` paths build one with no absence.
- Filtering happens where the output path is known. `apply_comparisons`
  takes the selection and the output path, drops every path that resolves
  to the output path, and records the absence cause when nothing remains —
  the selection's own cause if it carried one, otherwise the self-reference
  cause. `main.rs` passes `out_path`.
- Path equality: canonicalize both when both exist; otherwise compare the
  lexically normalized paths (`.` and `..` components resolved, relative
  paths joined to the current directory). The output file normally does not
  exist at comparison time — the report is written after the comparison —
  so canonicalize-only never fires.
- `Comparison` gains `reference_absence: Option<String>`
  (`#[serde(default, skip_serializing_if = "Option::is_none")]`), set only
  when `references` is empty and naming the case by its cause. The recorded
  strings are fixed, one per cause, so the four cases stay distinguishable:
  - `no series index at <path>`
  - `series <name> has no stored epoch baseline yet` (the goal series before
    its first report exists)
  - `the only candidate reference is this report's own output path <path>`
  - `no reference paths were given` (explicit empty selection, e.g. sweep)
  Historical reports load with `None`. `main.rs` prints the absence line
  after `wrote <path>` when it is set.
- A skipped self-reference is never an error and never counts as severe.

**Defect 2 — temporal memory outside the comparison block.** `case_readings`
adds three readings immediately after `memory_different_from_either_fraction`,
with these exact names, each read from the
`temporal_memory_sensitivity.per_seed` row whose outer `seed` equals the
case seed, and `None` when the indicator is `Undefined` or the row is absent:

| Reading name | Source |
| --- | --- |
| `temporal_memory_previous_slots_different_from_either_fraction` | `previous_slots.different_from_either_fraction` |
| `temporal_memory_persisted_outputs_different_from_either_fraction` | `persisted_outputs.different_from_either_fraction` |
| `temporal_memory_operator_state_different_from_either_fraction` | `operator_state.different_from_either_fraction` |

**Defect 3 — unversioned indicators.** `LineageDiversity`, `MemorySensitivity`
and `StructureSizeDistribution` gain
`#[serde(default, skip_serializing_if = "Option::is_none")] pub version: Option<String>`.
New reports always write `Some(<token>)`; a historical report without the
field loads as `None` and re-serializes without it, so an unmeasured token is
never an empty string. The tokens are `pub const` items in `bench.rs`, one
place to move when a definition moves:

| Struct | Constant | Token |
| --- | --- | --- |
| `LineageDiversity` | `LINEAGE_DIVERSITY_VERSION` | `lineage-diversity-v1` |
| `MemorySensitivity` | `MEMORY_SENSITIVITY_VERSION` | `memory-sensitivity-v1` |
| `StructureSizeDistribution` | `REACHABLE_STRUCTURE_VERSION` | `reachable-structure-v1` |

`StructureSizeDistribution` appears both pooled on `GoalIndicators` and per
case on `GoalCaseObservation`; both carry the token.

Invariants carried from the track: additive observation consumes no
production RNG, selects no survivors, changes no execution; every existing
reading and counter is byte-identical to T11.F09's reports; `SCHEMA_VERSION`
stays 1 because every new field is serde-defaulted and historical reports
round-trip.

## Implementation Tasks

- [x] `apply_comparisons` takes the output path, skips a self-reference, and
      records `reference_absence` with the fixed cause strings; `main.rs`
      passes `out_path` and prints the absence line.
- [x] `case_readings` emits the three temporal readings by their fixed names.
- [x] The three version constants and `version` fields; every constructor
      writes them.
- [x] Tests (TDD): a synthetic report compared against its own path records
      the self-reference absence and no `references` entry; explicit
      `--baseline` naming the output path is also skipped; the three
      series-index absence causes are each recorded; the temporal readings
      appear by name in a world-set case comparison and are `None` when the
      indicator is `Undefined`; a new report carries all three tokens and
      T12.F04's stored goal report loads with `version` absent and
      re-serializes unchanged.
- [x] `docs/progress/readings/t14-f01-closure-comparison-and-indicator-coverage-integrity.md`
      holds command transcripts and the comparison-block excerpt of the
      stored goal report.

## Verification

- [x] Focused tests: `cargo test -p v3-cli` -> 75/11/20/11 passed, 0 failed;
      test names in the readings file.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` -> clean.
- [x] `make check` on the final feature code -> exit 0 at `bf53f3dd`
      (log `/private/tmp/claude-501/-Users-istefanek-projects-petri/3210ad1f-6aec-4896-90a3-53405123209a/scratchpad/t14-f01-check.log`).
- [x] Stored goal report `comparison.references[].path` names
      `t12-f04-baseline-world-set-goal.json` and
      `t11-f09-learned-state-inheritance-integrity-goal.json` and nothing
      else; every case's `readings` contains the three temporal names with a
      `current` value; `reference_absence` is absent.
- [x] Stored gate and goal reports carry the three version tokens (the gate
      profile leaves `lineage_diversity` and `memory_sensitivity` `Undefined`,
      so its report carries the reachable-structure token only).
- [x] Fresh run (`MUTANTS_ITERATE=0 make rust-mutants`), output at
  `/Users/istefanek/.local/share/petri-tools/mutants/t14-f01/mutants.out`. No
  timeouts. A confirming incremental pass reused that directory afterwards, so
  its `missed.txt` is superseded; the fresh run's survivor list is the table
  below.

  ```
  47 mutants tested in 4m: 1 missed, 34 caught, 12 unviable
  ```

  | Survivor | Resolution |
  | --- | --- |
  | `crates/v3-cli/src/main.rs:373:24: delete ! in run_bench` | killed by `an_explicit_compare_path_is_preferred_over_the_profile_default_selection` in `crates/v3-cli/tests/bench.rs`, which asserts an explicit `--compare` path is the sole reference of a sweep run whose profile default is empty |
- [x] Benchmark reports stored at
      `docs/progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity.json`
      and `-goal.json`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation-only report
integrity; the natural-analog rule does not apply and `v3-core` is untouched.
Gate: all six normalized counters exactly unchanged against T11.F09 and the
`remove-complementary-nutrition` epoch; wall time within the existing
threshold. Goal (`goal-worlds-v1`): references predeclared to be exactly the
series' two paths, T12.F04 (epoch baseline) and T11.F09 (last closed); all six
counters 0.000000% against both; every pre-existing per-case reading identical
to T11.F09's values; the three temporal readings appear with the same values
T11.F09's `temporal_memory_sensitivity` block already stores, so no indicator
is predeclared to move. Founder 10 s, evolved 180 s, drift 30 s per world and
the 15-minute goal investigation thresholds are unchanged. No epoch is
re-pinned. The second goal run is not required (user decision 2026-09-05).

**Measured verdict.** Gate: exit 0, severe=false, no threshold crossed; all
six counters 0.000000% against T11.F09 and the same deltas as T11.F09 against
the `remove-complementary-nutrition` epoch; wall 0.001391 ms per creature-tick
(+3.05% vs T11.F09, ok); epoch unchanged. Goal (`goal-worlds-v1`): exit 0,
severe=false; references exactly T12.F04 and T11.F09, `reference_absence`
absent; all six counters 0.000000% against T11.F09; against T12.F04 the report
carries the deltas T11.F09's stored report already carries (`plasticity_updates`
+40.89% flag, inherited), so the predeclaration's "0.000000% against both" was
mis-stated and is not a movement by this feature; every pre-existing per-case
reading identical to T11.F09's; the three temporal readings present in every
case with T11.F09's stored values (0.000000% vs T11.F09); founder 0.11 s,
evolved 0.47 s, drift walk 19.4 s, goal wall 492.7 s, all under threshold;
epoch unchanged.

- Reports: [gate](../../progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity.json),
  [goal](../../progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json).
- Full readings: [`docs/progress/readings/t14-f01-closure-comparison-and-indicator-coverage-integrity.md`](../../progress/readings/t14-f01-closure-comparison-and-indicator-coverage-integrity.md).

## Success Criteria

- [x] A `bench` run whose only series candidate is its own output path
      records `reference_absence` and no self-comparison.
- [x] The stored goal report's every case comparison lists the three
      temporal memory readings.
- [x] New reports carry version tokens on the three indicators; historical
      reports load unchanged.

## Notes for AI Agents

- Deferred: review P3 — `per_seed_indicator_readings` returns a fixed-size
  `[(String, Option<f64>); 6]` (`crates/v3-cli/src/bench.rs`), so a later T14
  per-seed reading is a two-place edit; make it a `Vec` when one is added.
- Deferred: review P3 — the historical round-trip test `include_str!`s the whole
  T12.F04 goal report into every lib-test compile; read it from
  `CARGO_MANIFEST_DIR` if the compile cost becomes noticeable.
- Exception: both stored reports were produced from the code committed as the
  self-review commit while their `git_revision` field reads the preceding
  build commit `de2559bd`; the self-review edits are mechanical refactors that
  cannot change report content (the self-reference filter never fired —
  `reference_absence` is null in both), so the reports were not re-measured.
- Exception: this feature's orchestrator ran as Opus 5 at effort `medium`
  in place of Fable 5.1, authorized by the user in the goal command
  (2026-09-11).
