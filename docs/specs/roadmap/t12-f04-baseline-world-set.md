# T12.F04 — Baseline World Set

**Status**: In Progress
**Last updated**: 2026-09-09
**Feature**: T12.F04
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

Three saved, fixed-map baseline worlds replace the goal profile's three
plains seed replicates: a food-differentiation world, a barrier-topology
world, and a composite. Each runs once per closure, under ten minutes for
the whole profile, and every closure from now on records the same per-world
readings so a world's trajectory can be followed as later features land.

This is the second pass at the feature. The first pass (Codex, 2026-09-09,
commits `398a21a9`..`73aaa379` on this branch) built the substrate and the
profile plumbing but shipped three unbalanced recipes and no way to follow a
world across closures; this pass reviews that work, redesigns the recipes,
and adds the tracking.

## Non-Goals

- No change to creature economics, founders, mutation, or the short gate.
- No claim that food specialization or barrier awareness evolves; the set
  only makes those pressures present and measurable.
- No seasons, disturbance, manual overlays (T12.F05), or new recipe format.
- No repair of the substrate facts recorded under Inputs; they belong to
  T11 and T03.F04.

## Inputs and Invariants

**Dependencies.** [T12.F01](t12-f01-seeded-terrain-in-the-world-config.md)
(seeded terrain layers), [T12.F02](t12-f02-world-recipe-save-and-load.md)
(partial-config recipes, digests). The
[research note](../../strategy/world-seeding-research-2026-09-08.md) holds
the archetype evidence. The track roadmap and the workflow's
[standard-baseline contract](../../workflow.md#environmental-pressures-in-the-standard-baseline)
fix the protocol: Orchards/11, Canyon/22, Confluence/33, one 2,000-tick run
each at 1600² with 10,000 production founders, `goal-worlds-v1` series.

**Already implemented by the first pass, kept after review.**
Per-type `energy_per_unit`, `growth_rate`, `recovery_spawn_rate` (null
inherits the shared value) and `initial_fertility_only` on
`world.food.types[]`; the `FbmThreshold` terrain variant; tick-zero
`passable_connectivity`; per-case goal profile execution with per-case
neighborhood, drift, and structure observations; frontend and reference-spec
carry-through. Their tests live in `crates/v3-core/tests/baseline_worlds.rs`
and the `v3-cli` bench tests.

**Substrate facts that shape the design.**

- Founders eat only food type 0: `decode_food_type_idx` rounds the Eat
  parameter, the founder emits 0, and a world with only type-1 food is extinct
  by tick 50 (probe, 2026-09-09). Codex's diagnostic found 0 of 1,000 mutant
  births selecting another type. A second food type is therefore a latent
  niche, not a choice the living population makes; the typed-eat counter added
  here reads whether that ever changes.
- Eaten cells hold zero density and regrow only by spread from a neighbor at
  or above 80% of max density or by recovery spawns below the 1% floor, so
  each type's growth rate and fertility set how fast grazed ground returns.
- Population booms to the 100,000 cap within about 50 ticks in every world;
  the boom's length is set by the tick-zero standing crop
  (coverage × density × energy per unit), and the plateau by regrowth reachable
  by creatures. Runtime is dominated by creature-ticks in the boom plus a
  fixed food-update cost of roughly 40 ms per tick per food type at 1600².
- A blocked move still pays `move_cost` and the tick; the failed-action
  penalty is only 3% ramped at tick 2,000. Barrier pressure comes from
  frequent blocked moves in a looped landscape, not from a spanning-tree maze
  (Codex's maze collapsed the population to 7). Founders are barrier-blind
  (no barrier input is wired), and the screening bisection showed the
  interaction that matters: poor grass (1.6 energy per bite) persists on open
  ground, production grass persists among barriers, poor grass among barriers
  collapses; the edge rule made no difference.
- Small fertility patches are grazed to nothing and never return; large ones
  are inexhaustible. The production 5–15-cell blobs cannot hold a population
  once half of them sit under rock (both fBm canyon drafts collapsed to single
  digits on the production fertility), while sixty 40–90-cell meadows hold a
  plateau in the low thousands to low tens of thousands.
- Codex's `FbmThreshold` samples the noise field in bounds-local
  coordinates, so two layers with different bounds read different parts of
  the same field; nested same-seed layers can only grade a region's edge when
  the field is sampled at world coordinates.
- Drift depth reads only the mutation config and the food-type count, never
  the world. Its depth-2,000 changed/all-birth reading is 0.005 with one food
  type (byte-identical to T11.F18's accepted exception) and 0.006 with two,
  both below the T11.F17 track floor of 0.008. T12.F04 cannot move it.

**Design decisions (recorded assumptions).**

- Maps are fixed: each recipe pins `world.world_seed`, so terrain and
  fertility are identical across run seeds and closures; run seeds still vary
  food placement, founder placement, and the run RNG. Resolves the research
  note's open Section 7 decision in favor of "saved baseline maps".
- Recipes stay partial configs of environmental differences only; the goal
  profile applies its own size and founder count, and a test asserts the
  checked-in recipes set neither.
- **Orchards in grassland** (food differentiation, `world_seed` 1104): type 0
  Grass is the founders' food, diffuse across 65% of the world at density
  0.4, 4 energy per unit, growth 0.05 on a background fertility of 0.15 (very
  slow) except in sixty meadows of radius 40–90 (fertility up to 2.0); type 1
  Fruit is rich (15 per unit), placed only in twenty-four orchards of radius
  50–110 (`initial_fertility_only`), fast-regrowing there, and edible only by
  a lineage that changes its Eat parameter. A literal diffuse-only grass went
  extinct by tick 400 in screening; the reversed assignment (founders on the
  rich patches) sat at the 100,000 cap for 1,000 ticks.
- **Canyon country** (barrier topology, `world_seed` 2204): one whole-world
  thresholded fBm field (frequency 0.005, threshold 0.02: about 47% barrier,
  98% of passable cells in one component) plus a 3% rubble field; production
  one-food defaults except fertility, which is forty-five valley meadows of
  radius 30–70 over a 0.1 background.
- **Confluence** (composite, `world_seed` 3304): Bounded edges; a canyon
  massif in the north-east (one fBm field at four nested thresholds stepping
  from 0.02 to 0.32 so rock density fades), an archipelago field in the
  south-west (0.0 to 0.3), a jagged ridge line system, a boulder field, both
  foods from Orchards with grass at density 0.5 and 5 per unit (the poor
  grass does not persist among barriers), sixty meadows, eighteen orchards,
  and a low-frequency fertility gradient; about 77% passable, 99.5% in one
  component. Regions overlap rather than tile.
- Balance targets, measured before storing: no extinction, no case pinned at
  the population cap past the boom, plateau populations in the low thousands
  to low tens of thousands, and the complete `make bench PROFILE=goal` under
  600 s on the recording host (target 480 s).

## Implementation Tasks

- [x] Food substrate, `FbmThreshold`, connectivity, per-case goal profile,
      frontend and reference carry-through (first pass; reviewed here).
- [ ] `v3-cli world inspect --config <recipe> --seed <n> [--png <path>]`:
      prints connectivity and per-type habitat, food-cell, and standing-crop
      readings as JSON and writes a downsampled PNG preview (barriers,
      per-type fertility, tick-zero food). Replaces the ignored
      `inspect_saved_world_layouts` test.
- [ ] `FbmThreshold` samples the noise field at world coordinates
      (`bounds.x + x`, `bounds.y + y`) instead of bounds-local ones, so a
      bounded layer equals the matching sub-rectangle of the whole-world
      layer at the same seed; the runtime pattern endpoint follows.
- [ ] Redesign the three recipes under `experiments/worlds/` with pinned
      `world_seed`, regenerate `previews/*.png` with the command above, and
      rewrite `experiments/worlds/README.md` (design intent, applied
      readings, how to inspect and run).
- [ ] Per-world tracking in the report: persistence samples and each
      `GoalCaseObservation` carry cumulative typed-eat counts per type (from
      applied successful Eat actions), per-type standing food density (from
      the applied growth summary), move attempts, moves blocked by barrier,
      and avoidable blocked moves by barrier-reader state.
- [ ] World-set comparisons: profile identity for `goal-worlds-v1` ignores
      per-case digests; each reference comparison gains a per-case block (by
      case name) with `inputs_changed` and both digests, deltas for
      persistence, per-case work counters, lineage, memory, drift,
      neighborhood, structure, typed eats, and blocked moves. A digest change
      is labeled, never an error, and never a claim of unchanged inputs.
      Missing cases in a reference are recorded absent.
- [ ] Dashboard: the Goal worlds tab shows, per world, series over closure
      order for the readings above, population-trajectory overlays by
      closure, and markers where inputs changed; `docs/progress.md` gains a
      world-set table; `docs/reference/v3-cli-contract-spec.md` records the
      command, the sample fields, and the comparison rule.
- [ ] Store gate and goal reports, list the goal report in the
      `goal_worlds` series, and record the readings below.

## Verification

- [x] `world inspect` tests: JSON readings match `passable_connectivity` and
      direct grid counts on a small recipe; PNG decodes to the downsampled
      size; a missing recipe or seed is a validation error.
- [x] Recipe tests: all four load; the three goal recipes pin `world_seed`
      and set neither size nor population; Orchards' fruit is strictly richer,
      patchier, and faster-regrowing than grass and grass has positive
      fertility on every passable cell; Canyon's largest component holds at
      least 95% of passable cells and at least 15% of cells are barriers;
      Confluence has both foods, Bounded edges, and terrain.
- [x] Telemetry tests (TDD): typed-eat counts rise only on successful typed
      eats; standing density equals the growth summary; blocked-move fields
      equal the stats maps; samples carry the fields at the 100-tick cadence;
      property test that per-case counters sum to the profile totals.
- [x] Comparison tests: a reference with changed case digests compares with
      `inputs_changed: true` and correct deltas; a reference with a different
      case list records the absent case; gate and single-config behavior are
      unchanged (existing tests).
- [ ] Dashboard checked in a browser against the stored report; `make
      check` (records the tested commit) and `make check-docs` at closure.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass:
      summary line, output path, and every survivor resolved.
- [ ] Reports: `docs/progress/features/t12-f04-baseline-world-set.json`
      (gate) and `...-goal.json` (goal, replacing the first pass's unbalanced
      reading, which is preserved as
      `...-goal-first-pass.json` and stays out of the series index).
- [x] Second goal run: not applicable (workflow decision 2026-09-05).

### Implementation verification record

Implementer pass, 2026-09-09, worktree `.claude/worktrees/t12-f04` (branch
`worktree-t12-f04`) on base `041c7697`. Recipe files, `experiments/worlds/`
previews and README, `docs/progress.md` prose and the sections above are the
orchestrator's; everything below is what the implementer ran and what it read.

**Commands and results** (all from the worktree root; every count below is
from the final sweep, after the mutation remediation added its tests):

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first; the action phase gained a typed-eat counter) | ok, 24 passed |
| `cargo test -p v3-core --test baseline_worlds` | ok, 18 passed, 1 ignored (the pre-existing food-choice diagnostic) |
| `cargo test -p v3-core --lib simulation::tick::tests::actions` | ok, 24 passed (of the crate's 1,265 unit tests) |
| `cargo test -p v3-cli` | ok, 66 lib + 11 `main` + 18 `tests/bench.rs` + 11 `tests/cli.rs` passed |
| `cargo fmt --all` | applied |
| `make check` | passed end to end (exit 0), including the new `rust-test-baseline-worlds` target |
| `make roadmap-check` | validation passed |
| `make check-docs` | quality, bench-wait, mutation-wrapper, dev-stack and skill-cache checks passed |
| `MUTANTS_ITERATE=0 make rust-mutants` | see "Mutation record" below |

`legacy_default_short_run_identity` still reads the pre-feature fingerprint
`13138541837675773035`, so the telemetry counters changed no simulation state.

The recipe tests were written against the first pass's placeholder recipes and
initially failed on two assertions; the orchestrator's redesigned recipes landed
mid-pass and both now pass. One assertion was corrected rather than the recipe:
Orchards' fruit is patchy through `initial_fertility_only` over a blob fertility
layer, not through a lower `initial_coverage`, so the test measures patchiness
on the applied map (fruit's fertile-cell count against grass's, which covers
every passable cell) instead of on a config field. `experiments/worlds/previews/`
still holds the first pass's images; regenerating them with `world inspect` is
the orchestrator's step, and the README's applied readings already match what
the command prints.

**Build configuration change.** `crates/v3-core/tests/baseline_worlds.rs` was
run by no `make` target: `make check` passed while two of the recipe tests this
feature adds were red. `rust-test-baseline-worlds` now runs it inside
`rust-test-all`. Verification that never runs is not verification, and the file
also holds the `legacy_default_short_run_identity` fingerprint pin.

**Dependency.** `png = "=0.18.1"` on `v3-cli`, pinned exactly, for the preview
encoder and its decode in the unit test. 0.18 adds only `fdeflate` on top of the
already-locked `flate2`/`miniz_oxide`/`crc32fast`/`bitflags` 2.x; 0.17 would
have pulled a second, 1.x `bitflags`. `make check`'s dependency audit passes.

**Recorded decisions and deviations.**

- Exit codes: a missing, unreadable or resolver-rejected recipe is a validation
  error that exits `1` and writes nothing. An argument that fails to parse at
  all (a non-numeric or omitted `--seed`) is clap's usage error and exits `2`,
  which collides with this repository's "runtime error" code. Keeping clap's
  declarative parsing was preferred to hand-parsing the seed; the collision is
  now written down in `docs/reference/v3-cli-contract-spec.md` Section 3.
- `world inspect` resolves a recipe with `resolve_config` alone (deep merge,
  ramp validation, `normalize`, `apply_startup_overrides`) rather than the goal
  profile's `build_config`. `build_config` additionally forces 1600x1600 and
  10,000 founders; the checked-in recipes set none of those and a test asserts
  they never will, so the two resolutions are byte-identical for them and the
  plain one also inspects a small recipe at its own size.
- World-set comparison identity strips the whole `cases` block, not only
  `config_digest`. A recipe edit therefore also relabels `recipe_path` and
  `food_type_count` per case instead of erroring, and a case name the reference
  never ran is recorded `absent_in_reference` rather than failing identity.
  Everything else in the profile block still has to match exactly.
- `inputs_changed` is true when the reference case's digest **or** its run seed
  differs. Swapping seeds between two cases keeps the profile's seed list and
  every digest identical, and would otherwise read as unchanged inputs.
- The preview max-pools barriers exactly as specified, so at Canyon's
  downsample factor of 4 a fine rubble field reads as more walled than its
  52.7% passable fraction: any block holding one barrier is drawn gray. The
  JSON `passable_connectivity` is the quantitative reading; the PNG is a shape
  preview.

**`world inspect` applied against the stored recipes** (release build,
`--seed` as the goal profile assigns it):

- Orchards/11: 1600x1600 Wrap, `world_seed` 1104 pinned, connectivity
  2,560,000/2,560,000 passable in one component, 10,000 founders placed. Grass
  fertile on every cell (mean 0.581150), 1,664,000 food cells, standing energy
  2,662,400.04. Fruit fertile on 468,253 cells (0.182911 of the world, mean
  1.517091), 398,015 food cells, standing energy 5,970,225.00.
- Canyon/22: `world_seed` 2204 pinned, 1,349,524 passable cells (0.527158),
  largest component 0.980559 of passable, one food type, 728,743 food cells.

**Dashboard browser check.** `docs/progress` served with
`python3 -m http.server 8731`; `goal_worlds.epoch_baseline` was pointed at
`t12-f04-baseline-world-set-goal-first-pass.json` for the check and then
restored (`git diff docs/progress/benchmark-series.json` is empty). The Goal
worlds tab rendered three world groups, 63 cards, 12 headline tiles and 9
"Not measured in these reports" chart placeholders with an empty console: the
first-pass report predates every tracking field, so those charts and tiles read
as unmeasured rather than zero. Historical goal (30 cards) and gate (28 cards)
tabs were unchanged. With the epoch pointed back at the not-yet-stored
`...-goal.json`, `loadAll` now skips the missing report instead of failing the
whole page: the status line reads "0 goal-worlds-v1 closures ... 1 listed
report not stored yet" and the empty state renders without the error notice.

**Simplification pass.** Run on the feature diff against its merge base with
`main` (single-pass inline review; the Agent fan-out is unavailable in this
context). Applied: `six`, `fraction_or_undefined` and `UNDEFINED` moved from
`bench.rs` to `crates/v3-cli/src/lib.rs` as one shared set with a new
`mean_or_undefined`, and `inspect.rs`'s copies deleted; `main.rs`'s config
reader extracted to `read_recipe` and reused by `load_config`; the per-seed
totals accumulation in `run_deterministic` extracted to a pure
`accumulate_totals` (which is what the new property test exercises); the
per-case work-counter readings zipped against the existing `COUNTER_NAMES`
so they cannot drift from the profile counter list; the derived tracking
fractions moved into one `WorldTracking::fractions` so a total and its fraction
cannot disagree. Skipped: `VALUE_ONLY_CASE_READINGS` stays a one-element array
rather than a bare constant, because it names a category the next non-rate
reading joins.

**Mutation record.** Fresh `MUTANTS_ITERATE=0 make rust-mutants` against the
merge base `c95457d9`, after the simplify pass and after the remediation
described below:

```
282 mutants tested in 18m: 239 caught, 41 unviable, 2 timeouts
```

Output path: `~/.local/share/petri-tools/mutants/t12-f04/mutants.out`
(`run-mode.txt` records `fresh`). Survivor list, complete:

| Survivor | Resolution |
| --- | --- |
| `crates/v3-core/src/kernel/world.rs:57:48: replace && with \|\| in WorldState::passable_connectivity` | **deferred** — the mutated visited guard never terminates, so it times out instead of failing; see "Notes for AI Agents". |
| `crates/v3-core/src/kernel/world.rs:57:32: delete ! in WorldState::passable_connectivity` | **deferred** — same guard, same reason. |

Nothing was missed by every test. The first fresh run of this pass
(`279 mutants tested in 19m: 27 missed, 209 caught, 41 unviable, 2 timeouts`)
left 27 missed mutants, all in this feature's new code and all **killed** by
strengthening tests, never by editing the code under test:

- `case_readings` took the first matching row rather than the case's own
  (`seed ==`, `depth == 2_000`, the `creature_ticks > 0` guard and its
  division) and `parse_reading` was never exercised on a parsed string.
  `case_readings_follow_the_case_seed_and_observation` now stamps every
  per-seed, lineage, memory, drift and fraction row with a value derived from
  its own seed and asserts each compared reading, including that a case with no
  creature-ticks reads unmeasured rather than dividing by zero;
  `an_undefined_reading_parses_as_unmeasured` pins the parse.
- `render_preview`'s block offsets, block-cell count and pooling accumulator
  were only exercised at the origin block and at downsample factor 1.
  `a_food_block_is_the_mean_over_its_own_cells_normalized_by_the_maximum` and
  the rewritten `a_downsampled_block_max_pools_barriers_and_mean_pools_food`
  assert exact colors for named non-origin blocks at factor 4, over a world
  whose per-cell maximum is not 1.0.
- `blend`'s hue accumulation survived because the expectations were computed by
  `blend` itself. `blend_adds_each_type_hue_over_bare_ground_and_barriers_win`
  pins the literal RGB values instead.
- The preview's normalization guard was unreachable from `render_preview`'s
  tests. It is now the named `intensity` function with its own unit test,
  including the nonpositive-ceiling case.
- `read_world`'s strict `> 0.0` fertility test needed ground at exactly zero
  fertility: `fertile_cells_exclude_passable_cells_whose_fertility_is_exactly_zero`
  builds a blob layer over a zero floor and asserts the fertile count is
  strictly below the passable count.

The intermediate `MUTANTS_ITERATE=1` run between those two
(`29 mutants tested in 5m: 3 missed, 24 caught, 2 timeouts`) is recorded as
remediation feedback only, not as closure evidence.

## Performance and Goal Impact

Natural analogs: food profitability against return rate (orchard fruit in
grass), landscape barriers (canyon country), and a patchy composite habitat;
all reach creatures through what they can eat, where they can walk, and what
their existing sensors report.

Predeclared cost: the gate profile is unchanged and must read within
threshold against T11.F18 and the pinned epoch. The goal profile is a new
series; its budget is the 600 s wall-clock bound above, and the new
telemetry adds one counter increment per applied eat and per blocked move
plus a per-type density read already produced by the growth summary.

Readings to record here at closure, per case: final, minimum, peak, and
plateau population; births; mean energy; passable fraction and largest
component; typed eats per type; blocked-move fraction and avoidable fraction
by reader state; lineage entropy and clades; memory sensitivity; drift
changed/all at 1,000 and 2,000 against the 0.0015 and 0.008 floors; founder
and evolved neighborhood fractions; structure median; wall-clock per case and
for the whole command.

## Success Criteria

- [ ] `make bench PROFILE=goal` runs Orchards, Canyon, and Confluence once
      each from fixed maps, all three persist to tick 2,000 without pinning
      at the cap, and the command completes under 600 s on the recording host.
- [ ] A later closure's goal report can be compared per world against this
      one, with changed recipe inputs labeled, and the dashboard shows each
      world's readings over closure order.
- [ ] Reviewed diff, mutation record, stored reports, and closure checks
      pass; the drift-floor reading is recorded with the user's decision.

## Notes for AI Agents

- Second pass, 2026-09-09: Fable 5.1 orchestrator took over the Codex
  branch (`codex/t12-f04`, rebased onto `main` at `c95457d9` in
  `.claude/worktrees/t12-f04`, branch `worktree-t12-f04`). Recipe design and
  balancing were done by the orchestrator (config data, tuned by screening
  runs), a recorded deviation from "write no feature code yourself"; all Rust,
  frontend, and dashboard code went through the implementer.
- First-pass review findings (Fable, 2026-09-09): (1) Orchards and
  Confluence used grass fertility blobs covering nearly the whole world, so
  regrowth held the population at the 100,000 cap for 1,400 ticks and the run
  took 1,047 s; (2) Canyon was a whole-world perfect maze (a spanning tree)
  and collapsed to 7 creatures; (3) `compare_against_path` errors on any
  profile difference and `bench` exits before writing the report, so the
  first recipe edit required by the standard-baseline contract would discard
  a ten-minute run; (4) maps derived from the run seed although the goal asks
  for saved baseline maps; (5) the goal profile silently overrides a recipe's
  size and founders; (6) the first report lacked per-case structure
  distributions (already remediated); (7) previews came from an ignored test
  writing to `/private/tmp`. Items 1, 2, 4, 5, 7 are fixed by the tasks above;
  3 by the comparison task; 6 stays fixed.
- First-pass verification worth keeping: viability gate ran first after the
  food edits (24 tests); the pre-feature short-run fingerprint
  `13138541837675773035` is pinned by `legacy_default_short_run_identity`;
  the gate report at `5f87c8f4` read every deterministic field equal to
  T11.F18; three fresh mutation passes ended at `112 mutants tested: 84
  caught, 26 unviable, 2 timeouts`, the two timeouts being the visited-guard
  mutations in `WorldState::passable_connectivity` (`world.rs:57`), deferred
  as P2 because the mutated traversal never terminates. Its first goal report
  is kept as `t12-f04-baseline-world-set-goal-first-pass.json` for the
  record and is not a series member.
- Drift floor: the depth-2,000 changed/all-birth floor is a T11.F17 track
  floor (0.008, strict not-below). The reading here is the substrate's, not
  the worlds': 10/2,000 with one food type (identical to T11.F18, closed on
  the user's written exception on 2026-09-08) and 12/2,000 with two. Closure
  needs the user's decision to extend that exception or to hold this feature
  open until a T11 repair lands; the orchestrator does not grant it.
- Deferred P2 (mutation): the two `passable_connectivity` visited-guard
  mutants time out rather than fail; killing them needs a watchdog test,
  which is not worth adding for score. Confirmed still the only survivors by
  the implementer's fresh run on 2026-09-09 (`world.rs:57:48` `&&` to `||`,
  `world.rs:57:32` deleted `!`); the mutated traversal revisits cells forever,
  so the mutant hangs instead of producing a wrong reading, and every other
  mutant in the feature diff is caught.
- Deferred P3 (test wiring, found 2026-09-09): `crates/v3-core/tests/
  mutational_neighborhood.rs` is run by no `make` target, the same gap this
  pass closed for `baseline_worlds.rs`. Left alone here because it belongs to
  T11, not to this feature; a T11 feature that touches it should add
  `rust-test-mutational-neighborhood` to `rust-test-all`.
