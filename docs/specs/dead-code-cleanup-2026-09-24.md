# Dead Code Cleanup (2026-09-24)

**Status**: Complete
**Scope**: Maintenance; no roadmap feature ID, benchmark, or mutation gate.
Branch `worktree-agent-ab9ad86388da4abd8` from `main` at `2a346774`.

## Goal

Delete code that nothing compiles or calls and correct stale comments, with no
change to simulation behavior.

## Non-Goals

- The `shared.{fertility,annealing,initial_density,initial_coverage}` config
  copy, `config/simulation.rs` sync code, `single_type()`, and the frontend
  (another agent owns these concurrently).
- Any behavior, default, RNG draw, trajectory, digest, or stored-report change.
- Refactoring beyond the deletion or comment edit named per item.

## Items

Line numbers are at `2a346774`. "Zero callers" means a repo-wide grep
(excluding `target/`, `node_modules/`, `.git/`) finds only the definition.

1. **Uncompiled food-resource files — delete.**
   `crates/v3-core/src/kernel/food_resource/{growth,depletion,reconfigure}.rs`.
   `kernel/mod.rs:2` declares `pub mod food_resource;` and
   `food_resource/mod.rs` is the single line `pub use crate::kernel::ordinary_food::*;`;
   no `mod growth|depletion|reconfigure` exists anywhere. `growth.rs` imports
   `super::depletion` and `super::resolve_neighbor_static`, which the live
   `mod.rs` does not define, so the files cannot compile as written. The only
   other mentions are git-blob census rows in
   `docs/progress/historical-benchmark-manifest.json` (historical, untouched).
   **Keep `mod.rs`**: `kernel/world.rs:5` imports `FoodGrowthSummary` and
   `FoodResource` through it.
2. **`MutationSkipReason::BudgetExhausted` — delete.** Variant at
   `crates/v3-core/src/mutation/types/mod.rs:34` and its `as_key` arm at `:44`.
   Never constructed; the only other code reference is the validity `matches!`
   arm at `crates/v3-core/tests/viability.rs:1028` (removed). No serialized
   report, v3-server key list, or frontend source contains the string.
   The two "minimum enum" lists in `docs/reference/v3-mutation-spec.md:752`
   and `docs/reference/v3-evolution-observability-spec.md:144` drop the entry.
3. **`dominant_food_type_at` — delete both.** `kernel/world.rs:153-158`
   (delegate, doc calls it a "compatibility helper") and
   `kernel/ordinary_food/mod.rs:67-80`. Zero callers.
4. **Zero-caller accessors — delete.**
   - `ComplexityEffect::is_increasing` (`mutation/types/mod.rs:14-18`). Its
     sibling `is_decreasing` is live (`mutation/engine/mod.rs:174` and others)
     and stays.
   - `FoodResource::food_type_configs` (`kernel/ordinary_food/mod.rs:213-216`),
     which returns `&self.config.types`; unrelated to the shared-copy fields.
     Its return type is the only use of the `FoodTypeConfig` import at
     `ordinary_food/mod.rs:9`, which is removed with it.
   - `DirtyRect::as_view_rect` (`crates/v3-server/src/query/cache.rs:24-32`),
     plus the `ViewRect` import if it becomes unused.
   - **Kept:** `World::set_food` (`world.rs:161`) is not made `#[cfg(test)]`:
     production callers in `kernel/paint.rs:60,67,81` (outside its `cfg(test)`
     module at `:127`) and integration tests under `crates/v3-core/tests/`.
5. **Stale attributes and comments.**
   - Remove `#[cfg_attr(not(test), allow(dead_code))]` from
     `runtime/vm.rs:33` (`execute_vm_node`, called by `runtime/mesh.rs:162`)
     and `runtime/traced_vm.rs:13` (`execute_vm_node_traced`, called by
     `runtime/traced_mesh.rs:130`).
   - `creature/genome/vote.rs:7`: "nothing reads the surface until T19.F04" is
     stale (T19.F04 closed; `runtime/mesh.rs:493` commits from it). Reword to
     state the pass end commits from the surface (T19.F04).
   - `creature/genome/cgp.rs:115`: `CustomOutput` says "12 slots, indices 0-11";
     `CUSTOM_OUTPUT_COUNT` is 24 (`cgp.rs:172`). Fix to "24 slots, indices
     0-23". The adjacent RouterGate (8 = `MAX_GATE_SLOTS`) and slot (16) counts
     are correct.
   - `contracts/inputs.rs:111`: `EnergyConsumedThisTick` is not "consumed by
     Eat actions"; it reads `(start_energy - energy).max(0)` over the mesh run
     (`runtime/mesh.rs:323,382`, resolved in `runtime/inputs.rs:85` as a
     fraction of max energy): energy spent since the tick's mesh run began
     (earlier dispatches and hop ramp charges); a VM read also adds its own
     in-flight step debt (`runtime/vm.rs:391`), while a graph node sees the
     pre-dispatch value (`runtime/cgp/execute.rs:166`). Reword to that.
   - `creature/genome/mod.rs:310`: rename test
     `vm_instruction_all_42_variants_constructible` to
     `vm_instruction_all_39_variants_constructible`; the enum has 39 variants
     and the test already asserts 39. No other reference to the name exists.
   - **Kept:** `runtime/action_decode.rs:28` `Terminate | Decide => NoOp` arm.
     It is required for match exhaustiveness, is documented at `:12`, and is
     covered by `pass_control_sinks_decode_to_noop`; changing it to
     `unreachable!()` would be a behavior change.

## Invariants

- No production code path changes: every deletion is uncompiled or has zero
  callers, and every other edit is a comment, attribute, or test name.
- No pinned test value, digest, trajectory, or stored report moves. If one
  does, stop: the item was live.
- `MutationSkipReason` serde loses one variant; no stored payload contains it.

## Verification

- Repo-wide grep for each deleted identifier (`BudgetExhausted`,
  `dominant_food_type_at`, `is_increasing`, `food_type_configs`,
  `as_view_rect`, `food_resource/{growth,depletion,reconfigure}`) returns
  nothing except this plan, the git-blob census
  `docs/progress/historical-benchmark-manifest.json`, strategy JSON source
  hashes, and the closed T10.F11 spec's dated audit row
  (`docs/specs/roadmap/t10-f11-cross-process-reproducibility-of-seeded-runs.md:239`),
  all historical records left unchanged.
- `make check` exits 0 once, with output in a log file.

## Review

| Round | Codex job | Verdict |
| --- | --- | --- |
| PRD 1 | `task-mufxskpi-0qd6sn` | not-ready: unused `FoodTypeConfig` import (fixed); three advisories applied or rebutted |
| PRD 2 | `task-mufxyqkx-e2h9u0` | ready |
