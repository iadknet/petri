# Startup Progress and Viability Toggle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make long world initialization observable in API/UI and add a server flag to disable viability probing.

**Architecture:** Extend server status with startup initialization stage and probe-enabled metadata, move expensive startup/restart work outside long-held write locks, and add a CLI flag (`--disable-viability-probe`) that flows into `AppState` startup behavior.

**Tech Stack:** Rust (`petri-server` with Axum + tracing + clap), React + TypeScript protocol/UI (`web`).

### Task 1: RED Tests for Probe Toggle

**Files:**
- Modify: `crates/petri-server/src/lib.rs`
- Modify: `crates/petri-server/src/main.rs`

**Steps:**
1. Add server integration test showing non-viable startup can still start when viability probe is disabled.
2. Add main-arg parsing test showing `--disable-viability-probe` is recognized.
3. Run targeted tests and verify failure.

### Task 2: Implement Probe Toggle and Status Metadata

**Files:**
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/lib.rs`
- Modify: `crates/petri-server/src/main.rs`
- Modify: `crates/petri-server/Cargo.toml`

**Steps:**
1. Add `AppStateOptions { viability_probe_enabled }`.
2. Wire options through app state construction.
3. Skip viability probe in startup viability evaluation and start/restart when disabled.
4. Add `viability_probe_enabled` to `SimulationStatus`.
5. Ensure startup progress phase/stage stays visible while long init runs and add startup/restart timing logs.

### Task 3: Wire Web Status Display

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/components/ControlHeader.tsx`
- Modify: `web/src/app/App.tsx`

**Steps:**
1. Add status fields for initialization stage and probe enabled.
2. Display initialization stage and probe mode in control header.

### Task 4: Verification Gate

**Steps:**
1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`
