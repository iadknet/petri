# Petri V2 CP-3 Fine-Grained Execution Checklist

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Execute CP-3 backend/frontend stabilization with one small behavior slice per commit, each gated by explicit checklist evidence.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** `v2/crates/v2-server`, `v2/crates/v2-cli`, and `v2/web` implementation/testing needed for CP-3 exit; excludes CP-1/CP-2 runtime policy redesign.
**Docs Impact:** Adds a checklist-driven CP-3 execution plan that consolidates backend and frontend stabilization into atomic slices.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: CP-3 surfaces real runtime behavior (not synthetic placeholders).
- `GP-02`: preserves clean ownership boundaries between core policy, transport, and UI orchestration.
- `GP-03`: enforces small-slice red-green commits and clear gate evidence.
- `GP-04`: ensures status/frame/health and operator controls are reliable and visible.

## Boundary Impact

- `v2-server` remains transport/lifecycle owner and maps payloads from core state.
- `v2-web` owns client/store/presentation behavior and real-browser resilience.
- `v2-core` is consumed for startup/tick/state truth; no transport policy moved into core.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-server/src/state.rs` | change | Remaining synthetic fallback paths must be removed slice-by-slice. |
| `v2/web/src/App.tsx` and store modules | change | UI orchestration should be store-first and test-locked. |
| `v2/crates/v2-server/tests/*` and `v2/web/*test*` | change | CP-3 must close with explicit contract and browser-realistic coverage. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should frontend tests hide CORS via same-origin proxy shortcuts? | No; CP-3 keeps cross-origin browser flow so CORS regressions are visible. | user+agent | resolved |
| Should startup founders share one phenotype baseline? | Yes; startup founders begin with one baseline phenotype. | user+agent | resolved |
| Should CP-3 close with partial frontend/backend gates? | No; CP-3 closes only when full matrix gate is green. | user+agent | resolved |
| Should startup world seeding carry forward v1 food tuning defaults before API fields exist? | Yes; use deterministic internal defaults now and defer wire-level tuning fields to a follow-up contract slice. | user+agent | resolved |

## Checklist Operating Rules

1. Keep one active slice at a time.
2. Start each slice with a failing targeted test.
3. Keep backend and frontend changes in separate commits unless a contract seam requires both.
4. Run only v2 tests and v2 web checks.
5. Update checklist checkboxes in the same commit as implementation.

## Task List

### Slice P3-B1: Startup world initialization realism

**Files:**
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/src/world_seed.rs`
- Create: `v2/crates/v2-core/src/viability.rs`
- Create: `v2/crates/v2-core/tests/world_seed.rs`
- Create: `v2/crates/v2-core/tests/startup_viability_gate.rs`
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/startup_world_init.rs`

**Checklist:**
- [x] Add/adjust failing tests for deterministic seeded food presence and startup world realism.
- [x] Run: `cd v2 && cargo test -p v2-server --test startup_world_init` and confirm failure is expected.
- [x] Implement minimal startup/reset world-init fix.
- [x] Re-run targeted test and confirm pass.
- [x] Commit slice `P3-B1`.

### Slice P3-B2: Frame creature realism from state (no synthetic reshuffle)

**Files:**
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/tests/frame_creatures.rs`

**Checklist:**
- [x] Add/adjust regression tests for stable creature IDs/positions across no-tick frames.
- [x] Run: `cd v2 && cargo test -p v2-server --test frame_creatures` and confirm behavior gate is covered.
- [x] Verify frame mapping already sources creatures from runtime state (no code-path fix required in this slice).
- [x] Re-run targeted test and confirm pass.
- [x] Commit slice `P3-B2`.

### Slice P3-B3: Tick dynamics and action-count truthfulness

**Files:**
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/server.rs`
- Modify: `v2/crates/v2-server/tests/tick_dynamics.rs`
- Modify: `v2/crates/v2-server/tests/ws_stream.rs`

**Checklist:**
- [x] Add/adjust failing tests for real tick-driven state changes and non-synthetic action counts.
- [x] Run: `cd v2 && cargo test -p v2-server --test tick_dynamics` and confirm failure is expected.
- [x] Implement minimal tick-loop integration fix.
- [x] Re-run: `cd v2 && cargo test -p v2-server --test tick_dynamics`.
- [x] Re-run: `cd v2 && cargo test -p v2-server --test ws_stream`.
- [x] Commit slice `P3-B3`.

### Slice P3-B4: Founder phenotype uniform startup baseline

**Files:**
- Modify: `v2/crates/v2-core/src/phenotype.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/startup_founder_phenotype.rs`
- Modify: `v2/crates/v2-core/tests/phenotype_evolution.rs`

**Checklist:**
- [ ] Add/adjust failing tests for startup founder phenotype uniformity and deterministic baseline.
- [ ] Run: `cd v2 && cargo test -p v2-server --test startup_founder_phenotype` and confirm failure is expected.
- [ ] Implement minimal founder-baseline fix.
- [ ] Re-run server/core targeted tests and confirm pass.
- [ ] Commit slice `P3-B4`.

### Slice P3-B5: Startup viability guard behavior

**Files:**
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/tests/startup_viability.rs`

**Checklist:**
- [ ] Add/adjust failing tests for non-viable startup handling policy.
- [ ] Run: `cd v2 && cargo test -p v2-server --test startup_viability` and confirm failure is expected.
- [ ] Implement minimal viability handling fix (deterministic normalize-or-reject behavior per contract).
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `P3-B5`.

### Slice P3-B6: Backend regression and contract closeout

**Files:**
- Modify: `v2/crates/v2-server/tests/payloads.rs`
- Modify: `v2/crates/v2-server/tests/lifecycle.rs`
- Modify: `v2/crates/v2-server/tests/http_transport.rs`

**Checklist:**
- [ ] Add/adjust regression tests for lifecycle, payload envelope, and CORS preflight contract.
- [ ] Run: `cd v2 && cargo test -p v2-server`.
- [ ] Resolve remaining backend contract regressions.
- [ ] Re-run: `cd v2 && cargo test -p v2-server` and confirm full pass.
- [ ] Commit slice `P3-B6`.

### Slice P3-F1: Protocol client defensive parsing

**Files:**
- Modify: `v2/web/src/features/protocol/client.ts`
- Modify: `v2/web/src/features/protocol/decoders.ts`
- Modify: `v2/web/src/features/protocol/protocol.test.ts`

**Checklist:**
- [ ] Add/adjust failing tests for invalid JSON/HTML and malformed error envelope paths.
- [ ] Run: `cd v2/web && npm run test -- --runInBand protocol` and confirm failure is expected.
- [ ] Implement minimal protocol client error-normalization fix.
- [ ] Re-run targeted tests and confirm pass.
- [ ] Commit slice `P3-F1`.

### Slice P3-F2: Store-first orchestration extraction

**Files:**
- Modify: `v2/web/src/App.tsx`
- Modify: `v2/web/src/features/simulation/store/simulationStore.ts`
- Modify: `v2/web/src/features/simulation/store/simulationStore.test.ts`
- Create/Modify: `v2/web/src/features/simulation/store/simulationEffects.ts`

**Checklist:**
- [ ] Add/adjust failing tests for store-owned orchestration behavior.
- [ ] Run: `cd v2/web && npm run test -- --runInBand simulationStore` and confirm failure is expected.
- [ ] Implement minimal extraction from `App.tsx` into store/effects.
- [ ] Re-run targeted tests and confirm pass.
- [ ] Commit slice `P3-F2`.

### Slice P3-F3: Connection/protocol status banner resilience

**Files:**
- Modify: `v2/web/src/features/protocol/ProtocolBanner.tsx`
- Modify: `v2/web/src/features/layout/app-shell.test.tsx`
- Modify: `v2/web/src/App.tsx`

**Checklist:**
- [ ] Add/adjust failing tests for `connecting/reconnecting/error/protocol_mismatch` states.
- [ ] Run targeted frontend tests and confirm failure is expected.
- [ ] Implement minimal banner-state wiring fix.
- [ ] Re-run targeted tests and confirm pass.
- [ ] Commit slice `P3-F3`.

### Slice P3-F4: Runtime controls unhappy-path behavior

**Files:**
- Modify: `v2/web/src/features/runtime/runtimeControls.test.tsx`
- Modify: `v2/web/src/features/simulation/store/simulationStore.ts`

**Checklist:**
- [ ] Add/adjust failing tests for invalid lifecycle transitions and disabled busy states.
- [ ] Run targeted frontend tests and confirm failure is expected.
- [ ] Implement minimal controls-state/error-path fix.
- [ ] Re-run targeted tests and confirm pass.
- [ ] Commit slice `P3-F4`.

### Slice P3-F5: Paint interaction guardrails

**Files:**
- Modify: `v2/web/src/features/paint/paint-interaction.test.tsx`
- Modify: `v2/web/src/features/paint/paintState.ts`
- Modify: `v2/web/src/features/viewport/ViewportCanvas.tsx`

**Checklist:**
- [ ] Add/adjust failing tests for preview-during-drag and single-commit-on-pointer-up behavior.
- [ ] Run targeted frontend tests and confirm failure is expected.
- [ ] Implement minimal paint interaction fix.
- [ ] Re-run targeted tests and confirm pass.
- [ ] Commit slice `P3-F5`.

### Slice P3-F6: Cross-origin browser smoke and CORS gate

**Files:**
- Modify: `v2/web/playwright.config.ts`
- Modify: `v2/web/e2e/gate-smoke.spec.ts`
- Modify: `v2/crates/v2-server/tests/http_transport.rs`

**Checklist:**
- [ ] Add/adjust failing e2e/CORS checks for explicit cross-origin server/browser communication.
- [ ] Run: `cd v2/web && npm run test:e2e` and confirm failure is expected.
- [ ] Implement minimal CORS/test harness fix.
- [ ] Re-run e2e and confirm pass.
- [ ] Commit slice `P3-F6`.

### Slice P3-F7: Visual polish with functional regression protection

**Files:**
- Modify: `v2/web/src/app/layout.css`
- Modify: `v2/web/src/app/AppShell.tsx`
- Modify: `v2/web/src/features/layout/app-shell.test.tsx`

**Checklist:**
- [ ] Add/adjust tests for critical labels/control visibility at standard desktop widths.
- [ ] Implement style/layout polish without changing protocol/state behavior.
- [ ] Run: `cd v2/web && npm run test`.
- [ ] Run: `cd v2/web && npm run build`.
- [ ] Commit slice `P3-F7`.

### Slice P3-X1: CP-3 gate closeout

**Files:**
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`
- Modify: `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`

**Checklist:**
- [ ] Run full CP-3 gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md`.
- [ ] Confirm server + cli + web contract gates are all green.
- [ ] Update checkpoint/program status only after full gate pass.
- [ ] Commit slice `P3-X1`.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. Run the `CP-3` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-3 exit`).

## Risks and Rollback

- Risk: mixing UX/style and protocol logic in one slice can hide real regressions.
- Risk: same-origin shortcuts can mask CORS failures that real browsers expose.
- Rollback:
1. Revert only the failing slice commit.
2. Restore failing test for that slice.
3. Re-implement with narrower scope and rerun targeted gate.
