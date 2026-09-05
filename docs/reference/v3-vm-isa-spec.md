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

The VM defines **41 opcodes**.

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
| 25 | `WriteWorldActionMeta` | slot_idx, src | writes world-action metadata slot (`slot_idx` in `0..7`; invalid slot write ignored) |
| 26 | `WriteRouteTarget` | src_reg | write candidate route target value (`f32`) |

### Action Queue

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 27 | `PushAction` | action_type | decode meta buffer and push action onto queue; silent no-op if at cap |
| 28 | `PopAction` | none | remove last action from queue; no-op if empty |
| 29 | `ReadActionQueueLength` | dst | `dst = queue.len() as f32` |
| 30 | `ReadActionQueueType` | index_src, dst | `dst = queue[reg[index_src]].action_type()` (OOB yields `0.0`) |
| 31 | `ReadActionQueueParam` | index_src, param_slot, dst | `dst = queue[reg[index_src]].param(param_slot)` (OOB yields `0.0`) |
| 32 | `SetPriorityBid` | src | read `regs[src]`, clamp non-negative, deduct bid from energy, set creature's turn-order priority bid (last-write-wins) |
| 33 | `ExecuteActionQueue` | none | terminal: return accumulated action queue for execution |

### Halt and Shared Memory Slots

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 34 | `Halt` | none | stop VM execution |
| 35 | `LoadSlot` | dst, slot_reg | `dst = shared_memory[regs[slot_reg] % 16]` |
| 36 | `StoreSlot` | slot_reg, src | `shared_memory[regs[slot_reg] % 16] = sanitize(regs[src])` |
| 37 | `LoadSlotImm` | dst, slot_idx | `dst = shared_memory[slot_idx % 16]` |
| 38 | `StoreSlotImm` | slot_idx, src | `shared_memory[slot_idx % 16] = sanitize(regs[src])` |
| 39 | `LoadSlotPrev` | dst, slot_idx | `dst = prev_shared_memory[slot_idx % 16]` |
| 40 | `ClearSlot` | slot_idx | `shared_memory[slot_idx % 16] = 0.0` |

Removed from active V3 mesh ISA:
- `ReadSensorCell`, `ReadSensorCreature`, `ReadSensorSummary`
- `ReadNeighborCreature`
- `EmitInternal`
- `EmitWorldAction` (replaced by `PushAction` + `ExecuteActionQueue`)

These were replaced by unified `ReadInput` + `InputReference` dataflow and
`output_slots` routing semantics.

---

## 3. VM Execution Rules

- PC starts at `0`; normal step increments by `+1`.
- Per-opcode energy metering applies; exhausted energy halts node execution.
- `ExecuteActionQueue` is terminal: halts VM and returns accumulated actions.
- `Halt` halts VM without returning actions (mesh uses action queue state).
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
- **World-action metadata slot index**: valid when `< 8`; otherwise write ignored.

If `register_count == 0`, VM halts immediately (no action emission).

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

Invalid `ref_idx` is a soft default and yields `0.0`.

### Routing Write Semantics

`WriteRouteTarget(src_reg)` sets VM node's raw route value.
- Multiple writes in one VM run use last-write-wins.
- If never written, default raw route value is `0.0`.
- Mesh executor applies routing conversion rules from
  `v3-mesh-execution-spec.md`.

---

## 4. Fault Semantics

VM execution loop must be crash-proof for evolved genomes.

Cross-runtime fallback outcomes (for example chain-level `WorldAction::NoOp`
resolution) are canonical in `v3-mesh-execution-spec.md` (Section 4,
authoritative soft-default matrix).

Soft defaults / graceful behavior:
- invalid register/constant/index operands use normalization rules
- invalid payload/meta writes are ignored
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
| WriteWorldActionMeta | 0.14 |
| WriteRouteTarget | 0.10 |
| PushAction | 0.24 |
| PopAction | 0.10 |
| ReadActionQueueLength | 0.08 |
| ReadActionQueueType | 0.12 |
| ReadActionQueueParam | 0.12 |
| SetPriorityBid | 0.20 |
| ExecuteActionQueue | 0.24 |
| Halt | 0.05 |
| LoadSlot | 0.12 |
| StoreSlot | 0.14 |
| LoadSlotImm | 0.10 |
| StoreSlotImm | 0.12 |
| LoadSlotPrev | 0.10 |
| ClearSlot | 0.12 |

