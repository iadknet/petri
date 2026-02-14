# Creature Brain Mesh Stage 2: Backends and Energy Metering (MAJOR REFACTOR/REWRITE)

> **Stage Type:** This stage is part of a **MAJOR REFACTOR/REWRITE** program.

**Goal:** Implement backend execution contracts for `graph` and `vm` node types with unified energy accounting and energy-bounded execution safety.
**Goal IDs:** GP-01, GP-03
**Scope:** Backend execution modules and energy-cost policy in `petri-core` and `petri-graph`; excludes mutation/ecology rollout.
**Docs Impact:** Add stage plan file; no canonical docs changed until integration cutover.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: enables heterogeneous node cognition (`graph` + `vm`) inside one creature mesh.
- `GP-03`: energy accounting and backend contracts are test-first to prevent silent infinite-loop/runtime hazards.

## Boundary Impact

- Dependency direction: preserved; `petri-graph` remains graph semantics owner, `petri-core` owns runtime policy/energy charging.
- Public API / wire format: internal runtime fields expand for per-dispatch and per-backend energy metrics.
- Test migration: prior `energy_per_think_step` tests replaced by dispatch-entry + backend compute cost semantics.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-graph/src/eval/*` | change | Graph evaluation becomes a backend implementation behind new mesh execution trait. |
| `crates/petri-core/src/world/tick.rs` | change | Tick loop must charge dispatch entry cost + backend cost + action cost. |
| `crates/petri-core/src/config.rs` | change | New config knobs required for dispatch base cost and VM opcode cost table defaults. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should graph cost be dynamic op metering or static tariff in v1? | Static tariff in v1. | user+agent | resolved |
| Should VM loops be bounded by hard dispatch cap? | No; bounded by per-op energy drain and energy exhaustion. | user+agent | resolved |
| Should backend errors kill the creature or noop the dispatch? | Kill creature in v1 to avoid silent corrupted execution. | agent | resolved |

### Task 1: Add failing energy/backend contract tests

Files:
- Add: `crates/petri-core/tests/mesh_energy.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

Steps:
1. Add failing tests for dispatch entry energy charge application.
2. Add failing tests for graph static tariff charging per dispatch.
3. Add failing tests showing VM loop dies by energy exhaustion.
4. Add failing tests for action-cost charging after world-action commit.

### Task 2: Implement backend interface and graph adapter

Files:
- Add: `crates/petri-core/src/world/backends/mod.rs`
- Add: `crates/petri-core/src/world/backends/graph_backend.rs`
- Modify: `crates/petri-graph/src/lib.rs`
- Modify: `crates/petri-core/src/world/mesh.rs`

Steps:
1. Define shared backend trait contract (inputs, emitted outputs, backend diagnostics).
2. Adapt existing graph evaluator to backend trait execution.
3. Precompute per-node graph tariff metadata from graph shape.

### Task 3: Implement VM backend with opcode metering

Files:
- Add: `crates/petri-core/src/world/backends/vm_backend.rs`
- Add: `crates/petri-core/src/world/backends/vm_types.rs`
- Modify: `crates/petri-core/src/world/mesh.rs`

Steps:
1. Define minimal opcode set for v1 world parity and internal routing support.
2. Implement per-op energy charging and opcode execution loop.
3. Stop VM execution on energy exhaustion and surface terminal reason.

### Task 4: Wire unified energy policy into tick runtime

Files:
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-core/src/types.rs`

Steps:
1. Apply `dispatch_entry_cost` before backend execution.
2. Apply backend compute cost returned by graph/VM backends.
3. Apply world-action side effect cost after commit.
4. Update diagnostics fields for dispatch/backend/action energy components.

## Checkpoint Boundaries

### Entry Checkpoint (`S2-ENTRY`)

Required before starting:
1. Program checkpoint `CP-1` is marked `go`.
2. Stage-1 kernel interface is frozen for this stage window.
3. Stage-2 failing tests are added before backend implementation edits.

### Midpoint Checkpoint (`S2-MID`)

Required before task 4:
1. Graph backend adapter compiles and passes graph-contract tests.
2. VM backend opcode metering tests fail then pass via TDD.

Stop conditions:
1. Backend trait contracts require repeated redesign.
2. Energy policy cannot be represented without changing stage-1 kernel semantics.

### Exit Checkpoint (`S2-EXIT`)

Required to close stage:
1. Dispatch entry, graph tariff, VM per-op, and action-cost tests pass.
2. Stage verification commands pass for both `petri-core` and `petri-graph`.
3. Stage-3 mutation/evolution assumptions are documented against the frozen backend interface.

Go / stop rule:
1. `go` to stage 3 only if backend and energy contracts remain unchanged through a full pass.
2. `stop` and issue a stage-2 amendment if contracts are still unstable.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cargo test -p petri-core mesh_energy -- --nocapture`
3. `cargo test -p petri-core`
4. `cargo test -p petri-graph`

## Risks and Rollback

- Risk: VM backend policy churn could destabilize early runtime behavior.
- Risk: graph tariff calibration might over-penalize large but useful graph nodes.
- Rollback: revert stage-2 backend commits and keep stage-1 graph-only execution placeholder.
