# Petri - V3 Program Roadmap

## Guiding Principles

- V3 mesh architecture is the active implementation target.
- Legacy v1/v2 stacks are reference-only during this program.
- Checkpoint gates and reference specs are the source of truth for completion.
- Deterministic, test-backed contracts take priority over feature breadth.

## Active Program Structure

Primary architecture/design plan:
- `docs/plans/2026-02-18-v3-mesh-refactor-design.md`

Canonical reference specs:
- `docs/reference/v3-mesh-execution-spec.md`
- `docs/reference/v3-genome-spec.md`
- `docs/reference/v3-sensor-spec.md`
- `docs/reference/v3-graph-backend-spec.md`
- `docs/reference/v3-vm-isa-spec.md`
- `docs/reference/v3-creature-lifecycle-spec.md`

## Current Program Status

| stage | status | summary |
| --- | --- | --- |
| Architecture reconciliation | complete | Mesh execution model and soft-default philosophy stabilized in docs. |
| Reference specification sync | complete | VM, graph, genome, sensor, and lifecycle specs aligned. |
| Legacy doc archival | complete | Superseded V3 plans and legacy refs archived/pointerized. |
| Implementation planning | next | Convert canonical docs into executable milestone plans and task slices. |

## Current Focus

- Preserve strict abstraction boundaries while implementing mesh runtime.
- Enforce crash-proof evaluation under junk-DNA mutation behavior.
- Keep determinism guarantees explicit and test-backed.

## Done Criteria for Current Documentation Phase

- No active-doc conflicts on target architecture direction.
- Canonical references are internally consistent.
- Superseded references point to canonical replacements.
- Active plan and strategy docs agree on V3 as target.
