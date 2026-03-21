# Phase 3: Mutation, Trace, and Verification

**Parent plan:** `master_plan.md`

> Invoke `rust-skills` for all Rust implementation work.

This phase updates mutation operators to be gate-aware, adds MutateGateBias,
updates trace types, fixes founder genomes, and runs the full verification gate.

**Spec reference:** `docs/superpowers/specs/2026-03-20-per-target-gate-routing-design.md`

---

### Task 11: Update routing mutation operators

**Files:**
- Modify: `v3/crates/v3-core/src/mutation/topology/routing.rs`

- [ ] **Step 1: Write failing tests for AddRouteTarget**

```rust
#[test]
fn add_route_target_assigns_lowest_unused_slot() {
    // Genome with node that has targets at slots [0, 2]
    // AddRouteTarget should assign slot 1 (lowest unused)
    // gate_bias should be -1.0
}

#[test]
fn add_route_target_noop_when_at_max_gate_slots() {
    // Node with MAX_GATE_SLOTS targets
    // AddRouteTarget should return Err(NoApplicableTarget) or skip
}
```

- [ ] **Step 2: Implement `lowest_unused_slot`**

```rust
use crate::contracts::MAX_GATE_SLOTS;

// Compile-time guard: bitmask fits in u8
const _: () = assert!(MAX_GATE_SLOTS <= 8, "lowest_unused_slot uses u8 bitmask");

fn lowest_unused_slot(targets: &[RouteTarget]) -> Option<u8> {
    let used: u8 = targets.iter()
        .filter(|t| (t.slot as usize) < MAX_GATE_SLOTS)
        .fold(0u8, |mask, t| mask | (1 << t.slot));
    (0..MAX_GATE_SLOTS as u8).find(|&s| used & (1 << s) == 0)
}
```

- [ ] **Step 3: Update `apply_add_route_target`**

```rust
fn apply_add_route_target(...) -> Result<TargetReachability, MutationSkipReason> {
    // ... existing node selection ...
    let slot = lowest_unused_slot(&genome.nodes[node_idx].targets)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_id = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    genome.nodes[node_idx].targets.push(RouteTarget {
        target_id,
        slot,
        gate_bias: -1.0,
    });
    Ok(reachability)
}
```

- [ ] **Step 4: Update `apply_remove_route_target`**

No change to logic — `targets.remove(target_slot)` removes the entire
`RouteTarget`. Freed slot is naturally available for reuse.

- [ ] **Step 5: Update `apply_retarget_node_target`**

Change only `target_id`, preserve `slot` and `gate_bias`:
```rust
genome.nodes[node_idx].targets[target_slot].target_id = new_target;
```

- [ ] **Step 6: Update `apply_swap_route_targets`**

Swap only `target_id` fields, preserving slot and bias at each position:
```rust
let a_id = genome.nodes[node_idx].targets[a].target_id;
let b_id = genome.nodes[node_idx].targets[b].target_id;
genome.nodes[node_idx].targets[a].target_id = b_id;
genome.nodes[node_idx].targets[b].target_id = a_id;
```

- [ ] **Step 7: Run tests**

Run: `cd v3 && cargo test -p v3-core mutation::topology::routing`
Expected: PASS

- [ ] **Step 8: Commit**

```
git commit -m "feat: routing mutation operators are gate-aware"
```

---

### Task 12: Add MutateGateBias operator

**Files:**
- Modify: `v3/crates/v3-core/src/mutation/topology/mod.rs`
- Modify: `v3/crates/v3-core/src/mutation/topology/routing.rs`
- Modify: `v3/crates/v3-core/src/mutation/types/mod.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn mutate_gate_bias_nudges_bias_within_clamp() {
    // Create genome with a target at gate_bias = 0.0
    // Apply MutateGateBias with a seeded RNG
    // Verify gate_bias changed and is within [-4.0, 4.0]
}

#[test]
fn mutate_gate_bias_noop_on_empty_targets() {
    // Node with no targets
    // MutateGateBias should return Err(NoApplicableTarget)
}
```

