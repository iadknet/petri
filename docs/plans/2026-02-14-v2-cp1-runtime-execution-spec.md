# Petri V2 CP-1 Runtime Execution Spec

**Goal:** Define implementation-complete runtime semantics for checkpoint `CP-1D` so `v2-core` can execute mesh cognition with deterministic FIFO routing, energy metering, and immediate world-action halt.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2/crates/v2-core` runtime execution semantics, core runtime types, and test expectations; excludes mutation/ecology and transport/UI wiring.
**Docs Impact:** Adds CP-1 implementation spec consumed by Stage 2 plan; no canonical strategy docs updated in this step.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: locks the cognition execution model that all higher-level behavior depends on.
- `GP-03`: specifies deterministic ordering and explicit test gates to avoid runtime ambiguity.
- `GP-04`: defines structured runtime outcomes for inspectable behavior.

## Boundary Impact

- `v2/crates/v2-core` gains a `runtime` module that composes existing `mesh`, `energy`, and `backends`.
- No `v2-core` dependency on `v2-server`, `v2-cli`, or `v2-web`.
- Legacy crates remain untouched and unreferenced.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/mesh.rs` | keep | Retain existing schema and queue types as runtime inputs. |
| `v2/crates/v2-core/src/energy.rs` | keep | Reuse current charging primitives and extend only if needed. |
| `v2/crates/v2-core/src/backends.rs` | change | Backend contract must expose enough info for runtime integration. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should runtime dispatch use FIFO or priority scheduling? | FIFO only for CP-1. | user+agent | resolved |
| Should world-action arbitration pick highest confidence? | No; first valid emitted action commits immediately. | user+agent | resolved |
| What halts execution when no action is emitted? | Queue drain returns implicit no-op outcome. | user+agent | resolved |
| Should runtime expose full local sensor picture (food + creature metadata)? | Yes, via `SensorFrame` consumed by graph/VM sensor queries. | user+agent | resolved |
| Is sensor radius globally configurable? | Yes, global `sensor_radius` in `RuntimeConfig` applies to all creatures. | user+agent | resolved |
| Should immediate-neighbor inputs be first-class and complete like sensor queries? | Yes; expose 8-direction neighbor refs/opcodes with metadata parity to `SensorFrame`. | user+agent | resolved |
| How should mixed invalid/valid emitted world actions be handled? | Emitted outputs are processed in order; invalid world-action metadata errors immediately and later actions are not considered. | user+agent | resolved |
| Is CP-1 intended to guarantee full-run deterministic replay? | No; CP-1 guarantees deterministic runtime ordering semantics and deterministic test fixtures only. | user+agent | resolved |

## Specification Dependencies

- Schema/ISA contract source of truth:
  - `docs/plans/2026-02-14-v2-core-schema-vm-isa-spec.md`
