# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Petri is an artificial life simulator with evolvable hybrid computation graphs (neural + logic nodes). Rust simulation backend with a React/TypeScript web frontend connected via WebSocket (MessagePack) and REST (JSON). Currently at Stage 2 of the roadmap (Rich Environment), with Stage 1 (Minimal Viable Life) complete.

## Commands

### Rust

- **Build all:** `cargo build --workspace`
- **Test all:** `cargo test --workspace`
- **Test single crate:** `cargo test -p petri-core` (or `petri-graph`, `petri-server`, `petri-cli`)
- **Test single test:** `cargo test -p <crate> <test_name> -- --exact`
- **Lint:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Format check:** `cargo fmt --all --check`
- **Format fix:** `cargo fmt --all`
- **Run server:** `cargo run -p petri-server` (serves on `127.0.0.1:4000`)
- **Run CLI:** `cargo run -p petri-cli -- run --ticks 1000 --sample-every 25`
- **Run ablation:** `cargo run -p petri-cli -- ablation --ticks 500`
- **Run benchmark:** `cargo run -p petri-cli --bin stage1_benchmark -- --ticks 200 --assert-min --min-ticks-per-second 30`

### Web (run from `web/` directory)

- **Install deps:** `npm install`
- **Dev server:** `npm run dev` (serves on `127.0.0.1:5173`, expects backend on `127.0.0.1:4000`)
- **Build:** `npm run build` (runs `tsc -b && vite build`)
- **Unit tests:** `npm run test`
- **E2E tests:** `npm run test:e2e`
- **Watch tests:** `npm run test:watch`

### Completion Gate (run all before claiming done)

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`

## Architecture

### Crate Dependency Graph

```
petri-graph  (pure computation, no runtime deps)
    ↑
petri-core   (world state, tick loop, depends on petri-graph)
    ↑
petri-server (axum/tokio REST+WebSocket, depends on petri-core)
petri-cli    (headless runner, depends on petri-core)
```

**Hard rule:** `petri-core` and `petri-graph` must have zero dependencies on `tokio`, `axum`, or async runtime. They are pure computation libraries.

### Crate Responsibilities

- **petri-graph:** Controller graph types (`ComputationGraph`, `NodeKind`, `Edge`), evaluation engine, mutation operators, phenotype color derivation. Pure computation — takes sensor inputs, produces action outputs.
- **petri-core:** World grid, creature storage (`slotmap::HopSlotMap`), food/barrier placement, tick loop (sense→think→act→world-update→emit), action resolution, snapshot serialization, paint API. Owns `WorldConfig`.
- **petri-server:** Axum REST + WebSocket transport, simulation lifecycle (idle→running→paused→pending-restart), startup draft management, runtime config patching, frame streaming (MessagePack via `rmp-serde`).
- **petri-cli:** Headless simulation runner, ablation harness, throughput benchmark binary.

### Rust Module Layout

Each crate uses `lib.rs` as an export-focused surface (`mod` + `pub use`). Logic lives in submodules:

- `petri-core/src/world/`: `mod.rs` (World struct + public API), `tick.rs`, `food.rs`, `perception.rs`, `paint.rs`, `snapshot.rs`, `spawn.rs`, `helpers.rs`, `tests.rs`
- `petri-graph/src/eval/`: `mod.rs`, `evaluate.rs`, `mutate.rs`, `presets.rs`, `node_utils.rs`
- `petri-server/src/app_state/`: `mod.rs`, `lifecycle.rs`, `startup_draft.rs`, `runtime_patch.rs`, `paint.rs`, `viability.rs`, `status.rs`, `types.rs`, `tests.rs`

### Web Frontend

React 18 + TypeScript + Vite. Feature-based folder structure under `web/src/features/simulation/`:

- `components/` — UI panels (StartupDraftPanel, RuntimeTuningPanel, SimulationControls, ViewportCanvas, CreatureInspectorPanel, PaintToolbar, etc.)
- `store/simulationStore.ts` — central state management
- `api/simulationApiClient.ts` — REST client
- `ws/frameStreamClient.ts` — WebSocket MessagePack frame consumer
- `web/src/protocol.ts` — frame type definitions and decoding
- `web/src/canvasRenderer.ts` — direct ImageData buffer rendering

**Key boundary rules:** UI components must not call `fetch` or create `WebSocket` directly. Transport lives in dedicated client modules. Canvas drawing is isolated from transport/control logic.

### Wire Protocol

- WebSocket frames: MessagePack (binary) via `rmp-serde` (Rust) / `@msgpack/msgpack` (JS)
- REST API: JSON
- Food payload is quantized bytes (0..255) for transport efficiency
- When changing frame/protocol types, update Rust producer and TypeScript consumer together

### Simulation Lifecycle

Server boots in `idle` state. Use `POST /simulation/start` to begin. `POST /simulation/restart` rebuilds from current startup draft with a new seed. Web UI exposes start/pause/resume/restart controls. Startup-only config changes during a run trigger a pending-restart state.

## Workflow Conventions

- Write plans in `docs/plans/YYYY-MM-DD-<topic>.md` for multi-step changes
- Use TDD: failing test first, minimal implementation, keep tests green
- Prefer isolated worktrees under `.worktrees/` with `codex/` branch prefix
- Keep commits focused and atomic
- Use deterministic seeds for simulation tests
- Favor targeted test runs during iteration (`cargo test -p <crate> <test_name> -- --exact`); run full workspace suite before completion

## Modularity Rules

- Prefer `pub(crate)` by default; only expose `pub` items that are part of crate API
- If a Rust production file exceeds ~400 lines, evaluate splitting by concern; ~600 lines requires a split
- Integration-style tests go in `crates/<crate>/tests/`, not in `src/lib.rs`
- Separate behavioral refactors from behavior changes (first move/split, then modify in follow-up commits)
- Keep transport in `petri-server`, simulation policy in `petri-core`, graph representation/eval/mutation in `petri-graph`

## Toolchains

- **Rust:** pinned to `1.93.0` via `rust-toolchain.toml`
- **Node:** `25.6.0` via Volta (pinned in `web/package.json`)
- **npm:** `11.8.0` via Volta

## Project Invariants

- `initial_creatures` is best-effort (bounded by occupancy and max creatures)
- `food_growth_rate` must be honored directly (no hidden minimum floor)
- Localhost defaults (`127.0.0.1`) are intentional
- `petri-core`/`petri-graph` must remain runtime-agnostic (no tokio/axum)

## Reference

- `docs/reference/creature-controller-reference.md` — creature/controller structure, node semantics, mutation operator reference
- `petri-architecture.md` — full architecture design document
- `petri-roadmap.md` — implementation roadmap and stage definitions
- `AGENTS.md` — project-level agent instructions
- `web/AGENTS.md` — frontend-specific agent instructions
