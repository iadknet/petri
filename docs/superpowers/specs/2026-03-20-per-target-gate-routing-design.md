# Per-Target Gate Routing Design

## Problem

Creatures evolve complex genome topologies (many reachable nodes) but only ever
execute 2-3 of them. The current single-scalar routing primitive creates an
evolutionary trap: the path from "simple working circuit" to "more complex
working circuit" requires too many coordinated mutations.

### Root cause

Each node produces a single `f32` routing scalar that gets binned into a target
index. This is fragile in three ways:

1. **Positional instability.** When topology mutations add/remove targets, the
   scalar silently re-maps to a different branch, killing working sub-circuits
   (the "15→19" problem).
2. **High mutation coordination cost.** Activating a new branch requires
   simultaneously: a new target (topology mutation), a routing scalar that
   sometimes picks it (continuous value change), useful computation in the new
   node, and wiring that feeds back usefully. No single mutation can create a
   viable new branch.
3. **No conditional branching.** A single scalar cannot express "if X, go to
   target A; if Y, go to target B." All routing is unconditional.

### Evidence

Trace analysis of a representative creature (ID `1060856935364` at tick 11058)
showed:

- Mesh path was always `15 → 17` across 5 consecutive ticks
- The graph node 15's router was `−0.0045 * neighbor_occupied[5]`, never
  producing a positive value — always routing to target index 0
- The alternate branch `15 → 19 → 18` existed in the genome but was
  behaviorally inert
- The creature's entire decision-making reduced to one upstream slot value
  decoded as a direction

The architecture has ample compute capacity (1024 mesh hops, 10000 VM steps,
persistent shared memory). The bottleneck is evolvability, not expressivity.

## Goals

1. **Remove structural barriers to evolving complex cognition.** Make the
   evolutionary step from 2→3+ useful nodes achievable through incremental
   single mutations.
2. **Enable conditional branching.** Allow nodes to route to different targets
   based on runtime computation (input-dependent path selection).
3. **Preserve stability of existing routes.** Adding a new target must not
   break existing routing behavior.
4. **Architect for future sub-circuit modularity.** The design should extend
   cleanly toward call/return (depth-1 first, bounded stack later) without
   requiring a second routing redesign.

### Non-goals

- Parallel activation / fan-out (too much complexity, too many pitfalls)
- Backward compatibility with old serialized genomes or traces
- Routing inertia / hysteresis (defer until flickering is observed in practice)

## Design: Per-Target Gate Routing (A+)

Replace the single routing scalar with **per-target gate scores**. Each target
is independently addressed via a stable slot identifier. The mesh executor picks
the target with the highest effective score.

### Domain boundaries

This feature touches genome, runtime, mutation, and trace domains. To maintain
clean separation of concerns:

| Type / Constant | Home | Rationale |
|-----------------|------|-----------|
| `RouteTarget` | `contracts/routing.rs` | Shared across genome, runtime, mutation, and trace — same pattern as `NodeId` and `InputReference` |
| `MAX_GATE_SLOTS` | `contracts/routing.rs` | Constrains genome (target cap), runtime (gate map size), mutation (slot range), and trace (score count) |
| `RouteGateMap` | `runtime/routing.rs` | Runtime-only type produced during execution |
| `resolve_gated_route` | `runtime/routing.rs` | Routing resolution logic, depends only on contracts types |
| `TraceGateScore` | `runtime/trace/domain.rs` | Trace-domain type, depends only on contracts |
| Routing mutation impls | `mutation/topology/routing.rs` | New file, split from topology/mod.rs |

### Prerequisite refactors

These existing-code changes are required to support clean boundaries:

1. **New `contracts/routing.rs`** — new file for `RouteTarget` and
   `MAX_GATE_SLOTS`, following the existing file-per-concern pattern
   (`ids.rs`, `inputs.rs`, `actions.rs`).

2. **Split `mutation/topology/mod.rs`** into:
   - `topology/mod.rs` — `TopologyOperator` enum, dispatch, shared helpers
   - `topology/structural.rs` — AddNode, RemoveNode, CopyNode, SpliceNode,
     CopyMeshBackwardSlice, CopyMeshForwardSlice, ChangeEntryNode,
     SwapNodeBackend, RewriteNodeId
   - `topology/routing.rs` — AddRouteTarget, RemoveRouteTarget,
     RetargetNodeTarget, SwapRouteTargets, MutateGateBias,
     `lowest_unused_slot`
   - `topology/birth.rs` — unchanged

