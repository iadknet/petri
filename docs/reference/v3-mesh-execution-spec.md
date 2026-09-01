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
hops = 0
max_mesh_hops = validated(config.max_mesh_hops, default=1024, min=1)

loop:
  if hops >= max_mesh_hops:
    return action_queue_or_noop()

  evaluate current node with upstream_slots -> NodeResult {
    output_slots: [f32; 12],
    route: RouteDecision,
    terminal: bool,
    energy_exhausted: bool,
  }

  if energy_exhausted:
    return [WorldAction::NoOp]

  if terminal:
    return action_queue_or_noop()

  if node.targets is empty:
    return action_queue_or_noop()

  target_idx = resolve_route_index(node.targets.len(), route)

  target_id = node.targets[target_idx]
  if target_id is missing from genome node set:
    return action_queue_or_noop()

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

1. Node execution returns `terminal = true`.
2. Energy reaches zero during node evaluation (`energy_exhausted = true`).
3. `max_mesh_hops` failsafe triggers.
4. Runtime hits a broken routing state handled by soft default (preserve queue;
   return `NoOp` when queue is empty).

`max_mesh_hops` is configuration-controlled.

Safety rules:
- value must be `>= 1`
- invalid values (for example `0`) fall back to default (`1024`)
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
| VM route decision (`RouteDecision::VmWrap`) with negative/out-of-range/non-finite raw value | Map to signed route index and wrap with `rem_euclid(targets.len())` |
| Graph route decision (`RouteDecision::CgpNormalized`) with out-of-range/non-finite raw value | Sanitize + clamp to `[0.0, 1.0]`, then bin via `idx = min(floor(clamp01(raw) * targets.len()), targets.len()-1)` |
| Routed target id missing | Return `WorldAction::NoOp` |
| Routing requested but `targets` is empty | Return `WorldAction::NoOp` |
| `ReadInput` `ref_idx` out of range | Yield `0.0` |
| `ReadInput` `sub_idx` out of range (compound) | Yield `0.0` |
| Scalar input with `sub_idx > 0` | Yield `0.0` |
| `UpstreamSlot` slot out of range | Yield `0.0` |
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
  No partial state mutations (graph_state writes, memory writes) from the
  interrupted node persist. The mesh returns `WorldAction::NoOp` immediately.

Dynamic introspection values (for example `EnergyCurrent`) are read live from
mutating `energy` during execution.

---

## Test-Mode Reproducibility Notes

Project-level determinism scope is canonical in root `AGENTS.md`: production
runtime determinism is not a product requirement.

This section defines V3-local harness controls for deterministic tests.

For deterministic tests, use a fixed mode that pins:
- Routing conversion (`NaN -> -1`, `+inf -> i64::MAX`, `-inf -> i64::MIN`) and
  `rem_euclid` wrapping for VM route decisions.
- CGP routing conversion (`sanitize_f32(raw)`, clamp to `[0.0, 1.0]`, bounded
  binning to `0..targets.len()-1`) for graph route decisions.
- Float sanitation rules from VM/graph specs before routing decisions.
- Node iteration order (`nodes` order and internal graph order).
- Graph convergence loop order and stop criteria
  (`max_graph_relax_iters`, epsilon threshold, stable-pass rule).
- Soft-default fallback constants (`0.0`, `NoOp`).

Tick-order/action-arbitration reproducibility controls are specified separately
in `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
Arbitration)`).
