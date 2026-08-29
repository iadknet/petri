---
title: Tick Phase System — Implementation Plan
status: ready_to_implement
---

**Goal:** Extract the monolithic `run_tick` function into discrete phase functions, making each phase independently testable and the orchestrator a thin ~35-line dispatcher.

**Goal IDs:** GP-02, GP-03, GP-04

**Scope:**
- In: `crates/v3-core/src/simulation/tick.rs` (phase extraction), `crates/v3-core/src/simulation/stats.rs` (add `reset_per_tick()` method), `crates/v3-core/src/simulation/outcomes.rs` (visibility change to `pub`), `crates/v3-core/src/simulation/mod.rs` (outcomes module visibility)
- Out: No behavior changes, no new phases, no action algorithm modifications, no plugin system, no phase reordering

**Docs Impact:**
- OTel design spec (`docs/superpowers/specs/2026-03-20-opentelemetry-observability-design.md`) references shorter phase names (`run_phase_1`, `run_phase_2`, `run_phase_2_5`) — update to match actual names when that feature moves to implementation.
- The needs_refinement doc (`docs/features/needs_refinement/refactors/tick-phase-system.md`) uses stale phase numbering (doesn't match `v3-tick-orchestration-spec.md`). It will be archived into `ready_to_implement/tick-phase-system/refinement.md` as part of the commit step.

**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

| Goal ID | Work Items |
|---------|------------|
| GP-02 (Clean Architecture Boundaries) | Enforces separation between tick phases via explicit function boundaries and typed parameter passing. Eliminates 500+ line monolithic function. Each phase's data dependencies become visible in its signature. |
| GP-03 (High-Confidence Iteration) | Each phase becomes independently testable with controlled inputs. Existing tests validate behavior preservation — all 260+ library tests and 18 viability tests serve as the regression suite. |
| GP-04 (Keep Behavior Observable) | Extracted phases with explicit signatures enable per-phase instrumentation — the documented OTel prerequisite. Each phase can be independently wrapped with tracing spans by `v3-server`. |

## Boundary Impact

- **Dependency direction:** No change. All phase functions remain within `v3-core::simulation::tick`.
- **Public API:** `run_tick` and `run_phase_0` signatures unchanged. New phase functions are `pub` visibility — the OTel design spec lists this refactor as a prerequisite and expects `v3-server` to call phase functions individually. Making them `pub` now avoids a follow-up visibility change and is consistent with `run_phase_0` (already `pub`). API surface tradeoff acknowledged: this exposes phase signatures before any external consumer exists, but the alternative (`pub(crate)`) would require re-touching v3-core when OTel is implemented.
- **Wire-format:** No changes.
- **Test migration:** No test changes needed — all existing tests call `run_tick` or `run_phase_0` which maintain their signatures.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `tick` ↔ `actions` module | keep | Phase 2 function calls the same action application functions (`apply_move`, `apply_reproduce`, etc.). Module boundary preserved. |
| `tick` ↔ `outcomes` module | change | `outcomes` module and `OutcomeAccumulator` struct must become `pub` (from `pub(crate)`) for E0446 compliance. `OutcomeRecord` stays `pub(crate)` (not in any public signature). `OutcomeAccumulator` methods (`snapshot_energy`, `record_action_result`, etc.) stay `pub(crate)` (only called within v3-core). No new coupling — just visibility widening of the struct and module. |
| `tick` ↔ `runtime` module | keep | Phase 1 calls `execute_creature_mesh` / `execute_creature_mesh_traced` as before. |
| `tick` ↔ `stats` module | change | Add `reset_per_tick()` method to `SimStats`. Moves stat reset responsibility to the type that owns the data. Correct ownership direction. |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| How should inter-phase data be passed? | Individual parameters per phase function. Each phase takes exactly what it needs — phases have different parameter sets, so a shared struct would carry unused fields. Individual params make data flow explicit. | plan | resolved |
| Should action trace/logging be factored out? | No. Tracing stays integrated in Phase 1 cognition where it naturally belongs (trace extraction from parallel batch + sequential traced execution). Trace target liveness check stays in orchestrator between Phase 0 and Phase 1. | plan | resolved |
| Where should stat resets live? | New `SimStats::reset_per_tick()` method called by orchestrator before Phase 0. Stat resets are per-tick setup, not phase logic. | plan | resolved |
| Borrow checker patterns for phase functions? | All phase functions take `&mut Simulation` as first param. Internal borrow splitting handled within each function — Phase 1 already does this for sensor assembly (`&sim.world`, `&sim.creatures`) vs mesh execution (`&mut sim.creatures`). | plan | resolved |
| Phase numbering formalization? | No enum. Function names follow existing convention: `run_phase_0` (exists), `run_phase_1_cognition`, `run_phase_2_actions`, `run_phase_2_5_reward_learning`. Static dispatch; no runtime phase selection needed. | plan | resolved |

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Note: This is a pure structural refactor with no behavior changes. Existing tests (260+ library tests, 18 viability tests) serve as the regression suite. No new failing tests are expected — the primary validation is that all existing tests continue to pass with identical behavior after each extraction step.

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

## Design

### Phase Function Signatures

```rust
// New method on SimStats (stats.rs)
impl SimStats {
    /// Zero all `last_tick_*` fields. Called once at the start of each tick.
    pub(crate) fn reset_per_tick(&mut self) { ... }
}

// Existing — unchanged
pub fn run_phase_0(sim: &mut Simulation);

/// Phase 1: Assemble sensor snapshots and execute creature meshes in parallel.
/// The optional trace target is extracted from the parallel batch and run
/// sequentially with detailed trace recording. Reads `sim.tick` for trace
/// tick numbering.
pub fn run_phase_1_cognition(
    sim: &mut Simulation,
    queue: &[CreatureId],
    runtime_config: &RuntimeConfig,
    trace_target: Option<CreatureId>,
    trace: &mut Option<ActiveTrace>,
) -> Vec<(CreatureId, MeshOutput)>;

/// Phase 2: Execute creature decisions sequentially in priority-bid order.
/// Handles action application, conflict resolution, failed-action penalties,
/// compute cost accumulation, and stat finalization. Reads `sim.tick` for
/// action log entries; callers must not modify `sim.tick` before this phase
/// completes. `successful_spawn_targets` (reproduction contention tracking)
/// is local to this function.
///
/// Design note: `failed_action_penalty` is pre-computed by the caller because
/// it derives from a non-trivial config method call. `sim.tick` is NOT extracted
/// as a parameter — it's a simple field read already available via `&mut Simulation`,
/// and adding a 6th parameter for a value accessible on `sim` would be redundant.
/// Note: `apply_reproduce` and `apply_steal_energy` (which take `&mut Simulation`)
/// do NOT use `failed_action_penalty` — the penalty is applied in this function's
/// loop after a failed action, not inside the action functions themselves. Those
/// action functions also increment `sim.stats.last_tick_reproduce` and
/// `sim.stats.last_tick_steal` directly, so stat mutation is layered between this
/// function and the action functions it calls.
pub fn run_phase_2_actions(
    sim: &mut Simulation,
    decisions: Vec<(CreatureId, MeshOutput)>,
    outcome_acc: &mut OutcomeAccumulator,
    reproduce_rng: &mut SmallRng,
    failed_action_penalty: f32,
);

/// Phase 2.5: Apply reward-modulated plasticity updates for creatures with
/// eligible graph nodes, using outcome signals accumulated during Phase 2.
pub fn run_phase_2_5_reward_learning(
    sim: &mut Simulation,
    outcome_acc: &OutcomeAccumulator,
);
```

### Orchestrator Shape

After refactoring, `run_tick` becomes (~35 lines):

```rust
pub fn run_tick(sim: &mut Simulation, trace: &mut Option<ActiveTrace>) {
    sim.stats.reset_per_tick();
    run_phase_0(sim);

    // Snapshot surviving creature energies for Phase 2.5 reward learning.
    let mut outcome_acc = OutcomeAccumulator::default();
    for (id, creature) in sim.creatures.iter() {
        outcome_acc.snapshot_energy(id, creature.energy);
    }

    // Build turn queue: stable sort for reproducibility, then shuffle.
    let mut queue: Vec<_> = sim.creatures.keys().collect();
    queue.sort();
    queue.shuffle(&mut sim.rng);

    // Check if traced creature died during Phase 0.
    if let Some(ref mut active) = trace {
        if !active.is_complete() && !sim.creatures.contains_key(active.creature_id) {
            active.ticks_remaining = 0;
        }
    }

    // Derive separate RNG and config for phase functions.
    // Note: sim.tick is stable across all phase calls — it increments only at the end.
    let mut reproduce_rng = SmallRng::seed_from_u64(sim.rng.next_u64());
    let runtime_config = sim.config.runtime.clone(); // clone needed: sim passed as &mut to phases
    let trace_target = trace.as_ref().filter(|t| !t.is_complete()).map(|t| t.creature_id);
    let failed_action_penalty = sim.config.failed_action_penalty_for_tick(sim.tick);

    let mut decisions = run_phase_1_cognition(sim, &queue, &runtime_config, trace_target, trace);
    sort_by_priority_bid(&mut decisions); // Intentionally in orchestrator: semantic boundary between cognition and execution.
    run_phase_2_actions(sim, decisions, &mut outcome_acc, &mut reproduce_rng, failed_action_penalty);
    run_phase_2_5_reward_learning(sim, &outcome_acc);

    sim.tick += 1;
}
```

### Import Distribution

The `use` statements currently inside `run_tick` (lines 147-176) must be distributed to the appropriate phase functions during extraction. Imports only used within one phase move into that phase function; shared imports stay at module level or in the orchestrator.

### File Organization

All phase functions remain in `tick.rs` — consistent with the existing `run_phase_0` pattern. File size stays roughly the same (~830 lines) since this is an intra-file restructuring (code moves between functions, not between files), with the only reduction being the ~20 lines of stat reset moved to `stats.rs`. Each phase is clearly bounded by its function signature.

### Re-exports

No new re-exports from `simulation/mod.rs` — consistent with `run_phase_0` precedent (not re-exported). Callers use the full module path: `v3_core::simulation::tick::run_phase_1_cognition`. Re-exports can be added as a follow-up if ergonomics are needed for `v3-server`.

### Out-of-scope Optimization Note

Several `Vec` allocations in Phase 1 (`inputs`, `work`, `parallel_decisions`) could benefit from `Vec::with_capacity(queue.len())`. This is a performance optimization that can be done as a follow-up, not part of this refactor.

## Implementation Steps

- [ ] Step 0: Promote `OutcomeAccumulator` visibility — Change `outcomes` module from `pub(crate)` to `pub` in `mod.rs`. Change `OutcomeAccumulator` struct from `pub(crate)` to `pub` in `outcomes.rs`. Leave `OutcomeRecord` as `pub(crate)` (not in any public signature). Leave `OutcomeAccumulator` methods as `pub(crate)` (only called within v3-core). Required for Rust E0446 compliance: `pub` phase functions cannot reference `pub(crate)` types in their signatures. Run `cargo check` to verify.
- [ ] Step 1: Add `SimStats::reset_per_tick()` method — Extract the ~20 lines of stat zeroing (tick.rs lines 179-199) into a `pub(crate) fn reset_per_tick(&mut self)` method on `SimStats` in `stats.rs`. The reset includes mixed operations: scalar fields set to `0`/`0.0`, `Vec::clear()` for `last_tick_predation_events`, and `HashMap::clear()` calls if any map fields are per-tick. Note: the food depletion/inhibition stats (`last_tick_food_occupancy_depletion_*`, `last_tick_food_cells_with_type_inhibition`, `last_tick_food_growth_suppressed_*`) are also overwritten by `record_food_growth_summary()` inside `run_phase_0`; include them in the reset for safety with a comment noting the redundancy. Completeness check: grep for `last_tick_` fields in the `SimStats` struct and verify all are covered by the reset method. Update `run_tick` to call `sim.stats.reset_per_tick()`. Run full test suite to verify no behavior change.
- [ ] Step 2: Extract `run_phase_1_cognition` — Move sensor assembly (tick.rs lines 240-279) and batch cognition/trace handling (tick.rs lines 282-368) from `run_tick` into a new `pub fn run_phase_1_cognition` with the signature above. Move relevant `use` statements from `run_tick` into the new function (sensor, mesh execution, perception, visibility imports). Update `run_tick` to call it. Run full test suite.
- [ ] Step 3: Extract `run_phase_2_actions` — Move the action execution loop (tick.rs lines 370-762) from `run_tick` into a new `pub fn run_phase_2_actions`. Compute cost accumulators (`compute_total_sum`, `compute_vm_sum`, etc.) and their stat finalization (lines 746-762, writing to `sim.stats`) stay inside this function — they are Phase 2-local concerns. `successful_spawn_targets: HashSet<Position>` also stays local. Move relevant `use` statements (action imports, `WorldAction`, `HashSet`). Update `run_tick` to call it. Run full test suite.
- [ ] Step 4: Extract `run_phase_2_5_reward_learning` — Move the reward learning pass (tick.rs lines 764-824) from `run_tick` into a new `pub fn run_phase_2_5_reward_learning`. Update `run_tick` to call it. Verify `run_tick` is now ~35 lines. Run full test suite + viability tests explicitly (`cargo test -p v3-core --test viability`).
- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke `rust-skills`. Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 10 (Pass 1: 4 dispatches, Pass 2: 4 dispatches, Pass 3: 2 dispatches)