3. **Derive `FIXED_SINK_COUNT` from components** in `cgp.rs`:
   ```rust
   pub const FIXED_SINK_COUNT: usize =
       CUSTOM_OUTPUT_COUNT as usize  // 24 output slots
       + SHARED_MEMORY_SLOTS         // 16 write slots
       + SHARED_MEMORY_SLOTS         // 16 clear slots
       + MAX_GATE_SLOTS;             // 8 router gate slots
   ```

4. **Drop `VmTrace.final_route_value`** — routing trace data captured at mesh
   level only, via `TraceRouteDecision`. Eliminates duplication between VM
   trace and mesh trace.

### Data model

#### Contracts (shared types)

```rust
// contracts/routing.rs

/// Maximum distinct gate slots per node.
/// Constrains genome (target cap), runtime (gate map size),
/// mutation (slot range), and trace (score count).
pub const MAX_GATE_SLOTS: usize = 8;

/// A single routing target with stable identity and evolvable bias.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteTarget {
    pub target_id: NodeId,
    /// Stable slot identifier (0..MAX_GATE_SLOTS). Survives topology
    /// mutations. VM/CGP writes gate scores keyed by this slot, not by
    /// position in the list.
    pub slot: u8,
    /// Genome-level evolvable bias added to the runtime gate score.
    /// Clamped to [-4.0, 4.0].
    pub gate_bias: f32,
}
```

#### Genome

```rust
// creature/genome/mod.rs

pub struct NodeGenome {
    pub node_id: NodeId,
    pub input_refs: Vec<InputReference>,
    pub backend_def: BackendDef,
    pub targets: Vec<RouteTarget>,  // was: Vec<NodeId>
}
```

#### Runtime

```rust
// runtime/routing.rs

/// Per-slot gate scores produced by node execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RouteGateMap {
    pub scores: [f32; MAX_GATE_SLOTS],
}

impl Default for RouteGateMap {
    fn default() -> Self {
        Self { scores: [0.0; MAX_GATE_SLOTS] }
    }
}
```

```rust
// runtime/types.rs

pub(crate) struct NodeResult {
    pub output_slots: [f32; OUTPUT_SLOT_COUNT],
    pub route_gates: RouteGateMap,  // was: route: RouteDecision
    pub terminal: bool,
    pub energy_exhausted: bool,
}
```

#### Deletions

- `RouteDecision` enum — deleted entirely
- `resolve_route_index()` function — deleted entirely
- `TraceRouteKind` enum — deleted entirely
- `NodeResult::exhausted_with_route()` — collapsed into `exhausted()`

#### NodeResult constructor signatures

```rust
impl NodeResult {
    pub fn halted(output_slots: [f32; OUTPUT_SLOT_COUNT], route_gates: RouteGateMap) -> Self;
    pub fn terminal(output_slots: [f32; OUTPUT_SLOT_COUNT], route_gates: RouteGateMap) -> Self;
    pub fn exhausted() -> Self; // returns RouteGateMap::default()
}
```

### Invariants

Per node, enforced at mutation time (not runtime-checked per tick):

1. `targets.len() <= MAX_GATE_SLOTS`
2. All `slot` values in `0..MAX_GATE_SLOTS`
3. `slot` values unique within a node's `targets`
4. `gate_bias` clamped to `[-4.0, 4.0]` (8 mutation steps min-to-max at ±0.5
   deltas; wide enough for stable preference, narrow enough that runtime gate
   scores can override)
5. Runtime gate writes sanitized via `sanitize_f32()`

### Routing resolution

The effective score for each target is:

```
effective(target) = target.gate_bias + route_gates.scores[target.slot]
```

The mesh executor picks the target with the highest effective score. Ties are
broken by position (first target wins via strict `>`). This gives the first
target implicit "default" priority.

```rust
// runtime/routing.rs

#[inline]
fn resolve_gated_route(
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

**Key properties:**

- Single-target fast path skips scoring entirely.
- All-zero gates + all-zero biases → first target wins (identical to current
  behavior for simple creatures).
- No heap allocation. Gate map is 32 bytes on stack.
- `sanitize_f32` on runtime gate writes prevents NaN/Inf from corrupting
  argmax.
- Out-of-range slots (>= MAX_GATE_SLOTS) degrade gracefully to runtime
  score 0.0.

### VM backend changes

**Instruction:**

```rust
// Deleted:
WriteRouteTarget { src: u8 },

