# V3 World-State Docs Integration Plan

**Goal:** Add a canonical V3 world/grid reference spec and integrate it into
existing V3 reference docs with targeted edits only, so documentation is
implementation-ready without touching code.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Create one new world/grid reference spec, apply minimal ownership
cross-links in four existing reference specs, and validate doc consistency via
harnesses; excludes code changes and broad strategy/roadmap rewrites.
**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/reference/v3-world-grid-spec.md` | Create canonical world/grid config + validity ownership |
| `docs/reference/v3-tick-orchestration-spec.md` | Add targeted ownership links to world/grid spec |
| `docs/reference/v3-sensor-spec.md` | Add targeted world-input ownership link |
| `docs/reference/v3-reproduction-spec.md` | Add targeted spawn-validity and edge-mode ownership links |
| `docs/reference/v3-runtime-config-spec.md` | Clarify world config ownership is outside runtime-config scope |
| `docs/plans/archive/2026-02-21-v3-world-docs-integration.md` | Create docs-only execution and readiness gate plan |

**Supersedes:** none

**Superseded-By:** none

## Goal Alignment

- `GP-01`: protects cognition-focused implementation by making world/grid
  semantics explicit and non-ambiguous for future runtime work.
- `GP-02`: preserves clean boundaries by assigning world/grid ownership to one
  canonical reference.
- `GP-03`: reduces rework risk via harness-validated cross-spec consistency.
- `GP-04`: keeps behavior observable by clarifying move/spawn validity semantics
  used by tick/reproduction flows.

## Boundary Impact

- `v3-world-grid-spec.md` becomes canonical owner for world defaults and spatial
  validity primitives.
- Tick/sensor/reproduction/runtime-config references keep their current
  behavioral contracts and only consume world ownership by link.
- No crate/module implementation boundary changes occur in this plan.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/reference/v3-tick-orchestration-spec.md` | keep | Tick still owns phase order, queue arbitration, and immediate action semantics. |
| `docs/reference/v3-runtime-config-spec.md` | keep | Runtime-config still owns cognition/energy/mutation defaults, not world defaults. |
| `docs/reference/v3-sensor-spec.md` | keep | Sensor categories remain canonical; world neighbor semantics now link to world owner. |
| `docs/reference/v3-reproduction-spec.md` | keep | Reproduction still owns result semantics; world primitive ownership is only referenced. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should this pass rewrite strategy/roadmap docs? | No. Keep this pass minimal and reference-focused. | user+agent | resolved |
| Should this pass touch code? | No. This pass is strictly docs-only. | user+agent | resolved |
| Is world/grid behavior redesign in scope? | No broad redesign; targeted canonicalization and consistency only. | user+agent | resolved |

## Task List

### Task 1: Canonical world/grid spec creation

**Files:**
- Create: `docs/reference/v3-world-grid-spec.md`

- [x] Define canonical world config keys:
  `world.width`, `world.height`, `world.edge_mode`,
  `world.food.growth_rate`, `world.food.initial_density`,
  `world.food.initial_coverage`.
- [x] Define world validity primitives:
  neighbor resolution by edge mode, move target validity, spawn target
  validity.
- [x] Keep runtime/genome/tick ownership out of this doc except cross-links.

### Task 2: Targeted integration in existing reference docs

**Files:**
- Modify: `docs/reference/v3-tick-orchestration-spec.md`
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: `docs/reference/v3-reproduction-spec.md`
- Modify: `docs/reference/v3-runtime-config-spec.md`

- [x] Add `v3-world-grid-spec.md` to related references where relevant.
- [x] Add ownership-link language only; avoid broad narrative rewrite.
- [x] Ensure out-of-bounds and spawn validity wording is consistent across docs.

### Task 3: Readiness gate and scope enforcement

**Files:**
- Create: `docs/plans/archive/2026-02-21-v3-world-docs-integration.md`

- [x] Record docs-only scope and no-code-change rule.
- [x] Include implementation readiness checklist below.

## Implementation Readiness Checklist

- [x] World/grid ownership is canonical in `v3-world-grid-spec.md`.
- [x] Tick/sensor/reproduction/runtime-config specs reference world ownership
      without conflicting semantics.
- [x] Edge-mode behavior (`wrap` vs `bounded`) is consistent across all touched
      docs.
- [x] Move/spawn validity semantics are defined once and referenced elsewhere.
- [x] `scripts/check-doc-harness.sh --mode warn` passes.
- [x] `scripts/check-architecture-harness.sh --mode warn` passes.
- [x] `scripts/check-plan-harness.sh --mode strict` passes.
- [x] Active docs contain no requirement to adapt prior `v3` code as migration
      input.

## Verification Commands

- `scripts/check-doc-harness.sh --mode warn`
- `scripts/check-architecture-harness.sh --mode warn`
- `scripts/check-plan-harness.sh --mode strict`
- `rg -n "v3-world-grid-spec.md" docs/reference/v3-*.md`
- `rg -n "resolve_neighbor|is_valid_spawn_cell|is_valid_move_cell|edge_mode|world\\.width|world\\.height" docs/reference/v3-*.md`

## Verification Results (2026-02-21)

- `scripts/check-doc-harness.sh --mode warn`: pass (`violations=0`)
- `scripts/check-architecture-harness.sh --mode warn`: pass (`violations=0`, baseline warnings only)
- `scripts/check-plan-harness.sh --mode strict`: pass (`violations=0`, `warnings=0`)

## Risks and Rollback

- Risk: terminology drift between world spec and reproduction wording.
  - Mitigation: keep shared primitive names aligned and verify with targeted
    grep checks.
- Risk: scope creep into strategy/code changes.
  - Mitigation: enforce file scope in this plan and keep edits reference-only.
- Rollback: revert only touched reference docs and this plan file to prior
  revision; re-apply minimal integration edits.

**Review cycles:** 1

Cycle 1: Narrowed reset effort to docs-only world-state integration and
readiness gating, with no strategy rewrite and no code edits.
