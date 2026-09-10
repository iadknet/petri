# T12.F04 — baseline world set: measured readings

Measured evidence relocated on 2026-09-09 from
[`docs/specs/roadmap/t12-f04-baseline-world-set.md`](../../specs/roadmap/t12-f04-baseline-world-set.md),
unchanged in content. The spec keeps the predeclaration, the verdict, the
final mutation survivor record, and every user decision. Machine-written
reports: [gate](../features/t12-f04-baseline-world-set.json), [goal](../features/t12-f04-baseline-world-set-goal.json).

This feature ran four passes; each pass's verification record and mutation
run is below in order.

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

#### Follow-up pass

Second implementer pass, 2026-09-09, same worktree, on base `f0b9ec58` (a fresh
agent; the pass above could not be resumed). Two items: world-coordinate
`FbmThreshold` sampling and preview readability.

**`FbmThreshold` samples the noise field at world coordinates.** `fbm.rs` now
exposes `FbmField`, a seeded field with `new(octaves, frequency, lacunarity,
persistence, seed)` (the same XOR-folded u64 seed and builder) and
`sample(world_x, world_y) -> f32` clamped to [-1, 1]. `generate_fbm` is a loop
over `sample` and produces the identical grid, so fertility layers are
unchanged. `generate_pattern` builds the field from its one `rng.gen::<u64>()`
draw and tests `field.sample(bounds.x + x, bounds.y + y) > threshold` directly,
dropping the intermediate bounds-local `Grid` allocation; the strict `>`,
row-major output order and the `u16::MAX` coordinate clipping are unchanged, and
the runtime pattern endpoint follows through the same function.

Tests, red before the change:

- `fbm_bounded_layer_equals_the_whole_world_layer_inside_its_bounds` (proptest,
  `baseline_worlds.rs`) draws a seed, a threshold and a rectangle over a 32-cell
  world, clips it the way `seed_simulation` clips a layer's bounds, and asserts
  the bounded layer's point set equals the whole-world layer's cut to that
  rectangle. It failed on the old code at the first case
  (`x = 0, y = 1, width = 1, height = 1`); the shrink it saved is left in the
  worktree as `crates/v3-core/tests/baseline_worlds.proptest-regressions` for
  the orchestrator to commit.
- `fbm_threshold_is_strict_at_zero_and_clips_coordinate_edges` no longer claims
  the `u16::MAX` corner samples zero. Perlin fBm is exactly zero at world
  (0, 0), so a layer at the origin excludes (0, 0) at threshold 0.0 and includes
  it at -1.0 (and the -1.0 set is a superset), which is the same strictness
  witness under the new semantics. The two coordinate-clipping witnesses at the
  `u16::MAX` corner (1 point, then 6) are unchanged.
- `fbm_unique_in_bounds_and_threshold_monotone` still passes unchanged;
  `reproducibility.rs` (which carries an fBm terrain layer) and `terrain.rs`
  pass; `legacy_default_short_run_identity` still reads
  `13138541837675773035`.

`docs/reference/v3-world-grid-spec.md` replaces "translated to the bounds
origin" with world-coordinate sampling and the nested-layer consequence.

**Which stored recipes move.** Measured with the release `world inspect` after
the change: Orchards/11 and Canyon/22 print readings byte-identical to the ones
recorded above (Canyon 1,349,524 passable, 0.527158, largest component
0.980559, 728,743 food cells; Orchards grass mean 0.581150 / 1,664,000 food
cells / 2,662,400.039673 standing energy, fruit 468,253 fertile / 0.182911 /
1.517091 / 398,015 / 5,970,225.000000). Orchards has no terrain and Canyon's two
layers are whole-world (`bounds: null`), where local and world coordinates
coincide. Confluence's eight nested `FbmThreshold` regions are bounded, so its
map does move: its outermost north-east layer (bounds 1000,0 600x620, seed 3301,
threshold 0.02) goes from 190,725 barrier cells under the old semantics to
183,194 under the new, sharing only 88,138 — the nested thresholds now grade one
field instead of reading four copies of the same local origin. Confluence/33
after the change:

