# T30.F09 — Retire the Pre-Lab Experiments in Core

**Status**: Complete
**Last updated**: 2026-10-28
**Feature**: T30.F09
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

The two capability experiments placed in `v3-core` before the T22 lab rule
leave the workspace with everything only they used: the recruitment-paths
experiment (`neighborhood/recruitment_paths`, `712951212`) and T20.F01's
discovery and ecological-opportunity assay (`neighborhood/opportunity`,
`a0939c050`), their goal-profile
indicator and its wall-clock field, the `recruitment` and `input-opportunity`
commands, and the recruitment half of the slow tier. The authored
controllers the lab builds its comparators from move into `v3-lab`, with one
Ring-wiring operation in place of today's two. Simulation behavior, draw
streams and every surviving reading are unchanged; the goal summary's shape
changes.

## Non-Goals

- A new lab assay or reading: `docs/lab.md`'s why-not ladder already
  routes the questions the two experiments asked.
- `neighborhood/recruitment.rs` and `neighborhood/recruitment/` (T13.F01's
  module-recruitment census, the drift walk's `recruitment` and
  `opportunities` blocks), `neighborhood/input_use/` and
  `neighborhood/drift.rs` stay as they are.
- The Graph's tolerance of partial sink catalogs: T30.F12.
- The VM and Topology test-only samplers (`WORKAROUND(T25.F08)`): T30.F05.
- History keeps its text: closed specs, `docs/progress.md` rows,
  `docs/progress/features/*.json`, `docs/progress/readings/`,
  `docs/progress/historical-benchmark-manifest.json`, `docs/strategy/` and
  `docs/archive/`; no stored report or summary is rewritten.

## Inputs and Invariants

Inputs:

- The row and its Notes entry: retire rather than move (user, at T25.F11's
  closure); widened to T20.F01's assay and the `input-opportunity` command
  (user, 2026-10-27); the controllers and `Family` move to the lab with one
  shared Ring-wiring operation
  ([F09 design review](../../progress/readings/t30-f09-design-review.md)).
- `AGENTS.md`'s lab rule and `docs/architecture.md` section 2: `v3-lab` owns
  capability assays and is never depended on.
- The T25.F11 accept this row pays: `WORKAROUND(T25.F11)` at
  `neighborhood/recruitment_paths/mod.rs:1`.
- `docs/workflow.md`'s slow-tests row and the Makefile's test targets.
- The code inventory on `ac3747005` in the
  [readings](../../progress/readings/t30-f09.md).

Invariants:

