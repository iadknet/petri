# V3 Server Query and Projection Spec

Reference specification for the server-local query/projection layer used by the viewport transport refactor.

Status: Active

Related references:
- `v3-server-api-protocol-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-world-grid-spec.md`
- `v3-tick-orchestration-spec.md`

---

## 1. Purpose and Scope

This document defines:
- the canonical server-local `ProjectionSnapshot` contract;
- projection publication and revision semantics;
- running-state read freshness policy for server query endpoints;
- world-static revision rules;
- spatial-index expectations for viewport queries;
- session backpressure/coalescing rules for viewport delivery.

This document does not define:
- HTTP/WS wire shapes (owned by `v3-server-api-protocol-spec.md`);
- internal simulation mutation logic (owned by `v3-core`);
- frontend merge behavior beyond the revision data required at the boundary.

---

## 2. Ownership Boundary

Canonical server ownership split:
- `command`: authoritative simulation lifecycle and mutation
- `query`: projection state and spatial indexes
- `transport`: protocol codecs, sessions, and view assembly
- `http`: request validation and delegation

Rules:
- `query` and `transport` do not mutate simulation state.
- `transport` does not read `Simulation` directly.
- running-state read endpoints consume published projection state rather than taking authoritative live reads from the simulation mutex.

---

## 3. Projection Snapshot Contract

Canonical conceptual shape:

```rust
pub struct ProjectionSnapshot {
    pub projection_revision: u64,
    pub tick: u64,
    pub status: StatusProjection,
    pub health: HealthProjection,
    pub world_static_revision: u64,
    pub barrier_mask: Box<[u8]>,
    pub food_density_u8: Box<[u8]>,
    pub creatures: Box<[CreatureVizRecord]>,
    pub creature_tile_index: CreatureTileIndex,
    pub predation_events: Box<[PredationEventProjection]>,
    pub perf: ProjectionPerf,
}
```

Rules:
- `projection_revision` is monotonic for the server process lifetime.
- every published snapshot is internally coherent; all fields in the snapshot derive from the same publication pass.
- read endpoints and websocket events must not mix fields from different revisions.

---

## 4. Publication Semantics

Publication responsibilities:
- `startup`, `pause`, `step`, `paint`, and config mutations publish synchronously before the handler returns.
- while running, the command loop publishes at the configured projection cadence.

Freshness rules:
- `/status` while running is projection-backed and may lag by up to one publish interval.
- `/snapshot` is projection-backed.
- websocket view payloads are projection-backed.
- paused and idle state reads still reflect the latest synchronously published projection and are therefore exact to the post-mutation simulation state.

---

## 5. World-Static Revision Contract

`world_static_revision` semantics:
- increments only when world topology changes in a way that invalidates the static world payload
- minimum invalidators:
  - startup
  - barrier paint add/remove
  - world-width/height/edge-mode changes

Non-invalidators:
- food growth/consumption
- creature movement, death, or reproduction
- non-topology config changes

---

## 6. Spatial Query Contract

Query-layer responsibilities:
- maintain a server-local spatial index over `CreatureVizRecord`
- serve detail/inspect viewport queries from intersecting index tiles only
- serve overview queries from bounded aggregate grids rather than exact whole-world dynamic payloads

Canonical default:
- fixed tile buckets over the world grid
- tile size is an implementation detail, but it must be stable and benchmarked

Steady-state rule:
- no per-session whole-world creature scan is allowed for viewport delivery
- overview payloads are aggregate-only and intentionally omit predation-event
  streams; exact predation events are detail-view data

---

## 7. Session Coalescing and Backpressure

Per websocket connection:
- exactly one active subscription
- latest `request_id` wins
- stale queued view payloads may be dropped
- only the newest active subscription for the newest published revision must be deliverable

Required event metadata:
- `projection_revision`
- `tick`
- `request_id`

This metadata exists so downstream consumers can reject stale or mixed payloads.

---

## 8. Perf Telemetry Boundary

Projection-facing performance data is transport/runtime wall-clock telemetry, not simulation energy-cost accounting.

Required distinction:
- energy-cost metrics remain owned by applied simulation behavior
- wall-clock projection and transport timings are transport-layer telemetry

Wall-clock metric examples:
- projection capture duration
- view assembly duration
- encoded payload size
- active session count

---

## 9. Policy References

- Determinism scope is canonical in root `AGENTS.md`.
- World geometry and topology semantics are canonical in `v3-world-grid-spec.md`.
- Tick ordering and mutation semantics are canonical in `v3-tick-orchestration-spec.md`.