- 1600x1600 Bounded, `world_seed` 3304 pinned, 1,961,687 passable
  (0.766283984375), largest component 0.9953555281754939 of passable, 10,000
  founders placed. Grass fertile on every passable cell (mean 0.545019),
  1,275,097 food cells, standing energy 3,187,742.50. Fruit fertile on 208,503
  cells (0.081446 of the world, mean 1.504632), 177,228 food cells, standing
  energy 2,658,420.00.

These match the "about 77% passable, 99.5% in one component" the recipe was
designed for. `experiments/worlds/previews/confluence.png` is stale against this
map; regenerating it is the orchestrator's step.

**Preview readability.** `render_preview` mean-pools the barrier fraction per
block instead of max-pooling a boolean, and `blend` takes that fraction and
fades the habitat or food color to the barrier gray in proportion to it: a block
with no barrier keeps its color exactly, an entirely walled block is solid gray,
and anything between lies strictly between. Canyon's 52.7%-passable map no
longer reads as mostly wall at its downsample factor of 4. JSON readings are
untouched. `a_downsampled_block_grays_in_proportion_to_its_barrier_fraction`
asserts the three cases on named non-origin blocks at factor 4 in both panels,
and `blend`'s literal color pins gain `blend(1.0, ..)`, `blend(0.5, ..)`,
`blend(0.25, ..)` and the above-one clamp. `docs/reference/v3-cli-contract-spec.md`
records the new pooling rule.

**Supersedes.** The "Recorded decisions and deviations" bullet above stating
that the preview max-pools barriers, so Canyon reads as more walled than its
passable fraction, describes the first pass and no longer holds; the JSON
`passable_connectivity` remains the quantitative reading either way.

