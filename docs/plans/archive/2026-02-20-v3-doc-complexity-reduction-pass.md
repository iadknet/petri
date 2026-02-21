# V3 Docs Complexity-Reduction Pass

**Goal:** Reduce active V3 documentation complexity and drift by centralizing determinism and soft-default policy authority and splitting oversized architecture planning docs into concise index + companions.

**Goal IDs:** GP-02, GP-03, GP-04

**Scope:** Active docs/contracts only. Includes policy canonicalization and architecture-plan restructuring. Excludes runtime code changes and archive rewrites.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `AGENTS.md` | Update canonical determinism scope statement |
| `docs/strategy/architecture.md` | Align determinism wording + canonical reference |
| `docs/strategy/roadmap.md` | Align determinism wording + canonical reference |
| `docs/strategy/goals.md` | Add canonical determinism reference for GP-03 |
| `docs/plans/2026-02-18-v3-mesh-refactor-design.md` | Refactor to concise index |
| `docs/plans/2026-02-18-v3-mesh-runtime-execution-companion.md` | Create |
| `docs/plans/2026-02-18-v3-mesh-schema-module-companion.md` | Create |
| `docs/reference/v3-mesh-execution-spec.md` | Mark authoritative soft-default matrix + canonical V3 reproducibility guidance |
| `docs/reference/v3-vm-isa-spec.md` | De-duplicate cross-runtime fallback + determinism text |
| `docs/reference/v3-graph-backend-spec.md` | De-duplicate cross-runtime fallback + determinism text |
| `docs/reference/v3-creature-lifecycle-spec.md` | Replace detailed reproducibility section with canonical reference |
| `docs/reference/v3-genome-spec.md` | Keep soft-default guidance summary-only |
| `docs/reference/v3-mutation-spec.md` | Replace detailed reproducibility section with canonical reference |
| `docs/reference/v3-reproduction-spec.md` | Replace detailed reproducibility section with canonical reference |
| `docs/reference/v3-evolution-observability-spec.md` | Replace detailed reproducibility section with canonical reference |

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

- **GP-02:** Canonical policy ownership becomes explicit (AGENTS for global determinism scope, mesh execution spec for runtime fallback behavior).
- **GP-03:** Test reproducibility is retained while removing implicit requirement that runtime be deterministic in production.
- **GP-04:** Fallback and reproducibility policies remain discoverable through clear source-of-truth references.

---

## Boundary Impact

- No crate or runtime ownership changes.
- Documentation boundary changes:
  - architecture plan split into index + companions;
  - determinism policy centralized globally;
  - runtime soft-default policy centralized in one V3 runtime spec.

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `AGENTS.md` | change | Canonical global determinism scope should be explicit and stable. |
| `docs/reference/v3-mesh-execution-spec.md` | change | Canonical chain-level fallback matrix should live in one place. |
| `docs/reference/v3-vm-isa-spec.md` + `docs/reference/v3-graph-backend-spec.md` | keep | Backend-local fallback details stay local; wording/linking is refined so chain-level policy is referenced, not duplicated. |
| `docs/plans/2026-02-18-v3-mesh-refactor-design.md` | change | Oversized architecture plan converted into index with companion breakdowns. |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should determinism policy stay V3-local? | No. Canonical statement is project-level in `AGENTS.md`. | user+agent | resolved |
| Should soft-default behavior be canonical in VM/Graph specs? | No. Canonical matrix is in `v3-mesh-execution-spec.md`. | user+agent | resolved |
| Should architecture plan remain monolithic? | No. Split into index + runtime companion + schema/module companion. | user+agent | resolved |
| Should archive docs be updated in this pass? | No. Active docs only. | user+agent | resolved |

---

## Implementation Checklist

### Task 1: Global determinism policy canonicalization

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/strategy/architecture.md`
- Modify: `docs/strategy/roadmap.md`
- Modify: `docs/strategy/goals.md`

**Checklist:**
- [x] Add canonical determinism scope in `AGENTS.md`.
- [x] Replace ambiguous deterministic-runtime wording in strategy docs.
- [x] Add short references from strategy docs to AGENTS canonical statement.

### Task 2: Split oversized architecture plan

**Files:**
- Modify: `docs/plans/2026-02-18-v3-mesh-refactor-design.md`
- Create: `docs/plans/2026-02-18-v3-mesh-runtime-execution-companion.md`
- Create: `docs/plans/2026-02-18-v3-mesh-schema-module-companion.md`

**Checklist:**
- [x] Refactor index plan to concise navigational role.
- [x] Add `See also` links in index to both companions.
- [x] Add `Parent plan` back-reference in both companions.
- [x] Ensure each companion contains required metadata/sections.

### Task 3: Centralize soft-default runtime authority

**Files:**
- Modify: `docs/reference/v3-mesh-execution-spec.md`
- Modify: `docs/reference/v3-vm-isa-spec.md`
- Modify: `docs/reference/v3-graph-backend-spec.md`
- Modify: `docs/reference/v3-creature-lifecycle-spec.md`
- Modify: `docs/reference/v3-genome-spec.md`
- Modify: `docs/reference/v3-mutation-spec.md`

**Checklist:**
- [x] Mark authoritative soft-default matrix in mesh execution spec.
- [x] Keep VM/Graph fallback text backend-local and linked to mesh execution matrix.
- [x] Remove/shorten duplicated soft-default normative detail elsewhere.

### Task 4: De-duplicate V3 determinism/reproducibility notes

**Files:**
- Modify: `docs/reference/v3-mesh-execution-spec.md`
- Modify: `docs/reference/v3-vm-isa-spec.md`
- Modify: `docs/reference/v3-graph-backend-spec.md`
- Modify: `docs/reference/v3-mutation-spec.md`
- Modify: `docs/reference/v3-reproduction-spec.md`
- Modify: `docs/reference/v3-evolution-observability-spec.md`
- Modify: `docs/reference/v3-creature-lifecycle-spec.md`

**Checklist:**
- [x] Keep detailed V3 reproducibility guidance only in mesh execution spec.
- [x] Replace other detailed sections with concise references to AGENTS + mesh execution spec.

### Task 5: Verification and completion metadata

**Files:**
- Modify: `docs/plans/archive/2026-02-20-v3-doc-complexity-reduction-pass.md`

**Checklist:**
- [x] Run doc harness in warn mode.
- [x] Run architecture harness in warn mode.
- [x] Run reproducibility/soft-default/determinism grep checks.
- [x] Update checklist and review cycle metadata.

---

## Verification Commands

- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-doc-harness.sh --mode warn`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-architecture-harness.sh --mode warn`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "## Test-Mode Reproducibility|## Test-Mode Reproducibility Notes" docs/reference/v3-*.md`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "authoritative soft-default matrix" docs/reference/v3-*.md`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "determinism|deterministic|reproduc" AGENTS.md docs/strategy/*.md docs/reference/v3-*.md`

---

## Risks and Rollback

- Risk: over-compression removes useful implementation context.
  - Mitigation: companion plans preserve detail while index remains navigable.
- Risk: reference drift after section removals.
  - Mitigation: canonical references + grep/harness checks.
- Risk: determinism wording regresses in future edits.
  - Mitigation: single canonical statement in AGENTS plus policy references.
- Rollback: revert this pass's doc commits and restore previous plan/spec shape.

---

## Review cycles: 2

Cycle 1: Established complexity-reduction task set and canonical policy locations before implementation.

Cycle 2: Executed architecture-plan split, centralized determinism/soft-default policy authority, and completed warn-mode harness + grep verification.
