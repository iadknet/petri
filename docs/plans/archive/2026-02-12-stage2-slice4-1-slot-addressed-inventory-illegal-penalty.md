# Stage 2 Slice 4.1 (Slot-Addressed Generic Inventory + Illegal-Action Penalty) Implementation Plan

**Goal:** Implement slot-addressed, type-agnostic inventory actions and global illegal-action penalties for creature tick actions, with full inspector/API exposure and frame payload unchanged.

## Task 1: Add failing tests for graph/core semantics
- Add `petri-graph` tests for new selector/action nodes and deterministic bin mapping behavior.
- Add `petri-core` tests for slot-addressed pickup/put, illegal reason emission, penalty stacking, move/reproduce attempt-cost semantics, and death-drop ordering.

## Task 2: Implement graph surface and evaluation
- Extend `SensorInputs`, `ActionOutputs`, and `NodeKind` with inventory/touch/per-slot signals.
- Wire evaluator clamps and output mapping.
- Keep founder behavior latent while supporting new outputs.

## Task 3: Implement core simulation behavior
- Add slot storage model and evolvable slot capacity (`1..12`, default `1`, mutation step `±1`).
- Implement selector binning (`[-1,1]`, equal-width bins) for direction and slot selection.
- Implement generic pickup/put semantics and tick integration ordering.
- Add illegal attempt tracking (action + reason), global diagnostics counter, and penalty/attempt-cost charging rules.
- Add config knobs: `illegal_action_energy_penalty` and `energy_per_inventory_attempt`.

## Task 4: Wire snapshot/server transport
- Extend snapshot/state serde with backwards-compatible defaults.
- Add startup/runtime config patching for new knobs in `petri-server`.
- Extend creature detail payload with slot state + illegal-attempt records.

## Task 5: Wire web protocol + inspector
- Update TS protocol for new config and creature-detail fields.
- Add startup/runtime slider controls for the two new config knobs.
- Extend inspector to show slot state, inventory signal values, and illegal-attempt records.

## Task 6: Update roadmap and verify gate
- Update `petri-roadmap.md` Slice 4.1 text to redesigned scope.
- Run required completion gate:
  1. `cargo fmt --all --check`
  2. `cargo test --workspace`
  3. `cargo clippy --workspace --all-targets -- -D warnings`
  4. `cd web && npm run build`
