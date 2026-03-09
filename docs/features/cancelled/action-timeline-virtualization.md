---
title: Action Timeline Segment Virtualization
tags: [frontend, architecture]
size: S
depends-on: []
status: cancelled
---

## Cancellation Note

Cancelled when the legacy action timeline implementation was removed alongside the old inspector. There is no active timeline surface left to virtualize in the current inspector architecture.

## Problem Statement

The `ActionTimeline` component (`frontend/src/components/inspector/ActionTimeline.tsx`) renders the full action log history as a horizontal bar of color-coded DOM segments. Each entry in the action log is rendered as an individual `<div>` element inside the `TimelineBar` sub-component, with inline styles for width (6px), color, opacity, and optional failure border. An SVG `EnergyOverlay` path and a `TickAxis` with absolutely-positioned tick labels are layered on top.

The backend ring buffer capacity is 500 entries (`ActionLogConfig::capacity`, default 500 in `v3/crates/v3-core/src/config/simulation.rs`). The creature detail endpoint is polled at up to 10Hz (`MIN_FETCH_INTERVAL = 100ms` in `useCreatureDetail.ts`), and the entire `actionLog` array reference is replaced on every fetch cycle (no deep equality check in the store). This means `ActionTimeline` re-renders up to 10 times per second, and each render creates up to 500 interactive DOM segments (each with `role="button"`, `tabIndex`, `onClick`, `onKeyDown`, `title`, and 4-5 inline style properties), plus the SVG energy path and tick axis labels.

At the current 500-entry cap this is within acceptable DOM performance on modern hardware. However, the scrollable container (`overflow-x-auto`) means only a fraction of segments are visible at any time — the inspector panel is typically ~300-400px wide, while the full bar is 500 x 6px = 3000px. All 500 segments are mounted and reconciled regardless of scroll position.

If the action log capacity is increased (e.g., to support longer creature histories), or if the inspector is opened on lower-powered devices, this could become a performance bottleneck. Profiling would show wasted DOM reconciliation work for off-screen segments.

## User Stories / Acceptance Criteria

- As a user inspecting a creature on a low-powered device, I want the action timeline to render smoothly at 10Hz without frame drops so that the inspector remains responsive while observing live creature behavior.
- As a developer increasing the action log capacity beyond 500 entries, I want the timeline rendering cost to scale with visible segments rather than total entries so that longer histories do not degrade inspector performance.
- As a developer, I want the virtualized timeline to preserve all existing interactive behavior (segment click-to-select, detail panel, following auto-scroll, tick axis labels, energy overlay) so that no user-facing functionality regresses.

### Acceptance Criteria

1. Only segments within or near the visible scroll window are mounted as DOM elements.
2. Spacer elements maintain correct total scrollable width and scroll position.
3. All existing interactions work identically: click-to-select, keyboard navigation, auto-scroll following, detail panel.
4. Energy overlay SVG and tick axis labels continue to render correctly.
5. `npm run build` passes with zero errors.
6. No visual or behavioral regressions.

### Trigger Criteria

This optimization should be implemented when any of the following occur:
- Browser profiling shows >2ms per ActionTimeline render cycle
- The action log capacity is increased beyond 500
- User reports of inspector jank on target hardware

## Out of Scope

- Changing the action log ring buffer capacity (backend config).
- Reducing the 10Hz polling frequency or adding incremental `since_tick` fetching (tracked separately under creature-detail-api-optimization).
- Canvas-based rendering (a more radical approach that would eliminate DOM segments entirely; worth considering only if virtualization proves insufficient).
- Virtualizing other inspector components (MemoryHexView, NodeGraph, etc.).

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should this be implemented proactively or only after profiling confirms jank? | Defer until profiling shows a problem or capacity is increased beyond 500 | - | open |
| Custom hook vs external virtualization library (@tanstack/react-virtual)? | Lean toward custom — uniform geometry (fixed 6px width, single row) makes a library overkill | - | open |
| Should the energy overlay SVG path also be virtualized (clipped to visible range)? | Likely unnecessary at 500 entries (single DOM element), revisit if capacity grows to thousands | - | open |
| What overscan buffer size balances flicker prevention vs DOM savings? | Start with ~20 segments (~120px) on each side; tune based on scroll performance | - | open |
