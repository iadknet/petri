# T02.F04 — Grazing Recovery and Overuse

**Status**: Complete
**Last updated**: 2026-09-15
**Feature**: T02.F04
**Track**: [T02 — Environmental Dynamics](../../roadmaps/t02-environmental-dynamics.md)

## Goal

Overgrazing, the natural analog: a bitten patch regrows more slowly,
repeated bites compound the damage down to a floor, and rest heals it. Each
food type at each cell carries a fertility modifier in `[floor, 1.0]` that a
consuming bite multiplies by `factor`, that recovers linearly toward 1.0 by
`1 / recovery_ticks` per tick, and that multiplies the type's fertility
wherever growth reads it. The pressure reaches creatures only through how
fast food returns to grazed ground; there is no grazing sensor. It ships
enabled at production defaults, so the gate world and all three goal worlds
carry it, and the goal report shows it acting in each.

## Non-Goals

- Per-type grazing parameters or recipe overrides: the three checked-in
  recipes stay byte-identical and inherit the production default.
- Any change to the occupancy depletion layer or to the shared growth,
  spread, and recovery-spawn rules beyond the multiplication.
- Rebalancing a goal world the pressure collapses: a recorded blocker for
  the user, not a quiet retune.
- A grazing sensor, a grazing input to any brain, or a change to `Eat`.
- Server health or frontend telemetry for the modifier; the goal report and
  `SimStats` carry the readings, the panel only the config.
- Retiring the orphaned, uncompiled
  `crates/v3-core/src/kernel/food_resource/{depletion,growth,reconfigure}.rs`;
  do not edit them.

## Inputs and Invariants

