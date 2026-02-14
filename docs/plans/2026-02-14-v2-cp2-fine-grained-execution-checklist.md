# Petri V2 CP-2 Fine-Grained Execution Checklist

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Execute CP-2 mutation/ecology work via atomic, checklisted slices so each behavioral change is verified before moving to the next.
**Goal IDs:** GP-01, GP-03, GP-04
**Scope:** `v2/crates/v2-core` evolution, reproduction-memory inheritance, ecology pressures, and non-collapse gates.
**Docs Impact:** Adds a checklist-driven execution plan for CP-2 that operationalizes the CP-2 spec and test matrix.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: increases behavior search-space quality through mutation operators and ecology pressures.
- `GP-03`: uses strict red-green slices to avoid mutation/ecology regressions.
- `GP-04`: keeps ecology health metrics and non-collapse assumptions test-backed.

## Boundary Impact

- Work stays inside `v2-core` (`evolution/*`, `ecology/*`, and related tests).
- No transport or UI ownership shifts.
- Runtime schema/ISA boundaries remain unchanged.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core/src/evolution/*` | change | CP-2 behavior must be implemented and locked via invariant tests. |
| `v2/crates/v2-core/src/ecology/*` | change | Ecology pressure behavior should be tuned and regression-protected. |
| `v2/crates/v2-core/tests/*` | change | CP-2 requires deterministic, seed-fixed invariants/non-collapse checks. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should CP-2 slices combine mutation and ecology changes in one commit? | No; mutation and ecology slices should remain separate. | user+agent | resolved |
| Should offspring memory inheritance remain mandatory? | Yes; byte-for-byte copy at reproduction commit is required. | user+agent | resolved |
| Should CP-2 close without non-collapse evidence? | No; CP-2 closes only when non-collapse gate is green. | user+agent | resolved |

## Checklist Operating Rules

1. Keep one slice active at a time.
2. Use failing deterministic tests when behavior is missing; if behavior already meets intent, harden regression coverage and capture pass evidence.
3. Use fixed seeds in all mutation/ecology behavior tests.
4. Run only v2 tests.
5. If a checklist step conflicts with architecture boundaries or project goals, stop and ask for guidance.
6. Update checklist boxes in the same commit as code.

## Task List

### Slice E1: Mutation default contract lock

**Files:**
- Modify: `v2/crates/v2-core/tests/mutation_invariants.rs`
- Modify: `v2/crates/v2-core/src/evolution/config.rs`

**Checklist:**
- [x] Add/adjust regression test coverage for full CP-2 mutation default values (including per-weight defaults).
- [x] Run: `cd v2 && cargo test -p v2-core --test mutation_invariants` and confirm default-contract coverage.
- [x] Verify `evolution/config.rs` default values and validation align with CP-2 spec.
- [x] Re-run targeted test and confirm pass.
- [x] Commit slice `E1`.

### Slice E2: Core mutation operators (add/remove/retarget)

**Files:**
- Modify: `v2/crates/v2-core/src/evolution/operators.rs`
- Modify: `v2/crates/v2-core/tests/mutation_invariants.rs`

**Checklist:**
- [x] Add/adjust regression tests for add/remove/retarget operator behavior.
- [x] Run: `cd v2 && cargo test -p v2-core --test mutation_invariants` and confirm operator behavior coverage.
- [x] Verify operator implementations preserve bounded structural semantics and valid retargeting.
- [x] Re-run targeted test and confirm pass.
- [x] Commit slice `E2`.

### Slice E3: Node/subgraph duplication behavior

**Files:**
- Modify: `v2/crates/v2-core/src/evolution/operators.rs`
- Modify: `v2/crates/v2-core/tests/mutation_invariants.rs`

**Checklist:**
- [ ] Add/adjust failing tests for node duplication and bounded subgraph duplication behavior.
- [ ] Run: `cd v2 && cargo test -p v2-core --test mutation_invariants` and confirm failure is expected.
- [ ] Implement minimal duplication/remap fix.
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `E3`.

### Slice E4: Repair/discard pipeline invariants

**Files:**
- Modify: `v2/crates/v2-core/src/evolution/validation.rs`
- Modify: `v2/crates/v2-core/tests/mutation_repair.rs`

**Checklist:**
- [ ] Add/adjust failing tests for repair success and unsatisfiable discard behavior.
- [ ] Run: `cd v2 && cargo test -p v2-core --test mutation_repair` and confirm failure is expected.
- [ ] Implement minimal repair/discard pipeline fix.
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `E4`.

### Slice E5: Reproduction memory inheritance

**Files:**
- Modify: `v2/crates/v2-core/src/evolution/mod.rs`
- Modify: `v2/crates/v2-core/tests/reproduction_memory_inheritance.rs`

**Checklist:**
- [ ] Add/adjust failing tests for byte-for-byte memory copy semantics.
- [ ] Run: `cd v2 && cargo test -p v2-core --test reproduction_memory_inheritance` and confirm failure is expected.
- [ ] Implement minimal inheritance-path fix.
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `E5`.

### Slice E6: Phenotype evolution contract checks

**Files:**
- Modify: `v2/crates/v2-core/src/phenotype.rs`
- Modify: `v2/crates/v2-core/tests/phenotype_evolution.rs`

**Checklist:**
- [ ] Add/adjust failing tests for founder baseline, deterministic mutation, and channel-step behavior.
- [ ] Run: `cd v2 && cargo test -p v2-core --test phenotype_evolution` and confirm failure is expected.
- [ ] Implement minimal phenotype evolution fix.
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `E6`.

### Slice E7: Ecology pressure formulas

**Files:**
- Modify: `v2/crates/v2-core/src/ecology/mod.rs`
- Modify: `v2/crates/v2-core/src/ecology/config.rs`
- Modify: `v2/crates/v2-core/tests/ecology_pressures.rs`

**Checklist:**
- [ ] Add/adjust failing tests for crowding monotonicity, scarcity attenuation/recovery, and regime smoothing.
- [ ] Run: `cd v2 && cargo test -p v2-core --test ecology_pressures` and confirm failure is expected.
- [ ] Implement minimal ecology pressure fix.
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `E7`.

### Slice E8: Non-collapse baseline viability

**Files:**
- Modify: `v2/crates/v2-core/src/ecology/mod.rs`
- Modify: `v2/crates/v2-core/tests/ecology_noncollapse.rs`

**Checklist:**
- [ ] Add/adjust failing non-collapse baseline test with fixed seed and startup baseline.
- [ ] Run: `cd v2 && cargo test -p v2-core --test ecology_noncollapse` and confirm failure is expected.
- [ ] Implement minimal non-collapse calibration fix.
- [ ] Re-run targeted test and confirm pass.
- [ ] Commit slice `E8`.

### Slice E9: CP-2 gate closeout

**Files:**
- Modify: `docs/plans/2026-02-13-creature-brain-mesh-stage-3-evolution-ecology.md`
- Modify: `docs/plans/2026-02-14-v2-checkpoint-boundaries.md`

**Checklist:**
- [ ] Run full CP-2 gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md`.
- [ ] Confirm deterministic pass behavior on required suites.
- [ ] Update checkpoint status only when full gate is green.
- [ ] Commit slice `E9`.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. Run the `CP-2` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-2 exit`).

## Risks and Rollback

- Risk: mutation and ecology tuning can oscillate if too many variables change in one slice.
- Risk: non-collapse tuning can overfit one seed if deterministic coverage is too narrow.
- Rollback:
1. Revert only the failing slice commit.
2. Restore red test state for that slice.
3. Reapply minimal fix without widening scope.
