# Petri V2 Greenfield Program (Fresh Start)

> **Program Type:** This is a **MAJOR GREENFIELD REWRITE** program. Existing runtime code is reference-only.

**Goal:** Build a new `v2` simulation app from scratch, optimized for mesh-DNA cognition and emergent behavior, without refactoring or integrating legacy runtime code.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Create a fully isolated app under `v2/` (`core`, `server`, `cli`, `web`) plus greenfield planning docs; excludes in-place edits to legacy runtime/server/web codepaths.
**Docs Impact:** Rewrite this program and stage plans around greenfield `v2`; add `v2` boundary docs and remove legacy refactor framing from active plans.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: greenfield architecture removes legacy constraints and focuses directly on richer creature cognition.
- `GP-02`: clean boundaries are easier to enforce in a new app root than through incremental refactor.
- `GP-03`: isolated workspace/tooling enables stable TDD without regression drag from legacy behavior.
- `GP-04`: observability can be designed in from day one rather than retrofitted.

## Boundary Impact

- Dependency direction in `v2` is `v2-core -> v2-server/v2-cli`; `v2-web` consumes `v2-server` contracts only.
- Legacy crates (`petri-core`, `petri-server`, `petri-cli`, `web`) remain untouched implementation-wise and are not linked into `v2`.
- Shared logic reuse is `copy-only`: if code is reused, copy into `v2` and adapt locally; no cross-root imports.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/*` legacy runtime | keep | Legacy app remains as historical/reference artifact while `v2` is built in isolation. |
| `docs/plans/2026-02-13-creature-brain-mesh-stage-*.md` | change | Stage plans must describe greenfield build tasks, not refactor-in-place tasks. |
| `v2/` (new root) | change | New code owner boundary for all runtime/application implementation work. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should `v2` live in this repository or a separate repo? | In this repo under `v2/`, with isolated tooling and boundaries. | user+agent | resolved |
| Should `v2` launch with full app skeleton or core-only? | Full skeleton (`core`, `server`, `cli`, `web`) from day one. | user+agent | resolved |
| Should old code be integrated as dependencies? | No; copy-only when needed. | user+agent | resolved |

## Stage Breakdown

### Stage 1: Foundation and Guardrails

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-1-runtime-kernel.md`

Deliverables:
1. `v2/` workspace and crate/app skeleton with independent toolchain manifests.
2. Guardrail docs and static checks establishing copy-only/no-legacy-import policy.
3. Minimal executable app loop proving isolated end-to-end wiring.

### Stage 2: Mesh Runtime Kernel and Backends

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`

Deliverables:
1. Mesh genome schema and packet FIFO runtime in `v2-core`.
2. Graph and VM backend contracts with explicit energy metering.
3. Routing/action semantics tests in greenfield core test suite.

### Stage 3: Evolution and Ecology Engine

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`

Deliverables:
1. Strong asexual mutation engine with duplication operators.
2. Ecology pressure systems tuned for novelty-through-environment.
3. Baseline non-collapse integration scenarios for `v2`.

### Stage 4: Product Surface and Stabilization

Plan file: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

Deliverables:
1. `v2-server` APIs and `v2-web` protocol/UI alignment.
2. `v2-cli` experiment/ablation/reporting flows for fast play iteration.
3. Canonical docs that point to `v2` as active architecture target.

## Program Checkpoint Boundaries

| checkpoint | boundary trigger | required evidence | go / stop rule |
| --- | --- | --- | --- |
| `CP-0 Greenfield Bootstrap` | Before stage 2 | `v2` skeleton, guardrails doc, and stage-1 verification pass | Stop if any `v2` crate imports legacy crates |
| `CP-1 Runtime Kernel Freeze` | After stage 2 | Kernel semantics tests green and stable API boundaries | Stop if queue/action semantics still changing across runs |
| `CP-2 Evolution Baseline` | After stage 3 | Mutation invariants + ecology non-collapse checks green | Stop if baseline runs collapse or mutation invariants fail |
| `CP-3 Product Surface Gate` | Before program close | `v2-server/web/cli` integration tests and build gates green | Stop if protocol or behavior mismatches remain |

## Cross-Stage Handoff Rules

1. Implementation edits for this program occur under `v2/` only, except plan/doc updates.
2. Any intentional code reuse from legacy must be copied into `v2` and recorded in stage notes.
3. Stage starts require previous checkpoint `go` evidence in commit notes.
4. If scope expands, update relevant stage plan first.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `scripts/check-doc-harness.sh --mode warn`
3. `scripts/check-architecture-harness.sh --mode warn`
4. Stage-local `v2` commands defined in each stage plan

## Risks and Rollback

- Risk: duplicated code in early greenfield stages can drift from legacy behavior unexpectedly.
- Risk: maintaining two app roots temporarily raises documentation burden.
- Risk: premature optimization in `v2` can delay proving core semantics.
- Rollback strategy:
1. Keep each stage atomic and independently reversible.
2. If a stage fails checkpoint criteria, revert that stage and re-plan rather than patching legacy paths.
