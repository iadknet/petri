# Petri V3 VM Instruction Set Reference

Reference specification for the v3 VM instruction set used by mesh VM nodes.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-runtime-config-spec.md`

VM structural-mutation semantics and the mesh-wide node-type evolvability
contract are defined in `v3-mutation-spec.md`.

---

## 1. Registers and Values

- Register values are IEEE-754 `f32`.
- Boolean truthiness is `value >= 0.5`.
- VM operand handling is mutation-safe: genome-derived operand values must not
  crash VM execution.

---

## 2. Instruction Set

The VM defines **39 opcodes** (T19.F04). The `#` column is the index of the
fresh-instruction draw (`mutation/vm/operators.rs`), which covers all 39;
`AddVote` draws its sink uniformly over the 27-sink catalog.

### Arithmetic and Data Movement

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 0 | `Noop` | none | No operation. |
| 1 | `LoadConst` | dst, const_idx | `dst = constants[const_idx]` (constant index normalization applies). |
| 2 | `Move` | dst, src | `dst = src` |
| 3 | `Add` | dst, a, b | `dst = a + b` |
| 4 | `Sub` | dst, a, b | `dst = a - b` |
| 5 | `Mul` | dst, a, b | `dst = a * b` |
| 6 | `Div` | dst, a, b | `dst = if b == 0 { 0.0 } else { a / b }` |
| 7 | `Min` | dst, a, b | `dst = min(a, b)` |
| 8 | `Max` | dst, a, b | `dst = max(a, b)` |
| 9 | `Abs` | dst, src | `dst = abs(src)` |
| 10 | `Neg` | dst, src | `dst = -src` |
| 11 | `Clamp01` | dst, src | `dst = clamp(src, 0.0, 1.0)` |

### Comparison and Logic

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 12 | `CmpGt` | dst, a, b | `dst = if a > b { 1.0 } else { 0.0 }` |
| 13 | `CmpLt` | dst, a, b | `dst = if a < b { 1.0 } else { 0.0 }` |
| 14 | `CmpEq` | dst, a, b, eps | `dst = if abs(a-b) <= clamp_eps(eps) { 1.0 } else { 0.0 }` |
| 15 | `And` | dst, a, b | boolean `and` using truthiness |
| 16 | `Or` | dst, a, b | boolean `or` using truthiness |
| 17 | `Not` | dst, src | boolean `not` using truthiness |

### Type Conversion

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 18 | `ToI32` | dst, src | round ties-away-from-zero, store as f32 |
| 19 | `ToU8` | dst, src | clamp `[0,255]`, round ties-away-from-zero, store as f32 |
| 20 | `ToBool` | dst, src | `dst = if truthy(src) {1.0} else {0.0}` |

### Control Flow

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 21 | `JumpIfZero` | cond, offset | if `!truthy(cond)` then jump (signed target wrapping applies) |
| 22 | `Jump` | offset | unconditional jump (signed target wrapping applies) |

### Input Reads

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 23 | `ReadInput` | dst, ref_idx, sub_idx | reads `input_refs[ref_idx]` with sub-value index `sub_idx`; invalid index yields `0.0` |

### Output and Routing Writes

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 24 | `WriteInternalPayload` | slot_idx, src | overwrites payload slot value (payload buffer starts from incoming `upstream_slots`; invalid slot write ignored) |
| 25 | `WriteActionParam` | slot_idx, src | `action_params[slot_idx / 2][slot_idx % 2] = regs[src]` on the tick's parameter surface (`slot_idx` in `0..8`, kinds in `VoteKind` order `Eat, Move, Reproduce, StealEnergy`; an invalid slot is ignored) |
| 26 | `WriteRouteGate` | slot, src_reg | write one routing gate score (`f32`) |

### Action Queue Reads

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 27 | `ReadActionQueueLength` | dst | `dst = queue.len() as f32` (the actions committed by earlier passes this tick) |
| 28 | `ReadActionQueueType` | index_src, dst | `dst = queue[reg[index_src]].action_type()` (OOB yields `0.0`) |
| 29 | `ReadActionQueueParam` | index_src, param_slot, dst | `dst = queue[reg[index_src]].param(param_slot)` (OOB yields `0.0`) |

