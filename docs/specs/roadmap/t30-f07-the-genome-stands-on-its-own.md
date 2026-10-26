# T30.F07 — The Genome Stands on Its Own

**Status**: In Progress
**Last updated**: 2026-10-26
**Feature**: T30.F07
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Offspring inherit and brains run exactly as before, with the genome a domain
of its own: `crates/v3-core/src/genome/` is its one home beside the creature
record, the four censuses that read creatures live with the other readers in
`neighborhood/`, and the profiling harness is a `v3-cli` subcommand under
the binary's telemetry switch. Debts G24, G25 and G26 are paid and deleted,
and `scripts/architecture-check.mjs` classifies every module by its path
alone. Every pin, deterministic block and existing test value is identical
to main's; any that moves is a defect.

## Non-Goals

- No change to any genome type, operation, analysis, serde form, draw,
  default or test value, or to what any reader computes; no compatibility
  re-export (no `creature::genome` or `creature::<reader>` path survives
  under `crates/`, in code or comments).
- No other module moves: the rest of `creature/` stays (G12 is T30.F05's,
  G27 T30.F08's); `neighborhood/` keeps its name (section 3).
- No change to the checker's rules or the debt list's form beyond
  invariants 4 and 5.
- No new `profile-ticks` option, summary or report: its telemetry is
  invariant 3's record pair, nothing more.
- No rewrite of old paths in closed specs, notes, readings or archives.

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F07 (G24, G25, G26; the
`dhat-heap` feature, `dhat` dependency and telemetry flag; "lands while no
row that edits genome code is in flight"), the track
[design review](../../progress/readings/t30-design-review.md) findings 13
and 33, `docs/architecture.md` sections 1–3 and 5, the T21 telemetry-flag
rule (one global switch per binary) and the T21.F06 measurement-command rule
(a record pair exported after the last timed region), and the code on
`7b396c98`. Paths in this spec are today's. The code sites
read, the options weighed and the preparatory audit are in the
[readings](../../progress/readings/t30-f07.md).

Invariants:

1. `crates/v3-core/src/genome/` is the genome's one home, declared
   `pub mod genome;` in `lib.rs`; `creature/mod.rs` declares no `genome`
   and re-exports nothing from it. Every path under `crates/` names
   `crate::genome` or `v3_core::genome`; genome-internal relative paths are
   unchanged; `grep -rn "creature::genome" crates` is empty.
2. `drives`, `ring_rules`, `sensor_census` and `plasticity_census` with
   their three test files live in `neighborhood/`, declared in
   `neighborhood/mod.rs`, whose module doc describes the readers domain of
   section 3 rather than the T11.F01 indicator alone; `ring_rules` reaches
   `sensor_census` inside `neighborhood`. `neighborhood/companions.rs`, a
   re-export of two genome items, is deleted with its `pub mod` and
   `pub use`; its parity test (`creature/genome/companions.rs:144-156`)
   keeps its structural assertion (a `LoadSlotPrev` genome reads shared
   memory) and loses the two re-export comparisons, and its one production
   caller, `v3-cli/src/bench/indicators.rs:32-33`, names `v3_core::genome`.
   `grep -rnE "creature::(drives|ring_rules|sensor_census|plasticity_census)" crates`
   is empty.
3. `v3-cli profile-ticks [--ticks N] [--creatures N] [--world-size N]`
   (defaults 200, 200, 200) is today's harness: the default config sized
   by the arguments and normalized, seed 42, `run_tick` per tick, the two
   stderr lines, and the `dhat` allocator and heap profiler under the
   `dhat-heap` feature.
   The feature and its optional `dhat` dependency move to
   `crates/v3-cli/Cargo.toml`; `crates/v3-core` keeps neither and has no
   `src/bin/`. The
   subcommand is a measurement command under the global `--telemetry`
   switch, as `bench`, `recruitment` and `input-opportunity` are: the
   Telemetry table's record pair through `v3_cli::telemetry::Measurement`,
   started before the timed loop and exported after it, nothing inside it,
   and nothing with the switch off. Argument handling takes clap's
   semantics (usage errors exit 2, a repeated option is rejected) where the
   hand parser exited 1 or panicked; for valid arguments the resolved
   workload is unchanged.
4. `scripts/architecture-check.mjs` has no `creature`→`genome` case, no
   `CREATURE_READERS` list and no `bin/` skip, and none of the three
   `WORKAROUND(G24|G25|G26)` comments. Its tests keep every assertion with
   example paths under `genome/` and `neighborhood/` (the readers test:
   `neighborhood/sensor_census.rs` may import `creature::state`,
   `creature/state.rs` may not import `neighborhood::sensor_census`).
   `make architecture-check architecture-check-test` passes.
5. `docs/architecture.md` section 5 has no G24, G25 or G26 row, and rows
   G1–G4, G14 and G15 name the moved items by their new paths
   (`genome::...`, `neighborhood::drives`). `scripts/architecture-debts.txt`
   re-keys, and neither pays nor adds, the two lines whose owner moves: G2
   `sensors/static_inputs.rs` to the checker's form for
   `crate::genome::{OutcomeChannel, OUTCOME_CHANNEL_COUNT}` (one line per
   root item), and G14 `simulation/actions/reproduction.rs` to
   `neighborhood::drives`. Sections 1–4 are not edited.
6. The proptest regression files follow their modules
   (`proptest-regressions/genome/cgp/container_tests.txt`,
   `proptest-regressions/neighborhood/sensor_census.txt`); neither remains
   under `proptest-regressions/creature/`.
7. No behavior change: the gate's deterministic blocks are identical to the
   epoch, every existing test value that a simulation produces is unchanged,
   no test is ignored or deleted (the checker's readers fixture and the
   companions parity test are rewritten, not dropped), and the only new
   tests are the harness's own. The frontend's sha256 fingerprint of
   `cgp/group.rs` (`petriVizDrift.test.ts:21,73`) is a source-review
   marker, not a simulation value; it takes the new path and hash, and
   the `runtime/cgp/eval.rs` one (`:20`, import line only) its new hash.

## Architecture

Owners touched: genome (gains its home at level 5), the creature record
(loses the genome submodule and the four readers), readers
(`neighborhood/`, gains the four censuses and loses a re-export shim),
`v3-cli` (gains the harness under "commands, benchmarks, measurements") and
`v3-core` (loses the binary section 2 forbids it). Debts paid: G24, G25,
G26; their rows and the checker's three special cases are deleted in this
feature (task 5), since the checker fails on a stale debt line once the
imports move. No boundary changes and no desired-state edit: section 3 already
names `genome (genome/)` and `readers (neighborhood/)`, section 2 already
places binaries outside the library crate, and section 5's remaining rows
only change the paths they cite. No shared rule changes owner: the checker
loses its path special cases, and G2 and G14 stay with T25.F11, re-keyed.
The row touches the import lines of every feature that edits genome code,
so it lands while no such row is in flight (track Notes).

## Telemetry

No counter, config value, switch or work is added to a run; each
`profile-ticks` invocation appears in the stack as a measurement.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `measurement.started` | log record | `v3-cli profile-ticks` beginning its timed loop, through `v3_cli::telemetry::Measurement` (T21.F06) | `petri.command` `profile-ticks`, `petri.seed` (`profile::SEED`), `petri.config_digest` of the effective config, the resource set and the measurement's own `petri.run_id` |
| `measurement.ended` | log record | the loop's end, exported after the last timed region | `petri.exit_code`, `petri.wall_seconds`, body `{"creatures_remaining":N,"ticks":N}`; no summary path, raw hash or byte count (none applies) |

## Verification Plan

**Question.** F07 claims the genome, four readers and the profiling harness
can change home with no change to any trajectory, draw, value or cost; a
moved deterministic block or a changed test value would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The track's verification note declares each row gate with pins unchanged; the gate's deterministic blocks against the epoch are the byte-identity reading of the tick loop whose every module is renamed, in seconds, and no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `run` | The workflow's trigger names any change to `crates/v3-lab`, whose 12 files change their genome import lines here, and the tier holds the lab's end-to-end runs (`Makefile:80`), which `make check` does not run. |
| Telemetry overhead check | `run`, `profile-ticks` workload | The workflow's telemetry rule 3 runs it when a feature adds work to a run; the harness gains a record pair, so, as T21.F06 did for `bench`, the check runs on the harness's own path: `PETRI_OVERHEAD_WORKLOAD=profile-ticks scripts/telemetry-overhead`, ceiling 1.10. |

## Implementation Tasks

- [x] Genome: `git mv crates/v3-core/src/creature/genome
      crates/v3-core/src/genome`; `pub mod genome;` in `lib.rs`; delete the
      declaration in `creature/mod.rs`; rewrite `crate::creature::genome` →
      `crate::genome` and `v3_core::creature::genome` → `v3_core::genome`
      across `crates/`, and the grouped import at
      `crates/v3-core/tests/baseline_worlds.rs:746`; shorten the
      `include_str!` path at `creature/genome/vote.rs:434` by one `../` so it still
      reaches `frontend/src/types/vote-sinks.json`; `git mv` the
      `container_tests.txt` regression file; in `frontend/`, the path and
      sha256 in `petriVizDrift.test.ts:21,73` and the path in the
      `vote-sinks.ts:10` comment (invariants 1, 6, 7; audit R1).
- [x] Readers: `git mv` the seven files to `crates/v3-core/src/neighborhood/`;
      declare the four modules in `neighborhood/mod.rs` and delete them from
      `creature/mod.rs`; rewrite every path to them (sites in the
      readings); `git mv` the
      `sensor_census.txt` regression file; rewrite the `neighborhood/mod.rs`
      module doc (audit R4); delete `neighborhood/companions.rs` with its
      `pub mod` and `pub use`, and rewrite the parity test at
      `creature/genome/companions.rs:144-156` to assert only against the genome API
      (audit R5) (invariants 2, 6).
- [x] Harness: `crates/v3-cli/src/profile.rs` with an options struct
      (ticks, creatures, world size) and a run function returning the
      elapsed time and remaining population; `Commands::ProfileTicks` in
      `main.rs` with clap arguments defaulting to 200; under
      `#[cfg(feature = "dhat-heap")]`, the `dhat` global allocator in
      `main.rs` and the heap profiler scoped to the subcommand, as the
      binary scoped it; the feature and dependency moved from
      `crates/v3-core/Cargo.toml` to `crates/v3-cli/Cargo.toml`; `git rm
      crates/v3-core/src/bin/profile_ticks.rs`; the doc comment carries
      today's two workflows (samply on the `profiling` build, `dhat` through
      the feature) with the new invocation; the measurement pair through
      `main.rs`'s `measure` module, with
      `profile-ticks` added to the two doc comments that list the
      measurement commands (`crates/v3-telemetry/src/lib.rs:332`,
      `v3-cli/src/main.rs:12-15`); a `profile-ticks` workload in
      `scripts/telemetry-overhead` beside `run` and `bench`
      (`v3-cli profile-ticks --ticks T`); one unit test runs the function for two
      ticks on a 20×20 world with ten creatures, and one test in
      `crates/v3-cli/tests/telemetry/measurement.rs` runs the subcommand
      with `--telemetry on` against the receiver fixture and checks the
      pair's command, digest, exit code and body (invariant 3; audit R2).
- [x] Checker: delete the three special cases and their comments from
      `scripts/architecture-check.mjs`; rewrite the fixture paths in
      `scripts/architecture-check.test.mjs` (invariant 4).
- [x] Debts and docs: delete the G24, G25 and G26 rows and update the paths
      in G1–G4, G14 and G15 in `docs/architecture.md`; re-key the G2 and G14
      lines in `scripts/architecture-debts.txt` (invariant 5).
- [ ] `make check`; the gate profile; `make rust-test-slow` at Close.

## Verification

- [x] Paths: the two greps of invariants 1 and 2 -> empty;
      `ls crates/v3-core/src/creature/genome crates/v3-core/src/bin` -> no
      such directory; `grep -n "dhat" crates/v3-core/Cargo.toml` -> empty.
- [x] Architecture: `make architecture-check architecture-check-test` ->
      exit 0 with no G24, G25 or G26 entry (20/20 tests); `grep -n
      "WORKAROUND(G2[456])" scripts/architecture-check.mjs` -> empty;
      `npx vitest run src/components/inspector/petriVizDrift.test.ts` -> 91/91.
- [x] Harness: `cargo run -p v3-cli -- profile-ticks --ticks 2 --creatures
      10 --world-size 20 --telemetry off` -> two stderr lines, exit 0;
      `cargo check -p v3-cli --features dhat-heap` -> exit 0; `cargo test
      -p v3-cli profile` -> the unit and measurement-record tests pass.
- [x] Crate suites: `cargo test -p v3-core -p v3-cli -p v3-server -p v3-lab
      -p v3-telemetry` -> pass counts in the readings, no expected value
      edited, no test ignored or deleted; `cargo check --workspace
      --all-targets`, `cargo fmt --all --check`, `make rust-clippy` -> exit 0.
- [ ] `make check` -> exit 0 at the tested commit; `make rust-test-slow` ->
      exit 0 on the final code (Verification Plan).
- [ ] Overhead: `PETRI_OVERHEAD_WORKLOAD=profile-ticks scripts/telemetry-overhead`
      -> inconclusive twice, no ratio (readings); user decision pending.
- [ ] Gate profile: summary stored at `docs/progress/features/t30-f07.json`,
      local raw hash and byte count and verification time checked, series
      entry points to the summary, deterministic blocks identical to the
      epoch, no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** A code move with no new work
inside simulation execution: no draw, value or default changes, the
harness's record pair sits outside its timed loop, and the harness is not in
the gate's path. Expected compute cost: none; the gate is compared against the current
epoch (`t27-f03-giving-energy.json`) with the standard thresholds and no
re-pin. Every deterministic block is expected identical to the epoch; a
differing block is a defect, not a cost to accept. No indicator can move.
Overhead check: `scripts/telemetry-overhead`'s method on the harness (as
T21.F06 did for `bench`), ceiling 1.10; expected well under it, since the
record pair sits outside the timed loop.

