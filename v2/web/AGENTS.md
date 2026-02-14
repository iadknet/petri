# v2 web AGENTS.md

Local instructions for `v2/web`.

## Scope

`v2/web` owns desktop UI composition and protocol consumption for `v2-server`.

## Architecture Rules

- Keep code modular by feature under `src/features/*`.
- UI components should not call `fetch` or create `WebSocket` directly.
- Put transport/protocol parsing in dedicated client/decoder modules.
- Keep render logic separate from state orchestration and side effects.
- Keep transport endpoint resolution centralized in store/client helpers and covered by unit tests.

## Product Scope Rules (`v2alpha1`)

- Desktop only; mobile/touch support is out of scope.
- Paint uses live local preview during drag and a single commit on pointer-up.
- Canonical world state comes from server responses/events only.
- Backward compatibility is not required; protocol shifts may require restart.

## Frontend Quality Gates

- A stage is not complete until the app works in plain local boot with no frontend env overrides (`v2-server` on `127.0.0.1:4100` plus `v2/web` via `npm run dev`).
- A stage is not complete if browser console errors or `pageerror` events appear in core flows.
- Preserve a strong usability baseline: readable hierarchy, clear control affordances, stable status visibility, and no layout breakage at common desktop widths.
- Avoid inline one-off layout styles when equivalent reusable CSS classes can be used.

## Test Policy (Gate-Only)

- Primary coverage: unit + integration (`npm run test`).
- Playwright is limited to CP-3 gate-smoke flows (`npm run test:e2e`).
- No nightly/scheduled frontend test track in this phase.
- Keep e2e focused on startup/runtime controls and paint happy path plus critical failure signals (console/page errors).
- Keep Playwright transport cross-origin (`VITE_API_BASE`/`VITE_WS_BASE`) so browser CORS behavior is observable.
- For transport/startup wiring changes, add unit coverage for defaults and env overrides.
- For each new frontend feature or bug fix, include at least one non-happy-path automated test (example: server error envelope, reconnect, invalid transition, or invalid payload).
