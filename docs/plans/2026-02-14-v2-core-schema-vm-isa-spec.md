# Petri V2 Core Schema and VM ISA Spec

**Goal:** Lock the CP-1 core contracts for genome structure, typed node inputs/outputs, backend definitions, and VM instruction set so runtime implementation is unambiguous.
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
| Should graph nodes have custom mini-language in CP-1? | No, keep graph backend minimal with simple expression operators. | user+agent | resolved |
| Should internal packet fields be untyped strings? | No, typed value schema is required. | user+agent | resolved |

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

`VmBackendDef`:
- `register_count: u8` (`1..=32`)
- `program: Vec<VmInstruction>` (`1..=128` instructions)
- `constants: Vec<f32>` (`0..=64`)

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
- `memory_slot_count`

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
12. `CmpGt { dst, a, b }`
13. `CmpLt { dst, a, b }`
14. `CmpEq { dst, a, b, epsilon }`
15. `JumpIfZero { cond, offset }`
16. `Jump { offset }`
17. `EmitInternal { output_index }`
18. `EmitWorldAction { output_index }`
19. `Halt`

### VM execution rules

1. PC starts at `0`.
2. Out-of-range PC terminates program (`halted=true`).
3. Jump offsets are signed relative offsets.
4. Each executed instruction consumes `vm_per_op_cost` energy.
5. Execution halts on first emitted world action (runtime-level rule still applies).

## Graph Backend Contract (v1)

`GraphOperator`:
1. `Passthrough`
2. `WeightedSum`
3. `Threshold { threshold: f32 }`
4. `Clamp01`

Rules:
1. Graph backend is single-pass deterministic evaluation.
2. Graph compute energy uses static tariff from runtime config.
3. Graph emits outputs from `output_definitions` after operator evaluation.

## Task List

### Task 1: Add failing schema contract tests

Files:
- Create: `v2/crates/v2-core/tests/mesh_schema_contract.rs`

Steps:
1. Add failing tests for backend/node_type mismatch.
2. Add failing tests for typed input/output field validation.
3. Add failing tests for graph/vm backend bound checks.

### Task 2: Add failing VM ISA tests

Files:
- Create: `v2/crates/v2-core/tests/vm_isa.rs`

Steps:
1. Add failing tests for arithmetic and compare ops.
2. Add failing tests for jumps and loop energy exhaustion behavior.
3. Add failing tests for emit/halt semantics.

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
4. `cd v2 && cargo test -p v2-core --test mesh_runtime`
5. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: over-large ISA in CP-1 may slow implementation unnecessarily.
- Risk: schema strictness can break earlier CP-1 tests without coordinated updates.
- Rollback:
1. Revert schema/ISA expansion commit.
2. Re-introduce in smaller slices while keeping this spec as target contract.
