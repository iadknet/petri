---
title: Incremental Projection Platform with Event-Driven Invalidation
tags: [core, architecture]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

The viewport transport refactor (now completed) introduced a server-local projection layer in `v3-server::query` that publishes `ProjectionSnapshot` objects with spatial indexing and revision tracking. This projection layer successfully decoupled read-side concerns from the simulation mutex, but it has two structural limitations that will compound as the system grows.

**Limitation 1: The projection layer is server-local and single-consumer.** The types `ProjectionSnapshot`, `ProjectionStore`, `CreatureTileIndex`, and the cache helpers (`build_food_density_u8`, `build_barrier_mask`) all live inside `v3-server::query` and are tightly coupled to the server's websocket transport. The CLI (`v3-cli`) builds its own entirely separate tick-sample projections by reading `Simulation` fields directly. The creature inspector endpoint (`GET /v3/simulation/creature/:id`) bypasses the projection layer entirely and locks the simulation mutex for direct reads. Future consumers -- metrics exporters, replay/export pipelines, a potential standalone inspector tool -- would each need to duplicate projection logic or thread it through the server. A shared projection platform with types and spatial indexes outside the server-local module tree would let all consumers share a single query surface.

**Limitation 2: Every projection publication rebuilds from scratch.** On every publish cycle (currently gated at 100ms intervals during running state), `build_ws_frame` in `lifecycle.rs` performs a full scan of all creatures (including genome complexity computation), a full scan of every world cell for food density and barrier status, and then `ProjectionSnapshot::from_ws_frame` rebuilds the entire `food_density_u8` dense array, the entire `barrier_mask` bit-array, and the entire `CreatureTileIndex` spatial index from scratch. For a 256x256 world with 2000 creatures this is acceptable, but the cost scales as O(world_cells + creatures) per publish and will become a bottleneck for larger worlds or higher publish rates. The existing `DirtyRect` infrastructure in `query::cache` already tracks paint-time dirty regions and `world_static_changed` flags, but these are used only for downstream delivery filtering -- they do not feed back into the projection build itself to enable incremental updates.

Event-driven invalidation is the mechanism that makes the projection platform incremental: instead of rescanning the full world, the projection layer would receive domain events (creature moved, food changed in region, barrier topology changed) and update only the affected portions of its derived state.

## User Stories / Acceptance Criteria

- As a server transport developer, I want projection types and spatial indexes to live in a shared crate or module boundary so that I can reuse them across the websocket transport, HTTP snapshot endpoint, and creature inspector without duplicating query logic.
- As a CLI developer, I want to consume the same projection types and query surfaces used by the server so that CLI tick-sample output stays consistent with server-side projections without maintaining a parallel implementation.
- As a simulation developer, I want the projection layer to incrementally update its derived state (food density grid, barrier mask, creature tile index) based on domain-level change events rather than rebuilding from scratch every publish cycle, so that projection cost stays proportional to what changed rather than total world size.
- As a transport developer, I want topology-dirty, food-region-dirty, and creature-visualization-dirty tracking to flow from the simulation tick into the projection layer so that I can skip rebuilding unchanged projection facets on each publish.
- As a future replay/export consumer, I want to subscribe to the same projection query surface used by the live server so that offline analysis tools share the same data contracts and spatial index capabilities without coupling to server internals.

## Detailed Design (Sketch)

### Shared projection boundary

Extract projection types (`ProjectionSnapshot`, `CreatureTileIndex`, `ViewRect`, `DirtyRect`, cache helpers) from `v3-server::query` into either a new `v3-projection` crate or a `projection` module within `v3-core`. The exact crate boundary decision depends on whether the projection types need to depend on `v3-core` types (they do -- `CreatureSnapshot`, `FramePayload`) or whether a thinner contract layer can be defined. The key constraint is that `v3-server`, `v3-cli`, and future consumers can all depend on the shared projection boundary without depending on each other.

### Domain change events

