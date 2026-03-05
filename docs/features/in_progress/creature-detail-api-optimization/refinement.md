---
title: Creature Detail API Optimization and Documentation
tags: [core, frontend, architecture]
size: M
depends-on: []
status: ready
---

## Problem Statement

The `GET /v3/simulation/creature/:id` endpoint has two compounding issues: a
bandwidth problem and a documentation gap.

### Bandwidth

The frontend polls this endpoint at up to 10 Hz while a creature is selected
(`useCreatureDetail.ts`, `MIN_FETCH_INTERVAL = 100ms`). Every response includes
the full payload: scalar stats, the complete genome, the full 8-byte memory
array, and the entire action log ring buffer (up to 500 entries at default
`ActionLogConfig.capacity`).

Each `ActionLogEntry` serializes to roughly 150 bytes of JSON (8 fields
including tick, action_type, result, direction, energy_before, energy_after,
amount, priority_bid). A full 500-entry action log produces approximately 75 KB
of JSON. At 10 Hz that is ~750 KB/s of transfer for a single inspected creature.

The payload contains fields the frontend does not need on every poll:

- **Genome** (`creature.genome`): Immutable during a creature's lifetime. The
  frontend store (`creatureInspector.ts` line 101) already guards against
  redundant updates: `if (!genome) { genome = detail.genome; }`. Yet the server
  serializes and transmits the full genome on every request.
- **Action log**: Only new entries (those added since the last fetch) carry
  information. In steady state the client already has ~498 of the 500 entries
  and receives 2 new ones per 100 ms interval, but the server sends all 500
  every time.
- **Memory**: 8 bytes, negligible cost, but included for completeness.

Additionally, the Axum server has no HTTP compression middleware. The
`tower-http` crate is not in `v3-server/Cargo.toml` and no `CompressionLayer`
is applied in `lib.rs::router()`. JSON action logs are highly repetitive and
would compress 80-90% with gzip or brotli.

### Documentation gap

The `GET /v3/simulation/creature/:id` endpoint is not documented in
`docs/reference/v3-server-api-protocol-spec.md`. The spec covers lifecycle
endpoints (sections 4.1-4.8), the sample sub-endpoints (4.9-4.10), and the
WebSocket stream (section 5), but the creature detail GET endpoint has no
section. The `action_log` field shape, the `ActionLogEntry` structure, and the
phenotype sub-object are undocumented at the protocol level.

## User Stories / Acceptance Criteria

- As a **frontend developer**, I want to request only action_log entries newer
  than a given tick (`since_tick` query parameter) so that steady-state polling
  transfers ~2 entries instead of ~500, reducing payload size by ~98%.

- As a **frontend developer**, I want to opt out of heavy fields like `genome`
  and `action_log` via a sparse field-selection query parameter (e.g.
  `exclude=genome,action_log`) so that subsequent polls after the initial fetch
  skip immutable or already-cached data.

- As an **operator**, I want the Axum server to apply gzip/brotli compression
  via `tower-http::CompressionLayer` so that all JSON responses are
  transparently compressed, reducing transfer size 80-90% for compressible
  payloads.

- As a **frontend developer**, I want the frontend `useCreatureDetail` hook and
  `api.getCreature()` to use `since_tick` and field exclusion parameters when
  available, merging incremental action_log updates into the local store and
  skipping redundant genome fetches after the first successful response.

- As a **contributor**, I want the `GET /v3/simulation/creature/:id` endpoint
  and its response schema (including `action_log`, `ActionLogEntry`, and
  `phenotype` sub-objects) documented in
  `docs/reference/v3-server-api-protocol-spec.md` as a new section (4.X) so
  that the API reference is complete.

## Proposed Approach

### Server-side (v3-server)

1. **`since_tick` query parameter**: Accept an optional `since_tick: u64` on
   `GET /v3/simulation/creature/:id`. When present, filter `action_log` entries
   to only those with `entry.tick > since_tick`. Return a `latest_tick` field in
   the response so the client knows what value to send next. When absent, return
   the full log (backward compatible).

2. **Sparse field exclusion**: Accept an optional `exclude` query parameter
   (comma-separated field names: `genome`, `memory`, `action_log`). Excluded
   fields are omitted from the JSON response entirely (not set to null). This
   lets the client skip genome after the first fetch and skip action_log when
   using `since_tick` on a separate request is not desired.

3. **`tower-http` CompressionLayer**: Add `tower-http` (with `compression-gzip`
   and `compression-br` features) to `v3-server/Cargo.toml`. Wrap the router
   with `CompressionLayer::new()` in `lib.rs::router()`. This benefits all
   endpoints, not just creature detail.

### Frontend

4. **Incremental action_log fetching**: After the first successful full fetch,
   subsequent polls pass `since_tick={last_seen_tick}` and append only new
   entries to the store's `actionLog` array, trimming to capacity if needed.

5. **Skip genome after first fetch**: After the first successful response that
   includes `genome`, subsequent polls pass `exclude=genome` (or
   `exclude=genome,memory` if memory caching is also desired).

6. **Accept-Encoding**: Modern browsers send `Accept-Encoding: gzip, br` by
   default, so the frontend needs no changes for compression -- it is
   transparent.

### Documentation

7. **New spec section**: Add section 4.X to
   `docs/reference/v3-server-api-protocol-spec.md` documenting:
   - `GET /v3/simulation/creature/:id` request (path param, query params)
   - Full response schema with all fields
   - `ActionLogEntry` field definitions
   - `since_tick` and `exclude` query parameter semantics
   - Error cases (404 creature not found)

## Out of Scope

- **Pagination of action_log**: The ring buffer is capped at 500 entries and
  `since_tick` filtering handles incremental delivery. Cursor-based pagination
  adds complexity for minimal gain at this scale.
- **Binary/MessagePack encoding for creature detail**: The server already uses
  MessagePack for WebSocket transport; extending it to the HTTP creature detail
  endpoint is a separate concern.
- **Rate limiting or server-side push**: Replacing HTTP polling with WebSocket
  push for creature detail is a larger architectural change. The 10 Hz poll cap
  is acceptable with the bandwidth reductions proposed here.
- **Action log persistence or export**: The ring buffer is ephemeral per
  creature lifetime. Persisting it is a different feature.
- **Changes to the `POST/GET /v3/simulation/creature/:id/sample` endpoints**:
  These already have spec documentation (sections 4.9-4.10) and are unrelated
  to the polling bandwidth issue.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `exclude` use a whitelist (`fields=stats,phenotype`) or blacklist (`exclude=genome,action_log`) pattern? Blacklist is simpler for backward compatibility but whitelist is more explicit. | Blacklist (`exclude=`) for backward compat | agent | resolved |
| Should `since_tick` return entries where `tick > since_tick` or `tick >= since_tick`? Strictly-greater avoids duplicating the boundary entry the client already has. | `tick > since_tick` (strictly greater) | agent | resolved |
| Should compression be applied globally or only to specific content types? `tower-http::CompressionLayer` defaults to compressing responses with compressible content types, which is reasonable. | Default CompressionLayer behavior (global) | agent | resolved |
| What minimum response size threshold should compression use? Very small responses (< 256 bytes) may not benefit from compression overhead. | Use tower-http default (no minimum) initially; tune if needed | agent | resolved |
| Should the `latest_tick` response field be added at top level or nested? Top level is simplest and consistent with `tick` in other endpoints. | Top-level `latest_tick`, always present | agent | resolved |
