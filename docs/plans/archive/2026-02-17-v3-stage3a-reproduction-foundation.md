# V3 Stage 3A: Reproduction Foundation

**Goal:** Add the first Stage 3 reproduction loop so creatures can create offspring through tick actions while preserving Stage 2 viability behavior.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Add reproduction action contracts, reproduction config knobs, creature reproduction helper, heuristic runtime reproduction choice, and tick spawn plumbing. Excludes genome mutation, VM execution, and graph operators.

**Docs Impact:**
- Adds this implementation plan only.
- No canonical strategy/reference doc edits required for this slice.
- No stale docs retired in this slice.

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-14-v3-architecture-design.md`

## Goal Alignment

- **GP-01:** Introduces population growth mechanics (births) as a prerequisite for evolving ecology.
- **GP-02:** Keeps reproduction policy in `v3-core` (`creature/`, `tick/`, `runtime/`) with no transport leakage.
- **GP-03:** Uses TDD on reproduction execution and integration paths, plus targeted regressions for energy and spawn semantics.
- **GP-04:** Uses existing `TickStats.births` field with real action-driven births (no synthetic counters).

## Boundary Impact

- `v3-core/contracts`: add `WorldAction::Reproduce` variant.
- `v3-core/config`: add reproduction-related energy knobs.
- `v3-core/creature`: add reproduction helper and memory-copy-safe state plumbing.
- `v3-core/runtime`: heuristic can emit reproduce action when conditions are met.
- `v3-core/tick`: action executor returns optional offspring; orchestrator appends deferred spawns after Phase 2.
- Dependency direction remains unchanged (`v3-server -> v3-core`, no reverse dependency).
- Wire/API impact: none in this slice (server transport unchanged).

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/tick/orchestrator.rs` | keep | Preserve phase-based model and deferred spawn insertion to protect fairness and SlotMap key stability. |
| `v3/crates/v3-core/src/tick/actions.rs` | change | Reproduction action semantics belong in action execution, not runtime/sensors. |
| `v3/crates/v3-core/src/runtime/executor.rs` | change | Runtime chooses action intent only; world mutation still stays in tick actions. |
| `v3/crates/v3-server/src/state.rs` | keep | No server API changes needed; server should consume behavior via unchanged `SimulationState::tick`. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should reproduction include explicit energy transfer metadata or use a fixed config value? | Include `energy_amount` in `WorldAction::Reproduce` and clamp/validate against config in action execution. | agent | resolved |
| Should mutation be included in this slice? | No; keep offspring as deterministic clone foundation and defer mutation to Stage 3B. | agent | resolved |
| Should heuristic prioritize reproduction over eating? | No; keep eat-first policy to preserve Stage 2 survival dynamics and reduce regression risk. | agent | resolved |

## Implementation Checklist

### Task 1: Contracts and Config for Reproduction

**Files:**
- Modify: `v3/crates/v3-core/src/contracts/outputs.rs`
- Modify: `v3/crates/v3-core/src/config/energy/costs.rs`
- Modify: `v3/crates/v3-core/src/config/energy/lifecycle.rs`
- Modify: `v3/crates/v3-core/tests/contracts_test.rs`

**Checklist:**
- [x] Add failing contract test coverage for `WorldAction::Reproduce` shape.
- [x] Add reproduction config defaults (`reproduce_cost`, minimum parent energy, default offspring energy).
- [x] Extend `WorldAction` with `Reproduce { direction, energy_amount }`.
- [x] Re-run targeted contracts/config tests.

### Task 2: Creature Reproduction Primitive

**Files:**
- Modify: `v3/crates/v3-core/src/creature/mod.rs`
- Modify: `v3/crates/v3-core/src/creature/state.rs`
- Create: `v3/crates/v3-core/src/creature/reproduction.rs`
- Modify: `v3/crates/v3-core/tests/creature_state_test.rs`

**Checklist:**
- [x] Add failing tests for offspring generation increment, phenotype inheritance, and memory copy behavior.
- [x] Add persistent 1024-byte memory field on `CreatureState`.
- [x] Implement `create_offspring(parent, spawn_position, initial_energy, config, rng)` helper with deterministic clone semantics for this slice (mutation deferred).
- [x] Re-run targeted creature tests.

### Task 3: Runtime Reproduction Intent

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/executor.rs`
- Modify: `v3/crates/v3-core/tests/runtime_executor_test.rs`

**Checklist:**
- [x] Add failing heuristic test for reproduce decision when energy is high and a passable neighbor exists.
- [x] Update heuristic priority order: eat, then reproduce when eligible, then move, then noop.
- [x] Re-run targeted runtime tests.

### Task 4: Tick Action Execution and Deferred Spawn Integration

**Files:**
- Modify: `v3/crates/v3-core/src/tick/actions.rs`
- Modify: `v3/crates/v3-core/src/tick/orchestrator.rs`
- Modify: `v3/crates/v3-core/tests/tick_actions_test.rs`
- Modify: `v3/crates/v3-core/tests/tick_integration_test.rs`

**Checklist:**
- [x] Add failing action test(s) for reproduction success/failure semantics.
- [x] Extend `ActionResult` with optional offspring payload.
- [x] Implement `execute_reproduce` with energy-cost-first policy and occupancy/barrier checks.
- [x] Integrate deferred offspring insertion in orchestrator and real births counting.
- [x] Re-run targeted tick tests.

### Task 5: Stage Slice Verification and Status Update

**Files:**
- Modify: `docs/plans/2026-02-17-v3-stage3a-reproduction-foundation.md`

**Checklist:**
- [x] Mark completed checklist items in this plan.
- [x] Run `cd v3 && cargo test --workspace`.
- [x] Run `cd v3 && cargo fmt --all --check`.
- [x] Run `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`.

## Verification Commands

- `cd v3 && cargo test -p v3-core --test contracts_test`
- `cd v3 && cargo test -p v3-core --test creature_state_test`
- `cd v3 && cargo test -p v3-core --test runtime_executor_test`
- `cd v3 && cargo test -p v3-core --test tick_actions_test`
- `cd v3 && cargo test -p v3-core --test tick_integration_test`
- `cd v3 && cargo test --workspace`
- `cd v3 && cargo fmt --all --check`
- `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`

## Risks and Rollback

- Risk: Reproduction priority may destabilize Stage 2 viability assumptions.
  - Mitigation: Keep eat-first heuristic and add integration regressions for survival + births.
- Risk: Energy transfer semantics could create free-energy bugs.
  - Mitigation: enforce cost-first, bounded transfer, and explicit tests for failure paths.
- Rollback: Revert files touched in this slice and re-run Stage 2 viability tests to confirm baseline restoration.

## Review cycles: 2

Cycle 1 (architecture review): Identified a forward-compatibility gap where `create_offspring()` could omit `config`/`rng` and force a breaking signature change in Stage 3B mutation work. Resolved by requiring a forward-compatible signature in this plan.

Cycle 2 (goal alignment review): Confirmed scope maps cleanly to GP-01/GP-02/GP-03/GP-04 and does not violate crate boundaries or transport ownership. No additional revisions required.
