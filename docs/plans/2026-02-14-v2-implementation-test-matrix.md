# Petri V2 Implementation Test Matrix

**Goal:** Define the full checkpoint-gated verification matrix so implementation progress is measured against the right behaviors, not just compile success.
**Goal IDs:** GP-03, GP-04
**Scope:** Test coverage map for `CP-0`..`CP-3`, including required suites, pass criteria, and command gates; excludes new feature design work.
**Docs Impact:** Adds the execution test matrix referenced by checkpoint and stage plans.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-03`: enforces reliable, fast, and meaningful test gates at each checkpoint.
- `GP-04`: ensures runtime behavior and ecosystem health are observable in tests.

## Boundary Impact

- No crate boundary changes.
- Adds a shared verification contract used by stage plans.
- Keeps test ownership aligned with crate ownership (`v2-core`, `v2-server`, `v2-cli`, `v2-web`).

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md` | change | Stage 2 should reference mandatory runtime test groups. |
| `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md` | change | Stage 3 should reference mutation/ecology non-collapse checks. |
| `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md` | change | Stage 4 should reference protocol fixture and integration gates. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should each checkpoint require a full workspace test pass? | Yes at exit; partial suites allowed at midpoint. | user+agent | resolved |
| Should nondeterministic tests be allowed in checkpoint gates? | No; checkpoint gates require deterministic fixtures/seeds. | user+agent | resolved |
| Should performance checks block CP-1/CP-2? | No hard perf gate until CP-3; only reliability gates before then. | user+agent | resolved |
| Should CP-1 require explicit neighbor-input parity tests in addition to sensor tests? | Yes; neighbor mapping/normalization and VM/graph neighbor access are mandatory CP-1 gates. | user+agent | resolved |

## Checkpoint Test Matrix

| checkpoint | required suites | pass criteria |
| --- | --- | --- |
| `CP-0` | workspace/build skeleton checks | `v2` Rust workspace checks and `v2/web` build succeeds |
| `CP-1` | `mesh_kernel`, `mesh_energy`, `mesh_backends`, `mesh_runtime`, `mesh_schema_contract`, `vm_isa`, `vm_memory`, `vm_io`, `vm_sensor_queries`, `vm_neighbor_queries`, `vm_opcode_costs`, `vm_input_mapping`, `vm_output_overrides`, `vm_numeric_determinism`, `sensor_frame_contract`, `neighbor_input_contract`, `graph_sensor_inputs`, `graph_neighbor_inputs`, `graph_operator_richness`, `graph_stateful_ops`, `sensor_radius_global_config` | all tests pass; runtime semantics, schema/ISA contracts, global configurable sensor radius, full-radius sensor richness, complete neighbor-input parity, VM memory/I/O/sensor+neighbor opcodes, emitted-output ordering/error precedence, `max_input_slots` edge validation, slot mapping/override lifecycle, numeric determinism, opcode-cost behavior, and graph fixed-function richness match CP-1 specs |
| `CP-2` | `mutation_invariants`, `mutation_repair`, `reproduction_memory_inheritance`, `ecology_pressures`, `ecology_noncollapse` | all tests pass on fixed seeds; no invariant violations, offspring memory-copy semantics hold, duplication/remap defaults are enforced, and non-collapse threshold contract is met using actual seeded baseline population |
| `CP-3` | server lifecycle/payload/ws tests, cli ndjson tests, web protocol fixtures, end-to-end smoke | all tests pass; schema/version parity across all surfaces, lifecycle transition contract is enforced, non-2xx responses follow error-envelope schema, and ws event/payload schemas are fixture-locked |

## Required Test Categories

### Unit

1. schema validity and metadata checks
2. operator-level mutation behavior
3. protocol parser/serializer roundtrips

### Integration

1. queue + backend + energy runtime integration
2. reproduction + mutation + ecology interaction flow
3. server lifecycle and ws stream behavior

### Smoke

1. bounded-run non-crash simulation
2. bounded-run non-collapse sanity for CP-2
3. minimal e2e (`server + web` connect and receive status/frame)

## Determinism and Reliability Rules

1. Checkpoint tests must use fixed seeds where randomness exists.
2. Time-based assertions should use tolerance windows, not exact wall-clock assumptions.
3. Any flaky test blocks checkpoint exit until stabilized or rewritten.
4. Long-running tests must be marked and excluded from default quick loop.

## Command Gates by Checkpoint

### `CP-0` exit

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo check`
3. `cd v2/web && npm run build`

### `CP-1` exit

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

### `CP-2` exit

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core --test mutation_invariants`
3. `cd v2 && cargo test -p v2-core --test mutation_repair`
4. `cd v2 && cargo test -p v2-core --test reproduction_memory_inheritance`
5. `cd v2 && cargo test -p v2-core --test ecology_pressures`
6. `cd v2 && cargo test -p v2-core --test ecology_noncollapse`
7. `cd v2 && cargo test -p v2-core`

### `CP-3` exit

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo fmt --all --check`
3. `cd v2 && cargo clippy --workspace --all-targets -- -D warnings`
4. `cd v2 && cargo test --workspace`
5. `cd v2/web && npm run test`
6. `cd v2/web && npm run build`

## Task List

### Task 1: Align stage plans with this matrix

Files:
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

Steps:
1. Add matrix references in each stage.
2. Ensure verification command sections are aligned and non-conflicting.
3. Keep stage stop/go rules tied to required suite groups.

### Task 2: Add verification-readme cross reference

Files:
- Modify: `docs/plans/README.md`

Steps:
1. Add short note that active stage plans should reference this matrix for gate consistency.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`

## Risks and Rollback

- Risk: over-large default test runs may slow dev loop.
- Risk: incomplete matrix adoption can create conflicting gate definitions across plans.
- Rollback:
1. Revert matrix and referencing changes.
2. Reintroduce gate updates in smaller slices per stage plan.
