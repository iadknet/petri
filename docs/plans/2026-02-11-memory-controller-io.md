# Memory Controller IO Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let creature controllers read from and write to the per-creature memory register every tick.

**Architecture:** Add one graph input node (`InputMemoryRead`) and one graph output node (`OutputMemoryWrite`) in `petri-graph`, then wire `petri-core` tick execution to feed/read the active memory bit and persist writes back into the creature memory register. Keep existing movement/eat/reproduce behavior unchanged.

**Tech Stack:** Rust (`petri-graph`, `petri-core`), existing DAG evaluator, world tick loop, Cargo test suite.

### Task 1: Add Failing Tests (RED)

**Files:**
- Modify: `crates/petri-graph/tests/node_support.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Write failing evaluator test for memory input/output**

Add a graph test that:
- uses `NodeKind::InputMemoryRead` to drive `OutputMemoryWrite`
- verifies `memory_write` output follows input and is clamped to `[0, 1]`

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-graph memory_io_nodes_are_supported_and_clamped -- --exact`
Expected: FAIL because node kinds/output field do not exist yet.

**Step 3: Write failing world tick integration test**

Add a `petri-core` test with a custom controller that:
- reads memory input bit
- writes inverted value to memory output
- verifies register slot changes after tick

**Step 4: Run test to verify it fails**

Run: `cargo test -p petri-core world::tests::memory_write_output_updates_creature_register -- --exact`
Expected: FAIL because world tick does not yet feed/apply memory IO.

### Task 2: Implement Memory IO In `petri-graph`

**Files:**
- Modify: `crates/petri-graph/src/types.rs`
- Modify: `crates/petri-graph/src/eval/evaluate.rs`
- Modify: `crates/petri-graph/src/eval/mod.rs`
- Modify: `crates/petri-graph/src/eval/node_utils.rs`
- Modify: `crates/petri-graph/src/eval/presets.rs`
- Modify: `crates/petri-graph/tests/mutation_behavior.rs` (classification helpers)

**Step 1: Extend node and I/O types**

Add:
- `SensorInputs.memory_read: f32`
- `ActionOutputs.memory_write: f32`
- `NodeKind::InputMemoryRead`
- `NodeKind::OutputMemoryWrite`

**Step 2: Implement evaluation logic**

In evaluator:
- map `InputMemoryRead` from `inputs.memory_read.clamp(0.0, 1.0)`
- include `OutputMemoryWrite` in output node summation
- clamp assigned `outputs.memory_write` to `[0, 1]`

**Step 3: Update node classification/mutation compatibility**

Treat new nodes as input/output (not hidden) in:
- compute node counting
- node utility classifiers
- mutation test hidden-node match lists

**Step 4: Update presets/founders for evolvability**

Insert `InputMemoryRead` with input nodes and `OutputMemoryWrite` with outputs for all presets/founder graphs, keeping edges valid (forward DAG ordering intact).

### Task 3: Implement Memory IO In `petri-core` Tick Loop

**Files:**
- Modify: `crates/petri-core/src/world/tick.rs`

**Step 1: Feed memory_read input**

For each creature tick:
- pick active register slot index deterministically from current age and register length
- map bit value to `0.0`/`1.0` and pass into `SensorInputs.memory_read`

**Step 2: Apply memory_write output**

After evaluation:
- threshold `outputs.memory_write > 0.5`
- write bit back into active register slot

### Task 4: Verify Green

**Step 1: Run targeted tests**

Run:
- `cargo test -p petri-graph memory_io_nodes_are_supported_and_clamped -- --exact`
- `cargo test -p petri-core world::tests::memory_write_output_updates_creature_register -- --exact`
- `cargo test -p petri-graph`
- `cargo test -p petri-core`

Expected: PASS.

**Step 2: Run full completion gate**

Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

Expected: all PASS.
