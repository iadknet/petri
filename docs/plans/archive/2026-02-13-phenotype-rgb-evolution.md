> **Archive note (2026-02-14):** Moved from active plans after the greenfield `v2` reset. Retained for historical context only.

# Phenotype RGB Evolution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace HSV-based phenotype inheritance with RGB-channel mutation and a heritable polarity bit so distant lineages diverge visually faster.
**Goal IDs:** GP-01, GP-03
**Scope:** Update `petri-core` phenotype state/mutation logic/tests plus any directly affected protocol typings/docs; exclude transport-level payload schema additions and legacy snapshot compatibility.
**Docs Impact:** Update `README.md` phenotype behavior note; no canonical docs changed; no stubs retired.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: phenotype mutation becomes channel-local and directionally heritable, making visual divergence better aligned with lineage separation.
- `GP-03`: behavior change is delivered test-first with deterministic seed coverage.

## Boundary Impact

- Dependency direction: unchanged (`petri-core` only; no crate boundary changes).
- Public API / wire format: frame/detail keep `phenotype_color`; snapshot creature internals move away from `phenotype_hue`/`phenotype_saturation` to RGB+polarity fields (no backward compatibility).
- Test migration: replace HSV assumptions in `petri-core` tests with RGB seed fixtures and new mutation assertions.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `crates/petri-core/src/world/tick.rs` reproduction/mutation flow | change | Phenotype inheritance is simulation policy and belongs in core tick logic. |
| `crates/petri-core/src/types.rs` snapshot/state transport structs | change | Internal creature state fields must reflect RGB+polarity to keep snapshot serialization coherent. |
| `web/src/protocol.ts` | keep | UI rendering relies on `phenotype_color` and can treat internal snapshot fields as optional. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should legacy snapshots with HSV phenotype fields still import? | No; explicit non-goal for this task. | user+agent | resolved |
| What should mutation constants be for channel-step, polarity-flip chance, and weight randomization range? | Use conservative defaults in `world/color.rs` constants and keep them centralized for tuning. | agent | resolved |

### Task 1: Add failing color mutation tests

Files:
- Modify: `crates/petri-core/src/world/color.rs`

Steps:
1. Add unit tests for channel-wise mutation and polarity-flip behavior using deterministic `SmallRng`.
2. Run focused test target to verify new tests fail before implementation changes.

### Task 2: Implement RGB+polarity phenotype model in core world state

Files:
- Modify: `crates/petri-core/src/world/color.rs`
- Modify: `crates/petri-core/src/world/mod.rs`
- Modify: `crates/petri-core/src/world/spawn.rs`
- Modify: `crates/petri-core/src/world/tick.rs`
- Modify: `crates/petri-core/src/world/snapshot.rs`
- Modify: `crates/petri-core/src/types.rs`
- Modify: `crates/petri-core/src/world/tests.rs`

Steps:
1. Replace HSV storage fields with `phenotype_color`, `phenotype_channel_weights`, and `phenotype_channel_positive_increment`.
2. Implement RGB mutation helper that mutates exactly one weighted-selected channel per mutation event and uses per-channel polarity bits for sign.
3. Add rare selected-channel polarity flip logic during inheritance and re-randomize the selected channel weight each mutation.
4. Update snapshot/state structs and constructors to store/load RGB + channel weights + channel polarity bits directly.
5. Update world tests/fixtures to compile and assert new behavior.

### Task 3: Verify and document behavior update

Files:
- Modify: `README.md`

Steps:
1. Update phenotype behavior sentence to reflect RGB-channel mutation plus rare polarity flips.
2. Run required quality gates relevant to this change.

## Verification Commands

1. `scripts/check-doc-harness.sh --mode warn`
2. `scripts/check-architecture-harness.sh --mode warn`
3. `scripts/check-plan-harness.sh --mode strict`
4. `cargo fmt --all --check`
5. `cargo test -p petri-core`
6. `cargo clippy --workspace --all-targets -- -D warnings`
7. `cargo test --workspace`
8. `cd web && npm run build`

## Risks and Rollback

- Risk: phenotype migration touches many test fixtures and snapshot fields, increasing compile-break surface.
- Risk: channel mutation parameters may be too subtle or too strong and need tuning.
- Rollback: revert commits touching phenotype field migration and restore HSV helpers/fields.
