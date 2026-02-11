# E2E Smoke Tests

This folder contains local Playwright smoke checks for the web control flow.

## What this verifies

- App loads and shows idle placeholder.
- Start, Pause/Resume, and Restart controls work in a basic lifecycle path.
- No uncaught browser console/runtime errors during the smoke flow.

## Run

```bash
cd web
npm run test:e2e
```

For interactive debugging:

```bash
cd web
npm run test:e2e:headed
# or
npm run test:e2e:debug
```

## Local runtime behavior

`playwright.config.ts` starts:

1. `petri-server` on `127.0.0.1:4000`
2. Vite dev server on `127.0.0.1:5173`

Playwright starts dedicated test servers for both services to guarantee a clean
idle simulation state at test start.