**Commands and results** (worktree root, `PATH` prefixed with the petri-tools
and aqua bin directories; every entry rerun after the simplification pass):

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first) | ok, 24 passed |
| `cargo test -p v3-core --test baseline_worlds --test reproducibility --test terrain` | ok, 19 passed / 1 ignored, 3 passed, 10 passed |
| `cargo test -p v3-core --lib fertility` / `--lib patterns` | ok, 32 passed / ok, 45 passed |
| `cargo test -p v3-cli` | ok, 66 lib + 11 `main` + 18 `tests/bench.rs` + 11 `tests/cli.rs` passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all` | applied |
| `make check` | exit 0 |
| `make check-docs` | exit 0 |
| `make roadmap-check` | validation passed |
| `MUTANTS_ITERATE=0 make rust-mutants` | see below |

**Simplification pass.** Run on this pass's diff against `f0b9ec58`
(single-pass inline review; the Agent fan-out is unavailable in this context).
Applied: `blend`'s two-stage clamp-then-mix loop collapsed to one
`std::array::from_fn` over the channels, dropping a shadowed intermediate array;
the barrier accumulator written as a plain `if` rather than
`f32::from(u8::from(bool))`; the duplicated point-set collection in
`fbm_threshold_is_strict_at_zero_and_clips_coordinate_edges` extracted to one
closure taking the params; the `WORLD` const moved above the `proptest!` block
that uses it. Skipped: converting `generate_pattern`'s fBm loop back to an
iterator chain — the nested loop already allocates only the output `Vec` and
reads more plainly than a `flat_map` with a captured field.

**Preview viewed.** `world inspect --config experiments/worlds/canyon-country.json
--seed 22 --png` was written to a scratch path (not to
`experiments/worlds/previews/`) and looked at: the habitat panel reads as roughly
half gray rock against dark open ground with the meadow blobs picked out, and the
3% rubble field as a faint speckle rather than a wash of gray — the readable map
the mean-pooled fraction was meant to produce, consistent with the 0.527158
passable fraction.

**Mutation record (follow-up pass).** Fresh `MUTANTS_ITERATE=0 make rust-mutants`
against the merge base `c95457d9`, run after the simplification pass with nothing
else on the machine:

```
309 mutants tested in 26m: 256 caught, 51 unviable, 2 timeouts
```

Output path: `~/.local/share/petri-tools/mutants/t12-f04/mutants.out`
(`run-mode.txt` records `fresh`). Survivor list, complete — no mutant was missed
by every test, and the two survivors are the same pre-existing pair the pass
above deferred:

| Survivor | Resolution |
| --- | --- |
| `crates/v3-core/src/kernel/world.rs:57:48: replace && with \|\| in WorldState::passable_connectivity` | **deferred** — timed out at 120 s rather than failing; the mutated visited guard never terminates. See "Notes for AI Agents". |
| `crates/v3-core/src/kernel/world.rs:57:32: delete ! in WorldState::passable_connectivity` | **deferred** — same guard, same reason. |

Every mutant in this pass's own code — `FbmField::new`, `FbmField::sample`,
`generate_fbm`, `generate_pattern`'s fBm arm, `blend` and `render_preview` — was
caught, with no test added after the run.

#### Remediation pass

Third implementer pass, 2026-09-09, same worktree, on base `948d0bc3` (a fresh
agent; earlier passes cannot be resumed here). Three review findings: a true
barrier-awareness reading, an exact barrier count in `reproducibility.rs`, and
seeds carried in the goal recipe list. Nothing outside `crates/`,
`docs/reference/v3-cli-contract-spec.md`, `docs/progress/index.html`, and this
subsection was touched.

**P1 — the barrier-awareness reading now has a per-state denominator.**
`avoidable_blocked_move_fraction_by_reader_state` divided avoidable blocks *of
any cause* by every move *every* genome attempted, so it read 20.75% for
Orchards' no-reader state in a world with no barriers at all and was never a
per-state rate. `WorldTracking` now also carries
`move_attempts_with_barrier_neighbor_by_reader_state` and
`moves_blocked_barrier_with_barrier_neighbor_by_reader_state`, read straight
from the `SimStats` maps `tick.rs` already increments, in the persistence
samples at the existing cadence and in each `GoalCaseObservation`. The derived
`barrier_blocked_fraction_by_reader_state[state]` is
`moves_blocked_barrier_with_barrier_neighbor[state]` over
`move_attempts_with_barrier_neighbor[state]` — that state's own moves made from
a cell with a neighboring barrier, the only moves at which reading the barrier
ring could change anything — and is `Undefined` when that denominator is zero,
so a barrier-free world reports no rate rather than a fabricated zero. The old
totals are kept and the old fraction is renamed
`avoidable_blocked_share_of_all_moves_by_reader_state`, with both denominators
written into the field docs and into
`docs/reference/v3-cli-contract-spec.md`. The per-case comparison follows all
four reader-state readings under the new names; the dashboard's "Blocked moves"
chart takes the two per-state barrier-block rates as its emphasized headline
series and names each series' denominator, and the "Goal worlds" identity table
gains the per-state attempts-beside-a-barrier row so an `Undefined` rate is
readable.

Consequence, recorded rather than hidden: the stored
`...-goal.json` predates every new field and used the old name for the
avoidable fraction, so on that report the two headline series and both
avoidable series read unmeasured, not zero. The orchestrator's re-measurement
fills them.

Tests. The first two below were written against the old struct and failed to
compile, which is the red this pass recorded; the other three were extended
after `observe` and `fractions` landed, to cover the new fields the whole-struct
and by-name assertions would otherwise have passed over:

- `tracking_fractions_divide_each_reading_by_its_own_denominator` uses distinct
  numerators and denominators per state (attempts 4/16, blocks 1/10) so a
  swapped state or a swapped numerator is visible, and asserts the
  zero-denominator and the barrier-free cases read `Undefined`.
- `observe_reads_each_barrier_counter_from_its_own_stats_map` stamps three
  `SimStats` maps with different values under both reader-state keys and
  asserts `observe` reads each field from its own map and key.
- `case_readings_follow_the_case_seed_and_observation` stamps all four
  reader-state fractions with seed-derived values and asserts each compared
  reading by name.
- `persistence_samples_carry_world_tracking_on_the_births_cadence` asserts the
  new counters are subsets of the totals they must be subsets of, on every
  sample.
- `tracking_fields_default_when_absent_and_survive_a_round_trip` keeps the
  pre-T12.F04 sample parsing and pins the new denominator's wire key. Every new
  field is `#[serde(default)]`; nothing in the report carries
  `deny_unknown_fields`, so a historical report still parses and the renamed key
  reads as unmeasured.

