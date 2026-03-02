# V3 Viewport Transport Server Companion

**Parent plan:** `docs/plans/2026-03-02-v3-viewport-transport-plan.md`

**Goal:** Refactor `v3-server` into explicit command/query/transport boundaries and deliver the projection-backed server transport foundation for `v3alpha2`.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Included: server module realignment, projection publication, projection-backed `/status` and `/snapshot`, websocket session registry, revision semantics, dirty-rect invalidation, and bounded spatial view assembly. Excluded: frontend consumption, CLI transport changes, and core-runtime micro-optimizations.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/reference/v3-server-api-protocol-spec.md` | Update HTTP/WS contract ownership |
| `docs/reference/v3-server-query-projection-spec.md` | Define canonical projection semantics |
| `docs/reference/v3-evolution-observability-spec.md` | Clarify transport-facing perf telemetry |
| `docs/strategy/architecture.md` | Add server command/query/transport ownership |

**Supersedes:** none

**Superseded-By:** none

**See also:**
- `docs/plans/2026-03-02-v3-viewport-transport-plan.md`
- `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md`
- `docs/reference/v3-server-query-projection-spec.md`

## Goal Alignment

- **GP-01:** Bounded view assembly removes whole-world dynamic payload generation from steady-state transport.
- **GP-02:** The server gains a clean read-side boundary instead of coupling handlers and websocket transport directly to `Simulation`.
- **GP-03:** Projection publication, bootstrap snapshot, and session semantics are all test-driven and benchmarked.
- **GP-04:** Status, health, and perf payloads remain behavior-backed through projection publication.

## Boundary Impact

- `command` owns mutation and simulation lifecycle only.
- `query` owns projection state, spatial indexes, and read freshness policy.
- `transport` owns protocol codecs, sessions, subscriptions, and view assembly only.
- `http` owns validation and delegation only.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-server/src/handlers/lifecycle.rs` | change | It currently mixes lifecycle mutation, tick loop, frame construction, and websocket delivery. |
| `v3/crates/v3-server/src/state.rs` | change | It currently combines simulation authority, payload types, and broadcast transport. |
| `v3/crates/v3-core` authoritative simulation | keep | The projection layer reads from command-published snapshots rather than changing core ownership. |
| `docs/reference/v3-server-api-protocol-spec.md` | keep | HTTP/WS wire ownership remains server-owned; only the structure changes. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| How fresh are running-state reads? | Projection-backed while running; synchronous republish after mutating endpoints. | user+agent | resolved |
| How does initial bootstrap work? | `GET /v3/simulation/snapshot` without viewport params returns a bootstrap overview snapshot. | user+agent | resolved |
| How are mixed-publish reads prevented? | Every read payload carries `projection_revision`; all fields in a response come from one published snapshot. | user+agent | resolved |
| How should lagging sessions behave? | Drop stale intermediate views; only the latest subscription request must be honored. | user+agent | resolved |

## Execution Requirements

- Any Rust code changes in this companion must explicitly load and apply `rust-skills` before editing code or reviewing implementation.
- Tests for new server behavior must be written first and observed failing before implementation code is added.
- Benchmark gates and profiling checkpoints must be run according to the shared verification matrix before phase-complete claims.
- After every task in this companion that changes Rust code, run a recursive `rust-skills` review pass, fix every new finding introduced by that task, and rerun the review until it reports no new findings.
- No task in this companion passes while a `rust-skills` recursive review still has unresolved new findings.

## Architecture Target

```text
v3/crates/v3-server/src/
  app_state.rs
  command/
    mod.rs
    service.rs
    run_loop.rs
  query/
    mod.rs
    projection.rs
    cache.rs
    spatial_index.rs
  transport/
    mod.rs
    protocol.rs
    session.rs
    session_registry.rs
    view_assembler.rs
  http/
    mod.rs
    lifecycle.rs
    status.rs
    snapshot.rs
    paint.rs
    creature.rs
  error.rs
  lib.rs
```

## Projection Publication Contract

- `ProjectionSnapshot` includes:
  - `projection_revision`
  - `tick`
  - `status`
  - `health`
  - `world_static_revision`
  - `barrier_mask`
  - quantized food grid
  - creature visualization records
  - creature tile index
  - recent predation events
  - transport perf metrics
