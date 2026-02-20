# Petri - V3 Target Architecture

## Purpose

This document defines the active target architecture for the V3 mesh runtime.
Legacy root crates, v1/v2 docs, and root `web/` are retained as historical
reference only.

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`, `GP-04`

## Goal Alignment

- `GP-01`: richer creature cognition via multi-node mesh execution.
- `GP-02`: explicit module boundaries across kernel, sensors, creature, runtime,
  tick, and contracts.
- `GP-03`: test-reproducible, high-confidence iteration with crash-proof soft defaults.
- `GP-04`: observable behavior and runtime introspection suitable for evolution
  debugging.

Determinism scope is canonical in `AGENTS.md` (`Determinism Scope (Canonical)`).

## Boundary Impact

Active dependency direction in V3 core:
- `kernel` -> world reality only
- `sensors` -> world/static snapshots
- `creature` -> genome/state/mutation/reproduction
- `runtime` -> mesh/VM/graph execution
- `tick` -> orchestration and action application
- `contracts` -> shared boundary types (`WorldAction` and related)

No dependency from active V3 docs into legacy implementation contracts.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/kernel` | keep | world reality independent of cognition backend details |
| `v3/crates/v3-core/src/sensors` | keep | snapshot assembly separated from runtime mutation logic |
| `v3/crates/v3-core/src/runtime` | keep | owns chain evaluation, routing, VM/graph execution |
| `v3/crates/v3-core/src/tick` | keep | phase orchestration and action application stay outside runtime internals |
| legacy `crates/petri-*`, `v2/`, root `web/` | keep | traceability without active coupling |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Are there unresolved architecture-direction questions for the active V3 target? | No unresolved direction questions at this time. | user+agent | resolved |
| Where is determinism policy canonical? | Root `AGENTS.md` (`Determinism Scope (Canonical)`). | user+agent | resolved |

## Repository Architecture (Active Slice)

```text
petri/
|- v3/
|  |- crates/
|  |  \- v3-core/
|  |     |- kernel/
|  |     |- sensors/
|  |     |- creature/
|  |     |- runtime/
|  |     |- tick/
|  |     \- contracts/
|  \- crates/v3-server/    # service surfaces over v3-core
|- docs/                   # canonical strategy/plans/reference docs
|- crates/                 # legacy runtime stacks (reference-only)
|- v2/                     # legacy rewrite program (reference-only)
\- web/                    # legacy root web client (reference-only)
```

## Runtime Truthfulness Invariant

- Reported runtime values must derive from applied simulation behavior.
- No synthetic values in state/telemetry surfaces where behavior-backed values
  are expected.
- Applies to current and future runtime-facing APIs and inspectors.

## Compatibility Posture

- Root compatibility stubs remain pointer-only:
  - `petri-architecture.md`
  - `petri-roadmap.md`
  - `petri-technology-review.md`
- Legacy implementation/docs are retained for traceability, not as active
  targets.
