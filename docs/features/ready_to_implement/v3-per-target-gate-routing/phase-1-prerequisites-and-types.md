# Phase 1: Prerequisites and Core Types

**Parent plan:** `master_plan.md`

> Invoke `rust-skills` for all Rust implementation work.

This phase creates the shared types, refactors the topology module, and updates
the core data model. After this phase, the code compiles with the new type
system but still uses stub/default routing behavior.

**Spec reference:** `docs/superpowers/specs/2026-03-20-per-target-gate-routing-design.md`

---

### Task 1: Create `contracts/routing.rs`

**Files:**
- Create: `v3/crates/v3-core/src/contracts/routing.rs`
- Modify: `v3/crates/v3-core/src/contracts/mod.rs`

- [ ] **Step 1: Create `routing.rs` with types**

```rust
// v3/crates/v3-core/src/contracts/routing.rs

use super::NodeId;

/// Maximum distinct gate slots per node.
pub const MAX_GATE_SLOTS: usize = 8;

/// A single routing target with stable identity and evolvable bias.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteTarget {
    pub target_id: NodeId,
    /// Stable slot identifier (0..MAX_GATE_SLOTS).
    pub slot: u8,
    /// Genome-level evolvable bias. Clamped to [-4.0, 4.0].
    pub gate_bias: f32,
}
```

- [ ] **Step 2: Add module to `contracts/mod.rs`**

Add `pub mod routing;` and re-export: `pub use routing::{RouteTarget, MAX_GATE_SLOTS};`

- [ ] **Step 3: Add size assertion**

```rust
// Prevent accidental size regression — RouteTarget is stored in Vec per node.
const _: () = assert!(std::mem::size_of::<RouteTarget>() <= 16);
```

- [ ] **Step 4: Write unit test for RouteTarget**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_target_serde_roundtrip() {
        let rt = RouteTarget {
            target_id: NodeId::new(42),
            slot: 3,
            gate_bias: -1.5,
        };
        let json = serde_json::to_string(&rt).unwrap();
        let back: RouteTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, back);
    }
}
```

- [ ] **Step 5: Run test**

Run: `cd v3 && cargo test -p v3-core contracts::routing`
Expected: PASS

- [ ] **Step 6: Commit**

```
git add v3/crates/v3-core/src/contracts/routing.rs v3/crates/v3-core/src/contracts/mod.rs
git commit -m "feat: add RouteTarget and MAX_GATE_SLOTS to contracts"
```

---

### Task 2: Split `mutation/topology/mod.rs`

**Files:**
- Modify: `v3/crates/v3-core/src/mutation/topology/mod.rs` (keep enum + dispatch only)
- Create: `v3/crates/v3-core/src/mutation/topology/structural.rs`
- Create: `v3/crates/v3-core/src/mutation/topology/routing.rs`

This is a pure refactor — no behavior change. Move function implementations to
the new files. The `TopologyOperator` enum and `TopologyMutator::apply` dispatch
stay in `mod.rs`.

- [ ] **Step 1: Create `structural.rs`**

Move these functions from `mod.rs`:
- `apply_add_node`
- `apply_remove_node`
- `apply_change_entry_node`
- `apply_swap_node_backend`
- `apply_rewrite_node_id`
- `apply_copy_node`
- `apply_copy_mesh_backward_slice`
- `apply_copy_mesh_forward_slice`
- `apply_splice_node`
- Helper: `next_node_id`

Add `use` imports for types used by these functions. Make functions `pub(super)`.

- [ ] **Step 2: Create `routing.rs`**

Move these functions from `mod.rs`:
- `apply_retarget_node_target`
- `apply_add_route_target`
- `apply_remove_route_target`
- `apply_swap_route_targets`

Make functions `pub(super)`.

Note: the topology module also has a `tests.rs` file (referenced via `mod tests;`
in `mod.rs`). Tests call functions that move to the new files — update `use`
imports in `tests.rs` if needed (functions are `pub(super)` so tests in sibling
`tests.rs` access them through `super::routing::*` and `super::structural::*`).

- [ ] **Step 3: Update `mod.rs`**

Add `mod structural;` and `mod routing;`. Update `TopologyMutator::apply` to
call `structural::apply_*` and `routing::apply_*` instead of local functions.
Remove moved functions from `mod.rs`. Keep the `TopologyOperator` enum,
`TopologyMutator`, weight/complexity methods, and `random()` in `mod.rs`.

- [ ] **Step 4: Run all topology tests**

Run: `cd v3 && cargo test -p v3-core mutation::topology`
Expected: All existing tests PASS (pure refactor, no behavior change)

- [ ] **Step 5: Run full test suite to confirm no regressions**

Run: `cd v3 && cargo test --workspace`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```
git add v3/crates/v3-core/src/mutation/topology/
git commit -m "refactor: split topology mutations into structural.rs + routing.rs"
```

---

### Task 3: Derive `FIXED_SINK_COUNT` from components

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/cgp.rs`

