# Decomposition Review — 2026-09-13

**Status**: Review report; proposes candidate work, changes nothing
**Snapshot**: `main` at `c0332d77` (T15.F01 closed)
**Raw evidence**: `~/.local/share/petri-tools/mutants/*/mutants.out/{outcomes.json,log/}`
(38 runs, 2026-09-04 → 2026-09-13), the fresh-run summary lines recorded in
`docs/specs/roadmap/*.md`, and warm timing measurements taken on the user's
machine today with the host otherwise idle.

## 1. The question, answered

> Mutation tests are taking longer and longer. Is that a sign the tests need
> decomposing into smaller files and smaller sets?

**No.** Splitting test files changes nothing the mutation gate pays for, and
adding more integration-test files makes it slightly worse. The gate's cost is
almost entirely **compilation**, and the single biggest item is a build setting,
not code shape. Two things grew at once:

1. **Mutant count per feature tracks the size of the production diff**, at
   roughly 10 mutants per 100 changed production lines (range 7–30 across the
   38 runs; the line count includes inline test blocks, so the true density is
   somewhat higher). Recent features simply shipped larger diffs (T13.F01 181
   mutants / 1,846 lines; T14.F03 170 / 842; T15.F01 126 / 1,152; T12.F04 333 /
   2,973) than the mid-T11 run of features (20–50 mutants on 120–750 lines).
   Decomposition changes this only at the margin: cargo-mutants selects a
   mutant when *any line of its span* is in the diff, and function-body
   replacement mutants span the whole function (`in_diff.rs`), so touching one
   line of a 170-line `bench.rs` function pulls in that function's one to three
   replacement mutants. Bounded and second-order; diff size is the driver.
2. **Cost per v3-core mutant roughly doubled** (T11.F17: 120 v3-core mutants in
   10 min; T13.F01: 181 in 13 min; T14.F03: 170 in 20 min; T14.F05: 92 in 10
   min — with two workers, ≈6 s → ≈9–13 s serial each), driven by the build
   medians in §2. **T15.F01's 126 mutants in 29 min is a different case**: all
   v3-cli, where the cost is the test phase (§2.2, §4.3). The answer in both
   cases is not "smaller test files".

Everything below is measured, not inferred.

## 2. What one mutant actually costs

cargo-mutants applies a mutant, runs `cargo test --no-run --profile mutants
--package <crate>`, then `cargo test` on that package. For every v3-core mutant
the build phase recompiles the crate **twice** (the plain lib and the `--test`
lib that carries every inline test module) and relinks 11 integration-test
binaries plus `profile_ticks` twice — 15 rustc invocations. The test phase
usually runs only the first binary, because a caught mutant fails there and
cargo stops.

Recorded phase medians across all 38 stored runs:

| period | baseline build | per-mutant build (median) | per-mutant test (median) |
| --- | --- | --- | --- |
| 2026-09-04 – 09-05 (before `fcf00f6c`) | 18–23 s | 1–3 s | 13–25 s (v3-core, unoptimised test profile) |
| 2026-09-05 – 09-09 (`opt-level = 1` mutants profile) | 30–68 s | 2–6 s | ~1 s |
| 2026-09-10 – 09-13 | 36–70 s | 8–33 s | ~1 s (v3-core); 10–30 s (v3-cli) |

The break at 2026-09-05 19:55 is `fcf00f6c` (`docs/specs/mutation-test-efficiency.md`),
which introduced the `mutants` profile at `opt-level = 1`: baseline test fell
from 42–65 s to ~9 s and baseline build stepped up from ~20 s to 31–36 s, then
grew with the crate. That pass traded build time for test time and left `lto`
at its default; this review finds the build side is now dominated by that
default.

Per-mutant **test** time has not moved for v3-core: the whole v3-core suite runs
in ~6 s (unit binary 3.3 s, eleven integration binaries ~3 s together), and a
caught mutant fails in the first binary in under a second. Per-mutant **build**
time is what grew, and it varies by *which file* was mutated:

| mutated file (recent runs) | median build | fan-in |
| --- | --- | --- |
| `runtime/cgp/effects.rs`, `runtime/vm.rs`, `plasticity/hebbian.rs`, `genome/cgp_mesh_annotations.rs` | 2–5 s | leaf |
| `simulation/tick.rs`, `kernel/occupancy_grid.rs`, `plasticity/reward.rs` | 10–18 s | hub |
| `creature/sensor_census.rs`, `genome/cgp_analysis.rs`, `simulation/energy_accounting.rs`, `simulation/reproductive_success.rs`, `neighborhood/drift.rs` | 19–34 s | hub |
| `runtime/traced_vm.rs` | 49 s (n=1) | hub |

