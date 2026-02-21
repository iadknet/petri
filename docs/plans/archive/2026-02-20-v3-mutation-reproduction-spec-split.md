# V3 Mutation/Reproduction Spec Split

**Goal:** Split V3 lifecycle details into dedicated mutation and reproduction specs with explicit responsibility boundaries and minimal evolution observability contracts.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Reference-doc and architecture-doc updates only. Includes new mutation/reproduction/observability specs, lifecycle doc slimming, and cross-reference synchronization. Excludes runtime code changes, API payload changes, and storage implementation changes.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/reference/v3-mutation-spec.md` | Create |
| `docs/reference/v3-reproduction-spec.md` | Create |
| `docs/reference/v3-evolution-observability-spec.md` | Create |
| `docs/reference/v3-creature-lifecycle-spec.md` | Refactor to overview/index |
| `docs/reference/v3-genome-spec.md` | Update cross-links |
| `docs/reference/v3-mesh-execution-spec.md` | Update cross-links |
| `docs/reference/v3-vm-isa-spec.md` | Update cross-links |
| `docs/reference/v3-graph-backend-spec.md` | Update cross-links |
| `docs/reference/v3-sensor-spec.md` | Update cross-links |
| `docs/README.md` | Update reference index |
| `docs/plans/2026-02-18-v3-mesh-refactor-design.md` | Update docs-impact/reference-spec links |

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-18-v3-mesh-refactor-design.md`

## Goal Alignment

- **GP-01:** Clarifies evolution mechanisms (mutation + reproduction boundaries) so implementation can evolve behavior without hidden policy coupling.
- **GP-02:** Explicitly separates ownership: mutation engine orchestration, domain-specific mutators, reproduction action flow, and observability contract boundaries.
- **GP-03:** Reduces ambiguity in failure-path policy (`rollback + skip`) and arbitration behavior (`first-wins`) to improve high-confidence implementation/testing.
- **GP-04:** Defines required minimal telemetry signals for mutation skips, reproduction rejections, and spawn outcomes.

## Boundary Impact

- No crate/module code boundary changes in this slice.
- Reference boundary changes:
  - `v3-creature-lifecycle-spec.md` becomes summary/index.
  - Mutation and reproduction normative details move to dedicated specs.
  - Observability contract is isolated in a dedicated spec so policy remains explicit without forcing storage/transport design.