### Halt, Shared Memory Slots, and Priority Bid

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 30 | `Halt` | none | stop VM execution; the dispatch commits its outputs and votes |
| 31 | `LoadSlot` | dst, slot_reg | `dst = shared_memory[regs[slot_reg] % 16]` |
| 32 | `StoreSlot` | slot_reg, src | `shared_memory[regs[slot_reg] % 16] = sanitize(regs[src])` |
| 33 | `LoadSlotImm` | dst, slot_idx | `dst = shared_memory[slot_idx % 16]` |
| 34 | `StoreSlotImm` | slot_idx, src | `shared_memory[slot_idx % 16] = sanitize(regs[src])` |
| 35 | `SetPriorityBid` | src | read `regs[src]`, clamp non-negative, record it as the creature's turn-order priority bid (last-write-wins across the tick); the mesh settles the bid once at tick end (Section 9) |
| 36 | `LoadSlotPrev` | dst, slot_idx | `dst = prev_shared_memory[slot_idx % 16]` |
| 37 | `ClearSlot` | slot_idx | `shared_memory[slot_idx % 16] = 0.0` |

### Action Vote

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 38 | `AddVote` | sink, src | `contribution[sink] += regs[src]` on the dispatch-local vote vector (`sink` in `0..27`, the `VoteSink` catalog order; an invalid sink writes nothing and still costs). The dispatch's contribution replaces this node's earlier one in the pass at every exit but energy exhaustion; the pass end reads the summed votes (`v3-mesh-execution-spec.md` Section 2). |

Removed from active V3 mesh ISA:
- `ReadSensorCell`, `ReadSensorCreature`, `ReadSensorSummary`
- `ReadNeighborCreature`
- `EmitInternal`
- `EmitWorldAction` (replaced by `PushAction` + `ExecuteActionQueue`, T09)
- `PushAction`, `PopAction`, `ExecuteActionQueue`, `WriteDirectionBid`
  (replaced by `AddVote` and the pass loop, T19.F04); `WriteWorldActionMeta`
  became `WriteActionParam`

The sensor reads were replaced by unified `ReadInput` + `InputReference`
dataflow and `output_slots` routing semantics; the action opcodes by votes.

---

## 3. VM Execution Rules

- PC starts at `0`; normal step increments by `+1`.
- Per-opcode energy metering applies; exhausted energy halts node execution.
- Activity ramp (T03.F10): within one node dispatch the instructions past a free
  allowance also pay a charge that rises linearly with the step index, so a
  dispatch that runs to `max_vm_steps` costs a lethal share of a creature's
  energy while ordinary programs and short bounded loops stay nearly free. The
  ramp index resets at every node dispatch; the per-pass hop cap and the
  per-tick hop ramp (`v3-mesh-execution-spec.md` Sections 2 and 5) bound the
  chain, revisits included. Formula and constants: Section 6.
- A dispatch accumulates its opcode and ramp charges locally and subtracts the
  sum from the creature's energy exactly once, on whichever exit path ends it.
  Mid-dispatch, the creature's effective energy is its energy minus that
  accumulator: `ReadInput` of `EnergyCurrent` resolves to the effective energy,
  `EnergyConsumedThisTick` includes the accumulator, and exhaustion triggers
  when the effective energy reaches zero or below. `SetPriorityBid` records
  its bid without charging it (Section 9).
- No opcode ends the tick or the pass: `Halt` (or running off the program
  end) ends the dispatch, which commits its payload, route gates, and votes;
  the mesh then routes (`v3-mesh-execution-spec.md` Section 2).
- VM runtime enforces a configurable step cap `max_vm_steps` per node
  evaluation (default `1024`; canonical owner:
  `v3-runtime-config-spec.md`).
- Determinism scope is canonical in `AGENTS.md` and V3 harness reproducibility
  controls are specified in `v3-mesh-execution-spec.md`
  (`Test-Mode Reproducibility Notes`).

