# Lower Food Defaults And Zero Runtime Rates Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce default food availability and allow startup/runtime food spawn and growth controls to be lowered to `0.0`.

**Architecture:** Keep simulation behavior unchanged in `petri-core`, adjust default startup tuning in `petri-server`, and align frontend slider bounds with backend-accepted ranges. Verify through focused backend/frontend tests and full repository quality gates.

**Tech Stack:** Rust (`petri-server`, `petri-core`), React + TypeScript (`web`), Vitest, Cargo test/clippy/fmt.

### Task 1: Add failing backend test for lower default startup food settings

**Files:**
- Modify: `crates/petri-server/src/lib.rs`

**Step 1: Write the failing test**

Add a test that reads `/simulation/status` and asserts startup draft defaults for:
- `initial_food_density`
- `food_spawn_rate`
- `food_growth_rate`

with lower expected values than current defaults.

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-server default_startup_draft_uses_lower_food_settings`
Expected: FAIL because `StartupDraft::viable_default()` still uses older higher values.

### Task 2: Add failing frontend test for zero-min food sliders

**Files:**
- Create: `web/src/features/simulation/store/simulationStore.test.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`

**Step 1: Write the failing test**

Add a test asserting:
- `STARTUP_LIMITS.initial_food_density.min === 0`
- `STARTUP_LIMITS.food_spawn_rate.min === 0`
- `STARTUP_LIMITS.food_growth_rate.min === 0`

**Step 2: Run test to verify it fails**

Run: `cd web && npm run test -- simulationStore.test.ts`
Expected: FAIL due to existing non-zero mins.

### Task 3: Implement minimal backend and frontend changes

**Files:**
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `web/src/features/simulation/store/simulationStore.ts`

**Step 1: Lower startup default food settings**

Update `StartupDraft::viable_default()` defaults to lower values for:
- `initial_food_density`
- `food_spawn_rate`
- `food_growth_rate`

**Step 2: Allow slider mins to reach zero**

Update food-related `STARTUP_LIMITS` mins to `0` for startup/runtime sliders.

**Step 3: Run tests to verify green**

Run:
- `cargo test -p petri-server default_startup_draft_uses_lower_food_settings`
- `cd web && npm run test -- simulationStore.test.ts`

Expected: PASS.

### Task 4: Full verification gates

**Files:**
- Modify if needed: `README.md` (only if user-visible defaults documentation requires updates)

**Step 1: Rust gates**

Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

**Step 2: Frontend gates**

Run:
- `cd web && npm run test`
- `cd web && npm run test:e2e`
- `cd web && npm run build`

**Step 3: Final validation**

Confirm runtime food spawn/growth sliders can be set to `0.00` and startup food defaults are lower than previous values.
