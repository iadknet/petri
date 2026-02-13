# Stage 2 Slice 3 (Barrier-Aware Food + Sensor Radius) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement Stage 2 Slice 3 by making food spawn/spread respect barriers and occupied cells, and by adding a configurable `sensor_radius` that is wired through backend and frontend controls.

**Architecture:** Keep behavior changes in `petri-core` (`world/food.rs` + `world/perception.rs`) and configuration plumbing in existing config/state layers (`petri-core::WorldConfig`, `petri-server` startup/runtime patch surfaces, `web` protocol/store/panels). Preserve existing crate boundaries (`petri-core` simulation policy, `petri-server` transport/state orchestration, `web` UI controls). Drive all behavior changes with regression tests first.

**Tech Stack:** Rust (`petri-core`, `petri-server`), TypeScript/React (`web`), Vitest/MSW, cargo test/clippy/fmt, npm test/build/e2e.

### Task 1: Add failing `petri-core` regression tests for slice 3 behavior

**Files:**
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Write failing tests for barrier/occupancy-aware food rules**
Add tests that assert:
- food spread does not select barrier neighbors
- food spread does not select occupied neighbors
- fallback spawn does not place food in barrier cells
- fallback spawn does not place food in occupied cells

Example test skeleton to add:
```rust
#[test]
fn food_spread_skips_barrier_neighbors() {
    // setup world, set source food above threshold, mark one neighbor barrier
    // tick and assert only non-barrier target changed
}
```

**Step 2: Write failing tests for configurable sensor radius**
Add tests that assert:
- nearest food outside configured radius is not detected
- distance normalization uses configured `sensor_radius` value

**Step 3: Run targeted test scope and confirm failures**
Run: `cargo test -p petri-core food_spread_skips_barrier_neighbors`
Expected: FAIL before implementation.

**Step 4: Commit test scaffolding**
```bash
git add crates/petri-core/src/world/tests.rs
git commit -m "test(core): add slice3 food and sensor radius regressions"
```

### Task 2: Implement `petri-core` behavior and config changes

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/world/helpers.rs`
- Modify: `crates/petri-core/src/world/perception.rs`
- Modify: `crates/petri-core/src/world/food.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Add `sensor_radius` to `WorldConfig`**
- Add field with serde default/back-compat behavior.
- Set default radius to existing baseline (12).

**Step 2: Replace fixed sensor radius usage in perception**
- Use `self.config.sensor_radius` for scan loop bounds.
- Pass radius into distance normalization helper.

**Step 3: Update food logic to respect barriers/occupancy**
- Exclude barrier/occupied cells from spread targets.
- Exclude barrier/occupied cells from fallback spawn writes.
- Keep existing growth semantics for valid cells.

**Step 4: Run focused crate tests**
Run: `cargo test -p petri-core`
Expected: PASS.

**Step 5: Commit core implementation**
```bash
git add crates/petri-core/src/config.rs crates/petri-core/src/world/mod.rs crates/petri-core/src/world/helpers.rs crates/petri-core/src/world/perception.rs crates/petri-core/src/world/food.rs crates/petri-core/src/world/tests.rs
git commit -m "feat(core): implement barrier-aware food and configurable sensor radius"
```

### Task 3: Wire `sensor_radius` through `petri-server` startup/runtime surfaces

**Files:**
- Modify: `crates/petri-server/src/app_state/types.rs`
- Modify: `crates/petri-server/src/app_state/runtime_patch.rs`
- Modify: `crates/petri-server/src/app_state/startup_draft.rs`
- Modify: `crates/petri-server/src/app_state/viability.rs`
- Modify: `crates/petri-server/src/app_state/tests.rs`
- Modify: `crates/petri-server/tests/simulation_lifecycle.rs`

**Step 1: Add startup + runtime patch fields**
- Add `sensor_radius` in `StartupDraft`, `StartupDraftPatch`, and `RuntimeConfigPatch`.
- Include validation/clamping (e.g. `1..=64`).

**Step 2: Propagate to world config conversion paths**
- Include field in `build_world_config` and `startup_draft_from_config`.
- Include in startup probe seed hash for deterministic viability behavior.

**Step 3: Add/adjust tests first (failing) then implement**
- Startup draft patch updates `sensor_radius` and marks pending restart when applicable.
- Runtime patch updates/clamps `sensor_radius`.

**Step 4: Run targeted server tests**
Run: `cargo test -p petri-server simulation_lifecycle`
Expected: PASS.

**Step 5: Commit server wiring**
```bash
git add crates/petri-server/src/app_state/types.rs crates/petri-server/src/app_state/runtime_patch.rs crates/petri-server/src/app_state/startup_draft.rs crates/petri-server/src/app_state/viability.rs crates/petri-server/src/app_state/tests.rs crates/petri-server/tests/simulation_lifecycle.rs
git commit -m "feat(server): expose sensor radius in startup and runtime config"
```

### Task 4: Add frontend controls/types/tests for `sensor_radius`

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/api/simulationApiClient.ts`
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Modify: `web/src/features/simulation/components/RuntimeTuningPanel.tsx`
- Modify: `web/src/features/simulation/store/simulationStore.test.ts`
- Modify: `web/src/App.test.tsx`
- Modify: `web/src/test/handlers.ts`

**Step 1: Add failing UI tests first**
- Assert Startup Draft shows and updates `sensor_radius` slider.
- Assert Runtime Tuning shows and updates runtime `sensor_radius` slider.

**Step 2: Add protocol/runtime/startup typings**
- Add `sensor_radius` in `StartupDraft`, runtime config, and config patch types.
- Add slider limits in `STARTUP_LIMITS`.

**Step 3: Implement controls**
- Startup panel slider with existing `onUpdate` path.
- Runtime panel slider with existing optimistic `onUpdateRuntimeField` path.

**Step 4: Run frontend tests and build**
Run:
- `cd web && npm run test`
- `cd web && npm run build`
Expected: PASS.

**Step 5: Commit frontend changes**
```bash
git add web/src/protocol.ts web/src/features/simulation/api/simulationApiClient.ts web/src/features/simulation/store/simulationStore.ts web/src/features/simulation/components/StartupDraftPanel.tsx web/src/features/simulation/components/RuntimeTuningPanel.tsx web/src/features/simulation/store/simulationStore.test.ts web/src/App.test.tsx web/src/test/handlers.ts
git commit -m "feat(web): add sensor radius controls and typing"
```

### Task 5: Final verification + docs

**Files:**
- Modify: `README.md`

**Step 1: Update docs for user-visible control**
- Mention sensor radius in startup/runtime control summaries.

**Step 2: Run full project completion gate**
Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run test`
- `cd web && npm run test:e2e`
- `cd web && npm run build`
Expected: all PASS.

**Step 3: Commit docs/verification-ready state**
```bash
git add README.md
git commit -m "docs: mention sensor radius control in stage2 slice3"
```

### Boundary Impact

- Crate dependency direction remains unchanged: `petri-graph -> petri-core -> petri-server/petri-cli`.
- Public API/wire-format changes are additive only (`sensor_radius` field added to config/startup payloads).
- Test migration approach: keep behavior regressions in existing unit/integration test files nearest touched behavior (`petri-core` world tests, `petri-server` lifecycle tests, `web` store/component tests).
