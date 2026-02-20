# V3 Stage 3B: Genome and Mutation Foundation

**Goal:** Introduce heritable creature genomes and deterministic mutation plumbing so reproduction can produce offspring genomes that differ from founders while preserving Stage 2/3A runtime behavior.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Add minimal forward-compatible genome representation, mutation config defaults, mutation application during offspring creation, and Stage 3 viability assertions for genome divergence. Excludes VM execution engine replacement, graph backend execution, inventory actions, and transport API changes.

**Docs Impact:**
- Adds this implementation plan only.
- No canonical strategy/reference docs changed in this slice.
- No stale docs retired in this slice.

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-14-v3-architecture-design.md`

## Goal Alignment

- **GP-01:** Enables evolutionary search pressure by introducing mutable heritable genome state.
- **GP-02:** Keeps genome and mutation ownership in `v3-core/creature` with config policy in `v3-core/config`.
- **GP-03:** Uses TDD for mutation/reproduction behaviors and adds an intent-level viability regression.
- **GP-04:** Extends viability evidence beyond births to include observable genome divergence.

## Boundary Impact

- `v3-core/config`: add runtime mutation config (`config/runtime/mutation.rs`) and thread into `SimulationConfig`.
- `v3-core/creature`: add `genome.rs` and `mutation.rs`; extend `CreatureState` to carry genome state.
- `v3-core/creature/reproduction`: apply mutation on offspring genome creation while preserving energy/memory semantics from Stage 3A.
- `v3-core/seed`: keep fixed founder phenotype baseline, but seed creatures with deterministic founder genome template.
- `v3-server`: no API/transport changes; server continues consuming `SimulationState::tick`.
- Dependency direction remains unchanged (`v3-server -> v3-core` only).

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/tick/*` | keep | Tick remains orchestration/action execution owner; mutation stays out of tick to avoid policy leakage. |
| `v3/crates/v3-core/src/creature/reproduction.rs` | change | Offspring construction is the correct boundary for genome inheritance + mutation application. |
| `v3/crates/v3-core/src/runtime/executor.rs` | keep | Stage 3B intentionally preserves heuristic runtime to isolate genome/mutation risk before VM cutover. |
| `v3/crates/v3-server/src/state.rs` | keep | Server continues to own transport lifecycle only; no mutation policy should move into server. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should Stage 3B implement the full multi-node mesh genome schema now? | No. Implement a minimal VM-oriented genome representation that is forward-compatible with later node/graph expansion. | agent | resolved |
| Where should mutation defaults live? | In `config/runtime/mutation.rs` and referenced by `SimulationConfig`, consumed by `creature::mutation`. | agent | resolved |
| Should Stage 2 viability test be replaced? | No. Keep Stage 2 viability gate and add a dedicated Stage 3 viability regression for genome divergence. | agent | resolved |

## Implementation Checklist

### Task 1: Runtime Mutation Config Wiring

**Files:**
- Create: `v3/crates/v3-core/src/config/runtime/mod.rs`
- Create: `v3/crates/v3-core/src/config/runtime/mutation.rs`
- Modify: `v3/crates/v3-core/src/config/mod.rs`
- Modify: `v3/crates/v3-core/tests/contracts_test.rs`

**Checklist:**
- [x] Add `RuntimeConfig` with nested `MutationConfig` to `SimulationConfig`.
- [x] Define mutation defaults for Stage 3B (`mutation_probability`, min/max mutation events, constant-jitter magnitude).
- [x] Add/adjust config tests asserting mutation defaults are wired into `SimulationConfig::default()`.

### Task 2: Genome Type Foundation in Creature Module

**Files:**
- Create: `v3/crates/v3-core/src/creature/genome.rs`
- Modify: `v3/crates/v3-core/src/creature/mod.rs`
- Modify: `v3/crates/v3-core/src/creature/state.rs`
- Modify: `v3/crates/v3-core/src/seed.rs`
- Modify: `v3/crates/v3-core/tests/creature_state_test.rs`
- Modify: `v3/crates/v3-core/tests/tick_actions_test.rs`
- Modify: `v3/crates/v3-core/tests/tick_integration_test.rs`
- Modify: `v3/crates/v3-core/tests/runtime_executor_test.rs`

