---
title: Action Cost Helper Refactor
status: ready
tags: [core, architecture]
size: S
depends-on: []
---

# Action Cost Helper Refactor — Implementation Plan

**Goal:** Centralize the repeated complexity-adjusted action cost calculation into a single helper method on `EnergyConfig`, replacing 9 production call sites and 8 test call sites.
**Goal IDs:** GP-02, GP-03
**Scope:** Pure refactor within `v3/crates/v3-core/src/`. No behavior change, no new features, no config changes.
**Docs Impact:** None — no canonical docs touched. This is an internal code-only refactor.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- **GP-02 (Clean Architecture Boundaries):** Centralizing the cost calculation removes 9 duplicated inline computations, reducing the risk of inconsistency when the formula changes. The helper lives on `EnergyConfig`, keeping energy-related logic within the config module's responsibility.
- **GP-03 (High-Confidence Iteration):** Fewer places to update when the cost formula changes means fewer opportunities for partial updates and regressions. The helper is independently testable.

## Boundary Impact

- **No boundary changes.** The helper is a new method on the existing `EnergyConfig` type in `v3/crates/v3-core/src/config/simulation.rs`. No new modules, crates, or public API surface changes.
- **Dependency direction unchanged.** `simulation/actions/*` and `simulation/tick.rs` already depend on `config::SimulationConfig`. The new method adds no new dependencies.
- **No wire-format changes.** No serialization or API surface affected.
- **Test migration:** Test call sites that replicate the pattern will be updated to use the helper. No test files are added or removed.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3/crates/v3-core/src/config/simulation.rs` (`EnergyConfig`) | keep — add method here | `EnergyConfig` already owns `action_cost_multiplier()`; the new helper composes it with a base cost. Natural home. |
| `v3/crates/v3-core/src/simulation/actions/` | keep — update call sites only | Action functions remain responsible for orchestrating the action; they delegate cost computation to the config helper. |
| `v3/crates/v3-core/src/simulation/tick.rs` | keep — update call sites only | Tick orchestration delegates cost computation to the config helper for failed-action penalties. |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should the helper live on `Simulation`, `EnergyConfig`, or be a free function? | `EnergyConfig` method — it already owns `action_cost_multiplier()`, and the new helper composes that with a base cost. No `Simulation` access needed. | agent | resolved |
| Should the helper compute-and-deduct or just compute the adjusted cost? | Compute-only — returns `f32`. More composable; predation uses `steal_cost_rate * requested_amount * mult` which doesn't fit a deduct API. Callers do `creature.energy -= helper_result`. | agent | resolved |
| Should failed-action penalties use the same helper? | Yes — the pattern is identical: `base_cost * multiplier`. The helper is generic over any base cost value. | agent | resolved |

## Helper Design

New method on `EnergyConfig`:

```rust
/// Returns the complexity-and-age-adjusted cost for a given base cost.
///
/// Formula: `base_cost * action_cost_multiplier(complexity, age)`
#[inline]
#[must_use]
pub fn adjusted_action_cost(&self, base_cost: f32, complexity: u32, age: u64) -> f32 {
    base_cost * self.action_cost_multiplier(complexity, age)
}
```

**Signature:** `(base_cost, complexity, age) -> f32`. Uses primitive types only to avoid introducing a dependency from `config` to `creature::state::CreatureState`.

### Call site transformation

Standard pattern (noop, eat, move, reproduce, failed-action penalties):
```rust
// Before:
let mult = config.energy.action_cost_multiplier(creature.genome.complexity(), creature.age);
creature.energy -= config.energy.costs.noop_cost * mult;
// After:
creature.energy -= config.energy.adjusted_action_cost(
    config.energy.costs.noop_cost, creature.genome.complexity(), creature.age,
);
```

Predation pattern (base cost is computed, not a config field):
```rust
// Before:
let mult = sim.config.energy.action_cost_multiplier(...);
let cost = sim.config.predation.steal_cost_rate * requested_amount * mult;
// After:
let cost = sim.config.energy.adjusted_action_cost(
    sim.config.predation.steal_cost_rate * requested_amount, ...
);
```

### Call sites inventory

**9 production call sites:**
1. `actions/mod.rs:18` — `apply_noop`
2. `actions/mod.rs:36` — `apply_eat`
3. `actions/mod.rs:68` — `apply_move`
4. `actions/reproduction.rs:137` — `apply_reproduce`
5. `actions/predation.rs:62` — `apply_steal_energy`
6. `tick.rs:347` — failed eat penalty
7. `tick.rs:375` — failed move penalty
8. `tick.rs:420` — failed reproduce penalty
9. `tick.rs:468` — failed steal penalty

**8 test call sites** (in `actions/mod.rs`, `actions/predation.rs`, `tick.rs`):
- `actions/mod.rs:187,212` (2 sites)
- `actions/predation.rs:302,326,357` (3 sites)
- `tick.rs:904,976,997` (3 sites)

**2 test call sites in `config/simulation.rs`** test `action_cost_multiplier` directly — these stay unchanged since `action_cost_multiplier` is not removed.

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Note: This is a pure refactor, so TDD here means: write a test for the new helper method first, verify it fails (method doesn't exist), implement the method, verify all tests pass.

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Implementation Steps

- [ ] Step 1: Write a test for `EnergyConfig::adjusted_action_cost` that verifies `base_cost * action_cost_multiplier(complexity, age)`. Confirm it fails to compile (method doesn't exist yet).
- [ ] Step 2: Implement `EnergyConfig::adjusted_action_cost` in `v3/crates/v3-core/src/config/simulation.rs`. Confirm the new test passes and all existing tests pass.
- [ ] Step 3: Replace production call sites in `actions/mod.rs` (3 sites: `apply_noop`, `apply_eat`, `apply_move`) with the helper. Run `cargo test`.
- [ ] Step 4: Replace production call site in `actions/reproduction.rs` (1 site: `apply_reproduce`) with the helper. Run `cargo test`.
- [ ] Step 5: Replace production call site in `actions/predation.rs` (1 site: `apply_steal_energy`) with the helper. Run `cargo test`.
- [ ] Step 6: Replace production call sites in `tick.rs` (4 sites: failed-action penalties) with the helper. Run `cargo test`.
- [ ] Step 7: Update test call sites in `actions/mod.rs`, `actions/predation.rs`, and `tick.rs` to use the helper where appropriate (8 sites). Run `cargo test`.
- [ ] Review Gate: Code review — invoke `rust-skills` on full branch diff. Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 3
