# Cognition-First Tick Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a cognition-first creature lifecycle where per-tick internal deliberation is energy-bounded, haltable, and followed by at most one world interaction (including explicit no-op).

**Architecture:** Extend controller I/O in `petri-graph` for halt/no-op, confidence introspection, and explicit energy-awareness inputs (`energy_start_tick`, `energy_spent_tick`, `energy_remaining`), then refactor `petri-core` tick semantics from multi-action-per-tick to think-loop plus one-action arbitration. Wire config/runtime/protocol updates through server and web contracts, preserving deterministic replay by using per-creature seeded RNG for tie-breaks.

**Tech Stack:** Rust (`petri-core`, `petri-graph`, `petri-server`, `petri-cli`), TypeScript (`web` protocol/tests), Cargo, npm.

## Boundary Impact

- Crate dependency direction remains unchanged: `petri-graph -> petri-core -> petri-server/petri-cli`.
- Public API/wire-format changes are expected for controller I/O and creature-inspector diagnostics fields.
- Test migration approach:
  - graph contract tests in `crates/petri-graph/tests/*`
  - world lifecycle/arbitration tests in `crates/petri-core/src/world/tests.rs`
  - transport/protocol tests in `crates/petri-server/tests/*` and `web/src/protocol.test.ts`

### Task 1: Add graph contract tests for new cognition I/O (RED)

**Files:**
- Modify: `crates/petri-graph/tests/node_support.rs`
- Modify: `crates/petri-graph/tests/mutation_behavior.rs`

**Step 1: Add failing tests for new outputs and inputs**

- Add tests for `halt` and `no_op` outputs.
- Add tests for introspection input channels (previous-step and running-max confidence arrays).
- Add tests for new energy inputs (`energy_start_tick`, `energy_spent_tick`, `energy_remaining`) and their expected per-step update rules.

**Step 2: Add failing tests for clamping and movement-confidence derivation helper compatibility**

- Ensure movement confidence consumers can derive from `sqrt(move_x^2 + move_y^2)` with clamped axis outputs.

**Step 3: Run targeted tests to confirm RED**

Run:
- `cargo test -p petri-graph node_support -- --nocapture`
- `cargo test -p petri-graph mutation_behavior -- --nocapture`

Expected: FAIL on missing node kinds/fields/eval support.

### Task 2: Implement graph I/O extensions (GREEN)

**Files:**
- Modify: `crates/petri-graph/src/types.rs`
- Modify: `crates/petri-graph/src/eval/evaluate.rs`
- Modify: `crates/petri-graph/src/eval/mod.rs`
- Modify: `crates/petri-graph/src/eval/node_utils.rs`
- Modify: `crates/petri-graph/src/eval/presets.rs`

**Step 1: Extend types**

- Add `NodeKind` variants for new introspection inputs.
- Add `NodeKind` variants for `OutputHalt` and `OutputNoOp`.
- Add corresponding fields to `SensorInputs` and `ActionOutputs`, including the three energy-awareness inputs.

**Step 2: Extend evaluator behavior**

- Read introspection input values from `SensorInputs`.
- Populate `halt` and `no_op` output fields with clamping.

**Step 3: Update node classification and preset/founder graphs**

- Ensure new nodes are correctly treated as input/output, not hidden.
- Include new outputs in founder/preset output surfaces.

**Step 4: Re-run targeted graph tests to confirm GREEN**

Run:
- `cargo test -p petri-graph node_support -- --nocapture`
- `cargo test -p petri-graph mutation_behavior -- --nocapture`

Expected: PASS.

### Task 3: Add world-level cognition semantics tests (RED)

**Files:**
- Modify: `crates/petri-core/src/world/tests.rs`

**Step 1: Add failing tests for think-loop termination rules**

- `halt` terminates loop.
- Energy exhaustion terminates loop and can cause death.

**Step 2: Add failing tests for one-action arbitration**

- Exactly one world interaction max per tick.
- Explicit `no_op` can win and result in no world interaction.

**Step 3: Add failing tests for introspection channels**

