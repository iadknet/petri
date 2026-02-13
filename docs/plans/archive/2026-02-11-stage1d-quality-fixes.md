# Stage 1d Quality Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the Stage 1 high-priority quality issues by honoring `food_growth_rate` config semantics, making initial spawning reliably best-effort, and reducing per-frame wire payload size.

**Architecture:** Keep simulation behavior in `petri-core`, transport concerns in `petri-server`, and visualization adaptation in `web`. Add regression tests first for each issue, then apply the smallest behavior-preserving implementation needed to satisfy those tests.

**Tech Stack:** Rust workspace (`petri-core`, `petri-server`), Axum + MessagePack transport, React + TypeScript canvas client.

### Task 1: Food Growth Contract Regression (TDD)

**Files:**
- Modify: `crates/petri-core/src/world.rs`

**Step 1: Write the failing test**
- Add test `food_growth_rate_zero_prevents_spawn_growth` in `world.rs` tests.
- Configure world with `initial_creatures: 0`, `food_spawn_rate: 1.0`, `food_growth_rate: 0.0`.
- Run one tick and assert total food remains `0.0`.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p petri-core food_growth_rate_zero_prevents_spawn_growth -- --exact`
- Expected: FAIL because current code enforces a minimum growth increment.

**Step 3: Write minimal implementation**
- In `update_food`, replace hard floor `food_growth_rate.max(0.05)` with `food_growth_rate.max(0.0)`.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p petri-core food_growth_rate_zero_prevents_spawn_growth -- --exact`
- Expected: PASS.

### Task 2: Spawn Reliability Under High Occupancy (TDD)

**Files:**
- Modify: `crates/petri-core/src/world.rs`

**Step 1: Write the failing test**
- Add deterministic test `spawn_random_creature_finds_free_cell_beyond_random_attempt_window`.
- Build a world where exactly one cell is free and preselect that cell to be absent from the first 64 random probes.
- Assert `spawn_random_creature(0)` returns `Some(_)`.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p petri-core spawn_random_creature_finds_free_cell_beyond_random_attempt_window -- --exact`
- Expected: FAIL due the current 64-attempt random-only search.

**Step 3: Write minimal implementation**
- Change spawn search to random-start linear probe across all cells (`O(n)` worst case), returning first empty cell.
- Preserve randomness by choosing a random start index.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p petri-core spawn_random_creature_finds_free_cell_beyond_random_attempt_window -- --exact`
- Expected: PASS.

### Task 3: Frame Payload Reduction Regression (TDD)

**Files:**
- Modify: `crates/petri-server/src/lib.rs`
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/canvasRenderer.ts`

**Step 1: Write the failing test**
- Add server-side test `frame_payload_size_stays_below_threshold_for_default_world`.
- Build default `World`, take `frame()`, serialize with `rmp_serde::to_vec_named`, assert payload is below a strict threshold.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p petri-server frame_payload_size_stays_below_threshold_for_default_world -- --exact`
- Expected: FAIL with current float-based `food` payload.

**Step 3: Write minimal implementation**
- Change `WorldFrame.food` from `Vec<f32>` to quantized bytes (`Vec<u8>`).
- Quantize food values in `World::frame` (0..=`food_max_density` mapped to 0..255).
- Update TS protocol type and renderer to consume byte food values correctly.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p petri-server frame_payload_size_stays_below_threshold_for_default_world -- --exact`
- Expected: PASS.

### Task 4: Clippy Quality Gate Cleanup

**Files:**
- Modify: `crates/petri-core/src/world.rs`

**Step 1: Apply lint-safe cleanup**
- Remove unnecessary `as u64` casts for `CreatureId` ffi conversion.

**Step 2: Verify lint gate**
- Run: `cargo clippy --workspace --all-targets -- -D warnings`
- Expected: PASS.

### Task 5: Final Verification

**Files:**
- No additional edits expected

**Step 1: Rust verification**
- Run: `cargo test --workspace`
- Expected: PASS.

**Step 2: Web verification**
- Run: `cd web && npm run build`
- Expected: PASS.

**Step 3: Formatting check**
- Run: `cargo fmt --all --check`
- Expected: PASS.
