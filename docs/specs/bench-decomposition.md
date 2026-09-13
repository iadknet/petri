# Bench Module Decomposition

**Status**: Complete
**Last updated**: 2026-09-13
**Scope**: Maintenance; no roadmap feature ID, dependency row, or closure changes

## Goal

Split `crates/v3-cli/src/bench.rs` (8,171 lines: 4,255 production, 3,915
inline test) into a `bench/` module tree along its existing seams, with
`bench.rs` remaining the module root and re-exporting the current public
surface unchanged. Pure move: no behaviour, signature, name, or test-assertion
changes.

## Non-Goals

- Adding, rewriting, or removing tests; changing test coverage shape. The
  unit-test gap on the bench observers is a separate, later change.
- Renaming items, changing signatures or visibility beyond what the module
  boundaries require, reordering statements inside a function, or moving bench
  code into `v3-core`.
- Touching `bench/artifacts.rs`, `main.rs`, `inspect.rs`, or the integration
  tests except where an import path must change.
- Any change to benchmark profiles, thresholds, stored reports, or readings.

## Inputs and Invariants

The user request on 2026-09-13 is authoritative. Follow [the workflow](../workflow.md)
with the feature-template sections adapted here; roadmap ownership and
checkbox requirements do not apply. Rationale and measurements are in
[the decomposition review](../strategy/decomposition-review-2026-09-13.md) §4.3.
Work starts from the `main` commit that carries this spec, in a Claude worktree
named `bench-decomposition` under `.claude/worktrees/`.

**Current surface.** `main.rs`, `inspect.rs`, `bench/artifacts.rs`, and
`tests/{bench,bench_artifacts,cli}.rs` reach 27 names through `bench::` /
`v3_cli::bench::` (`Report`, `ProfileParams`, `ComparisonLevel`,
`ReferenceSelection`, `SeriesIndex`, `Indicator`, `NeighborhoodSizes`,
`Recipe`, `GoalCase`, `ThroughputRates`, `BenchmarkSeriesIndex`,
`build_report`, `build_report_with_threads`, `build_config`,
`gate_profile_params`, `goal_profile_params`, `compare_against`,
`compare_against_path`, `apply_comparisons`, `apply_comparisons_for_outputs`,
`default_gate_references`, `default_goal_references`,
`deterministic_block_json`, `report_json_pretty`, `throughput_rates`, `rfc3339_now`,
`artifacts`). Every one of these paths must still resolve after the split, via
`pub use` in `bench.rs`. Callers do not change.

**Target layout.** `bench.rs` keeps the `use` block, `pub mod` declarations,
and re-exports; each new file owns whole items (a type, its `impl` blocks, and
its `From` impls stay together; no function is split). Line ranges are from
`main` at `c0332d77` and are a starting map, not a contract — the implementer
may move an item to a neighbouring module where the map would separate a type
from its only user, and records each such deviation under Implementation
Tasks.

| module | contents | source lines (approx.) |
| --- | --- | --- |
| `bench/profiles.rs` | schema/counter constants, `NeighborhoodSizes`, `Recipe`, `ProfileParams`, gate/goal params, `GOAL_RECIPES`, `GoalCase`, `build_config`, `profile_block` | 34–348 |
| `bench/schema.rs` | serialised report types and `undefined_*` constructors: `Report` through `Indicator<T>` and the indicator/neighborhood/persistence structs; `Environment`, `Throughput*`, `Host`, `SeedWallClock`, `Comparison*`, `ComparisonLevel` | 350–878, 1140–1249, 1290–1410, 2035–2140, 2164–2293 |
| `bench/indicators.rs` | indicator computations over a finished simulation: `module_recruitment`, `mutation_opportunities`, drift depth, `mesh_execution`, `generation_distribution`, lineage/clade diversity, memory sensitivity, neighborhood battery helpers, `structure_size_distribution`, `assemble_goal_indicators` | 879–1139, 1250–1289, 2585–2947, 3007–3112 |
| `bench/tracking.rs` | per-tick observers and their `From<&v3_core::…>` impls: `MutationOutcomeTotals` … `EnergyFlowTracking`, `WorldTracking`, `CognitionTracking`, `PersistenceSample`, `OccupancyGrid`, `SensorCensus`, `PopulationReadings`, `PersistenceAccumulator` | 1411–2034, 2294–2402 |
| `bench/run.rs` | `SeedRun`, `GoalObservation`, `RunTimings`, `run_one_seed`, `prepare_goal_case`, totals, `run_deterministic`, host/environment detection, `rfc3339_now`, `build_environment`, `build_report*`, JSON helpers, `throughput_rates` | 2141–2163, 2403–2584, 2948–3006, 3113–3492 |
| `bench/comparison.rs` | readings, `compare_cases`, `compare_against*`, `ComparisonInputs`, `MeasuredIdentity`, `ReferenceSelection`, `apply_comparisons*`, series index | 3493–4255 |

