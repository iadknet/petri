# V3 Mesh Refactor Design

**Goal:** Re-architect v3 creature brains from single VM nodes to a mesh of VM and Graph nodes with routing, output slots, and unified sensor access.

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** Architecture decisions for the mesh refactor including documentation strategy, code module changes, genome schema, mesh execution model, sensor categories, and implementation order. Excludes detailed implementation plans (deferred to companion plan files).

**Docs Impact:**

| Doc | Action |
|-----|--------|
| `docs/plans/2026-02-14-v3-architecture-design.md` | Archive to `docs/plans/archive/` |
| `docs/plans/2026-02-18-v3-mesh-refactor-design.md` | New slim primary architecture doc |
| `docs/reference/v3-vm-isa-spec.md` | Update in-place (add routing opcodes) |
| `docs/reference/v3-graph-operator-spec.md` | Rewrite → `v3-graph-backend-spec.md` |
| `docs/reference/v3-genome-sensor-spec.md` | Split → `v3-genome-spec.md` + `v3-sensor-spec.md` |
| `docs/reference/v3-creature-lifecycle-spec.md` | Refactor to lifecycle overview/index |
| `docs/reference/v3-mutation-spec.md` | New |
| `docs/reference/v3-reproduction-spec.md` | New |
| `docs/reference/v3-evolution-observability-spec.md` | New |
| `docs/reference/v3-mesh-execution-spec.md` | New |
| `docs/README.md` | Update reference links |

**Supersedes:** `docs/plans/2026-02-14-v3-architecture-design.md` (archived, not deleted)

**Superseded-By:** none

---

## Goal Alignment

- **GP-01**: Mesh architecture enables richer emergent behavior through multi-node creature brains with Graph perception layers, VM action layers, dynamic routing between behavioral modules, and iterative feedback loops.
- **GP-02**: Clean separation maintained — genome types define structure, runtime executes it, contracts bridge module boundaries. New concepts (output slots, routing) stay runtime-internal.
- **GP-03**: Three-layer testing strategy (contract, intent, integration) extends to mesh concepts. Non-collapse contract validates mesh with a multi-node founder.

---

## Boundary Impact

- All changes within `v3/crates/v3-core/` — no new crates
- Graph backend lives in `runtime/graph.rs` (not a separate crate)
- Module dependency direction unchanged
- `contracts/` boundary preserved: WorldAction crosses runtime→tick, output slots stay runtime-internal

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `kernel/` | keep | World primitives are completely independent of creature brain architecture |
| `contracts/` | keep | WorldAction remains the runtime→tick interface; inter-node data stays in runtime |
| `tick/actions.rs` | keep | Action execution is independent of how creatures decide actions |
| `runtime/` | change | Single-node executor → mesh chain evaluator; add graph backend |
| `creature/genome.rs` | change | NodeGenome gains targets, GraphBackendDef, unified InputReference |
| `sensors/` | change (minor) | Split static/dynamic introspection; gather static snapshot once |
| `v3-graph-backend-spec.md` | keep | Active canonical graph backend spec for mini computation graph semantics |
| `v3-genome-spec.md` + `v3-sensor-spec.md` | keep | Active canonical split of genome structure vs sensor model contracts |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should v3 use a separate crate for graph backend? | No — keep in v3-core as `runtime/graph.rs` | user+agent | resolved |
| How should GraphBackendDef store edges? | Co-located inputs per node (not separate edge array) | user+agent | resolved |
| Should output slot count be fixed or evolvable? | Fixed constant (12) | user+agent | resolved |
| Where does graph local state live? | `HashMap<NodeId, Vec<f32>>` on CreatureState keyed by mesh node id | user+agent | resolved |
| How is graph energy cost calculated? | Flat per-internal-node cost | user+agent | resolved |
| How should entry node upstream slots be handled? | Zeroed `[0.0; 12]` only at chain start; later re-entry uses routed upstream slots | user+agent | resolved |
| Are NeighborCell/NeighborCreature world inputs? | Yes — unified under `WorldInputKey` | user+agent | resolved |
| Should GraphNodeKind have explicit sensor input variants? | No — use `InputRef(idx)` into shared `input_refs` | user+agent | resolved |
| Is terminality static or dynamic? | Dynamic — VM nodes decide at runtime whether to emit or route | user+agent | resolved |
| Should the simple founder be VM-only? | No — simple founder should be a mesh with multiple node types | user+agent | resolved |
| How should sensors handle values that change during mesh evaluation? | Three categories: World (snapshot), Static Introspection (snapshot), Dynamic Introspection (live from creature state) | user+agent | resolved |
| Should mesh execution rely only on energy for termination? | No — energy remains primary, with configurable `max_mesh_hops` failsafe (validated, non-zero) | user+agent | resolved |

