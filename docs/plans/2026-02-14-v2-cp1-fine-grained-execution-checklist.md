# Petri V2 CP-1 Fine-Grained Execution Checklist

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete CP-1 using one atomic runtime behavior slice at a time, with explicit red-green-verify-commit checkboxes per slice.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2/crates/v2-core` runtime integration (`mesh`, `energy`, `backends`, `runtime`) and CP-1 test gates; excludes ecology/mutation and product-surface work.
**Docs Impact:** Adds a checklist-driven CP-1 execution plan that references the CP-1 runtime/spec contracts and the shared command matrix.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: closes the core execution substrate with deterministic runtime behavior.
- `GP-03`: enforces strict checkpoint progress through tiny red-green slices.
- `GP-04`: locks inspectable runtime outcomes and error semantics.

## Boundary Impact

- Scope is `v2-core` only.
- Runtime policy remains in `v2-core`; no transport/UI coupling.
- Schema/ISA ownership remains aligned with `mesh.rs`, `backends.rs`, and `runtime.rs` boundaries.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/runtime.rs` | change | CP-1 completion requires final integration semantics and error mapping. |
| `v2/crates/v2-core/src/backends.rs` | keep | Backend contract exists; slices should only tighten composition boundaries. |
| `v2/crates/v2-core/tests/*.rs` | change | CP-1 closeout requires fixture-locked runtime edge coverage. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should this checklist replace CP-1 runtime/spec docs? | No; this checklist executes those specs in smaller slices. | user+agent | resolved |
| Should one slice include multiple runtime behaviors? | No; one behavior seam per slice, one commit per slice. | user+agent | resolved |
| Should CP-1 close before full matrix gate passes? | No; CP-1 closes only after full CP-1 command gate is green. | user+agent | resolved |

## Checklist Operating Rules

1. Keep exactly one slice `in progress` at a time.
2. Use red-green when behavior is missing; if behavior already matches intent, add/adjust regression coverage and record pass evidence instead of forcing artificial failure.
3. Keep each slice to one behavior seam and one commit.
4. Run only v2 tests.
5. If a checklist step conflicts with architecture boundaries or project goals, stop and ask for guidance.
6. Update checklist boxes in this file in the same commit as code changes.

## Task List

### Slice K1: Runtime outcome contract lock

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`

**Checklist:**
- [x] Verify tests cover `CommittedAction`, `ImplicitNoOp`, `EnergyExhausted`, and `RuntimeError` shape/fields.
- [x] Run: `cd v2 && cargo test -p v2-core --test mesh_runtime` and confirm behavior gate coverage.
- [x] Verify `runtime.rs` implementation populates and returns the expected outcome variants/fields.
- [x] Re-run targeted test and confirm pass.
- [x] Commit slice `K1`.

### Slice K2: Dispatch entry charge and FIFO seed behavior

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_kernel.rs`

**Checklist:**
- [x] Verify tests cover entry charge order and deterministic queue seeding from `entry_node_id`.
- [x] Run: `cd v2 && cargo test -p v2-core --test mesh_runtime` and confirm behavior gate coverage.
- [x] Verify queue+entry-charge implementation in `runtime.rs` (`entry_node_id` seed, FIFO queue, entry charge before backend dispatch).
- [x] Re-run: `cd v2 && cargo test -p v2-core --test mesh_runtime`.
- [x] Re-run: `cd v2 && cargo test -p v2-core --test mesh_kernel`.
- [x] Commit slice `K2`.

### Slice K3: Backend compute metering composition

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/src/backends.rs`
- Modify: `v2/crates/v2-core/tests/mesh_energy.rs`
- Modify: `v2/crates/v2-core/tests/mesh_backends.rs`

**Checklist:**
- [x] Verify tests cover runtime-owned ledger application and backend compute metadata usage.
- [x] Run: `cd v2 && cargo test -p v2-core --test mesh_energy` and confirm behavior gate coverage.
- [x] Verify runtime/backends contract implementation for energy ledger ownership and backend compute metering.
- [x] Re-run: `cd v2 && cargo test -p v2-core --test mesh_energy`.
- [x] Re-run: `cd v2 && cargo test -p v2-core --test mesh_backends`.
- [x] Commit slice `K3`.

### Slice K4: Emitted output ordering and invalid metadata precedence

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`

