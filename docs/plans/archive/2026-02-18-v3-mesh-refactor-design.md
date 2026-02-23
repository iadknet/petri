# V3 Mesh Refactor Design (Index)

**Goal:** Define a concise, canonical architecture index for the V3 mesh refactor that links to detailed companion plans and active reference specs.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** High-level architecture decisions, module boundary posture, implementation-order summary, and references to detailed runtime/schema companion plans. Excludes deep execution/schema narrative now maintained in companion plans and active reference specs.

**Docs Impact:**

| Doc | Action |
|-----|--------|
| `docs/plans/archive/2026-02-18-v3-mesh-refactor-design.md` | Refactor to concise index |
| `docs/plans/archive/2026-02-18-v3-mesh-runtime-execution-companion.md` | Create |
| `docs/plans/archive/2026-02-18-v3-mesh-schema-module-companion.md` | Create |
| `docs/reference/v3-mesh-execution-spec.md` | Canonical execution/soft-default reference |
| `docs/reference/v3-tick-orchestration-spec.md` | Canonical tick flow/arbitration reference |
| `docs/reference/v3-runtime-config-spec.md` | Canonical runtime config defaults/validation reference |
| `docs/reference/v3-genome-spec.md` | Canonical genome parseability reference |
| `docs/reference/v3-vm-isa-spec.md` | VM backend-local behavior reference |
| `docs/reference/v3-graph-backend-spec.md` | Graph backend-local behavior reference |

**Supersedes:** `docs/plans/2026-02-14-v3-architecture-design.md` (archived, not deleted)

**Superseded-By:** none

**See also:**
- `docs/plans/archive/2026-02-18-v3-mesh-runtime-execution-companion.md`
- `docs/plans/archive/2026-02-18-v3-mesh-schema-module-companion.md`

---

## Goal Alignment

- **GP-01:** Mesh cognition remains centered on multi-node VM/Graph execution and routing-driven behavior composition.
- **GP-02:** Ownership boundaries stay explicit across runtime, genome, sensors, tick, and contracts.
- **GP-03:** High-confidence iteration remains test-driven with crash-proof runtime semantics and test reproducibility where needed.
- **GP-04:** Observability, mutation, and reproduction contracts remain explicit and reviewable as separate canonical specs.

---

## Boundary Impact

- Active architecture remains within `v3/crates/v3-core` and `v3/crates/v3-server`; no new crates.
- `contracts/` remains the runtime-to-tick boundary via `WorldAction`.
- Execution internals (routing/output slots/graph recurrence) remain runtime-local details.
- Split-plan structure reduces policy duplication risk and keeps architecture intent readable.

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `kernel/` | keep | world reality remains independent from controller backend details |
| `contracts/` | keep | `WorldAction` remains the only runtime→tick contract surface |
| `tick/actions.rs` | keep | action application remains separate from cognition internals |
| `runtime/` | keep | mesh chain evaluation and backend dispatch remain runtime-owned; companion docs refine details |
| `creature/genome.rs` | keep | schema evolution is bounded by parseability rules and runtime soft defaults; companion docs refine details |
| `sensors/` | keep | static snapshot and dynamic introspection split remains intentional; companion docs refine details |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should architecture details stay in one plan file? | No. Keep this file as index and move deep details to two companion plans. | user+agent | resolved |
| Where is execution fallback behavior canonical? | `v3-mesh-execution-spec.md` (authoritative soft-default matrix). | user+agent | resolved |
| Is determinism a runtime product requirement? | No. Determinism is test/harness-scoped only (canonical in `AGENTS.md`). | user+agent | resolved |

---

## Architecture Index

Primary reference specs:
- `docs/reference/v3-mesh-execution-spec.md`
- `docs/reference/v3-tick-orchestration-spec.md`
- `docs/reference/v3-runtime-config-spec.md`
- `docs/reference/v3-genome-spec.md`
- `docs/reference/v3-sensor-spec.md`
- `docs/reference/v3-vm-isa-spec.md`
- `docs/reference/v3-graph-backend-spec.md`
- `docs/reference/v3-creature-lifecycle-spec.md`
- `docs/reference/v3-mutation-spec.md`
- `docs/reference/v3-reproduction-spec.md`
- `docs/reference/v3-evolution-observability-spec.md`

Companion plans:
- Runtime/execution detail: `docs/plans/archive/2026-02-18-v3-mesh-runtime-execution-companion.md`
- Schema/module detail: `docs/plans/archive/2026-02-18-v3-mesh-schema-module-companion.md`

---

## Implementation Order (Summary)

1. Finalize schema and parseability contracts.
2. Finalize tick orchestration and runtime chain evaluation contracts.
3. Finalize backend-local VM/Graph semantics against canonical runtime rules.
4. Keep mutation/reproduction/observability policies linked and non-duplicative.
5. Keep strategy and AGENTS policy language aligned with canonical determinism scope.

---

## Task Outline (Index Maintenance)

### Task 1: Maintain architecture index and companion links

**Files:**
- Modify: `docs/plans/archive/2026-02-18-v3-mesh-refactor-design.md`
- Modify: `docs/plans/archive/2026-02-18-v3-mesh-runtime-execution-companion.md`
- Modify: `docs/plans/archive/2026-02-18-v3-mesh-schema-module-companion.md`

- [ ] Keep this file concise and navigational.
- [ ] Keep `See also` links current.
- [ ] Keep companion `Parent plan` back-references current.

### Task 2: Keep canonical reference list synchronized

**Files:**
- Modify: `docs/README.md` (if reference list changes)
- Modify: this index plan file

- [ ] Ensure active reference spec set is complete.
- [ ] Ensure no links point to archived/superseded active contracts.

---

## Verification Commands

- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-doc-harness.sh --mode warn`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-architecture-harness.sh --mode warn`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "See also|Parent plan" docs/plans/2026-02-18-v3-mesh-*.md`

---

## Risks and Rollback

- Risk: index becomes too sparse and loses important intent.
  - Mitigation: keep explicit architecture index + concise implementation-order summary.
- Risk: companion/index links drift.
  - Mitigation: enforce bidirectional links and grep checks.
- Rollback: restore prior monolithic plan content from git history and remove companion files.

---

## Review cycles: 2

Cycle 1: Original architecture synthesis and initial mesh contract consolidation.

Cycle 2: Complexity-reduction pass refactoring this plan into index + companion structure.
