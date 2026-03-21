# Phase 2: Backends and Mesh Executor

**Parent plan:** `master_plan.md`

> Invoke `rust-skills` for all Rust implementation work.

This phase implements the actual gate routing logic: `resolve_gated_route`, the
mesh executor integration, VM backend WriteRouteGate, and CGP RouterGate sinks.
After this phase, routing works end-to-end with the new gate system.

**Spec reference:** `docs/superpowers/specs/2026-03-20-per-target-gate-routing-design.md`

---

### Task 7: Implement `resolve_gated_route` with TDD

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/routing.rs`

- [ ] **Step 1: Write failing tests**

```rust
use crate::contracts::{NodeId, RouteTarget, MAX_GATE_SLOTS};

#[test]
fn gated_route_empty_targets_returns_none() {
    let gates = RouteGateMap::default();
    assert_eq!(resolve_gated_route(&[], &gates), None);
}

#[test]
fn gated_route_single_target_skips_scoring() {
    let targets = vec![RouteTarget { target_id: NodeId::new(5), slot: 0, gate_bias: -99.0 }];
    let gates = RouteGateMap::default();
    // Single target always wins regardless of bias
    assert_eq!(resolve_gated_route(&targets, &gates), Some((0, NodeId::new(5))));
}

#[test]
fn gated_route_picks_highest_effective_score() {
    let targets = vec![
        RouteTarget { target_id: NodeId::new(1), slot: 0, gate_bias: 0.0 },
        RouteTarget { target_id: NodeId::new(2), slot: 1, gate_bias: 0.0 },
    ];
    let mut gates = RouteGateMap::default();
    gates.scores[1] = 1.0; // slot 1 has higher runtime score
    assert_eq!(resolve_gated_route(&targets, &gates), Some((1, NodeId::new(2))));
}

#[test]
fn gated_route_bias_plus_runtime() {
    let targets = vec![
        RouteTarget { target_id: NodeId::new(1), slot: 0, gate_bias: 2.0 },
        RouteTarget { target_id: NodeId::new(2), slot: 1, gate_bias: -1.0 },
    ];
    let mut gates = RouteGateMap::default();
    gates.scores[1] = 4.0; // effective: -1.0 + 4.0 = 3.0 > 2.0
    assert_eq!(resolve_gated_route(&targets, &gates), Some((1, NodeId::new(2))));
}

#[test]
fn gated_route_first_target_wins_ties() {
    let targets = vec![
        RouteTarget { target_id: NodeId::new(1), slot: 0, gate_bias: 0.0 },
        RouteTarget { target_id: NodeId::new(2), slot: 1, gate_bias: 0.0 },
    ];
    let gates = RouteGateMap::default(); // all zeros — tied
    assert_eq!(resolve_gated_route(&targets, &gates), Some((0, NodeId::new(1))));
}

