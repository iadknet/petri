# Creature Brain Mesh Stage 4: Integration Cutover and Doc Rebaseline (MAJOR REFACTOR/REWRITE)

> **Stage Type:** This stage is part of a **MAJOR REFACTOR/REWRITE** program.

**Goal:** Complete the mesh-runtime cutover across server/CLI/web contracts, remove legacy cognition pathways, and rebaseline canonical docs and quality gates.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Cross-crate integration updates, contract migration, doc rebaseline, and full completion gate execution; excludes post-cutover feature expansion.
**Docs Impact:** Update canonical strategy/reference docs and top-level README for mesh-runtime reality; mark this stage as superseding transitional cognition docs/plans as needed.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: ships the new cognition model end-to-end across runtime and clients.
- `GP-02`: confirms crate boundaries remain intact after cross-layer contract changes.
- `GP-03`: completes strict verification gates before declaring rewrite complete.
- `GP-04`: preserves practical observability during and after cutover.

## Boundary Impact

- Dependency direction: unchanged, but contracts crossing crate boundaries are rewritten.
- Public API / wire format: runtime, snapshot, and inspector payloads move fully to mesh-genome/state fields.
- Test migration: integration tests and UI/protocol tests migrate to mesh contracts; legacy cognition tests removed.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-server/src/*` runtime endpoints | change | Server contracts must serialize/patch new mesh runtime structures. |
| `crates/petri-cli/src/*` run/ablation tooling | change | CLI diagnostics and report fields must align with mesh runtime metrics. |
| `web/src/protocol.ts` and consumers | change | Web client must consume updated frame/detail/snapshot schemas. |
| `docs/reference/creature-controller-reference.md` | change | Reference must describe mesh runtime, not legacy single-controller cognition flow. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should legacy snapshot import compatibility remain? | No; backward compatibility remains out of scope for this rewrite. | user+agent | resolved |
| Should strict doc/architecture harnesses be required at completion? | Yes, per repository completion gate policy and current date threshold. | agent | resolved |
| Should old cognition-first transitional docs remain active? | No; archive/supersede once mesh docs are canonical. | agent | resolved |

### Task 1: Add failing integration and protocol tests

Files:
- Modify: `crates/petri-server/src/tests/*`
- Modify: `crates/petri-cli/src/*tests*`
- Modify: `web/src/**/*.test.ts*`

Steps:
1. Add failing tests for server runtime payloads using mesh fields.
2. Add failing snapshot import/export tests for new schema requirements.
3. Add failing web protocol tests for creature detail/frame mesh payload consumption.

### Task 2: Complete cross-crate contract migration

Files:
- Modify: `crates/petri-server/src/runtime.rs`
- Modify: `crates/petri-server/src/protocol.rs`
- Modify: `crates/petri-cli/src/*`
- Modify: `web/src/protocol.ts`
- Modify: `web/src/components/**/*`

Steps:
1. Replace legacy cognition and controller fields with mesh-genome/state fields.
2. Update runtime patch/startup draft structures for new energy/runtime knobs.
3. Ensure inspector UI and charts consume new diagnostics correctly.

### Task 3: Remove legacy runtime pathways and dead code

Files:
- Modify: `crates/petri-core/src/world/*`
- Modify: `crates/petri-graph/src/*`

Steps:
1. Remove old single-controller arbitration path and obsolete node/output assumptions.
2. Delete stale tests and fixtures tied to removed semantics.
3. Run clippy and address warnings introduced by removals.

### Task 4: Rebaseline canonical docs and plan links

Files:
- Modify: `README.md`
- Modify: `docs/reference/creature-controller-reference.md`
- Modify: `docs/strategy/architecture.md`
- Modify: `docs/strategy/roadmap.md`
- Modify: `docs/plans/archive/*` (as needed)

Steps:
1. Update docs to describe the mesh-runtime as current architecture.
2. Archive/supersede transitional plans that no longer reflect active implementation.
3. Ensure root compatibility stubs remain short and canonical-link oriented.

## Checkpoint Boundaries

### Entry Checkpoint (`S4-ENTRY`)

Required before starting:
1. Program checkpoint `CP-3` is marked `go`.
2. Stage-3 payload and telemetry fields are frozen for integration cutover.
3. Stage-4 failing integration/protocol tests are added before migration edits.

### Midpoint Checkpoint (`S4-MID`)

Required before task 3:
1. Server, CLI, and web compile against mesh contracts.
2. Contract migration tests fail then pass in each boundary layer (core/server/web).

Stop conditions:
1. Cross-crate contract drift appears (server and web disagree on payload schema).
2. Legacy path removal starts before integration tests are green.

### Exit Checkpoint (`S4-EXIT`)

Required to close stage:
1. Legacy cognition pathways removed and replacement tests pass.
2. Canonical docs rebaselined and transitional docs/plans archived/superseded.
3. Full completion gate command set passes.

Go / stop rule:
1. `go` (rewrite complete) only if all completion gates pass in one final pass.
2. `stop` and open a follow-up cutover-fix stage if any gate fails.

## Verification Commands

1. `scripts/check-doc-harness.sh --mode warn`
2. `scripts/check-architecture-harness.sh --mode warn`
3. `scripts/check-plan-harness.sh --mode strict`
4. `cargo fmt --all --check`
5. `cargo clippy --workspace --all-targets -- -D warnings`
6. `cargo test --workspace`
7. `cd web && npm run build`

## Risks and Rollback

- Risk: integration cutover can create simultaneous breakages across server/web/CLI.
- Risk: doc drift if canonical and root stub updates are not synchronized.
- Rollback: revert stage-4 integration/doc commits while preserving stage-1..3 core runtime work.