// Added:
/// Write register value as gate score for the given slot.
/// Invalid slots (>= MAX_GATE_SLOTS) are silently ignored.
WriteRouteGate { slot: u8, src: u8 },
```

**Execution state** (`vm.rs`):

```rust
// Deleted:
let mut route_target: f32 = 0.0;

// Added:
let mut route_gates = RouteGateMap::default();
```

**Instruction handler:**

```rust
VmInstruction::WriteRouteGate { slot, src } => {
    let s = *slot as usize;
    if s < MAX_GATE_SLOTS {
        route_gates.scores[s] = sanitize_f32(regs[nr(*src, reg_count)]);
    }
}
```

**Opcode cost:** 0.10 (same as old `WriteRouteTarget`).

All return paths emit `route_gates` instead of
`RouteDecision::VmWrap { raw_value: route_target }`.

Both `execute_vm_node` and `execute_vm_node_traced` must be updated in lockstep.

**What this enables:** A VM program can conditionally route based on inputs:

```
ReadInput { dst: 0, ref_idx: energy_current }
CmpGt { dst: 1, a: 0, b: 2 }          // r1 = energy > threshold?
WriteRouteGate { slot: 0, src: 1 }      // gate for "safe" target
Not { dst: 3, src: 1 }
WriteRouteGate { slot: 1, src: 3 }      // gate for "explore" target
```

### CGP backend changes

**OutputSinkKind:**

```rust
// Deleted:
RouterOutput,

// Added:
/// Gate score for routing slot n. Fixed catalog: 8 sinks (0..7).
RouterGate(u8),
```

8 fixed `RouterGate(0)` through `RouterGate(7)` sinks replace the single
`RouterOutput` in the sink catalog constructed by `new_with_fixed_outputs()`.
`FIXED_SINK_COUNT` is derived from components (see prerequisite refactor #3).
Mutations evolve edges into these fixed sinks — they do not add or remove sinks.

**Sink catalog ordering change:** The old `RouterOutput` sat at index
`CUSTOM_OUTPUT_COUNT` (24). The new `RouterGate(0..7)` sinks occupy indices
24-31, shifting `WriteSlot(0)` from index 25 to index 32. All tests asserting
specific sink indices (in `cgp.rs` and `cgp_founder.rs`) must be updated.

**Graph edge mutation weighting:** With 8 routing sinks instead of 1, random
sink selection is 8x more likely to target routing. This is intentional —
evolution should have more surface area to discover routing-relevant wiring.
Graph mutations treat `RouterGate(n)` sinks identically to other fixed sinks.

**Effects pass** (`effects.rs`):

```rust
// Deleted:
OutputSinkKind::RouterOutput => {
    applied_value = sanitize_f32(wsum);
    route_raw_value = applied_value;
    applied = true;
}