Introduce lightweight change-event signals emitted by the simulation tick loop:

- **CreatureSetChanged** -- creatures added (birth), removed (death), or moved. Carries a list of affected creature IDs and their old/new positions.
- **FoodRegionChanged** -- food density changed in a bounding rect. Emitted by food growth, food consumption (eat action), and food painting.
- **TopologyChanged** -- barrier mask changed. Emitted by barrier painting. Already partially tracked by `world_static_changed` in `query::cache`.
- **StatsChanged** -- aggregate stats updated (always, every tick). This is a lightweight signal that the status/health projection facets need refreshing.

These events are not a full event-sourcing log; they are ephemeral per-tick invalidation signals consumed by the projection layer and then discarded.

### Incremental projection updates

The projection store would maintain persistent derived state and apply incremental updates:

- **Food density grid**: On `FoodRegionChanged`, update only the cells within the dirty bounding rect rather than rebuilding the full `width * height` dense array.
- **Barrier mask**: On `TopologyChanged`, rebuild only the affected byte ranges. Since topology changes are rare (paint-only), this is a smaller win but preserves the pattern.
- **Creature tile index**: On `CreatureSetChanged`, remove affected creatures from their old tile buckets, insert into new tile buckets. This replaces the current full-rebuild via `CreatureTileIndex::build`.
- **Status/health projections**: Always refreshed (cheap scalar aggregates).

### Consumer integration points

- **Server websocket transport**: Continues to consume `ProjectionSnapshot` as today, but the snapshot is now incrementally maintained rather than rebuilt.
- **Server HTTP endpoints**: `/snapshot` and creature inspector could both query the shared projection surface.
- **CLI**: Can optionally construct a lightweight `ProjectionStore` for its tick-sample output, sharing types and spatial query logic.
- **Future replay consumers**: Can instantiate their own `ProjectionStore` and feed it domain events from a recorded event stream.

## Out of Scope

- Full event-sourcing or persistent event log. This feature is about ephemeral invalidation signals, not durable event storage.
- Replay/export pipeline implementation. This feature establishes the projection boundary that replay consumers would use, but does not build the replay system itself.
- Simulation determinism changes. Domain events are derived from already-applied simulation mutations; they do not change simulation semantics.
- Multi-process or network-distributed projection. The shared boundary is in-process only.
- Frontend changes. The wire protocol and frontend transport client are unaffected; improvements are server-internal.
- Creature inspector endpoint refactoring to use projections (follow-up work that becomes possible once the shared boundary exists).
- New crate creation decision. Whether this is a new `v3-projection` crate or a module within `v3-core` is an open question to be resolved during planning.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should the shared projection types live in a new `v3-projection` crate or as a module within `v3-core`? | TBD -- depends on dependency direction analysis. A new crate avoids adding transport-adjacent types to `v3-core` but adds a crate boundary. | agent+user | open |
| What is the right granularity for creature change events -- per-creature or batched per-tick? | TBD -- per-tick batched events are simpler and avoid high-frequency event overhead, but per-creature events enable finer-grained spatial index updates. | agent | open |
| Should the CLI be a mandatory consumer of the shared projection platform, or should it remain a lightweight direct-read consumer? | TBD -- the CLI's needs are simpler (scalar aggregates, no spatial queries) and may not justify the dependency. | user | open |
| How should the projection store handle the initial bootstrap (tick 0) vs incremental updates? | TBD -- likely a full-build on bootstrap followed by incremental updates, but the transition needs a clean API. | agent | open |
| What is the performance threshold that justifies this work? | TBD -- profiling on larger worlds (512x512+, 5000+ creatures) would quantify the current rebuild cost and establish whether incremental updates provide meaningful improvement at current scale. | user+agent | open |
| Should domain events be emitted synchronously within the tick loop or buffered and drained post-tick? | TBD -- synchronous emission is simpler but adds overhead to hot tick paths; post-tick drain keeps the tick loop clean but requires buffering. | agent | open |