- Shared gate matrix:
  - `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

## Runtime Contract

### Runtime types (to add in `v2/crates/v2-core/src/runtime.rs`)

1. `RuntimeConfig`
- `dispatch_entry_cost: f32`
- `graph_base_tariff: f32`
- `vm_opcode_cost_multiplier: f32`
- `sensor_radius: u16` (global for world, applies to all creatures, minimum `1`)
- `action_costs: RuntimeActionCosts`

2. `RuntimeActionCosts`
- `move_cost: f32`
- `eat_cost: f32`
- `reproduce_cost: f32`
- `inventory_pickup_cost: f32`
- `inventory_put_cost: f32`
- `noop_cost: f32`

3. `RuntimeContext`
- `config: RuntimeConfig`
- `energy_before_tick: f32`
- `memory_bytes: [u8; 1024]`
- `graph_state_slots: Vec<f32>` (node-local persistent state, per graph node)
- `sensor_frame: SensorFrame` (full in-range world snapshot with rich metadata)

4. `RuntimeOutcome`
- `CommittedAction { action: WorldActionDef, energy_spent: f32, energy_remaining: f32, dispatches: usize }`
- `ImplicitNoOp { energy_spent: f32, energy_remaining: f32, dispatches: usize }`
- `EnergyExhausted { energy_spent: f32, dispatches: usize }`
- `RuntimeError { error: RuntimeError, dispatches: usize }`

5. `RuntimeError`
- `Schema(MeshSchemaError)`
- `InvalidTarget(NodeId)`
- `InvalidActionMetadata(WorldActionKind)`
- `InsufficientEnergyForDispatch`
- `VmFault(VmFaultCode)`

6. `VmFaultCode`
- `InvalidRegisterIndex`
- `InvalidConstIndex`
- `InvalidOutputIndex`
- `InvalidOutputFieldIndex`
- `InvalidSensorField`
- `InvalidNeighborField`
- `InvalidNeighborDirection`

### Execution algorithm (single creature, single tick)

1. Validate genome before dispatch. Invalid schema returns `RuntimeError::Schema`.
2. Build `sensor_frame` for the acting creature (full in-range snapshot).
3. Seed FIFO queue with entry packet (`entry_node_id`).
4. Loop while queue not empty:
- charge dispatch entry cost first
- if energy reaches zero after entry charge: return `EnergyExhausted`
- dispatch next packet (FIFO)
- execute backend by target node type
- backend returns emitted outputs in deterministic list order
- VM backend supports opcode-level `ReadInput` and output write instructions (`WriteInternalPayload`, `WriteWorldActionMeta`)
- VM backend supports rich sensor query opcodes (`ReadSensorCell`, `ReadSensorCreature`, `ReadSensorSummary`)
- VM backend supports complete 8-direction neighbor query opcodes (`ReadNeighborCell`, `ReadNeighborCreature`)
- allow VM backend to mutate creature `memory_bytes` through memory opcodes
- allow graph backend to mutate graph-local state slots via bounded fixed-function operators
- allow graph backend to consume rich `Sensor*` and `Neighbor*` input references from `sensor_frame`
- enforce schema/ISA fault policy: hard-fault invalid register/const/output indices, invalid sensor/neighbor field discriminants, and invalid neighbor directions; soft-default invalid input index
- charge backend compute energy (`graph_base_tariff * graph_operator_multiplier` or VM opcode-table metering)
- if backend reports exhaustion: return `EnergyExhausted`
- process emitted outputs in emitted-list order:
  - `InternalTarget`: validate target node exists, else return `RuntimeError::InvalidTarget`; enqueue when valid
  - `WorldAction`: validate metadata; invalid metadata returns `RuntimeError::InvalidActionMetadata` immediately
  - first valid `WorldAction` commits immediately
    - charge action cost by action kind
    - if action cost exceeds remaining energy, clamp to `0.0` and still return `CommittedAction`
    - halt immediately; do not process remaining queue entries
5. If queue drains without action, return `ImplicitNoOp`.

### Energy charging order (normative)

For each dispatch:
1. `dispatch_entry_cost`
2. backend compute cost
- VM compute cost uses opcode baseline table from schema/ISA spec multiplied by `vm_opcode_cost_multiplier`
- Graph compute cost uses operator multiplier table from schema/ISA spec with `graph_base_tariff`
3. action cost (only if action emitted and committed)

Notes:
- Energy is clamped to `[0, +inf)` at each operation.
- No separate dispatch cap in CP-1.
- Runtime must never panic due to user genome input; return `RuntimeError`.
- Runtime config validation rejects `sensor_radius=0` before tick execution.
- Memory arena is fixed at `1024` bytes and persists across ticks for a living creature.
- Graph local state slots persist across ticks for living creatures.
- Sensor frame exposes full local radius metadata (food, occupancy, creature phenotype and stats).
- Neighbor inputs provide first-class 8-direction aliases over `SensorFrame` with matching normalization and wrap behavior.

### Determinism rules

1. FIFO queue order is stable and deterministic.
2. Internal target enqueues preserve emitted output order.
3. Runtime does not iterate maps for ordering-sensitive logic.
4. Tests use deterministic fixtures and avoid RNG in CP-1 runtime tests.
5. Full-run deterministic replay across entire simulations is not required by CP-1.

## Task List

### Task 1: Add failing CP-1 runtime integration tests

Files:
- Create: `v2/crates/v2-core/tests/mesh_runtime.rs`

Steps:
1. Add test for dispatch/backends/action charging order.
2. Add test for immediate halt on first world action.
3. Add test for queue drain to implicit no-op.
4. Add test for energy exhaustion mid-dispatch.
5. Add test for invalid action metadata path returning runtime error.
6. Add test for opcode-cost multiplier affecting VM exhaustion timing.
7. Add test coverage for VM fault mapping and soft-default `ReadInput` out-of-range behavior.
8. Add test coverage for graph stateful operator persistence and operator-aware tariff charging.
9. Add test coverage for full-radius sensor-frame queries and metadata normalization.
10. Add test coverage for 8-direction neighbor query mapping, normalization, and graph neighbor-input access.
11. Add test coverage for emitted-output ordering when invalid world-action metadata appears before later valid world actions.
12. Add test coverage for `sensor_radius=0` runtime-config rejection.

### Task 2: Implement runtime module

Files:
- Create: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`

Steps:
1. Add runtime types from this spec.
2. Implement runtime executor with normative charging order.
3. Integrate `mesh`, `energy`, and `backends` without changing crate boundaries.

### Task 3: Tighten backend contract for runtime composition

Files:
- Modify: `v2/crates/v2-core/src/backends.rs`

Steps:
1. Ensure backend execution returns enough compute/exhaustion metadata.
2. Keep graph/vm behaviors aligned with existing tests.
3. Add focused unit tests if contract shape changes.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test mesh_kernel`
3. `cd v2 && cargo test -p v2-core --test mesh_energy`
4. `cd v2 && cargo test -p v2-core --test mesh_backends`
5. `cd v2 && cargo test -p v2-core --test mesh_runtime`
6. `cd v2 && cargo test -p v2-core --test mesh_schema_contract`
7. `cd v2 && cargo test -p v2-core --test vm_isa`
8. `cd v2 && cargo test -p v2-core --test vm_memory`
9. `cd v2 && cargo test -p v2-core --test vm_io`
10. `cd v2 && cargo test -p v2-core --test vm_sensor_queries`
11. `cd v2 && cargo test -p v2-core --test vm_neighbor_queries`
12. `cd v2 && cargo test -p v2-core --test vm_opcode_costs`
13. `cd v2 && cargo test -p v2-core --test vm_input_mapping`
14. `cd v2 && cargo test -p v2-core --test vm_output_overrides`
15. `cd v2 && cargo test -p v2-core --test vm_numeric_determinism`
16. `cd v2 && cargo test -p v2-core --test sensor_frame_contract`
17. `cd v2 && cargo test -p v2-core --test neighbor_input_contract`
18. `cd v2 && cargo test -p v2-core --test graph_sensor_inputs`
19. `cd v2 && cargo test -p v2-core --test graph_neighbor_inputs`
20. `cd v2 && cargo test -p v2-core --test graph_operator_richness`
21. `cd v2 && cargo test -p v2-core --test graph_stateful_ops`
22. `cd v2 && cargo test -p v2-core --test sensor_radius_global_config`
23. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: duplicated charging (backend + runtime) can silently overcharge energy.
- Risk: ambiguous outcome semantics can create stage-4 API churn later.
- Rollback:
1. Revert `runtime.rs` and related test changes.
2. Keep CP-1A/B/C baseline intact and re-implement from this spec.
