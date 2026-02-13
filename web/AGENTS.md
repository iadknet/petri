# Frontend AGENTS.md

Frontend-specific instructions for coding agents working in `web/`.

## Instruction Layering

- Follow root policy in `../AGENTS.md`.
- Follow instruction-layering contract in `../docs/standards/agent-instruction-layering.md`.
- Keep this file focused on frontend-local boundaries and tests.

## Stack and Scope

- Keep the frontend stack as React + Vite + TypeScript.
- Do not introduce framework migrations (Next.js/Svelte/etc.) in this phase.
- Preserve current Stage 1e behavior and API contracts.

## Architecture Boundaries

- UI components must not call `fetch` or create `WebSocket` instances directly.
- Transport concerns belong in dedicated client modules.
- State orchestration belongs in a simulation store/controller layer.
- Canvas drawing logic stays isolated from transport and control logic.

## Component Responsibility Rules

- Keep components focused and composable.
- Avoid single files that mix transport, orchestration, rendering, and control UI.
- Prefer feature-level folders for simulation UI and behavior.
- Keep protocol parsing/normalization outside presentational components.

## Protocol and Type Discipline

- Keep all server payloads explicitly typed.
- Update producer/consumer type contracts together when frame or API payloads change.
- Decode and normalize protocol data in one place before UI consumption.
- Prefer behavior assertions over implementation details in tests.

## Layered Testing Policy

Use a layered strategy for frontend verification:

1. Fast deterministic tests:
   - Vitest + Testing Library + MSW
   - No live backend dependency
2. Browser smoke tests:
   - Playwright (`@playwright/test`)
   - Local-only in this phase (not CI-gated)

Required frontend test commands:

- `npm run test`
- `npm run test:e2e`
- `npm run build`

## Frontend TDD Requirement

- For frontend behavior changes and bug fixes:
  - add or adjust a failing test first
  - implement minimal changes to pass
  - keep tests green through refactors

## Completion Gate (Frontend)

Before claiming frontend completion, run and confirm all pass in `web/`:

1. `npm run test`
2. `npm run test:e2e`
3. `npm run build`

## Browser Automation Workflow

- When debugging or validating browser behavior manually, use the installed `playwright` skill workflow.
- Store Playwright artifacts under `output/playwright/` when artifacts are needed.
