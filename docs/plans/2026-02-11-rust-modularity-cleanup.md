# Rust Module Boundary Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce oversized Rust modules, keep crate boundaries clean, and align test placement with Rust conventions without behavior changes.

**Architecture:** Keep existing crate dependency layering intact (`petri-graph -> petri-core -> petri-server/petri-cli`), split large files by responsibility using `mod.rs + submodules`, and move integration-style tests out of `src/lib.rs` into crate `tests/` folders. Preserve all public exports and wire types.

**Tech Stack:** Rust workspace (`1.93.0`), Axum, serde, `cargo test`, `cargo clippy`.

## Public API / Interface Changes
1. No external API changes to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/lib.rs`.
2. No external API changes to `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/lib.rs`.
3. No external API changes to `/Users/istefanek/claude-evolution-game/crates/petri-server/src/lib.rs`.
4. No HTTP route or payload changes in `/Users/istefanek/claude-evolution-game/crates/petri-server/src/api.rs`.
5. No simulation frame/snapshot schema changes in `/Users/istefanek/claude-evolution-game/crates/petri-core/src/types.rs`.

## Task 0: Prepare Branch and Plan Artifact
**Files:** create `/Users/istefanek/claude-evolution-game/docs/plans/2026-02-11-rust-modularity-cleanup.md`.
1. Create branch `codex/rust-modularity-cleanup` (prefer `.worktrees/` isolation).
2. Save this plan into `/Users/istefanek/claude-evolution-game/docs/plans/2026-02-11-rust-modularity-cleanup.md`.
3. Record baseline verification output from:
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
4. Commit only the plan document.

## Task 1: Move `petri-server` Integration Tests Out of `src/lib.rs`
**Files:** modify `/Users/istefanek/claude-evolution-game/crates/petri-server/src/lib.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/tests/http_health.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/tests/http_cors.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/tests/simulation_lifecycle.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/tests/snapshot_and_creature.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/tests/payload_regression.rs`.
1. Extract all integration-style tests from `/Users/istefanek/claude-evolution-game/crates/petri-server/src/lib.rs` into the new `tests/` files grouped by behavior.
2. Leave `/Users/istefanek/claude-evolution-game/crates/petri-server/src/lib.rs` as exports/modules only.
3. Keep tests using public crate APIs (`build_router`, `AppState`, `sim_loop`).
4. Run `cargo test -p petri-server`.
5. Commit.

## Task 2: Move `petri-graph` Behavioral Tests Out of `src/lib.rs`
**Files:** modify `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/lib.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/tests/palette_eval.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/tests/mutation_behavior.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/tests/node_support.rs`.
1. Move high-level graph behavior tests from `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/lib.rs` into `tests/`.
2. Keep `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/lib.rs` as `mod` and `pub use` surface.
3. Run `cargo test -p petri-graph`.
4. Commit.

## Task 3: Split `petri-core` World Module by Concern
**Files:** replace `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world.rs` with module directory; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/mod.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/tick.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/food.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/perception.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/snapshot.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/spawn.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/helpers.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/tests.rs`.
1. Move `World` type and public API entrypoints to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/mod.rs`.
2. Move tick sequencing and per-creature lifecycle logic to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/tick.rs`.
3. Move food update/growth/spread/spawn logic to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/food.rs`.
4. Move scan/sensor logic to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/perception.rs`.
5. Move snapshot encode/decode logic to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/snapshot.rs`.
6. Move spawn/reproduction helpers to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/spawn.rs`.
7. Move low-level helpers (`axis_step`, `map_axis`, `wrap_axis`, `quantize_food`, `push_event`) to `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/helpers.rs`.
8. Move/retain world tests in `/Users/istefanek/claude-evolution-game/crates/petri-core/src/world/tests.rs` with unchanged assertions.
9. Run `cargo test -p petri-core`.
10. Commit.

## Task 4: Split `petri-graph` Eval Module by Concern
**Files:** replace `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval.rs` with module directory; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/mod.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/evaluate.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/mutate.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/presets.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/node_utils.rs`.
1. Keep `ComputationGraph` and `MutationConfig` definitions in `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/mod.rs`.
2. Move `evaluate` implementation to `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/evaluate.rs`.
3. Move mutation operations to `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/mutate.rs`.
4. Move palette/founder graph constructors to `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/presets.rs`.
5. Move node classification/random node helper functions to `/Users/istefanek/claude-evolution-game/crates/petri-graph/src/eval/node_utils.rs`.
6. Preserve serde shape and behavior.
7. Run `cargo test -p petri-graph`.
8. Commit.

## Task 5: Split `petri-server` App State Module by Concern
**Files:** replace `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state.rs` with module directory; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/mod.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/types.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/startup_draft.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/runtime_patch.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/viability.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/lifecycle.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/status.rs`; create `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/tests.rs`.
1. Move enums/DTOs/errors/internal state structs to `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/types.rs`.
2. Move startup draft defaults/patching/validation to `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/startup_draft.rs`.
3. Move runtime config patching to `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/runtime_patch.rs`.
4. Move viability probe and deterministic startup seed logic to `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/viability.rs`.
5. Move start/restart orchestration to `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/lifecycle.rs`, with shared helper to remove duplicated flow.
6. Keep status projection helper in `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/status.rs`.
7. Keep public `AppState` method signatures stable in `/Users/istefanek/claude-evolution-game/crates/petri-server/src/app_state/mod.rs`.
8. Run `cargo test -p petri-server`.
9. Commit.

## Task 6: Final Verification and Modularity Report
**Files:** optional documentation touch to `/Users/istefanek/claude-evolution-game/README.md` only if behavior/commands change (expected: no change needed).
1. Run:
`cargo fmt --all --check`
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
`cd /Users/istefanek/claude-evolution-game/web && npm run build`
2. Produce final size report for Rust files and split by production-vs-test lines.
3. Confirm targets:
Production modules ideally below ~400 lines unless justified.
`src/lib.rs` files remain export-focused.
Integration tests primarily live under `crates/*/tests/`.
4. Commit final cleanup if needed.

## Test Cases and Scenarios
1. `petri-core` behavior invariants stay unchanged for creature lifecycle, food update, and snapshot round-trip.
2. `petri-core` invariants stay unchanged for `initial_creatures` best-effort and direct honoring of `food_growth_rate`.
3. `petri-server` endpoints keep status codes and payloads for health, config patching, startup draft, start/restart, snapshot, and creature detail.
4. `petri-server` pending restart and viability probe behavior stays unchanged.
5. `petri-server` frame payload-size regression assertion remains covered.
6. `petri-graph` palette behavior, evaluation bounds, and mutation semantics remain unchanged under existing deterministic tests.

## Assumptions and Defaults
1. Scope is fixed to Balanced split (modularization + test relocation, no product redesign).
2. Public APIs and wire formats remain stable.
3. Integration-style tests are moved to `tests/` while private implementation tests may remain inline only when necessary.
4. Gemini MCP is unavailable in this environment (`gemini-analyze-code` missing), so local review is the active fallback path.

## Baseline Verification

### `cargo test --workspace`
- Result: PASS
- Highlights:
  - `petri-core`: 33 tests passed
  - `petri-graph`: 18 tests passed
  - `petri-server` lib: 21 tests passed
  - `petri-server` main: 4 tests passed
  - `petri-cli`: 4 tests passed

### `cargo clippy --workspace --all-targets -- -D warnings`
- Result: PASS
- No warnings promoted to errors.
