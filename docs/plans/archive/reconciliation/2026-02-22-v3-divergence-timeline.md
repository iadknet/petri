# V3 Divergence Timeline (2026-02-22)

## Chronology (Absolute Timestamps)

| Timestamp | Source | Event |
| --- | --- | --- |
| 2026-02-21T16:23:12-08:00 | git `d0a6237` | Stage scaffolding commit lands (`v3` workspace/crate structure). |
| 2026-02-21T16:26:44-08:00 | git `f5c8877` | Stage 1 implementation commit lands (`contracts`, `config`, `kernel`). |
| 2026-02-21T16:39:07-08:00 | git `163d9f4` | Stage 2 implementation commit lands (`creature` schema/founder/parseability). |
| 2026-02-21T17:23:35-08:00 | git `8066855` | Stage 3a implementation begins (`NodeResult` + `sanitize_f32`). |
| 2026-02-21T20:34:27-08:00 | git `da7405d` | Stage 3a task sequence completed (VM integration tests). |
| 2026-02-21T23:59:36.899Z | Claude history line 315 | Master plan initially written to `.claude/plans/quizzical-forging-lake.md`. |
| 2026-02-22T00:09:36.754Z | Claude history line 325 | Master plan edited to add explicit checkmark-tracking principle. |
| 2026-02-22T00:09:50.999Z | Claude history line 329 | Master plan edited to add status column/checkmarks in stage table. |
| 2026-02-22T08:01:15-08:00 | git `6e17d84` | Stage 3b graph executor commit lands (22 operators, relaxation, atomic state). |
| 2026-02-22T08:13:12-08:00 | git `858cf29` | Stage 3b mesh executor commit lands (mesh chain + soft defaults). |
| 2026-02-22T08:20:51-08:00 | git `076055b` | Docs metadata normalization for stage-3a/3b plans lands. |

## First Divergence Point

Tracking divergence starts at master-plan creation time (`2026-02-21T23:59:36.899Z`):
- Stage 1, Stage 2, and most of Stage 3a implementation commits already existed before the master stage-status table was introduced.
- The master table and stage checklists remained unchecked while implementation continued, so status tracking was stale from inception of that artifact.

## Content Drift (Master vs Implemented Truth)

1. Stage 3b operator count mismatch:
- Historical master text uses "21 operators".
- Implemented and tested code uses 22 (`GraphNodeKind` includes 22 variants).
- Evidence: `docs/plans/archive/2026-02-21-v3-implementation-master-plan.recovered.md`, `v3/crates/v3-core/src/creature/genome.rs`, `docs/plans/2026-02-21-v3-stage-3b-graph-mesh.md`.

2. VM step-cap default mismatch:
- Historical master text cites default `max_vm_steps` as `256`.
- Runtime config and Stage 3a detailed plan specify `1024`.
- Evidence: `docs/plans/archive/2026-02-21-v3-implementation-master-plan.recovered.md`, `v3/crates/v3-core/src/config/simulation.rs`, `docs/plans/2026-02-21-v3-stage-3a-sensors-vm.md`.

## Root-Cause Classification

1. Process drift:
- Checkmarks/status were not updated as commits landed.

2. Document drift:
- Historical master wording diverged from detailed stage plans and code-level truth.

3. Harness blind spot (now corrected):
- Recovered historical master lived in active `docs/plans/` path and was scanned as an active harness target despite being historical evidence.

