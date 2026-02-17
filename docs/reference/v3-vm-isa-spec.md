# Petri V3 VM Instruction Set Reference

> Reference specification for the v3 VM instruction set. Adapted from v2 design.
> This is a reference document, not an implementation plan.

---

## 1. Registers and Values

The VM uses **f32 register values**. Every register holds a single IEEE-754 32-bit float.

**Boolean semantics.** There is no dedicated boolean type. Truthiness is defined
by the threshold `value >= 0.5` (true) and `value < 0.5` (false). Instructions
that produce boolean-like results write `1.0` (true) or `0.0` (false).

**Invalid register handling.** Any instruction that references a register index
outside the valid register file **panics immediately**. In v3's
panic-for-invariants philosophy, an out-of-bounds register access is a program
invariant violation, not a recoverable error. There are no `Result` types for
internal VM faults.

---

## 2. Instruction Set

The VM defines **38 opcodes**, listed below with brief semantics.

### Arithmetic and Data Movement

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 0 | **Noop** | (none) | No operation. |
| 1 | **LoadConst** | dst, const_idx | Load constant at `const_idx` into register `dst`. |
| 2 | **Move** | dst, src | Copy register `src` into register `dst`. |
| 3 | **Add** | dst, a, b | `dst = a + b` |
| 4 | **Sub** | dst, a, b | `dst = a - b` |
| 5 | **Mul** | dst, a, b | `dst = a * b` |
| 6 | **Div** | dst, a, b | `dst = a / b` (division by zero writes `0.0`) |
| 7 | **Min** | dst, a, b | `dst = min(a, b)` |
| 8 | **Max** | dst, a, b | `dst = max(a, b)` |
| 9 | **Abs** | dst, src | `dst = abs(src)` |
| 10 | **Neg** | dst, src | `dst = -src` |
| 11 | **Clamp01** | dst, src | `dst = clamp(src, 0.0, 1.0)` |

### Comparison and Logic

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 12 | **CmpGt** | dst, a, b | `dst = if a > b { 1.0 } else { 0.0 }` |
| 13 | **CmpLt** | dst, a, b | `dst = if a < b { 1.0 } else { 0.0 }` |
| 14 | **CmpEq** | dst, a, b, eps | `dst = if abs(a - b) <= eps { 1.0 } else { 0.0 }` (epsilon clamped; see section 5) |
| 15 | **And** | dst, a, b | `dst = if truthy(a) && truthy(b) { 1.0 } else { 0.0 }` |
| 16 | **Or** | dst, a, b | `dst = if truthy(a) \|\| truthy(b) { 1.0 } else { 0.0 }` |
| 17 | **Not** | dst, src | `dst = if truthy(src) { 0.0 } else { 1.0 }` |

### Type Conversion

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 18 | **ToI32** | dst, src | Round `src` to nearest i32 (ties-away-from-zero), store as f32. |
| 19 | **ToU8** | dst, src | Clamp `src` to [0.0, 255.0], round to nearest integer, store as f32. |
| 20 | **ToBool** | dst, src | `dst = if truthy(src) { 1.0 } else { 0.0 }` |

### Control Flow

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 21 | **JumpIfZero** | cond, offset | If `!truthy(cond)`, set `PC += offset`. |
| 22 | **Jump** | offset | Unconditional `PC += offset`. |

### Input / Sensor / Neighbor Reads

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 23 | **ReadInput** | dst, input_idx | Read input slot `input_idx` into `dst`. Invalid index yields `0.0` (soft default). |
| 24 | **ReadSensorCell** | dst, sensor_idx, field_idx | Read a cell-sensor field into `dst`. |
| 25 | **ReadSensorCreature** | dst, sensor_idx, field_idx | Read a creature-sensor field into `dst`. |
| 26 | **ReadSensorSummary** | dst, summary_idx | Read a summary-sensor field into `dst`. |
| 27 | **ReadNeighborCell** | dst, neighbor_idx, field_idx | Read a neighbor cell field into `dst`. |
| 28 | **ReadNeighborCreature** | dst, neighbor_idx, field_idx | Read a neighbor creature field into `dst`. |