- Architecture plan reference table is updated to keep canonical docs synchronized.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/reference/v3-creature-lifecycle-spec.md` | change | Lifecycle should describe phase-level semantics and link to dedicated mutation/reproduction contracts to avoid one-file policy sprawl. |
| `docs/reference/v3-mesh-execution-spec.md` | keep | Runtime soft-default/termination policy remains authoritative there; links to mutation/reproduction contracts are updated rather than duplicated. |
| `docs/reference/v3-vm-isa-spec.md` + `docs/reference/v3-graph-backend-spec.md` | keep | Backend execution semantics stay in backend specs; mutation ownership is referenced via updated links, not duplicated. |
| `docs/plans/2026-02-18-v3-mesh-refactor-design.md` | change | Architecture doc must list the new split reference docs to prevent canonical-doc drift. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should this slice include code changes? | No. Docs/contracts only. | user+agent | resolved |
| Should lifecycle stay monolithic? | No. Split into mutation + reproduction specs and keep lifecycle as overview/index. | user+agent | resolved |
| How should contested spawn cells be resolved? | First processed reproduce action against current world state wins; later contenders fail the same invalid-target gate (single rejection outcome). | user+agent | resolved |
| What should happen when a mutation event yields non-parseable genome? | Rollback event and skip it; record skip reason telemetry. | user+agent | resolved |
| Should determinism be runtime requirement? | No. Runtime determinism is not required; reproducibility guidance is test-mode only. | user+agent | resolved |
| Where should telemetry contract live? | Dedicated `v3-evolution-observability-spec.md` with minimal required schema. | user+agent | resolved |
| Diagram style for mutation/reproduction docs? | ASCII diagrams in new split specs. | user+agent | resolved |

## Implementation Checklist

### Task 1: Add mutation/reproduction/observability split specs

**Files:**
- Create: `docs/reference/v3-mutation-spec.md`
- Create: `docs/reference/v3-reproduction-spec.md`
- Create: `docs/reference/v3-evolution-observability-spec.md`

**Checklist:**
- [x] Create mutation spec with explicit engine vs domain mutator boundaries.
- [x] Create reproduction spec with immediate action-time resolution and
  first-processed-wins arbitration.
- [x] Create observability spec with minimal required counters/events/reasons.
- [x] Use ASCII diagrams in mutation and reproduction specs.

### Task 2: Convert lifecycle to overview/index and sync references

**Files:**
- Modify: `docs/reference/v3-creature-lifecycle-spec.md`
- Modify: `docs/reference/v3-genome-spec.md`
- Modify: `docs/reference/v3-mesh-execution-spec.md`
- Modify: `docs/reference/v3-vm-isa-spec.md`
- Modify: `docs/reference/v3-graph-backend-spec.md`
- Modify: `docs/reference/v3-sensor-spec.md`

**Checklist:**
- [x] Keep lifecycle phases and invariants summary.
- [x] Replace deep mutation/reproduction normative detail with concise summaries + links.
- [x] Add cross-links from active reference docs to new mutation/reproduction/observability specs.
- [x] Avoid policy duplication across reference docs.

### Task 3: Update top-level docs index and architecture plan links

**Files:**
- Modify: `docs/README.md`
- Modify: `docs/plans/2026-02-18-v3-mesh-refactor-design.md`

**Checklist:**
- [x] Add new reference docs to docs index.
- [x] Update architecture doc docs-impact and reference-spec tables to include split specs.
- [x] Keep architecture wording consistent with lifecycle overview role.

### Task 4: Verification and final status

**Files:**
- Modify: `docs/plans/archive/2026-02-20-v3-mutation-reproduction-spec-split.md`

**Checklist:**
- [x] Run doc harness warn mode.
- [x] Run architecture harness warn mode.
- [x] Verify new docs are discoverable via grep checks.
- [x] Confirm no `Determinism Requirements` sections remain in active v3 reference docs.
- [x] Mark checklist items complete and record review cycles.

## Verification Commands

- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-doc-harness.sh --mode warn`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-architecture-harness.sh --mode warn`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "v3-mutation-spec|v3-reproduction-spec|v3-evolution-observability-spec" docs`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "Determinism Requirements" docs/reference/v3-*.md`

## Risks and Rollback

- Risk: mutation/reproduction rules drift due to duplicated statements.
  - Mitigation: lifecycle doc is index-like; dedicated split specs are normative.
- Risk: telemetry contract remains too vague for future code implementation.
  - Mitigation: define required fields/reasons as normative minimums in observability spec.
- Risk: architecture doc references drift from canonical reference set.
  - Mitigation: update docs-impact/reference-spec tables in architecture plan in same slice.
- Rollback: revert newly added split specs and restore lifecycle detail sections, then rerun warn-mode harness checks.

## Review cycles: 4

Cycle 1 (architecture + goals): validated that splitting mutation/reproduction contracts improves GP-02 boundary clarity and GP-04 observability without changing runtime ownership or crate dependency direction.

Cycle 2 (verification): completed warn-mode doc/architecture harness checks plus grep-based assertions for new doc discoverability and absence of `Determinism Requirements` headings in active v3 reference docs.

Cycle 3 (spawn-arbitration simplification): replaced multi-reason spawn rejection contract with a single invalid-target gate and clarified immediate action-time validation against current world state.

Cycle 4 (tick orchestration alignment): removed deferred queue/commit terminology
from active contracts, aligned reproduction to immediate action resolution, and
linked arbitration ownership to `v3-tick-orchestration-spec.md`.
