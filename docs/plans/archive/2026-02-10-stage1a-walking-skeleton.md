# Stage 1a Walking Skeleton Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver the first runnable end-to-end vertical slice: Rust simulation tick loop + WebSocket stream + browser canvas renderer with moving creatures, food, energy, reproduction, and death.

**Architecture:** Build a Rust workspace with `petri-core` (pure simulation), `petri-server` (axum WebSocket + REST), and `petri-cli` (headless runner). Use messagepack frames for streaming world snapshots. Implement a minimal React + Vite frontend that renders directly to `<canvas>` from streamed frames.

**Tech Stack:** Rust (`slotmap`, `rand`, `serde`, `axum`, `tokio`, `rmp-serde`), TypeScript (`react`, `vite`, `@msgpack/msgpack`), Canvas `ImageData`.

### Task 1: Workspace Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `crates/petri-core/Cargo.toml`
- Create: `crates/petri-server/Cargo.toml`
- Create: `crates/petri-cli/Cargo.toml`

**Step 1: Write failing build check**

Run: `cargo check -p petri-core`
Expected: FAIL (`Cargo.toml` missing)

**Step 2: Add workspace and crate manifests**

- Add workspace members and shared deps.
- Add minimal crate manifests for `petri-core`, `petri-server`, `petri-cli`.

**Step 3: Re-run build check**

Run: `cargo check --workspace`
Expected: FAIL (no source files yet)

### Task 2: Core Simulation (No Graph Yet)

**Files:**
- Create: `crates/petri-core/src/lib.rs`
- Create: `crates/petri-core/src/config.rs`
- Create: `crates/petri-core/src/world.rs`
- Create: `crates/petri-core/src/types.rs`
- Test: `crates/petri-core/src/world.rs` (inline unit tests)

**Step 1: Write failing tests**

Add tests for:
- world initializes with expected creature count and bounds
- `tick()` changes creature positions over time
- creatures lose energy and die
- creatures can eat food and gain energy
- high-energy creatures can reproduce (with cap checks)

**Step 2: Run tests to verify failure**

Run: `cargo test -p petri-core`
Expected: FAIL (`World` not implemented)

**Step 3: Implement minimal core**

- Grid world as `Vec<Cell>` with row-major indexing.
- Creature storage with `HopSlotMap`.
- Random-walk controller (8-direction move).
- Food spawn/growth, eat action, energy decay.
- Reproduction with offspring mutation placeholder (RNG seed inheritance + generation increment).
- Death and event-log ring buffer.
- Snapshot structs for transport (`WorldFrame`, `CreatureSnapshot`).

**Step 4: Re-run tests**

Run: `cargo test -p petri-core`
Expected: PASS

### Task 3: Server Stream + Config API

**Files:**
- Create: `crates/petri-server/src/main.rs`
- Create: `crates/petri-server/src/app_state.rs`
- Create: `crates/petri-server/src/api.rs`
- Create: `crates/petri-server/src/sim_loop.rs`

**Step 1: Write failing server smoke test**

Add a basic async test ensuring:
- `/health` returns 200
- WebSocket endpoint upgrades successfully

Run: `cargo test -p petri-server`
Expected: FAIL (server not implemented)

**Step 2: Implement server**

- Shared state wraps world config + pause state.
- Background simulation loop ticks at configurable TPS.
- Each tick publishes MessagePack world frame via `broadcast`.
- `GET /health`, `GET /config`, `PATCH /config`.
- `GET /ws` streams MessagePack binary frames.

**Step 3: Re-run server tests**

Run: `cargo test -p petri-server`
Expected: PASS

### Task 4: CLI Runner

**Files:**
- Create: `crates/petri-cli/src/main.rs`

**Step 1: Write failing run check**

Run: `cargo run -p petri-cli -- --ticks 5`
Expected: FAIL (binary missing)

**Step 2: Implement headless loop**

- Run world for N ticks.
- Print population, average energy, and tick count per interval.

**Step 3: Re-run check**

Run: `cargo run -p petri-cli -- --ticks 5`
Expected: PASS with stats output

### Task 5: Web Renderer Skeleton

**Files:**
- Create: `web/package.json`
- Create: `web/tsconfig.json`
- Create: `web/index.html`
- Create: `web/vite.config.ts`
- Create: `web/src/main.tsx`
- Create: `web/src/App.tsx`
- Create: `web/src/styles.css`
- Create: `web/src/protocol.ts`
- Create: `web/src/canvasRenderer.ts`

**Step 1: Write failing frontend build check**

Run: `cd web && npm run build`
Expected: FAIL (`package.json` missing)

**Step 2: Implement app**

- Connect to `/ws` and decode MessagePack frames.
- Render food + creatures to canvas via `ImageData`.
- Add pause/resume and TPS slider wired to `PATCH /config`.
- Show population and average energy.

**Step 3: Re-run build check**

Run: `cd web && npm run build`
Expected: PASS

### Task 6: Integration Verification

**Files:**
- Modify: `README.md`

**Step 1: Add run instructions**

- `cargo run -p petri-server`
- `cd web && npm install && npm run dev`
- `cargo run -p petri-cli -- --ticks 1000`

**Step 2: Verify end-to-end commands**

Run:
- `cargo test --workspace`
- `cargo run -p petri-cli -- --ticks 20`
- `cargo run -p petri-server`

Expected:
- tests pass
- CLI prints stats
- server starts and logs tick activity

**Step 3: Commit**

```bash
git add Cargo.toml crates web README.md docs/plans/2026-02-10-stage1a-walking-skeleton.md
git commit -m "feat: add stage1a walking skeleton simulation pipeline"
```