### VM step-cap safety

- `max_vm_steps` is configuration-controlled for tuning and experiments.
- Value must be `>= 1`.
- Invalid values (for example `0`) fall back to default (`1024`).
- Canonical owner for VM runtime config defaults/validation:
  `v3-runtime-config-spec.md`.

### Jump target safety

Jump targets are mutation-safe:
- compute provisional target PC using signed offset semantics
- if `program_len == 0`, VM halts immediately
- otherwise set `pc = provisional_pc.rem_euclid(program_len as i64) as usize`

This replaces hard-fault behavior and avoids mutation-induced dead halts from
negative/out-of-range jump targets.

### Operand normalization rules

All genome-derived indexes are handled without panic:

- **Register index**: normalized by modulo `register_count`.
- **Constant index**: if `constants` empty -> `0.0`; else modulo `constants.len()`.
- **Payload slot index**: valid when `< 12`; otherwise write ignored.
- **Action parameter slot index**: valid when `< 8`; otherwise write ignored.
- **Vote sink index**: valid when `< 27`; otherwise the vote is dropped.

If `register_count == 0`, VM halts immediately (no votes).

Mutation width changes preserve effective register identity: they canonicalize
operands under the old width and skip a shrink that would remove a referenced
effective register. The founder has 20 registers; its original program uses r0
through r15, leaving r16 through r19 available to mutations.

### `ReadInput` and Upstream Slot Resolution

`ReadInput { dst, ref_idx, sub_idx }` resolves through `NodeGenome.input_refs`.

- `ref_idx` selects the `InputReference` from `input_refs`.
- `sub_idx` selects a sub-value within compound inputs (e.g. `ActionQueue`).
  For scalar inputs, `sub_idx > 0` returns `0.0`.

If the referenced variant is `InputReference::UpstreamSlot(slot)`:
- value is `upstream_slots[slot]` when `slot < 12`
- otherwise `0.0`

The decision-state references (T19.F05, `v3-sensor-spec.md` Sections 3.3,
3.6, 3.7) resolve per instruction: `ActionVotes`, `PreviousPassVotes`, and
`CommitCounts` read the mesh's vote vectors and bars as they stand at that
`ReadInput`, `HopsThisTick` the tick's hop count; an `AddVote` earlier in the
same dispatch is staged, not committed, so a later `ReadInput` of
`ActionVotes` does not see it. `PreviousOutcome` reads the frozen snapshot.
A decision compound's `sub_idx` at or past its width yields `0.0`.

Invalid `ref_idx` is a soft default and yields `0.0`.

### Routing Write Semantics

`WriteRouteGate(slot, src_reg)` sets one of eight per-slot gate scores.
- Scores reset to zero at every node dispatch; invalid slots are ignored.
- Multiple writes to the same slot use last-write-wins and sanitize values.
- Mesh routing selects the maximum bias-plus-score among all targets, the
  node itself included, retaining the earliest target on ties; see
  `v3-mesh-execution-spec.md`.
- T11.F15 route addition inserts this write before the first `Halt` using
  the existing structural reference repair and a uniformly sampled register.

---

## 4. Fault Semantics

VM execution loop must be crash-proof for evolved genomes.

Cross-runtime fallback outcomes (for example chain-level `WorldAction::NoOp`
resolution) are canonical in `v3-mesh-execution-spec.md` (Section 4,
authoritative soft-default matrix).

Soft defaults / graceful behavior:
- invalid register/constant/index operands use normalization rules
- invalid payload and parameter writes, and invalid vote sinks, are ignored
- out-of-range jump targets wrap into valid program range (when program non-empty)
- invalid `ReadInput` `ref_idx` or `sub_idx` yields `0.0`

Implementation bugs outside mutation-space (for example corrupted in-memory
instruction representation) are still defects, but evolved operands do not
panic the VM.

Slot addressing is never invalid; all slot indices wrap with `% 16`.

---

## 5. Numeric Safety and Repeatability

All register writes pass through `sanitize_f32`:
- `NaN -> 0.0`
- `+/-inf -> +/-1_000_000_000.0`
- finite values clamped to `[-1e9, 1e9]`