That pattern — cost proportional to how widely the mutated item is used inside
the crate — is the signature of incremental re-codegen of the **`lib (test)`
unit**, which for v3-core is one 68,000-line compilation unit: 26,051
production lines plus 41,926 lines of inline tests. The 11 integration binaries
are a separate, much smaller unit each (5,838 lines total).

### 2.1 Direct measurement of one hub-file mutant

Warm `target/mutants`, host idle, one-token semantic change in
`crates/v3-core/src/simulation/tick.rs:735` (`+= 1` → `+= 2`), then
`cargo test --profile mutants -p v3-core --no-run --timings`:

| unit | current profile (`lto = false`) | `lto = "off"` |
| --- | --- | --- |
| `v3-core lib (test)` | **14.3 s** | **3.4 s** |
| `v3-core lib` | 2.7 s | 1.9 s |
| 11 integration test links + 2 `profile_ticks` | 0.4–1.0 s each, in parallel | same |
| **wall** | **14.4 s** | **3.5 s** |

`-Ztime-passes` on the current profile's `lib (test)` compile: 12.0 s total, of
which `LLVM_thinlto` 9.2 s and `finish_ongoing_codegen` 8.9 s (overlapping);
the front end is ~3 s. In Cargo, `lto = false` means *thin-local LTO stays on*
for any `opt-level > 0`; only `lto = "off"` disables it. The mutants profile
(`Cargo.toml:59`) uses `opt-level = 1` with `lto = false`, so every mutant pays a
ThinLTO pass over a 68k-line unit. A comment-only touch of the same file (no
codegen change) rebuilds in 3.9 s under either setting, which confirms the cost
is codegen, not parsing or type-checking.

### 2.2 The other side of that trade: test runtime

Thin-local LTO does make the simulation code faster, and the tests run it:

| suite (full run, warm) | `lto = false` | `lto = "off"` |
| --- | --- | --- |
| v3-core, sum of per-binary times (two runs each) | 5.78 s / 5.82 s | 6.93 s / 6.91 s |
| v3-cli, sum of per-binary times (two runs each) | 26.0 s / 25.9 s | 32.4 s / 32.4 s |
| v3-cli mutant build (`bench.rs`, three-site one-token change; n=1) | 4.1 s | 3.0 s |

So for a **v3-core** mutant the change is roughly −11 s build, +0.2 s test (a
caught mutant runs only the 3.3 s unit binary, so +19% of that). For a
**v3-cli** mutant it is −1 s build and up to +6.5 s test — but only when the
mutant survives the 1 s unit-test binary and reaches `tests/bench.rs`, which
spawns the real binary and runs simulations for 22 s. That second case is the
T15.F01 case, and it is a test-shape problem (§4.3), not a profile problem.

### 2.3 Fixed cost per run

Every run also pays: the baseline build in a fresh scratch copy (40–70 s, 102
crates), the baseline test (5–13 s), the second worker's cold build of its own
scratch copy (82 dependency crates), and a 17-crate dependency recompile on the
first mutant because the baseline builds every package that has mutants
(`--package v3-cli --package v3-core`) while each mutant builds one, and the two
package sets unify dev-dependency features differently. That is a
~2-minute floor: T14.F06 tested 2 mutants in 1.6 min, T15.F01's 3-mutant
remediation pass took 2.6 min. It only matters for the small incremental passes.

## 3. Levers, ranked by measured effect

