# Petri V2 Fine-Grained Execution Index

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Provide a single checklist-driven execution order for CP-1, CP-2, and CP-3 so work advances one small verified slice at a time.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Planning-only execution index and order-of-operations for checklist plans; excludes direct runtime/frontend code changes.
**Docs Impact:** Adds one source-of-truth index for checklist plans and stage execution order.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: keeps implementation momentum focused on behavior-complete slices.
- `GP-02`: preserves checkpoint ownership and boundary discipline.
- `GP-03`: enforces explicit verification and commit gates per slice.
- `GP-04`: ensures product-surface stability is built on verified core/runtime behavior.

## Boundary Impact

- No crate/module code boundary changes.
- Adds planning boundary: one active checklist slice at a time across all checkpoints.
- Stage/checkpoint status updates must follow gate evidence from the matrix.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/plans/2026-02-14-v2-checkpoint-boundaries.md` | change | Should reference checklist execution docs as operational plans. |
| `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md` | change | Program execution dependencies should include checklist-based plans. |
| `docs/plans/2026-02-14-v2-implementation-test-matrix.md` | keep | Remains command gate source of truth; checklist plans consume it. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should execution jump between checkpoints in parallel? | No; execute checkpoint order `CP-1 -> CP-2 -> CP-3`. | user+agent | resolved |
| Should multiple slices be in progress simultaneously? | No; exactly one slice in progress at any time. | user+agent | resolved |
| Should checklist plans duplicate full command matrices? | No; all plans reference the shared matrix DRY source. | user+agent | resolved |

## Intent-First Execution Directive

1. Functional and architectural intent are the primary success criteria for each slice.
2. If any checklist item conflicts with architecture boundaries or project goals, stop and ask for guidance before continuing.
3. Prefer red-green when behavior is missing; if behavior is already correct, execute a regression-hardening verification slice and capture evidence.
4. Keep checklist wording synchronized with what was actually verified and implemented.

## Task List

### Task 1: Execute CP-1 checklist slices

Files:
- Modify: `docs/plans/2026-02-14-v2-cp1-fine-grained-execution-checklist.md`
- Modify: `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`

Steps:
1. Complete slices `K1`..`K9` in order, one commit per slice.
2. Keep checklist boxes up to date in the same commit as implementation.
3. Update checkpoint/program status only after full `CP-1` gate is green.

### Task 2: Execute CP-2 checklist slices

Files:
- Modify: `docs/plans/2026-02-14-v2-cp2-fine-grained-execution-checklist.md`
- Modify: `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`

Steps:
1. Complete slices `E1`..`E9` in order, one commit per slice.
2. Keep checklist boxes up to date in the same commit as implementation.
3. Update checkpoint/program status only after full `CP-2` gate is green.

### Task 3: Execute CP-3 checklist slices

Files:
- Modify: `docs/plans/2026-02-14-v2-cp3-fine-grained-execution-checklist.md`
- Modify: `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`

Steps:
1. Complete slices `P3-B1`..`P3-B6`, `P3-F1`..`P3-F7`, and `P3-X1` in order.
2. Keep checklist boxes up to date in the same commit as implementation.
3. Update checkpoint/program status only after full `CP-3` gate is green.

### Execution Order

- [ ] Complete `CP-1` checklist plan: `docs/plans/2026-02-14-v2-cp1-fine-grained-execution-checklist.md`
- [ ] Complete `CP-2` checklist plan: `docs/plans/2026-02-14-v2-cp2-fine-grained-execution-checklist.md`
- [ ] Complete `CP-3` checklist plan: `docs/plans/2026-02-14-v2-cp3-fine-grained-execution-checklist.md`

### Per-Slice Gate (apply to every slice)

- [ ] Mark one slice as `in progress`.
- [ ] Add/adjust targeted test coverage for exactly one behavior seam (failing when a real gap exists).
- [ ] Run targeted test and capture evidence of failure or already-correct behavior.
- [ ] Implement needed fix, or explicitly verify existing implementation if already correct.
- [ ] Re-run targeted test and confirm pass.
- [ ] Run required local regression tests for touched seam.
- [ ] Commit only that slice.
- [ ] Update checklist status boxes in the same commit.

### Checkpoint Gate (apply at each checkpoint exit)

- [ ] Run checkpoint command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md`.
- [ ] Confirm all gate commands are green.
- [ ] Update checkpoint status in `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`.
- [ ] Update status snapshot in `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`.
- [ ] Commit checkpoint-closeout status update.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. For implementation slices, run the corresponding checkpoint gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md`.

## Risks and Rollback

- Risk: partial checklist updates can create false confidence about checkpoint status.
- Risk: skipping the red-first step can hide behavior gaps.
- Rollback:
1. Revert only the slice/closeout commit that violated the gate flow.
2. Restore checklist state and rerun the required gate.
