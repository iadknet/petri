# Petri V2 Frontend Wireframe and UX Spec

**Goal:** Define complete desktop-first `v2-web` wireframes, interaction states, and UX contracts for `v2alpha1`, including food/barrier painting, before frontend implementation begins.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Wireframes and interaction contracts for `v2/web` plus required server-facing paint/edit API expectations; excludes implementation code.
**Docs Impact:** Adds CP-3 frontend wireframe source-of-truth and state matrix reference consumed by Stage 4 and frontend implementation planning.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: UI exposes creature behavior and environment effects clearly enough to reason about evolution outcomes.
- `GP-02`: Wireframes preserve boundary ownership (`v2-web` presentation only, `v2-server` transport, `v2-core` simulation policy).
- `GP-03`: State matrix and acceptance criteria reduce implementation ambiguity and rework.
- `GP-04`: Wireframes include observability surfaces (status, health, inspector, action summaries).

## Boundary Impact

- `v2-web` owns layout, user interactions, and client-side state orchestration.
- `v2-server` remains the sole mutation and read boundary for world edits and simulation lifecycle actions.
- `v2-core` remains policy owner for world mutation legality and simulation semantics.
- This spec defines UI/UX contracts only; CP-3 API spec remains the protocol authority.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/web/src/App.tsx` skeleton | change | Needs full app-shell and feature-surface structure replacing placeholder content. |
| `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md` | change | Needs explicit paint/edit endpoint contract before implementation. |
| `v2/crates/v2-core` simulation policy ownership | keep | World edit semantics must remain server/core-owned, not redefined in web logic. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should paint/edit be available while simulation is `running`? | No; allow only in `idle` and `paused`. | user+agent | resolved |
| Should paint strokes apply continuously per pointer-move or as one batched commit? | World-state mutation is batched on pointer-up, but UI shows a live local preview overlay while dragging. | user+agent | resolved |
| Which brush shapes/sizes are required in `v2alpha1`? | Square brushes only: `1x1`, `3x3`, `5x5`. | user+agent | resolved |
| Is snapshot import/export part of this frontend scope? | No, remains out of scope unless re-planned. | user+agent | resolved |
| Should minimap/heatmap overlays be required in `v2alpha1`? | Defer; not required for initial CP-3 exit. | user+agent | resolved |

## Frontend Coverage Contract (`v2alpha1`)

Platform scope:
1. `v2alpha1` is desktop-only.
2. Mobile/tablet layouts are intentionally out of scope.

Required product surfaces:
1. Startup Draft panel (`seed`, world size/wrap, sensor radius, population, runtime defaults).
2. Runtime controls (`start`, `pause`, `step`, TPS controls, state indicator).
3. Viewport render (`creatures`, `food`, `barriers`) with frame tick label.
4. Paint mode (`food`, `barrier`, `erase_food`, `erase_barrier`, brush size, clear-all).
5. Creature inspector (selected creature identity + key telemetry fields).
6. Run-health summary (`population`, `mean_energy`, births/deaths window, action counts).
7. Protocol/connection banner (disconnected, reconnecting, version mismatch, server error envelope summary).

## Wireframes (Text Form)

Desktop (`v2alpha1`, primary):

```text
+-----------------------------------------------------------------------------------+
| Header: Simulation State | Tick | Protocol Version | Connection Badge            |
+---------------------------+-------------------------+------------------------------+
| Left Rail                | Viewport Canvas/Grid    | Right Rail                   |
| - Startup Draft          | - Creature Layer        | - Creature Inspector         |
| - Runtime Controls       | - Food Layer            | - Run Health Metrics         |
| - Paint Toolbar          | - Barrier Layer         | - Last Error / Events        |
| - Paint Actions          | - Selection Overlay     | - Action Count Breakdown     |
+---------------------------+-------------------------+------------------------------+
| Footer: Command hints / keyboard shortcuts / API latency summary                 |
+-----------------------------------------------------------------------------------+
```

## Paint UX Contract

1. Paint mode is explicit; simulation controls remain visible while painting.
2. Tool options: `food`, `barrier`, `erase_food`, `erase_barrier`.
3. Brush sizes: `1x1`, `3x3`, `5x5`.
4. A stroke collects pointer samples client-side, renders a live preview overlay during drag, and sends one batched request on pointer-up.
5. `clear_all` requires confirm dialog and is available only in editable phases (`idle`, `paused`).
6. Invalid edit phase (`running`) surfaces non-blocking error toast + inline badge.
7. After successful paint commit, UI refreshes from canonical server frame/status responses.

## Required CP-3 Paint API Addendum (for implementation planning)

Expected contract to be added to CP-3 API spec before coding:
1. `POST /v2/simulation/world/paint`
2. Request fields:
   - `action: "stroke" | "clear_all"`
   - `tool: "food" | "barrier" | "erase_food" | "erase_barrier"` (required for `stroke`)
   - `brush_half_extent: 0 | 1 | 2`
   - `points: [{ x: u16, y: u16 }]` (`stroke` only)
3. Response fields:
   - `protocol_version`
   - `state`
   - `tick`
   - `paint_result: { touched_cells: u32 }`
4. Error cases:
   - `409 invalid_state_transition` (phase not editable)
   - `400 invalid_request` (bad payload)
   - `422 validation_rejected` (out-of-range or malformed values)
5. Client preview rule:
   - Preview rendering is local-only and must not mutate canonical world state until the batched paint commit is accepted.

## Task List

### Task 1: Publish frontend wireframe and state contracts

Files:
- Create: `docs/plans/2026-02-14-v2-frontend-wireframe-spec.md`
- Create: `docs/reference/v2-frontend-state-matrix.md`

Steps:
1. Record desktop wireframes and required surfaces.
2. Define phase-by-phase UI state matrix and allowed actions.
3. Capture paint interaction constraints and non-goals.

### Task 2: Align Stage 4 with frontend wireframe authority

Files:
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-rewrite-program.md`

Steps:
1. Add this wireframe spec as an explicit Stage 4 dependency.
2. Require wireframe sign-off before frontend implementation tasks.
3. Keep Stage 4 checkpoint rules aligned with CP-3 and matrix gates.

### Task 3: Add CP-3 paint addendum planning hook

Files:
- Modify: `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`
- Modify: `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

Steps:
1. Add paint/edit endpoint contract section to CP-3 API spec.
2. Add paint workflow coverage requirement to CP-3 pass criteria.
3. Keep command-gate ownership in matrix (no duplicated command lists).

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`

## Risks and Rollback

- Risk: wireframes overfit legacy UX patterns and miss v2-specific observability needs.
- Risk: paint UX decisions can force CP-3 protocol churn if not locked before coding.
- Rollback:
1. Revert this wireframe spec and related dependency-link edits.
2. Re-issue wireframes with narrower scope and explicit unresolved decisions.