- [ ] **Step 1: Replace hardcoded constant**

Find the `FIXED_SINK_COUNT` constant and replace with:

```rust
pub const FIXED_SINK_COUNT: usize =
    CUSTOM_OUTPUT_COUNT as usize     // 24 output slots
    + SHARED_MEMORY_SLOTS as usize   // 16 write slots
    + SHARED_MEMORY_SLOTS as usize   // 16 clear slots
    + 1;                             // 1 RouterOutput (will become MAX_GATE_SLOTS in Task 10)
```

Note: `SHARED_MEMORY_SLOTS` is `u8`, so `as usize` casts are required.

Note: keep `+ 1` for now (still RouterOutput). Task 10 changes this to
`+ MAX_GATE_SLOTS` when RouterGate sinks are added.

- [ ] **Step 2: Verify computed value matches original**

Add a compile-time assertion if one doesn't exist:
```rust
const _: () = assert!(FIXED_SINK_COUNT == 57);
```

- [ ] **Step 3: Run tests**

Run: `cd v3 && cargo test -p v3-core creature::genome::cgp`
Expected: PASS

- [ ] **Step 4: Commit**

```
git commit -m "refactor: derive FIXED_SINK_COUNT from component constants"
```

---

### Task 4: Replace `RouteDecision` with `RouteGateMap`

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/routing.rs`

- [ ] **Step 1: Write test for RouteGateMap default**

```rust
#[test]
fn route_gate_map_default_is_all_zeros() {
    let map = RouteGateMap::default();
    assert!(map.scores.iter().all(|&s| s == 0.0));
}
```

- [ ] **Step 2: Add RouteGateMap type**

```rust
use crate::contracts::MAX_GATE_SLOTS;

/// Per-slot gate scores produced by node execution.
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub(crate) struct RouteGateMap {
    pub scores: [f32; MAX_GATE_SLOTS],
}

// Hot-path type size assertions — prevent accidental regressions.
const _: () = assert!(std::mem::size_of::<RouteGateMap>() == 32);


impl Default for RouteGateMap {
    fn default() -> Self {
        Self { scores: [0.0; MAX_GATE_SLOTS] }
    }
}
```

- [ ] **Step 3: Run test**

Run: `cd v3 && cargo test -p v3-core runtime::routing`
Expected: PASS (new test passes, old RouteDecision tests still pass since we haven't deleted it yet)

- [ ] **Step 4: Commit**

```
git commit -m "feat: add RouteGateMap type to runtime/routing.rs"
```

---

### Task 5: Update `NodeResult` in `runtime/types.rs`

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/types.rs`
- Modify: All files that construct or read `NodeResult.route`

This is a large mechanical change. Replace `route: RouteDecision` with
`route_gates: RouteGateMap` in `NodeResult` and update all constructors and
call sites.

- [ ] **Step 1: Change `NodeResult` struct**

Replace `pub route: RouteDecision` with `pub route_gates: RouteGateMap`.

- [ ] **Step 2: Update constructors**

```rust
pub fn halted(output_slots: [f32; OUTPUT_SLOT_COUNT], route_gates: RouteGateMap) -> Self {
    Self { output_slots, route_gates, terminal: false, energy_exhausted: false }
}

pub fn terminal(output_slots: [f32; OUTPUT_SLOT_COUNT], route_gates: RouteGateMap) -> Self {
    Self { output_slots, route_gates, terminal: true, energy_exhausted: false }
}

pub fn exhausted() -> Self {
    Self {
        output_slots: [0.0; OUTPUT_SLOT_COUNT],
        route_gates: RouteGateMap::default(),
        terminal: true,
        energy_exhausted: true,
    }
}
```

