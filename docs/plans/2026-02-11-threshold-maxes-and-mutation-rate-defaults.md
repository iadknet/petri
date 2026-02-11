# Threshold Maxes and Mutation Defaults Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Increase most configurable max thresholds significantly and reduce default mutation rates substantially.

**Architecture:** Keep canonical default mutation-rate values in `petri-core::WorldConfig` and `petri-graph::MutationConfig`, raise startup validation ceilings in `petri-server::StartupDraft::validate`, and raise frontend slider ceilings in `web` `STARTUP_LIMITS` so UI and server constraints stay aligned.

**Tech Stack:** Rust workspace (`petri-core`, `petri-graph`, `petri-server`), React + TypeScript frontend (`web`), Rust tests + Vitest.

### Task 1: Add Backend Regression Tests (RED)

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-server/src/app_state.rs`

**Steps:**
1. Add a test asserting `WorldConfig::default()` uses the lowered mutation-rate defaults.
2. Add a test asserting `StartupDraft::validate()` accepts values above old max ceilings (for `initial_creatures`, `max_creatures`, `width`, `height`, `energy_initial`, `energy_per_tick_decay`, `energy_per_move`).
3. Run targeted server tests and verify failure before implementation.

Run:
```bash
cargo test -p petri-core default_mutation_rates_are_lowered
cargo test -p petri-server startup_draft_validation_allows_expanded_upper_bounds
```

### Task 2: Implement Backend Cap/Default Changes (GREEN)

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-graph/src/eval.rs`
- Modify: `crates/petri-server/src/app_state.rs`

**Steps:**
1. Lower default mutation rates in `WorldConfig::default`.
2. Lower default mutation rates in `MutationConfig::default`.
3. Raise startup-draft max validation ceilings in `StartupDraft::validate`.
4. Re-run targeted server tests and verify pass.

Run:
```bash
cargo test -p petri-core default_mutation_rates_are_lowered
cargo test -p petri-server startup_draft_validation_allows_expanded_upper_bounds
```

### Task 3: Add Frontend Regression Tests (RED)

**Files:**
- Modify: `web/src/features/simulation/store/simulationStore.test.ts`

**Steps:**
1. Add assertions for expanded startup slider max values for key controls.
2. Run targeted web test and verify failure before implementation.

Run:
```bash
cd web && npm run test -- src/features/simulation/store/simulationStore.test.ts
```

### Task 4: Implement Frontend Threshold and Fixture Updates (GREEN)

**Files:**
- Modify: `web/src/features/simulation/store/simulationStore.ts`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/App.test.tsx`

**Steps:**
1. Raise most `STARTUP_LIMITS.*.max` values significantly while preserving valid probability caps (`<= 1.0`).
2. Update web test runtime defaults to the lowered mutation-rate defaults for consistency with backend behavior.
3. Re-run targeted web tests and verify pass.

Run:
```bash
cd web && npm run test -- src/features/simulation/store/simulationStore.test.ts
```

### Task 5: Full Verification Gate

**Steps:**
1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`