| # | lever | kind | effect | cost / risk |
| --- | --- | --- | --- | --- |
| 1 | `lto = "off"` in `[profile.mutants]` | one-line build config | v3-core hub mutants 14 s → 3.5 s build; leaf mutants ~3 s → ~2 s; cold baseline 51 s → 39 s | +19% test runtime (≈ +0.2 s per caught v3-core mutant, +1 s on the baseline; up to +6 s on a v3-cli mutant that reaches the bench integration tests). Must be validated the way `docs/specs/mutation-test-efficiency.md` validated `opt-level`: a fixed workload (T14.F03's 170-mutant diff is a good one — mostly v3-core hubs) with identical outcomes. Changing the profile mid-feature is a tool-configuration change under the workflow and forces a fresh run. |
| 2 | Unit-test the v3-cli bench code directly instead of only through the spawned binary | test shape | a v3-cli mutant caught by the 1 s unit binary costs ~5 s instead of ~30 s; T15.F01 had 21 misses and ~90 catches that mostly ran the 22 s `tests/bench.rs` | Ongoing discipline for T15 features; see §4.3. Not retroactive machinery. |
| 3 | Keep production diffs small per feature | scope | mutant count is roughly 0.1 × changed production lines (range 0.07–0.3) | Already the workflow's intent; worth stating in the spec template that a 2,000-line feature buys a 20-minute gate. |
| 4 | Move `simulation::energy_accounting` (and the other back-edges in §4.2) down a layer | app decomposition | reduces the fan-in of hub files, which is what makes their mutants 3–10× more expensive than leaf mutants | Small, mechanical; but its mutation-time effect is second-order once lever 1 removes the ThinLTO multiplier. Do it for layering reasons, not speed. |
| 5 | Consolidate the 11 v3-core integration binaries into fewer | test decomposition | saves ~9 CPU-seconds of linking per mutant, but they link in parallel, so ~1–2 s wall | Small. The opposite of "more, smaller test files": each new `tests/*.rs` is one more binary relinked per mutant. Don't add binaries for organisation; add modules under an existing binary (the `creature_workflow_e2e/` directory pattern already does this). |
| 6 | Split v3-core into crates | structural | a leaf-crate mutant would rebuild only its crate | **Changes the gate's meaning.** cargo-mutants tests the mutated package only by default (`test_package` / `test_workspace` exist in 27.1.0 to widen it). Splitting would drop downstream simulation tests from leaf-crate mutants unless `test_workspace = true`, which then rebuilds everything anyway. Not recommended for this reason; the layering in §4.2 is clean enough that it *could* be done later if there were another motive. |
| — | Split inline `#[cfg(test)]` blocks into sibling `tests.rs` files | test decomposition | **zero** — a sibling `mod tests;` file is compiled into the same `lib (test)` unit | Do it for navigability where blocks exceed ~1,000 lines (§4.3); expect no speedup. |
| — | Split big test functions / files under `tests/` | test decomposition | zero | Same. |

What would *not* help: nextest (already measured, 184 s vs 178 s), lowering
`opt-level` (282 s vs 178 s, already measured), or a coverage-to-test mapping
(explicitly a non-goal of the efficiency spec, and the numbers above show the
test phase is not the problem).

## 4. Application and test code: decomposition review

### 4.1 Shape by module (production vs test lines, current `main`)

| module | prod | test | files | note |
| --- | --- | --- | --- | --- |
| `v3-core/src/mutation` | 5,921 | 12,940 | 32 | tests 2.2× code; `graph/operators.rs` 2,175 lines (1,195 inline test) |
| `v3-cli/src` | ~5,100 | ~4,600 | 5 | `bench.rs` **8,171 lines**: 4,255 prod + 3,915 inline test; `bench/artifacts.rs` 881 (T15.F01) |
| `v3-core/src/neighborhood` | 3,995 | 3,632 | 16 | `recruitment_paths/` already a directory module with `fixtures.rs` |
| `v3-core/src/runtime` | 3,675 | 8,399 | 31 | `mesh.rs` 1,364 / `traced_mesh.rs` 1,146 — a traced twin of the mesh executor |
| `v3-core/src/simulation` | 3,240 | 6,827 | 25 | `tick.rs` 1,572; `actions/mod.rs` **144 prod / 1,552 test** |
| `v3-core/src/creature` | 3,091 | 4,450 | 16 | `genome/cgp.rs` 1,185 |
| `v3-core/src/kernel` | 2,286 | 1,765 | 19 | `ordinary_food/ecology.rs` 926 |
| `v3-core/src/config` | 1,317 | 1,437 | 3 | `simulation.rs` **2,561 lines** in three files |
| `v3-server/src` | ~4,200 | ~1,800 | 25 | `tests/server.rs` is one 3,749-line binary |
| `v3-core/src/sensors` | 1,164 | 1,222 | 6 | |
| `v3-core/src/patterns`, `contracts` | 1,273 | 1,254 | 15 | |
| **workspace** | **36,272** | **60,472** | | 99 files carry inline test blocks (27,411 lines) |

Growth since 2026-09-01: production lines 40,777 → 62,640 (that figure counts
inline test blocks as production; the split above does not), v3-core
integration binaries 4 → 11, v3-cli 1 → 3.

**Function level is fine.** `clippy::too_many_lines` is enforced at 167
(`clippy.toml`) with seven explained allows, the largest being
`v3-server/src/state.rs::build_ws_frame` (345 lines, flat field copying). The
review's findings are at file and module level.

### 4.2 Layering

`crate::` references between top-level v3-core modules in production code, rows
use columns:

```
              config contracts creature kernel mutation neighborhood patterns runtime sensors simulation
config             .      2        .      .       .          .          2        .       .       .
creature           4     10        .      .       1          .          .        1       .       3
kernel            13      7        .      .       .          .          .        .       .       .
mutation          13     17       22      .       .          .          .        4       .       .
neighborhood      16      8       19      1      19          .          .        5       3       3
patterns           .      .        .      7       .          .          .        .       .       .
runtime            8     16       25      .       .          .          .        .       9       4
sensors            4      6        4      4       .          .          .        .       .       .
simulation         7      9       21      6       4          .          1        9       5       .
```

The intended order — `config`/`contracts` → `kernel`/`sensors` → `creature` →
`runtime`/`mutation` → `simulation` → `neighborhood` — holds except for four
back-edges, all small and all localised:

| back-edge | where | what it is |
| --- | --- | --- |
| `runtime` → `simulation` (4), `creature` → `simulation` (3) | `runtime/types.rs:82,135`, `runtime/vm.rs:11`, `runtime/cgp/execute.rs:14`, `creature/state.rs:154,207,209` | all `simulation::energy_accounting::{DeathCause, CognitionEnergyObservation, applied_debit, observe_energy_change}`. Energy accounting is a leaf that three layers use; it lives one layer too high. Moving it beside `creature` (or into `kernel`) removes both back-edges. It is also one of the most expensive files to mutate (19.7 s median). |
| `creature` → `mutation` (1) | `creature/state.rs:12` | `MutationOperator`, a type used on creature state; belongs in `mutation::types` re-exported lower, or in `contracts`. |
| `creature` → `runtime` (1) | `creature/state.rs:114` | `runtime::plasticity::traces::decay_eligibility_traces` called from creature state; either the call belongs in `runtime`, or the trace-decay rule belongs with the state it decays. |

No cycles beyond these. This is a healthy graph; the fixes are three small
moves, not a restructuring.

### 4.3 File-level hot spots

**`crates/v3-cli/src/bench.rs` (8,171 lines).** Three times the next-largest
file. The production half (4,255 lines, 109 functions) is five things that its
`impl` blocks already delineate:

1. profile and recipe parameters (`gate_profile_params`, `goal_profile_params`,
   `goal_recipes_for`, `GoalCase`, `build_config`) — lines ~60–350;
2. the report schema (`Report`, `PerSeed`, `TrackedFractions`, the `Indicator<T>`
   family, `undefined_*` constructors) — ~350–1,420;
3. tracking observers that project `v3_core` state (`WorldTracking`,
   `OccupancyGrid`, `SensorCensus`, `PopulationReadings`, the `From<&v3_core::…>`
   impls, `PersistenceAccumulator`) — ~1,420–2,450;
4. the run loop (`run_one_seed`, `run_deterministic`, `throughput_rates`,
   `assemble_goal_indicators`) — ~2,450–3,800;
5. comparison (`ComparisonInputs`, `ReferenceSelection`, `ComparisonLevel`,
   `case_readings`) — ~3,800–4,255.

`bench/artifacts.rs` (T15.F01) already started this split. The remaining four
are natural `bench/{profiles,report,tracking,run,comparison}.rs` modules with
the 3,915-line test block distributed alongside. Why it matters beyond tidiness:
`bench.rs` is the most-mutated file in the repository (131 mutants in T12.F04,
46 in T10.F09, 42 in T01.F12, 30 in T11.F16, 26 in T14.F07) because every
telemetry feature lands there, and it is in the crate whose tests are slowest
per mutant. The split does not change mutant counts, but it makes the next
point actionable.

**v3-cli coverage shape.** `bench/artifacts.rs` has 3 inline tests for 881
lines; its coverage is `tests/bench_artifacts.rs` (921 lines, 19 tests, 4 of
which spawn the binary) and `tests/bench.rs` (20 tests, 6 spawning the binary,
22 s). Every mutant in that file therefore ran ~1 s of unit tests, passed, then
ran 22–27 s of end-to-end tests. Because cargo runs the unit binary first and
stops on the first failing binary, **a unit test that kills a mutant is worth
~25 s per mutant in this crate**. The tracking observers and the report schema
are pure functions over `v3_core` types and are the natural unit-test targets;
the spawned-binary tests should be the thin end-to-end layer. This is the one
place where the user's instinct about test shape is right — not smaller files,
but tests closer to the code.

**`config/simulation.rs` (2,561 lines: 1,194 prod + 1,367 test).** One file
holding every config struct, its defaults, validation, and the patch/apply
logic. A `config/{world,creature,mutation,…}.rs` split by the struct families it
already contains is straightforward; the test block splits the same way.

**`mutation/graph/operators.rs` (2,175 lines: 980 prod + 1,195 test)** and
`mutation/vm/operators.rs` (1,111). Operator families (structural, weight,
routing, memory) are already named in the code; a directory module per family
mirrors `mutation/topology/`, which has this shape already.

**`simulation/tick.rs` (1,572 lines)** with its tests already in
`tick/tests/` (11 files). The phase functions are separable
(`phase0`/actions/energy/reproduction are how the tests are already split); the
production file has not followed. Its mutants cost 12–18 s each because
everything in the crate depends on it — lever 1 helps this file most.

**`runtime/mesh.rs` (1,364) and `runtime/traced_mesh.rs` (1,146).** A traced
twin of the executor. Worth a look at whether the trace is a decorator over one
executor rather than a parallel implementation; if the two must stay in sync by
hand, that is a correctness risk independent of size. Not measured here.

**`simulation/actions/mod.rs`: 144 production lines carrying 1,552 lines of
tests.** The tests belong to the action modules they exercise
(`reproduction.rs`, `predation.rs`, `cgp_reproduction.rs`, …); the `mod.rs`
block is where they accumulated. Pure navigability, zero gate effect.

### 4.4 Test organisation

- **Ratio.** 1.67 test lines per production line workspace-wide, 2.2× in
  `mutation`, 2.3× in `runtime`. This is a consequence of the mutation gate
  (survivor remediation adds tests) and TDD; not a problem in itself, but it is
  why the `lib (test)` unit is 2.6× the lib.
- **Inline blocks over 1,000 lines** (`bench.rs` 3,915; `actions/mod.rs` 1,552;
  `config/simulation.rs` 1,367; `graph/operators.rs` 1,195) should move to
  sibling `tests.rs` or `tests/` directories, as `runtime/tests/`,
  `tick/tests/`, `mutation/vm/tests.rs` already do. Same compilation unit,
  same cost; better files.
- **Support helpers are not duplicated.** `tests/common/mod.rs`,
  `tick/tests/support.rs`, `creature_workflow_e2e/support.rs`,
  `recruitment_paths/fixtures.rs` and the `v3-cli/tests/bench.rs` helpers share
  no function names, and simulation seeding has one owner
  (`simulation/seeding.rs`).
- **Integration binaries.** v3-core has 11; several hold one to four tests
  (`applied_trajectory` 1, `vm_all_opcodes_e2e` 1, `recruitment_paths` 2,
  `priority_bid_reachability` 2, `reproducibility` 3, `mutational_neighborhood`
  4). Each is a link per mutant. When a new
  integration test is needed, prefer a module inside an existing binary
  (`creature_workflow_e2e/` is the pattern). Do not split the existing ones.
- **`v3-server/tests/server.rs` (3,749 lines, one binary)** is the right
  *number* of binaries and the wrong number of files; a `server/` directory
  module with one `server.rs` entry keeps one link and gives it structure.
- **Property tests** exist where the repository rules ask for them
  (`proptest-regressions/` present for `baseline_worlds`); nothing to change.

## 5. Recommended sequence

1. **Validate and adopt `lto = "off"` for `[profile.mutants]`** against a fixed
   workload with identical mutant outcomes, following the method already used
   for `opt-level`. Expected: v3-core-heavy gates drop by roughly 60%; v3-cli
   gates are neutral to slightly slower until step 2 lands. This is a
   tool-configuration change and needs the user's authorization before it can
   be applied under the workflow.
2. **For T15 features, require unit tests on the bench observers and report
   schema** in the spec's test plan, with the spawned-binary tests kept to the
   end-to-end contract. This is a spec-template line, not machinery.
3. **Split `v3-cli/src/bench.rs`** along the five seams in §4.3 as a
   maintenance feature (no behaviour change; the mutation gate on a pure move
   should produce zero mutants, which is also its verification).
4. **Move `simulation::energy_accounting` down a layer** and resolve the two
   `creature/state.rs` back-edges, as a second small maintenance feature.
5. Then, opportunistically as features touch them: `config/simulation.rs`,
   `mutation/graph/operators.rs`, `tick.rs`, and the four inline test blocks
   over 1,000 lines.

None of steps 3–5 will make the mutation gate measurably faster. Step 1 will;
step 2 will for the crate where step 1 does not.
