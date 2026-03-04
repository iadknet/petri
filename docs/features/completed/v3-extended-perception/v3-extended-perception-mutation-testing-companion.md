# V3 Extended Perception Mutation and Testing Companion

**Goal:** Define how extended-perception sensors become reachable through mutation, how graph fan-out should behave, and what verification matrix is required before implementation can be called complete.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Input-ref mutation reachability, compound fan-out posture, test coverage, perf regression checks, and acceptance criteria. Excludes deep runtime boundary design and identity-lifecycle semantics.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-03-01-v3-extended-perception-mutation-testing-companion.md` | Create |
| `docs/plans/2026-03-01-v3-extended-perception-plan.md` | Link as parent/index plan |
| `docs/reference/v3-sensor-spec.md` | Add canonical compound widths and new sensor families |

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-03-01-v3-extended-perception-plan.md`

---

## Goal Alignment

- **GP-01:** Mutation reachability ensures evolution can actually discover the new sensing surface.
- **GP-02:** Graph fan-out stays within the existing mutation/runtime ownership model instead of introducing a backend-specific side path.
- **GP-03:** The feature has an explicit verification matrix, including perf guards and snapshot correctness tests.
- **GP-04:** Trace/debug and reducer tests make the new sensing layer inspectable rather than opaque.

## Boundary Impact

- `mutation/input_ref` owns random acquisition of the new sensor families.
- `mutation/compound` owns the compound widths that drive graph fan-out.
- `runtime/` and `sensors/` remain consumers of those contracts rather than mutation owners.
- Test ownership spans unit, integration, and perf smoke coverage, but the canonical sensor widths still live in the sensor spec.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/mutation/input_ref` | change | Random input acquisition must include the new sensor families or the feature is unreachable. |
| `v3/crates/v3-core/src/mutation/compound` | keep | Existing `sub_value_count() -> create_fan_out_nodes()` remains the correct graph fan-out path. |
| `v3/crates/v3-core/src/runtime` | keep | Runtime should not gain a mutation-specific workaround for compound sensors. |
| `docs/reference/v3-sensor-spec.md` | keep | Compound widths and addressing stay canonical there, not in mutation docs. |

## Mutation Reachability Overview

Extended perception is only useful if mutation can discover it. The mutation surface therefore has to grow deliberately:
- add the six new sensor families to the input-ref pool
- keep them conservative rather than dominant
- preserve the existing backend-neutral fan-out path for graph genomes

This companion keeps the mutation weighting and test expectations together so they do not get lost in the runtime companion.

## Input-Ref Mutation Surface

Planned v1 distribution change:
- current pool size `38`
- new pool size `44`
- each new sensor family gets exactly one slot
- existing weights stay unchanged

New sensor families entering the pool:
- `AreaFoodSummary`
- `AreaBarrierSummary`
- `AreaOccupancySummary`
- `NearbyCreatureCore`
- `NearbyCreatureVitals`
- `NearbyCreatureIdentity`

Conservative weighting keeps radius-1 local sensors as the dominant random discovery path for simple genomes while still making extended perception reachable.

## Compound Fan-Out Rules

Graph fan-out remains automatic through the existing path:
1. mutate or replace an `InputReference`
2. call `sub_value_count(reference, config)`
3. if the count is greater than `1`, call `create_fan_out_nodes(...)`

Design consequences:
- no new graph-only mutation path is added
- VM retains compound addressing without auto-fan-out
- compound widths for the new sensor families must stay canonical in `docs/reference/v3-sensor-spec.md`

## Task List

### Task 1: Define mutation reachability in the plan/spec package

**Files:**
- Modify: this companion
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: `docs/plans/2026-03-01-v3-extended-perception-plan.md`

- [ ] Record the pool expansion from `38` to `44`.
- [ ] Record the one-slot-per-new-family conservative weighting.
- [ ] Keep widths canonical in the sensor spec rather than copying them here repeatedly.

### Task 2: Define graph fan-out and VM posture

**Files:**
- Modify: this companion
- Modify: `docs/reference/v3-sensor-spec.md`

- [ ] Keep graph fan-out on the existing compound path.
- [ ] Keep VM on the existing compound-addressing path.
- [ ] Avoid backend-specific extended-perception mutation rules.

### Task 3: Define the verification matrix

**Files:**
- Modify: this companion
- Modify: the main plan
- Modify: the runtime companion
- Modify: the identity companion

- [ ] Capture identity, snapshot, visibility, reducer, runtime, trace, mutation, and perf checks.
- [ ] Keep verification commands explicit.
- [ ] Keep acceptance criteria aligned with the split-plan package.

## Docs Impact

| category | docs |
| --- | --- |
| updated canonical references | `docs/reference/v3-sensor-spec.md` |
| companion linkage | this file plus the main plan and runtime companion |
| retired or superseded docs | none |

## Test Matrix

Minimum future coverage:
- identity seeding/inheritance/mutation-trigger tests
- frozen snapshot correctness tests
- config-flow tests for `PerceptionConfig`
- visibility-table lifecycle and cache-layout tests
- conditional-assembly tests
- LOS and strict-corner tests
- reducer correctness tests
- nearby-ranking tests
- VM and graph runtime integration tests
- mutation reachability and graph fan-out tests
- trace/debug transport tests
- perf/regression smoke tests
- `size_of::<PerceptionSnapshot>() <= 256` guard

## Performance and Regression Checks

Required hot-path constraints to preserve:
- no per-creature heap allocation during the LOS scan
- no per-ray heap allocation inside shared visibility tables
- conditional skip for genomes that do not reference extended perception
- frozen export build should remain linear in creature count
- runtime input resolution should not become the bottleneck once perception is assembled

## Acceptance Criteria

This companion is satisfied when:
- the new sensor families are reachable through mutation
- graph fan-out remains on the existing compound path
- the test matrix covers identity, visibility, reducers, runtime, transport, and perf
- canonical widths are defined once in the sensor spec
- no verification requirement depends on tribal knowledge outside the plan package

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should extended-perception variants receive extra mutation weight because they are more expressive? | No for v1. Keep them conservative and observe ecology first. | user+agent | resolved |
| Should graph get a backend-specific shortcut instead of compound fan-out? | No. Keep the existing compound path and revisit only if profiling forces a redesign. | user+agent | resolved |
| Should perf checks stop at unit tests? | No. Include at least one regression/perf smoke path covering large-crowd perception assembly. | user+agent | resolved |

## Assumptions and Defaults

- mutation pool expands from `38` to `44`
- each new extended-perception family gets one slot
- existing mutation weights otherwise stay unchanged
- graph fan-out continues to use `sub_value_count() -> create_fan_out_nodes()`
- VM continues to use compound addressing without auto-fan-out

## Verification Commands

- `scripts/check-plan-harness.sh --mode strict`
- `scripts/check-doc-harness.sh --mode strict`
- `rg -n "38|44|AreaFoodSummary|NearbyCreatureIdentity|fan-out|sub_value_count" docs/plans/2026-03-01-v3-extended-perception-mutation-testing-companion.md docs/reference/v3-sensor-spec.md`

## Risks and Rollback

- Risk: the new sensors are spec'd but unreachable through mutation.
  - Mitigation: keep the mutation-pool expansion explicit in this companion and in the eventual implementation checklist.
- Risk: graph fan-out rules drift from the canonical compound widths.
  - Mitigation: keep widths owned by the sensor spec and reference them here instead of duplicating them everywhere.
- Rollback: if extended perception is postponed entirely, archive this companion together with the main plan rather than leaving a stale active mutation surface behind.

**Review cycles:** 3
