# Creature Controller Reference Docs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add durable reference documentation for creature/controller structure and mutation behavior, then link it from the main project entrypoint docs.

**Architecture:** Keep transient execution notes in `docs/plans/` and place stable technical references in a new `docs/reference/` directory. Source all table entries directly from `petri-graph` and `petri-core` to prevent roadmap/spec drift.

**Tech Stack:** Markdown docs, Rust source references (`petri-graph`, `petri-core`), README navigation updates.

### Task 1: Establish durable reference docs location

**Files:**
- Create: `docs/reference/creature-controller-reference.md`
- Modify: `README.md`

**Step 1: Create the creature/controller structure table**
- Document `NodeKind` values with category (input/hidden/output), semantics, and evaluation behavior.
- Document related structural entities: `SensorInputs`, `ActionOutputs`, `Edge`, `ComputationGraph`, and runtime `Creature` state.

**Step 2: Create the mutation function table**
- Document all mutation-related functions in `ComputationGraph` and world-level mutation config shaping in `petri-core`.
- Include trigger rates, clamping/constraints, and whether each function can no-op.

**Step 3: Add discoverability entry**
- Add a short “Reference docs” section to `README.md` linking to the new document.

### Task 2: Verify and finalize

**Files:**
- Verify only (no new files expected)

**Step 1: Run project quality gate**
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

**Step 2: Confirm documentation-only change scope**
- Validate `git diff --stat` shows docs-only edits.