**Tests.** The `mod tests` block (4,256–8,171) is distributed the same way: each
test moves, unchanged, to `#[cfg(test)] mod tests;` in the module that owns the
item it exercises (`bench/<module>/tests.rs`); a test that exercises items from
several modules goes to `bench/tests.rs`. Test bodies, names, and assertions
do not change; only `use` lines do.

**Purity rule.** Excluding the new files' first lines, the diff consists only
of moved lines and: `use`/`pub use`/`mod` lines, visibility widening
(`pub(super)` or `pub(crate)`) on items that now cross a module boundary, and
doc comments or attributes that must accompany a moved item, plus two
mechanical consequences of the move that `make check` forces: `cargo fmt`
rewraps of lines whose length changed only by a visibility keyword or by
dedenting out of `mod tests` (AST-preserving), and `include_str!` literals
gaining one `../` per extra directory level (source-file-relative, included
bytes unchanged). No other insertion or deletion. Existing `#[allow]`
attributes move with their items; none are added.

**Constraints.** Workspace lints (`clippy::too_many_lines` at 167, correctness
and suspicious denied) apply unchanged. Shell remains POSIX `sh`. Preserve any
unrelated user changes in the worktree.

## Implementation Tasks

- [x] Record the baseline: `cargo test -p v3-cli -- --list 2>/dev/null | grep -c ': test$'`
      is 162 on `c100db3b` (101 `lib.rs` unit, 11 `main.rs` unit, 20 `bench`,
      19 `bench_artifacts`, 11 `cli`; the earlier estimate of 151 omitted the
      `main.rs` unit binary) and the pre-move gate summary (Verification,
      first item).
- [x] Create the six modules per the target layout, moving whole items; add
      `pub use` re-exports in `bench.rs` so every current `bench::` path resolves.
      `bench.rs` re-exports every item that was `pub` before the move (not only
      the 27 names callers use) and privately imports the four comparison
      thresholds so `artifacts.rs`'s `super::FLAG_PERCENT` paths still resolve.
- [x] Distribute the test module per the Tests rule; fix `use` paths only.
      Each test went to the module owning its primary subject (the item its
      assertions target), not to every module whose types it touches; the
      five tests that drive `run_one_seed`/`run_deterministic` to assert on
      tracking, comparison, or companion output sit in `bench/tests.rs`
      together with the three helpers used from several test modules
      (`small_profile`, `small_world_set_params`, `small_world_set_report`,
      widened to `pub(super)`).
- [x] Run `cargo check --workspace --all-targets`, `cargo clippy --workspace
      --all-targets`, and `cargo test -p v3-cli`; resolve only visibility and
      import errors.
- [x] Record any deviations from the target map here, one line each, with the
      reason.
- Deviation: `FLAG_PERCENT`, `SEVERE_PERCENT`, `WALL_CLOCK_FLAG_PERCENT`,
  `WALL_CLOCK_SEVERE_PERCENT` live in `bench/comparison.rs`, not `profiles.rs`;
  they are comparison thresholds whose only users are `counter_level` and
  `artifacts.rs`.
- Deviation: `undefined_generation_distribution` lives in `bench/schema.rs`, not
  `indicators.rs`; `NeighborhoodEvolvedSeed` names it in a
  `#[serde(default = "...")]` string, which resolves in the struct's module.
- Deviation: `Comparison` lives in `bench/comparison.rs`, not `schema.rs`; its
  field doc links `` [`apply_comparisons`] ``, and `rustdoc::broken_intra_doc_links`
  is denied while an import used only by a doc link is reported unused.
- Exception: the six `include_str!` literals gained one `../` per extra
  directory level (`bench/profiles.rs` x3, `bench/comparison.rs` x1,
  `bench/schema/tests.rs` x2, which sits two levels down); `include_str!`
  paths are source-file-relative, so the bytes included are unchanged. The
  pure alternative is keeping `GOAL_RECIPES`, `locked_rand_version`, and the
  two historical-report tests in `bench.rs`.
- Exception: `cargo fmt` (a `make check` gate) rewrapped four signatures that
  `pub(super) ` pushed past 100 columns (`compare_cases`,
  `generation_distribution`, `memory_sensitivity`, `goal_case`) and rejoined
  thirteen dedented test statements that now fit on one line
  (`comparison/tests.rs` 205, 212, 437; `profiles/tests.rs` 58;
  `run/tests.rs` 69, 86, 140, 459, 492, 540; `schema/tests.rs` 46;
  `tests.rs` 160; `tracking/tests.rs` 572); both are AST-preserving.
- Size: `tracking.rs` (763) + `tracking/tests.rs` (1,147) and `indicators.rs`
  (797) + `indicators/tests.rs` (843) exceed ~1,300 combined; no natural
  neighbouring module exists and no extra split was invented. Every single
  file is under 1,150 lines.

## Verification

- [x] Pre-move gate profile on the starting commit (`c100db3b`), summary written outside
      the tree:
      `make bench PROFILE=gate FEATURE=bench-decomposition SUMMARY_OUT=/tmp/bench-decomposition-before.json`
      (raw output under the local default). Exit 0 on 2026-09-13; summary
      96,493 bytes at `/tmp/bench-decomposition-before.json`, all 12 metric
      levels `ok`.
