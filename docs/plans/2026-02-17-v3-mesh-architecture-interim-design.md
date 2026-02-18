# V3 Mesh Architecture — Interim Design

**Purpose:** Capture brainstorming decisions for the v3 creature mesh
architecture. This is an interim design reference, not an implementation plan.
It will be used to reconcile changes needed to the v3 architecture design,
reference specs, and stage plans before implementation begins.

**Date:** 2026-02-17

**Status:** Draft — pending architecture reconciliation

---

## 1. Design Decisions Summary

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Two backend types: **VM** and **Graph** | VM = general-purpose bytecode; Graph = v1-style signal processing. Different strengths, composable in mesh. |
| D2 | Graph backend = **mini computation graph** (not single-operator) | Single-operator graph nodes can't compute dynamic direction or do meaningful cognition alone. Mini-graph preserves v1's proven architecture. |
| D3 | **Only VM nodes can be terminal** (emit world actions). Graph nodes always route to a target. | Clean separation: Graph = perception/signal processing layer; VM = action computation layer. Every graph creature must end in a VM node. |
| D4 | VM nodes can **either** route to a target (graph or VM) **or** emit a world action | VM nodes are versatile — they serve as both action emitters and intermediate computation. |
| D5 | **Single entry point**, linear routing chain | Entry node evaluates → routes to target → target evaluates → routes or emits action. Simple execution model, no topological sort. |
| D6 | **Dynamic routing** to N candidate targets | Genome bakes in a list of candidate target NodeIds. Runtime computation selects which target. Enables behavioral "modes." |
| D7 | **All sensors globally available** to every node | No sensor forwarding between nodes. Every node reads sensors directly. Eliminates sensor curation complexity. |
| D8 | **12 custom output slots** per node, default 0.0 | Fixed-size output interface. All slots passed to target. Target decides what to read. Simple, uniform. |

---

## 2. Backend Types

### VM Backend (unchanged from Stage 3C)

Turing-complete bytecode interpreter. 38 opcodes. Dynamic action emission via
`EmitWorldAction`. Energy metered per-opcode.

**Capabilities:**
- Read any sensor directly (existing opcodes)
- Read upstream output slots from routing parent
- Compute arbitrary logic (loops, conditionals, memory access)
- Write to custom output slots (for routing to downstream nodes)
- Select routing target (for non-terminal VM nodes)
- Emit world action with dynamic direction/metadata (terminal only)

### Graph Backend (new — v1-inspired)

A mini computation graph contained within a single `NodeGenome`. Feed-forward
evaluation through internal nodes connected by weighted edges.

**Capabilities:**
- Read any sensor directly (via internal Input nodes)
- Read upstream output slots (via internal InputUpstreamSlot nodes)
- Process signals through operators with weighted connections
- Write to custom output slots (via internal CustomOutput nodes)
- Select routing target (via internal RouterOutput node)
- **Cannot emit world actions** — must route to a downstream node

**Internal node categories:**

| Category | Examples | Role |
|----------|----------|------|
| **Input (sensor)** | `InputFoodHere`, `InputEnergy`, `InputFoodDirection` | Read sensor values directly from world/creature state |
| **Input (upstream)** | `InputUpstreamSlot(n)` | Read output slot n from the upstream routing parent |
| **Compute** | `WeightedSum`, `Threshold`, `Sigmoid`, `Tanh`, `Relu`, `Add`, `Multiply`, `Min`, `Max`, `Abs`, `Negate`, `Select`, `Clamp01`, `GreaterThan`, `Constant(f32)`, `DecayIntegrator`, `Momentum`, `Oscillator`, `AdaptiveGain` | Signal processing with weighted inputs |
| **Output (custom)** | `CustomOutput(slot_idx)` | Write computed value to output slot 0..11 |
| **Output (router)** | `RouterOutput` | Produce a value that indexes into the target list |

**Evaluation:** Iterate internal nodes in array order (same as v1). Each node
computes weighted sum of inputs from upstream internal nodes, applies its
operator, writes result. After full evaluation, CustomOutput and RouterOutput
values are collected.

---

## 3. Mesh Execution Model

