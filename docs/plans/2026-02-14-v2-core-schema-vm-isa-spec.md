# Petri V2 Core Schema and VM ISA Spec

**Goal:** Lock the CP-1 core contracts for genome structure, creature memory model, typed node inputs/outputs, backend definitions, and VM instruction set so runtime implementation is unambiguous.
**Goal IDs:** GP-01, GP-03
**Scope:** `v2/crates/v2-core` schema and VM/backend contracts; excludes mutation/ecology policy and transport protocols.
**Docs Impact:** Adds schema+ISA contract plan consumed by Stage 2 and CP-1 runtime spec.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: provides the expressive substrate for richer emergent behavior.
- `GP-03`: prevents drift by defining strict, testable schema and ISA contracts.

## Boundary Impact

- Scope stays in `v2-core` type contracts and backend executor assumptions.
- No transport/API coupling introduced.
- Legacy schema remains untouched.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/mesh.rs` | change | Add explicit `backend_def` and typed I/O contracts. |
| `v2/crates/v2-core/src/backends.rs` | change | Backend behavior must align to schema/ISA definitions. |
| `v2/crates/v2-core/tests/mesh_kernel.rs` | change | Update/add tests for stricter schema and typed field validation. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should VM be stack-based or register-based in v1? | Register-based for easier deterministic metering. | user+agent | resolved |
| Should graph nodes have custom mini-language in CP-1? | No; use richer fixed-function graph operators instead of a programmable graph DSL. | user+agent | resolved |
| Should internal packet fields be untyped strings? | No, typed value schema is required. | user+agent | resolved |
| What is per-creature VM memory size in v1? | Fixed at `1024` bytes (`1 KiB`) per creature. | user+agent | resolved |
| How are `ReadInput` slots mapped and normalized? | Slot index maps to ordered `input_refs`; normalization uses the table in this spec. | user+agent | resolved |
| How do output-write overrides behave across VM execution? | Per-dispatch override buffers; last-write-wins; explicit reset rules. | user+agent | resolved |
| What happens for invalid VM indices? | Register/const/output index faults are hard runtime errors; input index is soft-default `0.0`. | user+agent | resolved |
| What numeric determinism rules apply? | Canonical sanitize/clamp/rounding rules are mandatory. | user+agent | resolved |
| Should v1 include logic/conversion opcodes? | Yes (`And`, `Or`, `Not`, `Clamp01`, `ToI32`, `ToU8`, `ToBool`). | user+agent | resolved |
| Should graph nodes include temporal and aggregation richness in CP-1? | Yes, via bounded fixed-function operators (integrator, momentum, oscillator, pooling, adaptive gain). | user+agent | resolved |

## Genome Schema Contract

### `CreatureGenome`

1. `entry_node_id: NodeId`
2. `nodes: Vec<NodeGenome>`
3. `evolution_params: Option<EvolutionParams>`

Validation:
1. `nodes` non-empty
2. unique node IDs
3. `entry_node_id` exists
4. all output targets resolve

### Creature runtime memory contract

1. Every creature has a persistent `1 KiB` memory arena during its lifetime:
- `memory: [u8; 1024]`
2. Memory arena is runtime state, not encoded directly in genome fields.
3. VM instructions may read/write this arena.
4. Memory state persists across ticks for a living creature.

### `NodeGenome`

1. `node_id: NodeId`
2. `node_type: NodeType` (`graph` or `vm`)
3. `backend_def: BackendDef`
4. `output_definitions: Vec<OutputDefinition>`
5. `local_state_init: Vec<u8>`

`backend_def` must match `node_type`:
- `node_type=graph` => `BackendDef::Graph`
- `node_type=vm` => `BackendDef::Vm`

### `BackendDef`

1. `BackendDef::Graph(GraphBackendDef)`
2. `BackendDef::Vm(VmBackendDef)`

`GraphBackendDef`:
- `operator: GraphOperator`
- `inputs: Vec<InputReference>`
- `coefficients: Vec<f32>` (length must equal `inputs` for weighted operators)
- `bias: f32`
- `state_slot_count: u8` (`0..=8`, graph-local persistent scalar state slots)
- `operator_params: GraphOperatorParams`

`VmBackendDef`:
- `register_count: u8` (`1..=32`)
- `program: Vec<VmInstruction>` (`1..=128` instructions)
- `constants: Vec<f32>` (`0..=64`)
- `max_input_slots: u8` (`1..=64`, used for bounds validation of `ReadInput`)

## Typed Input/Output Contract

### `InputReference`

1. `World(WorldInputKey)`
2. `Introspection(IntrospectionInputKey)`
3. `Packet(PacketFieldKey)`

`WorldInputKey` (v1):
- `food_here`
- `nearest_food_distance`
- `nearest_food_direction`
- `nearest_creature_distance`
- `nearest_creature_direction`
- `occupied_here`

`IntrospectionInputKey` (v1):
- `energy_current`
- `energy_spent_this_tick`
- `energy_remaining_this_tick`
- `age_ticks`
- `memory_bytes_total` (always `1024` in v1)

`PacketFieldKey`:
- stable string key from packet payload map

### Packet field values

`PacketValue`:
1. `Bool(bool)`
2. `I32(i32)`
3. `F32(f32)`
4. `U8(u8)`

`PayloadField`:
- `key: String`
- `value: PacketValue`

### Output definitions

`OutputDefinition::InternalTarget`:
- `target_node_id: NodeId`
- `input_refs: Vec<InputReference>`
- `payload_fields: Vec<PayloadField>`

`OutputDefinition::WorldAction`:
- `action_kind: WorldActionKind`
- `action_metadata_fields: Vec<ActionMetadataField>`

`ActionMetadataField`:
1. `Direction(i32)`
2. `Amount(u8)`
3. `Slot(u8)`

VM emit override rule:
1. VM may override output field values before emit via write-output opcodes.
2. Override target is identified by `(output_index, field_index)` in the selected output definition.
3. Override source value is `f32` and coerced to target field type at emit time.

### `ReadInput` slot mapping and normalization contract

1. For each dispatch, runtime builds `resolved_input_slots: Vec<f32>` by iterating `input_refs` in order.
2. `ReadInput { dst, input_index }` reads `resolved_input_slots[input_index]`.
3. `input_index >= resolved_input_slots.len()` returns `0.0` (soft default, not a fault).

Normalization table for built-in input keys:

| input key | mapped value |
| --- | --- |
| `food_here` | `food_density / 255.0` |
| `nearest_food_distance` | `clamp(distance / sensor_radius, 0.0, 1.0)`, default `1.0` |
| `nearest_food_direction` | `atan2(dy, dx) / PI` in `[-1.0, 1.0]`, default `0.0` |
| `nearest_creature_distance` | `clamp(distance / sensor_radius, 0.0, 1.0)`, default `1.0` |
| `nearest_creature_direction` | `atan2(dy, dx) / PI` in `[-1.0, 1.0]`, default `0.0` |
| `occupied_here` | `1.0` if occupied else `0.0` |
| `energy_current` | raw energy units (`f32`) |
| `energy_spent_this_tick` | raw energy units (`f32`) |
| `energy_remaining_this_tick` | raw energy units (`f32`) |
| `age_ticks` | raw ticks as `f32` |
| `memory_bytes_total` | constant `1024.0` |

Packet value conversion:
1. `Bool` -> `0.0` or `1.0`
2. `I32` -> `f32`
3. `F32` -> sanitized `f32` (see numeric determinism)
4. `U8` -> `f32` in `[0.0, 255.0]`

### Output override lifecycle contract

1. VM execution starts with empty override buffers.
2. `WriteInternalPayload` and `WriteWorldActionMeta` write into per-dispatch buffers keyed by `(output_index, field_index)`.
3. Multiple writes to the same key are `last-write-wins`.
4. `EmitInternal`/`EmitWorldAction` apply current overrides for that `output_index`.
5. After emit, overrides for emitted `output_index` are cleared.
6. On dispatch end (`Halt`, exhaustion, or program end), all override buffers are cleared.

## VM ISA Contract (v1)

### Registers and values

1. VM uses `f32` register values.
2. Boolean semantics use `0.0` (false) / `1.0` (true).
3. Invalid register index is a runtime error outcome, never panic.

### Instruction set

1. `Noop`
2. `LoadConst { dst, const_idx }`
3. `Move { dst, src }`
4. `Add { dst, a, b }`
5. `Sub { dst, a, b }`
6. `Mul { dst, a, b }`
7. `Div { dst, a, b }` (division-by-zero writes `0.0`)
8. `Min { dst, a, b }`
9. `Max { dst, a, b }`
10. `Abs { dst, src }`
11. `Neg { dst, src }`
12. `Clamp01 { dst, src }`
13. `CmpGt { dst, a, b }`
14. `CmpLt { dst, a, b }`
15. `CmpEq { dst, a, b, epsilon }`
16. `And { dst, a, b }`
17. `Or { dst, a, b }`
18. `Not { dst, src }`
19. `ToI32 { dst, src }`
20. `ToU8 { dst, src }`
21. `ToBool { dst, src }`
22. `JumpIfZero { cond, offset }`
23. `Jump { offset }`
24. `ReadInput { dst, input_index }`
25. `WriteInternalPayload { output_index, payload_field_index, src }`
26. `WriteWorldActionMeta { output_index, metadata_field_index, src }`
27. `EmitInternal { output_index }`
28. `EmitWorldAction { output_index }`
29. `Halt`
30. `LoadMem8 { dst, addr_reg }`
31. `StoreMem8 { addr_reg, src }`
32. `LoadMem8Imm { dst, addr }`
33. `StoreMem8Imm { addr, src }`

### VM execution rules

1. PC starts at `0`.
2. Out-of-range PC terminates program (`halted=true`).
3. Jump offsets are signed relative offsets.
4. Each executed instruction consumes opcode-specific energy:
- `effective_cost(opcode) = vm_opcode_base_cost(opcode) * vm_opcode_cost_multiplier`
5. Execution halts on first emitted world action (runtime-level rule still applies).
6. Memory address resolution uses wrapping semantics over `1024` bytes:
- `resolved_addr = raw_addr.rem_euclid(1024)`
7. `LoadMem8*` writes byte value as `f32` in `[0.0, 255.0]`.
8. `StoreMem8*` clamps source register to `[0.0, 255.0]`, rounds to nearest integer, and writes `u8`.
9. `ReadInput` reads from per-dispatch resolved input slots; out-of-range index yields `0.0`.
10. `WriteInternalPayload` and `WriteWorldActionMeta` write pending overrides for emit.
11. Emit-time coercion rules for write-output overrides:
- `f32 -> i32`: round to nearest integer
- `f32 -> u8`: clamp `[0.0, 255.0]` and round
- `f32 -> bool`: `>= 0.5` is `true`, else `false`
12. If remaining energy is below an opcode's effective cost, that opcode does not execute and VM exits as exhausted.

### Invalid index and fault semantics

1. Invalid register index in any register-addressing opcode is a hard VM runtime fault.
2. Invalid `const_idx` in `LoadConst` is a hard VM runtime fault.
3. Invalid `output_index` or field index for write/emit opcodes is a hard VM runtime fault.
4. Invalid `input_index` is a soft default (`0.0`) and not a fault.
5. Memory addresses are never invalid (wrapping semantics).
6. Jump target outside program bounds halts VM (not a fault).

### Numeric determinism contract

1. VM math uses IEEE-754 single-precision (`f32`).
2. Every arithmetic/conversion write goes through `sanitize_f32`:
- `NaN -> 0.0`
- `+/-Inf -> clamp to +/-1_000_000_000.0`
- finite values clamped to `[-1_000_000_000.0, 1_000_000_000.0]`
3. Float->integer rounding uses ties-away-from-zero (`round()` semantics).
4. `CmpEq` epsilon is clamped to `[1e-6, 1.0]`.
5. Truthiness for logical ops and `ToBool` is `value >= 0.5`.

### VM opcode baseline cost table (v1 defaults)

Global scalar:
- `vm_opcode_cost_multiplier` default: `1.0`

Per-opcode baseline (`vm_opcode_base_cost`):

| opcode | base cost |
| --- | --- |
| `Noop` | `0.05` |
| `LoadConst` | `0.08` |
| `Move` | `0.08` |
| `Add` | `0.12` |
| `Sub` | `0.12` |
| `Mul` | `0.12` |
| `Div` | `0.16` |
| `Min` | `0.12` |
| `Max` | `0.12` |
| `Abs` | `0.10` |
| `Neg` | `0.10` |
| `Clamp01` | `0.10` |
| `CmpGt` | `0.12` |
| `CmpLt` | `0.12` |
| `CmpEq` | `0.12` |
| `And` | `0.12` |
| `Or` | `0.12` |
| `Not` | `0.10` |
| `ToI32` | `0.10` |
| `ToU8` | `0.10` |
| `ToBool` | `0.10` |
| `JumpIfZero` | `0.14` |
| `Jump` | `0.10` |
| `ReadInput` | `0.12` |
| `WriteInternalPayload` | `0.14` |
| `WriteWorldActionMeta` | `0.14` |
| `EmitInternal` | `0.20` |
| `EmitWorldAction` | `0.24` |
| `Halt` | `0.05` |
| `LoadMem8` | `0.16` |
| `StoreMem8` | `0.18` |
| `LoadMem8Imm` | `0.14` |
| `StoreMem8Imm` | `0.16` |

## Graph Backend Contract (v1)

`GraphOperator`:
1. `Passthrough`
2. `WeightedSum`
3. `Threshold { threshold: f32 }`
4. `Clamp01`
5. `DecayIntegrator { state_slot: u8, alpha: f32 }`
6. `Momentum { state_slot: u8, beta: f32 }`
7. `Oscillator { phase_slot: u8, frequency: f32, amplitude: f32, bias: f32 }`
8. `SumPool`
9. `MeanPool`
10. `MaxPool`
11. `AdaptiveGain { gain_slot: u8, learning_rate: f32, min_gain: f32, max_gain: f32 }`

`GraphOperatorParams` normalization and bounds:
1. `alpha` and `beta` are clamped to `[0.0, 1.0]`.
2. `frequency` is clamped to `[0.0, 8.0]` cycles per tick.
3. `amplitude` is clamped to `[0.0, 10.0]`.
4. `learning_rate` is clamped to `[0.0, 0.1]`.
5. `state_slot` references must be `< state_slot_count`.

Graph local-state contract:
1. Each graph node owns `state_slot_count` persistent `f32` slots.
2. State persists across ticks for living creatures.
3. State resets to zero on creature birth unless explicitly initialized via `local_state_init`.
4. Graph state updates are deterministic and operator-local (no cross-node state writes).

Operator semantics:
1. `DecayIntegrator`: `s = (1 - alpha) * s + alpha * input0`; output `s`.
2. `Momentum`: `delta = input0 - input1`; `s = beta * s + (1 - beta) * delta`; output `s`.
3. `Oscillator`: `phase = fract(phase + frequency * dt)` with `dt=1`; output `bias + amplitude * sin(2*pi*phase)`.
4. `SumPool`: output sum of all inputs.
5. `MeanPool`: output arithmetic mean of inputs (or `0.0` for empty input list).
6. `MaxPool`: output max of inputs (or `0.0` for empty input list).
7. `AdaptiveGain`: `gain = clamp(gain + learning_rate * input1, min_gain, max_gain)`; output `gain * input0`.

Graph cost model (still static, but operator-aware):
1. Runtime uses static graph tariff multiplied by operator multiplier.
2. `effective_graph_cost = graph_base_tariff * graph_operator_cost_multiplier(operator)`
3. Default multipliers:
- `Passthrough`: `0.7`
- `WeightedSum`: `1.0`
- `Threshold`: `0.9`
- `Clamp01`: `0.8`
- `DecayIntegrator`: `1.2`
- `Momentum`: `1.3`
- `Oscillator`: `1.4`
- `SumPool`: `1.0`
- `MeanPool`: `1.1`
- `MaxPool`: `1.2`
- `AdaptiveGain`: `1.3`

Rules:
1. Graph backend remains fixed-function (no graph mini-language/bytecode in CP-1).
2. Graph backend is deterministic and bounded-time per dispatch.
3. Graph compute energy uses static operator-aware tariff from runtime config.
4. Graph emits outputs from `output_definitions` after operator evaluation.

## Task List

### Task 1: Add failing schema contract tests

Files:
- Create: `v2/crates/v2-core/tests/mesh_schema_contract.rs`
- Create: `v2/crates/v2-core/tests/graph_operator_richness.rs`
- Create: `v2/crates/v2-core/tests/graph_stateful_ops.rs`

Steps:
1. Add failing tests for backend/node_type mismatch.
2. Add failing tests for typed input/output field validation.
3. Add failing tests for graph/vm backend bound checks.
4. Add failing tests for graph operator parameter and state-slot bounds.

### Task 2: Add failing VM ISA tests

Files:
- Create: `v2/crates/v2-core/tests/vm_isa.rs`
- Create: `v2/crates/v2-core/tests/vm_memory.rs`
- Create: `v2/crates/v2-core/tests/vm_io.rs`
- Create: `v2/crates/v2-core/tests/vm_opcode_costs.rs`
- Create: `v2/crates/v2-core/tests/vm_input_mapping.rs`
- Create: `v2/crates/v2-core/tests/vm_output_overrides.rs`
- Create: `v2/crates/v2-core/tests/vm_numeric_determinism.rs`

Steps:
1. Add failing tests for arithmetic and compare ops.
2. Add failing tests for jumps and loop energy exhaustion behavior.
3. Add failing tests for emit/halt semantics.
4. Add failing tests for memory load/store and address wrapping behavior.
5. Add failing tests for input read/output write opcode semantics.
6. Add failing tests verifying baseline opcode cost table and multiplier scaling.
7. Add failing tests for `ReadInput` slot ordering and normalization mapping.
8. Add failing tests for override lifecycle and invalid output index faults.
9. Add failing tests for numeric sanitize/clamp/rounding determinism.

### Task 3: Implement schema and VM ISA contracts

Files:
- Modify: `v2/crates/v2-core/src/mesh.rs`
- Modify: `v2/crates/v2-core/src/backends.rs`
- Modify: `v2/crates/v2-core/src/energy.rs`

Steps:
1. Extend schema types/validation to match this contract.
2. Implement VM instruction execution and graph operator handling.
3. Keep runtime deterministic and panic-free on invalid genomes.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test mesh_schema_contract`
3. `cd v2 && cargo test -p v2-core --test vm_isa`
4. `cd v2 && cargo test -p v2-core --test vm_memory`
5. `cd v2 && cargo test -p v2-core --test vm_io`
6. `cd v2 && cargo test -p v2-core --test vm_opcode_costs`
7. `cd v2 && cargo test -p v2-core --test vm_input_mapping`
8. `cd v2 && cargo test -p v2-core --test vm_output_overrides`
9. `cd v2 && cargo test -p v2-core --test vm_numeric_determinism`
10. `cd v2 && cargo test -p v2-core --test graph_operator_richness`
11. `cd v2 && cargo test -p v2-core --test graph_stateful_ops`
12. `cd v2 && cargo test -p v2-core --test mesh_runtime`
13. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: over-large ISA in CP-1 may slow implementation unnecessarily.
- Risk: schema strictness can break earlier CP-1 tests without coordinated updates.
- Rollback:
1. Revert schema/ISA expansion commit.
2. Re-introduce in smaller slices while keeping this spec as target contract.
