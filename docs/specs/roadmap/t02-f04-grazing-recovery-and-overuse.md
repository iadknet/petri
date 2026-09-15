# T02.F04 — Grazing Recovery and Overuse

**Status**: In Progress
**Last updated**: 2026-09-15
**Feature**: T02.F04
**Track**: [T02 — Environmental Dynamics](../../roadmaps/t02-environmental-dynamics.md)

## Goal

Overgrazing, the natural analog: a bitten patch regrows more slowly than an
unbitten one, repeated bites compound the damage down to a floor, and the
damage heals with rest. Each food type at each cell carries a fertility
modifier in `[floor, 1.0]` that a consuming bite multiplies by `factor`, that
recovers linearly toward 1.0 by `1 / recovery_ticks` per tick, and that
multiplies the type's fertility wherever growth reads it. The pressure reaches
creatures only through how fast food returns to the ground they graze; there
is no grazing sensor. It ships enabled at production defaults, so the gate
world and all three goal worlds carry it, and the goal report shows it acting
in each.

## Non-Goals

- Per-type grazing parameters, per-world recipe overrides, or a recipe edit:
  the three checked-in recipes stay byte-identical and inherit the production
  default.
- Any change to the occupancy depletion layer (`deposit_per_occupied_tick`
  0.08, `RECOVERY_PER_TICK` 0.03, `MIN_GROWTH_MULTIPLIER` 0.35) or to the
  shared growth, spread, and recovery-spawn rules beyond the multiplication.
- Rebalancing a goal world that the pressure collapses: that is a recorded
  blocker for the user, not a quiet retune.
- A grazing sensor, a grazing input to any brain, or a change to `Eat`.
- Server health or frontend telemetry for the modifier; the goal report and
  `SimStats` carry the readings, the runtime panel carries only the config.
- Retiring the orphaned `crates/v3-core/src/kernel/food_resource/{depletion,
  growth,reconfigure}.rs` (not compiled; `mod.rs` re-exports
  `ordinary_food`). Do not edit them.

## Inputs and Invariants