---

## 1. Documentation Strategy

### New primary architecture doc

This document is the primary architecture/design plan for the mesh refactor. It is focused on:
- Module boundaries and dependency flow
- Mesh execution model (with diagram)
- Data flow diagrams
- References to spec files for all detailed designs
- Completed stages (1–3C) archived separately

### Reference spec changes

| Current File | Action | Result |
|--------------|--------|--------|
| `v3-vm-isa-spec.md` | Update in-place | Add `WriteRouteTarget`, upstream slot reading via `ReadInput` + `UpstreamOutput` ref |
| `v3-graph-operator-spec.md` | Rewrite | → `v3-graph-backend-spec.md`: mini computation graph, co-located edges, internal node kinds, stateful operators, evaluation rules |
| `v3-genome-sensor-spec.md` | Split | → `v3-genome-spec.md`: NodeGenome, BackendDef, GraphBackendDef, CreatureGenome, validation rules |
| | | → `v3-sensor-spec.md`: Three-category sensor model, InputReference enum, sensor resolution |
| `v3-creature-lifecycle-spec.md` | Refactor | Lifecycle phase/invariant overview; links to detailed split mutation/reproduction specs |
| (new) | Create | `v3-mutation-spec.md`: mutation engine boundaries, domain mutators, rollback+skip policy, parseability gate |
| (new) | Create | `v3-reproduction-spec.md`: offspring draft contract, inheritance semantics, spawn queue, first-wins arbitration |
| (new) | Create | `v3-evolution-observability-spec.md`: minimal counters, event schemas, skip/rejection reason contracts |
| (new) | Create | `v3-mesh-execution-spec.md`: Chain evaluation, routing, output slots, energy metering, dynamic terminality |

### Archived docs

Move to `docs/plans/archive/`:
- `2026-02-14-v3-architecture-design.md`
- Completed stage plans (2026-02-14-v3-phase1, 2026-02-17-v3-stage2, 2026-02-17-v3-stage3a/b/c)

---

## 2. Mesh Execution Model

### Chain evaluation

```
execute_creature_mesh(genome, static_inputs, energy, memory, graph_state, config)
│
├── current_node = genome.entry_node_id
├── upstream_slots = [0.0; 12]
├── hops = 0
│
└── LOOP:
    │
    ├── If hops >= max_mesh_hops → return NoOp
    │
    ├── Evaluate current_node (VM or Graph)
    │   ├── Costs energy (per-opcode for VM, per-internal-node for Graph)
    │   ├── If insufficient energy → return NoOp
    │   └── Returns: NodeResult { output_slots, route_target, world_action? }
    │
    ├── If world_action emitted (VM only) → return WorldAction (TERMINAL)
    │
    ├── If no targets and no action → return NoOp
    │
    ├── route_idx = map(route_target):
    │      NaN -> -1, +inf -> i64::MAX, -inf -> i64::MIN,
    │      finite -> floor(value) clamped to i64 range
    ├── target_idx = route_idx.rem_euclid(targets.len() as i64) as usize
    ├── Select target id: targets[target_idx]
    ├── If selected target id missing from genome -> NoOp
    ├── upstream_slots = node_result.output_slots
    ├── current_node = target (may be same node — self-targeting valid)
    └── hops += 1
```

### Key rules

- **No visited set** — nodes can be evaluated multiple times; loops and self-targeting are valid
- **Failsafe hop cap** — `max_mesh_hops` is configurable but validated (`>= 1`) and cannot be disabled (prevents deadlock if costs are zero)
- **Dynamic terminality** — VM nodes decide at runtime whether to emit a WorldAction (terminal) or route to a target (non-terminal); the same node can do either on different ticks
- **Junk DNA allowed** — broken routing and dangling IDs are tolerated; runtime degrades to `NoOp` rather than panicking
- **Output slots** — 12 f32 values (fixed constant), default 0.0, passed from each node to the next
- **Entry node upstream semantics** — zeroed `[0.0; 12]` only on first hop; later routes into entry receive routed slots like any other node

### Energy metering

- **VM nodes**: per-opcode cost (existing model, unchanged)
- **Graph nodes**: flat per-internal-node cost (configurable)
- Energy drains during evaluation — downstream nodes see current (reduced) energy via Dynamic Introspection

