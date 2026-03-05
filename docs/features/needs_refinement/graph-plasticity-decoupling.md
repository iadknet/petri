---
title: Graph Evaluation / Plasticity Decoupling
tags: [core, architecture, genome]
size: M
depends-on: []
status: needs-review
---

## Problem Statement

The graph evaluation loop in `v3/crates/v3-core/src/runtime/graph.rs` (`execute_graph_impl`) is directly coupled to plasticity modules at three points:

1. **Pre-loop init** (lines ~261-269) — initializes plasticity weight state before the relaxation loop begins.
2. **Inner-loop weight selection** (lines ~320-340) — during each relaxation iteration, selects between static genome weights and dynamic plasticity weights for plasticity-enabled edges.
3. **Post-convergence updates** (lines ~398-420) — after the graph converges, applies Hebbian learning updates to plasticity weights.

Additionally, `tick.rs` (lines ~514-572) contains a Phase 2.5 reward learning pass that iterates over all creatures to apply reward-modulated plasticity updates, representing another coupling point between the simulation loop and specific plasticity algorithms.

The plasticity system itself is split across three modules:
- `runtime/plasticity/hebbian.rs` — pure Hebbian learning (weight init, update rules, weighted input collection)
- `runtime/plasticity/traces.rs` — eligibility trace management for reward-modulated nodes
- `runtime/plasticity/reward.rs` — three-factor reward-modulated weight updates

Creature state (`GraphRuntimeState`) also holds plasticity-specific fields (`plasticity_weights`, `eligibility_traces`), coupling the state representation to specific learning algorithms.

This tight coupling means adding a new learning algorithm (e.g., STDP, neuromodulation, competitive learning) requires modifying the graph evaluation hot path, the tick loop, and creature state — rather than registering a new backend.

A more extensible design would have graph evaluation produce "learning events" (pre-activation values, post-activation values, convergence signals) dispatched to registered plasticity backends, decoupling the graph relaxation loop from the specifics of any learning algorithm.

## User Stories / Acceptance Criteria

- As a simulation developer, I want to add new learning algorithms without modifying the graph evaluation loop so that the hot path remains stable.
- As a researcher, I want to experiment with different plasticity rules by swapping backends rather than editing core graph code.

### Acceptance Criteria

1. Graph evaluation (`execute_graph_impl`) does not directly call into any plasticity module. Instead, it produces learning event data (pre/post activation pairs, convergence signals) consumed by a plasticity interface.
2. A `PlasticityBackend` trait (or equivalent abstraction) defines the interface for learning algorithms: init, per-iteration weight selection, and post-convergence update.
3. Hebbian learning and reward-modulated learning are implementations of this trait, not hard-coded call sites.
4. Phase 2.5 reward learning in `tick.rs` dispatches through the same abstraction.
5. All existing plasticity tests continue to pass with identical behavior.
6. No measurable performance regression in the graph evaluation hot path (benchmark before/after).

## Out of Scope

- Adding new learning algorithms — this is purely a structural refactor to enable future additions.
- Changing `GraphRuntimeState` field layout (can be addressed as a follow-up once the trait boundary is clear).
- Dynamic plugin loading or runtime algorithm selection — compile-time dispatch is sufficient.
- Modifying plasticity parameters or behavior.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should the plasticity trait use static dispatch (generics) or dynamic dispatch (trait objects)? | Pending — static dispatch avoids vtable overhead in the hot path but reduces flexibility | - | open |
| Should `GraphRuntimeState` plasticity fields be moved behind the trait boundary? | Pending — would further decouple state but increases refactor scope | - | open |
| How should the Phase 2.5 reward pass integrate with the trait? Separate method or same interface? | Pending | - | open |
| Should learning events be buffered and batch-processed, or processed inline? | Pending — buffering could enable parallelism but adds allocation | - | open |