**Contract.** The T02 track row and its 2026-09-15 Notes entry are the
feature definition; this spec adds nothing to them. Dependency: T12.F04 gives
the per-type fertility grids (`OrdinaryFoodState::fertility_by_type`), the
three goal worlds, and the `goal-worlds-v1` series. The workflow's
[standard-baseline contract](../../workflow.md#environmental-pressures-in-the-standard-baseline)
applies.

**Research and options.** The existing occupancy depletion layer
(`OccupancyDepletionLayer` in `ordinary_food/ecology.rs`) is the local prior
art: one `Grid<f32>` per world, deposited by standing, decayed per tick,
multiplied into every growth delta. The grazing layer is its per-type,
bite-driven sibling with the opposite sign convention (a modifier that
starts at 1.0 and is pulled down), and the two multiply beside each other.
Alternatives weighed and rejected: (a) a bite rate limit per cell instead of
a compounding floor, rejected because a bite zeroes the cell and local growth
skips empty cells, so re-grazing is already paced by recolonization and the
floor is the only thing that bounds repeated damage; (b) lowering the type's
fertility grid itself, rejected because the habitat map is pinned by
`world_seed` and must stay the tick-zero reading recipes and the app show.
Theory already read in full for T12.F04 (Charnov 1976, marginal value; a
patch is worth revisiting when its intake rate recovers to the habitat mean)
is what makes a slow-returning grazed patch a memory problem: the median goal
generation is 141 ticks, so a 500-tick single-bite recovery spans several
generations. No external dependency is involved.

**Mechanics (source of truth for the implementer).**

- State: one `Grid<f32>` modifier per configured food type, initialized to
  1.0, owned by the ordinary-food module beside `density_by_type`. Resized to
  1.0 when the catalog changes; reset to 1.0 when `enabled` flips; untouched
  by live changes to `factor`, `floor`, or `recovery_ticks`.
- Bite: whenever a consume path removes a positive amount of type `t` at a
  cell, `m[t][cell] = max(floor, m[t][cell] * factor)`. `consume_type` is the
  production path (`apply_typed_eat` → `World::consume_food_type`);
  `consume_any` applies the same rule to each type it removed so no future
  caller bypasses it. A zero-amount consume is not a bite. Disabled: no bite
  is recorded.
- Recovery: at the start of each food growth pass, before any fertility read,
  every non-barrier cell of every type advances
  `m = min(1.0, m + 1 / recovery_ticks)`. This runs for empty cells too: the
  grazed cell is empty by construction, and the growth loop's
  `source <= 0.0 → continue` must not skip it. Barrier cells stay 1.0.
- Read: the modifier multiplies the mapped fertility value
  (`map_fertility(...)`) at its three read sites in `grow` — the source cell
  for local growth, the target cell for spread, and the spawned cell for
  recovery spawns — so both `base_cell_fertility` and the post-inhibition
  `cell_fertility` carry it and the type-inhibition telemetry does not absorb
  grazing's share. Disabled: the read is 1.0. `effective_fertility_grid`
  keeps returning the ungrazed habitat reading; its doc comment says so
  instead of "the same reading `grow` uses".
- Isolation: a bite of type A never changes type B's modifier; the occupancy
  depletion multiplier is unchanged and multiplies beside the grazing read.
- Defaults (production, user-fixed): `enabled` true, `factor` 0.5, `floor`
  0.05, `recovery_ticks` 1000. From 1.0 the floor is reached on the fifth
  bite (0.5, 0.25, 0.125, 0.0625, 0.05); a single bite recovers in 500 ticks,
  a floored cell in 950.
- Config: `world.food.shared.grazing.{enabled: bool, factor: f32, floor:
  f32, recovery_ticks: u32}` on `FoodResourceConfig`, `#[serde(default)]` so
  every stored recipe and config loads. Normalization mirrors
  `normalize_food_shared`: `factor` and `floor` finite and clamped to
  `[0.0, 1.0]`, invalid falls back to the default; `floor <= factor` is not
  required; `recovery_ticks` minimum 1, zero or missing falls back to 1000.
  Reference rows go into `docs/reference/v3-world-grid-spec.md` Section 4 and
  the runtime-editable table and knob list of
  `docs/reference/v3-runtime-config-spec.md`; the config examples in
  `docs/reference/v3-server-api-protocol-spec.md` gain the block.
- Runtime panel: `FoodParametersSection.tsx` gains a grazing toggle and the
  three numeric fields beside the occupancy depletion ones, with the
  `types/config.ts`, `startupConfig.ts` merge/normalize, and fixture
  carry-through. The server's `patch_config` already reaches
  `reconfigure_food`, so a live edit applies through
  `apply_config_transition`.
- Telemetry: `FoodGrowthSummary` gains per-type `mean_grazing_modifier` over
  passable cells and `grazed_cells` (modifier below 1.0), gathered in the
  recovery pass, and `SimStats` records them like the depletion fields. The
  existing applied typed-eat totals are the bite count: an applied `Eat`
  with food is exactly one bite while grazing is enabled, so no second
  counter is added. The goal report records, per case, the final-tick mean
  modifier and grazed-cell share by type next to `typed_eat_share`. That
  reading, the typed-eat totals, and the changed per-case `config_digest`
  are the evidence the pressure is enabled and acting in each world.
- Determinism: the layer uses no RNG; seeded runs stay reproducible.

**Trajectory pins that legitimately move.** Because the default is on, these
stored identities change and are re-pinned once, old and new values recorded
in the readings file: `legacy_default_short_run_identity` (baseline_worlds),
`accounting_preserves_pre_feature_sampled_trajectories_and_actions`
(applied_trajectory), the three digests in
`checked_in_goal_recipe_identities_are_unchanged_by_json_precision`, and any
reproducibility fixture that hashes a default-config run. A pin that moves
for any other reason is a defect.

## Implementation Tasks

- [ ] Run `cargo test -p v3-core --test viability` first, then TDD the
      modifier: config struct, defaults, normalization, storage, bite,
      recovery, three read sites, telemetry.
- [ ] Property tests (proptest, v3-core): modifier always within
      `[floor, 1.0]`; a bite is non-increasing and a recovery tick is
      non-decreasing; from any value, recovery reaches exactly 1.0 within
      `ceil((1 - m) * recovery_ticks) + 1` ticks (the extra tick absorbs f32
      accumulation); a type-A bite leaves type B bit-identical.
- [ ] Focused fixtures: a bitten cell recolonizes from a dense neighbor at
      `factor` of the unbitten rate; a floored cell at `floor`; the empty
      grazed cell recovers; disabled reads 1.0 everywhere and re-enabling
      starts from 1.0; occupancy depletion still multiplies beside it.
- [ ] Goal-world integration: a v3-cli bench test asserts each checked-in
      recipe resolves with grazing enabled at the production defaults; the
      per-case grazing readings appear in the goal report schema and
      comparison block.
- [ ] Config surface: reference-spec rows, runtime panel fields and tests,
      frontend types/fixtures, `patch_config` round-trip.
- [ ] Re-pin the moved trajectory identities and record old/new in
      `docs/progress/readings/t02-f04.md`.

## Verification

- [ ] `cargo test -p v3-core --test viability` (first) -> result in
      [`docs/progress/readings/t02-f04.md`](../../progress/readings/t02-f04.md).
- [ ] Focused and property tests named above -> test names and results in
      the readings file.
- [ ] `cargo test -p v3-cli --test bench_artifacts` including the
      recipe-carries-grazing assertion -> readings file.
- [ ] Frontend: `npx vitest run` on the touched config-panel tests, `npx tsc
      --noEmit -p .`, `npx biome check src` -> readings file.
- [ ] `make check` -> clean.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t02-f04-grazing-recovery-and-overuse.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.**

Natural analog: overgrazing. Herbivory lowers a patch's regrowth capacity,
repeated grazing compounds it, and rest restores it. It reaches creatures
through the world alone: how much food returns to a cell they or their
ancestors already ate from.

Compute cost: one full-grid recovery pass per food type per tick (2.56M cells
per type at 1600²) plus one multiply at each fertility read and one clamp per
bite. Expected under 3% of goal wall-clock and negligible on the 128² gate.
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

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t02-f04-grazing-recovery-and-overuse.json),
  [goal](../../progress/features/t02-f04-grazing-recovery-and-overuse-goal.json).
- Full readings: [`docs/progress/readings/t02-f04.md`](../../progress/readings/t02-f04.md).

## Success Criteria

- [ ] A consuming bite multiplies only its type's modifier at that cell,
      clamped to the floor, and the modifier recovers linearly to 1.0.
- [ ] The modifier multiplies fertility at local growth, spread target, and
      recovery spawn; occupancy depletion still applies beside it.
- [ ] The four values are config under `world.food.shared.grazing`, are
      normalized, round-trip through the runtime panel and `patch_config`,
      and are documented in the reference specs.
- [ ] The gate world and all three goal worlds run with grazing enabled at
      the production defaults, and the goal report shows applied eats and
      grazed cells in each.
- [ ] Verification items above pass; mutation and benchmark records exist.

## Notes for AI Agents

- Decision: the defaults (`factor` 0.5, `floor` 0.05, `recovery_ticks`
  1000, enabled) and the T12.F04-only dependency are the user's 2026-09-15
  re-scope; only the user changes them.