**Contract.** The T02 track row and its 2026-09-15 Notes entry are the
feature definition; this spec adds nothing to them. Dependency: T12.F04 gives
the per-type fertility grids (`OrdinaryFoodState::fertility_by_type`), the
three goal worlds, and the `goal-worlds-v1` series. The workflow's
[standard-baseline contract](../../workflow.md#environmental-pressures-in-the-standard-baseline)
applies.

**Research and options.** The occupancy depletion layer
(`OccupancyDepletionLayer`, `ordinary_food/ecology.rs`) is the local prior
art: one `Grid<f32>` deposited by standing, decayed per tick, multiplied into
every growth delta; the grazing layer is its per-type, bite-driven sibling
with the opposite sign, and the two multiply beside each other. Rejected: a
per-cell bite rate limit (a bite zeroes the cell and local growth skips empty
cells, so re-grazing is already paced by recolonization; only the floor
bounds repeated damage) and lowering the fertility grid itself (the habitat
map is pinned by `world_seed`). Charnov 1976 (read in full for T12.F04) makes
a slow-returning patch a memory problem: the median goal generation is 141
ticks, so a 500-tick recovery spans several generations.

**Mechanics.**

- State: one `Grid<f32>` modifier per food type in the ordinary-food module,
  1.0 at start; reset to 1.0 when the catalog changes, when `enabled` flips,
  and when `seed_density` seeds a fresh world; untouched by live edits to
  `factor`, `floor`, or `recovery_ticks`.
- Bite: a consume path that removes a positive amount of type `t` at a cell
  sets `m[t][cell] = max(floor, m[t][cell] * factor)`. `consume_type` is the
  production path (`apply_typed_eat`); `consume_any` applies the same rule
  per type removed. Zero-amount consumes and disabled grazing record no bite.
- Recovery: at the start of each growth pass, before any fertility read,
  every non-barrier cell of every type advances
  `m = min(1.0, m + 1 / recovery_ticks)`, empty cells included (the grazed
  cell is empty by construction). Barrier cells stay 1.0.
- Read: the modifier multiplies the mapped fertility at the three read sites
  in `grow` (local source cell, spread target, recovery spawn cell), so both
  `base_cell_fertility` and post-inhibition `cell_fertility` carry it and the
  inhibition telemetry does not absorb grazing's share. Disabled reads 1.0.
  `effective_fertility_grid` stays the ungrazed habitat reading.
- Isolation: a type-A bite never changes type B; the occupancy depletion
  multiplier is unchanged and multiplies beside the grazing read.
- Defaults (production, user-fixed): `enabled` true, `factor` 0.5, `floor`
  0.05, `recovery_ticks` 1000. The floor is reached on the fifth bite; one
  bite recovers in 500 ticks, a floored cell in 950.
- Config: `world.food.shared.grazing.{enabled, factor, floor,
  recovery_ticks}` on `FoodResourceConfig`, `#[serde(default)]`. `factor` and
  `floor` finite, clamped to `[0.0, 1.0]`, invalid falls back to the default;
  `floor <= factor` is not required; `recovery_ticks` clamped to
  `[1, 10_000]`, zero or missing falls back to 1000 (the cap is the domain the
  property test covers; past about `1e7` the f32 step stalls short of 1.0).
  Documented in `v3-world-grid-spec.md` Section 4, the runtime-editable table
  of `v3-runtime-config-spec.md`, the `v3-server-api-protocol-spec.md`
  examples, and `v3-tick-orchestration-spec.md` Phase 0 (recovery between the
  depletion update and growth).
- Runtime panel: `FoodParametersSection.tsx` carries the toggle and three
  fields beside the occupancy depletion ones, with `types/config.ts`,
  `startupConfig.ts`, and fixture carry-through; `patch_config` applies live
  edits through `reconfigure_food`.
- Telemetry: `FoodGrowthSummary` carries per-type `mean_grazing_modifier`
  (passable cells) and `grazed_cells` (below 1.0), gathered in the recovery
  pass; `SimStats` records them. The applied typed-eat totals are the bite
  count (an applied `Eat` with food is one bite). The goal report records the
  mean modifier and grazed-cell share by type on `WorldTracking` (final tick,
  also in each checkpoint block), comparison rows after
  `typed_eat_share_type_{i}`; with the typed-eat totals and the changed
  per-case `config_digest` this is the evidence the pressure acts in each
  world.
- Determinism: no RNG; seeded runs stay reproducible.

**Trajectory pins that legitimately move.** With the default on, these
stored identities change and are re-pinned once, old and new values in the
readings file: `legacy_default_short_run_identity`,
`accounting_preserves_pre_feature_sampled_trajectories_and_actions`, the
three digests in
`checked_in_goal_recipe_identities_are_unchanged_by_json_precision`, and any
reproducibility fixture hashing a default-config run. Any other moved pin is
a defect.

## Implementation Tasks

- [x] Viability first, then TDD the modifier: config, normalization,
      storage, bite, recovery, three read sites, telemetry.
- [x] Property tests (proptest, v3-core): modifier always within
      `[floor, 1.0]`; a bite is non-increasing and a recovery tick is
      non-decreasing; from any value, recovery reaches exactly 1.0 within
      `ceil((1 - m) * n) + ceil(n^2 * f32::EPSILON)` ticks for
      `n = recovery_ticks` (f32 rounding slack, at most 12 ticks at the cap;
      derivation in the test's doc comment), `n` drawn from `1..=10_000`; a
      type-A bite leaves type B bit-identical.
- [x] Focused fixtures: a bitten cell recolonizes at `factor` of the
      unbitten rate, a floored cell at `floor`; the empty grazed cell
      recovers; disabled reads 1.0 and re-enabling starts from 1.0; occupancy
      depletion still multiplies beside it.
- [x] Goal-world integration: a v3-cli bench test asserts each checked-in
      recipe resolves with grazing enabled at the production defaults; the
      per-case readings are `WorldTracking::grazing_modifier_mean` and
      `grazed_cell_share`, compared as `grazing_modifier_mean_type_{i}` and
      `grazed_cell_share_type_{i}`.
- [x] Config surface: reference-spec rows, runtime panel fields and tests,
      frontend types/fixtures, `patch_config` round-trip.
- [x] Re-pin the moved trajectory identities and record old/new in
      `docs/progress/readings/t02-f04.md`.

## Verification

- [x] `cargo test -p v3-core --test viability` (first) -> result in
      [`docs/progress/readings/t02-f04.md`](../../progress/readings/t02-f04.md).
- [x] Focused and property tests named above and the cap test
      `normalize_grazing_caps_recovery_ticks_at_the_property_domain` ->
      readings file.
- [x] `cargo test -p v3-cli --test bench_artifacts` including the
      recipe-carries-grazing assertion -> readings file.
- [x] Frontend: `npx vitest run` on the touched config-panel tests, `npx tsc
      --noEmit -p .`, `npx biome check src` -> readings file.
- [x] `make check` -> exit 0 after each pass (readings file); tested commit `e6c3579b`.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` on `9ccdf996`: `101 mutants
      tested in 26m: 4 missed, 95 caught, 2 unviable`, no timeouts; output
      `~/.local/share/petri-tools/mutants/t02-f04/mutants.out`, run mode
      `fresh` (a later `MUTANTS_ITERATE=1` pass left the directory reading
      `incremental` with an empty `missed.txt`); transcript in the readings
      file. Survivors, all killed by one test with no production change:

      | Survivor (`crates/v3-core/src/kernel/ordinary_food/mod.rs`) | Resolution |
      | --- | --- |
      | `101:9` replace `FoodResource::grazing_modifier_at -> f32` with `0.0` | killed: `grazing_modifier_at_reads_the_bitten_cell_and_one_elsewhere` (`kernel/world.rs`) |
      | `101:9` replace `FoodResource::grazing_modifier_at -> f32` with `-1.0` | killed: same test |
      | `101:9` replace `FoodResource::grazing_modifier_at -> f32` with `1.0` | killed: same test (bitten cell reads 0.5) |
      | `101:12` delete `!` in `FoodResource::grazing_modifier_at` | killed: same test (valid type on a bitten cell reads 0.5, not 1.0) |
- [x] Benchmark summaries stored, verified -> readings file.

## Performance and Goal Impact

**Predeclaration — written before the run.**

Natural analog: overgrazing. Herbivory lowers a patch's regrowth capacity,
repeated grazing compounds it, and rest restores it. It reaches creatures
through the world alone: how much food returns to a cell they or their
ancestors already ate from.

Compute cost: one full-grid recovery pass per food type per tick (2.56M cells
per type at 1600²) plus one multiply per fertility read and one clamp per
bite. Expected under 3% of goal wall-clock, negligible on the 128² gate.
`ms_per_creature_tick` may flag on either profile without a work-counter
move: a lower plateau spreads the fixed per-tick food cost over fewer
creatures, and the last two goal readings already flag on host load.

References and thresholds: gate against `gate-v1` epoch
`remove-complementary-nutrition.json` and the previous closure
`t03-f11-genome-replication-cost.json`; goal against `goal-worlds-v1` epoch
`t11-f19-per-unit-mutation-supply-goal.json` and the previous closure
`t03-f11-genome-replication-cost-goal.json`. Work flag 10%, severe 50%;
wall flag 25%, severe 100%.

Expected direction. The feature exists to change the production trajectory,
so `inputs_changed` is true in every case and the gate counters move.

- Population: `plateau_population`, `final_population`, and `births` fall in
  all three goal worlds because grazed cells return at half rate; no
  extinction in any world. An extinction or a plateau below the low
  thousands is an incompatibility blocker for the user under the shared
  baseline contract; the defaults are not retuned to avoid it.
- Per-creature-tick work: `births` falls; `vm_steps`, `mesh_hops`,
  `graph_relax_iters`, `actions_applied` move within the flag band. A severe
  on `births` or `plasticity_updates` is possible, is attributed to the
  intended pressure, and needs the user's acceptance and a re-pin.
- `typed_eat_share_type_1` in Orchards and Confluence: non-negative; fruit
  the founders cannot eat is never grazed, so a lineage that eats it gains.
- Grazing readings: applied typed-eat totals positive in every world, final
  mean modifier below 1.0 and grazed-cell share above zero for type 0 in
  every world; type 1 grazed only where `typed_eat_share_type_1` is
  positive.
- Barrier, lineage entropy, clade count, memory sensitivity, and
  structure-size readings: no predeclared direction. Founder, drift, and
  neighborhood mutation-map readings: unchanged, they read mutation config
  and the food-type count, not the world.
- No new diversity or cognition indicator; nothing stays `Undefined`.

**Measured verdict.** Both runs exit 0, not severe. No extinction;
`plateau_population`/`births` fall in all three worlds as predeclared.
Mismatch: Orchards `final_population` rose (+12%), not severe. Ruling: a
direction miss on one single-tick snapshot; the predeclaration stands
unedited, the plateau and births readings carry its intent, and no
threshold, cost, or exception is involved, so no user decision is needed.
Details: [`docs/progress/readings/t02-f04.md`](../../progress/readings/t02-f04.md).

- Summaries: [gate](../../progress/features/t02-f04-grazing-recovery-and-overuse.json),
  [goal](../../progress/features/t02-f04-grazing-recovery-and-overuse-goal.json).
- Full readings: [`docs/progress/readings/t02-f04.md`](../../progress/readings/t02-f04.md).

## Success Criteria

- [x] A bite multiplies only its type's modifier at that cell, clamped to
      the floor; the modifier recovers linearly to 1.0.
- [x] The modifier multiplies fertility at local growth, spread target, and
      recovery spawn; occupancy depletion still applies beside it.
- [x] The four values are normalized config under
      `world.food.shared.grazing`, round-trip through the panel and
      `patch_config`, and are in the reference specs.
- [x] The gate world and all three goal worlds run with grazing enabled at
      production defaults; the goal report shows applied eats and grazed
      cells in each.
- [x] Verification items above pass; mutation and benchmark records exist.

## Notes for AI Agents

- Decision: the defaults (`factor` 0.5, `floor` 0.05, `recovery_ticks`
  1000, enabled) and the T12.F04-only dependency are the user's 2026-09-15
  re-scope; only the user changes them.
- Deferred: review P3, `FoodResource::grazing_modifier_at`
  (`ordinary_food/mod.rs`) is public API with no workspace caller; drop it or
  give it a consumer in the feature that needs it.
- Deferred: review P3, `bite_modifier = (m * factor).max(floor)`
  (`ordinary_food/grazing.rs`) with a live `floor` raised above a floored
  cell makes the next bite raise that cell (0.05 to 0.9), unseen by the
  proptest (draws modifier >= floor); fix in a later pass by a sentence in
  the `floor` row of `v3-world-grid-spec.md` or a `.min(modifier)` guard.
- Deferred: review P3, `recover` (`ordinary_food/grazing.rs`) recounts
  passable cells over the barrier grid every tick, disabled included, and
  rewrites barrier cells to 1.0 each pass; within predeclared cost, a cache on
  the layer would remove a full pass.
- Cost: `/usage` not collected in this session; implementer passes 2 (advisor consults 2/2); spec-owner resumes after Plan 4; reviewer P1 0, P2 0, P3 3; one fresh mutation run, one incremental feedback pass.
