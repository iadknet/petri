# V3 Gap Backlog (2026-02-22)

Status source: Stage 1-3b evidence matrices in `docs/plans/reconciliation/`.

## Spec-Correction Gaps

| gap_id | stage | description | severity | required_fix | validation_command | owner |
| --- | --- | --- | --- | --- | --- | --- |
| GAP-SPEC-001 | 3b | Historical master references "21 operators" while implemented `GraphNodeKind` surface is 22. | Medium | Keep recovered file archival-only; ensure canonical reconciliation master and future active docs consistently state 22 operators. | `rg -n "21 operators|22 operators|GraphNodeKind" docs/plans docs/reference v3/crates/v3-core/src/creature/genome.rs` | v3-docs |
| GAP-SPEC-002 | 3a | Historical master references `max_vm_steps` default 256, but stage docs and runtime config use 1024. | Medium | Canonical reconciliation master defines 1024 as implemented baseline; follow-up spec correction if any remaining active docs mention 256. | `rg -n "max_vm_steps" docs/plans docs/reference v3/crates/v3-core/src/config/simulation.rs` | v3-docs |

## Implementation-Missing Gaps

No `MISSING` or `PARTIAL` checklist items were found for audited stages 1, 2, 3a, 3b.

## Audit-Scope Gaps

| gap_id | stage | description | severity | required_fix | validation_command | owner |
| --- | --- | --- | --- | --- | --- | --- |
| GAP-AUDIT-001 | 4-7 | Stages 4-7 were intentionally not audited in this pass and remain non-canonical for completion tracking. | High | Run the same evidence-gated reconciliation workflow for stages 4-7 before marking any of them done in canonical status docs. | `rg -n "\| \*\*4\*\*|\| \*\*5\*\*|\| \*\*6\*\*|\| \*\*7\*\*" docs/plans/2026-02-22-v3-implementation-reconciliation-master-plan.md` | v3-planning |