// Added:
OutputSinkKind::RouterGate(slot) => {
    let s = slot as usize;
    if s < MAX_GATE_SLOTS {
        let val = sanitize_f32(wsum);
        route_gates.scores[s] = val;
        applied_value = val;
        applied = true;
    }
}
```

The function's local state changes from `let mut route_raw_value = 0.0f32;` to
`let mut route_gates = RouteGateMap::default();`.

`MeshWriteClass::Route` in `cgp_mesh_annotations.rs` must recognize
`RouterGate(_)` instead of `RouterOutput`.

### Mesh executor changes

The routing block in `mesh.rs` (lines 136-159) and `traced_mesh.rs` changes
from index-based resolution to gated resolution:

```rust
match resolve_gated_route(&node.targets, &result.route_gates) {
    Some(id) => {
        if find_node_index(&genome.nodes, id).is_none() {
            return MeshOutput { /* soft default: NoOp */ };
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

### Mutation operator changes

All routing mutation implementations live in the new
`mutation/topology/routing.rs` file.

#### Modified operators

| Operator | Change |
|----------|--------|
| `AddRouteTarget` | Appends `RouteTarget { target_id, slot: lowest_unused_slot(), gate_bias: -1.0 }`. No-op if `targets.len() >= MAX_GATE_SLOTS` or all slots used. |
| `RemoveRouteTarget` | Removes target. Freed slot available for reuse. Other targets' slots unchanged. |
| `RetargetNodeTarget` | Changes `target_id` only. Preserves `slot` and `gate_bias`. |
| `SwapRouteTargets` | Swaps `target_id` values between two targets within the same node, preserving each position's `slot` and `gate_bias`. This is a semantic change from the current `targets.swap(a, b)` (which swaps entire elements). The new behavior explicitly swaps only `.target_id` fields so that slot identity and bias are positionally stable. |

#### New operator

```rust
MutateGateBias,
```

- Pick a random node (reachability-biased)
- Pick a random target from that node's `targets`
- Apply: `target.gate_bias += rng.gen_range(-0.5..0.5)`
- Clamp to `[-4.0, 4.0]`
- Weight: **4** (refinement tier)
- Complexity effect: **Neutral**
- No-op if node has no targets

`TopologyOperator::ALL` updates from 13 → 14 variants. The compile-time
assertion `Self::ALL.len() == 13` updates to `14`.

#### RouteTarget construction policy for structural operators

Operators that push to `targets` must construct valid `RouteTarget` values.
Policy for each:

| Operator | How targets are constructed |
|----------|---------------------------|
| `CopyNode` | Cloned node inherits source's `targets` (including slots and biases). Backlink added to source node uses `lowest_unused_slot()` with `gate_bias: 0.0` (deliberate structural link, immediately competitive). Backlink is skipped if source node has no free slots (preserves invariant #1). |
| `SpliceNode` | New spliced node gets `vec![RouteTarget { target_id: downstream, slot: 0, gate_bias: 0.0 }]`. Upstream node's existing RouteTarget pointing to downstream is preserved (slot + bias intact), only `target_id` changes to the new node. |
| `CopyMeshBackwardSlice` / `CopyMeshForwardSlice` | Cloned subgraph targets preserve slots and biases. `target_id` values are remapped via the `id_map`. Backlinks to the original mesh use `lowest_unused_slot()` with `gate_bias: 0.0`. Backlink is skipped if source node has no free slots. |
| `AddNode` | New node created via `birth::new_topology_birth_node` with empty targets. |

`birth::new_topology_birth_node` signature changes from `targets: Vec<NodeId>`
to `targets: Vec<RouteTarget>`.

#### MutationOperator enum

`mutation/types/mod.rs` gains a `TopologyMutateGateBias` variant in the
`MutationOperator` enum. Complexity effect: Neutral. The cross-consistency
test (`complexity_effect_cross_consistency_with_domain_operators`) must cover
the new variant.

#### VM instruction mutation

In `mutation/vm/operators.rs`:

- Instruction generation: emit
  `WriteRouteGate { slot: rng.gen_range(0..MAX_GATE_SLOTS as u8), src }`
  instead of `WriteRouteTarget { src }`
- Field mutation: independently mutate `slot` (random in `0..MAX_GATE_SLOTS`)
  or `src` (random register)

#### Analysis and annotation updates

- `genome/analysis.rs`: write-class detection recognizes `WriteRouteGate`
- `genome/mesh_annotations.rs` and `genome/cgp_mesh_annotations.rs`:
  `MeshWriteClass::Route` matches `WriteRouteGate` and `RouterGate(_)`

### Slot assignment policy

New targets are assigned the lowest unused slot in `0..MAX_GATE_SLOTS`:

```rust
fn lowest_unused_slot(targets: &[RouteTarget]) -> Option<u8> {
    let used: u8 = targets.iter().fold(0u8, |mask, t| mask | (1 << t.slot));
    (0..MAX_GATE_SLOTS as u8).find(|&s| used & (1 << s) == 0)
}
```

Returns `None` when all 8 slots are in use → `AddRouteTarget` becomes a no-op.

**Reuse semantics:** Freed slots are immediately reusable. Old VM
`WriteRouteGate` instructions referencing a freed slot become no-ops until a new
target is assigned that slot. The new target starts with `gate_bias = -1.0`, so
even if an old instruction writes a positive runtime score, the target must
overcome the negative bias to win. This provides a natural buffer against
accidental "wake-up" of old routing logic.

### Founder genome changes

Founders are 2-node meshes. Changes are minimal:

- Node 0 (Graph):
  `targets: vec![RouteTarget { target_id: NodeId(1), slot: 0, gate_bias: 0.0 }]`
- Node 1 (VM): `targets: vec![]` (unchanged)

Single-target fast path means gate scoring is never invoked for founders.
Founders use `gate_bias: 0.0` (not -1.0) because they are established routes,
not dormant branches. The -1.0 default applies only to `AddRouteTarget`
mutations that create new speculative branches.

The Graph backend's `RouterOutput` sink becomes `RouterGate(0)`.

### Trace and observability changes

#### Deleted types

- `TraceRouteKind` — no longer needed (routing is backend-agnostic)
- `TraceRouteDecision` (old: `{ kind, raw_value }`) — replaced
- `VmTrace.final_route_value` — routing trace captured at mesh level only

#### New trace types

```rust
// runtime/trace/domain.rs

#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceRouteDecision {
    /// Gate scores for each target, keyed by slot.
    pub gate_scores: Vec<TraceGateScore>,
    /// Index of the winning target in the node's targets list.
    pub selected_target_idx: usize,
    /// NodeId of the winning target.
    pub selected_target_id: NodeId,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceGateScore {
    pub slot: u8,
    pub target_id: NodeId,
    pub gate_bias: f32,
    pub runtime_score: f32,
    /// gate_bias + runtime_score. Redundant but saves trace consumers
    /// from recomputing — critical for debugging routing behavior.
    pub effective_score: f32,
}
```

`RouteGateMap` itself does not need `Serialize` — gate scores are only
exposed through `TraceGateScore` (constructed at trace-capture time from the
gate map + target list).

#### Trace integration points

- `MeshHopTrace.route` → new `TraceRouteDecision`
- `MeshHopTrace.resolved_target_index` → folded into
  `TraceRouteDecision.selected_target_idx`
- `traced_vm.rs`: captures `route_gates` at all exit paths
- `traced_mesh.rs`: constructs `TraceRouteDecision` from gate map + target
  list
- CGP traced execution: captures per-slot weighted sums from `RouterGate(n)`
  sinks

### Evolutionary dynamics

#### Why new targets start at gate_bias = -1.0

If new targets started at 0.0, a single `MutateGateBias` could unconditionally
flip routing to an untested branch. Starting at -1.0 creates a safety margin:

- The new target is suppressed by default (−1.0 vs established 0.0)
- With ±0.5 deltas, it takes 2+ bias mutations to reach 0.0 (competitive)
- Runtime gate scores can override the bias conditionally (e.g., VM writes +1.5
  → effective = −1.0 + 1.5 = +0.5, beats default at 0.0 only when the VM
  computation says so)
- This is the intended evolutionary gradient: dormant → conditionally active →
  dominant

#### The path from 2→3 useful nodes

1. `AddRouteTarget` adds target →
   `{ node_X, slot: 2, gate_bias: -1.0 }` — **dormant**
2. Over generations, `MutateGateBias` raises bias:
   −1.0 → −0.5 → −0.2 — **still dormant** but margin shrinks
3. VM mutation adds `WriteRouteGate { slot: 2, src: r0 }` — if r0 computes a
   value > 0.7 based on inputs, the branch activates **conditionally**
4. Selection: if conditional activation is useful, creatures with this circuit
   survive and reproduce
5. Further `MutateGateBias` nudges strengthen or weaken the branch

Each step is a single mutation. No step requires coordination with other
mutations. Activation is gradual and conditional.

### Performance

- `RouteGateMap` is 32 bytes on stack (vs 4 bytes for old f32 scalar)
- Argmax over ≤8 targets is trivial compared to 10,000 VM steps per node
- No heap allocations added to the hot path
- Single-target nodes skip scoring entirely

### Future extension: depth-1 call/return

The design accommodates a future "C-minus" extension without requiring a second
routing redesign:

- The routing decision gains a `mode` field: `Jump` (default) or `Call`
- `Call` writes the current node to a single return slot
- A new `Return` routing mode pops the return slot
- The gate scoring mechanism is unchanged
- This is a separate future feature, not part of this design

### Scope of changes

#### Contracts (shared types)

| File | Change |
|------|--------|
| `contracts/routing.rs` | **New file.** `RouteTarget`, `MAX_GATE_SLOTS`. |

#### Runtime (execution + routing)

| File | Change |
|------|--------|
| `runtime/routing.rs` | Delete `RouteDecision`, `resolve_route_index`. Add `RouteGateMap`, `resolve_gated_route`. |
| `runtime/types.rs` | `NodeResult.route` → `.route_gates`. Collapse `exhausted_with_route` into `exhausted`. |
| `runtime/mesh.rs` | Use `resolve_gated_route`. |
| `runtime/vm.rs` | `route_target` → `route_gates`. `WriteRouteTarget` → `WriteRouteGate`. Opcode cost 0.10. |
| `runtime/traced_vm.rs` | Same as `vm.rs`, plus trace capture. |
| `runtime/traced_mesh.rs` | Same as `mesh.rs`, plus trace construction. |
| `runtime/cgp/effects.rs` | `RouterOutput` → `RouterGate(slot)` in effects pass. |
| `runtime/cgp/traced.rs` | Same as effects, plus trace capture. |
| `runtime/cgp/execute.rs` | Exhaustion path returns `RouteGateMap::default()`. |
| `runtime/trace/domain.rs` | Replace `TraceRouteDecision`, delete `TraceRouteKind`, add `TraceGateScore`. Drop `VmTrace.final_route_value`. |

#### Genome (structure)

| File | Change |
|------|--------|
| `creature/genome/mod.rs` | `targets: Vec<NodeId>` → `Vec<RouteTarget>`. Delete `WriteRouteTarget`, add `WriteRouteGate`. |
| `creature/genome/cgp.rs` | `RouterOutput` → `RouterGate(u8)`. Derive `FIXED_SINK_COUNT`. Update `new_with_fixed_outputs()`. Update sink ordering and reindex all assertions. |
| `creature/genome/analysis.rs` | Recognize `WriteRouteGate` in write-class analysis. |
| `creature/genome/mesh_annotations.rs` | `MeshWriteClass::Route` matches `WriteRouteGate`. |
| `creature/genome/cgp_mesh_annotations.rs` | `MeshWriteClass::Route` matches `RouterGate(_)`. |
| `creature/founder.rs` | Update founder genome construction. |

#### Mutation (evolution operators)

| File | Change |
|------|--------|
| `mutation/topology/mod.rs` | Add `MutateGateBias` to enum (14 variants). Dispatch to `routing.rs` and `structural.rs`. |
| `mutation/topology/routing.rs` | **New file.** AddRouteTarget, RemoveRouteTarget, RetargetNodeTarget, SwapRouteTargets, MutateGateBias, `lowest_unused_slot`. |
| `mutation/topology/structural.rs` | **New file.** AddNode, RemoveNode, CopyNode, SpliceNode, CopyMeshBackwardSlice, CopyMeshForwardSlice, ChangeEntryNode, SwapNodeBackend, RewriteNodeId. CopyNode/SpliceNode/slice operators construct `RouteTarget` per the construction policy above. |
| `mutation/topology/birth.rs` | `new_topology_birth_node` signature: `targets: Vec<NodeId>` → `Vec<RouteTarget>`. |
| `mutation/types/mod.rs` | Add `MutationOperator::TopologyMutateGateBias` variant. Complexity effect: Neutral. Update cross-consistency test. |
| `mutation/vm/operators.rs` | `WriteRouteTarget` → `WriteRouteGate` in instruction generation/mutation. |

#### Simulation + tests

| File | Change |
|------|--------|
| `simulation/seeding.rs` | Update if needed for new target format. |
| `creature/founder.rs` | `cgp_founder.rs` tests: update sink index assertions (RouterGate replaces RouterOutput, indices shift). |
| `tests/viability.rs` | Update test fixtures. |

#### Downstream (out of scope for this spec, follow-on work)

| Component | Impact |
|-----------|--------|
| `v3-server` (`transport/sample_protocol.rs`, `transport/sample_assembler.rs`) | Trace serialization wire format changes. `TraceRouteDecision` shape changes in JSON output. |
| Frontend (`frontend/src/types/trace.ts` + ~8 consumer files) | TypeScript trace types must match new `TraceRouteDecision` / `TraceGateScore` shape. `TraceRouteKind`, `VmTrace.final_route_value`, `MeshHopTrace.resolved_target_index` all change or are removed. |
