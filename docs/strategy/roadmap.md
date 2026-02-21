# Petri - V3 Program Roadmap

## Guiding Principles

- V3 mesh architecture is the active implementation target.
- Legacy v1/v2 stacks are reference-only during this program.
- Checkpoint gates and reference specs are the source of truth for completion.
- Test-reproducible, high-confidence contracts take priority over feature breadth.

## Active Program Structure

Primary architecture/design planning:
- `docs/plans/` (active plan set)

Canonical runtime/contracts specs:
- `docs/reference/` (active V3 specs)

## Current Program Status

| stage | status | summary |
| --- | --- | --- |
| Architecture reconciliation | complete | Mesh execution model and soft-default philosophy stabilized in docs. |
| Reference specification sync | complete | Execution/tick-orchestration/runtime-config, VM, graph, genome, sensor, lifecycle, mutation, reproduction, and observability specs aligned. |
| Legacy doc archival | complete | Superseded V3 plans and legacy refs archived/pointerized. |
| Implementation planning | next | Convert canonical docs into executable milestone plans and task slices. |

## Current Focus

- Preserve strict abstraction boundaries while implementing mesh runtime.
- Enforce crash-proof evaluation under junk-DNA mutation behavior.
- Keep test reproducibility guarantees explicit and harness-backed.

Determinism scope is canonical in `AGENTS.md` (`Determinism Scope (Canonical)`).

## Done Criteria for Current Documentation Phase

- No active-doc conflicts on target architecture direction.
- Canonical references are internally consistent.
- Superseded references point to canonical replacements.
- Active plan and strategy docs agree on V3 as target.