### Output / Action Writes

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 29 | **WriteInternalPayload** | slot_idx, src | Write register `src` to internal payload buffer at `slot_idx`. |
| 30 | **WriteWorldActionMeta** | slot_idx, src | Write register `src` to world-action meta buffer at `slot_idx`. |
| 31 | **EmitInternal** | action_type | Emit the current internal payload buffer as an internal action of the given type. |
| 32 | **EmitWorldAction** | action_type | Emit the current world-action meta buffer as a world action of the given type. **Halts execution** (first world action terminates the program). |

### Halt

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 33 | **Halt** | (none) | Immediately stop execution. |

### Memory Access

| # | Opcode | Operands | Semantics |
|---|--------|----------|-----------|
| 34 | **LoadMem8** | dst, addr_reg | Load byte from memory at address in `addr_reg` into `dst` as f32 `[0.0, 255.0]`. |
| 35 | **StoreMem8** | addr_reg, src | Store register `src` (clamped to `[0.0, 255.0]`, rounded to nearest integer) as a u8 at the memory address in `addr_reg`. |
| 36 | **LoadMem8Imm** | dst, imm_addr | Load byte from memory at immediate address `imm_addr` into `dst` as f32 `[0.0, 255.0]`. |
| 37 | **StoreMem8Imm** | imm_addr, src | Store register `src` (clamped to `[0.0, 255.0]`, rounded to nearest integer) as a u8 at immediate address `imm_addr`. |

---

## 3. VM Execution Rules

**Program counter.** The PC starts at `0`. After each instruction executes, PC
advances by 1 (unless modified by a jump). If PC reaches or exceeds the
instruction count, execution halts normally.

**Jump semantics.** `Jump` and `JumpIfZero` apply a signed offset to PC. The
offset is added to PC *after* the default +1 increment. Jumping past the end of
the program halts execution. Jumping to a negative PC panics (invariant
violation).

**Energy metering.** Each opcode deducts a base cost from the creature's
remaining energy budget for that tick. If the creature's energy is exhausted,
execution halts. In v3, energy is stored as `u32`; the f32 base costs from the
opcode cost table (section 6) will be scaled to integer energy units via a
global multiplier. Per-opcode metering is the canonical model for VM nodes;
graph nodes use a per-node complexity cost instead (see graph operator spec).

**Halt on first world action.** When `EmitWorldAction` executes, the action is
recorded and execution halts immediately. A creature may emit at most one world
action per tick.

**Memory wrapping.** All memory addresses are resolved modulo 1024:
`resolved_addr = raw_addr.rem_euclid(1024)`. Memory addresses are therefore
never invalid; any integer address maps to a valid byte.

**ReadInput semantics.** `ReadInput` with an invalid `input_idx` (out of range
for the input vector) returns `0.0` as a soft default. This is the one case
where an out-of-bounds index does *not* panic.

**Sensor and neighbor query semantics.** `ReadSensorCell`, `ReadSensorCreature`,
`ReadSensorSummary`, `ReadNeighborCell`, and `ReadNeighborCreature` read from
the pre-assembled `CreatureInputs` / `SensorFrame` built by the `sensors/`
module at the start of Phase 1 (cognition). The VM does not access `WorldState`
directly. Invalid sensor/neighbor indices are hard faults (panic). Sensor data
is provided as f32 values; the VM does not interpret their meaning.

---

## 4. Invalid Index and Fault Semantics

v3 uses a **panic-for-invariants** error philosophy. Internal errors are
programming bugs, not recoverable conditions. There are no `Result` return types
for VM-internal faults.

### Hard faults (panic)

The following index violations cause an immediate panic:

- **Invalid register index** -- register file access out of bounds.
- **Invalid const index** -- constant pool access out of bounds.
- **Invalid output slot index** -- payload/meta buffer access out of bounds.
- **Invalid sensor index** -- sensor query with out-of-range sensor ID.
- **Invalid neighbor index** -- neighbor query with out-of-range neighbor ID.
- **Invalid field index** -- sensor/neighbor field access out of bounds.

### Soft default (0.0)

- **Invalid input_index in ReadInput** -- returns `0.0`. This accommodates
  programs that were compiled against a different input vector length.

### Never invalid

