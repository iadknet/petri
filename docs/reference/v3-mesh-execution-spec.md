# V3 Mesh Execution Spec

Reference specification for evaluating a V3 creature mesh during cognition
runtime.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-runtime-config-spec.md`

---

## 1. Execution Entry and Runtime Boundary

Mesh execution is initiated by `tick/orchestrator.rs` during cognition.
Runtime receives split borrows so it can mutate execution state while reading
live dynamic introspection values.

This spec owns cognition/runtime chain behavior only. Top-level tick queue
construction, randomization, per-turn action arbitration, and newborn
eligibility are owned by `v3-tick-orchestration-spec.md`.

```rust
pub fn execute_creature_mesh(
    genome: &CreatureGenome,
    sensors: &SensorSnapshot,
    energy: &mut f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    graph_runtime: &mut GraphRuntimeState,
    config: &RuntimeConfig,
) -> MeshOutput
```

Boundary intent:
- `sensors/` owns local and extended perception snapshot assembly.
- `runtime/` owns node evaluation, routing, and soft-default behavior.
- `runtime` returns `MeshOutput` (`actions`, cost report, `priority_bid`) to
  tick orchestration.
- `tick/orchestrator` owns turn ordering and immediate action application.

---

## 2. Chain Evaluation Algorithm

Each tick evaluates a routing chain from `entry_node_id`.

```text
current_node_id = genome.entry_node_id
upstream_slots = [0.0; 12]
action_queue = []
visited = {}  # reset each tick
hops = 0
max_mesh_hops = validated(config.max_mesh_hops, default=1024, min=1)

loop:
  if hops >= max_mesh_hops:
    return action_queue_or_noop()

  mark current_node_id visited
  evaluate current node with upstream_slots -> NodeResult {
    output_slots: [f32; 12],
    route_gates: [f32; 8],
    terminal: bool,
    energy_exhausted: bool,
  }

  if energy_exhausted:
    return [WorldAction::NoOp]

  if terminal:
    return action_queue_or_noop()

  target = earliest argmax(gate_bias + route_gates[slot]) among unvisited IDs
  if no target remains:
    return action_queue_or_noop()

  target_id = target.target_id
  if target_id is missing from genome node set:
    return action_queue_or_noop()

  upstream_slots = output_slots
  current_node_id = target_id
  hops += 1
