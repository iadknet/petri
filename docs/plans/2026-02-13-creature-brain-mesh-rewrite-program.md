# Petri V2 Greenfield Program (Checkpoint-Driven Fresh Start)

> **Program Type:** This is a **MAJOR GREENFIELD REWRITE** program. Existing runtime code is reference-only.

**Goal:** Build a new `v2` simulation app from scratch, optimized for mesh-DNA cognition and emergent behavior, without refactoring or integrating legacy runtime code.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Create and ship a fully isolated app under `v2/` (`core`, `server`, `cli`, `web`) with checkpoint-gated execution plans; excludes in-place edits to legacy runtime/server/web codepaths.
**Docs Impact:** Update greenfield program/stage planning around explicit checkpoints and archive stale legacy-only active plans.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: greenfield architecture removes legacy controller constraints and centers richer cognition.
- `GP-02`: `v2/` boundaries keep policy ownership clean across runtime/transport/UI surfaces.
- `GP-03`: checkpoint gates force stable, test-backed progress before advancing.
- `GP-04`: observability is planned per stage instead of retrofitted after behavior churn.

## Boundary Impact

- Dependency direction in `v2` remains `v2-core -> v2-server/v2-cli`, with `v2-web` consuming `v2-server` contracts only.
- Legacy crates (`petri-core`, `petri-server`, `petri-cli`, `web`) stay reference-only and are never imported by `v2`.
- Reuse policy is copy-only: any borrowed implementation is copied into `v2`, adapted, and tested locally.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/*` legacy runtime | keep | Legacy app remains historical/reference while `v2` evolves independently. |
| `v2/` root ownership | keep | All rewrite implementation work remains isolated under `v2/`. |
| active `docs/plans/*` set | change | Active plans are checkpoint-scoped so execution state and ownership stay explicit. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should the rewrite continue as in-place refactor work? | No; all implementation stays greenfield under `v2/`. | user+agent | resolved |
| Should we optimize for checkpoint boundaries over monolithic plans? | Yes; stage work is broken into checkpoint slices with go/stop rules. | user+agent | resolved |
| Should stale legacy-oriented active plans remain in `docs/plans/`? | No; move stale plans into `docs/plans/archive/`. | user+agent | resolved |

## Checkpoint Authority

Checkpoint contract source of truth:
- `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`

Execution plans by checkpoint owner:
- `CP-0`: `docs/plans/2026-02-13-creature-brain-mesh-stage-1-runtime-kernel.md`
- `CP-1`: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`
- `CP-2`: `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`
- `CP-3`: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

## Current Status Snapshot

| checkpoint | status | required next move |
| --- | --- | --- |
| `CP-0 Greenfield Bootstrap` | complete | none |
| `CP-1 Runtime Kernel` | in progress | finish integrated runtime executor semantics in `v2-core` |
| `CP-2 Evolution + Ecology` | pending | begin only after `CP-1` exit criteria are green |
| `CP-3 Product Surface + Stabilization` | pending | begin only after `CP-2` exit criteria are green |

## Stage Breakdown

### Stage 1: Foundation and Guardrails (`CP-0`)

Plan file:
- `docs/plans/2026-02-13-creature-brain-mesh-stage-1-runtime-kernel.md`

Deliverables:
1. Isolated `v2/` workspace/tooling and app skeleton.
2. Copy-only/anti-coupling guardrail docs.
3. Baseline compile/build verification for `v2`.

### Stage 2: Mesh Runtime Kernel and Backends (`CP-1`)

Plan file:
- `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`

Deliverables:
1. Mesh schema validation + FIFO queue kernel.
2. Energy metering primitives and backend contracts.
3. Integrated runtime executor semantics for world-action halt and energy exhaustion.

### Stage 3: Evolution and Ecology (`CP-2`)

Plan file:
- `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`

Deliverables:
1. Strong asexual mutation engine with duplication operators.
2. Ecology pressure systems as the novelty driver.
3. Non-collapse baseline test scenarios.

### Stage 4: Product Surface and Stabilization (`CP-3`)

Plan file:
- `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

Deliverables:
1. `v2-server`, `v2-cli`, and `v2-web` parity and protocol stability.
2. End-to-end verification gates across `v2`.
3. Canonical docs rebased to `v2` target architecture.

## Cross-Stage Handoff Rules

1. Implementation edits occur under `v2/` only, except plan/doc updates.
2. Stage starts require prior checkpoint `go` evidence captured in commit notes.
3. Any intentional legacy code reuse must be copied into `v2` and documented.
4. Scope expansion requires plan update before code changes.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`
4. Stage-local `v2` verification commands listed in each stage plan

## Risks and Rollback

- Risk: parallel evolution of legacy and `v2` docs can drift if plan lifecycle is not maintained.
- Risk: unfinished checkpoint exit criteria can create downstream stage churn.
- Risk: policy drift can reintroduce accidental legacy coupling.
- Rollback strategy:
1. Keep stage commits atomic and checkpoint-labeled.
2. If a checkpoint fails its stop conditions, revert checkpoint-local commits and re-plan before continuing.
