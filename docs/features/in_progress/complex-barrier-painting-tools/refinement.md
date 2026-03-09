---
title: Complex Barrier Painting Tools
tags: [frontend, core, ui]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

Barrier painting is currently limited to free-form brush strokes at three fixed sizes (1x1, 3x3, 5x5). Creating interesting world topologies — mazes, spirals, noise fields, corridors — requires tedious manual painting that is slow, imprecise, and unrepeatable. Users need procedural pattern tools that can fill an area with a structured barrier pattern in one operation, enabling rapid creation of diverse environments for evolutionary experiments.

This feature also lays groundwork for a future "configurable barrier topology generation" feature that will integrate these same pattern algorithms into world initialization config.

## User Stories / Acceptance Criteria

- As a user, I want to select a barrier pattern tool (maze, spiral, noise, parallel lines, star) so that I can quickly create complex barrier topologies without manual pixel-by-pixel painting.
- As a user, I want to draw a bounding rectangle on the canvas to define the area where the pattern will be applied, so that I can control exactly where the pattern goes.
- As a user, I want to configure pattern-specific parameters (spacing, density, corridor width, arm count, etc.) before applying, so that I can tune the pattern to my needs.
- As a user, I want to see a live preview of the generated pattern at reduced opacity before confirming, so that I can adjust parameters or reposition before committing.
- As a user, I want the pattern generation to happen on the backend, so that the algorithms can be reused for world initialization in a future feature.

### Pattern Requirements

1. **Maze** — Generates a maze within the bounding rectangle.
   - Parameters: corridor width, wall thickness, algorithm style (e.g., recursive backtracker vs Prim's), open center size (radius of barrier-free area at center)
   - Must have at least one opening on each edge that intersects the world boundary or open space

2. **Spiral** — Generates a spiral barrier pattern.
   - Parameters: arm count, arm thickness, gap width, direction (CW/CCW), center offset, open center size (radius of barrier-free area at center)
   - Arms should be evenly spaced and smoothly curved

3. **Random noise** — Scatters barrier cells randomly within the bounds.
   - Parameters: density (0.0–1.0), cluster size (point noise vs blobby noise), seed
   - Should produce organic-looking distributions, not uniform grid noise

4. **Parallel lines** — Draws parallel squiggly/jagged lines across the bounds.
   - Parameters: line count or spacing, line thickness, jaggedness/amplitude, orientation (angle)
   - Lines should have natural-looking variation, not perfect sine waves

5. **Star patterns** — Radiating barrier lines from center points.
   - Parameters: point count, ray count per point, ray length, ray thickness
   - Multiple star centers distributed within bounds

### Technical Acceptance Criteria

- Backend exposes a new pattern-generation endpoint (or extends the existing paint endpoint) that accepts pattern type, bounding rectangle, and pattern-specific parameters
- Pattern algorithms live in `v3-core` kernel so they can be reused by future world-init config
- Frontend PaintToolbar gains pattern tool buttons; selecting one enters "area selection" mode
- Area selection draws a bounding rectangle via click-drag on the canvas
- A contextual configuration panel appears when a pattern tool is selected, showing pattern-specific parameter controls
- Live preview renders the generated pattern at reduced opacity within the bounding rectangle before the user confirms
- Confirming the pattern applies it as barrier cells (same semantics as current barrier painting: sets barrier flag, clears food, evicts creatures)
- All existing paint behavior (free-form brush, food painting, erasing) remains unchanged

## Out of Scope

- Undo/redo for paint operations (separate feature)
- World initialization config integration (planned as follow-on feature: "Configurable barrier topology generation")
- Pattern generation for food (only barriers)
- Combining multiple patterns in a single operation (user can apply patterns sequentially)
- Saving/loading pattern presets

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should pattern preview be computed server-side or client-side? Server-side ensures exact match but adds latency; client-side is instant but may diverge from final result. | Pending — likely server-side with debounced requests for parameter changes | - | open |
| Should bounding rectangle be constrained to the world bounds, or can it extend beyond (with clipping)? | Pending — likely clip to world bounds | - | open |
| Should patterns respect existing barriers (merge/union) or overwrite the entire bounding area? | Pending — likely union by default (only add barriers, don't clear existing ones) | - | open |
| Should the rectangle selection support rotation, or only axis-aligned rectangles? | Pending — likely axis-aligned only for v1 | - | open |
| What RNG seeding strategy for reproducible patterns? Explicit seed input, or auto-generated with display? | Pending | - | open |