- previous-step confidence inputs update correctly.
- running-max confidence inputs update correctly.
- energy-awareness inputs update correctly (`energy_start_tick` stable per tick; spent/remaining refreshed per think step).

**Step 4: Add failing tests for tie-break determinism**

- Equal final confidences resolved via per-creature seeded RNG deterministically across fixed seeds/snapshots.

**Step 5: Run targeted RED checks**

Run: `cargo test -p petri-core world::tests:: -- --nocapture`
Expected: FAIL for missing semantics.

### Task 4: Implement config/runtime fields for cognition cost

**Files:**
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-server/src/app_state/types.rs`
- Modify: `crates/petri-server/src/app_state/runtime_patch.rs`
- Modify: `crates/petri-server/src/app_state/startup_draft.rs` (if startup patch surface requires parity)

**Step 1: Add `energy_per_think_step` config field and defaults**

- Use strictly positive default.

**Step 2: Update runtime patch validation**

- Enforce non-negative/positive constraints consistent with design.

**Step 3: Add/adjust tests for config and runtime patch behavior**

- Verify field round-trip and clamp/validation semantics.

### Task 5: Refactor world tick lifecycle to cognition-first model

**Files:**
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/types.rs`

**Step 1: Introduce think-loop state per creature per tick**

- Maintain previous-step and running-max confidence trackers.
- Capture normalized tick-start energy once per tick for `energy_start_tick`.
- Recompute normalized `energy_spent_tick` and `energy_remaining` at the start of every think step.
- Feed these trackers back into controller inputs each step.

**Step 2: Implement halt/energy termination logic**

- Stop think loop on `halt` or insufficient energy for next think step.

**Step 3: Implement final-thought arbitration**

- Compute candidate confidences from final outputs.
- Derive move confidence from vector magnitude.
- Allow `no_op` as explicit candidate.
- Break exact ties using per-creature RNG.

**Step 4: Enforce one-world-action max**

- Execute only selected action path.
- Preserve existing legality checks/penalties per selected action type.

**Step 5: Extend diagnostics and inspector state**

- Add think-step counters and arbitration outcome visibility.

### Task 6: Update transport contracts and web protocol

**Files:**
- Modify: `crates/petri-server/tests/snapshot_and_creature.rs`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/protocol.test.ts`
- Modify: `web/src/test/handlers.ts`
- Modify: `web/src/features/simulation/components/CreatureInspectorPanel.tsx` (if new fields displayed)

**Step 1: Add failing protocol/server tests for new fields**

- Ensure `CreatureDetail` payload supports new cognition diagnostics/state.

**Step 2: Implement protocol type updates**

- Keep type names and field contracts synchronized across Rust/TS.

**Step 3: Re-run targeted tests**

Run:
- `cargo test -p petri-server snapshot_and_creature -- --nocapture`
- `cd web && npm test -- --run protocol`

Expected: PASS.

### Task 7: Regression and verification gates

**Files:**
- Verify-only across workspace

**Step 1: Run touched-crate test sweeps**

Run:
- `cargo test -p petri-graph`
- `cargo test -p petri-core`
- `cargo test -p petri-server`

**Step 2: Run full completion gate**

Run:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd web && npm run build`

Expected: all PASS.

**Step 3: Run informational benchmark**

Run: `cargo run -p petri-cli --bin stage1_benchmark -- --ticks 200`
Expected: benchmark prints values; no pass/fail threshold assertions required during stabilization.

### Task 8: Final docs sync after implementation

**Files:**
- Modify: `README.md`
- Modify: `docs/reference/creature-controller-reference.md`
- Modify: `petri-roadmap.md` (status checkboxes/notes only)

**Step 1: Move planned semantics to current sections where implementation landed**

- Update wording from "planned" to "implemented" only for completed behaviors.

**Step 2: Verify no stale planned/current contradictions**

Run: `rg -n "planned|not implemented|current" README.md docs/reference/creature-controller-reference.md petri-roadmap.md`
Expected: wording consistent with merged code state.
