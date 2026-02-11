# Creature Memory Register Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an evolvable per-creature memory register size trait with founders starting at 32 bits and offspring bounded by a hard maximum of 1024 bits.

**Architecture:** Keep memory-register state in `petri-core` creature runtime state, mutate size only at reproduction, and enforce fixed bounds in one helper to keep behavior deterministic and testable. Start with internal simulation behavior only to avoid unnecessary wire/protocol surface changes.

**Tech Stack:** Rust (`petri-core`), existing world tick/reproduction loop, workspace tests via Cargo.

### Task 1: Add Failing Tests For Memory Register Behavior

**Files:**
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Write failing test for founder register size**

Add a test asserting all seed creatures start with exactly 32 bits of memory capacity.

**Step 2: Run test to verify it fails**

Run: `cargo test -p petri-core founders_start_with_32_bit_memory_register -- --exact`
Expected: FAIL because memory-register field/logic does not exist yet.

**Step 3: Write failing test for evolvable bounded offspring size**

Add a test that drives reproduction and asserts offspring memory register size:
- can diverge from parent size over repeated births (evolvable)
- stays within `[1, 1024]`

**Step 4: Run test to verify it fails**

Run: `cargo test -p petri-core offspring_memory_register_size_evolves_within_bounds -- --exact`
Expected: FAIL because mutation/bounds logic does not exist yet.

### Task 2: Implement Memory Register Trait In Core World State

**Files:**
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/world/spawn.rs`
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/snapshot.rs`

**Step 1: Add memory register state to runtime creature model**

Introduce:
- bounded constants (`32` founder bits, `1024` max bits)
- per-creature register storage
- helper methods for initializing and mutating register size deterministically with existing RNG flow

**Step 2: Wire founders to small default register**

Ensure `spawn_random_creature` assigns 32-bit register state for seed creatures.

**Step 3: Wire offspring size mutation with hard bounds**

During reproduction, inherit parent register size and apply mutation step(s) before child creation, then clamp to `[1, 1024]`.

**Step 4: Preserve snapshot round-trip behavior**

Include register state in world snapshot save/load so imports/exports do not drop memory-register data.

### Task 3: Make Tests Green And Run Full Gates

**Files:**
- Modify: `crates/petri-core/src/world/tests.rs` (if fixture updates are required)
- Modify: `README.md` (only if user-visible behavior doc update is warranted)

**Step 1: Run targeted tests until green**

Run:
- `cargo test -p petri-core founders_start_with_32_bit_memory_register -- --exact`
- `cargo test -p petri-core offspring_memory_register_size_evolves_within_bounds -- --exact`

Expected: PASS.

**Step 2: Run touched crate tests**

Run: `cargo test -p petri-core`
Expected: PASS.

**Step 3: Run full required completion gate**

Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

Expected: all PASS.
