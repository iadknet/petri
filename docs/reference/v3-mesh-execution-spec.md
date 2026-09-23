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
- `runtime/` owns node evaluation, routing, the pass loop, action selection,
  and soft-default behavior.
- `runtime` returns `MeshOutput` (`actions`, cost report, `priority_bid`,
  work counters, `termination_reason`, and the final per-kind bars
  `commit_counts`) to tick orchestration.
- `tick/orchestrator` owns turn ordering and immediate action application.

---

## 2. Pass Loop and Action Selection

A tick is a sequence of passes (T19.F04). Each pass runs the routing chain
from `entry_node_id`; nodes vote into the 27-sink vote vector `V` (`Eat`, 8
`Move`, 8 `Reproduce`, 8 `StealEnergy`, `Terminate`, `Decide`) and write the
parameter surface `action_params[kind][0..2]`; at the pass end at most one
action commits. A node may be dispatched any number of times within a pass
(T19.F02): cycles and self-targets are legal, every dispatch runs on the
creature's live state, and the per-pass hop cap is the only structural bound
on a pass. `C[K]` is the bar of kind `K` (its commits so far this tick),
`unit` is 1.0, and `E[K]` is kind `K`'s effective vote.

```text
tick start: V = 0, V_prev = 0, C = 0, queue empty, bus zeroed, parameter
            surface zeroed, per-node contributions empty; internal state
            untouched
pass:
  hops_this_pass = 0; V_prev = V; V = 0; per-node contributions cleared
  run the chain from the entry (bus = the last dispatched node's output slots
  of the previous pass; state, memory, weights, traces live)
    after each committed dispatch: that node's contribution replaces its
    earlier one this pass; V = sanitized sum over nodes visited this pass
    if V[Decide] > 0 and (max_K E[K] > 0 or (queue non-empty and V[Terminate] > 0)):
      end the pass (Decided)
  the pass also ends at NoTargets, MissingNode, PassCapReached (hops_this_pass
  reaches max_mesh_hops), or EnergyExhausted; all keep V and the queue
  pass end:
    for K in {Eat, Move, Reproduce, StealEnergy}:
      best[K] = lowest-index argmax over K's sinks of V
      E[K] = V[best[K]] - C[K] * unit
    K* = argmax E  (ties: the kind committed in the previous pass, then lowest index)
    if EnergyExhausted: end the tick (EnergyExhausted), queue kept
    if E[K*] <= 0: end the tick (NoDecision)
    if queue non-empty and V[Terminate] >= E[K*]: end the tick (TerminateVoted)
    commit best[K*] decoded with the parameter surface of K*; C[K*] += 1
    if queue full: end the tick (ActionCapReached)
    next pass
exit: settle the bid once; actions = queue, or NoOp when empty
```

Commit decode (`runtime/action_decode.rs`): the direction is the winning
sink's; `Eat` reads its food type from `action_params[Eat][0]` (rounded,
non-finite or negative reads type 0), `Reproduce` its transfer fraction from
`action_params[Reproduce][1]` (clamped to `[0, 1]`), and `StealEnergy` its
amount from `action_params[StealEnergy][1]` (non-negative finite, else 0).
The parameter surface is zeroed at tick start, overwritten per visit, and
read at commit.

Guards and ties:
- `Decide` ends a pass only when the pass end would act: some kind's
  effective vote is positive, or the queue is non-empty and `Terminate` is
  positive (W17b), so a stray `Decide` is harmless (W18) and a cycle leaves by
  one `Terminate` edge instead of a capped pass. With every `E <= 0` the pass
  end is `NoDecision` whatever `Terminate` holds (W17, W17b).
- `Terminate` never commits, has no bar, wins its ties, and cannot end an
  empty tick (W15).
- A revisited node replaces its own contribution, so a cycle re-judges
  rather than inflates (W11 to W14, W17).
- Comparisons are on sanitized `f32`; no RNG is drawn.

What the rule gives (the worked cases in
`crates/v3-core/src/runtime/pass_loop_tests.rs` are the fixtures):
- A kind's vote `v` commits `ceil(v)` actions of that kind, each in the
  direction that wins its pass (W7): a vote is how many, the argmax is
  which.
- One edge into an unused kind adds exactly one action (W4); into a used
  kind's other sink it steers (W5); into a sink already voted on it raises
  the count and the rank (W6).
