---
title: Action Cost Helper Refactor
tags: [core, architecture]
size: S
depends-on: []
status: needs-review
---

## Problem Statement

The complexity-adjusted action cost pattern `config.energy.complexity_cost.multiplier(creature.genome.complexity())` is repeated 9 times in production code across 4 files in `v3/crates/v3-core/src/simulation/`:

- `actions/mod.rs` — 3 sites: `apply_noop`, `apply_eat`, `apply_move`
- `actions/reproduction.rs` — 1 site: `apply_reproduce`
- `actions/predation.rs` — 1 site: `apply_steal_energy`
- `tick.rs` — 4 sites: failed-action energy penalties for eat, move, reproduce, and steal

Additionally, 6 test call sites replicate the same pattern.

Each site follows the same structure: compute the multiplier from the creature's genome complexity, multiply it by the base action cost, and deduct the result from creature energy. The repetition makes it easy to introduce inconsistencies (e.g., forgetting the multiplier at one site, or changing the formula at some sites but not others).

A helper method (on `Simulation`, `EnergyConfig`, or as a standalone utility function) could centralize the cost-deduction-with-multiplier logic, reducing the 9 production call sites to single-line invocations and ensuring consistent cost calculation everywhere.

## User Stories / Acceptance Criteria

- As a simulation developer, I want action cost deduction centralized so that changing the cost formula requires editing one place, not nine.
- As a contributor, I want a clear, discoverable API for "deduct action cost from creature" so I don't have to copy-paste the multiplier pattern.

### Acceptance Criteria

1. A single helper function/method encapsulates the complexity-adjusted cost calculation and energy deduction.
2. All 9 production call sites use the helper instead of inline computation.
3. The helper is discoverable (well-named, on an appropriate type or module).
4. All existing tests pass with identical behavior — this is a pure refactor with no behavior change.
5. Test call sites are updated to use the helper where appropriate.

## Out of Scope

- Changing the cost formula itself (multiplier logic, base costs, etc.).
- Refactoring energy deduction beyond the complexity multiplier pattern.
- Adding new action types or cost categories.
- Performance optimization of the cost calculation.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should the helper live on `Simulation`, `EnergyConfig`, or be a free function in an `energy` utility module? | Pending — `EnergyConfig` method keeps it close to the config, but `Simulation` method has access to both config and creature | - | open |
| Should the helper both compute AND deduct energy, or just compute the adjusted cost (letting the caller deduct)? | Pending — compute-only is more composable but doesn't reduce call-site boilerplate as much | - | open |
| Should failed-action penalties (in tick.rs) use the same helper or a separate one, given they may diverge in the future? | Pending | - | open |
