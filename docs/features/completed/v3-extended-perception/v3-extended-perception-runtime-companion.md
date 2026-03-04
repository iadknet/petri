# V3 Extended Perception Runtime Companion

**Goal:** Define the frozen perception runtime boundary, config flow, visibility infrastructure, and trace/debug seams required to implement extended perception without breaking V3’s current module ownership model.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Sensor/runtime-facing snapshot types, visibility-table lifecycle, tick integration, `ResolveCtx` changes, and execution-sampler trace/debug seams. Excludes detailed mutation weighting and identity-lifecycle semantics.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-03-01-v3-extended-perception-runtime-companion.md` | Create |
| `docs/plans/2026-03-01-v3-extended-perception-plan.md` | Link as parent/index plan |
| `docs/reference/v3-sensor-spec.md` | Add canonical sensor families, timing, and formulas |
| `docs/reference/v3-mesh-execution-spec.md` | Align runtime entry boundary with `SensorSnapshot` |
| `docs/reference/v3-world-grid-spec.md` | Add `resolve_offset` ownership and local LOS step semantics |
| `docs/reference/v3-runtime-config-spec.md` | Add `runtime.perception.vision_radius` |
| `docs/reference/v3-server-api-protocol-spec.md` | Add sampler debug-perception transport contract |

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-03-01-v3-extended-perception-plan.md`

---

## Goal Alignment

- **GP-01:** The runtime contract supports richer sensing without forcing raw-grid fan-out as the only evolutionary substrate.
- **GP-02:** Frozen world access, visibility policy, runtime input resolution, and server tracing each keep a clear owner.
- **GP-03:** The design keeps hot-path allocations bounded and explicitly testable.
- **GP-04:** Optional perception debug traces make the richer sensor layer inspectable without bloating default traces.

## Boundary Impact

- `SensorSnapshot` becomes the runtime-facing frozen input bundle.
- `PerceptionWorldSnapshot<'a>` becomes the sensor-owned frozen world/export view used during cognition.
- `PerceptionConfig` becomes the only perception-owned config view consumed by reducers.
- `VisibilityTables` become shared sensor infrastructure rather than tick-local data.
- `ResolveCtx` changes, but runtime still never reads live world state directly.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/sensors` | change | Sensors need a richer frozen snapshot boundary, but still own assembly rather than execution. |
| `v3/crates/v3-core/src/runtime/inputs.rs` | change | Input resolution must read a unified sensor bundle once perception compounds exist. |
| `v3/crates/v3-core/src/simulation/tick.rs` | keep | Tick still owns phase orchestration and dispatch order, not LOS or reducer logic. |
| `v3/crates/v3-core/src/kernel/world*` | keep | Kernel remains the owner of geometry/occupancy truth and does not absorb visibility policy. |
| `v3/crates/v3-server/src/handlers/creature.rs` | keep | Server remains the transport owner for trace/debug request flags. |

## Sensor Runtime Overview

The runtime slice introduces one additional frozen layer between world truth and execution:
- `StaticInputs` remains the compact local/radius-1 bundle.
- `PerceptionSnapshot` holds extended-perception summaries and nearby-creature banks.
- `SensorSnapshot` wraps both so runtime can consume one stable input surface.

Normative formulas, field layouts, and visibility rules belong in `docs/reference/v3-sensor-spec.md`.

## New Runtime and Sensor Types

Planned types and responsibilities:
- `SensorSnapshot`: runtime-facing bundle of local and extended sensors
- `PerceptionSnapshot`: fixed-width extended-perception outputs
- `PerceptionGridView<'a>`: immutable frozen access to world grids during cognition
- `PerceptionWorldSnapshot<'a>`: frozen world/export context for assembly
- `PerceptionConfig`: narrow reducer config view, including the sole authoritative `vision_radius`
- `VisibilityTables`: cached local-ray infrastructure keyed by radius

Implementation guardrails to preserve:
- `PerceptionSnapshot::zero()` is a by-value stack/local fallback for non-perception genomes
- `size_of::<PerceptionSnapshot>() <= 256` is a regression guard
- visibility tables must use contiguous storage rather than nested per-ray allocations

## Frozen Snapshot Boundary

The frozen boundary should contain everything extended perception needs:
- immutable world/grid access for food, barriers, and occupancy
- frozen target-creature exports for observed metadata
- no live world queries from runtime input resolution

This keeps the two-phase tick contract intact:
- cognition sees a frozen post-Phase-0 snapshot
- action application still happens later against live current world state

## Config Flow

Perception reducers should read only `PerceptionConfig`, not the full simulation config.

Required fields:
- `vision_radius`
- `max_food_density`
- `max_energy`
- `min_reproduce_energy`

Ownership rule:
- `PerceptionConfig` is the sole radius owner
- `PerceptionWorldSnapshot` and `VisibilityTables` must not introduce competing radius sources of truth

## Visibility Infrastructure

Visibility uses local-offset-space LOS:
- local rays are precomputed per radius
- step resolution uses `resolve_offset`
- barriers are visible and opaque
- food and creatures are visible and non-opaque
- strict-corner blocking applies at diagonal steps