- A plan is a construction: nodes that read the queue or a clock change the
  winning direction or vote `Terminate` from pass to pass (W8 to W10, W19).
- Every kind inhibited is `NoOp` without a stall (W16).

Decision-state reads (T19.F05, `v3-sensor-spec.md` Section 3.6): a node reads
`V` (`ActionVotes`), `V_prev` (`PreviousPassVotes`), `C` (`CommitCounts`),
and the tick's hop count, the dispatch in flight included (`HopsThisTick`),
live at every read. A dispatch sees `V` as committed so far this pass, never
its own staged contribution; nothing commits mid-dispatch, so a graph
dispatch's evaluation, post-plasticity, and effects contexts read the same
values. The worked cases V1 to V6 are in
`crates/v3-core/src/runtime/decision_input_tests.rs` and
`crates/v3-core/src/simulation/tick/tests/previous_outcome.rs`.

Notes:
- The bus is zeroed at tick start only and carried between passes: pass
  `n + 1` starts from the output slots of pass `n`'s last dispatched node.
- A dispatch is one hop; a node dispatched again reads its own committed
  state and outputs from its previous dispatch (graph backends) and the
  shared memory its previous dispatch committed (both backends).
- For every node evaluation, `output_slots` starts as a copy of incoming
  `upstream_slots`; backend slot writes overwrite addressed slots only.
- Slots not written during a node evaluation pass through unchanged.
- Route ties keep the earliest vector position. A missing winner ends the
  pass softly without falling through.
- `max_mesh_hops` bounds hops per pass and resets per pass; reaching it ends
  the pass `PassCapReached`, keeps the votes and the queue, and increments
  `WorkCounters.pass_cap_hits`. The hop counter of the ramp (Section 5)
  never resets within the tick, so cycling every pass pays the ramp across
  passes (640 hops cost 18.5 energy at the defaults).
- Passes per tick are at most `max_actions_per_turn` (each pass either
  commits or ends the tick). `WorkCounters.passes` counts passes and
  `WorkCounters.decided_passes` the passes that ended `Decided`.
- **Copy interference** (T11.F08). A node copy (`CopyNode`, or a mesh slice
  copy) is a faithful clone: it keeps its original's input references, output
  slots, shared-memory addresses, and vote sinks, so activating it in the
  original's chain position reproduces the original's behavior. Those
  addresses are not remapped. When a copy and its original both run in one
  chain they write the same output slots and the same shared-memory slots,
  and the node evaluated later in the chain wins; their votes add, since
  contributions are per node. A lineage that wants two independent modules
  must move one copy's addresses by ordinary mutation.
- Canonical owner for `runtime.max_mesh_hops` and
  `runtime.max_actions_per_turn` defaults/validation:
  `v3-runtime-config-spec.md`.

---

## 3. Termination Contract

Each pass ends with a `PassEndReason`:

| Pass end | Condition |
| --- | --- |
| `Decided` | After a committed dispatch, `V[Decide] > 0` and either some `E[K] > 0` or a non-empty queue with `V[Terminate] > 0` |
| `PassCapReached` | `hops_this_pass` reaches `max_mesh_hops` before a dispatch |
| `NoTargets` | The dispatched node has no route target |
| `MissingNode` | The routed (or entry) node id is missing from the node set |
| `EnergyExhausted` | Energy reaches zero on the hop ramp or during a node evaluation |

The tick ends with a `TerminationReason`:

| Tick end | Condition |
| --- | --- |
| `NoDecision` | At a pass end the best effective vote is `<= 0` |
| `TerminateVoted` | The queue is non-empty and `V[Terminate] >= E[K*]` |
| `ActionCapReached` | A commit fills the queue to `max_actions_per_turn` |
| `EnergyExhausted` | A pass ended `EnergyExhausted`, or the bid settlement is an all-in |

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
- A node dispatched again within the tick, in the same pass or a later one,
  starts from its previous dispatch's commit; skipped modules hold state.
- Canonical semantics: `v3-graph-backend-spec.md`; config disposition:
  `v3-runtime-config-spec.md`.

---

## 4. Authoritative Soft-Default Matrix (Junk DNA Safe)

Runtime must never panic on malformed evolved topologies.

This section is the authoritative soft-default matrix for V3 chain-level runtime
behavior.

