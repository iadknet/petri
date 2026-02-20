# V3 Mesh Execution Spec

Reference specification for evaluating a V3 creature mesh during cognition.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
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
max_mesh_hops = validated(config.max_mesh_hops, default=128, min=1)

loop:
  if hops >= max_mesh_hops:
    return WorldAction::NoOp

  evaluate current node with upstream_slots -> NodeResult {
    output_slots: [f32; 12],
    route_target_idx: f32,
    world_action: Option<WorldAction>,
  }

  if world_action is Some(action):
    return action

  if node.targets is empty:
    return WorldAction::NoOp

  route_idx_i64 =
    if route_target_idx is NaN:
      -1
    else if route_target_idx is +infinite:
      i64::MAX
    else if route_target_idx is -infinite:
      i64::MIN
    else:
      floor(route_target_idx).clamp(i64::MIN as f32, i64::MAX as f32) as i64

  target_idx = route_idx_i64.rem_euclid(node.targets.len() as i64) as usize

  target_id = node.targets[target_idx]
  if target_id is missing from genome node set:
    return WorldAction::NoOp

  upstream_slots = output_slots
  current_node_id = target_id
  hops += 1
```

Notes:
- Entry upstream slots are zeroed only on the first hop.
- If routing later returns to the entry node, routed upstream slots are used.
- For every node evaluation, `output_slots` starts as a copy of incoming
  `upstream_slots`; backend slot writes overwrite addressed slots only.
- Slots not written during a node evaluation pass through unchanged.
- No visited set is used; self-loops are legal.
- Canonical owner for `runtime.max_mesh_hops` defaults/validation:
  `v3-runtime-config-spec.md`.

---

## 3. Termination Contract

A single chain evaluation terminates on the first matching condition:

1. VM emits `WorldAction`.
2. Energy reaches zero during node evaluation.
3. `max_mesh_hops` failsafe triggers.
4. Runtime hits a broken routing state handled by soft default (`NoOp`).

`max_mesh_hops` is configuration-controlled.

Safety rules:
- value must be `>= 1`
- invalid values (for example `0`) fall back to default (`128`)
- the cap cannot be disabled

Graph internal recurrence rule:
- Within one graph node evaluation, runtime iterates internal relaxation passes
  until convergence or `max_graph_relax_iters`, whichever comes first.
- `max_graph_relax_iters` must be `>= 1` and cannot be disabled.
- Canonical owner for graph convergence config defaults/validation:
  `v3-runtime-config-spec.md`.

---

## 4. Authoritative Soft-Default Matrix (Junk DNA Safe)

Runtime must never panic on malformed evolved topologies.

This section is the authoritative soft-default matrix for V3 chain-level runtime
behavior.

| Condition | Runtime behavior |
|---|---|
| `entry_node_id` missing from node set | Return `WorldAction::NoOp` |
| Routed index (negative, out-of-range, or non-finite) | Map to signed route index and wrap with `rem_euclid(targets.len())` |
| Routed target id missing | Return `WorldAction::NoOp` |
| Routing requested but `targets` is empty | Return `WorldAction::NoOp` |
| `ReadInput` index out of range | Yield `0.0` |
| `UpstreamOutput` slot out of range | Yield `0.0` |
| Node backend does not write an output slot | Preserve incoming `upstream_slots[slot]` |
| Graph edge source out of bounds | Input contributes `0.0` |
| Graph convergence not reached before `max_graph_relax_iters` | Use last computed pass outputs and continue |
| Graph state for `NodeId` missing | Allocate zero-initialized state and continue |

This policy intentionally allows junk DNA. Invalid offspring are culled by
selection pressure rather than strict genome repair.

---

## 5. Energy Metering

- VM nodes: energy deducted per opcode from VM cost table.
- Graph nodes: energy deducted per internal-node-per-pass evaluation.
- If energy is exhausted mid-node, evaluation halts and returns `NoOp`.

Dynamic introspection values (for example `EnergyCurrent`) are read live from
mutating `energy` during execution.

---

## Test-Mode Reproducibility Notes

Project-level determinism scope is canonical in `AGENTS.md` (`Determinism Scope
(Canonical)`): production runtime determinism is not a product requirement.

This section defines V3-local harness controls for deterministic tests.

For deterministic tests, use a fixed mode that pins:
- Routing conversion (`NaN -> -1`, `+inf -> i64::MAX`, `-inf -> i64::MIN`) and
  `rem_euclid` wrapping.
- Float sanitation rules from VM/graph specs before routing decisions.
- Node iteration order (`nodes` order and internal graph order).
- Graph convergence loop order and stop criteria
  (`max_graph_relax_iters`, epsilon threshold, stable-pass rule).
- Soft-default fallback constants (`0.0`, `NoOp`).
