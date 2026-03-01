# V3 Extended Perception Plan Package

**Goal:** Replace the monolithic extended-perception design draft with a split plan package plus canonical reference-spec updates that preserve V3 boundary ownership and give implementation a stable source of truth.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Create one main plan plus three companion plans, add the missing creature-identity reference spec, and move normative extended-perception rules into active V3 reference docs. Excludes Rust implementation of the feature itself.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-03-01-v3-extended-perception-plan.md` | Create main coordination/index plan |
| `docs/plans/2026-03-01-v3-extended-perception-identity-companion.md` | Create identity-domain companion |
| `docs/plans/2026-03-01-v3-extended-perception-runtime-companion.md` | Create runtime/perception companion |
| `docs/plans/2026-03-01-v3-extended-perception-mutation-testing-companion.md` | Create mutation/testing companion |
| `docs/reference/v3-creature-identity-spec.md` | Create canonical creature identity contract |
| `docs/reference/v3-sensor-spec.md` | Add canonical extended-perception sensor contract |
| `docs/reference/v3-mesh-execution-spec.md` | Align runtime execution boundary with `SensorSnapshot` |
| `docs/reference/v3-world-grid-spec.md` | Add `resolve_offset` ownership and local-offset geometry rules |
| `docs/reference/v3-reproduction-spec.md` | Add identity inheritance trigger ownership |
| `docs/reference/v3-startup-seeding-spec.md` | Add founder identity baseline linkage |
| `docs/reference/v3-creature-lifecycle-spec.md` | Add lifecycle-level identity summary |
| `docs/reference/v3-runtime-config-spec.md` | Add `runtime.perception.vision_radius` |
| `docs/reference/v3-server-api-protocol-spec.md` | Add execution-sampler perception-debug contract |

**Supersedes:** none

**Superseded-By:** none

**See also:**
- `docs/plans/2026-03-01-v3-extended-perception-identity-companion.md`
- `docs/plans/2026-03-01-v3-extended-perception-runtime-companion.md`
- `docs/plans/2026-03-01-v3-extended-perception-mutation-testing-companion.md`

---

## Goal Alignment

- **GP-01:** Extended perception expands creature decision inputs beyond radius-1 local sensors without forcing a raw-grid-only evolution surface.
- **GP-02:** The split package makes ownership explicit across `creature`, `sensors`, `runtime`, `tick`, `world-grid`, and server transport.
- **GP-03:** Moving normative details into reference specs reduces drift and gives future implementation/testing one canonical contract per concern.
- **GP-04:** The package includes explicit trace/debug ownership and server-protocol updates so richer sensing remains observable.

## Boundary Impact

- No new crate boundaries are introduced.
- `creature/` gains ownership of identity state and inheritance semantics.
- `sensors/` remains the owner of frozen perception assembly and visibility policy.
- `kernel/world-grid` remains the owner of coordinate and edge-mode geometry primitives only.
- `runtime/` remains the owner of input resolution and execution tracing, but now consumes `SensorSnapshot` rather than a local-only sensor bundle.
- `v3-server` remains the owner of HTTP transport, including any execution-sampler request/response changes.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/sensors` | keep | Snapshot assembly stays separate from runtime execution and from world mutation logic. |
| `v3/crates/v3-core/src/kernel` | keep | Geometry and occupancy truth stay in the world layer rather than leaking into sensors or runtime. |
| `v3/crates/v3-core/src/creature` | change | Creature identity becomes a first-class lifecycle concern instead of a sensor-local implementation detail. |
| `v3/crates/v3-core/src/runtime` | keep | Runtime still resolves already-frozen inputs and owns tracing, not world queries. |
| `v3/crates/v3-server` | keep | Protocol-level sampler changes stay transport-owned and do not move into core runtime docs. |

## Problem Statement

The earlier extended-perception writeup mixed four different concerns into one oversized document:
- implementation sequencing,
- architecture ownership,
- canonical sensor formulas,
- future testing/perf expectations.

That structure conflicted with the repository plan-splitting policy and created a second, drifting source of truth alongside the active reference specs. This package replaces that draft with one coordination plan, three focused companions, and updated reference docs that own the normative rules.

## Locked Product Decisions

- Extended perception uses summary sensors plus ranked nearby-creature detail banks rather than raw per-cell grids as the canonical v1 contract.
- Barriers are visible and opaque; food and creatures are visible and non-opaque.
- Visibility is computed in local-offset space with strict-corner blocking.
- `runtime.perception.vision_radius` is global for the simulation run; default `5`, valid range `1..=8`.
- Creature identity uses `lineage_id` plus `kin_tag`; raw identity values are internal and sensor-facing logic consumes bounded derived values only.
- Extended perception is assembled conditionally for genomes that reference the new sensor families.
- Visibility tables are shared process-wide infrastructure keyed by radius.
- Execution-sampler perception debugging is optional and request-controlled.

## Architecture Ownership Map

| concern | owner | reference target |
| --- | --- | --- |
| identity state and inheritance | `creature/` | `docs/reference/v3-creature-identity-spec.md` |
| world/grid geometry primitives | `kernel/world-grid` | `docs/reference/v3-world-grid-spec.md` |
| visibility policy and perception assembly | `sensors/` | `docs/reference/v3-sensor-spec.md` |
| runtime config key/default for radius | runtime config contract | `docs/reference/v3-runtime-config-spec.md` |
| runtime input resolution and trace payloads | `runtime/` | `docs/reference/v3-sensor-spec.md`, `docs/reference/v3-server-api-protocol-spec.md` |
| mutation reachability and graph fan-out | `mutation/` | `docs/reference/v3-sensor-spec.md` plus mutation/testing companion |