```
Entry node evaluates
    │
    ├── VM terminal? → emit world action (EmitWorldAction opcode)
    │
    └── Has targets? → select target
            │
            │  Graph node: RouterOutput → index into targets list
            │  VM node: WriteRouteTarget opcode → index into targets list
            │
            ├── Pass 12 custom output slots to selected target
            │
            └── Target node evaluates (receives upstream slots + global sensors)
                    │
                    ├── VM terminal? → emit world action
                    └── Has targets? → select next target → ...

If chain exhausts without emitting → default to WorldAction::NoOp
```

**Rules:**
- One entry point per creature (`entry_node_id`)
- Entry node can be Graph or VM
- Each node evaluates at most once per tick
- Chain must eventually reach a terminal VM node to emit an action
- Energy deducted during each node's evaluation
  - VM: per-opcode cost (existing model)
  - Graph: per-internal-node cost
- Max chain depth to prevent infinite routing loops (configurable)

---

## 4. Node Interface

### NodeGenome (genome-level)

```
NodeGenome {
    node_id: NodeId,
    backend: BackendDef,          // Vm(VmBackendDef) or Graph(GraphBackendDef)
    targets: Vec<NodeId>,         // candidate routing targets (empty = terminal VM)
}
```

**Constraint:** If `backend` is `Graph`, then `targets` must be non-empty
(graph nodes cannot be terminal).

### Sensor Access

All nodes read sensors directly — no forwarding between nodes.

- **Graph nodes:** internal `InputFoodHere`, `InputEnergy`, etc. reference any
  sensor value
- **VM nodes:** `ReadInput` with InputReferences, `ReadNeighborCell`,
  `ReadSensorCell`, etc. (existing opcodes)

### Upstream Output Access

When a node receives output slots from its routing parent:

- **Graph nodes:** internal `InputUpstreamSlot(n)` reads upstream output slot n
- **VM nodes:** new `InputReference::UpstreamOutput { slot: u8 }` variant,
  readable via `ReadInput`

### Custom Output Slots

Each node can write to 12 output slots (indices 0..11), all default to 0.0:

- **Graph nodes:** internal `CustomOutput(slot_idx)` nodes write their
  computed value to the specified slot
- **VM nodes:** `WriteInternalPayload(slot_idx, src_reg)` opcode
  (already exists in Stage 3C)

### Routing Target Selection

Nodes with non-empty `targets` select one target at runtime:

- **Graph nodes:** `RouterOutput` node produces scalar →
  `target_idx = clamp(round(value * (N-1)), 0, N-1)`
- **VM nodes:** new `WriteRouteTarget(idx)` opcode (or reuse meta buffer).
  Genome provides a default target index (index 0) used if VM doesn't
  explicitly set one.

---

## 5. Sample Creature Genomes

### Genome A: Single VM Node (current Stage 3C creature)

The simplest creature. One VM node, terminal, no graph, no routing.

