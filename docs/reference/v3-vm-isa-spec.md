# Petri V3 VM Instruction Set Reference

Reference specification for the v3 VM instruction set used by mesh VM nodes.

Status: Active

---

## 1. Registers and Values

- Register values are IEEE-754 `f32`.
- Boolean truthiness is `value >= 0.5`.
- Register and constant index violations are hard faults (panic).

---

## 2. Instruction Set

The VM defines **39 opcodes**.

### Arithmetic and Data Movement

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 0 | `Noop` | none | No operation. |
| 1 | `LoadConst` | dst, const_idx | `dst = constants[const_idx]` |
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
| 21 | `JumpIfZero` | cond, offset | if `!truthy(cond)` then jump |
| 22 | `Jump` | offset | unconditional jump |

### Input and Sensor Reads

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 23 | `ReadInput` | dst, input_idx | reads `NodeGenome.input_refs[input_idx]`; invalid index yields `0.0` |
| 24 | `ReadSensorCell` | dst, sensor_idx, field_idx | reads sensor-cell data |
| 25 | `ReadSensorCreature` | dst, sensor_idx, field_idx | reads sensor-creature data |
| 26 | `ReadSensorSummary` | dst, summary_idx | reads sensor-summary data |
| 27 | `ReadNeighborCell` | dst, neighbor_idx, field_idx | reads neighbor-cell data |
| 28 | `ReadNeighborCreature` | dst, neighbor_idx, field_idx | reads neighbor-creature data |

### Output and Routing Writes

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 29 | `WriteInternalPayload` | slot_idx, src | write internal payload slot |
| 30 | `WriteWorldActionMeta` | slot_idx, src | write world-action metadata slot |
| 31 | `EmitInternal` | action_type | emit internal action payload |
| 32 | `EmitWorldAction` | action_type | emit world action and halt |
| 33 | `WriteRouteTarget` | src_reg | write candidate route target value (`f32`) |

### Halt and Memory

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 34 | `Halt` | none | stop VM execution |
| 35 | `LoadMem8` | dst, addr_reg | read byte at wrapped address |
| 36 | `StoreMem8` | addr_reg, src | write byte at wrapped address |
| 37 | `LoadMem8Imm` | dst, imm_addr | read byte at wrapped immediate address |
| 38 | `StoreMem8Imm` | imm_addr, src | write byte at wrapped immediate address |

---

## 3. VM Execution Rules

- PC starts at `0`; normal step increments by `+1`.
- Jumps apply signed offsets; negative PC is a hard fault.
- Per-opcode energy metering applies; exhausted energy halts node execution.
- `EmitWorldAction` halts VM immediately.
- `Halt` halts VM without emitting a world action.

### `ReadInput` and Upstream Slot Resolution

`ReadInput(dst, input_idx)` resolves through `NodeGenome.input_refs`.

If the referenced variant is `InputReference::UpstreamOutput { slot }`:
- value is `upstream_slots[slot]` when `slot < 12`
- otherwise `0.0`

Invalid `input_idx` is a soft default and yields `0.0`.

### Routing Write Semantics

`WriteRouteTarget(src_reg)` sets VM node's `route_target_idx` output.
- Multiple writes in one VM run use last-write-wins.
- If never written, default `route_target_idx` is `0.0`.
- Mesh executor applies routing conversion rules from
  `v3-mesh-execution-spec.md`.

---

## 4. Fault Semantics

Hard faults (panic):
- invalid register index
- invalid constant index
- invalid payload/meta slot index
- invalid neighbor/sensor field index

Soft defaults:
- invalid `ReadInput` index -> `0.0`
- `InputReference::UpstreamOutput` slot out of range -> `0.0`

Memory addressing is never invalid; all addresses wrap with `rem_euclid(1024)`.

---

## 5. Numeric Determinism

All register writes pass through `sanitize_f32`:
- `NaN -> 0.0`
- `+/-inf -> +/-1_000_000_000.0`
- finite values clamped to `[-1e9, 1e9]`

Additional deterministic rules:
- float->int conversions use ties-away-from-zero
- `CmpEq` epsilon clamped to `[1e-6, 1.0]`
- division by zero returns `0.0`

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
| ReadSensorCell | 0.16 |
| ReadSensorCreature | 0.20 |
| ReadSensorSummary | 0.14 |
| ReadNeighborCell | 0.14 |
| ReadNeighborCreature | 0.18 |
| WriteInternalPayload | 0.14 |
| WriteWorldActionMeta | 0.14 |
| EmitInternal | 0.20 |
| EmitWorldAction | 0.24 |
| WriteRouteTarget | 0.10 |
| Halt | 0.05 |
| LoadMem8 | 0.16 |
| StoreMem8 | 0.18 |
| LoadMem8Imm | 0.14 |
| StoreMem8Imm | 0.16 |

v3 energy is `u32`; costs are scaled to integer units by global multiplier.

---

## 7. Output Lifecycle

VM node evaluation maintains:
- internal payload buffer
- world action metadata buffer
- route target register

All writes are last-write-wins per slot/register.

At node end:
- if world action emitted: action returned; routing ignored
- else route target is returned in `NodeResult.route_target_idx`
- payload/meta buffers are discarded after dispatch

---

## 8. Creature Memory Contract

- Memory size: 1024 bytes per creature.
- Persists across ticks for same creature.
- Copied byte-for-byte on reproduction.
- Addressing wraps with `rem_euclid(1024)`.
