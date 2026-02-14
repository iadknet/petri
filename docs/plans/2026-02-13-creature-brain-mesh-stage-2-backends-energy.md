# Petri V2 Stage 2: Mesh Kernel and Energy-Bounded Backends

> **Stage Type:** This stage is part of a **MAJOR GREENFIELD REWRITE** program.

**Goal:** Implement the new mesh cognition runtime in `v2-core` with FIFO routing semantics and energy-bounded graph/VM backend execution.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2-core` runtime schema, queue engine, backend contracts, and energy accounting; excludes reproduction/mutation/ecology and product surface API details.
**Docs Impact:** Update stage plan for greenfield kernel implementation tasks only.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: introduces the new cognition substrate targeted at richer behaviors.
- `GP-03`: kernel and energy semantics are implemented with strict TDD.
- `GP-04`: runtime diagnostics are designed into queue execution from first implementation.

## Boundary Impact

- Implementation is fully contained in `v2/crates/v2-core`.
- Legacy runtime fields and types are not modified.
- Backends are internal to `v2-core`; server/web contract decisions are deferred to stage 4.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/lib.rs` | change | Stage needs complete runtime module layout and core types. |
| `crates/petri-core/src/world/*` | keep | Legacy runtime remains untouched per greenfield policy. |
| `v2/docs/BOUNDARIES.md` | change | Must be updated with backend-specific anti-coupling guidance. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should queue order be FIFO in v1? | Yes. | user+agent | resolved |
| Should world action commit halt further dispatch immediately? | Yes. | user+agent | resolved |
| Should VM loops be bounded by dispatch caps or energy only? | Energy only in v1. | user+agent | resolved |

### Task 1: Add failing schema and queue semantics tests

Files:
- Create: `v2/crates/v2-core/tests/mesh_schema.rs`
- Create: `v2/crates/v2-core/tests/mesh_queue.rs`

Steps:
1. Add failing tests for genome validity and target resolution.
2. Add failing tests for FIFO dispatch behavior.
3. Add failing tests for immediate world-action commit and implicit noop-on-drain.

### Task 2: Implement mesh schema and queue runtime

Files:
- Create: `v2/crates/v2-core/src/mesh/mod.rs`
- Create: `v2/crates/v2-core/src/mesh/schema.rs`
- Create: `v2/crates/v2-core/src/mesh/queue.rs`

Steps:
1. Define `CreatureGenome`, `NodeGenome`, output schemas, and packet types.
2. Implement schema validation routines.
3. Implement queue kernel and execution outcome semantics.

### Task 3: Add failing energy/backends tests

Files:
- Create: `v2/crates/v2-core/tests/mesh_energy.rs`

Steps:
1. Add failing tests for dispatch-entry energy charging.
2. Add failing tests for graph static tariff charging.
3. Add failing tests for VM per-op metering and energy exhaustion termination.

### Task 4: Implement backend contracts and energy accounting

Files:
- Create: `v2/crates/v2-core/src/backends/mod.rs`
- Create: `v2/crates/v2-core/src/backends/graph.rs`
- Create: `v2/crates/v2-core/src/backends/vm.rs`
- Create: `v2/crates/v2-core/src/energy.rs`

Steps:
1. Implement backend interface and graph adapter.
2. Implement VM executor with per-op charging.
3. Integrate energy accounting into queue dispatch.

## Checkpoint Boundaries

### Entry Checkpoint (`S2-ENTRY`)

Required before starting:
1. Stage-1 `S1-EXIT` is `go`.
2. `v2` boundary docs confirm no legacy imports.

### Midpoint Checkpoint (`S2-MID`)

Required before task 4:
1. Queue/schema tests pass.
2. Energy tests fail for expected reasons before backend implementation.

Stop conditions:
1. Queue contract churn across tasks.
2. Backend interface requires coupling to `v2-server` or `v2-web`.

### Exit Checkpoint (`S2-EXIT`)

Required to close stage:
1. Queue and energy tests all green.
2. Runtime diagnostics are emitted from kernel outcomes.
3. Stage verification commands pass.

Go / stop rule:
1. `go` to stage 3 only if kernel/backends are stable for one full test pass.
2. `stop` and amend stage 2 if semantics still churn.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: VM scope can expand prematurely.
- Risk: energy defaults may make kernel behavior hard to tune.
- Rollback: revert stage-2 commits and keep stage-1 skeleton untouched.