**Measured verdict.** Overhead: inconclusive twice, no ratio, 89.6 s of
300 s used (the script reuses the `run` workload's T 6908: 15.1 s runs,
n 24; the permitted rerun at T 1800: 3.7 s runs, 16.5% spread, n 32).
Reported to the user under the time-cap rule; not rounded, not rerun.

- Summary of the declared profile:
  [gate](../../progress/features/t30-f07.json).
- Full readings: [`docs/progress/readings/t30-f07.md`](../../progress/readings/t30-f07.md).

## Success Criteria

- [ ] `crates/v3-core/src/genome/` is the genome's one home and the four
      readers live in `neighborhood/`; no old path remains under `crates/`.
- [ ] `v3-cli profile-ticks` runs today's harness as a measurement command
      under the binary's `--telemetry` switch and `dhat-heap` feature;
      `v3-core` has no binary, feature or `dhat` dependency for it.
- [ ] The checker classifies every module by its path alone, its tests
      pass, and `make check-docs` passes with G24–G26 deleted.
- [ ] The gate's deterministic blocks and every existing test value are
      unchanged.

## Notes for AI Agents

- Decision: audit R1–R5 (readings) are in-feature: the genome moves to
  `genome/` with no re-export, the readers to `neighborhood/`, the harness
  becomes `v3-cli profile-ticks`, the `neighborhood/mod.rs` doc describes
  the readers domain, and the `neighborhood/companions.rs` shim is deleted.
- Decision: `profile-ticks` is a measurement command under the T21.F06
  rule: one record pair exported after the timed loop, nothing inside it;
  run telemetry is not installed, since it would add work to the profiled
  loop.