Defined numeric rules:
- float->int conversions use ties-away-from-zero
- `CmpEq` epsilon clamped to `[1e-6, 1.0]`
- division by zero returns `0.0`
- jump targets use signed `rem_euclid` wrapping by `program_len`

---

## 6. Opcode Cost Table

| Opcode | Base Cost (f32) |
|---|---|
| Noop | 0.05 |
| LoadConst | 0.08 |
| Move | 0.08 |
| Add | 0.12 |
| Sub | 0.12 |
| Mul | 0.12 |
| Div | 0.16 |
| Min | 0.12 |
| Max | 0.12 |
| Abs | 0.10 |
| Neg | 0.10 |
| Clamp01 | 0.10 |
| CmpGt | 0.12 |
| CmpLt | 0.12 |
| CmpEq | 0.12 |
| And | 0.12 |
| Or | 0.12 |
| Not | 0.10 |
| ToI32 | 0.10 |
| ToU8 | 0.10 |
| ToBool | 0.10 |
| JumpIfZero | 0.14 |
| Jump | 0.10 |
| ReadInput | 0.12 |
| WriteInternalPayload | 0.14 |
| WriteActionParam | 0.14 |
| AddVote | 0.14 |
| WriteRouteGate | 0.10 |
| ReadActionQueueLength | 0.08 |
| ReadActionQueueType | 0.12 |
| ReadActionQueueParam | 0.12 |
| SetPriorityBid | 0.20 |
| Halt | 0.05 |
| LoadSlot | 0.12 |
| StoreSlot | 0.14 |
| LoadSlotImm | 0.10 |
| StoreSlotImm | 0.12 |
| LoadSlotPrev | 0.10 |
| ClearSlot | 0.12 |

v3 energy uses continuous scalar units (`f32`).
The k-th instruction executed within one node dispatch (k from 1) spends:

```text
step_energy_cost = opcode_base_cost * runtime.vm.opcode_cost_multiplier
                 + runtime.vm.step_ramp_cost
                   * max(0, k - runtime.vm.step_ramp_allowance)
```

The first term is the base opcode spend; the second is the T03.F10 activity
ramp. For a dispatch of n steps with `m = max(0, n - step_ramp_allowance)` the
ramp total has the closed form `step_ramp_cost * m * (m + 1) / 2`. At the
defaults (`step_ramp_allowance` 100, `step_ramp_cost` 1e-6) 200 steps cost about
0.005 energy, 1,000 steps about 0.405, and a dispatch that runs to the default
`max_vm_steps` of 10,000 costs about 49.

The dispatch accumulates these charges and settles the sum against the
creature's energy once (Section 3), so charges below the ulp of an `f32` energy
are not lost. `SetPriorityBid` adds nothing beyond its opcode and ramp
charge; the bid is settled by the mesh (Section 9).

Canonical owner for `runtime.vm.opcode_cost_multiplier`,
`runtime.vm.step_ramp_allowance`, and `runtime.vm.step_ramp_cost`:
`v3-runtime-config-spec.md`.

---

## 7. Output Lifecycle

VM node evaluation maintains:
- internal payload buffer (12 slots)
- a dispatch-local vote contribution (27 entries, `VoteSink` order)
- eight route gate scores

and writes the tick's parameter surface (`action_params`, 4 kinds by 2
slots) in place.

Initialization at the start of each VM node evaluation:
- internal payload buffer is copied from incoming `upstream_slots`
- the vote contribution is zeroed
- all eight route gate scores start at `0.0`

The parameter surface is zeroed at tick start only; a dispatch overwrites the
slots it writes and the value stands until the next write or the commit that
reads it.

All writes are last-write-wins per slot/register; votes add within the
dispatch. If `WriteInternalPayload` targets an invalid slot (`>= 12`), the
write is ignored and existing payload slot values are preserved.

### Votes, parameters, and the commit