Lifecycle of the shared tables:
- cached process-wide
- keyed by radius
- lazy-initialized on first use
- reused across ticks and simulations
- not stored inside the frozen world snapshot

## Tick Integration

Future implementation should preserve the current two-phase posture:
1. build `PerceptionWorldSnapshot` once from frozen post-Phase-0 state
2. build `PerceptionConfig` once from validated effective config
3. during cognition, assemble local `StaticInputs`
4. conditionally assemble `PerceptionSnapshot` only for genomes that reference extended-perception sensors
5. execute runtime against `SensorSnapshot`

The traced path should follow the same data flow, with only the additional trace capture differing.

## Runtime Resolution Changes

Runtime changes to document and later implement:
- `ResolveCtx` reads `SensorSnapshot`
- local scalar keys read from `sensors.local`
- extended compounds read from `sensors.perception`
- runtime continues to return `0.0` for invalid scalar/compound addressing

Canonical compound widths, `sub_idx` layouts, and absence defaults belong in `docs/reference/v3-sensor-spec.md`.

## Trace and Debug Integration

Default trace payload posture:
- existing `static_inputs` trace payload remains unchanged
- extended perception appears only in an optional debug section
- sampler request decides whether perception debug is recorded

Transport ownership:
- request/response schema belongs in `docs/reference/v3-server-api-protocol-spec.md`
- runtime trace payload structure belongs in the runtime/perception design, not in startup or world-grid docs

## Task List

### Task 1: Define the frozen perception runtime boundary

**Files:**
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: `docs/reference/v3-world-grid-spec.md`
- Modify: this companion

- [ ] Define `SensorSnapshot`, `PerceptionWorldSnapshot<'a>`, and `PerceptionGridView<'a>` at the architecture level.
- [ ] Keep visibility policy in sensors and geometry primitives in world-grid.
- [ ] Keep perception formulas and layouts in the sensor spec, not in this companion.

### Task 2: Define config and visibility infrastructure ownership

**Files:**
- Modify: `docs/reference/v3-runtime-config-spec.md`
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: this companion

- [ ] Make `PerceptionConfig` the only radius owner.
- [ ] Define process-wide cached `VisibilityTables`.
- [ ] Record the contiguous-storage requirement and the no-per-tick-rebuild rule.

### Task 3: Define tick/runtime/trace integration

**Files:**
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: `docs/reference/v3-server-api-protocol-spec.md`
- Modify: this companion

- [ ] Keep tick orchestration responsible only for building frozen inputs and invoking cognition.
- [ ] Keep runtime input resolution world-query-free.
- [ ] Keep perception debug optional in sampler output.

## Docs Impact

| category | docs |
| --- | --- |
| updated canonical references | `docs/reference/v3-sensor-spec.md`, `docs/reference/v3-mesh-execution-spec.md`, `docs/reference/v3-world-grid-spec.md`, `docs/reference/v3-runtime-config-spec.md`, `docs/reference/v3-server-api-protocol-spec.md` |
| companion linkage | this file plus the main plan |
| retired or superseded docs | none |

## Tests

Minimum future test coverage:
- frozen snapshot contains all reads needed for food/barrier/occupancy summaries
- `PerceptionConfig` is the only reducer config dependency
- visibility table cache is reused and does not allocate per ray
- non-perception genomes use `PerceptionSnapshot::zero()` without heap allocation
- traced and untraced execution keep identical behavior when debug perception is disabled

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should runtime resolve extended sensors lazily from live world state? | No. Runtime reads frozen snapshots only. | user+agent | resolved |
| Should visibility tables live in tick-local state? | No. They are shared sensor infrastructure cached by radius. | user+agent | resolved |
| Should perception debug bloat the default execution sampler payload? | No. Keep it request-controlled and optional. | user+agent | resolved |

## Assumptions and Defaults

- default `vision_radius = 5`
- valid radius range `1..=8`
- `PerceptionConfig` is the only perception config view reducers consume
- `PerceptionSnapshot::zero()` is stack/local by value in v1
- `VisibilityTables` are process-wide cached shared infrastructure
- contiguous storage is required for visibility tables
- runtime still never queries live world state during input resolution

## Verification Commands

- `scripts/check-plan-harness.sh --mode strict`
- `scripts/check-doc-harness.sh --mode strict`
- `rg -n "SensorSnapshot|PerceptionSnapshot|PerceptionWorldSnapshot|PerceptionConfig|VisibilityTables|vision_radius" docs/plans/2026-03-01-v3-extended-perception-runtime-companion.md docs/reference/v3-sensor-spec.md docs/reference/v3-world-grid-spec.md docs/reference/v3-runtime-config-spec.md docs/reference/v3-server-api-protocol-spec.md`

## Risks and Rollback

- Risk: runtime and sensor docs reintroduce competing radius ownership.
  - Mitigation: keep `PerceptionConfig` as the sole authoritative radius source.
- Risk: visibility caching is implemented with fragmented heap storage.
  - Mitigation: record contiguous layout as an explicit architectural constraint in this companion and the sensor spec.
- Rollback: revert to a local-only `StaticInputs` plan only if extended perception is explicitly deferred; otherwise keep this runtime companion authoritative.

**Review cycles:** 3
