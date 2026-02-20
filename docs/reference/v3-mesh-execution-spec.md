# V3 Mesh Execution Spec

Reference specification for evaluating a V3 creature mesh during cognition.

Status: Active

---

## 1. Execution Entry and Runtime Boundary

Mesh execution is initiated by `tick/orchestrator.rs` during cognition.
Runtime receives split borrows so it can mutate execution state while reading
live dynamic introspection values.

```rust
pub fn execute_creature_mesh(
    genome: &CreatureGenome,
    static_inputs: &StaticInputs,
    energy: &mut Energy,
    memory: &mut [u8; 1024],
    graph_state: &mut HashMap<NodeId, Vec<f32>>,
    config: &MeshConfig,
) -> WorldAction
```

Boundary intent:
- `sensors/` owns static snapshot assembly.
- `runtime/` owns node evaluation, routing, and soft-default behavior.
- `contracts/` boundary remains `WorldAction` only.

---

## 2. Chain Evaluation Algorithm

Each tick evaluates a routing chain from `entry_node_id`.

```text
current_node_id = genome.entry_node_id
upstream_slots = [0.0; 12]
hops = 0

loop:
  if hops >= MAX_MESH_HOPS:
    return WorldAction::NoOp

  evaluate current node -> NodeResult {
    output_slots: [f32; 12],
    route_target_idx: f32,
    world_action: Option<WorldAction>,
  }

  if world_action is Some(action):
    return action

  if node.targets is empty:
    return WorldAction::NoOp

  target_idx = route_target_idx.floor()
  if route_target_idx is NaN or infinite:
    target_idx = 0
  target_idx = max(target_idx, 0)

  target_id = node.targets[target_idx]
  if target_id missing/out of range:
    return WorldAction::NoOp

  upstream_slots = output_slots
  current_node_id = target_id
  hops += 1
```

Notes:
- Entry upstream slots are zeroed only on the first hop.
- If routing later returns to the entry node, routed upstream slots are used.
- No visited set is used; self-loops are legal.

---

## 3. Termination Contract

A single chain evaluation terminates on the first matching condition:

1. VM emits `WorldAction`.
2. Energy reaches zero during node evaluation.
3. `MAX_MESH_HOPS` failsafe triggers.
4. Runtime hits a broken routing state handled by soft default (`NoOp`).

`MAX_MESH_HOPS` is a hardcoded runtime failsafe and must not be disabled by
configuration.

---

## 4. Soft Default Contract (Junk DNA Safe)

Runtime must never panic on malformed evolved topologies.

| Condition | Runtime behavior |
|---|---|
| `entry_node_id` missing from node set | Return `WorldAction::NoOp` |
| Routed target id missing | Return `WorldAction::NoOp` |
| Routing requested but `targets` is empty | Return `WorldAction::NoOp` |
| `ReadInput` index out of range | Yield `0.0` |
| `UpstreamOutput` slot out of range | Yield `0.0` |
| Graph edge source invalid (`>= current_idx` or OOB) | Input contributes `0.0` |
| Graph state for `NodeId` missing | Allocate zero-initialized state and continue |

This policy intentionally allows junk DNA. Invalid offspring are culled by
selection pressure rather than strict genome repair.

---

## 5. Energy Metering

- VM nodes: energy deducted per opcode from VM cost table.
- Graph nodes: energy deducted per internal node evaluation.
- If energy is exhausted mid-node, evaluation halts and returns `NoOp`.

Dynamic introspection values (for example `EnergyCurrent`) are read live from
mutating `energy` during execution.

---

## 6. Determinism Requirements

To preserve reproducibility:
- Routing conversion uses explicit floor/clamp/NaN fallback rules.
- Float sanitation rules from VM/graph specs apply before routing decisions.
- Node iteration order is deterministic (`nodes` order and internal graph order).
- Soft-default fallbacks are deterministic constants (`0.0`, `NoOp`).