A node's votes say which action and how many; the pass end commits at most
one (`v3-mesh-execution-spec.md` Section 2): the kind with the highest
effective vote (its best sink's vote minus its bar), in the direction of its
best sink. The committed action reads its kind's parameter row:

| Kind | Sinks (`AddVote` index) | Parameters read at commit |
|---|---|---|
| `Eat` | 0 | `action_params[Eat][0]` = food type index (rounded; non-finite or negative reads 0) |
| `Move` | 1 to 8 (`Direction::ALL` order `N, NE, E, SE, S, SW, W, NW`) | none |
| `Reproduce` | 9 to 16 | `action_params[Reproduce][1]` = offspring transfer fraction, `clamp_unit_interval` |
| `StealEnergy` | 17 to 24 | `action_params[StealEnergy][1]` = amount, `clamp_non_negative_finite` |
| `Terminate` | 25 | never commits; ends a non-empty tick when it holds the best effective vote |
| `Decide` | 26 | never commits; ends the pass early when some kind's effective vote is positive, or the queue is non-empty and `Terminate` is positive |

`WriteActionParam` slot indexes: `0` `Eat[0]`, `1` `Eat[1]`, `2` `Move[0]`,
`3` `Move[1]`, `4` `Reproduce[0]`, `5` `Reproduce[1]`, `6`
`StealEnergy[0]`, `7` `StealEnergy[1]`. Slots no commit reads are reserved.

At node end:
- the internal payload buffer is emitted as `NodeResult.output_slots`
- per-slot scores are returned in `NodeResult.route_gates`
- the vote contribution replaces this node's earlier contribution in the pass
- payload and gate buffers are discarded after node dispatch

This makes `WriteInternalPayload` the VM path for producing routed output slots
and `AddVote` its path to action.

### Removed opcodes

`EmitWorldAction` was replaced by the action queue model (T09), and the queue
opcodes by votes (T19.F04): an action is committed by the pass loop, at most
one per pass, up to `max_actions_per_turn` per tick.

---

## 8. Shared Memory Slot Contract

- Each creature has 16 f32 shared memory slots (`shared_memory: [f32; 16]`).
- Persists across ticks for the same creature.
- At tick start: `prev_shared_memory` is snapshotted from `shared_memory`, then optional decay is applied.
- On reproduction: `shared_memory` is copied from parent to child; `prev_shared_memory` is zeroed.
- Slot addressing wraps with `% 16`.
- All slot writes pass through `sanitize_f32()` (NaN/Infinity → 0.0).
- VM operates on a working copy: committed on normal exit, discarded on energy exhaustion.

---

## 9. Priority Bid Mechanics

`SetPriorityBid { src }` allows creatures to bid energy for earlier execution in
Phase 2 (action resolution). Higher bidders act first, gaining priority access to
contested resources like food.

Semantics (T19.F02, T19.F04):
- Reads `regs[src]`, clamps to non-negative (`max(0.0, value)`; a non-finite
  read is `0.0`) and records it as the creature's bid for this tick.
  The opcode pays only its 0.20 opcode cost and the step's ramp charge.
- Last-write-wins if called multiple times, across nodes, revisits, and
  passes within the tick.
- The mesh settles the recorded bid exactly once, at tick end, on every
  exit: `paid = min(bid, energy)`, nothing when energy is already gone. If
  `bid >= energy` the creature goes all-in: energy is `0.0`, the tick ends
  `EnergyExhausted` with `DeathCause::PriorityBid`, and the actions already
  committed are kept; otherwise `energy -= paid` and the paid amount is the
  turn-order bid. A zero bid pays nothing and never exhausts. The `priority_bid` energy flow and death
  cause keep their keys.

Turn ordering:
- After cognition (Phase 1), decisions are stable-sorted by bid descending.
- Stable sort preserves the pre-existing random shuffle order among creatures
  with equal bids (including the default 0.0).
- Creatures that never call `SetPriorityBid` have bid 0.0 (no cost, no priority).

There is no cap on bid amount beyond the energy the creature actually has left
when the tick's passes end: a bid at or above that energy is an all-in. Natural
selection handles the economics: overbidding wastes energy and leads to
extinction.
