# Stage 2 Slice 1: Barrier Substrate + Frame/Render Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add barrier substrate state with frame streaming and frontend rendering, while keeping default worlds barrier-free and snapshot compatibility.

**Architecture:** Extend the core cell substrate with a `barrier` bit, enforce movement/spawn/reproduction blocking in `petri-core`, expose packed barrier data in frame payloads, persist barrier state in snapshots with backward-compatible serde defaults, and render barriers in the canvas layer using a fixed faded-rust color.

**Tech Stack:** Rust (`petri-core`, `petri-server`), TypeScript/React canvas (`web`), MessagePack (`rmp-serde`, `@msgpack/msgpack`), Vitest/Playwright.

## Task 1: Branch/worktree + plan baseline

**Files:**
- Create: `docs/plans/2026-02-11-stage2-slice1-barrier-substrate-frame-render.md`

### Steps
1. Ensure isolated worktree branch exists: `codex/stage2-slice1-barrier-substrate-frame-render`.
2. Save this implementation plan file before code edits.

## Task 2: Core substrate + frame schema

**Files:**
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/world/helpers.rs`
- Modify: `crates/petri-core/src/types.rs`

### Steps
1. Add failing tests for barrier frame packing in `crates/petri-core/src/world/tests.rs`.
2. Add `barrier: bool` to world `Cell` and initialize to `false`.
3. Add helper to pack cell barriers into LSB-first bit-array.
4. Add `WorldFrame.barrier_bits: Vec<u8>` and emit it from `World::frame()`.

## Task 3: Barrier movement and placement rules

**Files:**
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/spawn.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

### Steps
1. Add failing tests for movement block, founder spawn avoidance, and reproduction placement avoidance.
2. Prevent movement into barrier cells.
3. Prevent founder spawn onto barrier cells.
4. Prevent `find_empty_neighbor` from returning barrier cells.

## Task 4: Snapshot persistence (backward compatible)

**Files:**
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world/snapshot.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

### Steps
1. Add failing tests for snapshot barrier round-trip and legacy snapshot import without barriers.
2. Add `cells_barrier` with serde default to `WorldSnapshot`.
3. Export/import barrier vector with length-safe handling.
4. Ensure snapshot load never places creatures onto barrier cells.

## Task 5: Web protocol + renderer

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/protocol.test.ts`
- Modify: `web/src/canvasRenderer.ts`
- Create: `web/src/canvasRenderer.test.ts`
- Modify: `web/src/App.test.tsx`
- Modify: `web/src/test/handlers.ts`

### Steps
1. Add failing web protocol/renderer tests for `barrier_bits` and barrier rendering.
2. Extend frame and snapshot types with barrier fields.
3. Render barriers as `RGB(140, 90, 60)` before creature overlay.
4. Update typed frame/snapshot fixtures with barrier defaults.

## Task 6: Regression and verification

**Files:**
- Modify: `crates/petri-server/tests/payload_regression.rs`

### Steps
1. Keep payload regression coverage for enlarged frame schema.
2. Run full required gate:
   - `cargo fmt --all --check`
   - `cargo test -p petri-core`
   - `cargo test -p petri-server --tests`
   - `cargo test --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cd web && npm run test`
   - `cd web && npm run test:e2e`
   - `cd web && npm run build`
