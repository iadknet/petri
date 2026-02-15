# Runtime Behavior Realism Guidance Hardening

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Promote runtime behavior realism from checkpoint plan intent into canonical project guidance and required verification surfaces.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Canonical docs and AGENTS guidance updates plus shared matrix requirement updates; excludes runtime code implementation changes.
**Docs Impact:** Adds one canonical standards policy and updates architecture/index/AGENTS/matrix guidance to reference it.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: guards against inert creature/runtime behavior being treated as complete.
- `GP-02`: reinforces policy ownership in `v2-core` and transport mapping in `v2-server`.
- `GP-03`: adds explicit regression gate expectations for behavior realism.
- `GP-04`: makes telemetry truthfulness a project-level requirement.

## Boundary Impact

- Documentation-only boundary impact.
- No crate/module code boundary changes.
- Clarifies that runtime truthfulness is a core/transport contract boundary rule.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/strategy/architecture.md` | change | Must state runtime truthfulness invariant as architecture policy, not only in plans. |
| `AGENTS.md` and local `v2/*/AGENTS.md` | change | Must include non-negotiable runtime realism requirements for agent execution. |
| `docs/plans/2026-02-14-v2-implementation-test-matrix.md` | change | Must include explicit CP-3 behavior-realism gates to avoid contract-only false positives. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should behavior-realism requirements be plan-only? | No; they must be canonical policy and AGENTS guidance. | user+agent | resolved |
| Should this change add new harness code now? | No; policy + matrix + AGENTS guidance first, harness enforcement can follow separately. | user+agent | resolved |
| Should scope include runtime code changes in this plan? | No; this plan is guidance hardening only. | user+agent | resolved |

## Task List

### Task 1: Add canonical behavior-realism standard

Files:
- Create: `docs/standards/runtime-behavior-realism-policy.md`

Steps:
1. Define runtime truthfulness rules (no synthetic per-tick creature/telemetry placeholders).
2. Define boundary ownership (`v2-core` policy, `v2-server` mapping).
3. Define required verification expectations for CP-3+ work.

### Task 2: Update AGENTS guidance requirements

Files:
- Modify: `AGENTS.md`
- Modify: `v2/AGENTS.md`
- Modify: `v2/crates/v2-core/AGENTS.md`
- Modify: `v2/crates/v2-server/AGENTS.md`

Steps:
1. Add non-negotiable runtime realism requirement in root/project guidance.
2. Add local `v2` and crate-level guardrails that forbid synthetic tick telemetry.
3. Reference the canonical standards doc as source-of-truth.

### Task 3: Update architecture/index canonical docs

Files:
- Modify: `docs/strategy/architecture.md`
- Modify: `docs/README.md`

Steps:
1. Add architecture-level runtime truthfulness invariant.
2. Add standards index entry for the new policy.

### Task 4: Update shared checkpoint matrix requirements

Files:
- Modify: `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

Steps:
1. Add explicit CP-3 behavior-realism suite requirements.
2. Add explicit CP-3 pass-criteria language for real creature/action/energy/death dynamics.
3. Keep command-gate DRY and aligned with matrix ownership rules.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`

## Risks and Rollback

- Risk: requirements stated in docs but not yet mechanically lint-enforced can drift.
- Risk: adding too much detail in AGENTS can duplicate checklist-level guidance.
- Rollback:
1. Revert guidance-hardening commit.
2. Re-apply with narrower canonical references and less duplication.
