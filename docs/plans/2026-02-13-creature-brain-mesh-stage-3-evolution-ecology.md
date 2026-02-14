# Petri V2 Stage 3: Evolution and Ecology Engine

> **Stage Type:** This stage is part of a **MAJOR GREENFIELD REWRITE** program.

**Goal:** Implement strong asexual mutation and ecology pressure systems in `v2-core` to drive emergent novelty without explicit novelty rewards.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2-core` mutation operators, reproduction flow, and ecology dynamics; excludes final server/web protocol/UI rollout.
**Docs Impact:** Update stage plan to reflect greenfield mutation/ecology build path; consume CP-2 evolution/ecology spec and shared test matrix.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: expands behavior search space through mesh-structure mutation.
- `GP-03`: mutation validity and non-collapse tests keep iteration reliable.
- `GP-04`: stage introduces minimal telemetry proxies for runtime observability.

## Boundary Impact

- Stage remains inside `v2-core` runtime and config modules.
- No edits to legacy evolution/ecology modules.
- `v2-server` consumes stage outputs later without driving stage internals.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/evolution/*` | change | New mutation engine and invariants live here. |
| `v2/crates/v2-core/src/ecology/*` | change | Novelty-through-environment policy is implemented here. |
| `crates/petri-core/src/world/evolution.rs` | keep | Legacy evolution stays unchanged as historical implementation. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should novelty archives/scoring be included in v1? | No. | user+agent | resolved |
| Should sexual recombination be in scope? | No, asexual only. | user+agent | resolved |
| Should node and subgraph duplication be mandatory operators? | Yes. | user+agent | resolved |

## Specification Dependencies

- Evolution/ecology semantics source of truth:
  - `docs/plans/2026-02-14-v2-cp2-evolution-ecology-spec.md`
- Gate consistency source of truth:
  - `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

### Task 1: Add failing mutation invariant tests

Files:
- Create: `v2/crates/v2-core/tests/mutation_invariants.rs`
- Create: `v2/crates/v2-core/tests/mutation_repair.rs`

Steps:
1. Add failing tests for add/remove/retarget operators.
2. Add failing tests for node and bounded subgraph duplication.
3. Add failing tests ensuring generated genomes remain structurally valid.

### Task 2: Implement mutation engine

Files:
- Create: `v2/crates/v2-core/src/evolution/mod.rs`
- Create: `v2/crates/v2-core/src/evolution/config.rs`
- Create: `v2/crates/v2-core/src/evolution/operators.rs`
- Create: `v2/crates/v2-core/src/evolution/validation.rs`

Steps:
1. Implement operator set and mutation config.
2. Implement duplication operators and repair/validation pass.
3. Integrate mutation into reproduction pathways.

### Task 3: Add failing ecology behavior tests

Files:
- Create: `v2/crates/v2-core/tests/ecology_pressures.rs`
- Create: `v2/crates/v2-core/tests/ecology_noncollapse.rs`

Steps:
1. Add failing tests for heterogeneity/scarcity gradient effects.
2. Add failing tests for periodic regime shifts.
3. Add failing tests for local crowding pressure.

### Task 4: Implement ecology pressure systems and minimal telemetry

Files:
- Create: `v2/crates/v2-core/src/ecology/mod.rs`
- Create: `v2/crates/v2-core/src/ecology/config.rs`
- Create: `v2/crates/v2-core/src/ecology/regimes.rs`
- Create: `v2/crates/v2-core/src/telemetry.rs`

Steps:
1. Implement ecology pressure knobs and dynamics.
2. Implement non-intrusive telemetry proxies for run health.
3. Add baseline non-collapse integration test scenarios.

## Checkpoint Boundaries

### Entry Checkpoint (`S3-ENTRY`)

Required before starting:
1. Stage-2 `S2-EXIT` is `go`.
2. Kernel/backends API is frozen for this stage window.

### Midpoint Checkpoint (`S3-MID`)

Required before task 4:
1. Mutation invariant tests pass.
2. Ecology tests fail first, then pass with implementation.
3. Mutation defaults/weights are implemented as specified.

Stop conditions:
1. Mutation operators repeatedly produce invalid genomes.
2. Baseline ecology causes immediate deterministic collapse.

### Exit Checkpoint (`S3-EXIT`)

Required to close stage:
1. Mutation invariants and ecology tests pass.
2. Baseline non-collapse checks pass.
3. Stage verification commands pass.

Go / stop rule:
1. `go` to stage 4 only if stage baseline is stable enough for product-surface wiring.
2. `stop` and tune stage defaults if instability persists.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-core mutation_invariants`
3. `cd v2 && cargo test -p v2-core mutation_repair`
4. `cd v2 && cargo test -p v2-core ecology_pressures`
5. `cd v2 && cargo test -p v2-core ecology_noncollapse`
6. `cd v2 && cargo test -p v2-core`

## Risks and Rollback

- Risk: high mutation pressure may hide useful incremental behaviors.
- Risk: ecology pressure interplay may overfit to specific seeds.
- Rollback: revert stage-3 commits and keep stage-2 kernel/backends baseline.