- [ ] **Step 2: Add `MutateGateBias` to `TopologyOperator`**

```rust
pub enum TopologyOperator {
    // ... existing variants ...
    MutateGateBias,
}
```

Update `ALL` array (14 variants), compile-time assertion, `weight()` (return 4),
`complexity_effect()` (return Neutral), and `random()`.

- [ ] **Step 3: Implement `apply_mutate_gate_bias`**

```rust
pub(super) fn apply_mutate_gate_bias(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<usize> = genome.nodes.iter().enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
    let target = &mut genome.nodes[node_idx].targets[target_slot];
    target.gate_bias = (target.gate_bias + rng.gen_range(-0.5f32..0.5)).clamp(-4.0, 4.0);
    Ok(reachability)
}
```

- [ ] **Step 4: Add dispatch in `TopologyMutator::apply`**

```rust
TopologyOperator::MutateGateBias => {
    routing::apply_mutate_gate_bias(genome, reachable_nodes, bias, rng)
}
```

- [ ] **Step 5: Add `TopologyMutateGateBias` to `MutationOperator` enum**

In `mutation/types/mod.rs`, add the variant. Update `complexity_effect()`,
display/debug formatting, and any exhaustiveness checks.

- [ ] **Step 6: Run tests**

Run: `cd v3 && cargo test -p v3-core mutation`
Expected: PASS (including cross-consistency test)

- [ ] **Step 7: Commit**

```
git commit -m "feat: add MutateGateBias topology operator"
```

---

### Task 13: Update VM instruction mutation

**Files:**
- Modify: `v3/crates/v3-core/src/mutation/vm/operators.rs`

- [ ] **Step 1: Update instruction generation**

Where `WriteRouteTarget { src }` was generated, emit:
```rust
VmInstruction::WriteRouteGate {
    slot: rng.gen_range(0..MAX_GATE_SLOTS as u8),
    src: rng.gen_range(0..register_count),
}
```

- [ ] **Step 2: Update field mutation for WriteRouteGate**

In `mutate_instruction_raw_fields`, add a handler for `WriteRouteGate`:
```rust
VmInstruction::WriteRouteGate { slot, src } => {
    if rng.gen_bool(0.5) {
        *slot = rng.gen_range(0..MAX_GATE_SLOTS as u8);
    } else {
        *src = rng.gen_range(0..u8::MAX);
    }
}
```

- [ ] **Step 3: Run VM mutation tests**

Run: `cd v3 && cargo test -p v3-core mutation::vm`
Expected: PASS

- [ ] **Step 4: Commit**

```
git commit -m "feat: VM mutation generates WriteRouteGate instructions"
```

---

### Task 14: Update birth.rs and structural operators

**Files:**
- Modify: `v3/crates/v3-core/src/mutation/topology/birth.rs`
- Modify: `v3/crates/v3-core/src/mutation/topology/structural.rs`

- [ ] **Step 1: Update `new_topology_birth_node` signature**

Change `targets: Vec<NodeId>` → `targets: Vec<RouteTarget>`. Update all callers.

- [ ] **Step 2: Update CopyNode backlink**

When CopyNode adds a backlink to the source node:
```rust
if let Some(slot) = lowest_unused_slot(&genome.nodes[source_idx].targets) {
    genome.nodes[source_idx].targets.push(RouteTarget {
        target_id: new_id,
        slot,
        gate_bias: 0.0,
    });
}
// Skip backlink if no free slots
```

- [ ] **Step 3: Update SpliceNode**

New node gets: `vec![RouteTarget { target_id: downstream, slot: 0, gate_bias: 0.0 }]`

Upstream target has only `target_id` changed (slot + bias preserved):
```rust
genome.nodes[a_idx].targets[target_slot].target_id = c_id;
```

