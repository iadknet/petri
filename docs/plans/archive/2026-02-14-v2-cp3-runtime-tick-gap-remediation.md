# Petri V2 CP-3 Runtime Tick Gap Remediation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace food-only server tick behavior with deterministic creature tick behavior (actions, energy drain, and death) backed by `v2-core` runtime state.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** `v2/crates/v2-core` world tick primitives and `v2/crates/v2-server` tick-loop integration/tests; excludes protocol schema changes and frontend UI work.
**Docs Impact:** Adds a focused CP-3 remediation plan; no canonical strategy docs updated in this slice.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: creatures should exhibit runtime behavior beyond static startup snapshots.
- `GP-02`: simulation policy moves to `v2-core` instead of remaining synthetic in `v2-server`.
- `GP-03`: close behavior gap with explicit red-green tests.
- `GP-04`: status/frame telemetry must reflect actual creature dynamics.

## Boundary Impact

- Add `v2-core` world tick/state module for deterministic creature+food evolution.
- Keep `v2-server` responsible for transport/lifecycle orchestration and payload mapping.
- Preserve existing `v2alpha1` payload shapes and endpoint contracts.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-server/src/state.rs` | change | Current `advance_ticks` mutates only food and zeros action counts. |
| `v2/crates/v2-core/src/runtime.rs` | keep | CP-1 mesh runtime contract remains intact; this remediation adds world tick integration beside it. |
| `v2/crates/v2-server/src/api.rs` | keep | API surface/schema stay stable while backend state semantics become real. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should this slice implement full mesh-genome execution in server ticks? | No; implement deterministic minimal creature policy first, preserving room for deeper runtime wiring later. | user+agent | resolved |
| Should protocol payload fields change in this slice? | No; keep schema stable and only change value truthfulness. | user+agent | resolved |
| Should startup viability probe be rewritten now? | No; keep current startup viability guard and focus this slice on per-tick runtime behavior. | user+agent | resolved |

## Task List

### Task 1: Add failing behavior tests (red)

Files:
- Modify: `v2/crates/v2-server/tests/tick_dynamics.rs`
- Create: `v2/crates/v2-core/tests/world_state.rs`

Steps:
1. Add failing server test asserting running ticks mutate creature state (position and/or energy), not only food/tick.
2. Add failing server test asserting `last_action_counts` reflects applied tick actions (not all zero).
3. Add failing core world-state test asserting tick decay/death behavior occurs deterministically under no-food conditions.

### Task 2: Implement core world tick primitives (green)

Files:
- Create: `v2/crates/v2-core/src/world_state.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`
- Modify: `v2/crates/v2-core/tests/world_state.rs`

Steps:
1. Add core world-state data types and deterministic tick transition function.
2. Implement per-tick creature behavior with energy decay, deterministic movement/eat/noop actions, death removal, and food growth.
3. Return per-tick action/death counters and expose helper methods needed by server.

### Task 3: Integrate server tick loop with core world state

Files:
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/tick_dynamics.rs`

Steps:
1. Replace food-only `advance_ticks` logic with `v2-core` world tick calls.
2. Update status-derived counters (`mean_energy`, `population`, births/deaths window, `last_action_counts`) from real tick outputs.
3. Keep startup and frame payload schema unchanged while switching values to real runtime state.

### Task 4: Verify and close slice

Files:
- Modify: `docs/plans/2026-02-14-v2-cp3-fine-grained-execution-checklist.md`

Steps:
1. Run targeted gates and confirm pass.
2. Update checklist wording/status to reflect actual runtime behavior guarantees.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test world_state`
3. `cd v2 && cargo test -p v2-server --test tick_dynamics`
4. `cd v2 && cargo test -p v2-server`

## Risks and Rollback

- Risk: introducing world tick semantics may invalidate assumptions in status telemetry tests.
- Risk: deterministic movement policy may create brittle assertions if tests overfit exact coordinates.
- Rollback:
1. Revert this remediation slice.
2. Keep failing tests and re-implement with narrower per-behavior steps.