**Checklist:**
- [x] Add/adjust regression test where invalid world-action metadata precedes a later valid world action.
- [x] Run: `cd v2 && cargo test -p v2-core --test mesh_runtime` and confirm the behavior gate covers emitted ordering precedence.
- [x] Verify `runtime.rs` preserves emitted output order and returns `RuntimeError::InvalidActionMetadata` before any later valid world action can commit.
- [x] Re-run targeted test and confirm pass.
- [x] Commit slice `K4`.

### Slice K5: First valid world-action commit and immediate halt

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_kernel.rs`

**Checklist:**
- [x] Add/adjust regression test for immediate halt after first valid committed action (including queued internal dispatch that must not execute).
- [x] Run: `cd v2 && cargo test -p v2-core --test mesh_runtime` and confirm behavior gate coverage.
- [x] Verify `runtime.rs` halts on the first valid committed world action and does not continue queued dispatches.
- [x] Re-run: `cd v2 && cargo test -p v2-core --test mesh_runtime`.
- [x] Re-run: `cd v2 && cargo test -p v2-core --test mesh_kernel`.
- [x] Commit slice `K5`.

### Slice K6: Implicit no-op and exhaustion outcomes

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_energy.rs`

**Checklist:**
- [ ] Add/adjust failing tests for queue-drain no-op and exhaustion termination behavior.
- [ ] Run: `cd v2 && cargo test -p v2-core --test mesh_runtime` and confirm failure is expected.
- [ ] Implement minimal outcome-path fix.
- [ ] Re-run: `cd v2 && cargo test -p v2-core --test mesh_runtime`.
- [ ] Re-run: `cd v2 && cargo test -p v2-core --test mesh_energy`.
- [ ] Commit slice `K6`.

### Slice K7: Runtime config validation (`sensor_radius >= 1`)

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`
- Modify: `v2/crates/v2-core/tests/sensor_radius_global_config.rs`

**Checklist:**
- [ ] Add/adjust failing test for `sensor_radius=0` rejection.
- [ ] Run: `cd v2 && cargo test -p v2-core --test sensor_radius_global_config` and confirm failure is expected.
- [ ] Implement minimal runtime config guard.
- [ ] Re-run: `cd v2 && cargo test -p v2-core --test sensor_radius_global_config`.
- [ ] Re-run: `cd v2 && cargo test -p v2-core --test mesh_runtime`.
- [ ] Commit slice `K7`.

### Slice K8: VM fault mapping and graph state persistence cross-check

**Files:**
- Modify: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/src/backends.rs`
- Modify: `v2/crates/v2-core/tests/mesh_runtime.rs`
- Modify: `v2/crates/v2-core/tests/graph_stateful_ops.rs`

**Checklist:**
- [ ] Add/adjust failing tests for hard VM faults vs soft `ReadInput` default and graph state persistence.
- [ ] Run: `cd v2 && cargo test -p v2-core --test mesh_runtime` and confirm failure is expected.
- [ ] Implement minimal fault-mapping/state-persistence fix.
- [ ] Re-run: `cd v2 && cargo test -p v2-core --test mesh_runtime`.
- [ ] Re-run: `cd v2 && cargo test -p v2-core --test graph_stateful_ops`.
- [ ] Commit slice `K8`.

### Slice K9: CP-1 gate closeout

**Files:**
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`
- Modify: `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`

**Checklist:**
- [ ] Run full CP-1 gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md`.
- [ ] Record pass evidence in commit notes/body.
- [ ] Update checkpoint status only if gate is fully green.
- [ ] Commit slice `K9`.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. Run the `CP-1` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-1 exit`).

## Risks and Rollback

- Risk: broad fixes can blur behavior boundaries and invalidate slice-level trust.
- Risk: skipping red-first checks can hide false-positive "complete" status.
- Rollback:
1. Revert only the failing slice commit.
2. Re-introduce the red test for that slice.
3. Re-implement with narrower file scope.
