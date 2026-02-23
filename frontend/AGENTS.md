# AGENTS.md

Frontend-specific instructions for coding agents working in `frontend/`.

## Scope

- This file applies to all files under `frontend/`.
- Follow root policies in `/Users/istefanek/claude-evolution-game/AGENTS.md` first; this file adds frontend-specific constraints.

## Mission

- Deliver reliable, testable frontend behavior for simulation control and visualization.
- Keep frontend changes compatible with the existing v3-server API contract (`/v3/*` routes).

## Mandatory Skills (Enforced)

For any task that touches frontend code, agents must explicitly invoke and apply these skills before implementation:

1. `vercel-react-best-practices`
2. `vercel-composition-patterns`
3. `frontend-design`

For end-to-end frontend testing and browser automation, agents must use:

1. `agent-browser`

If a required skill is unavailable, state that clearly in the response and continue with the closest equivalent workflow.

## Frontend Development Rules

1. Keep boundaries clean:
   - API transport and request logic stay in `src/api/`.
   - Shared app state stays in `src/stores/`.
   - UI components stay in `src/components/`.
2. Do not bypass typed API contracts in `src/types/api.ts`.
3. Prefer composition patterns over boolean-prop mode switches when component behavior diverges.
4. Keep rendering paths efficient (avoid unnecessary re-renders and request waterfalls).
5. Preserve keyboard accessibility and visible interaction feedback for controls.
6. Do not introduce synthetic UI data that can diverge from server-applied simulation behavior.

## Testing Workflow (Required)

For behavior changes or bug fixes, follow TDD:

1. Add/adjust a failing test first (`vitest`).
2. Implement the fix.
3. Re-run tests and confirm pass.

Minimum verification for frontend changes:

1. `cd frontend && npm run lint`
2. `cd frontend && npm run test`
3. `cd frontend && npm run build`

## E2E Workflow with `agent-browser`

Use `agent-browser` for frontend e2e or flow validation work.

Canonical local e2e entrypoints:

1. `cd /Users/istefanek/claude-evolution-game/frontend && npm run test:e2e`
2. `cd /Users/istefanek/claude-evolution-game/frontend && npm run test:e2e -- --scenario E2E-03`
3. `cd /Users/istefanek/claude-evolution-game/frontend && npm run test:e2e -- --headed`

Recommended loop:

1. Prefer the scripted suite above (it self-manages stack lifecycle on isolated ports).
2. For ad-hoc debugging, run targeted `agent-browser` commands against the e2e runner ports.
3. Re-run `snapshot -i` after navigation or DOM-changing actions.
4. Verify expected UI/state transitions from simulation actions (startup/start/pause/step/config updates).

When authoring new e2e coverage, prefer reproducible scripted flows and keep scenarios focused on user-visible behavior regressions.

## Completion Criteria for Frontend Tasks

Do not claim completion until:

1. Required skills were used and stated.
2. Lint, unit tests, and build pass.
3. For UI behavior changes, at least one `agent-browser` smoke flow validates the affected path.