**Checklist:**
- [x] Add `CreatureGenome` + minimal constants pool with `PartialEq` for divergence detection.
- [x] Add deterministic founder genome constructor (`simple_founder`) used for seed and default creature construction.
- [x] Extend `CreatureState` with `genome` field and update constructor signature.
- [x] Update `seed.rs` to supply founder genome to seed creatures.
- [x] Update all test callsites for new constructor signature.
- [x] Add tests that new creatures have deterministic founder genome and that cloning carries genome data.

### Task 3: Mutation Engine + Reproduction Integration

**Files:**
- Create: `v3/crates/v3-core/src/creature/mutation.rs`
- Modify: `v3/crates/v3-core/src/creature/reproduction.rs`
- Modify: `v3/crates/v3-core/tests/creature_state_test.rs`

**Checklist:**
- [x] Add failing tests for mutation trigger/no-trigger paths in offspring generation.
- [x] Implement bounded deterministic mutation application using `MutationConfig`.
- [x] Apply mutation during `create_offspring()` after genome copy and before child construction finalization.
- [x] Preserve Stage 3A invariants: memory copy exact, generation increment, cost-first action semantics unchanged.

### Task 4: Stage 3 Viability Regression (Genome Divergence)

**Files:**
- Create: `v3/crates/v3-core/tests/seed_viability_stage3.rs`
- Modify: `docs/plans/2026-02-17-v3-stage3b-genome-mutation-foundation.md`

**Checklist:**
- [x] Add a Stage 3 viability test with reproduction enabled and deterministic mutation settings.
- [x] Assert births occur over the run.
- [x] Assert at least one generation>0 creature genome differs from the founder genome.
- [x] Keep test bounded for local iteration speed.

### Task 5: Verification + Plan Status Update

**Files:**
- Modify: `docs/plans/2026-02-17-v3-stage3b-genome-mutation-foundation.md`

**Checklist:**
- [x] Mark completed items in this plan.
- [x] Run `cd v3 && cargo test --workspace`.
- [x] Run `cd v3 && cargo fmt --all --check`.
- [x] Run `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`.

## Verification Commands

- `cd v3 && cargo test -p v3-core --test creature_state_test`
- `cd v3 && cargo test -p v3-core --test tick_actions_test`
- `cd v3 && cargo test -p v3-core --test seed_viability_stage3`
- `cd v3 && cargo test --workspace`
- `cd v3 && cargo fmt --all --check`
- `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`

## Risks and Rollback

- Risk: Minimal genome shape drifts from long-term mesh schema.
  - Mitigation: keep field names and limits aligned with v3 reference docs where practical and isolate representation to `creature/genome.rs`.
- Risk: Mutation randomness introduces flaky tests.
  - Mitigation: force deterministic seeds and explicit mutation probabilities in tests.
- Risk: Mutation changes accidentally destabilize Stage 2 viability behavior.
  - Mitigation: preserve Stage 2 viability test and add Stage 3-specific assertions instead of replacing Stage 2 checks.
- Rollback: Revert Stage 3B files and re-run Stage 2 + Stage 3A tests to confirm baseline restoration.

## Review cycles: 2

Cycle 1 (architecture + goal alignment): Confirmed Stage 3B should isolate genome/mutation from runtime VM cutover to avoid mixed-boundary changes; plan scope kept VM executor unchanged and mutation ownership confined to creature/config modules.

Cycle 2 (implementation review): Found and resolved gap where `seed.rs` was not listed in Task 2 but needed updating for genome parameter. Added seed.rs, tick_integration_test.rs, tick_actions_test.rs, runtime_executor_test.rs to Task 2's file list. All 56 tests pass, Stage 2 viability preserved, Stage 3 genome divergence verified.
