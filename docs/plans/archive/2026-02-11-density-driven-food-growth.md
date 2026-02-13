# Density-Driven Food Growth Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace fixed random food growth with density-driven local growth and thresholded spread, while keeping conditional fallback spawning.

**Architecture:** Extend `WorldConfig` with two new food-threshold knobs, rewrite `World::update_food` to operate from a pre-tick snapshot, then wire new config fields through server startup/runtime APIs and web controls. Preserve existing simulation boundaries (`petri-core` policy, `petri-server` transport/API, `web` UI/typing).

**Tech Stack:** Rust workspace (`petri-core`, `petri-server`) + React/TypeScript frontend (`web`).

## Behavior Contract

1. Local growth each tick: `delta = source_food * food_growth_rate`; add to same cell, capped by `food_max_density`.
2. Spread is only allowed when source food is at least `food_spread_threshold * food_max_density`.
3. Spread attempt targets exactly one random 4-neighbor (N/E/S/W).
4. Spread amount is `source_food * food_growth_rate`, added to target, capped by `food_max_density`.
5. Spread does not subtract source food.
6. Average density uses pre-tick snapshot: `sum(food) / (cell_count * food_max_density)`.
7. Fallback random spawning runs only when average density is below `food_spawn_floor_density`.
8. Fallback spawn attempts remain `round(cell_count * food_spawn_rate)`.
9. Fallback spawn amount per attempt is `food_max_density * food_growth_rate`.
10. `food_growth_rate == 0.0` still means no growth and no spawn contribution.

## Task 1: Core RED tests

**Files:**
- Modify: `crates/petri-core/src/world.rs`

1. Add failing tests for:
   - density-proportional local growth
   - no spread below threshold
   - spread at threshold with deterministic single-neighbor setup
   - no fallback spawn when average density is above floor
   - fallback spawn when average density is below floor
2. Run targeted test selection to verify failures.

## Task 2: Core implementation

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-core/src/world.rs`

1. Add `food_spread_threshold` and `food_spawn_floor_density` to `WorldConfig` with defaults `0.75` and `0.03`.
2. Rewrite `update_food` to:
   - use pre-tick food snapshot
   - apply local multiplicative growth
   - apply conditional one-neighbor spread
   - apply conditional fallback random spawn by world-average floor
3. Keep deterministic RNG usage and existing clamping invariants.
4. Run targeted core tests, then `cargo test -p petri-core`.

## Task 3: Server RED tests + implementation

**Files:**
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/lib.rs`

1. Add failing API tests for default/startup/runtime payloads containing new fields.
2. Extend startup/runtime patch structs and draft structs for new fields.
3. Add default values to `StartupDraft::viable_default`.
4. Validate new startup fields in `[0.0, 1.0]`.
5. Wire fields through:
   - `apply_patch`
   - `build_world_config`
   - `startup_draft_from_config`
   - `apply_runtime_patch`
   - `startup_probe_seed`
6. Run `cargo test -p petri-server`.

## Task 4: Web RED tests + implementation

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/api/simulationApiClient.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `web/src/features/simulation/store/simulationStore.test.ts`
- Modify: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Modify: `web/src/features/simulation/components/RuntimeTuningPanel.tsx`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/App.test.tsx`

1. Add failing tests for:
   - new slider limits in `STARTUP_LIMITS`
   - app rendering of new controls
2. Extend protocol/runtime/store/test handlers for new fields.
3. Add startup and runtime sliders for both fields.
4. Run `cd web && npm test` (or targeted vitest as available), then `cd web && npm run build`.

## Task 5: Completion gate

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`

## Defaults and Assumptions

- `food_spread_threshold` default: `0.75`
- `food_spawn_floor_density` default: `0.03`
- Spread neighborhood: 4-way (N/E/S/W)
- New knobs are startup + runtime configurable
- Existing food/creature co-location semantics remain unchanged
