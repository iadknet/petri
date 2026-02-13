# Raise Default Max Creatures Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ensure fresh simulation runs do not cap at `5000` by raising default `max_creatures`.

**Architecture:** Update both canonical runtime default (`petri-core::WorldConfig::default`) and startup draft default (`petri-server::StartupDraft::viable_default`) so startup and runtime initialization remain aligned.

**Tech Stack:** Rust (`petri-core`, `petri-server`) with unit/integration tests.

### Task 1: Add Failing Tests (RED)

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-server/src/lib.rs`

**Steps:**
1. Extend core config default test to assert higher `max_creatures`.
2. Extend server default startup draft test to assert higher `max_creatures`.
3. Run targeted tests and verify failure.

### Task 2: Implement Default Bump (GREEN)

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-server/src/app_state.rs`

**Steps:**
1. Raise `WorldConfig::default().max_creatures`.
2. Raise `StartupDraft::viable_default().max_creatures`.
3. Re-run targeted tests and verify pass.

### Task 3: Verification Gate

**Steps:**
1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`