```
┌─────────────────────────────────────────────────────┐
│  CreatureGenome                                     │
│  entry_node_id: NodeId(0)                           │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │  Node 0 (VM) ★ ENTRY, TERMINAL               │  │
│  │                                               │  │
│  │  targets: []                                  │  │
│  │                                               │  │
│  │  VM Program (21 instructions):                │  │
│  │    ReadInput r0, 0      ← FoodHere            │  │
│  │    ReadInput r2, 1      ← Energy              │  │
│  │    ...                                        │  │
│  │    EmitWorldAction(Eat)                       │  │
│  │     or EmitWorldAction(Move)                  │  │
│  │     or EmitWorldAction(Reproduce)             │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

### Genome B: Graph Perception → VM Action (simplest hybrid)

The minimum graph creature: a graph node processes sensors and routes to
a VM node that decides the action.

```
┌──────────────────────────────────────────────────────────────────────┐
│  CreatureGenome                                                      │
│  entry_node_id: NodeId(0)                                            │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Node 0 (Graph) ★ ENTRY, ROUTING                               │ │
│  │                                                                 │ │
│  │  targets: [NodeId(1)]                                           │ │
│  │                                                                 │ │
│  │  Internal graph:                                                │ │
│  │                                                                 │ │
│  │  InputFoodHere ──0.6──→ WeightedSum ──1.0──→ CustomOutput(0)   │ │
│  │  InputEnergy   ──0.4──╱                                         │ │
│  │                                                                 │ │
│  │  InputFoodDir  ──1.0──→ Sigmoid ──────1.0──→ CustomOutput(1)   │ │
│  │                                                                 │ │
│  │  RouterOutput: always 0.0 (only one target)                     │ │
│  │                                                                 │ │
│  │  Output slots: [food_energy_blend, processed_dir, 0, 0, ...]   │ │
│  └──────────────────────────────┬──────────────────────────────────┘ │
│                                 │ routes to Node 1                   │
│                                 │ passes 12 output slots             │
│                                 ▼                                    │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Node 1 (VM) TERMINAL                                          │ │
│  │                                                                 │ │
│  │  targets: []                                                    │ │
│  │                                                                 │ │
│  │  Reads:                                                         │ │
│  │    UpstreamOutput(0) → food_energy_blend from Node 0            │ │
│  │    UpstreamOutput(1) → processed_direction from Node 0          │ │
│  │    + any sensor directly (ReadNeighborCell, etc.)               │ │
│  │                                                                 │ │
│  │  VM Program:                                                    │ │
│  │    ReadInput r0, 0        ← upstream slot 0                     │ │
│  │    ReadInput r1, 1        ← upstream slot 1                     │ │
│  │    CmpGt r2, r0, r6      ← food+energy signal high?            │ │
│  │    JumpIfZero r2, +3                                            │ │
│  │    WriteWorldActionMeta 0, r1  ← direction from graph           │ │
│  │    EmitWorldAction 2      ← Move toward food                    │ │
│  │    ...                                                          │ │
│  │    EmitWorldAction 1      ← fallback: Eat                      │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

---

### Genome C: Multi-mode creature with conditional dispatch

Graph perception node routes to different VM "behavior modules" based on
conditions. Three behavioral modes: forage, reproduce, explore.

```
┌──────────────────────────────────────────────────────────────────────┐
│  CreatureGenome                                                      │
│  entry_node_id: NodeId(0)                                            │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Node 0 (Graph) ★ ENTRY, ROUTING                               │ │
│  │                                                                 │ │
│  │  targets: [NodeId(1), NodeId(2), NodeId(3)]                     │ │
│  │                                                                 │ │
│  │  Internal graph:                                                │ │
│  │                                                                 │ │
│  │  ┌─ Perception ────────────────────────────────────────────┐    │ │
│  │  │ InputFoodHere ──0.8──→ Threshold(0.1) ──→ CustomOut(0) │    │ │
│  │  │ InputEnergy   ──0.5──→ WeightedSum ──────→ CustomOut(1) │    │ │
│  │  │ InputFoodDir  ──1.0──→ Sigmoid ──────────→ CustomOut(2) │    │ │
│  │  └─────────────────────────────────────────────────────────┘    │ │
│  │                                                                 │ │
│  │  ┌─ Routing logic ────────────────────────────────────────┐    │ │
│  │  │ InputFoodHere ──1.0──╮                                  │    │ │
│  │  │ InputEnergy   ──0.8──┤──→ WeightedSum ──→ RouterOutput │    │ │
│  │  │                      ╯                                  │    │ │
│  │  │                                                         │    │ │
│  │  │ RouterOutput maps to:                                   │    │ │
│  │  │   ≈0.0 → targets[0] = Node 1 (forage)                  │    │ │
│  │  │   ≈0.5 → targets[1] = Node 2 (reproduce)               │    │ │
│  │  │   ≈1.0 → targets[2] = Node 3 (explore)                 │    │ │
│  │  └─────────────────────────────────────────────────────────┘    │ │
│  └──────────┬──────────────────┬──────────────────┬────────────────┘ │
│             │                  │                  │                   │
│    ┌─────── ▼ ───────┐ ┌───── ▼ ───────┐ ┌───── ▼ ───────┐         │
│    │ Node 1 (VM)     │ │ Node 2 (VM)    │ │ Node 3 (VM)    │        │
│    │ TERMINAL         │ │ TERMINAL       │ │ TERMINAL       │        │
│    │                  │ │                │ │                │         │
│    │ "Forage"        │ │ "Reproduce"    │ │ "Explore"      │        │
│    │                  │ │                │ │                │         │
│    │ Reads:           │ │ Reads:         │ │ Reads:         │        │
│    │  slot 0: food   │ │  slot 1: nrg   │ │  slot 2: dir   │        │
│    │  slot 2: dir    │ │  slot 0: food  │ │  + sensors     │        │
│    │  + sensors      │ │  + sensors     │ │                │         │
│    │                  │ │                │ │ Emits:         │        │
│    │ Emits:           │ │ Emits:         │ │  Move(random   │        │
│    │  Eat or          │ │  Reproduce     │ │   direction)   │        │
│    │  Move(toward     │ │  (direction +  │ │                │        │
│    │   food)          │ │   energy amt)  │ │                │        │
│    └─────────────────┘ └────────────────┘ └────────────────┘        │
└──────────────────────────────────────────────────────────────────────┘

Evaluation flow (example: food present, moderate energy):
  1. Node 0 evaluates internal graph
  2. RouterOutput ≈ 0.0 → selects targets[0] = Node 1 ("forage")
  3. Node 1 receives output slots [food_signal, energy_blend, food_dir, 0...]
  4. Node 1 VM reads upstream slots + sensors, computes direction
  5. Node 1 emits Move(East) → creature moves east toward food
```