| Condition | Runtime behavior |
|---|---|
| `entry_node_id` missing from node set | Pass ends `MissingNode` with no votes; the tick ends `NoDecision` with the queue kept |
| Routed target id missing | Pass ends `MissingNode`; votes and queue kept |
| No target | Pass ends `NoTargets`; votes and queue kept |
| Invalid gate slot | Runtime score is zero |
| All effective scores are NaN or negative infinity | Earliest target wins |
| `ReadInput` `ref_idx` out of range | Yield `0.0` |
| `ReadInput` `sub_idx` out of range (compound) | Yield `0.0` |
| Scalar input with `sub_idx > 0` | `sub_idx` is ignored; the scalar value is read |
| `UpstreamSlot` slot out of range | Yield `0.0` |
| Decision-state compound `sub_idx` at or past its width | Yield `0.0` (no wrap) |
| Node backend does not write an output slot | Preserve incoming `upstream_slots[slot]` |
| Graph edge source out of bounds | Input contributes `0.0` |
| Non-finite vote or parameter | Sanitized before summing; no NaN reaches a comparison |
| Per-pass hop cap reached | Pass ends `PassCapReached`; votes and queue kept; count `pass_cap_hits` |
| Every kind's effective vote `<= 0` | Tick ends `NoDecision`; the queue, or `NoOp` when empty |
| Graph state for `NodeId` missing | Allocate zero-initialized state and continue |

This policy intentionally allows junk DNA. Invalid offspring are culled by
selection pressure rather than strict genome repair.

---

## 5. Energy Metering

- VM nodes: energy deducted per opcode from VM cost table.
- Graph nodes: energy deducted per internal-node-per-visit evaluation.
- Per-tick hop ramp (T19.F01): the k-th mesh hop of a world tick (k from 1,
  counted across every pass) is charged
  `runtime.hop_ramp_cost * max(0, k - runtime.hop_ramp_allowance)` before the
  node is dispatched. Over n hops in a tick the ramp totals
  `hop_ramp_cost * m * (m + 1) / 2` with `m = max(0, n - hop_ramp_allowance)`.
  The hop index never resets within a tick and starts at 1 at every tick, so
  sustained neural activity within one tick costs metabolism. Defaults:
  allowance `32`, cost `1e-4`. The charge is a direct debit against the
  creature's energy and is attributed as the `mesh_ramp` energy flow and death
  cause. If it takes energy to `<= 0.0` the node is not dispatched and the
  pass ends `EnergyExhausted`; the hop is still counted and recorded as a
  dispatch.
- Priority bid (T19.F02, T19.F04): `SetPriorityBid` records
  `max(0, regs[src])` (non-finite reads as 0) as the creature's bid, last
  write wins across the whole tick including revisits and later passes, and
  pays only its opcode cost. The bid is settled exactly once, on every exit:
  `paid = min(bid, energy)`, and nothing is paid when energy is already
  gone. If `bid >= energy` the creature goes all-in: energy is `0.0`, the
  tick ends `EnergyExhausted` with `DeathCause::PriorityBid`, and the queue
  is kept with `MeshOutput.priority_bid = paid`; otherwise `energy -= paid`
  and `MeshOutput.priority_bid = paid`. A zero bid never exhausts.
- Exhaustion at any point (compute, ramp, bid) keeps the actions committed
  before it; the exhausted pass commits nothing. What a failed visit leaves,
  per substrate:

| Substrate | On exhaustion during the visit |
| --- | --- |
| Learned (pure Hebbian) weights | Applied and kept |
| Operator state and outputs | Not committed; the last committed values stand |
| Eligibility traces | Unchanged (updated only after commit) |
| Graph effects: memory writes, votes, parameters, gates | Not applied |
| A VM dispatch's memory copy and votes | Not committed (its parameter writes are never read: the exhausted pass commits nothing) |
| The bus (`upstream_slots`) | Unchanged |
| Committed queue and bid | Queue kept; bid settled once (nothing when energy is gone) |

  Actual charges and entered work remain recorded.

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
of the source backend; both preserve the bus and vote nothing within budget.
An unwired Graph detour stays free; once mutation wires one of its effect
surfaces the detour is entered, pays one node equivalent and applies that
effect. See
`v3-mutation-spec.md`. Extra dispatches/instructions retain their normal
work and energy accounting. Neutrality requires sufficient budgets and does
not promise equal downstream live-energy introspection or exhaustion outcomes.
Production, full trace and compact observations share the same dispatch loop.