- [ ] **Step 4: Update slice operators**

In `clone_and_remap_slice`:
- Iterate `target.target_id` instead of bare `NodeId` for remapping
- Preserve `slot` and `gate_bias` on cloned targets
- Backlinks: use `lowest_unused_slot()`, `gate_bias: 0.0`, skip if full

- [ ] **Step 5: Run tests**

Run: `cd v3 && cargo test -p v3-core mutation::topology`
Expected: PASS

- [ ] **Step 6: Commit**

```
git commit -m "feat: structural operators construct proper RouteTarget values"
```

---

### Task 15: Update genome analysis + mesh annotations

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/analysis.rs`
- Modify: `v3/crates/v3-core/src/creature/genome/mesh_annotations.rs`
- Modify: `v3/crates/v3-core/src/creature/genome/cgp_mesh_annotations.rs`

- [ ] **Step 1: Update write-class detection**

Replace `WriteRouteTarget` matches with `WriteRouteGate` in analysis.rs.
Replace `RouterOutput` matches with `RouterGate(_)` in cgp_mesh_annotations.rs.

- [ ] **Step 2: Ensure `MeshWriteClass::Route` covers both**

Both `WriteRouteGate { .. }` and `RouterGate(_)` map to `MeshWriteClass::Route`.

- [ ] **Step 3: Run tests**

Run: `cd v3 && cargo test -p v3-core creature::genome`
Expected: PASS

- [ ] **Step 4: Commit**

```
git commit -m "feat: genome analysis recognizes WriteRouteGate and RouterGate"
```

---

### Task 16: Update founder genome + seeding

**Files:**
- Modify: `v3/crates/v3-core/src/creature/founder.rs`
- Modify: `v3/crates/v3-core/src/simulation/seeding.rs` (if needed)

- [ ] **Step 1: Update founder targets**

Node 0 (Graph):
```rust
targets: vec![RouteTarget { target_id: NodeId::new(1), slot: 0, gate_bias: 0.0 }]
```

Node 1 (VM): `targets: vec![]` (unchanged)

- [ ] **Step 2: Update founder CGP sink**

Replace `RouterOutput` with `RouterGate(0)` in the founder graph construction.
Update any sink index references.

- [ ] **Step 3: Run founder tests**

Run: `cd v3 && cargo test -p v3-core creature::founder`
Expected: PASS

- [ ] **Step 4: Commit**

```
git commit -m "feat: founder genomes use RouteTarget and RouterGate(0)"
```

---

### Task 17: Update trace domain types

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/trace/domain.rs`

- [ ] **Step 1: Replace trace types**

Delete `TraceRouteKind`, old `TraceRouteDecision`, and `from_internal()`.