**P3 — the exact barrier count.** `reproducibility.rs`'s
`terrain_is_identical_across_independent_initialization_and_thread_counts` had
been weakened from `== 2` to `> 2` when the fBm layer landed. It now pins the
count the fixture actually produces, **117** (read from the failing assertion
after setting it to 0, then pinned).

**P3 — recipes carry their own seed.** `GOAL_RECIPES` is now
`[GoalRecipe; 3]` with `name`, `path`, `source`, and `seed`; `goal_case` and
`prepare_goal_case` take a `&GoalRecipe` and read the seed from it, and
`goal_profile_params().seeds` is derived from the recipes, so adding a world is
one entry. `goal_recipes_for` returns the recipe slice for `goal-worlds-v1` and
an empty slice for every other profile, and errors when a world-set profile's
seed list is not exactly the recipes' seeds in their order — a hard error, not a
silent drop or an index panic. It is called once at the top of
`run_deterministic`, which now returns `Result`, as do `build_report` and
`build_report_with_threads`; `main.rs` prints `error: {e}` and exits 1 on the
existing idiom, and tests `.expect("a valid profile")`. Case names, recipe paths, seeds, and order in
the report are unchanged — pinned by the `[11, 22, 33]` assertion and by the
existing `report.profile.cases` and replayed-run tests; no test pins report
bytes across this refactor.
`a_world_set_profile_must_name_the_seeds_its_recipes_carry` pins
`goal_profile_params().seeds == [11, 22, 33]` and asserts both a reordered and a
short seed list error, on a test-sized world set.

**Commands and results** (worktree root, `PATH` prefixed with the petri-tools
and aqua bin directories; every entry rerun after the mutation remediation):

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first) | ok, 24 passed |
| `cargo test -p v3-core --test reproducibility` | ok, 3 passed |
| `cargo test -p v3-core --test baseline_worlds` | ok, 19 passed, 1 ignored |
| `cargo test -p v3-cli` | ok, 68 lib + 11 `main` + 18 `tests/bench.rs` + 11 `tests/cli.rs` passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all` | applied |
| `make check` | exit 0 |
| `make roadmap-check` | validation passed |
| `MUTANTS_ITERATE=0 make rust-mutants` | see below |

**Simplification pass.** Run on this pass's diff against `948d0bc3`
(single-pass inline review; the Agent fan-out is unavailable in this context).
Applied: the seed-list error message uses the slice's own `{:?}` instead of a
hand-rolled `join_seeds` helper, which is deleted; `observe`'s three identical
two-key stats lookups collapse into one `by_reader_state` closure; the four
reader-state comparison readings collapse into one `by_reader_state_readings`
helper (which also brought `case_readings` back under the `too_many_lines`
clippy cap it had crossed); `profile_block`'s new parameter dropped an
unnecessary `'static` bound; `build_report_with_threads`'s shadowed
`deterministic` binding renamed to `profile_run`; and the test-sized world-set
profile extracted to `small_world_set_params`, reused by
`small_world_set_report`. Skipped: deriving `world_set` from
`!recipes.is_empty()` — `assemble_goal_indicators` has no recipes in scope, so
the two sites would have disagreed in form; both keep the direct name test.

**Mutation record (remediation pass).** The first fresh
`MUTANTS_ITERATE=0 make rust-mutants` of this pass returned
`325 mutants tested in 22m: 259 caught, 61 unviable, 5 timeouts`, and three of
those timeouts were in this pass's own code:

