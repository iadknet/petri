---
title: Ring-Based Vision Sensors
tags: [core, simulation, sensors, genome]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

Current area sensors (food, barrier, occupancy) aggregate over the entire vision radius into a single 7-field summary per type. This produces dense, high-dimensional inputs that are difficult for random mutations to wire up usefully — a creature must "get lucky" with connections to a large, undifferentiated input space before it can derive any survival benefit from vision.

By decomposing each area sensor into per-distance-ring variants (ring 1 = cells 1 step away, ring 2 = cells 2 steps away, etc.), each individual sensor becomes smaller and more tractable. A creature only needs to connect to a single ring's small input array to gain useful distance-specific information, lowering the evolutionary barrier for developing sensor usage. Over generations, creatures can incrementally evolve sensitivity to additional rings, building up spatial awareness piece by piece rather than needing to "understand" the full radius at once.

## User Stories / Acceptance Criteria

- As a simulation designer, I want area sensors decomposed into per-ring variants so that creatures have an easier evolutionary path to developing useful sensor connections.
- As a simulation designer, I want the ring count to stay dynamic with the runtime-configurable `vision_radius` (1-8) so that changing radius at runtime still works correctly.
- As a simulation observer, I want creatures to be able to evolve distance-aware behaviors (e.g., react differently to nearby vs. distant food) that emerge naturally from ring-specific sensor wiring.
- Acceptance: each area sensor type (food, barrier, occupancy) produces independent per-ring summaries rather than a single aggregate.
- Acceptance: genomes can reference specific ring sensors via `InputReference` / `WorldInputKey`.
- Acceptance: unused ring slots (when radius < max) return `0.0` gracefully.
- Acceptance: viability tests pass — the change does not break population sustainability.

## Out of Scope

- **Nearby creature sensors** — already distance-ranked (4 slots sorted by proximity). No ring decomposition needed.
- **Local radius-1 sensors** — `FoodHere`, `NeighborCell*` are already per-direction and stay unchanged.
- **Sensor spec documentation updates** — tracked separately if needed.
- **Frontend sensor visualization** — no UI changes in this feature.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| What fields per ring? Full 7 (total, gradient_x, gradient_y, nearest_dx, nearest_dy, nearest_dist, max_value) or a reduced set? Gradient fields may not make sense for a single ring. | pending | | open |
| How to handle the PerceptionSnapshot size budget? Current: 228/256 bytes. At max radius 8 with full 7 fields: 8 rings x 3 types x 7 fields = 168 floats (672 bytes) for area alone — far over budget. Options: (a) reduce fields per ring, (b) increase budget, (c) cap max rings below max radius, (d) make snapshot dynamically sized. | pending | | open |
| Should the aggregate (full-radius) summary be kept alongside per-ring summaries, or replaced entirely? Keeping both further increases size but preserves backward compatibility for existing genomes. | pending | | open |
| How to enumerate WorldInputKey variants for dynamic ring counts? Options: (a) fixed variants up to MAX_RADIUS with unused rings returning 0.0, (b) parameterized variant like `AreaFoodRing(u8)` with sub-index, (c) new compound key scheme. | pending | | open |
| Performance impact of computing per-ring reductions vs. single aggregate? The reducer currently iterates visible cells once per type. Per-ring would either require grouping cells by distance first or making N passes. | pending | | open |
| Should the conditional assembly optimization (`genome_uses_extended_perception`) be updated to check per-ring granularity, or is type-level gating sufficient? | pending | | open |
