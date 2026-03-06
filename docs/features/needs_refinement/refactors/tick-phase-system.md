---
title: Tick Phase System
tags: [core, architecture]
size: M
depends-on: []
status: needs-review
---

## Problem Statement

The simulation tick loop lives in a monolithic `run_tick` function (~523 lines) in `v3/crates/v3-core/src/simulation/tick.rs`. It executes 5 phases in sequence:

- **Phase 0** — Creature evaluation: runs each creature's genome (VM + graph), producing action queues. Already extracted to `run_phase_0`.
- **Phase 0.5** — Queue building: collects and sorts creature actions into priority-ordered queues per action type.
- **Phase 1** — Action execution: processes each action queue (eat, move, reproduce, steal, noop) with conflict resolution and energy deduction.
- **Phase 2** — World updates: food growth, energy decay, death processing, stat collection.
- **Phase 2.5** — Reward learning: iterates over creatures to apply reward-modulated plasticity updates based on action outcomes.

Only Phase 0 has been extracted to its own function. The remaining phases are inline in `run_tick`, making the function difficult to navigate, test in isolation, and extend. Adding a new tick-level pass (e.g., communication processing, aging mechanics, environmental effects) requires growing this already-large function.

A phase-based system where each phase is an extracted function (following the `run_phase_0` pattern that already exists) would improve readability, testability, and extensibility. Each phase could be tested independently with controlled inputs, and new phases could be added without modifying the core orchestration logic.

## User Stories / Acceptance Criteria

- As a simulation developer, I want each tick phase in its own function so that I can understand and modify one phase without reading 500+ lines of context.
- As a contributor adding new tick-level mechanics, I want a clear pattern for where new phases go and how they integrate.
- As a test author, I want to test individual phases in isolation with controlled inputs.

### Acceptance Criteria

1. Each tick phase is extracted to its own function following the `run_phase_0` pattern.
2. `run_tick` becomes a thin orchestrator (~50-80 lines) that calls phase functions in sequence.
3. Inter-phase data is passed explicitly via function parameters/return values, not implicit shared state.
4. All existing tick tests continue to pass with identical behavior.
5. All viability tests pass (merge gate).
6. No measurable performance regression.

## Out of Scope

- Dynamic phase registration or plugin system — static function calls are sufficient.
- Phase reordering or parallel phase execution.
- Changes to the runtime/VM internals within phases.
- Adding new phases — this refactor only extracts existing logic.
- Modifying action dispatch or conflict resolution algorithms.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| How should inter-phase data be passed? Struct of intermediate results, or individual parameters? | Pending — struct reduces parameter count but adds a type | - | open |
| Should the action trace/logging concern be factored out of individual phases into the orchestrator? | Pending | - | open |
| Where should stat resets (beginning-of-tick zeroing) live — in the orchestrator or in Phase 0? | Pending | - | open |
| Will Rust's borrow checker require specific ownership patterns for the phase functions (e.g., splitting borrows on Simulation)? | Pending — likely needs `&mut self` method calls or explicit field splitting | - | open |
| Should phase numbering be formalized (enum) or remain informal (function names only)? | Pending | - | open |
