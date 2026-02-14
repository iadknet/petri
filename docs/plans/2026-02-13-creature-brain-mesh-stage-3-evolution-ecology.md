# Creature Brain Mesh Stage 3: Evolution Engine and Ecology Expansion (MAJOR REFACTOR/REWRITE)

> **Stage Type:** This stage is part of a **MAJOR REFACTOR/REWRITE** program.

**Goal:** Build a mesh-genome asexual mutation engine and ecology-first environmental pressures that promote naturally emergent novelty without explicit novelty scoring.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** Evolution operators, reproduction integration, and ecology dynamics in `petri-core`; excludes final server/web contract cutover.
**Docs Impact:** Add stage plan file; canonical doc updates deferred to stage 4.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: introduces strong mutation and module reuse operators needed for richer behaviors.
- `GP-03`: ensures mutation validity invariants are covered by dedicated tests.
- `GP-04`: provides lightweight ecology/run-health signals for play-focused tuning.

## Boundary Impact

- Dependency direction: unchanged; all evolution/environment policy remains in `petri-core`.
- Public API / wire format: runtime diagnostics add minimal diversity/health counters, not deep lineage explainability.
- Test migration: reproduction and mutation tests shift from graph-controller assumptions to mesh-genome assumptions.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/src/world/spawn.rs` | change | Founder generation and offspring mutation must operate on mesh genomes. |
| `crates/petri-core/src/world/tick.rs` reproduction path | change | Offspring creation and parent energy transfer must use new genome operators. |
| `crates/petri-core/src/world/food.rs` | change | Ecology dynamics are the primary novelty source and need stronger pressure controls. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should explicit novelty score affect reproduction? | No; use natural viability selection only. | user+agent | resolved |
| Should sexual recombination be in v1? | No; asexual-only in this stage. | user+agent | resolved |
| Should mutation include node and bounded subgraph duplication? | Yes. | user+agent | resolved |

### Task 1: Add failing mutation and reproduction tests

Files:
- Add: `crates/petri-core/tests/mesh_mutation.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

Steps:
1. Add failing tests for add/remove/retarget node and output mutations.
2. Add failing tests for node duplication and bounded subgraph duplication invariants.
3. Add failing tests for offspring viability under asexual strong-mutation flow.

### Task 2: Implement mesh mutation operators

Files:
- Add: `crates/petri-core/src/world/evolution.rs`
- Modify: `crates/petri-core/src/world/spawn.rs`
- Modify: `crates/petri-core/src/world/tick.rs`

Steps:
1. Implement mutation operator set for structural, backend-local, and output-reference mutations.
2. Implement node duplication and bounded subgraph duplication operators.
3. Wire mutation config into founder and offspring generation paths.

### Task 3: Implement ecology-first pressure expansion

Files:
- Modify: `crates/petri-core/src/world/food.rs`
- Modify: `crates/petri-core/src/config.rs`
- Modify: `crates/petri-core/src/world/paint.rs`

Steps:
1. Add richer resource heterogeneity and scarcity-gradient controls.
2. Add periodic environment regime shifts (season-like parameter cycling).
3. Add local crowding pressure hooks to reward viable interaction strategies.

### Task 4: Add lightweight play telemetry

Files:
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world/mod.rs`

Steps:
1. Add run-health and minimal diversity proxies (no deep lineage explainability).
2. Keep telemetry compact and optional in frame/detail payloads.
3. Add tests for telemetry presence and bounds.

## Checkpoint Boundaries

### Entry Checkpoint (`S3-ENTRY`)

Required before starting:
1. Program checkpoint `CP-2` is marked `go`.
2. Stage-2 backend and energy APIs are frozen for this stage window.
3. Stage-3 failing mutation tests are committed first (TDD gate).

### Midpoint Checkpoint (`S3-MID`)

Required before task 3:
1. Mutation operators (including node/subgraph duplication) pass structural validity tests.
2. Reproduction path compiles with mesh genomes and no legacy controller dependencies.

Stop conditions:
1. Population collapse is near-total under baseline settings and prevents meaningful smoke runs.
2. Ecology changes require altering stage-2 backend/energy contracts.

### Exit Checkpoint (`S3-EXIT`)

Required to close stage:
1. Mutation invariants pass and ecology expansion tests pass.
2. Stage verification commands pass.
3. Stage-4 integration cutover assumptions are documented (payload fields, telemetry expectations).

Go / stop rule:
1. `go` to stage 4 only if baseline short-horizon runs stay stable and non-collapsing.
2. `stop` and tune stage-3 defaults if stability threshold is not met.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cargo test -p petri-core mesh_mutation -- --nocapture`
3. `cargo test -p petri-core world::tests::founder_seeded_hybrid_population_survives_short_horizon -- --exact`
4. `cargo test -p petri-core`

## Risks and Rollback

- Risk: strong mutation defaults may induce frequent population collapse.
- Risk: ecology pressure changes can invalidate current benchmark assumptions.
- Rollback: revert stage-3 commits and restore pre-stage ecology and mutation pathways.
