# V3 Observability Pass

**Goal:** Add observability needed to diagnose mutation black holes, barrier awareness failures, and live-circuit sparsity without changing simulation behavior.

**Goal IDs:** GP-02, GP-03, GP-04

**Scope:**
- In: backend/core observability counters, server transport exposure, creature diagnostics payload, frontend evolution diagnostics, frontend inspector diagnostics, regression coverage
- Out: mutation policy changes, world mechanics changes, bus-width changes, rolling event history/export

**Docs Impact:**
- Add this implementation plan under `docs/features/in_progress/v3-observability-pass/`
- Update `docs/reference/v3-evolution-observability-spec.md` for newly exposed required counters/reason fields if needed
- Update `docs/reference/v3-server-api-protocol-spec.md` if transport field contracts are extended

**Supersedes:** none

**Superseded-By:** none

> **Pre-promotion layout:** Checked `cd v3 && ...` verification commands below are preserved as historical evidence. Current Cargo commands run from the repository root.

---

## Goal Alignment

| Goal ID | Work Items |
|---------|------------|
| GP-02 | Add truthful runtime observability for mutation/operator skip behavior and action failure causes so evolution tuning decisions are based on applied behavior. |
| GP-03 | Add regression coverage for observability accounting invariants and wire contracts so diagnostics stay correct as runtime evolves. |
| GP-04 | Improve inspectability of live cognition (reachability, slot usage, barrier reads, outcomes) via API and existing inspector surfaces. |

---

## Boundary Impact

- `crates/v3-core/`
  - Extend simulation stats with typed counters for per-operator skips and action-failure causes.
  - Keep hot-path accounting typed; avoid server/string concerns in core.
- `crates/v3-server/`
  - Extend health/status and creature-detail payload assembly for new diagnostics.
  - Preserve existing sampler protocol for this pass.
- `frontend/`
  - Extend protocol/store typing and existing `EvolutionTab` and `UnifiedInspector` components with additive diagnostics UI.

Wire-format impact:
- Intentional additive payload change in status/health and creature detail responses.
- No breaking removal in existing fields for this pass.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| Core mutation engine behavior | keep | This pass is observability-only; diagnostics must not alter mutation/reproduction mechanics. |
| Core stats/accounting | change | Correct location for typed counters and cause classification semantics. |
| Server transport mapping | change | Correct location to expose string-key maps/JSON payloads from typed core counters. |
| Frontend stats/inspector UI | change | Existing surfaces are the intended diagnostic consumers; avoid introducing a parallel dashboard. |
| Sampler protocol | keep | Current sample payload already exposes barrier/inputs needed for this pass. |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Add rolling debug event retention in this pass? | No. Use snapshot + existing sample workflow only. | User | resolved |
| Build new dashboard or extend current surfaces? | Extend existing API + inspector/evolution surfaces only. | User | resolved |
| Subagent swarms execution mode with no `spawn_agent` tool available | Use parallel independent tool tasks plus explicit staged review passes as fallback. | Agent | resolved |

---

## Required Skills

- Process workflow: `superpowers:using-superpowers`, `superpowers:using-git-worktrees`, `superpowers:subagent-driven-development`, `superpowers:dispatching-parallel-agents`, `superpowers:requesting-code-review`, `superpowers:receiving-code-review`, `superpowers:verification-before-completion`
- Backend implementation/review: `rust-skills`
- Frontend implementation/review: `vercel-react-best-practices`, `vercel-composition-patterns`, `web-design-guidelines`

---

## Task Details

### Task 0: Setup and guardrails

**Files:**
- Create: `docs/features/in_progress/v3-observability-pass/master_plan.md`

**Steps:**
1. Create isolated worktree on branch `codex/observability-pass-v1`.
2. Load required superpowers and domain skills.
3. Record explicit blocking review gates and per-task verification gates in this plan.

### Task 1: Backend mutation and action-cause telemetry

**Files:**
- Modify: `crates/v3-core/src/simulation/stats.rs`
- Modify: `crates/v3-core/src/simulation/actions/reproduction.rs`
- Modify: `crates/v3-core/src/simulation/tick.rs`
- Modify: `crates/v3-server/src/state.rs`
- Modify: `crates/v3-server/src/handlers/lifecycle.rs`
- Modify: `crates/v3-server/src/transport/protocol.rs`
- Modify tests under `crates/v3-core/` and `crates/v3-server/tests/`