```

Notes:
- Entry upstream slots are zeroed only on the first hop.
- Each dispatched node is marked visited before execution and runs at most once.
- For every node evaluation, `output_slots` starts as a copy of incoming
  `upstream_slots`; backend slot writes overwrite addressed slots only.
- Slots not written during a node evaluation pass through unchanged.
- A visited top-scoring target falls through to the next eligible target.
  Ties keep the earliest eligible vector position. Missing winners still
  terminate softly without falling through. The cap bounds long acyclic chains.
- **Copy interference** (T11.F08). A node copy (`CopyNode`, or a mesh slice
  copy) is a faithful clone: it keeps its original's input references, output
  slots, and shared-memory addresses, so activating it in the original's chain
  position reproduces the original's behavior. Those addresses are not
  remapped, and nothing here makes them unique. When a copy and its original
  both run in one chain — a mesh slice copy can place them on the same path —
  they write the same output slots and the same shared-memory slots, and the
  node evaluated later in the chain wins: output slots are overwritten in
  place as the bus is handed downstream, and shared memory is committed per
  node evaluation. This is accounted for, not designed around; a lineage that
  wants two independent modules must move one copy's addresses by ordinary
  mutation.
- Canonical owner for `runtime.max_mesh_hops` defaults/validation:
  `v3-runtime-config-spec.md`.

---

## 3. Termination Contract

A single chain evaluation terminates on the first matching condition:

1. Energy reaches zero during node evaluation (`energy_exhausted = true`).
2. Node execution returns `terminal = true`.
3. `max_mesh_hops` failsafe triggers.
4. Runtime hits a broken routing state handled by soft default (preserve queue;
   return `NoOp` when queue is empty).

`max_mesh_hops` is configuration-controlled.

Safety rules:
- value must be `>= 1`
- invalid values (for example `0`) fall back to default (`1024`)
- the cap cannot be disabled

Graph internal recurrence rule:
- Phase 0 snapshots committed graph temporal state once per world tick.
- A visit is entered when the graph has a compute node or a wired effect
  surface; each entered visit evaluates once in index order, self/higher-index
  edges reading the tick-start outputs and lower-index edges the current-visit
  outputs. A zero-compute entered visit evaluates nothing and applies effects.
- Production mesh dispatch visits each node at most once; skipped modules hold state.
  Direct backend harness calls retain the frozen-base clock contract.
- Legacy convergence settings are accepted but ignored. Canonical semantics:
  `v3-graph-backend-spec.md`; config disposition: `v3-runtime-config-spec.md`.

---

## 4. Authoritative Soft-Default Matrix (Junk DNA Safe)

Runtime must never panic on malformed evolved topologies.

This section is the authoritative soft-default matrix for V3 chain-level runtime
behavior.

| Condition | Runtime behavior |
|---|---|
| `entry_node_id` missing from node set | Return `WorldAction::NoOp` |
| Routed target id missing | Preserve accumulated queue or return `NoOp` |
| No unvisited target remains | Preserve accumulated queue or return `NoOp` |
| Invalid gate slot | Runtime score is zero |
| All eligible effective scores are NaN or negative infinity | Earliest eligible target wins |
| `ReadInput` `ref_idx` out of range | Yield `0.0` |
| `ReadInput` `sub_idx` out of range (compound) | Yield `0.0` |
| Scalar input with `sub_idx > 0` | Yield `0.0` |
| `UpstreamSlot` slot out of range | Yield `0.0` |
| Node backend does not write an output slot | Preserve incoming `upstream_slots[slot]` |
| Graph edge source out of bounds | Input contributes `0.0` |
| Route points to a visited node | Filter that target before argmax |
| Graph state for `NodeId` missing | Allocate zero-initialized state and continue |

This policy intentionally allows junk DNA. Invalid offspring are culled by
selection pressure rather than strict genome repair.

---

## 5. Energy Metering

- VM nodes: energy deducted per opcode from VM cost table.
- Graph nodes: energy deducted per internal-node-per-visit evaluation.
- If energy is exhausted mid-node, evaluation halts and returns `NoOp`.
  No candidate graph temporal state/output or graph effects from the
  interrupted visit persist, including plasticity-cost exhaustion. Actual
  charges and entered work remain recorded; learned weights are not rolled back. The mesh returns `WorldAction::NoOp` immediately.

Dynamic introspection values (for example `EnergyCurrent`) are read live from
mutating `energy` during execution.

---

## Test-Mode Reproducibility Notes

Project-level determinism scope is canonical in root `AGENTS.md`: production
runtime determinism is not a product requirement.

This section defines V3-local harness controls for deterministic tests.

For deterministic tests, use a fixed mode that pins:
- Earliest eligible target wins score ties; NaN never beats an existing score.
- Float sanitation rules from VM/graph specs before routing decisions.
- Node iteration order (`nodes` order and internal graph order).
- Graph index order and frozen tick-start temporal read bases.
- Soft-default fallback constants (`0.0`, `NoOp`).

Tick-order/action-arbitration reproducibility controls are specified separately
in `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
Arbitration)`).

T11.F15 pairs new branches with backend gates and pass-through detours. T11.F18
chooses an empty Graph or Halt-only VM detour with equal probability independent
of the source backend; both preserve the bus and queued actions within budget.
An unwired Graph detour stays free; once mutation wires one of its effect
surfaces the detour is entered, pays one node equivalent and applies that
effect. See
`v3-mutation-spec.md`. Extra dispatches/instructions retain their normal
work and energy accounting. Neutrality requires sufficient budgets and does
not promise equal downstream live-energy introspection or exhaustion outcomes.
Production, full trace and compact observations share the same dispatch loop.