- [x] Post-move gate profile on the final code (`a8611434`):
      `make bench PROFILE=gate FEATURE=bench-decomposition` (summary at
      `docs/progress/features/bench-decomposition.json`, committed with its
      reading and series entry per the workflow). Exit 0 on 2026-09-13;
      summary 96,820 bytes, all 12 metric levels `ok`, `severe=false`.
- [x] The two summaries' `deterministic` blocks are byte-identical:
      `node -e 'const fs=require("fs"); for (const [f,o] of [[process.argv[1],"/tmp/bench-decomposition-before.det"],[process.argv[2],"/tmp/bench-decomposition-after.det"]]) fs.writeFileSync(o, JSON.stringify(JSON.parse(fs.readFileSync(f,"utf8")).deterministic, null, 1))' /tmp/bench-decomposition-before.json docs/progress/features/bench-decomposition.json && cmp /tmp/bench-decomposition-before.det /tmp/bench-decomposition-after.det`
      exits 0. `cmp` exit 0; before 96,493 bytes, after 96,820 bytes.
- [x] Test inventory unchanged: the `--list` count equals the recorded
      baseline (162), and `cargo test -p v3-cli` passes with the same
      per-binary pass counts (101 / 11 / 20 / 19 / 11 on the split tree,
      2026-09-13).
- [x] `make check` exits 0 in the worktree (2026-09-13, on the split tree;
      orchestrator rerun at `398ad543`, the tested commit).
- [x] Independent review by `roadmap-reviewer` (2026-09-13, no P1; two P3
      deferred below) of
      `git diff -M --color-moved=dimmed-zebra --color-moved-ws=allow-indentation-change <base>..HEAD -- crates/v3-cli`
      confirms the purity rule: every non-moved line is a `use`/`mod`/re-export,
      a visibility change, or an accompanying doc comment or attribute. Any
      other line is a P1.
- [x] Mutation gate: **not applicable**, user decision 2026-09-13. A pure move
      introduces no new logic; the 594 mutants cargo-mutants lists for
      `bench.rs` all exist on `main` today and were gated when their features
      closed, and their killing tests move with them. Running the gate would
      re-test all 594 in the crate whose per-mutant cost is highest (~2–3 h)
      to learn nothing the purity review and test inventory do not already
      establish.
- [x] Goal profile: not applicable; no measured quantity changes.

## Performance and Goal Impact

None intended or expected. The move adds no mechanism, no indicator, and no
work; the byte-identical `deterministic` block is the check. Wall-clock fields
are not compared.

## Success Criteria

- [x] `crates/v3-cli/src/bench.rs` is a module root under ~400 lines (57);
      every file under `crates/v3-cli/src/bench/` is under 1,150 lines, with
      the combined `tracking` and `indicators` pairs over ~1,300 as recorded
      under Implementation Tasks.
- [x] All Verification items above are checked, with the non-applicable ones
      closed with their recorded reason.
- [x] This spec is Complete on `main`, `make check` exited 0 on the exact
      content now on `main`, and the worktree and branch are removed.

## Notes for AI Agents

- Exception: the orchestrator ran as Opus 5 (not Fable 5.1), authorized by
  the user on 2026-09-13 for this spec only; not a precedent.
- Decision: the purity rule admits rustfmt rewraps and `include_str!` prefix
  changes (orchestrator, 2026-09-13) because `make check` runs
  `cargo fmt --check` and the alternative — leaving `GOAL_RECIPES`,
  `locked_rand_version`, and two tests in `bench.rs` — contradicts the
  target layout. The reviewer verifies each such line is AST-preserving.
- Deferred: review P3 — five `// ── … ──` section-heading comments moved
  verbatim now sit above `mod tests;` in modules they do not describe
  (`profiles.rs`, `run.rs`, `tracking.rs`, `schema.rs`, `comparison.rs`);
  delete them in a later change.
- Deferred: review P3 — `comparison.rs` `percent_delta` is `pub(super)` with
  no cross-module caller (its tests are a child module); drop the keyword in
  a later change.
- Cost: implementer 1 pass, 3 advisor consults; reviewer findings P1 0, P2 0,
  P3 2; `/usage` totals to be added by the user.
- Exception: no `simplify` pass ran after the build brief; a pure move must
  not be simplified, and the purity review replaces it.

- Roles: Fable 5.1 `medium` orchestrator in the main checkout; feature
  implementation and any remediation by fresh Opus 5 `medium`
  `roadmap-implementer` passes; the two gate runs and their records by
  `roadmap-benchmark-specialist`; the final diff review by `roadmap-reviewer`.
  No mutation specialist is spawned (Verification). `make roadmap-check` still
  runs under the implementer gate and must pass; it is unaffected by this work.
- Give the implementer this spec, the target-layout table, and the purity rule
  verbatim. If an item cannot be placed without splitting a function or
  changing a signature, the implementer stops and reports rather than
  improvising.
- If the deterministic blocks differ, the move is not pure: do not adjust the
  comparison; find the moved statement whose order changed.
