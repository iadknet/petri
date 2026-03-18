# Food Fertility Layer v1 — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a static fertility layer to the food system that creates persistent spatial structure, and extract the `FoodResource` abstraction as the foundation for future multi-food ecology.

**Architecture:** New `FoodResource` struct in `v3-core/src/kernel/` encapsulates food density, fertility grid, config, and growth logic. `WorldState` delegates food methods to `FoodResource`. Three fertility seeding algorithms (Uniform, Fbm, PoissonBlobs) generate the raw `[-1, 1]` fertility grid at startup, with mixed-mode layering. Annealing lerps the effective fertility range over time. Server transport adds `food_fertility_u8` to the world-static payload. Frontend adds a toggleable overlay.

**Tech Stack:** Rust (v3-core, v3-server), `noise` crate for Fbm, React/TypeScript (frontend), WebGL overlay rendering

**Spec:** `docs/superpowers/specs/2026-03-18-food-fertility-layer-design.md`

---

## Checkmark Protocol

Every step is a checkmark item. Implementing agents MUST update this file and companion files to check off items (`- [x]`) as completed. Do not proceed past Review Gates until checked off.

## Review Gate Protocol

1. Dispatch `superpowers:code-reviewer` with required domain skill
2. Fix all findings
3. Re-dispatch on changed code
4. Repeat until zero new findings
5. Re-run tests covering changed code
6. Only then check off the gate

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `v3/crates/v3-core/src/kernel/food_resource.rs` | `FoodResource` struct, growth logic, food accessors |
| `v3/crates/v3-core/src/kernel/fertility.rs` | `FertilityConfig`, `AnnealingConfig`, seeding algorithms, mixing pipeline |
| `v3/crates/v3-core/src/kernel/fertility/uniform.rs` | Uniform algorithm |
| `v3/crates/v3-core/src/kernel/fertility/fbm.rs` | Fbm algorithm (uses `noise` crate) |
| `v3/crates/v3-core/src/kernel/fertility/poisson_blobs.rs` | PoissonBlobs algorithm |
| `v3/crates/v3-core/src/kernel/fertility/mixing.rs` | Layer mixing pipeline |
| `v3/crates/v3-core/src/kernel/fertility/annealing.rs` | Annealing computation |

### Modified Files

| File | Change |
|------|--------|
| `v3/crates/v3-core/src/config/simulation.rs` | Rename `WorldFoodConfig` → `FoodResourceConfig`, add fertility/annealing fields |
| `v3/crates/v3-core/src/kernel/world.rs` | Add `food: FoodResource` field, delegate methods, `food()` accessor, `apply_food_config()` |
| `v3/crates/v3-core/src/kernel/mod.rs` | Add `pub mod food_resource; pub mod fertility;` |
| `v3/crates/v3-core/src/simulation/tick.rs:96` | Update `grow_food` call to pass `sim.tick` |
| `v3/crates/v3-core/Cargo.toml` | Add `noise` dependency |
| `v3/crates/v3-server/src/query/projection.rs` | Add `food_fertility_u8` to projection, update `same_world_static()` |
| `v3/crates/v3-server/src/query/cache.rs` | Add fertility quantization function |
| `v3/crates/v3-server/src/state.rs` | Add `food_fertility_u8` to `FramePayload` |
| `v3/crates/v3-server/src/handlers/lifecycle.rs` | Include fertility in frame building |
| `v3/crates/v3-server/src/handlers/status.rs` | Propagate config patch to `FoodResource` |
| `v3/crates/v3-server/src/transport/view_assembler.rs` | Include fertility in view payloads |
| `frontend/src/types/config.ts` | Add `FertilityConfig`, `AnnealingConfig` to `FoodConfig` |
| `frontend/src/stores/simulation.ts` | Store fertility grid data |
| `frontend/src/canvas/renderer.ts` | Render fertility overlay |

## Stage Overview

| Stage | Companion File | Scope |
|-------|---------------|-------|
| 0 | (this file) | Setup: worktree, baseline, scope freeze |
| 1 | `stage-1-backend-core.md` | FoodResource abstraction, fertility seeding, annealing, growth integration |
| 2 | `stage-2-server.md` | Server projection, transport, endpoints |
| 3 | `stage-3-frontend.md` | Protocol types, store, overlay rendering |
| 4 | (this file) | Docs, final verification, review gates |

---

## Stage 0: Setup

- [ ] Create worktree and branch (`codex/food-fertility-v1`)
- [ ] Run: `cargo test --workspace` — capture baseline (all tests should pass)
- [ ] Run: `cd frontend && npm run build` — capture baseline
- [ ] Freeze scope: this plan implements the spec at `docs/superpowers/specs/2026-03-18-food-fertility-layer-design.md` exactly

**See also:** `stage-1-backend-core.md` for Stage 1 tasks

---

## Stage 4: Docs and Final Verification

**Parent plan:** This file

- [ ] Update `docs/reference/v3-world-grid-spec.md`: add fertility layer section, reconcile config defaults (growth_rate: 0.05, spread_threshold_ratio: 0.8, recovery_spawn_rate: 0.01, recovery_floor_ratio: 0.01)
- [ ] Update startup seeding spec for fertility generation
- [ ] Update server projection and API protocol specs for `food_fertility_u8`

### Final Verification

- [ ] Run: `cargo test -p v3-core --test viability` (fertility disabled — production defaults)
- [ ] Run: `cargo test -p v3-core --test viability` (fertility + annealing enabled — `SimulationConfig::default()` with only `fertility.enabled = true` and `annealing.enabled = true`)
- [ ] Run: `cargo test --workspace`
- [ ] Run: `cargo test -p v3-server`
- [ ] Run: `cd frontend && npm run build && npm test`
- [ ] Run: profiling comparison — `cargo run --release -p v3-cli -- profile-ticks` with fertility off vs on

### Final Review Gates

- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `rust-skills` (backend) and Vercel skills (frontend). Fix all findings. Re-dispatch. Repeat until zero new findings.
- [ ] Review Gate (Architecture): Dispatch `superpowers:code-reviewer` for architecture and decomposition review — verify boundaries, dependency directions, separation of concerns vs `docs/strategy/`. Fix all. Re-dispatch until clean.
- [ ] Re-run `cargo test --workspace` and `cd frontend && npm test` after any review-introduced changes

---

## Acceptance Criteria

1. Fertility disabled path matches current behavior exactly (regression-safe)
2. Fertility generation is deterministic given the same seed
3. All algorithm outputs normalized to `[-1, 1]` before mapping
4. Fertility affects all food growth operations: proportional, spread, recovery spawn
5. Annealing correctly lerps fertility range over configured ticks
6. `FoodResource` abstraction contains all food state and logic; `WorldState` delegates
7. `food_fertility_u8` present in world-static transport
8. Frontend overlay toggleable and renders fertility grid
9. Viability tests pass with fertility disabled (backwards compatibility)
10. Viability tests pass with fertility + annealing enabled (founders survive)