| First-run survivor | Resolution |
| --- | --- |
| `bench.rs:226:5: replace goal_recipes_for -> Result<&'static[GoalRecipe], String> with Ok(Vec::leak(Vec::new()))` | **killed** — a test gap, not a code fault: `a_world_set_profile_must_name_the_seeds_its_recipes_carry` built its bad profiles from the real `goal_profile_params()`, so a mutant that skipped the guard started a 1600², 2,000-tick run and hit the 120 s test timeout instead of failing. The test now uses `small_world_set_params()`, so the unguarded run finishes and the assertion fails. |
| `bench.rs:226:20: replace != with == in goal_recipes_for` | **killed** — same test, same cause. |
| `bench.rs:230:21: replace != with == in goal_recipes_for` | **killed** — same test, same cause. |

Fresh rerun after that test change, against the merge base `c95457d9`, with
nothing else on the machine:

```
325 mutants tested in 21m: 262 caught, 61 unviable, 2 timeouts
```

Output path: `~/.local/share/petri-tools/mutants/t12-f04/mutants.out`
(`run-mode.txt` records `fresh`). Survivor list, complete — nothing missed by
every test, and the two survivors are the same pre-existing pair both earlier
passes deferred:

| Survivor | Resolution |
| --- | --- |
| `crates/v3-core/src/kernel/world.rs:57:48: replace && with \|\| in WorldState::passable_connectivity` | **deferred** — timed out at 120 s rather than failing; the mutated visited guard never terminates. See "Notes for AI Agents". |
| `crates/v3-core/src/kernel/world.rs:57:32: delete ! in WorldState::passable_connectivity` | **deferred** — same guard, same reason. |

No production code was edited to kill a mutant, and no `#[mutants::skip]` or
`exclude_re` entry was added.

**Dashboard checked in a browser.** `docs/progress` served with
`python3 -m http.server`; the Goal worlds tab rendered every world group with an
empty console. The "Blocked moves" card shows all five series with their
denominators in the legend, and on the stored report only
`blocked by barrier, of all moves` has data — the four reader-state series and
the new headline tile read unmeasured, which is the correct reading for a report
recorded before the fields existed.

**Not run.** `make bench` (either profile): the orchestrator re-measures after
this pass, so no report under `docs/progress/features/` was regenerated or
edited here.

#### Fold-in pass

Fourth implementer pass, 2026-09-09, same worktree, on base `3ba4a782` (a fresh
agent). One user-requested fold-in: carry every blocked move by its cause, not
only the barrier ones. Nothing outside `crates/v3-cli/src/bench.rs`,
`docs/progress/index.html`, `docs/reference/v3-cli-contract-spec.md`, and this
subsection was touched; `experiments/worlds/`, the roadmaps, and
`docs/progress.md` were not.

**What landed.** `SimStats.move_actions_blocked_total_by_cause` already counted
every blocked move under `MoveBlockedCause::{Barrier, Occupied, OutOfBounds}`,
but the report kept only the barrier entry, so a world that traded barrier
blocks for crowding read as if nothing had changed. `WorldTracking` now carries
`moves_blocked_total_by_cause: {barrier, occupied, out_of_bounds}`, read once
per cause from that map in `observe`, and `moves_blocked_barrier_total` is
assigned the same `barrier` value rather than looked up a second time — one
measurement, two keys on the wire, kept because reports stored before this pass
carry only the old key. Because `WorldTracking` is the one struct both the
persistence samples (existing `births_total` cadence) and each
`GoalCaseObservation` embed, both surfaces gained the totals together.

