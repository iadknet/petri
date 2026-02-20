# Petri V3 VM Instruction Set Reference

Reference specification for the v3 VM instruction set used by mesh VM nodes.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-sensor-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`

---

## 1. Registers and Values

- Register values are IEEE-754 `f32`.
- Boolean truthiness is `value >= 0.5`.
- VM operand handling is mutation-safe: genome-derived operand values must not
  crash VM execution.

---

## 2. Instruction Set

The VM defines **33 opcodes**.

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
| 23 | `ReadInput` | dst, input_idx | reads `NodeGenome.input_refs[input_idx]`; invalid index yields `0.0` |

### Output and Routing Writes

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 24 | `WriteInternalPayload` | slot_idx, src | overwrites payload slot value (payload buffer starts from incoming `upstream_slots`; invalid slot write ignored) |
| 25 | `WriteWorldActionMeta` | slot_idx, src | writes world-action metadata slot (`slot_idx` in `0..7`; invalid slot write ignored) |
| 26 | `EmitWorldAction` | action_type | emit world action and halt |
| 27 | `WriteRouteTarget` | src_reg | write candidate route target value (`f32`) |

### Halt and Memory

| # | Opcode | Operands | Semantics |
|---|---|---|---|
| 28 | `Halt` | none | stop VM execution |
| 29 | `LoadMem8` | dst, addr_reg | read byte at wrapped address |
| 30 | `StoreMem8` | addr_reg, src | write byte at wrapped address |
| 31 | `LoadMem8Imm` | dst, imm_addr | read byte at wrapped immediate address |
| 32 | `StoreMem8Imm` | imm_addr, src | write byte at wrapped immediate address |

Removed from active V3 mesh ISA:
- `ReadSensorCell`, `ReadSensorCreature`, `ReadSensorSummary`
- `ReadNeighborCell`, `ReadNeighborCreature`
- `EmitInternal`

These were replaced by unified `ReadInput` + `InputReference` dataflow and
`output_slots` routing semantics.

---

## 3. VM Execution Rules

- PC starts at `0`; normal step increments by `+1`.
- Per-opcode energy metering applies; exhausted energy halts node execution.
- `EmitWorldAction` halts VM immediately.
- `Halt` halts VM without emitting a world action.
- VM runtime enforces a configurable step cap `max_vm_steps` per node
  evaluation (default `1024`, sourced from runtime config).
- Production behavior is not required to be deterministic across runs. For
  deterministic tests, pin RNG seed and execution order in the harness.

### VM step-cap safety

- `max_vm_steps` is configuration-controlled for tuning and experiments.
- Value must be `>= 1`.
- Invalid values (for example `0`) fall back to default (`1024`).

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

VM execution loop must be crash-proof for evolved genomes.

Soft defaults / graceful behavior:
- invalid register/constant/index operands use normalization rules
- invalid payload/meta writes are ignored
- out-of-range jump targets wrap into valid program range (when program non-empty)
- invalid `ReadInput` index yields `0.0`

Implementation bugs outside mutation-space (for example corrupted in-memory
instruction representation) are still defects, but evolved operands do not
panic the VM.

Memory addressing is never invalid; all addresses wrap with `rem_euclid(1024)`.

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

`EmitWorldAction(action_type)` decodes using the canonical mapping below:

| `action_type` | Decoded `WorldAction` | Metadata usage |
|---|---|---|
| `0` | `NoOp` | none |
| `1` | `Eat` | none |
| `2` | `Move` | `meta[0]` = direction index |
| `3` | `Reproduce` | `meta[0]` = direction index, `meta[1]` = energy amount |
| other | `NoOp` | none |

Metadata decode rules:
- direction index uses `meta[0].round().clamp(0.0, 7.0)` and maps to
  `Direction::ALL` (`0=N,1=NE,2=E,3=SE,4=S,5=SW,6=W,7=NW`)
- reproduce energy amount uses `meta[1].max(0.0).round() as u32`
- unspecified metadata slots are reserved and ignored by current runtime action
  decoding
- `WriteWorldActionMeta` to `slot_idx >= 8` is ignored

At node end:
- if world action emitted: action returned; routing ignored
- otherwise internal payload buffer is emitted as `NodeResult.output_slots`
- route target is returned in `NodeResult.route_target_idx`
- payload/meta buffers are discarded after node dispatch

This makes `WriteInternalPayload` the VM path for producing routed output slots.

---

## 8. Creature Memory Contract

- Memory size: 1024 bytes per creature.
- Persists across ticks for same creature.
- Copied byte-for-byte on reproduction.
- Addressing wraps with `rem_euclid(1024)`.