v3 energy uses continuous scalar units (`f32`).
Opcode spend is:

```text
opcode_energy_cost = opcode_base_cost * runtime.vm.opcode_cost_multiplier
```

Canonical owner for `runtime.vm.opcode_cost_multiplier`:
`v3-runtime-config-spec.md`.

---

## 7. Output Lifecycle

VM node evaluation maintains:
- internal payload buffer (12 slots)
- world action metadata buffer (8 slots)
- route target register

Initialization at the start of each VM node evaluation:
- internal payload buffer is copied from incoming `upstream_slots`
- world action metadata buffer is zeroed
- route target register starts at `0.0`

All writes are last-write-wins per slot/register.
If `WriteInternalPayload` targets an invalid slot (`>= 12`), the write is
ignored and existing payload slot values are preserved.

World-action metadata buffer size is fixed:
- `WORLD_ACTION_META_SLOTS = 8`
- non-configurable (to keep VM behavior consistent across configs)

### Action encoding and metadata mapping

`PushAction(action_type)` decodes using the metadata buffer and the canonical
mapping below:

| `action_type` | Decoded `WorldAction` | Metadata usage |
|---|---|---|
| `0` | `NoOp` | none |
| `1` | `Eat` | `meta[0]` = food type index |
| `2` | `Move` | `meta[0]` = direction index |
| `3` | `Reproduce` | `meta[0]` = direction index, `meta[1]` = offspring transfer energy (scalar `f32`) |
| `4` | `StealEnergy` | `meta[0]` = direction index, `meta[1]` = steal amount |
| other | `NoOp` | none |

Each `PushAction` decodes from the *current* metadata buffer state and appends
to the action queue. The metadata buffer can be overwritten between pushes to
encode different actions.

Metadata decode rules:
- direction index uses `meta[0].round().clamp(0.0, 7.0)` and maps to
  `Direction::ALL` (`0=N,1=NE,2=E,3=SE,4=S,5=SW,6=W,7=NW`)
- reproduce energy amount uses non-negative scalar
  `clamp_non_negative_finite(meta[1])` (no integer rounding)
- unspecified metadata slots are reserved and ignored by current runtime action
  decoding
- `WriteWorldActionMeta` to `slot_idx >= 8` is ignored

At node end:
- if `ExecuteActionQueue` was called: `NodeResult.terminal` is true, mesh
  returns accumulated action queue
- otherwise internal payload buffer is emitted as `NodeResult.output_slots`
- route value is returned in `NodeResult.route` as
  `RouteDecision::VmWrap { raw_value }`
- payload/meta buffers are discarded after node dispatch

This makes `WriteInternalPayload` the VM path for producing routed output slots.

### Removed opcodes

`EmitWorldAction` was replaced by the action queue model
(`PushAction` + `ExecuteActionQueue`). The action queue allows multiple actions
per VM evaluation, with a configurable cap (`max_actions_per_turn`).

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

Semantics:
- Reads `regs[src]`, clamps to non-negative (`max(0.0, value)`).
- Deducts bid amount from creature energy (on top of the 0.20 opcode cost).
- If energy drops to zero or below after deduction, the creature is exhausted
  (`NodeResult::exhausted()`).
- Sets the creature's priority bid for this tick. Last-write-wins if called
  multiple times (consistent with `WriteRouteTarget`).

Turn ordering:
- After cognition (Phase 1), decisions are stable-sorted by bid descending.
- Stable sort preserves the pre-existing random shuffle order among creatures
  with equal bids (including the default 0.0).
- Creatures that never call `SetPriorityBid` have bid 0.0 (no cost, no priority).

There is no cap on bid amount — creatures can bid up to their remaining energy.
Natural selection handles the economics: overbidding wastes energy and leads to
extinction.
