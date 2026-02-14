# Petri — V2 Program Roadmap

## Guiding Principles

- `v2/` is the active implementation target.
- Legacy runtime and root web are reference-only during this program.
- Checkpoint gates are the source of truth for stage completion.
- Deterministic, test-backed contracts take priority over feature breadth.

## Active Program Structure

Checkpoint authority:
- `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`

Execution stages:
- `CP-0`: `docs/plans/2026-02-13-creature-brain-mesh-stage-1-runtime-kernel.md`
- `CP-1`: `docs/plans/2026-02-13-creature-brain-mesh-stage-2-backends-energy.md`
- `CP-2`: `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`
- `CP-3`: `docs/plans/2026-02-13-creature-brain-mesh-stage-4-cutover.md`

Implementation specs:
- `docs/plans/2026-02-14-v2-cp1-runtime-execution-spec.md`
- `docs/plans/2026-02-14-v2-core-schema-vm-isa-spec.md`
- `docs/plans/2026-02-14-v2-cp2-evolution-ecology-spec.md`
- `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`
- `docs/plans/2026-02-14-v2-frontend-wireframe-spec.md`
- `docs/plans/2026-02-14-v2-frontend-implementation-plan.md`
- `docs/plans/2026-02-14-v2-implementation-test-matrix.md`

## Checkpoint Status Snapshot

| checkpoint | status | summary |
| --- | --- | --- |
| `CP-0` | complete | `v2` workspace and boundary guardrails established. |
| `CP-1` | in progress | runtime kernel/backends semantics remain under active completion. |
| `CP-2` | pending | mutation/ecology checkpoint follows CP-1 exit. |
| `CP-3` | in progress | protocol/test baseline landed across server/cli/web; closeout continues. |

## CP-3 Scope Focus

- Stabilize `v2alpha1` contracts for HTTP, WebSocket, CLI NDJSON, and web decoders.
- Keep paint/lifecycle semantics fixture- and test-locked.
- Complete docs closeout and stale-plan retirement before final exit.

## Done Criteria for Program Close

- All checkpoint gates pass under matrix-defined commands.
- Canonical docs reflect `v2` as the target architecture.
- Compatibility stubs remain pointer-only and consistent.
- Closeout artifact exists at `docs/operations/v2-doc-closeout.md`.
