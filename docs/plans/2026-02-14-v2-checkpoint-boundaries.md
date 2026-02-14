# Petri V2 Checkpoint Boundary Map

**Goal:** Define explicit checkpoint entry/exit boundaries for the greenfield `v2` rewrite so execution can pause/resume safely without stage ambiguity.
**Goal IDs:** GP-02, GP-03
**Scope:** Planning-only checkpoint map, ownership, and go/stop criteria; excludes runtime code changes.
**Docs Impact:** Add checkpoint authority plan; align program/stage plans to it; archive stale active plans that do not match the greenfield direction.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-02`: checkpoint ownership makes architecture boundaries enforceable during handoffs.
- `GP-03`: explicit gates prevent progress claims without passing targeted verification.

## Boundary Impact

- No runtime crate boundaries change in this plan; impact is on planning boundaries only.
- Program and stage plans consume this document as checkpoint source of truth.
- Active plan inventory is tightened to `v2`-aligned work only.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md` | change | Program plan should reference a single checkpoint authority map. |
| `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md` | change | Stage 2 needs sub-checkpoint slices reflecting completed and pending work. |
| `docs/plans/2026-02-13-phenotype-rgb-evolution.md` | change | Legacy in-place plan no longer matches the greenfield rewrite direction. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should checkpoint status live in one place or be duplicated in every plan? | One source-of-truth map here, with brief references in stage plans. | user+agent | resolved |
| Should completed checkpoints remain active plans? | Keep stage plans active until program close, but mark status explicitly. | user+agent | resolved |
| Should stale pre-greenfield active plans be archived now? | Yes. | user+agent | resolved |

## Checkpoint Ledger

| checkpoint | owner plan | status | entry gate | exit gate |
| --- | --- | --- | --- | --- |
| `CP-0 Greenfield Bootstrap` | `docs/plans/2026-02-13-creature-brain-mesh-stage-1-runtime-kernel.md` | complete | Program doc declares greenfield isolation and copy-only policy | `v2` skeleton compiles/builds and boundary docs exist |
| `CP-1 Runtime Kernel` | `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md` | in progress | `CP-0` complete | Queue + energy + backend integration tests green |
| `CP-2 Evolution + Ecology` | `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md` | pending | `CP-1` complete and semantics frozen | Mutation invariants and non-collapse ecology checks green |
| `CP-3 Product Surface + Stabilization` | `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md` | in progress | `CP-2` complete | `v2-server/cli/web` integration gates and docs rebaseline green |

## Checkpoint Specs

| checkpoint | implementation spec |
| --- | --- |
| `CP-1 Runtime Kernel` | `docs/plans/2026-02-14-v2-cp1-runtime-execution-spec.md` |
| `CP-1 Runtime Kernel` (schema/ISA) | `docs/plans/2026-02-14-v2-core-schema-vm-isa-spec.md` |
| `CP-1 Runtime Kernel` (fine-grained checklist) | `docs/plans/2026-02-14-v2-cp1-fine-grained-execution-checklist.md` |
| `CP-2 Evolution + Ecology` | `docs/plans/2026-02-14-v2-cp2-evolution-ecology-spec.md` |
| `CP-2 Evolution + Ecology` (fine-grained checklist) | `docs/plans/2026-02-14-v2-cp2-fine-grained-execution-checklist.md` |
| `CP-3 Product Surface + Stabilization` | `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md` |
| `CP-3 Product Surface + Stabilization` (backend execution) | `docs/plans/2026-02-14-v2-cp3-backend-micro-implementation-plan.md` |
| `CP-3 Product Surface + Stabilization` (frontend execution) | `docs/plans/2026-02-14-v2-cp3-frontend-micro-implementation-plan.md` |
| `CP-3 Product Surface + Stabilization` (fine-grained checklist) | `docs/plans/2026-02-14-v2-cp3-fine-grained-execution-checklist.md` |

Shared test gate spec:
- `docs/plans/2026-02-14-v2-implementation-test-matrix.md`
- `docs/plans/2026-02-14-v2-fine-grained-execution-index.md`

## Task List

### Task 1: Program plan alignment

Files:
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`

Steps:
1. Reference this checkpoint map as the checkpoint authority.
2. Add status snapshot (`complete`, `in progress`, `pending`).
3. Keep stage ownership explicit per checkpoint.

### Task 2: Stage 2 checkpoint slice alignment

Files:
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`

Steps:
1. Separate Stage 2 into slice-level checkpoint blocks (`CP-1A`..`CP-1D`).
2. Mark already completed slices and define remaining slice entry/exit gates.
3. Align file targets with current `v2-core` layout.

### Task 3: Add per-checkpoint implementation specs

Files:
- Create: `docs/plans/2026-02-14-v2-cp1-runtime-execution-spec.md`
- Create: `docs/plans/2026-02-14-v2-core-schema-vm-isa-spec.md`
- Create: `docs/plans/2026-02-14-v2-cp2-evolution-ecology-spec.md`
- Create: `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`
- Create: `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

Steps:
1. Define runtime, evolution/ecology, and API/protocol contracts at implementation detail level.
2. Define a shared verification matrix used by all checkpoint plans.
3. Update stage plans to reference these specs.

### Task 4: Retire stale active plan

Files:
- Move: `docs/plans/2026-02-13-phenotype-rgb-evolution.md`
- To: `docs/plans/archive/2026-02-13-phenotype-rgb-evolution.md`

Steps:
1. Remove the stale legacy plan from active plan inventory.
2. Preserve it in archive for historical traceability.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`

## Risks and Rollback

- Risk: checkpoint status can get stale if updates are not part of each stage commit.
- Risk: stale plans may be copied forward accidentally if not archived.
- Rollback:
1. Revert checkpoint-map and plan-alignment commit.
2. Restore previous active plan set and retry with a smaller planning slice.