---

### Genome D: Deep mesh — Graph → Graph → VM (perception chaining)

Graph nodes can route to other graph nodes, building deeper perception
pipelines. The chain must always end at a VM node.

```
┌──────────────────────────────────────────────────────────────────────┐
│  CreatureGenome                                                      │
│  entry_node_id: NodeId(0)                                            │
│                                                                      │
│  Node 0 (Graph) ★ ENTRY, ROUTING                                    │
│  │  "Low-level perception"                                           │
│  │  Processes raw sensors into basic feature signals                 │
│  │  targets: [NodeId(1)]                                             │
│  │  Writes: slot 0 = food_signal, slot 1 = danger_signal            │
│  │                                                                   │
│  ▼                                                                   │
│  Node 1 (Graph) ROUTING                                              │
│  │  "High-level decision"                                            │
│  │  Reads: upstream slots 0,1 + direct sensors                      │
│  │  Integrates features into behavioral mode selection               │
│  │  targets: [NodeId(2), NodeId(3)]                                  │
│  │  Writes: slot 0 = refined_direction, slot 1 = urgency            │
│  │  RouterOutput → Node 2 (feed) or Node 3 (flee)                   │
│  │                                                                   │
│  ├──────────────┐                                                    │
│  ▼              ▼                                                    │
│  Node 2 (VM)   Node 3 (VM)                                          │
│  TERMINAL      TERMINAL                                              │
│  "Feed"        "Flee"                                                │
│  Eat/Move      Move away                                             │
│  toward food   from danger                                           │
└──────────────────────────────────────────────────────────────────────┘
```

---

### Genome E: VM → VM routing (pure VM mesh)

VM nodes can also route to other VM nodes. No graph nodes required.
The entry VM does initial computation and delegates to specialized VMs.