Delete `exhausted_with_route()`.

- [ ] **Step 3: Fix all compilation errors**

Update every call site that constructs `NodeResult`:
- `vm.rs`: all `NodeResult::halted(payload, RouteDecision::VmWrap { raw_value: route_target })` → `NodeResult::halted(payload, route_gates)`
- `traced_vm.rs`: same pattern
- `cgp/effects.rs`: `NodeResult::halted(..., RouteDecision::CgpNormalized { ... })` → `NodeResult::halted(..., route_gates)`
- `cgp/execute.rs`: exhaustion paths
- `cgp/traced.rs`: same pattern

At this point, `vm.rs` still has a local `route_target: f32` variable. Wrap it:
```rust
let mut route_gates = RouteGateMap::default();
// ... existing WriteRouteTarget handler writes to route_gates.scores[0] temporarily
```

This is a transitional state. Task 9 will complete the VM changes.

- [ ] **Step 4: Update all call sites that read `result.route`**

- `mesh.rs`: `result.route` → `result.route_gates` (Task 8 will change the resolution logic)
- `traced_mesh.rs`: same
- Test fixtures: update `NodeResult` construction in tests

For now, keep `resolve_route_index` working by constructing a `RouteDecision`
from `scores[0]` in mesh.rs. For VM nodes use `RouteDecision::VmWrap { raw_value: scores[0] }`,
for CGP nodes use `RouteDecision::CgpNormalized { raw_value: scores[0] }`. Match on
`node.backend_def` to determine which variant. This is transitional — Task 7-8
will delete `resolve_route_index` and `RouteDecision` entirely.

- [ ] **Step 5: Run tests**

Run: `cd v3 && cargo test --workspace`
Expected: All tests PASS (behavior preserved through transitional wiring)

- [ ] **Step 6: Commit**

```
git commit -m "refactor: replace RouteDecision with RouteGateMap in NodeResult"
```

---

### Task 6: Change `NodeGenome.targets` to `Vec<RouteTarget>`

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/mod.rs`
- Modify: All files that read/write `node.targets`

- [ ] **Step 1: Change field type**

In `NodeGenome`: `pub targets: Vec<NodeId>` → `pub targets: Vec<RouteTarget>`

- [ ] **Step 2: Fix all compilation errors**

This touches many files. For each, convert bare `NodeId` targets to
`RouteTarget` with sequential slot assignment and `gate_bias: 0.0`:

- `mutation/topology/routing.rs`: operators that push/remove/swap targets
- `mutation/topology/structural.rs`: CopyNode, SpliceNode, slice operators
- `mutation/topology/birth.rs`: `new_topology_birth_node` signature
- `creature/founder.rs`: founder genome construction
- `simulation/seeding.rs`: if it constructs targets directly
- `runtime/mesh.rs`: `node.targets[target_pos]` → `node.targets[target_pos].target_id`
- `runtime/traced_mesh.rs`: same
- Test fixtures across the codebase

For topology mutation operators, this is a temporary conversion — Task 11 will
implement the proper routing-aware logic. For now, use:
```rust
RouteTarget { target_id: node_id, slot: idx as u8, gate_bias: 0.0 }
```

**Important:** Use `gate_bias: 0.0` for all existing/converted targets. The
`-1.0` default only applies to `AddRouteTarget` (Task 11) when creating NEW
speculative branches. Existing targets must keep `0.0` to preserve current
routing behavior (first-target-wins tie-break).

- [ ] **Step 3: Run tests**

Run: `cd v3 && cargo test --workspace`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```
git commit -m "refactor: change NodeGenome.targets from Vec<NodeId> to Vec<RouteTarget>"
```

---

Phase 1 complete. The codebase now compiles with the new type system. Routing
behavior is preserved through transitional wiring (scores[0] maps to the old
scalar behavior). Phase 2 implements the actual gate routing logic.
