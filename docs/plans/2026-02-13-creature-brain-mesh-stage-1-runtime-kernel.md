# Creature Brain Mesh Stage 1: Runtime Kernel (MAJOR REFACTOR/REWRITE)

> **Stage Type:** This stage is part of a **MAJOR REFACTOR/REWRITE** program.

**Goal:** Introduce typed mesh-genome primitives and a queue-driven cognition kernel in `petri-core` with world-action terminal semantics.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** Core runtime type/system refactor and tick execution skeleton in `petri-core`; excludes VM opcode execution and ecology tuning.
**Docs Impact:** Add stage plan file and update program plan references; no canonical strategy docs changed in this stage.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: establishes the new cognition representation and routing model needed for richer behavior.
- `GP-03`: stage uses TDD for new schema/queue semantics to reduce rewrite regressions.
- `GP-04`: introduces minimal execution diagnostics needed to inspect routing/termination outcomes.

## Boundary Impact

- Dependency direction: unchanged; changes remain within `petri-core`.
- Public API / wire format: internal creature state/snapshot structs begin migrating away from single-controller fields.
- Test migration: add stage-specific tests for routing semantics and schema validation, independent of VM details.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/src/world/mod.rs` creature state definitions | change | Must hold mesh genome and node-local state instead of legacy single controller fields. |
| `crates/petri-core/src/world/tick.rs` cognition lifecycle | change | Must become FIFO packet routing with terminal world-action commit. |
| `crates/petri-core/src/types.rs` snapshot/detail payload types | change | Must carry mesh-structure data and stage-minimal routing diagnostics. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should queue policy be FIFO in v1? | Yes; keep runtime predictable while semantics stabilize. | user+agent | resolved |
| Should world action commit immediately on first valid action packet? | Yes; action commit is terminal for tick execution. | user+agent | resolved |
| Should queue drain without action become noop? | Yes; implicit noop keeps action semantics total. | user+agent | resolved |

### Task 1: Add failing tests for mesh kernel semantics

Files:
- Modify: `crates/petri-core/src/world/tests.rs`
- Add: `crates/petri-core/tests/mesh_kernel.rs`

Steps:
1. Add failing tests for genome schema validation and invalid target-node routing rejection.
2. Add failing tests for FIFO routing semantics and first-world-action terminal halt behavior.
3. Add failing test for implicit noop on queue drain with no action.

### Task 2: Introduce mesh DNA and output schemas

Files:
- Add: `crates/petri-core/src/world/mesh.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/types.rs`

Steps:
1. Define `CreatureGenome`, `NodeGenome`, `NodeType`, `InternalTargetDef`, `WorldActionDef`, and packet types.
2. Define schema validators for IDs, references, and action metadata field constraints.
3. Replace creature runtime storage from legacy single controller to genome + node-state placeholders.

### Task 3: Implement queue-driven tick kernel

Files:
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/helpers.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

Steps:
1. Seed queue with entry-node packet and run FIFO dispatch loop.
2. Integrate immediate commit on first valid world-action output.
3. Emit implicit noop when queue drains without action.
4. Keep placeholder backend execution path (graph-only adapter stub in stage 1).

### Task 4: Stage-minimal diagnostics and snapshot migration start

Files:
- Modify: `crates/petri-core/src/world/snapshot.rs`
- Modify: `crates/petri-core/src/types.rs`

Steps:
1. Add minimal routing diagnostics (`dispatch_count`, `termination_reason`).
2. Start migrating snapshot creature internals to mesh schema (no backward compatibility handling).
3. Update snapshot round-trip tests for new required fields.

## Checkpoint Boundaries

### Entry Checkpoint (`S1-ENTRY`)

Required before starting:
1. Program checkpoint `CP-0` is marked `go`.
2. Legacy cognition tests are inventoried and tagged for migration/delete decisions.
3. Stage-1 failing tests are committed before production edits (TDD gate).

### Midpoint Checkpoint (`S1-MID`)

Required before task 3:
1. Schema validation tests fail for expected reasons, then pass after task 2.
2. Mesh runtime types compile without introducing server/web contract edits in this stage.

Stop conditions:
1. Queue semantics remain ambiguous (non-deterministic or contradictory commit behavior).
2. Snapshot schema direction is unclear.

### Exit Checkpoint (`S1-EXIT`)

Required to close stage:
1. FIFO routing, immediate world-action halt, and implicit noop-on-drain tests pass.
2. Stage verification commands pass.
3. Stage-2 interface assumptions are explicitly documented in stage-1 notes.

Go / stop rule:
1. `go` to stage 2 only if runtime kernel API is stable for one full verification pass.
2. `stop` and amend stage 1 if kernel interfaces still churn.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cargo test -p petri-core mesh_kernel -- --nocapture`
3. `cargo test -p petri-core world::tests:: -- --nocapture`
4. `cargo test -p petri-core`

## Risks and Rollback

- Risk: broad type churn can temporarily break many existing tests.
- Risk: partial snapshot migration can desynchronize with server contracts before stage 4.
- Rollback: revert stage-1 commits and restore legacy single-controller tick path.
