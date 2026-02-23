# Petri - V3 Program Roadmap

## Guiding Principles

- V3 mesh architecture is the active implementation target.
- Checkpoint gates and reference specs are the source of truth for completion.
- Test-reproducible, high-confidence contracts take priority over feature breadth.

## Active Program Structure

Primary architecture/design planning:
- `docs/strategy/` (active strategy/design direction)
- `docs/plans/` (active plan set for new work)

Canonical runtime/contracts specs:
- `docs/reference/` (active V3 specs)

Archived implementation planning history:
- `docs/plans/archive/`

## Current Program Status

| stage | status | summary |
| --- | --- | --- |
| Architecture reconciliation | complete | Mesh execution model and soft-default philosophy stabilized in docs. |
| Reference specification sync | complete | Execution/tick-orchestration/runtime-config, VM, graph, genome, sensor, lifecycle, mutation, reproduction, and observability specs aligned. |
| Documentation archival | complete | Non-active docs moved to archive locations; archived-marked docs removed. |
| Runtime implementation | next | Continue execution against active strategy and reference specs. |

## Current Focus

- Preserve strict abstraction boundaries while implementing mesh runtime.
- Enforce crash-proof evaluation under junk-DNA mutation behavior.
- Keep test reproducibility guarantees explicit and harness-backed.

Determinism scope is canonical in `AGENTS.md` (`Determinism Scope (Canonical)`).

## Done Criteria for Current Documentation Phase

- No active-doc conflicts on target architecture direction.
- Canonical references are internally consistent.
- Superseded references point to canonical replacements.
- Active strategy, reference, and plan docs agree on V3 as target.
