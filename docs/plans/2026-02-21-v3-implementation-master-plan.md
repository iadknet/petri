# V3 High-Level Implementation Master Plan (Reconciled 2026-02-22)

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` for implementation work and keep status checkmarks synchronized between this master plan and the active stage subplan.

**Goal:** Keep a single, current source of truth for V3 implementation status across stages and subplans.

**Goal IDs:** GP-02, GP-03

**Scope:** Master-plan status management and stage tracking for V3 implementation in `.worktrees/v3-implementation`.

**Docs Impact:** Restores active master plan at the original path with explicit status checkmarks, maintenance protocol, and links to stage evidence artifacts.

**Supersedes:** `docs/plans/archive/2026-02-21-v3-implementation-master-plan.recovered.md`, `docs/plans/archive/2026-02-22-v3-implementation-reconciliation-master-plan.md`

**Superseded-By:** none

---

## Goal Alignment

- **GP-02:** Maintains clean planning boundaries by keeping one canonical master plan with explicit stage status ownership.
- **GP-03:** Uses evidence-gated completion criteria so status reflects verified reality, not intent.

## Boundary Impact

- Canonical master plan path is restored to `docs/plans/2026-02-21-v3-implementation-master-plan.md`.
- Stage subplans remain the detailed task-level source for checklist completion.
- Evidence matrices in `docs/plans/archive/reconciliation/` remain supporting artifacts for verification traceability.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/plans/2026-02-21-v3-implementation-master-plan.md` | change | Reinstated as canonical master status file with active checkmark tracking. |
| `docs/plans/2026-02-21-v3-stage-*.md` | keep | Subplans remain task-level execution and checklist detail. |
| `docs/plans/archive/reconciliation/*.md` | keep | Retained as evidence trail and divergence/gap analysis inputs. |
| `docs/plans/archive/*.md` | keep | Historical artifacts remain immutable references, not active status sources. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Which file is canonical for stage status? | This file (`docs/plans/2026-02-21-v3-implementation-master-plan.md`) is canonical. | user+agent | resolved |
| How are stage statuses marked complete? | Only with evidence-gated verification from subplans + tests/harness gates. | user+agent | resolved |
| How are historical/recovered plans handled? | Keep under `docs/plans/archive/` unchanged for provenance only. | user+agent | resolved |

---

## Stage Overview (Canonical Status)

| Status | Stage | Name | Active Subplan | Evidence Matrix |
| --- | --- | --- | --- | --- |
| `[x]` | 1 | Scaffolding + Contracts + Kernel | `docs/plans/2026-02-21-v3-stage-1-scaffolding-contracts-kernel.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-1-evidence-matrix.md` |
| `[x]` | 2 | Creature Schema | `docs/plans/2026-02-21-v3-stage-2-creature-schema.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-2-evidence-matrix.md` |
| `[x]` | 3a | Sensors + VM Backend | `docs/plans/2026-02-21-v3-stage-3a-sensors-vm.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-3a-evidence-matrix.md` |
| `[x]` | 3b | Graph Backend + Mesh Chain | `docs/plans/2026-02-21-v3-stage-3b-graph-mesh.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-3b-evidence-matrix.md` |
| `[x]` | 4 | Tick + Seeding + Viability E2E | `docs/plans/2026-02-21-v3-stage-4-tick-seeding-viability.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-4-evidence-matrix.md` |
| `[x]` | 5 | Mutation + Phenotype + Evolution | `docs/plans/2026-02-21-v3-stage-5-mutation-phenotype-evolution.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-5-evidence-matrix.md` |
| `[x]` | 6 | CLI + Server | `docs/plans/2026-02-21-v3-stage-6-cli-server.md` | `docs/plans/archive/reconciliation/2026-02-22-v3-stage-6-evidence-matrix.md` |
| `[x]` | 7 | Observability + Hardening | `docs/plans/2026-02-23-v3-stage-7-observability-hardening.md` | `docs/plans/archive/reconciliation/2026-02-23-v3-stage-7-evidence-matrix.md` |

---

## Checkmark Maintenance Protocol (Required)

1. `Subplan first`: update task-level checkboxes in the relevant stage subplan at the moment work is verified.
2. `Master second`: update the stage status checkmark in this master plan in the same commit where subplan status changes.
3. `Evidence required`: a stage may be `[x]` only when all stage checklist items are evidence-verified (code refs + tests/harness).
4. `Regression rule`: if a verified item regresses or becomes unclear, revert affected checklist lines to `[ ]` and set the master stage back to `[ ]` in the same commit.
5. `No stale merges`: do not merge implementation commits that change stage scope without corresponding master/subplan checkmark updates.
6. `Verification gate`: before claiming stage completion, run:
   - `scripts/check-plan-harness.sh --mode strict`
   - `scripts/check-doc-harness.sh --mode warn`
   - `scripts/check-architecture-harness.sh --mode warn`
   - `cd v3 && cargo test --workspace`
   - `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
   - `cd v3 && cargo fmt --all -- --check`

## Provenance and Timeline Anchors

- Recovered historical master write/edit window (Claude history):
  - `2026-02-21T23:59:36.899Z`
  - `2026-02-22T00:09:36.754Z`
  - `2026-02-22T00:09:50.999Z`
- Full divergence narrative:
  - `docs/plans/archive/reconciliation/2026-02-22-v3-divergence-timeline.md`

## Supporting Artifacts

- `docs/plans/archive/reconciliation/2026-02-22-v3-stage-1-evidence-matrix.md`
- `docs/plans/archive/reconciliation/2026-02-22-v3-stage-2-evidence-matrix.md`
- `docs/plans/archive/reconciliation/2026-02-22-v3-stage-3a-evidence-matrix.md`
- `docs/plans/archive/reconciliation/2026-02-22-v3-stage-3b-evidence-matrix.md`
- `docs/plans/archive/reconciliation/2026-02-22-v3-stage-4-evidence-matrix.md`
- `docs/plans/archive/reconciliation/2026-02-22-v3-gap-backlog.md`
- `docs/plans/archive/reconciliation/2026-02-22-v3-divergence-timeline.md`