1. No simulation behavior change: the gate report's `deterministic` block
   equals the epoch's once exactly one key is removed from the epoch's,
   `goal_indicators.recruitment_paths` (the string
   `unmeasured; recruitment-paths-v2 is goal-only` in every gate report);
   every remaining value is byte-identical. The gate summary's
   `deterministic` block equals the epoch summary's (summaries already omit
   the indicator), and its `environment` block lacks
   `recruitment_paths_wall_clock_ms` (`null` in the epoch's). Stored
   reports and summaries are not edited. Every deleted production item has
   no production caller outside the deleted set; no tick-loop, mutation-draw
   or genome code is edited beyond deleting `GraphMutator::apply`.
2. The lab's comparators are the same genomes: `area_food(founder)` and
   `barrier_comparator(founder)`, for the founder of `arena_config(64)`,
   serialize to the same canonical JSON before and after the move (the
   Vector controller at reference 7, the Ring reference at 8, `barrier[d] →
   Move(d)` at `RING_INHIBITION` for N, E, S, W, every node, weight and
   threshold unchanged). The pin is the SHA-256 of each genome's canonical
   JSON on `ac3747005` and on the final code, recorded in the readings; the
   lab's existing tests compare against current constants and pin nothing.
3. Dependency direction is unchanged: `v3-lab` imports `v3-core`; nothing
   imports `v3-lab`; `v3-core` gains no dependency and no new `pub` item.
4. After the change,
   `grep -rn "recruitment_paths\|recruitment-paths\|recruitment_seconds\|neighborhood::opportunity\|input-opportunity\|input_opportunity" crates Makefile scripts docs/workflow.md docs/benchmark-artifacts.md docs/reference docs/lab.md docs/architecture.md`
   matches nothing.
5. `GraphMutator::apply` is gone; every remaining caller passes its
   `SensorContext` to `apply_in`.
6. Stored reports and summaries still parse and compare: `bench/schema.rs`
   has no `deny_unknown_fields` and `bench/comparison.rs` never read
   `recruitment_paths`.
7. The slow tier's lab half is unchanged: `make rust-test-slow` runs the
   six `lab_run` tests the Makefile names and nothing else.
8. No test is deleted other than tests of deleted code; a test whose subject
   survives is kept and retargeted.

## Architecture

Owners touched: readers (`neighborhood/`), which lose two modules that were
capability experiments, the concept section 2 gives `v3-lab`; `mutation`
(`mutation/graph/mod.rs`), which loses a wrapper with no production caller;
`v3-cli` (commands, benchmark schema, runner, artifact projection);
`v3-telemetry` (the measurement command list and four record attributes
only the two commands set); and `v3-lab`, which gains the authored
controllers its comparators already compose. Debt paid: the T25.F11 unpaid
accept on `recruitment_paths`, marked at `neighborhood/recruitment_paths/mod.rs:1`;
section 5 never listed it because it breaks the lab rule rather than a
section-3 import, so no `scripts/architecture-debts.txt` line is involved.
Boundary changed: none; the desired state already places capability assays
in the lab, so `docs/architecture.md` needs no edit. One shared rule changes:
the slow-tests trigger, owned by `docs/workflow.md`'s Verification plan table
and applied by every spec's Verification Plan and by the `rust-test-slow`
target, keeps only its lab half, and the T02.F08 and T11.F31 worked
examples are reclassified under it. The Graph's
partial-catalog tolerance (T30.F12) is neither used nor paid: the deleted
fixtures build full catalogs (`recruitment_paths/fixtures.rs:526-535` maps
every sink to one of the same kind, emptying edge lists), so this row
touches nothing of it.

## Telemetry

Signals removed; none added; no counter, config value or switch changes.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `measurement.started` and `measurement.ended` with `petri.command` `recruitment` or `input-opportunity` | log records | the two retired commands | removed with them; `bench`, `profile-ticks`, `run` and the lab's `why-not` records are unchanged |
| `petri.stop_reason`, `petri.horizon`, `petri.gate_favorable` on `measurement.ended`; `petri.pilot` on `measurement.started` | record attributes | set only by the retired commands (`assay_end`; `--pilot`) | removed; `petri.incomplete` stays, the lab sets it |
| `recruitment_paths` goal indicator and `recruitment_paths_wall_clock_ms` | goal-profile report block and its timing | the retired experiment, run once per goal world set | removed; goal summaries from this closure on lack them; `docs/progress/benchmark-series.json` `counter_definition_changes` records the drop |

## Verification Plan

**Question.** F09 claims that deleting the two experiments, their indicator,
commands and tier half, and moving the controllers into the lab behind one
Ring-wiring operation, changes no trajectory, draw or surviving reading and
no lab comparator genome; a gate `deterministic` block differing from the
epoch's beyond the deleted `recruitment_paths` key, a comparator digest
that moves, or a lab test or slow-tier lab run failing, would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The track's verification note runs each row's gate with pins unchanged; the gate report's `deterministic` block against the epoch's, read with the retired `recruitment_paths` key excluded, is the byte-identity reading for a diff that touches `mutation/graph/mod.rs`, `neighborhood/mod.rs` and the benchmark schema, in seconds; no population reading is in question, and the only goal-only work that changes is the deleted experiment itself, so `goal` is not declared. |
| Slow tests (`make rust-test-slow`) | `run` | The workflow's trigger names any change to `crates/v3-lab`, which gains the controllers and the shared Ring operation; the tier holds the lab's end-to-end runs, which `make check` does not run, and the run also shows the retargeted `rust-test-slow` target. |

## Implementation Tasks

- [x] Pin first: before any other edit, record invariant 2's two digests
      in the readings (a lab test run with `--nocapture`, kept only in the
      working tree); the same step on the final code must print the same.
- [x] Lab: add `crates/v3-lab/src/controllers.rs` with the controller
      construction moved from `neighborhood/opportunity/controllers.rs`
      (`Family` with `reference()`, the constants with `RING_MOVES`,
      `controller`), dropping the assay-only parts
      (`Family::ALL`, `index`, `as_key`, `authored_sub_indices`,
      `authored_channels`, the inert `Z_k` scale, the `Scalar` family, the
      `AuthoredStructure` counts and derives nothing in the lab uses). One
      operation appends a family's reference at the vote node's next index
      and wires its structure reading that index; `controller()` (asserting
      the founder's seven references) and `comparator::barrier_comparator`
      (asserting eight) both call it, so the comparator no longer repeats
      the Ring wiring. `comparator.rs`, `calibration.rs`, `eval.rs` and
      `tests/lab_run.rs` import the lab module. The assay's test of
      production visibility (diagonal barriers hide no cardinal ring cell)
      moves to `sensors::visibility`'s tests.
- [x] Core: delete `neighborhood/recruitment_paths/`,
      `neighborhood/opportunity/`, their `pub mod` lines,
      `crates/v3-core/tests/recruitment_paths.rs` and
      `mesh_execution::ancestral_payload_replacement` (T13.F07's
      counterfactual, no other caller); delete
      `GraphMutator::apply`, its test callers passing a context to
      `apply_in`; `mutation/engine/mod.rs`'s doc names the drift walk as
      the one instrument on `FOUNDER_GENOME_SIZE_UNITS`.
- [x] CLI: delete `v3-cli/src/recruitment.rs`, `opportunity.rs`, their
      `pub mod` lines, the `Recruitment` and `InputOpportunity` arms, args
      and runners in `main.rs`, and the `measure` module's `AssayReadings`,
      `assay_end`, `read_summary`, `recruitment*` and `opportunity*`
      functions with `measure/tests.rs`, and the two commands' tests in
      `tests/cli.rs` and `tests/telemetry/measurement.rs`; the command doc
      comment lists the surviving commands.
- [x] Bench: delete `ProfileParams.recruitment` and every `Sizes` use,
      `GoalIndicators.recruitment_paths`, `undefined_recruitment_paths`,
      `timed_recruitment_paths`, the `recruitment_paths_wall_clock_ms`
      fields of `RunTimings` and the environment block, the artifact
      projection (`constructed_stage`, `recruitment_summary`, the
      `recruitment_paths` branch, its eight `claims` pointers with
      `MAX_CLAIMS` set to the eight that remain, the static
      `omitted_details` notes "per-proposal rows" and "replay paths", and
      the `wall_caps.recruitment_seconds_per_report` field of
      `measurement_evidence`) and the tests of those paths in
      `run/tests.rs` and `tests/bench_artifacts.rs`.
- [x] Telemetry: `MeasurementStart.command`'s doc lists the surviving
      commands; `MeasurementStart.pilot`, `MeasurementEnd.stop_reason`,
      `horizon` and `gate_favorable`, their attribute lines and their test
      rows are deleted.
- [x] Makefile: `rust-test-temporal-fixtures` loses its `recruitment_paths`
      line; `rust-test-slow` loses its recruitment line and names only the
      lab runs in its help text.
- [x] Docs: `docs/workflow.md`'s slow-tests `run` row keeps the lab half and
      the T02.F08 and T11.F31 worked-example rows read `not run`;
      `docs/benchmark-artifacts.md` drops the Recruitment S0 row, the
      recruitment items of the summaries-omit sentence and of the `claims`
      list (with its count); the four `docs/reference/` passages name the
      drift walk as the one instrument; `docs/progress/benchmark-series.json`
      gains a T30.F09 `counter_definition_changes` entry for
      `recruitment_paths` and `recruitment_paths_wall_clock_ms` (before: the
      `recruitment-paths-v3` goal-only block; after: absent; comparison: no
      like-for-like; past full reports keep the block and past summaries
      its wall clock, claims and omitted-path entry, as history).
- [x] Gate profile, `make rust-test-slow` and `make check` on the final
      code; readings and summary recorded.

## Verification

- [x] Invariant 4: the grep -> empty (command and output in the readings).
- [x] Invariant 5: `grep -rn "GraphMutator::apply(" crates` -> empty.
- [x] Invariant 2: the two comparator digests on `ac3747005` and on the
      final code after self-review, equal (commands and digests in the
      readings);
      `cargo test -p v3-lab` -> pass with the existing comparator tests'
      assertions unchanged.
- [x] Invariant 6: `cargo test -p v3-cli` -> pass, the artifact reader's
      historical-report tests among them.
- [x] Invariant 7: `make rust-test-slow` -> exit 0 on the final code, lab
      tests only.
- [x] Invariants 1, 3 and 8: `make check` -> exit 0; `make check-docs` ->
      exit 0.
- [x] Gate profile and invariant 1: summary at
      `docs/progress/features/t30-f09.json`; a structural diff of the gate
      report's `deterministic` block against the epoch's full report (the
      main checkout's `.bench-artifacts/t25-f11/gate.json`, when present
      locally) lists exactly the removed `goal_indicators.recruitment_paths`,
      and the same diff of the two summaries' `deterministic` blocks lists
      nothing (both done; diffs, hash and byte counts in the readings);
      series entry added, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** No production path changes:
`neighborhood/` is read after a run, never from the tick loop, and the
deleted `GraphMutator::apply` has no production caller once the experiment
is gone. Expected compute cost: none. The gate is compared against the
current epoch (`t25-f11.json`) with the standard thresholds and no re-pin.
The `deterministic` block is expected identical to the epoch's except for
the deleted `goal_indicators.recruitment_paths` key (invariant 1); any
other difference is a defect, not a cost. The gate never ran the
experiment, so its wall clock moves only with the host. No indicator can
move; `recruitment_paths` is removed, not redefined, and the series records
the drop.

**Measured verdict.** Gate: make and CLI exit 0, `severe=false`, metrics ok;
the `deterministic` diff lists only `recruitment_paths`; bytes in readings.

- Summary of the declared profile: [gate](../../progress/features/t30-f09.json).
- Full readings: [`docs/progress/readings/t30-f09.md`](../../progress/readings/t30-f09.md).

## Success Criteria

- [x] Invariant 4's grep is empty: both experiments, their indicator, both
      commands and the recruitment slow-tier line are gone.
- [x] The lab builds its comparators from its own `controllers.rs` through
      one Ring-wiring operation, and their genomes are unchanged
      (invariant 2's digests equal).
- [x] The gate's `deterministic` block equals the epoch's apart from the
      deleted `recruitment_paths` key; `make check`, `make rust-test-slow`
      and `make check-docs` exit 0 on the final code.
- [x] `docs/workflow.md`, `docs/benchmark-artifacts.md`, the four
      `docs/reference/` passages and `benchmark-series.json` describe the
      state after the change.

## Notes for AI Agents

- Decision: (user, 2026-10-27) the experiments retire rather than move,
  T20.F01's assay and the `input-opportunity` command included; the
  controllers and `Family` move to the lab
  with reference indices, constants and genomes unchanged.
- Decision: the controllers land in `crates/v3-lab/src/controllers.rs`;
  `controller()` and `barrier_comparator` share one family-append
  operation; the assay-only parts of the controllers (`Family` helpers, the
  `Scalar` family, the `AuthoredStructure` counts) are deleted, not moved.
- Decision: `GraphMutator::apply` is deleted outright; test callers pass an
  explicit `SensorContext`, never a hidden default.
- Cost: $37.11 (Opus 5.5 8.8M cache read); 3 implementer passes (advisor
  2/1/2); 3 spec-owner resumes; 2 Codex rounds, `ready`; review: P1 1,
  design 0, health P3 2.
