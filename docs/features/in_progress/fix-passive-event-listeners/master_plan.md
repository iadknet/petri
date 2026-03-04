# Fix Passive Event Listener preventDefault Warnings

**Goal:** Eliminate "Unable to preventDefault inside passive event listener" browser warnings during map wheel zoom and fix page-scroll-behind-canvas bug.
**Goal IDs:** GP-03
**Scope:** `frontend/src/components/WorldViewport.tsx` only. Replace React `onWheel` prop with native `useEffect`-based listener using `{ passive: false }`.
**Docs Impact:** None — no canonical docs affected.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-03` (Keep Iteration High-Confidence): Fixes a functional bug (page scrolls behind canvas during zoom) and eliminates noisy console warnings that obscure real errors during development.

## Boundary Impact

No crate or module boundaries change. The fix is entirely within `frontend/src/components/WorldViewport.tsx`.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `frontend/src/components/` | keep | Change is internal to WorldViewport; no new component APIs or cross-component dependencies |
| `frontend/src/stores/viewport.ts` | keep | Store API unchanged; only the event source changes from React synthetic to native |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Are there other components with this issue? | Only WorldViewport has `onWheel` with `preventDefault` | agent | resolved |
| Does removing React `onWheel` affect any other handler coordination? | No — wheel handling is independent of other mouse handlers on the canvas | agent | resolved |

## Required Skills

- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns` BEFORE writing any frontend code and before each review

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

- [x] Step 1: Replace React `onWheel` prop with native `useEffect` listener — In `WorldViewport.tsx`: (a) Remove `onWheel={handleWheel}` from the `<canvas>` JSX. (b) Change `handleWheel` from `React.WheelEvent` to native `WheelEvent` parameter type (drop the `React.` prefix). (c) Add a new `useEffect` that calls `canvasRef.current.addEventListener('wheel', handleWheel, { passive: false })` and returns a cleanup that removes the listener. Dependencies: `[handleWheel]`.
- [x] Step 2: Verify fix — Run `cd frontend && npm run lint && npm run test && npm run build` to confirm no regressions. Run `agent-browser` e2e smoke test on map zoom to confirm no console warnings and correct zoom behavior.
- [x] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke `vercel-react-best-practices` + `vercel-composition-patterns`. Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 1