**Steps:**
1. Add per-operator skip counter exposure and mutation reachability counter exposure to server payloads.
2. Add move-blocked and reproduction-invalid-target cause accounting with typed enums in core stats.
3. Thread new counters through health/status transport serialization.
4. Add regression tests for accounting invariants and payload contract fields.
5. Run targeted backend tests.

### Task 2: Creature diagnostics API exposure

**Files:**
- Modify: `crates/v3-server/src/handlers/creature.rs`
- Modify: `crates/v3-server/src/transport/sample_assembler.rs` (only if needed for derivation reuse)
- Modify tests in `crates/v3-server/tests/`

**Steps:**
1. Add `diagnostics` object to creature detail response.
2. Compute `current_inputs`, `live_circuit`, and `recent_actions` diagnostics server-side.
3. Keep large optional fields/exclusion behavior unchanged.
4. Add contract tests for diagnostics payload shape and values.
5. Run targeted backend tests.

### Task 3: Frontend evolution diagnostics

**Files:**
- Modify: `frontend/src/types/protocol.ts`
- Modify: `frontend/src/hooks/useViewSubscription.ts`
- Modify: `frontend/src/stores/stats.ts`
- Modify: `frontend/src/components/stats/EvolutionTab.tsx`
- Modify related frontend tests

**Steps:**
1. Extend typed health/status payload handling for new backend counters.
2. Add derived diagnostics rendering for per-operator skip behavior and reachability targeting.
3. Add/update tests covering payload ingestion and UI rendering.
4. Run targeted frontend tests.

### Task 4: Frontend inspector diagnostics cards

**Files:**
- Modify: `frontend/src/types/creature-detail.ts`
- Modify: `frontend/src/hooks/useCreatureDetailResource.ts`
- Modify: `frontend/src/components/inspector/UnifiedInspector.tsx`
- Modify/create focused inspector diagnostics components
- Modify related frontend tests

**Steps:**
1. Thread `diagnostics` payload into inspector state.
2. Add additive inspector diagnostics cards for barrier perception/outcomes and live-circuit structure.
3. Preserve existing visual language and existing mesh/sampler behavior.
4. Add/update tests for cards and empty-state behavior.
5. Run targeted frontend tests.

### Task 5: Integration verification and completion

**Files:**
- No new product files expected

**Steps:**
1. Run repo-required verification commands for touched surfaces.
2. Run final code review passes and ensure no unresolved findings.
3. Re-run impacted checks after any review-driven changes.

---

## Implementation Steps

- [x] Task 0: Setup isolated worktree and enforce process guardrails in this plan.
- [x] Task 1: Implement backend mutation/action observability counters and transport exposure.
- [x] Review Gate: Task 1 code review (backend) — invoke `rust-skills`, run substantive review via `superpowers:requesting-code-review`, fix all findings, re-review until clean.
- [x] Task 2: Implement creature diagnostics derivation and API exposure.
- [x] Review Gate: Task 2 code review (backend) — invoke `rust-skills`, run substantive review via `superpowers:requesting-code-review`, fix all findings, re-review until clean.
- [x] Task 3: Implement frontend evolution diagnostics surface.
- [x] Review Gate: Task 3 code review (frontend) — invoke `vercel-react-best-practices` + `vercel-composition-patterns`, run substantive review via `superpowers:requesting-code-review`, fix all findings, re-review until clean.
- [x] Task 4: Implement frontend inspector diagnostics cards.
- [x] Review Gate: Task 4 code review (frontend) — invoke `vercel-react-best-practices` + `vercel-composition-patterns` + `web-design-guidelines`, run substantive review via `superpowers:requesting-code-review`, fix all findings, re-review until clean.
- [x] Review Gate: Architecture & decomposition review — verify boundary consistency with `docs/strategy/` and relevant `AGENTS.md`, resolve issues before completion.
- [x] Verification: `scripts/check-doc-harness.sh --mode strict`
- [ ] Verification: `scripts/check-architecture-harness.sh --mode strict`
- [x] Verification: `scripts/check-plan-harness.sh --mode strict`
- [x] Verification: `cd v3 && cargo fmt --all -- --check`
- [x] Verification: `cd v3 && cargo test --workspace`
- [x] Verification: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
- [x] Verification: `cd frontend && npm run build`

_Note: `scripts/check-architecture-harness.sh --mode strict` currently reports pre-existing repository-wide baseline violations outside this feature diff; left unchecked pending separate cleanup._

---

**Review cycles:** 1