### Soft default runtime contract

- Missing `entry_node_id` → `WorldAction::NoOp`
- Missing routed target id (dangling NodeId) → `WorldAction::NoOp`
- Routed index (negative, out-of-range, non-finite) wraps with signed `rem_euclid`
- Empty `targets` when routing is required → `WorldAction::NoOp`
- Invalid input ref / upstream slot read → `0.0`
- Missing graph state for a node id → allocate zero-initialized state for that node

---

## 3. Genome Schema

### NodeGenome

```rust
pub struct NodeGenome {
    pub node_id: NodeId,
    pub input_refs: Vec<InputReference>,  // unified sensor mapping for both backends
    pub backend_def: BackendDef,
    pub targets: Vec<NodeId>,             // routing targets (may contain dangling ids; runtime handles safely)
}
```

`input_refs` is the unified sensor interface. Both VM (`ReadInput(idx)`) and Graph (`InputRef(idx)`) read sensors through the same indirection. New sensors only require a new `InputReference` variant, not changes to `GraphNodeKind`.

### BackendDef

```rust
pub enum BackendDef {
    Vm(VmBackendDef),        // existing (+ routing opcode additions)
    Graph(GraphBackendDef),  // new
}
```

### GraphBackendDef (co-located inputs)

```rust
pub struct GraphBackendDef {
    pub nodes: Vec<GraphInternalNode>,
}

pub struct GraphInternalNode {
    pub kind: GraphNodeKind,
    pub inputs: Vec<GraphInput>,   // weighted connections from earlier nodes
}

pub struct GraphInput {
    pub source_idx: u16,   // intended source node index; invalid indexes soft-default to 0.0 at runtime
    pub weight: f32,
}
```

Each internal node carries its own input edges. During evaluation, iterate nodes in array order — inputs are co-located, no edge lookup needed. If mutation produces invalid `source_idx` (`>= current_idx` or out of bounds), that input contributes `0.0` (junk-DNA safe fallback).

### GraphNodeKind

```rust
pub enum GraphNodeKind {
    // Read sensor by index into NodeGenome.input_refs
    InputRef(u8),

    // Read upstream output slot from routing parent
    InputUpstreamSlot(u8),     // slot 0..11

    // Compute operators
    Constant(f32),
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    Threshold(f32),
    GreaterThan,
    Sigmoid,
    Tanh,
    Relu,
    Select,
    Clamp01,
    WeightedSum,
    DecayIntegrator(f32),      // stateful: decay rate
    Momentum(f32),             // stateful: momentum param
    Oscillator(f32),           // stateful: frequency
    AdaptiveGain,              // stateful

    // Output nodes
    CustomOutput(u8),          // writes to output slot 0..11
    RouterOutput,              // produces routing target selection value
}
```

No sensor-specific input variants — `InputRef(idx)` reads from the shared `input_refs` list, just like the VM's `ReadInput`.

### InputReference (three categories + upstream)

```rust
pub enum InputReference {
    World(WorldInputKey),
    StaticIntrospection(StaticIntrospectionKey),
    DynamicIntrospection(DynamicIntrospectionKey),
    UpstreamOutput { slot: u8 },
}

pub enum WorldInputKey {
    FoodHere,
    NeighborCellFood(u8),          // direction_idx 0..7
    NeighborCellBarrier(u8),
    NeighborCellOccupied(u8),
    NeighborCreaturePresent(u8),
}

pub enum StaticIntrospectionKey {
    Generation,
    AgeTicks,
}

pub enum DynamicIntrospectionKey {
    EnergyCurrent,
    EnergyConsumedThisTick,
}
```

### Sensor resolution

- **World** and **StaticIntrospection**: resolved from a snapshot gathered once per creature per tick (before mesh evaluation starts)
- **DynamicIntrospection**: resolved live from creature state during node evaluation (energy changes as nodes execute)
- **UpstreamOutput**: resolved from the 12 output slots passed by the routing parent

---

## 4. Code Module Changes

### Unchanged (no code changes)

- `kernel/` — world primitives, position, direction, occupancy
- `tick/actions.rs` — action execution logic
- `seed.rs` — creature placement
- `lib.rs` — SimulationState

### Minor / additive changes

