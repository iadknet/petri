# Stage 1e Frontend-Controlled Simulation Startup and Tuning Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move startup authority to an explicit frontend-triggered lifecycle with idle boot, server-side startup draft persistence, runtime tuning, restart semantics, and viability guards.

**Architecture:** Keep lifecycle/state orchestration in `petri-server`, startup seeding semantics and world behavior in `petri-core`, and control-surface/UI behavior in `web`. Preserve existing frame stream format and `/ws` transport contract.

**Tech Stack:** Rust workspace (`petri-core`, `petri-server`) with Axum + WebSocket transport, React + TypeScript frontend.

### Task 1: Introduce Server Lifecycle State Machine and Contracts

**Files:**
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/api.rs`
- Modify: `crates/petri-server/src/sim_loop.rs`
- Modify: `crates/petri-server/src/lib.rs`

**Steps:**
1. Add startup draft/domain types (`StartupDraft`, `StartupDraftPatch`, `SimulationStatus`, `RuntimeConfigPatch`), explicit phases (`idle|running|paused`), and pending restart tracking.
2. Change boot behavior to idle (no active world).
3. Add endpoints:
   - `GET /simulation/status`
   - `GET /simulation/startup-draft`
   - `PATCH /simulation/startup-draft`
   - `POST /simulation/start`
   - `POST /simulation/restart`
4. Extend `PATCH /config` with live `food_spawn_rate` and `food_growth_rate` updates.
5. Keep `/health`, `/ws`, and `GET /config` behavior stable and backward-compatible.
6. Ensure sim loop ticks only when run exists and phase is running.

### Task 2: Startup Seeding and Viability Enforcement in Core/Server

**Files:**
- Modify: `crates/petri-core/src/world.rs`
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/lib.rs`

**Steps:**
1. Implement density-based initial food seeding semantics for startup worlds (`initial_food_density * cell_count` random unique cells to `food_max_density`).
2. Add deterministic viability preflight before start/restart:
   - validate ranges
   - run short probe (100 ticks, deterministic seed path)
   - require `population > 0` at probe end
3. Return machine-readable error codes for invalid/non-viable startup configs.
4. Add regression tests for idle start requirements, pending restart behavior, restart seed changes, runtime patch behavior, and non-viable rejection.

### Task 3: Frontend Idle-First Control Surface and Wiring

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/App.tsx`
- Modify: `web/src/styles.css`

**Steps:**
1. Add frontend protocol types matching new backend contracts.
2. Build startup draft controls (Core 6) and wire to `PATCH /simulation/startup-draft`.
3. Build simulation controls for Start/Restart/Pause-Resume and runtime knobs for tick speed + food spawn/growth.
4. Show idle placeholder when phase is idle and consume frame stream only when active.
5. Expose pending restart and run status/metrics in control bar.

### Task 4: Verification Gate

**Steps:**
1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`
