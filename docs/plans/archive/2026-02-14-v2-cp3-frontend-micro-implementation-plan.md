# Petri V2 CP-3 Frontend Micro-Implementation Plan

**Goal:** Deliver a stable, modular `v2/web` control surface by implementing frontend behavior in small, test-gated slices with explicit failure coverage.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** `v2/web` architecture, transport/error handling, runtime controls, paint workflows, and gate-critical Playwright coverage; includes minimal `v2-server` CORS/test adjustments only when required for browser-realistic tests.
**Docs Impact:** Adds a CP-3 frontend micro-slice execution plan and supersedes the prior broad frontend implementation plan.
**Supersedes:** `docs/plans/2026-02-14-v2-frontend-implementation-plan.md`
**Superseded-By:** none

## Goal Alignment

- `GP-01`: frontend reliably exposes runtime behavior and environment edits.
- `GP-02`: presentation/orchestration stay in web feature modules; protocol/server responsibilities stay separated.
- `GP-03`: each UI/transport fix includes at least one unhappy-path automated test.
- `GP-04`: protocol, connection, and health status remain visible and actionable in the UI.

## Boundary Impact

- `v2/web` receives modular store-first architecture and stricter transport/error guardrails.
- `v2-server` may receive narrow CORS/test harness changes required for real-browser coverage.
- `v2-core` receives no UI-driven policy changes.
- Protocol version and payload schema remain `v2alpha1`.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/web/src/App.tsx` | change | Keep app shell compositional; remove orchestration and side effects from top-level component. |
| `v2/web/src/features/simulation/store/simulationStore.ts` | change | Centralize transport, lifecycle orchestration, polling/ws behavior, and derived-state logic. |
| `v2/web/src/features/protocol/*` | change | Harden decoding and error handling for non-JSON and protocol mismatch cases. |
| `v2/crates/v2-server/src/server.rs` | keep | Backend remains contract owner; frontend must consume and validate, not duplicate server policy. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should Playwright continue using same-origin proxy shortcuts? | No. Keep cross-origin `VITE_API_BASE`/`VITE_WS_BASE` so CORS behavior is visible in real browser runs. | user+agent | resolved |
| Should new frontend slices require unhappy-path tests? | Yes. Every feature/fix slice adds at least one non-happy-path test. | user+agent | resolved |
| Should protocol/version mismatch be silent in UI? | No. Show explicit protocol/connection banner state. | user+agent | resolved |
| Should this plan include mobile/touch support? | No. Desktop-only remains the `v2alpha1` scope. | user+agent | resolved |

## Slice Execution Rules

1. One micro-slice per commit.
2. Add/adjust targeted tests each slice; start red-first when behavior is missing, otherwise run a regression-hardening verification slice.
3. Do not merge visual/style changes with transport/state refactors in the same slice.
4. Keep Playwright as gate-critical smoke only; move most coverage to unit/integration tests.
5. If a task conflicts with architecture boundaries or overall project goals, stop and ask for guidance before implementation.

## Micro-Slice Task List

### Slice F1: Transport endpoint + startup failure guard tests

Files:
- Modify: `v2/web/src/features/simulation/store/simulationStore.test.ts`
- Modify: `v2/web/src/features/protocol/protocol.test.ts`

Steps:
1. Add failing tests for default transport endpoint resolution and env overrides.
2. Add failing tests for startup failures where response is invalid JSON/HTML.
3. Add failing tests asserting useful error text reaches UI state.

### Slice F2: Harden protocol client error handling

Files:
- Modify: `v2/web/src/features/protocol/client.ts`
- Modify: `v2/web/src/features/protocol/decoders.ts`
- Modify: `v2/web/src/features/protocol/models.ts`

Steps:
1. Parse non-2xx responses defensively (invalid JSON and malformed envelopes included).
2. Normalize transport/protocol errors into stable typed messages.
3. Green F1 tests.

### Slice F3: Store-first architecture extraction

Files:
- Create: `v2/web/src/features/simulation/store/simulationEffects.ts`
- Create: `v2/web/src/features/simulation/store/simulationActions.ts`
- Create: `v2/web/src/features/simulation/store/simulationSelectors.ts`
- Modify: `v2/web/src/features/simulation/store/simulationStore.ts`
- Modify: `v2/web/src/App.tsx`

Steps:
1. Move lifecycle side effects and transport orchestration out of `App.tsx`.
2. Keep `App.tsx` focused on composition and prop wiring.
3. Add/adjust tests proving equivalent behavior after refactor.

### Slice F4: Connection and protocol banner resilience

Files:
- Modify: `v2/web/src/App.tsx`
- Create: `v2/web/src/features/protocol/ProtocolBanner.tsx`
- Modify: `v2/web/src/features/layout/app-shell.test.tsx`

Steps:
1. Add explicit states: `connecting`, `reconnecting`, `connected`, `error`, `protocol_mismatch`.
2. Ensure banner content is deterministic and testable.
3. Add failing tests for mismatch and reconnect transitions, then implement.

### Slice F5: Runtime control unhappy-path tests

Files:
- Create: `v2/web/src/features/runtime/runtimeControls.test.tsx`
- Modify: `v2/web/src/features/simulation/store/simulationStore.test.ts`

Steps:
1. Add failing tests for invalid transitions (`pause` from idle, `step` when not paused).
2. Add failing tests that server error envelope surfaces in UI.
3. Add failing tests for disabled-state behavior while busy.

### Slice F6: Paint workflow guardrails and tests

Files:
- Create: `v2/web/src/features/paint/paint-interaction.test.tsx`
- Modify: `v2/web/src/features/paint/paintState.ts`
- Modify: `v2/web/src/features/paint/paintClient.ts`
- Modify: `v2/web/src/features/viewport/ViewportCanvas.tsx`

Steps:
1. Add failing tests for local preview during drag and single commit on pointer-up.
2. Add failing tests for paint rejection while `running`.
3. Implement commit semantics and error surfacing with canonical frame refresh.

### Slice F7: Cross-origin e2e/CORS gate hardening

Files:
- Modify: `v2/web/playwright.config.ts`
- Modify: `v2/web/e2e/gate-smoke.spec.ts`
- Modify: `v2/crates/v2-server/tests/http_transport.rs`

Steps:
1. Keep browser-server communication explicitly cross-origin in e2e setup.
2. Extend smoke test to fail fast on console errors and connection banner errors.
3. Add/adjust server CORS test assertions needed by real-browser flows.

### Slice F8: Visual polish and layout stability pass

Files:
- Modify: `v2/web/src/app/layout.css`
- Modify: `v2/web/src/app/AppShell.tsx`
- Modify: `v2/web/src/features/*/*.tsx` (only style/layout touch points)

Steps:
1. Introduce/normalize CSS variables for coherent typography, color, spacing, and states.
2. Improve hierarchy/affordance consistency without changing protocol/state behavior.
3. Add/adjust UI tests for critical labels and control visibility at standard desktop widths.

### Slice F9: Frontend gate closeout

Files:
- Modify: `v2/web/e2e/gate-smoke.spec.ts`
- Modify: `v2/web/src/features/protocol/protocol.test.ts`
- Modify: `v2/web/src/features/simulation/store/simulationStore.test.ts`

Steps:
1. Add regression assertions for previously fixed transport/state issues.
2. Confirm startup/runtime/paint smoke path remains green with cross-origin transport.
3. Freeze fixture expectations for `v2alpha1` protocol behavior used by UI.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2/web && npm run test`
3. `cd v2/web && npm run build`
4. `cd v2/web && npm run test:e2e`
5. Run the `CP-3` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-3 exit`).

## Risks and Rollback

- Risk: mixing UX redesign and transport refactors in one slice can hide regressions.
- Risk: e2e-only validation misses edge cases; unit/integration guardrails must stay primary.
- Rollback:
1. Revert only the failing micro-slice commit.
2. Re-add a minimal failing test that reproduces the regression, then re-implement the slice narrowly.
