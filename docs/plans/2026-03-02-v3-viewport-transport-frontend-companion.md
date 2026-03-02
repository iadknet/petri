# V3 Viewport Transport Frontend Companion

**Parent plan:** `docs/plans/2026-03-02-v3-viewport-transport-plan.md`

**Goal:** Refactor the frontend around viewport state, world-view state, and render models so it can consume viewport-driven transport cleanly.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Included: viewport store, world-view store, transport client boundary, render-model boundary, selection and paint behavior under LOD, and `v3alpha2` snapshot/subscription consumption. Excluded: server-side protocol implementation, CLI behavior, and visual redesign unrelated to the viewport refactor.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/reference/v3-server-api-protocol-spec.md` | Update expected client/server message shapes |
| `docs/strategy/architecture.md` | Add viewport/world-view/render ownership |
| `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` | Share frontend verification commands |

**Supersedes:** none

**Superseded-By:** none

**See also:**
- `docs/plans/2026-03-02-v3-viewport-transport-plan.md`
- `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md`

## Goal Alignment

- **GP-01:** The canvas consumes only the view it needs, not the whole world.
- **GP-02:** Viewport state, transport decoding, and rendering responsibilities become explicit.
- **GP-03:** Frontend behavior changes are driven by failing tests first and validated with the shared matrix.
- **GP-04:** The UI keeps behavior-backed telemetry and inspector behavior while adopting layered view transport.

## Boundary Impact

- `src/api/` owns transport and request logic only.
- `src/stores/viewport.ts` owns camera state and derived view requests.
- `src/stores/worldView.ts` owns world-static and current view payloads.
- `src/canvas/renderModel.ts` adapts protocol payloads into renderer-friendly structures.
- `src/canvas/renderer.ts` owns drawing only and no longer owns camera state.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `frontend/src/api/websocket.ts` | change | It currently decodes protocol data and mutates stores directly. |
| `frontend/src/stores/simulation.ts` | change | It currently mixes simulation state and full-frame world data. |
| `frontend/src/canvas/renderer.ts` | change | It currently owns camera state and expects full-world frame payloads. |
| `frontend/src/components/WorldViewport.tsx` | keep | The viewport component remains the interaction shell, but delegates more state ownership to stores/controllers. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Where do numeric zoom thresholds live? | Frontend viewport policy only; the server sees semantic fidelity tiers. | user+agent | resolved |
| What happens when the user clicks in overview mode? | Selection is disabled until detail/inspect mode. | user+agent | resolved |
| How does reconnect bootstrap? | Fetch bootstrap `/snapshot`, then send the current `subscribe_view`. | user+agent | resolved |

## Execution Requirements

- Any frontend implementation or refactor work in this companion must explicitly load and apply `vercel-react-best-practices`, `vercel-composition-patterns`, and `frontend-design` before editing code.
- Any frontend smoke-flow or E2E validation must also use `agent-browser`.
- Do not bypass typed API contracts in `src/types/api.ts` and `src/types/protocol.ts`.
- Write failing frontend tests first for behavior changes before implementation code is added.
- After every task in this companion that changes frontend code, run a recursive review pass guided by `vercel-react-best-practices` and `vercel-composition-patterns`; if the task changes UI behavior or presentation, include `frontend-design` in the same loop. Fix every new finding introduced by that task and rerun until the review reports no new findings.
- No task in this companion passes while the required recursive frontend review still has unresolved new findings.

## Architecture Target

```text
frontend/src/
  api/
    rest.ts
    websocket.ts
    protocol.ts
  stores/
    simulation.ts
    viewport.ts
    worldView.ts
  hooks/
    useViewSubscription.ts
    useCreatureSelection.ts
    usePaintInteraction.ts
  canvas/
    camera.ts
    renderModel.ts
    renderer.ts
```

## Behavior Contracts

- `viewport` store is the source of truth for camera position, visible rect, and fidelity tier.
- The websocket transport client emits typed events only; it does not mutate Zustand stores directly.
- `worldView` store merges only monotonic `projection_revision` payloads.
- `world_static` is updated only when `world_static_revision` increases.
- Overview mode uses approximate aggregates and disables selection.
- Detail/inspect modes use exact visible-region payloads and allow selection.
- Inspect mode is the only tier that receives creature energy ratios for canvas bars.
- Paint refresh currently uses `POST /paint` invalidation metadata followed by a
  viewport-scoped `GET /snapshot`; this two-request waterfall is deliberate so
  paint responses stay small and reuse the same snapshot merge path as bootstrap.

## Task List

### Task 1: Introduce viewport and world-view stores

**Files:**
- Create: `frontend/src/stores/viewport.ts`
- Create: `frontend/src/stores/worldView.ts`
- Modify: `frontend/src/stores/simulation.ts`
- Modify: `frontend/src/stores/simulation.test.ts`

### Task 2: Decouple transport from store mutation

**Files:**
- Modify: `frontend/src/api/websocket.ts`
- Create: `frontend/src/api/protocol.ts`
- Modify: `frontend/src/api/websocket.test.ts`
- Create: `frontend/src/hooks/useViewSubscription.ts`

### Task 3: Introduce render-model boundary

**Files:**
- Create: `frontend/src/canvas/camera.ts`
- Create: `frontend/src/canvas/renderModel.ts`
- Modify: `frontend/src/canvas/renderer.ts`
- Modify: `frontend/src/components/WorldViewport.tsx`

### Task 4: Update LOD-specific interaction behavior

**Files:**
- Modify: `frontend/src/hooks/useCreatureSelection.ts`
- Modify: `frontend/src/hooks/usePaintInteraction.ts`
- Modify: `frontend/src/types/api.ts`
- Modify: `frontend/src/types/protocol.ts`

## Acceptance Criteria

- Camera state is not stored in the renderer.
- The transport client can reconnect and emit typed events without mutating stores directly.
- Overview mode disables selection and detail/inspect preserve exact selection from current view data.
- The renderer consumes a render model and never expects full-world dynamic payloads after cutover.
- Every frontend task has a recorded recursive Vercel-skill review pass with no new findings before task completion.

## Verification Commands

- Use `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` as the command source of truth.

## Risks and Rollback

- Moving camera state out of the renderer can break interaction math if not covered by tests.
- Transport/store decoupling can create stale UI merges if revision checks are omitted.
- If the viewport cutover needs to pause, keep the store and render-model split while deferring protocol consumption changes.

**Review cycles:** 1
