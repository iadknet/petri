# Creature Inspector Mesh Workspace Refactor

**Goal:** Re-align the creature inspector around a dedicated mesh workspace that makes topology, node internals, and execution focus inspectable without collapsing frontend boundaries.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:**
- In: frontend inspector resource/state split, inspector shell/workspace split, mesh analysis/layout/presentation domain modules, ELK-based mesh canvas, keyboard navigator, detail pane, mini trace, shared VM/graph presentation helpers, browser screenshot verification
- Out: backend/API changes, React Flow/Cytoscape migration, new runtime telemetry contracts

**Docs Impact:**
- Added this implementation plan under `docs/features/in_progress/creature-inspector-mesh-workspace/`

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

| Goal ID | Work Items |
|---------|-----------|
| GP-01 | The mesh workspace makes evolved decision graphs legible enough to inspect routing, backend composition, and execution focus, directly improving iteration on richer creature cognition |
| GP-02 | The refactor splits resource, playback, workspace UI state, and mesh-domain logic into explicit layers instead of extending a single inspector component |
| GP-03 | Pure mesh analysis/layout/trace-focus helpers are covered by unit tests, workspace behavior is covered by component tests, and browser screenshots verify user-visible layout states |
| GP-04 | The mini trace, detail pane, and shared VM/graph presentation expose concrete execution internals instead of hover-only summaries |

---

## Boundary Impact

- `frontend/src/stores/`
  - `creatureInspector.ts` split into structured detail/resource slices
  - `samplePlayback.ts` owns playback cursor and transport controls
  - `sampleSession.ts` owns sampling session state
  - `inspectorWorkspace.ts` owns inspector-local UI state
- `frontend/src/components/inspector/hooks/`
  - `useMeshDerivation.ts` owns mesh workspace derivation and selection/focus filtering
  - `useInspectorSampler.ts` owns inspector-local sampler orchestration
- `frontend/src/components/inspector/mesh/`
  - mesh UI primitives (`MeshCanvas`, `MeshControls`, React Flow wrappers, viewport controller) plus pure mesh-domain helpers for topology analysis, ELK layout, shared presentation, and trace focus
- `frontend/src/components/inspector/`
  - unified inspector shell, node detail surfaces, sampler bar, and shared presentation helpers

No backend dependency direction changes and no wire-format changes.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `frontend/src/api/` | keep | No transport contract changes were required for milestone 1 |
| `frontend/src/stores/` | change | Inspector data state, playback state, and workspace UI state are now separated by concern |
| `frontend/src/components/inspector/hooks/` | change | Inspector-local derivation and sampler orchestration moved under the inspector boundary instead of the shared hooks layer |
| `frontend/src/components/inspector/mesh/` | change | Mesh UI primitives and mesh-domain helpers now live together under a single canonical subtree |
| `frontend/src/components/inspector/` | change | The unified inspector shell owns presentation surfaces and composes the canonical mesh subtree |
| `frontend/src/types/` | change | `CreatureDetail` transport types moved out of `genome.ts` so genome types stay structural |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should milestone 1 replace the custom renderer with a third-party graph library? | No. Keep the custom renderer and use `elkjs` for layout only. Revisit a React Flow spike only if the upgraded workspace still feels constrained. | Agent | resolved |
| How should execution and mesh stay connected without over-coupling playback controls into the mesh tab? | Add a compact mini trace strip in `Mesh` and keep full transport/timelines in `Execution`. | Agent | resolved |
| How should SVG interaction stay accessible? | Make the canvas visual-first, but pair it with a semantic `MeshNavigator` and semantic node hit targets. | Agent | resolved |

---

## Required Skills

- Frontend architecture/review: `vercel-react-best-practices`, `vercel-composition-patterns`
- Frontend UI/design/review: `frontend-design`
- Browser verification: `agent-browser`

---

## Implementation Steps

- [x] Task 1: Realign creature detail and sampling into dedicated resource/state boundaries (`creatureDetailResource`, `sampleSessionResource`, `samplePlaybackStore`, inspector state slices).
- [x] Task 2: Replace the fixed inspector column with `InspectorShell`, explicit workspaces, and resizable inspector width state.
- [x] Task 3: Add mesh-domain modules (`meshAnalysis`, `meshLayout`, `meshPresentation`) and build the static mesh workspace (`MeshControls`, `MeshCanvas`, `MeshNavigator`, `MeshDetailPane`) on top of ELK layout.
- [x] Review Gate: Code review — run frontend review using `vercel-react-best-practices` and `vercel-composition-patterns`, use the `superpowers:code-reviewer` subagent when available, fix all blocking findings, and re-review until clean.
- [x] Review Gate: Architecture & decomposition review — review for boundary violations, decomposition opportunities, separation of concerns, and consistency with `docs/strategy/` plus relevant `AGENTS.md` files.
- [x] Task 4: Add `MeshMiniTrace`, `traceFocus`, active edge highlighting, and shared VM/graph presentation across Mesh and Execution surfaces.
- [x] Review Gate: Code review — run frontend review using `vercel-react-best-practices`, `vercel-composition-patterns`, and `frontend-design`, use the `superpowers:code-reviewer` subagent when available, fix all blocking findings, and re-review until clean.
- [x] Review Gate: Architecture & decomposition review — review for boundary violations, decomposition opportunities, separation of concerns, and consistency with `docs/strategy/` plus relevant `AGENTS.md` files.
- [x] Verification: `cd frontend && npm run test`
- [x] Verification: `cd frontend && npm run build`
- [ ] Verification: `cd frontend && npm run lint`
- [x] Verification: `agent-browser` screenshot pass on the mesh workspace default, resized, active-trace, and narrow-width states

---

**Review note:** `superpowers:code-reviewer` could not be dispatched from this environment because no Task/subagent execution tool was available. Manual frontend review was used as the fallback review path.

---

**Review cycles:** 1
