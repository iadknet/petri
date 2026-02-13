# Stage 2 Slice 5 (Phenotype Color Pipeline) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add deterministic phenotype colors derived from controller/genome structure and surface them in world rendering plus creature inspection payloads.

**Architecture:** Implement a deterministic color derivation function in `petri-graph::ComputationGraph`, then wire the derived color through `petri-core` creature frame/detail snapshots as additive payload fields. Keep the runtime path efficient by caching phenotype color on creature state and updating it when controller structure changes (spawn, offspring creation, snapshot restore). Update frontend protocol/renderer/inspector together so transport and visualization stay in lock-step.

**Tech Stack:** Rust (`petri-graph`, `petri-core`, `petri-server`), TypeScript/React (`web`), MessagePack (`rmp-serde` + `@msgpack/msgpack`), Vitest.

### Task 1: Add failing regression tests (RED)

**Files:**
- Modify: `crates/petri-graph/tests/node_support.rs` (or add `crates/petri-graph/tests/phenotype_color.rs`)
- Modify: `crates/petri-core/src/world/tests.rs`
- Modify: `crates/petri-server/tests/snapshot_and_creature.rs`
- Modify: `web/src/protocol.test.ts`
- Modify: `web/src/canvasRenderer.test.ts`
- Modify: `web/src/test/handlers.ts`

**Step 1: Add graph-level phenotype color tests**
- Add tests proving:
  - same graph structure produces the same color deterministically
  - structural changes (node/edge/parameter differences) alter phenotype color signal

**Step 2: Add core/server payload contract tests**
- Add/adjust `petri-core` tests asserting frame snapshots include phenotype color and that it matches controller-derived value.
- Extend creature detail endpoint test to assert phenotype color presence and 3-channel shape.

**Step 3: Add frontend contract/render tests**
- Extend protocol decode test fixture with `phenotype_color`.
- Add renderer expectation that creature pixels use phenotype color (not hardcoded white).

**Step 4: Run targeted tests and confirm RED**
Run:
- `cargo test -p petri-graph phenotype`
- `cargo test -p petri-core frame_creatures_include_phenotype_color`
- `cargo test -p petri-server creature_detail_endpoint_returns_last_inputs_outputs_and_events`
- `cd web && npm run test -- src/protocol.test.ts src/canvasRenderer.test.ts`

Expected:
- tests fail before implementation due missing phenotype field/method/wiring.

### Task 2: Implement deterministic genome-structure-to-color derivation (GREEN)

**Files:**
- Modify: `crates/petri-graph/src/eval/mod.rs` (and optionally add helper module)
- Modify: `crates/petri-graph/src/lib.rs` (only if exports need adjustment)

**Step 1: Add deterministic phenotype color API**
- Add `ComputationGraph::phenotype_color()` returning compact RGB bytes (3 channels).
- Derive color from deterministic structural fingerprint:
  - include palette, node kinds/parameters, and edges/weights
  - avoid nondeterminism from hashers with randomized seeds
  - canonicalize edge ordering when hashing to avoid incidental ordering artifacts

**Step 2: Keep generated colors visually usable**
- Map fingerprint to HSV/HSL with bounded saturation/value, then convert to RGB.
- Avoid near-black colors that are unreadable on the current viewport background.

**Step 3: Run targeted graph tests and confirm GREEN**
Run:
- `cargo test -p petri-graph phenotype`

### Task 3: Wire phenotype color through core frame/detail payloads

**Files:**
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/world/spawn.rs`
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/snapshot.rs`

**Step 1: Add additive payload fields**
- Extend `CreatureSnapshot` and `CreatureDetail` with `phenotype_color` (`[u8; 3]`).

**Step 2: Keep runtime efficient**
- Add cached phenotype color to internal `Creature` state.
- Set/refresh cache when controller is created or replaced:
  - founder spawn path
  - offspring creation path
  - snapshot restore path

**Step 3: Emit phenotype color in frame/detail builders**
- Populate snapshot/detail color from cached value.
- Keep existing payload fields unchanged.

**Step 4: Re-run targeted core/server tests**
Run:
- `cargo test -p petri-core frame_creatures_include_phenotype_color`
- `cargo test -p petri-server creature_detail_endpoint_returns_last_inputs_outputs_and_events`

### Task 4: Wire frontend protocol, renderer, and inspector

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/canvasRenderer.ts`
- Modify: `web/src/features/simulation/components/CreatureInspectorPanel.tsx`
- Modify: `web/src/protocol.test.ts`
- Modify: `web/src/canvasRenderer.test.ts`
- Modify: `web/src/test/handlers.ts`
- Modify: additional fixture files that construct `CreatureSnapshot`/`CreatureDetail` literals

**Step 1: Update protocol typing**
- Add `phenotype_color` to `CreatureSnapshot` and `CreatureDetail`.

**Step 2: Render phenotype colors in viewport**
- Use `creature.phenotype_color` RGB channels in canvas render loop.
- Preserve barrier/food layering and creature-over-barrier precedence.

**Step 3: Expose phenotype color in inspector**
- Add a compact swatch + channel/hex readout in creature inspector panel.

**Step 4: Re-run targeted web tests**
Run:
- `cd web && npm run test -- src/protocol.test.ts src/canvasRenderer.test.ts`

### Task 5: Completion verification gate

**Step 1: Run required project checks**
Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run test`
- `cd web && npm run test:e2e`
- `cd web && npm run build`

**Step 2: Documentation touch-up (if needed)**
- Update `README.md` only if user-visible viewport/inspector behavior needs explicit mention.

### Boundary Impact

- Crate dependency direction remains unchanged: `petri-graph -> petri-core -> petri-server/petri-cli`.
- Public API/wire-format changes are additive only (`phenotype_color` on creature frame/detail payloads).
- Test placement remains aligned with repository policy:
  - graph behavior in `crates/petri-graph/tests/*`
  - simulation behavior in `crates/petri-core/src/world/tests.rs`
  - server endpoint contract in `crates/petri-server/tests/*`
  - frontend protocol/rendering in `web/src/*.test.ts*`
