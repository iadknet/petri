---
title: Fix Passive Event Listener preventDefault Warnings
tags: [frontend, ui]
size: S
depends-on: []
status: needs-review
---

## Problem Statement

The browser console spams "Unable to preventDefault inside passive event listener invocation" warnings repeatedly during normal map interaction (scrolling/zooming). This happens because React 17+ registers `wheel` and `touch` event listeners on the document root as **passive by default** per the browser spec. When `e.preventDefault()` is called inside React's synthetic `onWheel` handler, the browser ignores it and logs a warning. This means the page may also scroll behind the canvas during zoom — a functional bug, not just a console nuisance.

## Root Cause

**File:** `frontend/src/components/WorldViewport.tsx`

The `handleWheel` callback (line ~157) calls `e.preventDefault()` inside a React `onWheel` prop handler. React delegates this to a passive root-level listener, so `preventDefault()` is silently ignored.

## User Stories / Acceptance Criteria

- As a user, I want to zoom the map with my scroll wheel without seeing console warnings or the page scrolling behind the canvas.
- **AC1:** Zero "Unable to preventDefault inside passive event listener" warnings in the console during normal map interaction.
- **AC2:** Scroll-wheel zoom on the canvas works correctly and prevents page scroll.
- **AC3:** No regressions to existing wheel zoom behavior (zoom-at-cursor, smooth zooming).

## Solution

Replace the React `onWheel={handleWheel}` prop on the `<canvas>` element with a `useEffect` that attaches a native event listener with `{ passive: false }` directly to the canvas DOM element. Remove the `onWheel` prop from JSX.

## Out of Scope

- Touch event handling (no touch handlers exist in the codebase currently).
- Refactoring other event handlers in WorldViewport.
- Any canvas rendering or camera logic changes.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Are there other components with the same issue? | Investigation found only WorldViewport.tsx | — | resolved |
