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

Each tick evaluates a routing chain (one pass, until T19.F04 makes passes
plural) from `entry_node_id`. A node may be dispatched any number of times
within a pass (T19.F02): cycles and self-targets are legal, every dispatch
runs on the creature's live state, and the per-pass hop cap is the only
structural bound.

```text
current_node_id = genome.entry_node_id
upstream_slots = [0.0; 12]
action_queue = []
bid = 0.0          # recorded by SetPriorityBid, settled once below
hops = 0
max_mesh_hops = validated(config.max_mesh_hops, default=64, min=1)

reason = loop:
  if hops >= max_mesh_hops:
    pass_cap_hits += 1
    break MaxHopsReached

  charge the per-tick hop ramp (Section 5); if energy <= 0: break EnergyExhausted
  evaluate current node with upstream_slots -> NodeResult {
    output_slots: [f32; 12],
    route_gates: [f32; 8],
    terminal: bool,
    energy_exhausted: bool,
  }

  if energy_exhausted: break EnergyExhausted
  if terminal:         break ActionEmitted

  target = earliest argmax(gate_bias + route_gates[slot]) over all targets
  if no target:                                    break NoTargets
  if target.target_id is missing from the node set: break MissingNode

  upstream_slots = output_slots
  current_node_id = target.target_id
  hops += 1

if reason != EnergyExhausted:
  settle the bid once (Section 5); an all-in makes reason = EnergyExhausted
return [NoOp] if reason == EnergyExhausted else action_queue_or_noop()
```

Notes:
- Entry upstream slots are zeroed only on the first hop.
- A dispatch is one hop; a node dispatched again reads its own committed
  state and outputs from its previous dispatch (graph backends) and the
  shared memory its previous dispatch committed (both backends).
- For every node evaluation, `output_slots` starts as a copy of incoming
  `upstream_slots`; backend slot writes overwrite addressed slots only.
- Slots not written during a node evaluation pass through unchanged.
- Ties keep the earliest vector position. A missing winner terminates
  softly without falling through. The cap bounds every pass, cyclic or not:
  a pass dispatches at most `max_mesh_hops` nodes, and reaching the cap keeps
  the queue (`MaxHopsReached`), increments `WorkCounters.pass_cap_hits`, and
  is paid through the per-tick hop ramp. At the defaults (cap 64, allowance
  32, cost `1e-4`) one capped pass costs `0.0528`, a NoOp's worth.
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

1. Energy reaches zero during the hop ramp or node evaluation
   (`energy_exhausted = true`), or the bid settlement is an all-in.
2. Node execution returns `terminal = true`.
3. The per-pass hop cap `max_mesh_hops` is reached before a dispatch.
4. Runtime hits a broken routing state handled by soft default (preserve queue;
   return `NoOp` when queue is empty).

`max_mesh_hops` is configuration-controlled.

Safety rules:
- value must be `>= 1`
- invalid values (for example `0`) fall back to default (`64`)
- the cap cannot be disabled

Graph internal recurrence rule:
- Phase 0 decays initialized eligibility once per world tick and snapshots
  nothing; graph temporal state is live within the tick (T19.F02).
- A visit is entered when the graph has a compute node or a wired effect
  surface; each entered visit evaluates once in index order, self/higher-index
  edges reading the last committed outputs and lower-index edges the
  current-visit outputs. A zero-compute entered visit evaluates nothing and
  applies effects.
- A node dispatched again within the tick starts from its previous
  dispatch's commit; skipped modules hold state.
- Canonical semantics: `v3-graph-backend-spec.md`; config disposition:
  `v3-runtime-config-spec.md`.

---

## 4. Authoritative Soft-Default Matrix (Junk DNA Safe)

Runtime must never panic on malformed evolved topologies.

This section is the authoritative soft-default matrix for V3 chain-level runtime
behavior.

| Condition | Runtime behavior |
|---|---|
| `entry_node_id` missing from node set | Return `WorldAction::NoOp` |
| Routed target id missing | Preserve accumulated queue or return `NoOp` |
| No target | Preserve accumulated queue or return `NoOp` |
| Invalid gate slot | Runtime score is zero |
| All effective scores are NaN or negative infinity | Earliest target wins |
| `ReadInput` `ref_idx` out of range | Yield `0.0` |
| `ReadInput` `sub_idx` out of range (compound) | Yield `0.0` |
| Scalar input with `sub_idx > 0` | Yield `0.0` |
| `UpstreamSlot` slot out of range | Yield `0.0` |
| Node backend does not write an output slot | Preserve incoming `upstream_slots[slot]` |
| Graph edge source out of bounds | Input contributes `0.0` |
| Per-pass hop cap reached | Preserve accumulated queue or return `NoOp`; count `pass_cap_hits` |
| Graph state for `NodeId` missing | Allocate zero-initialized state and continue |

This policy intentionally allows junk DNA. Invalid offspring are culled by
selection pressure rather than strict genome repair.

---

## 5. Energy Metering

- VM nodes: energy deducted per opcode from VM cost table.
- Graph nodes: energy deducted per internal-node-per-visit evaluation.
- Per-tick hop ramp (T19.F01): the k-th mesh hop of a world tick (k from 1)
  is charged `runtime.hop_ramp_cost * max(0, k - runtime.hop_ramp_allowance)`
  before the node is dispatched. Over n hops in a tick the ramp totals
  `hop_ramp_cost * m * (m + 1) / 2` with `m = max(0, n - hop_ramp_allowance)`.
  The hop index never resets within a tick and starts at 1 at every tick, so
  sustained neural activity within one tick costs metabolism. Defaults:
  allowance `32`, cost `1e-4`. The charge is a direct debit against the
  creature's energy and is attributed as the `mesh_ramp` energy flow and death
  cause. If it takes energy to `<= 0.0` the node is not dispatched, the
  evaluation ends `EnergyExhausted`, the queue is discarded, and the creature
  acts `NoOp`; the hop is still counted and recorded as a dispatch.
- Priority bid (T19.F02): `SetPriorityBid` records `max(0, regs[src])`
  (non-finite reads as 0) as the creature's bid, last write wins across the
  whole evaluation including revisits, and pays only its opcode cost. The
  bid is settled exactly once, after the chain, on every exit that is not
  already `EnergyExhausted`: `paid = min(bid, energy)`. If `bid >= energy`
  the creature goes all-in: energy is `0.0`, the evaluation ends
  `EnergyExhausted` with `NoOp` and `DeathCause::PriorityBid`; otherwise
  `energy -= paid` and `MeshOutput.priority_bid = paid`. A creature that
  exhausts on compute or the ramp pays no bid. A zero bid never exhausts.
- If energy is exhausted mid-node, evaluation halts and returns `NoOp`.
  What a failed visit leaves, per substrate:

| Substrate | On exhaustion during the visit |
| --- | --- |
| Learned (pure Hebbian) weights | Applied and kept |
| Operator state and outputs | Not committed; the last committed values stand |
| Eligibility traces | Unchanged (updated only after commit) |
| Graph effects: memory writes, actions, gates, params | Not applied |
| A VM dispatch's memory copy | Not committed |
| The bus (`upstream_slots`) | Unchanged |
| Queue and bid | Queue discarded, bid unpaid |

  Actual charges and entered work remain recorded. The mesh returns
  `WorldAction::NoOp` immediately.

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
- Graph index order and committed temporal read bases.
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
