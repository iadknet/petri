# Petri V2 Stage 2: Runtime Kernel and Energy-Bounded Backends (`CP-1`)

> **Stage Type:** This stage is part of a **MAJOR GREENFIELD REWRITE** program.

**Goal:** Complete checkpoint `CP-1` by delivering an integrated `v2-core` runtime executor that combines FIFO routing, backend execution, and energy-bounded halt semantics.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2-core` runtime schema/queue/energy/backend integration and tests; excludes mutation/ecology and product-surface protocol/UI work.
**Docs Impact:** Update stage-2 execution around checkpoint slices and align file targets with the current `v2-core` layout; consume CP-1 runtime spec and shared test matrix.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: delivers the new cognition execution substrate for richer behavior search.
- `GP-03`: enforces incremental TDD slices with explicit completion gates.
- `GP-04`: preserves inspectable runtime outcomes (`world_action`, `implicit_noop`, exhaustion/error paths).

## Boundary Impact

- Work remains isolated to `v2/crates/v2-core`.
- No modifications to legacy runtime code.
- `v2-server`, `v2-cli`, and `v2-web` stay out of scope for this stage.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/*.rs` runtime modules | change | Stage 2 owns mesh/energy/backend/runtime kernel implementation details. |
| `v2/crates/v2-core/tests/*.rs` | change | Stage 2 uses focused tests to lock queue/energy/backend semantics. |
| `crates/petri-core/src/world/*` legacy runtime | keep | Greenfield policy requires zero coupling or in-place legacy edits. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should Stage 2 be tracked as one monolithic block? | No; use slice checkpoints (`CP-1A`..`CP-1D`). | user+agent | resolved |
| Should dispatch limits exist separate from energy in v1? | No; energy is the execution budget. | user+agent | resolved |
| Should first valid world action halt queue execution immediately? | Yes. | user+agent | resolved |
| Should VM opcode costs be uniform or per-opcode? | Per-opcode baseline costs with global multiplier. | user+agent | resolved |
| Should graph richness come from a graph DSL? | No; richer fixed-function graph operators with bounded state are in scope. | user+agent | resolved |
| Should creatures have full in-range sensor visibility with rich metadata? | Yes, expose full sensor frame + rich query semantics in CP-1. | user+agent | resolved |
| Is sensor range globally configurable? | Yes, global `sensor_radius` in runtime/world config. | user+agent | resolved |

## Specification Dependencies

- Runtime semantics source of truth:
  - `docs/plans/2026-02-14-v2-cp1-runtime-execution-spec.md`
- Core schema + VM ISA source of truth:
  - `docs/plans/2026-02-14-v2-core-schema-vm-isa-spec.md`
- Gate consistency source of truth:
  - `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

## Slice Checkpoints

### `CP-1A`: Mesh schema + FIFO queue kernel (`complete`)

Files:
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/src/mesh.rs`
- Create: `v2/crates/v2-core/tests/mesh_kernel.rs`

Exit evidence:
1. Schema validation rejects missing entry nodes and invalid targets.
2. FIFO routing behavior is verified.
3. First world action commit halts execution and queue-drain returns implicit no-op.

### `CP-1B`: Energy metering primitives (`complete`)

Files:
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/src/energy.rs`
- Create: `v2/crates/v2-core/tests/mesh_energy.rs`

Exit evidence:
1. Dispatch entry, graph tariff, and action costs meter correctly.
2. VM opcode-table metering drains energy and terminates long loops.

### `CP-1C`: Backend execution contracts (`complete`)

Files:
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/src/backends.rs`
- Create: `v2/crates/v2-core/tests/mesh_backends.rs`

Exit evidence:
1. Graph backend emits outputs and charges static tariff.
2. VM backend is bounded by remaining energy and reports exhaustion.
3. Graph backend supports stateful/aggregating operators with deterministic bounded behavior.

### `CP-1D`: Integrated runtime executor (`pending`)

Files:
- Create: `v2/crates/v2-core/src/runtime.rs`
- Modify: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-core/tests/mesh_runtime.rs`

Steps:
1. Write failing integration tests for full dispatch loop with energy charging order.
2. Implement runtime executor that composes queue dispatch + backend execution + action commit halt.
3. Ensure immediate death/exhaustion semantics are represented in runtime outcomes.
4. Keep runtime deterministic for fixture-based tests.
5. Align runtime outcomes with CP-1 spec contract types.
6. Ensure VM input-read/output-write opcodes are wired end-to-end via runtime+backend contracts.
7. Ensure `ReadInput` slot mapping/normalization and VM numeric determinism rules are covered by tests.
8. Ensure graph operator richness (integrator/momentum/oscillator/pooling/adaptive gain) is covered by tests.
9. Ensure full-radius sensor frame richness (food + creature metadata including phenotype) is covered by graph/VM query tests.
10. Ensure `sensor_radius` is explicitly global and configurable with tests.

Exit gate:
1. Runtime integration tests pass with first-action halt and energy-exhaustion behavior verified.
2. `CP-1` is marked complete only when `CP-1D` is green.

## Checkpoint Boundaries

### Entry Checkpoint (`S2-ENTRY`): `go`

Required:
1. `CP-0` complete (`Stage 1` exit gate satisfied).
2. No legacy imports in `v2-core`.

### Midpoint Checkpoint (`S2-MID`): `go`

Required:
1. `CP-1A`, `CP-1B`, and `CP-1C` complete.
2. `CP-1D` tests exist and fail for expected reasons before implementation.

Stop conditions:
1. Runtime integration requires coupling to non-`v2-core` crates.
2. Queue/energy semantics churn without test updates.

### Exit Checkpoint (`S2-EXIT`): `pending`

Required:
1. `CP-1D` complete.
2. Full stage verification commands pass.

Go / stop rule:
1. `go` to Stage 3 only when `CP-1` is fully complete.
2. `stop` and re-plan if runtime semantics remain unstable after integration.

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
10. `cd v2 && cargo test -p v2-core --test vm_opcode_costs`
11. `cd v2 && cargo test -p v2-core --test vm_input_mapping`
12. `cd v2 && cargo test -p v2-core --test vm_output_overrides`
13. `cd v2 && cargo test -p v2-core --test vm_numeric_determinism`
14. `cd v2 && cargo test -p v2-core --test vm_sensor_queries`
15. `cd v2 && cargo test -p v2-core --test sensor_frame_contract`
16. `cd v2 && cargo test -p v2-core --test graph_sensor_inputs`
17. `cd v2 && cargo test -p v2-core --test graph_operator_richness`
18. `cd v2 && cargo test -p v2-core --test graph_stateful_ops`
19. `cd v2 && cargo test -p v2-core --test sensor_radius_global_config`
20. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: integrating queue + backend + energy may add hidden coupling or duplicate charging.
- Risk: runtime outcomes can drift if integration tests are too weak.
- Rollback:
1. Revert `CP-1D` implementation commits.
2. Keep completed slices (`CP-1A`..`CP-1C`) intact as stable baseline.
