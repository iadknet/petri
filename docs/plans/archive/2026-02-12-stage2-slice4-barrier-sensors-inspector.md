# Stage 2 Slice 4 (Barrier Sensors + Inspector Exposure) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add barrier-aware sensing (`SensorBarrierDirection`, `SensorBarrierDistance`) and expose those live values in creature inspector payloads/views.

**Architecture:** Extend sensory primitives in `petri-graph` first, then wire barrier nearest-neighbor scan values from `petri-core::World` perception into `SensorInputs` during tick evaluation, and finally surface the new `last_inputs` fields through existing REST/TypeScript inspector contracts. Keep wire-format changes additive and backward-safe (`serde` defaults for newly added fields).

**Tech Stack:** Rust (`petri-core`, `petri-graph`, `petri-server`), TypeScript/React (`web`), cargo test/clippy/fmt, Vitest.

### Task 1: Add failing core and graph regression tests (RED)

**Files:**
- Modify: `crates/petri-core/src/world/tests.rs`
- Modify: `crates/petri-graph/tests/node_support.rs`
- Modify: `crates/petri-graph/tests/palette_eval.rs`
- Modify: `crates/petri-graph/tests/mutation_behavior.rs`

**Step 1: Add failing `petri-core` tests for barrier sensor semantics**
- Add tests for:
  - nearest barrier direction/distance detection
  - default `(0.0, 1.0)` when no barrier is visible
  - radius cutoff behavior
  - inspector payload includes barrier inputs after tick

**Step 2: Add failing `petri-graph` tests for new node kinds**
- Add tests ensuring:
  - `InputBarrierDirection` clamps to `[-1, 1]`
  - `InputBarrierDistance` clamps to `[0, 1]`
  - founder hybrid graph includes barrier sensor nodes
  - founder outputs change when barrier sensor values change

**Step 3: Run targeted tests and confirm expected failure**
Run:
- `cargo test -p petri-core nearest_barrier_sensor_reports_direction_and_distance`
- `cargo test -p petri-graph barrier`

Expected:
- failures for missing fields/node kinds before implementation.

### Task 2: Implement barrier sensor primitives in graph/core (GREEN)

**Files:**
- Modify: `crates/petri-graph/src/types.rs`
- Modify: `crates/petri-graph/src/eval/evaluate.rs`
- Modify: `crates/petri-graph/src/eval/mod.rs`
- Modify: `crates/petri-graph/src/eval/node_utils.rs`
- Modify: `crates/petri-graph/src/eval/presets.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/world/perception.rs`
- Modify: `crates/petri-core/src/world/tick.rs`

**Step 1: Extend graph input surface**
- Add `barrier_direction` and `barrier_distance` to `SensorInputs` with defaults.
- Add `NodeKind::InputBarrierDirection` and `NodeKind::InputBarrierDistance`.
- Map both node kinds in evaluator clamps.
- Mark both as input nodes/non-compute nodes.

**Step 2: Add barrier sensing in world perception**
- Track nearest barrier candidate during scan loop.
- Convert nearest result with existing normalization helper.
- Add barrier direction/distance fields to `PerceptionScan`.
- Optionally expose a test-only helper `nearest_barrier_sensor`.

**Step 3: Feed barrier sensing into creature controller inputs**
- Populate `SensorInputs` in `tick.rs` with new barrier fields from `PerceptionScan`.

**Step 4: Keep founder graph sensor availability**
- Add barrier sensor nodes to founder hybrid preset and wire them into existing movement signal paths with conservative weights.

**Step 5: Re-run targeted tests**
Run:
- `cargo test -p petri-core perception_reports_nearest_barrier_direction_and_distance`
- `cargo test -p petri-graph node_support founder_hybrid_references_barrier_sensors`

Expected:
- passing targeted tests.

### Task 3: Add inspector exposure tests and implement payload/UI wiring

**Files:**
- Modify: `crates/petri-server/tests/snapshot_and_creature.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/features/simulation/components/CreatureInspectorPanel.tsx`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/App.test.tsx`

**Step 1: Add failing API/UI tests (RED)**
- Server integration test should assert `last_inputs.barrier_direction` and `last_inputs.barrier_distance` are present in creature detail JSON.
- UI test should assert inspector renders “Barrier direction” and “Barrier distance”.

**Step 2: Implement additive transport/view changes (GREEN)**
- Add fields to TypeScript `SensorInputs`.
- Update inspector panel to display both values.
- Update MSW fixtures and App test fixture payloads to include fields.

**Step 3: Re-run targeted tests**
Run:
- `cargo test -p petri-server creature_detail_endpoint_returns_last_inputs_outputs_and_events`
- `cd web && npm run test -- App.test.tsx`

Expected:
- all pass.

### Task 4: Final verification gate

**Files:**
- Modify: `README.md` (only if user-visible inspector/sensor behavior is documented there)

**Step 1: Run required project gate**
Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

**Step 2: If README needs an update, add it and rerun affected checks**
- Mention barrier sensors in inspector control summary if needed.

### Boundary Impact

- Crate dependency direction remains unchanged: `petri-graph -> petri-core -> petri-server/petri-cli`.
- Public API/wire-format changes are additive only (`SensorInputs` gains two new fields).
- Test placement remains aligned with current strategy:
  - `petri-core` behavior regression tests in `world/tests.rs`
  - `petri-graph` node/palette behavior in `tests/*.rs`
  - `petri-server` endpoint contract in integration test
  - `web` inspector rendering in existing component/app tests
