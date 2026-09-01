# Petri - V3 Target Architecture

## Purpose

This document defines the active target architecture for the V3 mesh runtime.

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`, `GP-04`

## Goal Alignment

- `GP-01`: richer creature cognition via multi-node mesh execution.
- `GP-02`: explicit module boundaries across kernel, sensors, creature, runtime,
  tick, and contracts.
- `GP-03`: test-reproducible, high-confidence iteration with crash-proof soft defaults.
- `GP-04`: observable behavior and runtime introspection suitable for evolution
  debugging.

Determinism scope is canonical in root `AGENTS.md`.

## Boundary Impact

Active dependency direction in V3 core:
- `kernel` -> world reality only
- `sensors` -> world/static snapshots
- `creature` -> genome/state/mutation/reproduction
- `runtime` -> mesh/VM/graph execution
- `tick` -> orchestration and action application
- `contracts` -> shared boundary types (`WorldAction` and related)

Active service/UI dependency direction for the viewport transport refactor:
- `v3-core` -> authoritative simulation behavior only
- `v3-server::command` -> mutates and advances `v3-core`
- `v3-server::query` -> reads command-published projection state only
- `v3-server::transport` -> assembles and delivers view payloads from query state only
- `v3-server::http` -> validates requests and delegates to command/query/transport services
- `frontend viewport state` -> computes desired view rect and fidelity tier
- `frontend world-view state` -> stores static world state plus the latest accepted view payload
- `frontend renderer` -> draws from a render model and camera state only

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/v3-core/src/kernel` | keep | world reality independent of cognition backend details |
| `crates/v3-core/src/sensors` | keep | snapshot assembly separated from runtime mutation logic |
| `crates/v3-core/src/runtime` | keep | owns chain evaluation, routing, VM/graph execution |
| `crates/v3-core/src/simulation/tick` | keep | phase orchestration and action application stay outside runtime internals |
| `crates/v3-server` | change | server command/query/transport boundaries are now explicit instead of living in a flat transport shell |
| `frontend/src` viewport transport path | change | viewport state, world-view state, and renderer input boundaries are now first-class |
| `docs/reference/*.md` | keep | executable contracts stay centralized under active V3 reference specs |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Are there unresolved architecture-direction questions for the active V3 target? | No unresolved direction questions at this time. | user+agent | resolved |
| Where is determinism policy canonical? | Root `AGENTS.md`. | user+agent | resolved |

## Repository Architecture (Active Slice)

```text
petri/
|- Cargo.toml               # root virtual workspace
|- crates/
|  |- v3-core/
|  |  \- src/
|  |     |- kernel/
|  |     |- sensors/
|  |     |- creature/
|  |     |- runtime/
|  |     |- simulation/tick/
|  |     \- contracts/
|  |- v3-server/            # command/query/transport surfaces over v3-core
|  \- v3-cli/               # command-line surfaces over v3-core
|- frontend/                # viewport/world-view/render client
\- docs/                    # canonical strategy, reference, and PRD docs
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
