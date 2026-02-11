# Stage 1f Frontend Standards, Layered Testing, and Moderate Refactor Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

## Goal

Introduce frontend-local agent guidance, establish layered frontend testing (Vitest + Testing Library + MSW + Playwright smoke), and refactor the frontend into modular transport/store/components while preserving Stage 1e behavior.

## Architecture

- Keep React + Vite + TypeScript.
- Keep backend API contracts unchanged.
- Move from monolithic `App.tsx` to feature-oriented modules:
  - API client
  - frame stream client
  - simulation store/controller
  - composable UI panels/components

## Task 0: Guidance Split

**Files:**
- Modify: `AGENTS.md`
- Add: `web/AGENTS.md`

**Steps:**
1. Add explicit delegation line in root `AGENTS.md` for frontend work.
2. Create `web/AGENTS.md` with architecture/testing/TDD/completion rules.

## Task 1: Add Layered Frontend Test Tooling

**Files:**
- Modify: `web/package.json`
- Modify: `web/tsconfig.app.json`
- Add: `web/vitest.config.ts`
- Add: `web/playwright.config.ts`
- Add: `web/src/test/setup.ts`
- Add: `web/src/test/server.ts`
- Add: `web/src/test/handlers.ts`

**Steps:**
1. Add Vitest/Testing Library/MSW/Playwright dependencies and scripts.
2. Configure Vitest jsdom setup and shared MSW test server hooks.
3. Configure Playwright for local smoke runs.
4. Ensure TypeScript config includes test files and vitest globals.

## Task 2: Add Required Frontend Tests (TDD)

**Files:**
- Add: `web/src/protocol.test.ts`
- Add: `web/src/App.test.tsx`
- Add: `web/e2e/smoke.spec.ts`
- Add: `web/e2e/README.md`

**Steps:**
1. Add protocol tests for frame decode and shape assumptions.
2. Add App behavior tests:
   - idle placeholder renders
   - start disabled when startup non-viable
   - startup draft patch path
   - runtime controls disable in idle
   - API error path surfaces UI error
3. Add Playwright smoke test with a basic lifecycle flow.
4. Add e2e README with local preconditions and run commands.

## Task 3: Moderate Refactor to Match Frontend Standards

**Files:**
- Move/modify: `web/src/App.tsx` -> `web/src/app/App.tsx`
- Modify: `web/src/main.tsx`
- Add: `web/src/features/simulation/api/simulationApiClient.ts`
- Add: `web/src/features/simulation/ws/frameStreamClient.ts`
- Add: `web/src/features/simulation/store/simulationStore.ts`
- Add: `web/src/features/simulation/components/ControlHeader.tsx`
- Add: `web/src/features/simulation/components/SimulationControls.tsx`
- Add: `web/src/features/simulation/components/StartupDraftPanel.tsx`
- Add: `web/src/features/simulation/components/RuntimeTuningPanel.tsx`
- Add: `web/src/features/simulation/components/LiveMetricsPanel.tsx`
- Add: `web/src/features/simulation/components/ViewportCanvas.tsx`

**Steps:**
1. Extract API transport from `App` into `simulationApiClient`.
2. Extract WebSocket transport from `App` into `frameStreamClient`.
3. Introduce a simulation store/controller for orchestration and state.
4. Split monolithic JSX into focused components.
5. Keep pan/zoom and canvas behavior unchanged.
6. Keep tests passing after each extraction batch.

## Verification

Run and confirm all pass:

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run test`
5. `cd web && npm run test:e2e`
6. `cd web && npm run build`
