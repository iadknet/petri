# World Size Default and UI Config Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set the default world size to `400x400` and make world dimensions configurable from the startup-draft UI.

**Architecture:** Keep canonical default values in `petri-core::WorldConfig`, expose startup world dimensions via `petri-server` startup draft contracts, and let the web startup controls patch those fields through existing `/simulation/startup-draft` flow. Preserve runtime-vs-startup boundaries (world size remains startup-only, applied on start/restart).

**Tech Stack:** Rust workspace (`petri-core`, `petri-server` with Axum), React + TypeScript frontend (`web`), Vitest + MSW tests.

### Task 1: Add Backend Regression Tests (RED)

**Files:**
- Modify: `crates/petri-server/src/lib.rs`

**Steps:**
1. Add a test proving default startup draft reports `width=400` and `height=400`.
2. Add a test proving `PATCH /simulation/startup-draft` accepts and returns world dimensions.
3. Run targeted server tests and confirm failures before implementation.

### Task 2: Implement Backend Dimension Support (GREEN)

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-server/src/app_state.rs`

**Steps:**
1. Change `WorldConfig::default()` to `width: 400`, `height: 400`.
2. Add `width`/`height` to `StartupDraft` and `StartupDraftPatch`.
3. Apply + validate patched dimensions and include them in startup viability seed.
4. Use startup draft dimensions in `build_world_config`.
5. Include dimensions in `startup_draft_from_config`.
6. Re-run targeted server tests and confirm pass.

### Task 3: Add Frontend Regression Tests/Type Updates (RED)

**Files:**
- Modify: `web/src/App.test.tsx`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/protocol.ts`

**Steps:**
1. Extend startup draft fixtures/types to include `width`/`height`.
2. Add/adjust tests asserting the new world-size controls render.
3. Run targeted web tests and confirm failures before UI implementation.

### Task 4: Implement Frontend Controls (GREEN)

**Files:**
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/App.test.tsx`

**Steps:**
1. Add startup limits for width/height.
2. Add width/height sliders to startup draft panel.
3. Ensure updates patch startup draft with numeric values.
4. Keep existing startup draft restart semantics unchanged.
5. Re-run targeted web tests and confirm pass.

### Task 5: Full Verification Gate

**Steps:**
1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`
