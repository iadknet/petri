# Petri V2 Frontend Full-Surface Implementation Plan

**Goal:** Implement a complete desktop-first `v2alpha1` frontend surface (controls, viewport, inspector, telemetry, and food/barrier paint tooling) against locked CP-3 contracts.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** `v2/web` UI architecture and behavior plus `v2-server` paint/edit transport required by the UI; excludes snapshot import/export and legacy API compatibility.
**Docs Impact:** Adds detailed frontend execution plan referenced by Stage 4; depends on frontend wireframe spec and CP-3 API contract.
**Supersedes:** none
**Superseded-By:** `docs/plans/2026-02-14-v2-cp3-frontend-micro-implementation-plan.md`

## Goal Alignment

- `GP-01`: frontend exposes runtime behavior and environment manipulation needed to guide evolution experiments.
- `GP-02`: plan keeps simulation policy in `v2-core`, mutation transport in `v2-server`, and presentation in `v2/web`.
- `GP-03`: implementation uses test-first slices and fixture-locked protocol behavior.
- `GP-04`: plan includes inspector and run-health surfaces as non-optional features.

## Boundary Impact

- `v2/web` owns wireframe-compliant layout, local UI state, and interaction orchestration.
- `v2-server` owns HTTP/WebSocket contracts and paint mutation endpoint behavior.
- `v2-core` owns world mutation validity and simulation semantics; no browser-specific logic.
- Legacy crates/apps remain untouched and unreferenced.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/web/src/*` | change | Replace skeleton with stateful frontend shell and feature modules. |
| `v2/crates/v2-server/src/*` | change | Add explicit paint/edit transport contract for frontend parity. |
| `v2/crates/v2-core/*` | keep | No frontend-driven policy shifts; server invokes existing core semantics. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should paint requests stream continuously while dragging? | No; batch stroke on pointer-up, with local preview rendered during drag. | user+agent | resolved |
| Should `clear_all` be allowed in `paused` only? | Allow in both `idle` and `paused`; reject in `running`. | user+agent | resolved |
| Should UI include keyboard shortcuts for paint and lifecycle controls? | Yes for core actions (`space`, `p`, `1..4`, `[`, `]`). | user+agent | resolved |
| Must web support touch interactions in `v2alpha1`? | No; desktop pointer/mouse interaction only in `v2alpha1`. | user+agent | resolved |
| Should minimap and heatmaps ship in `v2alpha1`? | Defer to follow-up unless Stage 4 scope is expanded. | user+agent | resolved |
| Should frontend test planning include nightly/scheduled e2e runs? | No; test execution is gate-driven only, with Playwright limited to checkpoint-critical smoke flows. | user+agent | resolved |

## Specification Dependencies

- Frontend UX source of truth:
  - `docs/plans/2026-02-14-v2-frontend-wireframe-spec.md`
- Protocol source of truth:
  - `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`
- Shared gate matrix source of truth:
  - `docs/plans/2026-02-14-v2-implementation-test-matrix.md`
- Stage owner plan:
  - `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

## Platform Scope

1. `v2alpha1` frontend implementation targets desktop only.
2. Mobile/tablet responsive UX is deferred to a follow-up checkpoint/plan.

## Frontend Test Policy (Gate-Only)

1. Unit and integration tests (`npm run test`) are the primary frontend coverage path.
2. Playwright e2e coverage is limited to gate-critical smoke behavior needed for `CP-3` exit.
3. Nightly/scheduled e2e runs are out of scope for this plan.
4. The authoritative command list is `docs/plans/2026-02-14-v2-implementation-test-matrix.md`.

## Implementation Slices

### Task 1: Lock CP-3 paint transport contract before coding

Files:
- Modify: `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`
- Modify: `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

Steps:
1. Add `POST /v2/simulation/world/paint` request/response/error schema.
2. Add phase/editability rules and invalid-state error semantics.
3. Add CP-3 pass-criteria language requiring paint workflow contract coverage.

### Task 2: Add failing server tests for paint and lifecycle interaction

Files:
- Create: `v2/crates/v2-server/tests/world_paint.rs`
- Modify: `v2/crates/v2-server/tests/lifecycle.rs`
- Modify: `v2/crates/v2-server/tests/payloads.rs`
- Modify: `v2/crates/v2-server/tests/ws_stream.rs`

Steps:
1. Add failing tests for paint stroke/clear contract.
2. Add failing tests for phase restrictions (`running` rejected).
3. Add failing tests for response payload and error-envelope conformance.
4. Add failing tests ensuring post-paint frame/status synchronization semantics and no world mutation before commit request.

### Task 3: Implement server paint endpoint and state wiring

Files:
- Modify: `v2/crates/v2-server/src/api.rs`
- Modify: `v2/crates/v2-server/src/state.rs`
- Modify: `v2/crates/v2-server/src/ws.rs`
- Modify: `v2/crates/v2-server/src/main.rs`

Steps:
1. Implement request parsing and validation.
2. Route stroke/clear operations through app state and core world mutation path.
3. Emit updated status/frame events according to ordering rules.
4. Preserve non-paint endpoint behavior and idempotency contracts.

### Task 4: Add failing web tests and fixtures for full frontend shell

Files:
- Modify: `v2/web/package.json`
- Create: `v2/web/vitest.config.ts`
- Create: `v2/web/playwright.config.ts`
- Create: `v2/web/src/test/setup.ts`
- Create: `v2/web/src/features/protocol/protocol.test.ts`
- Create: `v2/web/src/features/paint/paint-interaction.test.tsx`
- Create: `v2/web/src/features/layout/app-shell.test.tsx`
- Create: `v2/web/src/fixtures/protocol-v2alpha1/*.json`
- Create: `v2/web/e2e/gate-smoke.spec.ts`

Steps:
1. Add test tooling and scripts (`npm run test`).
2. Add failing parser/decoder fixture tests.
3. Add failing UI tests for desktop startup/runtime controls and state badges.
4. Add failing paint-mode tests for tool select, brush select, stroke commit, and invalid-phase errors.
5. Add failing Playwright gate-smoke coverage for startup/runtime flow and paint commit happy path.

### Task 5: Implement web app shell and feature modules

Files:
- Modify: `v2/web/src/App.tsx`
- Create: `v2/web/src/app/AppShell.tsx`
- Create: `v2/web/src/app/layout.css`
- Create: `v2/web/src/features/startup/StartupPanel.tsx`
- Create: `v2/web/src/features/runtime/RuntimeControls.tsx`
- Create: `v2/web/src/features/viewport/ViewportCanvas.tsx`
- Create: `v2/web/src/features/paint/PaintToolbar.tsx`
- Create: `v2/web/src/features/inspector/CreatureInspector.tsx`
- Create: `v2/web/src/features/health/RunHealthPanel.tsx`
- Create: `v2/web/src/features/protocol/client.ts`
- Create: `v2/web/src/features/protocol/models.ts`
- Create: `v2/web/src/features/protocol/decoders.ts`

Steps:
1. Implement wireframe-compliant desktop shell.
2. Implement protocol client and decoder boundary.
3. Implement startup/runtime flows against CP-3 endpoints.
4. Implement inspector and health surfaces.

### Task 6: Implement paint interaction end-to-end

Files:
- Modify: `v2/web/src/features/viewport/ViewportCanvas.tsx`
- Modify: `v2/web/src/features/paint/PaintToolbar.tsx`
- Create: `v2/web/src/features/paint/paintClient.ts`
- Create: `v2/web/src/features/paint/paintState.ts`

Steps:
1. Implement pointer/mouse stroke capture with live preview overlay and pointer-up batch commit.
2. Implement clear-all confirm action and editability guards by phase.
3. Surface paint errors non-blockingly (toast + inline badge).
4. Reconcile client view from authoritative server frame/status after commit.

### Task 7: Finalize docs and checkpoint alignment

Files:
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`
- Modify: `docs/README.md`
- Modify: `v2/README.md`

Steps:
1. Mark frontend wireframe and implementation plans as CP-3 execution dependencies.
2. Update docs index entries for new frontend references.
3. Keep compatibility stubs and canonical docs consistent with Stage 4 closeout policy.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. Run the `CP-3` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-3 exit`).

## Risks and Rollback

- Risk: frontend parity scope can expand beyond CP-3 and delay checkpoint exit.
- Risk: introducing paint transport can create protocol churn if contract is not frozen first.
- Risk: web test tooling setup can produce friction if introduced too late.
- Rollback:
1. Revert frontend implementation commits while preserving CP-3 protocol baseline.
2. Re-enter via smaller slices (`transport`, `shell`, `paint`) with locked fixtures.
