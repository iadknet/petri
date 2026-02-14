# Creature Brain Mesh Rewrite Program (MAJOR REFACTOR/REWRITE)

> **Program Type:** This is a **MAJOR REFACTOR/REWRITE** plan split into staged implementation documents.

**Goal:** Replace the current single-controller creature cognition model with a heterogeneous node-mesh DNA runtime (`graph` + `vm`) optimized for rich emergent behavior and fast reset-driven iteration.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** End-to-end cognition/runtime/evolution redesign across `petri-core` plus required contract updates in `petri-server`, `petri-cli`, and `web`; excludes backward compatibility for prior creature genomes/snapshots.
**Docs Impact:** Add staged rewrite plans under `docs/plans/`; update canonical strategy/reference docs during stage 4 cutover; no compatibility stubs retired in this program kickoff.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: introduces a richer cognition substrate (mesh routing, mixed node backends, typed internal/world outputs).
- `GP-02`: keeps transport concerns in server/web while locating simulation policy and cognition execution in `petri-core`.
- `GP-03`: enforces stage-local TDD and verification gates to keep rewrite risk controlled.
- `GP-04`: keeps lightweight but sufficient runtime observability for play-focused iteration.

## Boundary Impact

- Dependency direction: unchanged (`petri-graph -> petri-core -> petri-server/petri-cli`), but `petri-core` becomes owner of node-mesh execution and packet routing.
- Public API / wire format: `WorldFrame`, creature detail, and snapshot internals will change; no backward compatibility guarantee during this rewrite phase.
- Test migration: existing cognition tests tied to current arbitration/`OutputHalt` semantics migrate stage-by-stage to queue-driven mesh semantics.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/src/world/tick.rs` | change | Tick policy is the rewrite center and must own queue-driven cognition execution. |
| `crates/petri-graph/src/eval/*` | change | Graph backend survives but becomes one backend behind a shared node execution interface. |
| `crates/petri-server/src/*` runtime/snapshot endpoints | change | Contract wiring must reflect new genome/state shapes after core cutover. |
| `web/src/protocol.ts` and consumer panels | change | Client contracts and inspector fields must match rewritten runtime payloads. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should this program preserve deterministic replay across full runs? | No; determinism is test-scoped, not a product requirement for this rewrite phase. | user+agent | resolved |
| Should novelty scoring be explicit in selection? | No; novelty should emerge naturally from ecology and mutation pressure. | user+agent | resolved |
| Should an emergency dispatch cap exist beyond energy exhaustion? | No in v1; energy exhaustion is the sole execution budget limiter. | user+agent | resolved |

## Stage Breakdown

### Stage 1: Runtime Kernel and Genome Primitives

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-1-runtime-kernel.md`

Deliverables:
1. Typed DNA schema (`CreatureGenome`, `NodeGenome`, output definitions).
2. Unified internal packet contract and FIFO queue runtime skeleton.
3. Immediate world-action commit + implicit noop-on-drain semantics.

### Stage 2: Backend Execution and Energy Metering

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`

Deliverables:
1. Graph backend adapter with static complexity tariff.
2. VM backend with per-op energy metering and loop bounding via energy.
3. Shared backend interface and per-dispatch energy accounting.

### Stage 3: Evolution Engine and Ecology-First Emergence

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`

Deliverables:
1. Asexual strong-mutation engine for mesh genomes.
2. Node duplication and bounded subgraph duplication operators.
3. Ecology pressure expansion (resource heterogeneity, scarcity gradients, seasonal shifts).

### Stage 4: Integration Cutover and Contract Rebaseline

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

Deliverables:
1. Snapshot/runtime/server/web contract migration to mesh model.
2. Legacy cognition path removal.
3. Canonical doc rebaseline and full quality gate completion.

## Program Checkpoint Boundaries

| checkpoint | boundary trigger | required evidence | go / stop rule |
| --- | --- | --- | --- |
| `CP-0 Program Kickoff` | Before stage 1 starts | Program plan + all stage plans exist and pass plan harness strict mode | Stop if any stage plan is missing or has unresolved strict violations |
| `CP-1 Kernel Freeze` | After stage 1 verification passes | Stage-1 tests pass, queue semantics are stable, snapshot schema migration direction is locked | Stop if stage-1 semantics still churn or stage-1 tests are unstable |
| `CP-2 Backend/Energy Freeze` | After stage 2 verification passes | Graph tariff + VM per-op metering tests pass and backend interface is unchanged for one full pass | Stop if backend interface/energy policy is still changing |
| `CP-3 Evolution/Ecology Freeze` | After stage 3 verification passes | Mutation invariants pass and ecology controls are integrated without explicit novelty scoring | Stop if mutation validity or ecology safety checks are failing |
| `CP-4 Cutover Release Gate` | Before declaring rewrite complete | Stage-4 cross-crate tests + doc/architecture/plan harnesses + workspace tests/build all pass | Stop if any completion gate command fails |

## Cross-Stage Handoff Rules

1. No stage may begin implementation until prior checkpoint is marked `go`.
2. Every checkpoint handoff must include command outputs (summarized in commit/PR notes).
3. If scope expands beyond the stage plan, update the stage plan first, then continue.
4. If a checkpoint is `stop`, either:
5. land a scope-reduction amendment in the current stage plan, or
6. open a new stage file and supersede the previous stage plan.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`
4. Stage-local verification commands from each stage plan
5. Final cutover gates (stage 4):
6. `cargo fmt --all --check`
7. `cargo clippy --workspace --all-targets -- -D warnings`
8. `cargo test --workspace`
9. `cd web && npm run build`

## Risks and Rollback

- Risk: broad cross-crate contract churn can destabilize runtime/server/web integration.
- Risk: no backward compatibility increases reset frequency during tuning.
- Risk: VM + graph coexistence can increase mutation/search complexity if not constrained by schema validation.
- Rollback strategy:
1. Land each stage in isolated commits.
2. Keep stage gates green before proceeding.
3. Revert stage commits independently if later stage quality drops.