Add:
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TraceRouteDecision {
    pub gate_scores: Vec<TraceGateScore>,
    pub selected_target_idx: usize,
    pub selected_target_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TraceGateScore {
    pub slot: u8,
    pub target_id: NodeId,
    pub gate_bias: f32,
    pub runtime_score: f32,
    pub effective_score: f32,
}
```

- [ ] **Step 2: Drop `VmTrace.final_route_value`**

Remove the field from `VmTrace`. Routing trace data lives at mesh level only.

- [ ] **Step 3: Update `MeshHopTrace`**

- `route: TraceRouteDecision` stays (type changed)
- `resolved_target_index: usize` removed (folded into `TraceRouteDecision.selected_target_idx`)

- [ ] **Step 4: Fix compilation errors in trace consumers**

Update `traced_mesh.rs`, `traced_vm.rs`, and any test that constructs trace
types.

- [ ] **Step 5: Commit**

```
git commit -m "feat: trace types use gate score map instead of scalar route"
```

---

### Task 18: Update traced mesh + traced VM

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/traced_mesh.rs`
- Modify: `v3/crates/v3-core/src/runtime/traced_vm.rs`

- [ ] **Step 1: Update traced mesh trace construction**

After `resolve_gated_route`, construct `TraceRouteDecision`:
```rust
let gate_scores: Vec<TraceGateScore> = node.targets.iter().map(|t| {
    let runtime = if (t.slot as usize) < MAX_GATE_SLOTS {
        result.route_gates.scores[t.slot as usize]
    } else {
        0.0
    };
    TraceGateScore {
        slot: t.slot,
        target_id: t.target_id,
        gate_bias: t.gate_bias,
        runtime_score: runtime,
        effective_score: t.gate_bias + runtime,
    }
}).collect();

let trace_route = TraceRouteDecision {
    gate_scores,
    selected_target_idx: winning_idx, // from resolve_gated_route return
    selected_target_id: target_id,
};
```

`resolve_gated_route` returns `Option<(usize, NodeId)>` — both the winning
index and target ID. The traced mesh uses the index directly for
`selected_target_idx` and constructs `TraceGateScore` entries by iterating
targets + gate map. No duplicated argmax logic.

- [ ] **Step 2: Update traced VM**

Remove capture of `final_route_value`. The `route_gates` are captured at mesh
level, not VM level.

- [ ] **Step 3: Run traced execution tests**

Run: `cd v3 && cargo test -p v3-core runtime::traced`
Run: `cd v3 && cargo test -p v3-core runtime::trace`
Expected: PASS

- [ ] **Step 4: Commit**

```
git commit -m "feat: traced execution captures gate score details"
```

---

### Task 18b: Update v3-server transport types

**Files:**
- Modify: `v3/crates/v3-server/src/transport/sample_protocol.rs`
- Modify: `v3/crates/v3-server/src/transport/sample_assembler.rs`

- [ ] **Step 1: Update `RouteDecisionPayload` for new trace shape**

Replace scalar `final_route_value` and `resolved_target_index` with gate score
array and selected target info matching the new `TraceRouteDecision`.

- [ ] **Step 2: Update assembler to construct new payload**

The assembler reads `MeshHopTrace.route` (now `TraceRouteDecision` with gate
scores). Map to the updated protocol payload.

- [ ] **Step 3: Run server tests**

Run: `cd v3 && cargo test -p v3-server`
Expected: PASS

- [ ] **Step 4: Commit**

```
git commit -m "feat: v3-server transport uses gate score trace format"
```

---

### Task 18c: Update e2e tests

**Files:**
- Modify: `v3/crates/v3-core/tests/creature_workflow_e2e/routing_and_state.rs`
- Modify: `v3/crates/v3-core/tests/creature_workflow_e2e/flow_and_vm.rs`
- Modify: `v3/crates/v3-core/tests/vm_all_opcodes_e2e.rs`

- [ ] **Step 1: Update e2e tests**

Replace references to `RouterOutput`, `TraceRouteKind`, `WriteRouteTarget`,
`final_route_value`, and `resolved_target_index` with their new equivalents.

- [ ] **Step 2: Run e2e tests**

Run: `cd v3 && cargo test -p v3-core --test creature_workflow_e2e --test vm_all_opcodes_e2e`
Expected: PASS

- [ ] **Step 3: Commit**

```
git commit -m "fix: update e2e tests for gate routing types"
```

---

### Task 19-24: Verification Gate

- [ ] **Task 19:** Run `cd v3 && cargo fmt --all -- --check`
- [ ] **Task 20:** Run `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
- [ ] **Task 21:** Run `cd v3 && cargo test --workspace`
- [ ] **Task 22:** Run `cd v3 && cargo test -p v3-core --test viability` (merge gate)
- [ ] **Task 23:** Run `scripts/check-plan-harness.sh --mode strict`
- [ ] **Task 24:** Run `scripts/check-doc-harness.sh --mode strict`

All must pass before proceeding to review gates.

---

Phase 3 complete. All mutation operators are gate-aware, traces capture rich
routing data, founders use the new types, and the full verification gate passes.