#[test]
fn gated_route_out_of_range_slot_gets_zero_runtime() {
    let targets = vec![
        RouteTarget { target_id: NodeId::new(1), slot: 0, gate_bias: 0.0 },
        RouteTarget { target_id: NodeId::new(2), slot: 200, gate_bias: 1.0 },
    ];
    let gates = RouteGateMap::default();
    // slot 200 gets runtime 0.0, effective = 1.0 > 0.0
    assert_eq!(resolve_gated_route(&targets, &gates), Some((1, NodeId::new(2))));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd v3 && cargo test -p v3-core runtime::routing`
Expected: FAIL — `resolve_gated_route` not found

- [ ] **Step 3: Implement `resolve_gated_route`**

```rust
/// Resolve which target to route to based on gate scores.
///
/// effective(target) = gate_bias + runtime_gate[target.slot]
/// Winner = argmax. Ties broken by position (first wins via strict `>`).
#[inline]
#[must_use]
pub(crate) fn resolve_gated_route(
    targets: &[RouteTarget],
    gates: &RouteGateMap,
) -> Option<(usize, NodeId)> {
    if targets.is_empty() {
        return None;
    }
    if targets.len() == 1 {
        return Some((0, targets[0].target_id));
    }

    let mut best_idx = 0;
    let mut best_score = f32::NEG_INFINITY;
    for (i, target) in targets.iter().enumerate() {
        let runtime = if (target.slot as usize) < MAX_GATE_SLOTS {
            gates.scores[target.slot as usize]
        } else {
            0.0
        };
        let effective = target.gate_bias + runtime;
        if effective > best_score {
            best_score = effective;
            best_idx = i;
        }
    }
    Some((best_idx, targets[best_idx].target_id))
}
```

- [ ] **Step 4: Run tests**

Run: `cd v3 && cargo test -p v3-core runtime::routing`
Expected: All PASS

- [ ] **Step 5: Commit**

```
git commit -m "feat: implement resolve_gated_route with TDD"
```

---

### Task 8: Update mesh executor

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/mesh.rs`
- Modify: `v3/crates/v3-core/src/runtime/traced_mesh.rs`

- [ ] **Step 1: Replace routing logic in `mesh.rs`**

Replace the transitional routing code (from Task 5) with:

```rust
use crate::runtime::routing::resolve_gated_route;

// Replace the old resolve_route_index block:
match resolve_gated_route(&node.targets, &result.route_gates) {
    Some((_idx, id)) => {
        if find_node_index(&genome.nodes, id).is_none() {
            return MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            };
        }
        upstream_slots = result.output_slots;
        current_node_id = id;
        hops += 1;
    }
    None => {
        return MeshOutput {
            actions: side_outputs.action_queue.into_actions_or_noop(),
            cost_report: report,
            priority_bid: side_outputs.priority_bid,
        };
    }
}
```

Remove the `use` of `resolve_route_index`.

- [ ] **Step 2: Apply same change to `traced_mesh.rs`**

Same routing logic replacement. Update trace capture to pass `result.route_gates`
instead of `result.route`.

- [ ] **Step 3: Delete old `resolve_route_index` and `RouteDecision`**

In `routing.rs`: remove `RouteDecision` enum, `resolve_route_index` function,
and their tests. These are now fully replaced.

- [ ] **Step 4: Run mesh tests**

Run: `cd v3 && cargo test -p v3-core runtime::mesh`
Expected: PASS

- [ ] **Step 5: Run full test suite**

Run: `cd v3 && cargo test --workspace`
Expected: All PASS

- [ ] **Step 6: Commit**

```
git commit -m "feat: mesh executor uses resolve_gated_route"
```

---

### Task 9: VM backend — WriteRouteGate

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/mod.rs` (VmInstruction enum)
- Modify: `v3/crates/v3-core/src/runtime/vm.rs`
- Modify: `v3/crates/v3-core/src/runtime/traced_vm.rs`

- [ ] **Step 1: Replace VmInstruction variant**

In `VmInstruction` enum:
```rust
// Delete:
WriteRouteTarget { src: u8 },

// Add:
/// Write register value as gate score for the given slot.
WriteRouteGate { slot: u8, src: u8 },
```

- [ ] **Step 2: Fix all compilation errors from enum change**

Every `match` on `VmInstruction` will fail. Update:
- `vm.rs`: instruction handler
- `traced_vm.rs`: instruction handler
- `mutation/vm/operators.rs`: instruction generation/mutation (temporary — use slot 0)
- `genome/analysis.rs`: write-class detection
- `genome/mesh_annotations.rs`: annotation logic
- Any test fixtures constructing `WriteRouteTarget`

- [ ] **Step 3: Update VM execution in `vm.rs`**

Remove `let mut route_target: f32 = 0.0;` (already replaced in Task 5).

Handler:
```rust
VmInstruction::WriteRouteGate { slot, src } => {
    let s = *slot as usize;
    if s < MAX_GATE_SLOTS {
        route_gates.scores[s] = sanitize_f32(regs[nr(*src, reg_count)]);
    }
}
```

- [ ] **Step 4: Update opcode cost**

In `opcode_base_cost`:
```rust
VmInstruction::WriteRouteGate { .. } => 0.10,
```

- [ ] **Step 5: Apply same changes to `traced_vm.rs`**

Mirror all handler and state changes. Update trace capture.

- [ ] **Step 6: Write VM routing test**

```rust
#[test]
fn vm_write_route_gate_sets_slot_score() {
    // Build a minimal VM program:
    // LoadConst { dst: 0, const_idx: 0 }  // r0 = 2.5
    // WriteRouteGate { slot: 3, src: 0 }  // gate[3] = 2.5
    // Halt
    // Verify result.route_gates.scores[3] == 2.5
    // Verify other slots remain 0.0
}

#[test]
fn vm_write_route_gate_invalid_slot_is_noop() {
    // WriteRouteGate { slot: 255, src: 0 }
    // Verify all gate scores remain 0.0
}
```

- [ ] **Step 7: Run tests**

Run: `cd v3 && cargo test -p v3-core runtime::vm`
Expected: PASS

- [ ] **Step 8: Commit**

```
git commit -m "feat: VM backend uses WriteRouteGate instruction"
```

---

### Task 10: CGP backend — RouterGate sinks

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/cgp.rs`
- Modify: `v3/crates/v3-core/src/runtime/cgp/effects.rs`
- Modify: `v3/crates/v3-core/src/runtime/cgp/execute.rs`
- Modify: `v3/crates/v3-core/src/runtime/cgp/traced.rs`

- [ ] **Step 1: Replace OutputSinkKind variant**

In `cgp.rs`:
```rust
// Delete:
RouterOutput,

// Add:
RouterGate(u8),
```

- [ ] **Step 2: Update FIXED_SINK_COUNT**

Change the `+ 1` (RouterOutput) to `+ MAX_GATE_SLOTS`:
```rust
pub const FIXED_SINK_COUNT: usize =
    CUSTOM_OUTPUT_COUNT as usize
    + SHARED_MEMORY_SLOTS
    + SHARED_MEMORY_SLOTS
    + MAX_GATE_SLOTS;  // 8 RouterGate sinks
```

Update the compile-time assertion: `assert!(FIXED_SINK_COUNT == 64);`

- [ ] **Step 3: Update `new_with_fixed_outputs()`**

Replace the single `RouterOutput` sink with 8 `RouterGate(n)` sinks:
```rust
// Delete:
sinks.push(OutputSink { kind: OutputSinkKind::RouterOutput, inputs: vec![] });

// Add:
for slot in 0..MAX_GATE_SLOTS as u8 {
    sinks.push(OutputSink {
        kind: OutputSinkKind::RouterGate(slot),
        inputs: vec![],
    });
}
```

- [ ] **Step 4: Update effects pass**

In `apply_cgp_graph_effects`:
- Replace `let mut route_raw_value = 0.0f32;` with `let mut route_gates = RouteGateMap::default();`
- Replace the `RouterOutput` match arm with `RouterGate(slot)` handler
- Return `route_gates` in the NodeResult instead of `RouteDecision::CgpNormalized`

- [ ] **Step 5: Update exhaustion paths in `execute.rs`**

Replace `RouteDecision::CgpNormalized { raw_value: 0.0 }` with
`RouteGateMap::default()` in exhaustion returns.

- [ ] **Step 6: Apply same changes to `traced.rs`**

Mirror effects pass changes. Update trace capture for per-slot sink data.

- [ ] **Step 7: Fix all sink index assertions**

Tests in `cgp.rs` and `cgp_founder.rs` that assert specific sink indices
need updating. RouterGate(0..7) now occupy indices 24-31, shifting WriteSlot
sinks from index 25+ to 32+.

- [ ] **Step 8: Update `MeshWriteClass::Route` pattern**

In `cgp_mesh_annotations.rs`: match `RouterGate(_)` instead of `RouterOutput`.

- [ ] **Step 9: Run CGP tests**

Run: `cd v3 && cargo test -p v3-core creature::genome::cgp`
Expected: PASS

- [ ] **Step 10: Run full test suite**

Run: `cd v3 && cargo test --workspace`
Expected: All PASS

- [ ] **Step 11: Commit**

```
git commit -m "feat: CGP backend uses RouterGate(u8) sinks"
```

---

Phase 2 complete. Routing works end-to-end with the new gate system. The VM
writes per-slot gate scores, CGP produces per-slot weighted sums, and the mesh
executor resolves via argmax. Phase 3 updates mutation operators, traces, and
runs verification.