- **Memory addresses** -- always resolved via `rem_euclid(1024)`, so every
  integer address maps to a valid byte. No fault is possible.

---

## 5. Numeric Determinism Contract

All VM arithmetic operates on **IEEE-754 f32** values. Determinism is enforced
by sanitizing every f32 result before it is written to a register.

### sanitize_f32 rules

Every value written to a register passes through `sanitize_f32`:

| Condition | Result |
|-----------|--------|
| NaN | `0.0` |
| +Inf | `+1_000_000_000.0` |
| -Inf | `-1_000_000_000.0` |
| Finite, magnitude > 1e9 | Clamped to `[-1_000_000_000.0, +1_000_000_000.0]` |
| Finite, magnitude <= 1e9 | Unchanged |

### Float-to-integer rounding

Instructions that convert float to integer (`ToI32`, `ToU8`, `StoreMem8`,
`StoreMem8Imm`) use **ties-away-from-zero** rounding (Rust `f32::round()`
semantics).

### CmpEq epsilon

The `CmpEq` instruction takes an epsilon operand. Before comparison, epsilon is
clamped to the range `[1e-6, 1.0]`. The comparison succeeds if
`abs(a - b) <= clamped_epsilon`.

### Truthiness threshold

A value is truthy if `value >= 0.5`, falsy otherwise. Used by `And`, `Or`,
`Not`, `ToBool`, and `JumpIfZero`.

### Division by zero

`Div` with a zero divisor writes `0.0` to the destination register (not NaN or
Inf). This is applied before `sanitize_f32`.

---

## 6. VM Opcode Cost Table

Each opcode has a base energy cost. A global multiplier scales all costs
uniformly.

> **v3 note:** v3 uses `u32` energy. These f32 base costs will be scaled to
> integer energy units by multiplying by an integer scaling factor (e.g., 100 or
> 1000) so that all per-opcode costs become whole numbers. The relative ratios
> between opcodes are preserved exactly.

| Opcode | Base Cost (f32) |
|--------|----------------|
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
| Halt | 0.05 |
| LoadMem8 | 0.16 |
| StoreMem8 | 0.18 |
| LoadMem8Imm | 0.14 |
| StoreMem8Imm | 0.16 |

**Global multiplier.** All base costs above are multiplied by a single
configurable global multiplier before being deducted from the creature's energy
budget. This allows tuning overall VM execution cost without changing the
relative opcode weights.

---

## 7. Output Override Lifecycle

The VM maintains two output buffers:

- **Internal payload buffer** -- written by `WriteInternalPayload`.
- **World action meta buffer** -- written by `WriteWorldActionMeta`.

### Last-write-wins

Multiple writes to the same buffer slot within a single program execution
overwrite silently. Only the final value in each slot matters at emit time.

### Clear on emit

When `EmitInternal` executes, the internal payload buffer is consumed and
cleared. Subsequent `WriteInternalPayload` instructions write to a fresh buffer.

When `EmitWorldAction` executes, the world action meta buffer is consumed,
cleared, and execution halts.

### Clear on dispatch end

At the end of the creature's dispatch (after the VM halts for any reason), both
buffers are discarded. Un-emitted buffer contents do not persist across ticks.

---

## 8. Creature Memory Contract

Each creature has **1 KiB (1024 bytes)** of persistent memory.

**Persistence across ticks.** Memory contents survive between ticks. A
creature's memory is available at the start of every tick with the same contents
it had at the end of the previous tick.

**Byte-for-byte copy on reproduction.** When a creature reproduces, the child
receives an exact byte-for-byte copy of the parent's memory at the time of
reproduction. Parent and child memory are independent after the copy; subsequent
writes by either do not affect the other.

**Addressing.** All memory addresses are resolved via
`resolved_addr = raw_addr.rem_euclid(1024)`. There are no out-of-bounds memory
faults.

**Encoding.** Memory stores raw bytes (`u8`). `LoadMem8` / `LoadMem8Imm` read a
single byte and return it as an f32 in `[0.0, 255.0]`. `StoreMem8` /
`StoreMem8Imm` clamp the source register to `[0.0, 255.0]`, round to the
nearest integer (ties-away-from-zero), and write the resulting `u8`.