- One monotonic `projection_revision` is issued per publish.
- No HTTP response or websocket event may mix fields from multiple revisions.
- `startup`, `pause`, `step`, `paint`, and config mutations republish synchronously before returning.

## Query Boundary Contract

- `/status`, `/snapshot`, and websocket view assembly use the published projection while the sim is running.
- The query layer does not read `Simulation` directly after projection publication exists.
- `world_static_revision` changes only when barriers or topology change.
- Spatial indexes are server-local and derived from published visualization records, not from per-session scans.

## Session and View Contract

- Each websocket connection holds exactly one active subscription.
- Client messages:
  - `subscribe_view`
  - `unsubscribe_view`
- Server view payloads echo:
  - `request_id`
  - `projection_revision`
  - `tick`
- Latest `request_id` wins.
- If a session lags, intermediate view payloads may be dropped, but the latest active view must still be deliverable.

## Task List

### Task 1: Introduce server module boundaries

**Files:**
- Create: `v3/crates/v3-server/src/app_state.rs`
- Create: `v3/crates/v3-server/src/command/mod.rs`
- Create: `v3/crates/v3-server/src/command/service.rs`
- Create: `v3/crates/v3-server/src/command/run_loop.rs`
- Create: `v3/crates/v3-server/src/query/mod.rs`
- Create: `v3/crates/v3-server/src/query/projection.rs`
- Create: `v3/crates/v3-server/src/query/cache.rs`
- Create: `v3/crates/v3-server/src/query/spatial_index.rs`
- Create: `v3/crates/v3-server/src/transport/mod.rs`
- Create: `v3/crates/v3-server/src/transport/protocol.rs`
- Create: `v3/crates/v3-server/src/transport/session.rs`
- Create: `v3/crates/v3-server/src/transport/session_registry.rs`
- Create: `v3/crates/v3-server/src/transport/view_assembler.rs`
- Create: `v3/crates/v3-server/src/http/mod.rs`
- Modify: `v3/crates/v3-server/src/lib.rs`

### Task 2: Add projection publication and projection-backed reads

**Files:**
- Modify: `v3/crates/v3-server/src/command/service.rs`
- Modify: `v3/crates/v3-server/src/command/run_loop.rs`
- Modify: `v3/crates/v3-server/src/query/projection.rs`
- Modify: `v3/crates/v3-server/src/http/status.rs`
- Create: `v3/crates/v3-server/src/http/snapshot.rs`
- Modify: `v3/crates/v3-server/src/app_state.rs`

### Task 3: Add session-aware transport scaffolding

**Files:**
- Modify: `v3/crates/v3-server/src/transport/protocol.rs`
- Modify: `v3/crates/v3-server/src/transport/session.rs`
- Modify: `v3/crates/v3-server/src/transport/session_registry.rs`
- Modify: `v3/crates/v3-server/src/ws.rs`

### Task 4: Add bounded view assembly and invalidation hooks

**Files:**
- Modify: `v3/crates/v3-server/src/query/spatial_index.rs`
- Modify: `v3/crates/v3-server/src/transport/view_assembler.rs`
- Modify: `v3/crates/v3-server/src/http/paint.rs`
- Modify: `v3/crates/v3-server/src/query/cache.rs`

### Task 5: Update tests and remove monolithic transport

**Files:**
- Modify: `v3/crates/v3-server/tests/server.rs`
- Remove or retire old `WsFrame`/`/frame` assertions as the cutover lands.

## Acceptance Criteria

- No handler or websocket delivery path builds dynamic transport directly from `Simulation` once the projection boundary lands.
- `GET /v3/simulation/snapshot` supports bootstrap mode without viewport params.
- `/status` while running is projection-backed.
- All view payloads carry revision metadata.
- Whole-world creature scans are removed from steady-state per-session view assembly.
- Every server task has a recorded recursive `rust-skills` review pass with no new findings before task completion.

## Verification Commands

- Use `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` as the command source of truth.

## Risks and Rollback

- Projection publication bugs can create stale or mixed reads; keep response assembly single-snapshot only.
- Session handling introduces stateful concurrency risk; prefer latest-value semantics and bounded queues.
- If protocol cutover stalls, keep the command/query split and projection publication while reverting public view-message changes.

**Review cycles:** 1