- `contracts/inputs.rs` — restructure for three sensor categories
- `contracts/outputs.rs` — minimal changes
- `config/` — add `GraphConfig`, `MeshConfig`, graph mutation rates
- `sensors/` — produces static snapshot (World + StaticIntrospection)
- `creature/state.rs` — add `graph_state: HashMap<NodeId, Vec<f32>>` for stateful graph operators
- `tick/orchestrator.rs` — change runtime call to mesh executor

### Significant refactors (in-place)

- `creature/genome.rs` — NodeGenome changes, add GraphBackendDef, unify InputReference
- `creature/founders.rs` — new mesh founder (Graph → VM)
- `creature/mutation.rs` — add graph mutation operators
- `creature/reproduction.rs` — handle graph state initialization

### New / rewrite

- `runtime/executor.rs` — rewrite as mesh chain evaluator
- `runtime/graph.rs` — new graph evaluation engine
- `runtime/vm.rs` — add routing opcodes, upstream slot reading

### Runtime module structure

```
runtime/
├── mod.rs         ← public API: execute_creature_mesh()
├── mesh.rs        ← mesh chain evaluator
├── vm.rs          ← VM node evaluator (existing + additions)
├── graph.rs       ← graph node evaluator (new)
└── types.rs       ← OutputSlots, NodeResult, ChainContext (runtime-internal)
```

Inter-node data (output slots, routing state) lives in `runtime/types.rs` — it does not cross into `contracts/`. WorldAction remains in `contracts/outputs.rs` as the runtime→tick interface.

---

## 5. Testing Strategy

### Existing strategy (unchanged)

Three-layer testing: contract tests, intent verification tests, integration tests. No mocks in core.

### New test areas

1. **Graph evaluation** — internal node computation, weighted inputs, stateful operators, output slot collection
2. **Mesh chain evaluation** — routing, output slot passing, energy + hop-cap termination, self-targeting loops
3. **Dynamic terminality** — VM node conditionally emitting vs. routing
4. **Genome validation + soft defaults** — parseability checks plus runtime behavior for dangling targets/missing entry/invalid graph edges
5. **Graph mutation** — internal node add/remove, weight jitter, operator swap

### Non-collapse contract

- Simple founder is a mesh creature (Graph → VM) — mesh is validated by default
- Contract parameters unchanged: 2000 ticks, seed 42, population survival
- Mesh-specific: output slots pass non-zero values, routing transitions occur

---

## 6. Implementation Order

```
1. Genome schema refactor
   ├── Update NodeGenome (targets, remove OutputDefinition)
   ├── Add GraphBackendDef, GraphInternalNode, GraphNodeKind
   ├── Unify InputReference (World, StaticIntrospection, DynamicIntrospection, Upstream)
   ├── Update CreatureGenome::validate()
   └── Fix compilation cascade (founders, mutation, reproduction)

2. Runtime: Graph evaluator (new)
   ├── runtime/graph.rs — array-order evaluation with co-located inputs
   ├── Per-internal-node energy cost
   ├── Output slot collection + router output
   └── Unit tests

3. Runtime: Mesh executor (rewrite)
   ├── runtime/mesh.rs — chain evaluation loop
   ├── Energy-based termination + configurable `max_mesh_hops` failsafe
   ├── Output slot passing, dynamic terminality
   └── Integration tests

4. Runtime: VM additions
   ├── WriteRouteTarget opcode
   ├── ReadInput with UpstreamOutput reference support
   ├── Configurable `max_vm_steps` per VM node evaluation
   └── Update VM tests

5. Config + sensors
   ├── GraphConfig, MeshConfig
   ├── Static/dynamic sensor split
   └── CreatureState.graph_state keyed by NodeId

6. Founder + mutation
   ├── New simple founder (multi-node mesh)
   ├── Graph mutation operators
   └── Graph state initialization for offspring

7. Viability gate
   └── Non-collapse contract with mesh founder
```

---

## 7. Risks

| Risk | Mitigation |
|------|------------|
| Genome schema cascade breaks many files | Compiler-driven: change structs, fix all errors. Types enforce correctness. |
| Graph evaluator performance (runs every tick for every creature) | Co-located inputs avoid edge lookups. Profile after implementation. |
| Routing/VM loops deadlock when costs are zero/misconfigured | Enforce validated configurable `max_mesh_hops` and `max_vm_steps` failsafes plus energy metering. |
| Losing details from archived docs | Archive, don't delete. New spec files reference archived docs. |
| Stateful graph operators (DecayIntegrator etc.) introduce hidden state bugs | Unit test each stateful operator in isolation. Test state persistence across ticks. |

---

**Review cycles:** 1
