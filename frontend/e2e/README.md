# Frontend E2E Suite (Local Only)

This directory contains the local end-to-end suite for the frontend using `agent-browser`.

## Run

```bash
cd /Users/istefanek/claude-evolution-game/frontend
npm run test:e2e
```

Run a single scenario:

```bash
npm run test:e2e -- --scenario E2E-03
```

Run headed:

```bash
npm run test:e2e -- --headed
```

## What the runner does

1. Chooses free backend/frontend ports (or respects `E2E_BACKEND_PORT` / `E2E_FRONTEND_PORT`).
2. Starts `/Users/istefanek/claude-evolution-game/scripts/dev.sh` with those ports.
3. Waits for backend and frontend readiness.
4. Runs scenarios sequentially with isolated `agent-browser --session` names.
5. Stops all processes even on failure.

## Environment overrides

- `E2E_STARTUP_TIMEOUT_MS` (default `60000`)
- `E2E_CMD_TIMEOUT_MS` (default `30000`)
- `E2E_ARTIFACT_DIR` (default `frontend/.artifacts/e2e`)
- `E2E_BACKEND_PORT` (optional fixed backend port)
- `E2E_FRONTEND_PORT` (optional fixed frontend port)

## Artifacts

Artifacts are written only on failed scenarios:

- `snapshot-interactive.json`
- `snapshot-interactive-cursor.json`
- `diff-snapshot.json`
- `errors.json`
- `console.json`
- `annotated.png`
- `commands.log`
- `stack.log`
- `failure.txt`

## Troubleshooting

- Ensure `agent-browser` is installed and runnable in shell:
  `agent-browser --help`
- If browser binaries are missing:
  `agent-browser install`
- For flaky local startup timing, raise `E2E_STARTUP_TIMEOUT_MS`.
