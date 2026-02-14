# v2 web AGENTS.md

Local instructions for `v2/web`.

## Scope

`v2/web` owns desktop UI composition and protocol consumption for `v2-server`.

## Architecture Rules

- Keep code modular by feature under `src/features/*`.
- UI components should not call `fetch` or create `WebSocket` directly.
- Put transport/protocol parsing in dedicated client/decoder modules.
- Keep render logic separate from state orchestration and side effects.

## Product Scope Rules (`v2alpha1`)

- Desktop only; mobile/touch support is out of scope.
- Paint uses live local preview during drag and a single commit on pointer-up.
- Canonical world state comes from server responses/events only.
- Backward compatibility is not required; protocol shifts may require restart.

## Test Policy (Gate-Only)

- Primary coverage: unit + integration (`npm run test`).
- Playwright is limited to CP-3 gate-smoke flows (`npm run test:e2e`).
- No nightly/scheduled frontend test track in this phase.
- Keep e2e focused on startup/runtime controls and paint happy path.
