---
title: Multi-Crate v3-server Split
tags: [core, architecture]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

The `v3-server` crate (~3,200 lines across 30 files) currently houses four distinct
architectural boundaries within a single crate:

1. **Command** (`command/`, `handlers/`) -- mutates simulation state: lifecycle
   transitions (startup, start, pause, step), the tick run-loop, paint operations,
   creature sampling, and config patching. Depends on `v3-core` for `Simulation`,
   `run_tick`, `seed_simulation`, and `SimulationConfig`.

2. **Query** (`query/`) -- read-side projection layer: `ProjectionStore` publishes
   immutable snapshots built from `WsFrame` data, `CreatureTileIndex` provides
   spatial lookups, and `cache` holds dirty-rect/food-density/barrier-mask
   materialization. Depends on shared `state` types but not on `v3-core` simulation
   mutators.

3. **Transport** (`transport/`, `ws.rs`) -- WebSocket session management,
   client/server protocol codec (MessagePack), view assembly (overview and detail
   payloads), and the frame delivery cursor. Depends on query projections and shared
   protocol types, but not directly on `v3-core` simulation internals.

4. **Shared types and wiring** (`state.rs`, `types.rs`, `error.rs`, `app_state.rs`,
   `lib.rs`, `bin/server.rs`) -- `AppState`, `WsFrame`, `StatusPayload`,
   `FramePayload`, protocol version, config digest, error envelope, and HTTP router
   assembly.

These boundaries are already documented in `docs/strategy/architecture.md` as
explicit dependency directions (`command -> v3-core`, `query -> command-published
projection`, `transport -> query state only`, `http -> delegates to
command/query/transport`). However, the single-crate structure cannot enforce these
dependency directions at compile time. Nothing prevents a transport module from
directly importing `v3-core::simulation::run_tick` or a query module from acquiring
a mutable simulation handle. The boundaries exist by convention and code review
only.

Several `//! Transitional ...` doc comments throughout the crate (in `command/`,
`query/cache.rs`, `transport/session.rs`, `transport/view_assembler.rs`,
`http/*.rs`, `app_state.rs`) indicate that the current module layout was designed as
a stepping stone toward eventual crate extraction.

Splitting into multiple crates would:

- **Enforce dependency direction at compile time.** A `v3-transport` crate that
  depends on `v3-query` but not `v3-core` cannot accidentally reach into simulation
  internals.
- **Enable independent compilation.** Changes to transport protocol types would not
  recompile command-side simulation logic.
- **Improve contribution clarity.** Each crate's `Cargo.toml` makes its dependency
  surface explicit, reducing onboarding friction.
- **Align with the architecture doc's stated boundary model.** The split would
  formalize what is currently a documented aspiration.

## Proposed Crate Structure

| New crate | Current modules | Key dependencies |
|-----------|----------------|------------------|
| `v3-server-types` | `state.rs`, `types.rs`, `error.rs` | `v3-core` (for `SimulationConfig`, `PaintTool`, `PaintStats`), `serde`, `serde_json`, `sha2`, `hex`, `axum` (for `IntoResponse` on `AppError`) |
| `v3-query` | `query/projection.rs`, `query/cache.rs`, `query/spatial_index.rs` | `v3-server-types`, `v3-core` (for `build_ws_frame` via a trait or moved function) |
| `v3-transport` | `transport/protocol.rs`, `transport/session.rs`, `transport/session_registry.rs`, `transport/view_assembler.rs` | `v3-server-types`, `v3-query`, `serde`, `rmp-serde` |
| `v3-server` (shell) | `lib.rs`, `app_state.rs`, `handlers/*`, `http/*`, `ws.rs`, `command/*`, `bin/server.rs` | `v3-core`, `v3-server-types`, `v3-query`, `v3-transport`, `axum`, `tokio` |

## User Stories / Acceptance Criteria

- As a developer, I want compile-time enforcement of the command/query/transport
  dependency directions so that boundary violations are caught before code review.
- As a developer, I want to modify transport protocol types without recompiling
  simulation tick logic so that iteration on the wire format is faster.
- As a developer, I want each crate's `Cargo.toml` to document its true dependency
  surface so that I can understand module responsibilities at a glance.
- As a developer, I want the existing test suite (unit tests, integration tests, and
  viability tests) to pass without behavioral changes after the split so that the
  refactor is purely structural.
- As a developer, I want the `http` module's re-export shims (currently one-line
  `pub use` stubs) to be collapsed or removed during the split so that the final
  structure has no vestigial indirection layers.

## Out of Scope

- Changing any runtime behavior, protocol format, or API contract. This is a pure
  structural refactor.
- Splitting `v3-core` into sub-crates. That is a separate concern with different
  stability criteria.
- Introducing a shared incremental query/projection platform (see `ideas.md`
  brainstorm). The split should accommodate that future work but not implement it.
- Changing the WebSocket transport to a different protocol (e.g., gRPC, SSE).
- Moving `build_ws_frame` logic out of `handlers/lifecycle.rs`. That function
  straddles the command/query boundary today (it reads `SimHandle` and produces
  `WsFrame`). The split will need to decide where it lives, but refactoring its
  internal logic is out of scope.
- Frontend changes. The split is entirely backend/Rust-side.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Are the command/query/transport boundaries stable enough to split now, or should more features land first? | Needs assessment -- the original brainstorm intentionally deferred this. | User | open |
| Where should `build_ws_frame` live? It reads `SimHandle` (command-side) but produces `WsFrame` (shared type consumed by query). Options: keep in command crate, move to a shared boundary trait, or extract as a standalone adapter. | Undecided | User+Agent | open |
| Should `AppError` stay in `v3-server-types` or in the shell crate? It imports `axum::response::IntoResponse`, which would add axum as a dependency of the types crate. | Undecided -- could use a feature flag, keep in shell, or split into a domain error + axum adapter. | User+Agent | open |
| Should the `http/` re-export shim layer be removed entirely, or does it serve a useful routing-organization purpose? | Likely remove -- the shims are all single-line `pub use` re-exports with `//! Transitional` comments. | User+Agent | open |
| How should benchmarks (`benches/transport.rs`) be assigned across crates? | Undecided -- likely moves to `v3-transport` or stays in shell with integration-level deps. | User+Agent | open |
| Should `v3-server-types` be named `v3-server-types` or `v3-server-contracts` for consistency with `v3-core/contracts`? | Undecided | User+Agent | open |