The field is `Option<MovesBlockedByCause>`, not a defaulted struct, on purpose:
the world-set per-case comparison follows the three totals as raw readings
(`moves_blocked_barrier_total`, `moves_blocked_occupied_total`,
`moves_blocked_out_of_bounds_total`), and the reference report it compares
against — the stored `...-goal.json` — predates the field. A defaulted struct
would have made that reference read a measured zero and reported the first real
measurement as an infinite change; `None` reads unmeasured, the same discipline
the remediation pass applied to the reader-state rates. `serde(default)` keeps
the historical parse working, and nothing in the report uses
`deny_unknown_fields`.

The dashboard's "Blocked moves" chart takes the two new causes as secondary
series, each named for its cause and its denominator ("blocked by an occupied
cell, of all moves", "blocked at the world edge, of all moves"), divided by
`moves_attempted_total` in the same JS-side way the per-creature-tick charts
divide; the emphasized headline pair is untouched. The reference spec's
persistence-sample field list names the new object, its source stats map, that
its `barrier` entry is the same measurement as `moves_blocked_barrier_total`,
and that it is absent rather than zeroed in an older report.

Tests. The red was a compile failure: the new
`observe_reads_each_blocked_move_cause_from_its_own_stats_key` referenced a
field and a type that did not exist yet (`no field
moves_blocked_total_by_cause on type WorldTracking`, `failed to resolve: use of
undeclared type MovesBlockedByCause`), reported by the compile-check hook.

- `observe_reads_each_blocked_move_cause_from_its_own_stats_key` stamps three
  distinct counts under the three cause keys and asserts each lands in its own
  field and that `moves_blocked_barrier_total` is the map's barrier entry; it
  also asserts an unstamped run reads `Some(default)` — zero, not absent.
- `persistence_samples_carry_world_tracking_on_the_births_cadence` now asserts,
  on every sample, that the by-cause barrier entry equals
  `moves_blocked_barrier_total` and that the three causes together are a subset
  of `moves_attempted_total`.
- `tracking_fields_default_when_absent_and_survive_a_round_trip` pins the wire
  key `moves_blocked_total_by_cause.occupied`; the pre-T12.F04 parse assertion
  is unchanged and still passes.
- `case_readings_follow_the_case_seed_and_observation` stamps three different
  multiples of each case's seed (`seed`, `seed * 3`, `seed * 5`) so a swapped
  cause is visible, asserts all three readings by name, and leaves the first
  case's field `None` to assert an unmeasured reading rather than zero.
- `tracking_fractions_divide_each_reading_by_its_own_denominator` and
  `world_set_case_tracking_matches_a_replayed_run` needed no new assertions:
  the first is a whole-struct literal, the second a whole-struct equality
  against a replayed `observe`.

**Commands and results** (worktree root, `PATH` prefixed with the petri-tools
and aqua bin directories):

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first) | ok, 24 passed |
| `cargo test -p v3-cli` | ok, 69 lib + 11 `main` + 18 `tests/bench.rs` + 11 `tests/cli.rs` passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all` | applied |
| `make check` | exit 0 |
| `make roadmap-check` | validation passed |
| `MUTANTS_ITERATE=0 make rust-mutants` | see below |

**Simplification pass.** Run on this pass's diff against `3ba4a782`
(single-pass inline review; the Agent fan-out is unavailable in this context).
Applied: `by_cause_readings` dropped its function-pointer indirection for a
three-entry array mapped once into `(String, Option<f64>)`, and the new
`observe` test reuses its one simulation for both the unstamped and the stamped
assertion instead of building a second one. Skipped: a generic `ByCause<T>`
mirroring `ByReaderState<T>` (only `u64` is ever carried, so the type parameter
would be unused ceremony) and folding the three
`get().copied().unwrap_or_default()` lookups into a shared free function
(it would rewrite two adjacent pre-existing lookups for no behavior change).

**Dashboard checked in a browser.** `docs/progress` served with
`python3 -m http.server`, driven headless. The Goal worlds tab rendered with an
empty console, the "Blocked moves" card shows all seven series with their
denominators in the legend, and its table view reads `—` for both new causes on
the stored pre-fold-in report — unmeasured, not a fabricated zero.

**Not run.** `make bench` (either profile).
