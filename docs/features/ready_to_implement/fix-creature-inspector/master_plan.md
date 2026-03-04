**Goal:** Fix the creature inspector crash caused by unhandled `ActionQueue` input reference variant and add defensive guards against future schema drift.
**Goal IDs:** GP-04, GP-03
**Scope:** Frontend only — `frontend/src/types/genome.ts`, `frontend/src/components/inspector/inputRefUtils.ts`, and defensive guards in `NodeGraph.tsx`, `SamplerPlaybackPanel.tsx`, `MeshHopTimeline.tsx`. No backend changes.
**Docs Impact:** none
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

| Goal ID | Work Items |
|---------|------------|
| GP-04 (Keep Behavior Observable) | Restore creature inspector functionality so all creatures are inspectable regardless of genome composition |
| GP-03 (Keep Iteration High-Confidence) | Add defensive `typeof` guards so future backend enum additions degrade gracefully instead of crashing |

## Boundary Impact

No crate or module boundaries change. This is a frontend-only bug fix affecting type definitions and display utilities.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| Frontend type contract (`types/genome.ts`) | change | Must match backend `InputReference` enum — add `"ActionQueue"` variant |
| Inspector display utilities (`inputRefUtils.ts`) | change | Must handle unit enum variants before `in` operator — add string guards |
| Backend `InputReference` serialization | keep | Serde's default externally-tagged representation is correct; frontend must adapt |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Color for `ActionQueue` in node graph | Use `#f59e0b` (amber) — distinct from World (green), introspection (blue), upstream (gray) | — | resolved |

## Required Skills

- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns` BEFORE writing any frontend code and before each review
- Frontend UI/design: invoke `web-design-guidelines` and `frontend-design` BEFORE writing any UI code; use `agent-browser` for screenshot-based design validation after each step

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Frontend: e2e tests using `agent-browser` MUST be written per user-facing step.
Frontend UI changes: use `agent-browser` screenshots + `web-design-guidelines` review after each step.
Repeat screenshot + review until clean (recursive).

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Implementation Steps

- [ ] **Step 1: Update `InputReference` type** — In `frontend/src/types/genome.ts`, add `| "ActionQueue"` to the `InputReference` union type so the TypeScript type matches the backend enum.

- [ ] **Step 2: Fix `inputRefUtils.ts`** — Add `typeof ref === "string"` early-return guard in both `formatInputRef()` and `inputRefColor()` before any `"key" in ref` checks. For `"ActionQueue"`, return label `"ActionQueue"` and color `#f59e0b` (amber). For any other unknown string, return the string itself as label and gray as color (graceful degradation).

- [ ] **Step 3: Add defensive guards to `NodeGraph.tsx`** — Add `typeof def === "string"` guards before `"Vm" in def` and `"Graph" in backendDef` checks (lines 127, 142, 439, 445). If `def` is a string, treat it as an unknown backend type with a neutral display.

- [ ] [parallel] **Step 4: Add defensive guards to `SamplerPlaybackPanel.tsx` and `MeshHopTimeline.tsx`** — Add `typeof backend_trace === "string"` guards before `"Vm" in backend_trace` / `"Graph" in backend_trace` checks. If string, show a fallback "unsupported trace type" message.

- [ ] **Step 5: Run lint, tests, and build** — `cd frontend && npm run lint && npm run test && npm run build`. Fix any issues.

- [ ] **Step 6: E2E validation with `agent-browser`** — Start the simulation, seed or find a creature with `ActionQueue` input refs, click to open inspector, verify it renders without errors, verify `ActionQueue` displays with amber color in node graph tooltip, check browser console for no TypeErrors.

**Review cycles:** 1