## Public API and Type Changes

The eventual implementation described by this package introduces or changes:
- `WorldInputKey::{AreaFoodSummary, AreaBarrierSummary, AreaOccupancySummary, NearbyCreatureCore, NearbyCreatureVitals, NearbyCreatureIdentity}`
- `CreatureState.identity: CreatureIdentityState`
- `SensorSnapshot`
- `PerceptionSnapshot`
- `PerceptionWorldSnapshot<'a>`
- `PerceptionGridView<'a>`
- `PerceptionConfig`
- `VisibilityTables`
- `ResolveCtx` reading from `SensorSnapshot`
- `ActiveTrace.include_perception_debug`
- sampler request field `include_perception_debug`
- runtime config field `runtime.perception.vision_radius`

## Implementation Slices

1. **Reference-spec groundwork:** add the missing identity spec and move normative extended-perception rules into the active references.
2. **Identity domain:** introduce lifecycle-owned identity state, founder initialization, and inheritance trigger ownership.
3. **Perception runtime:** document the frozen snapshot boundary, config flow, visibility tables, tick integration, and trace/debug seams.
4. **Mutation and verification:** document evolvability, graph fan-out behavior, test coverage, and perf checks.

## Sequencing and Dependencies

1. Reference-spec groundwork
   - add `v3-creature-identity-spec.md`
   - update sensor/world-grid/reproduction/seeding/runtime-config docs
2. Identity domain implementation
3. Perception snapshot and visibility infrastructure
4. Runtime resolution and trace integration
5. Mutation reachability changes
6. Test and perf verification
7. Doc harness / architecture harness / plan harness verification

## Task List

### Task 1: Create the split plan package

**Files:**
- Create: `docs/plans/2026-03-01-v3-extended-perception-plan.md`
- Create: `docs/plans/2026-03-01-v3-extended-perception-identity-companion.md`
- Create: `docs/plans/2026-03-01-v3-extended-perception-runtime-companion.md`
- Create: `docs/plans/2026-03-01-v3-extended-perception-mutation-testing-companion.md`

- [ ] Keep this main file concise and index-like.
- [ ] Keep each companion focused on one domain slice.
- [ ] Ensure `See also` and `Parent plan` links are bidirectional and stable.

### Task 2: Move normative details into canonical specs

**Files:**
- Create: `docs/reference/v3-creature-identity-spec.md`
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: `docs/reference/v3-world-grid-spec.md`
- Modify: `docs/reference/v3-reproduction-spec.md`
- Modify: `docs/reference/v3-startup-seeding-spec.md`
- Modify: `docs/reference/v3-creature-lifecycle-spec.md`
- Modify: `docs/reference/v3-runtime-config-spec.md`
- Modify: `docs/reference/v3-server-api-protocol-spec.md`

- [ ] Put sensor widths, `sub_idx` layouts, formulas, and visibility rules in the reference specs.
- [ ] Keep plan files focused on ownership, sequencing, verification, and rollout.
- [ ] Add cross-spec links wherever canonical ownership changed.

### Task 3: Preserve future implementation order

**Files:**
- Modify: this main plan
- Modify: all three companion plans

- [ ] Keep the seven-step sequencing order canonical in the main plan.
- [ ] Keep future code-file targets explicit in the companion plans.
- [ ] Keep verification commands and rollback notes present in every active plan file.

## Docs Impact

| category | docs |
| --- | --- |
| new active plan files | all four `docs/plans/2026-03-01-v3-extended-perception-*.md` files |
| new canonical reference | `docs/reference/v3-creature-identity-spec.md` |
| updated canonical references | sensor, mesh-execution, world-grid, reproduction, startup-seeding, creature-lifecycle, runtime-config, server-api docs |
| retired or superseded docs | none |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should the split leave deep sensor tables in the main plan? | No. Put normative tables and formulas in `docs/reference/` only. | user+agent | resolved |
| Does the split change crate boundaries? | No. It clarifies ownership but does not introduce new crates or dependency directions. | user+agent | resolved |
| Should plan files document future code-file targets even before implementation starts? | Yes. Active plans must remain executable by an implementer with little local context. | user+agent | resolved |

## Verification Commands

- `scripts/check-plan-harness.sh --mode strict`
- `scripts/check-doc-harness.sh --mode strict`
- `scripts/check-architecture-harness.sh --mode strict`
- `rg -n "See also|Parent plan|Review cycles" docs/plans/2026-03-01-v3-extended-perception-*.md`

## Risks and Rollback

- Risk: the main plan stays too large and becomes another shadow spec.
  - Mitigation: keep normative formulas, widths, and request/response details in the reference docs only.
- Risk: ownership language drifts between companion plans and active references.
  - Mitigation: each companion explicitly references its canonical spec owners and keeps only migration/order guidance locally.
- Rollback: remove the new active plan files and restore a single archived draft only if the split proves unmaintainable. Do not keep both active at once.

**Review cycles:** 3
