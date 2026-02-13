# Rust Test Runtime Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Speed up day-to-day Rust test loops without reducing test coverage or relaxing completion gates.

**Architecture:** Keep full workspace verification as-is while adding explicit targeted-test guidance in `AGENTS.md`. Reduce avoidable `petri-server` test runtime by adding a fast test constructor with viability probe disabled, then preserve viability coverage by keeping dedicated viability tests on the probe-enabled constructor.

**Tech Stack:** Rust (`cargo test`, `tokio`), repository policy docs (`AGENTS.md`).

### Task 1: Update Rust test policy guidance

**Files:**
- Modify: `AGENTS.md`

1. Add a `Rust Test Strategy` section under testing guidance.
2. Document fast-loop commands (`cargo test -p <crate> <test_name> -- --exact`, `cargo test -p <crate>`).
3. Keep full gate requirement (`cargo test --workspace`) and clarify when to run it earlier.
4. Document parallelization guidance and optional measured thread tuning.

### Task 2: Add fast server test constructor and coverage guardrails

**Files:**
- Modify: `crates/petri-server/src/app_state.rs`
- Modify: `crates/petri-server/src/lib.rs`

1. Add `AppState::new_for_tests_fast()` that disables viability probe.
2. Add tests proving constructor behavior (`new_for_tests` keeps probe enabled, fast constructor disables it).
3. Switch non-viability `petri-server` API tests to `new_for_tests_fast()`.
4. Keep viability-focused tests (`non_viable_startup_config_is_rejected`, `status_reports_non_viable_startup_draft`) on `new_for_tests()`.

### Task 3: Verification

Run:
- `cargo test -p petri-server --lib tests::health_endpoint_returns_ok -- --exact`
- `cargo test -p petri-server --lib tests::patch_config_updates_runtime_values -- --exact`
- `cargo test -p petri-server --lib tests::non_viable_startup_config_is_rejected -- --exact`
- `cargo test -p petri-server --lib tests::status_reports_non_viable_startup_draft -- --exact`
- `cargo test -p petri-server --lib tests::non_viable_startup_can_start_when_viability_probe_is_disabled -- --exact`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`