```
┌──────────────────────────────────────────────────────────────────────┐
│  CreatureGenome                                                      │
│  entry_node_id: NodeId(0)                                            │
│                                                                      │
│  Node 0 (VM) ★ ENTRY, ROUTING                                       │
│  │  "Dispatcher"                                                     │
│  │  Reads sensors, computes which behavior to activate               │
│  │  targets: [NodeId(1), NodeId(2)]                                  │
│  │  Writes: slot 0 = food_assessment, slot 1 = direction_hint       │
│  │  WriteRouteTarget → 0 (eat mode) or 1 (move mode)                │
│  │                                                                   │
│  ├──────────────┐                                                    │
│  ▼              ▼                                                    │
│  Node 1 (VM)   Node 2 (VM)                                          │
│  TERMINAL      TERMINAL                                              │
│  "Eat mode"    "Move mode"                                           │
│  Emits: Eat    Emits: Move                                           │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 6. Valid Mesh Topologies

| Topology | Entry | Chain | Terminal | Valid? |
|----------|-------|-------|----------|--------|
| Single VM | VM | — | VM (same) | Yes (Stage 3C current) |
| Graph → VM | Graph | — | VM | Yes (simplest hybrid) |
| Graph → Graph → VM | Graph | Graph... | VM | Yes (deep perception) |
| VM → VM | VM | — | VM | Yes (pure VM mesh) |
| VM → Graph → VM | VM | Graph | VM | Yes (VM delegates to graph) |
| Graph only | Graph | — | Graph | **No** (graph cannot be terminal) |
| Single Graph | Graph | — | — | **No** (must end at VM) |

**Invariant:** Every valid mesh must terminate at a VM node.

---

## 7. Open Questions for Architecture Reconciliation

| # | Question | Notes |
|---|----------|-------|
| Q1 | How should `GraphBackendDef` be structured in Rust? | Likely `{ nodes: Vec<GraphNodeKind>, edges: Vec<GraphEdge> }` mirroring v1's `ComputationGraph`. |
| Q2 | Should the 12 output slot count be a fixed constant or a genome-level evolvable parameter? | Fixed is simpler. Evolvable adds mutation surface. |
| Q3 | How does the VM read upstream output slots? | Options: new `InputReference::UpstreamOutput { slot }` variant (read via `ReadInput`), or new dedicated opcode `ReadUpstreamSlot(dst, slot_idx)`. |
| Q4 | What graph compute node types to include? | Merge v1 set (Add, Multiply, Negate, Abs, Min, Max, Threshold, GreaterThan, Sigmoid, Tanh, Relu, Select, Constant) with v3 spec additions (DecayIntegrator, Momentum, Oscillator, AdaptiveGain, Clamp01, SumPool, MeanPool, MaxPool). |
| Q5 | How does the RouterOutput node combine inputs to produce a routing scalar? | Simplest: it's a regular compute node (e.g., WeightedSum) whose output is interpreted as a route index. `target_idx = clamp(round(value * (N-1)), 0, N-1)`. |
| Q6 | Should graph local state (DecayIntegrator, Momentum, etc.) live in `HashMap<NodeId, Vec<f32>>`? | Yes — decided during brainstorming. Needs reconciliation with `CreatureState`. |
| Q7 | What mutation operators apply to the graph's internal structure? | Add/remove internal nodes, add/remove edges, change weights, change operator types, jitter operator params. Subset of the lifecycle spec's 11 mutation operators. |
| Q8 | How does the `test` founder exercise graph features? | Needs at least one graph NodeGenome routing to a VM node. |
| Q9 | Does the `simple` founder stay VM-only? | Recommend yes. Graph nodes reachable via `add_node` mutation over generations. |
| Q10 | How is graph energy cost calculated? | Proposed: `base_tariff * operator_cost_multiplier` per internal node evaluated. Deducted from creature energy before routing. |
| Q11 | What reference specs need updating? | `v3-graph-operator-spec.md`, `v3-genome-sensor-spec.md`, `v3-creature-lifecycle-spec.md`, `v3-vm-isa-spec.md`, `v3-architecture-design.md`. |

---

## 8. Impact on Existing v3 Specs

| Spec | Change needed |
|------|---------------|
| `v3-architecture-design.md` | Add Graph backend, update NodeGenome schema, add mesh execution model, update `runtime/graph.rs` description, add `targets` to NodeGenome (replacing OutputDefinition). |
| `v3-graph-operator-spec.md` | Major rewrite: operators become internal compute node types within mini-graph. Add Input, CustomOutput, and RouterOutput node types. Add edge/weight model. Remove single-operator `GraphBackendDef`. |
| `v3-genome-sensor-spec.md` | Update `BackendDef` enum, `GraphBackendDef` structure, replace `OutputDefinition` with `targets: Vec<NodeId>`, add `UpstreamOutput` InputReference variant, add `InputUpstreamSlot` graph node type. |
| `v3-creature-lifecycle-spec.md` | Add graph-specific mutation operators (internal node add/remove, edge weight jitter, operator swap). Update structural invariants. Add graph validity rules. |
| `v3-vm-isa-spec.md` | Add routing opcode (`WriteRouteTarget` or equivalent). Document upstream output slot reading semantics. |

---

## 9. Decisions NOT Made (deferred)

- **Inventory system** — pickup/place actions, slot-addressed storage
- **SensorFrame** — Chebyshev radius scan (ReadSensorCell/Creature/Summary)
- **Phenotype evolution** — mutation changes phenotype RGB
- **Advanced mutation** — full 11-operator mutation system with graph support
- **Multi-node evaluation limits** — max nodes per creature, max chain depth
- **Routing cycle prevention** — max depth limit or visited-set check
