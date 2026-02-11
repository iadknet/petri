# Slice 2: Paint API + Paint UI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver Stage 2 Slice 2 world painting with a dedicated paint mode and floating viewport toolbar backed by a single `POST /simulation/world/paint` API.

**Architecture:** Add paint mutation primitives to `petri-core::World`, wire phase-aware paint behavior in `petri-server::AppState` (idle startup layer vs paused live world), and expose one paint endpoint. In web, add paint-mode state plus floating controls in viewport and commit brush strokes on pointer-up.

**Tech Stack:** Rust (`axum`, `serde`, `tokio`), React + TypeScript + Vitest + MSW.

## Summary decisions
- Dedicated paint mode toggle in viewport.
- Paint allowed only in `idle` and `paused`.
- Tools: `food`, `barrier`, `erase_food`, `erase_barrier`.
- Square brushes only (`1x1`, `3x3`, `5x5` => `brush_half_extent` `0|1|2`).
- Stroke requests batch points and commit once on pointer-up.
- Idle edits persist in startup paint layer and clip to bounds on world resize.
- Barrier paint removes creatures in target cells; these removals do not increment death diagnostics.
- Clear action wipes food+barriers (paused world or idle layer).
- Idle preview modes: `paint_layer` and `full_startup`.

## API contract
### Endpoint
- `POST /simulation/world/paint`

### Request
```json
{
  "action": "stroke|clear_all|preview",
  "tool": "food|barrier|erase_food|erase_barrier",
  "brush_half_extent": 0,
  "points": [{"x":1,"y":2}],
  "idle_preview_mode": "paint_layer|full_startup"
}
```

### Response
```json
{
  "phase": "idle|paused",
  "stats": {
    "affected_cells": 0,
    "food_set_cells": 0,
    "food_cleared_cells": 0,
    "barrier_set_cells": 0,
    "barrier_cleared_cells": 0,
    "creatures_removed": 0
  },
  "frame": { "...WorldFrame...": true }
}
```

### Errors
- `409` `paint_phase_not_editable` when phase is `running` or `starting`.
- `400` for malformed/invalid paint payload.

## TDD slices
1. `petri-core` paint ops + tests.
2. `petri-server` paint endpoint/app-state + tests.
3. `web` paint API/store/UI + tests.
4. Full verification gates.
