# V3 Tick Orchestration Spec

Reference specification for canonical per-tick orchestration and world-action
arbitration in V3.

Status: Active

Related references:
- `v3-creature-lifecycle-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-reproduction-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-runtime-config-spec.md`
- `v3-sensor-spec.md`
- `v3-world-grid-spec.md`
- `v3-startup-seeding-spec.md`
- `v3-server-api-protocol-spec.md`
- `v3-cli-contract-spec.md`

---

## 1. Purpose and Scope

This document defines:
- Canonical per-tick phase order.
- Turn-queue construction and randomization contract.
- Immediate action-application semantics.
- Conflict-resolution semantics for world actions.
- Newborn eligibility timing.
- Tick-level test reproducibility controls.

This document does not define:
- Cognition runtime internals (owned by `v3-mesh-execution-spec.md`).
- Reproduction child drafting/mutation internals (owned by
  `v3-reproduction-spec.md` and `v3-mutation-spec.md`).
- World/grid data model, edge-mode semantics, and target-validity primitives
  (owned by `v3-world-grid-spec.md`).
- Startup seeding/founder baseline policy (owned by
  `v3-startup-seeding-spec.md`).
- External transport lifecycle controls (`startup`, `start`, `pause`, `step`)
  (owned by `v3-server-api-protocol-spec.md`).
- Observability semantic requirements (counters/reasons) are owned by
  `v3-evolution-observability-spec.md`.
- Transport/API mapping for observability surfaces is owned by
  `v3-server-api-protocol-spec.md` and `v3-cli-contract-spec.md`.

---

## 2. Ownership Boundaries

```text
[tick orchestrator]
  owns:
    - phase order
    - turn queue source/sort/shuffle
    - per-turn dispatch order
    - newborn eligibility timing
      |
      +--> [runtime mesh executor]
      |      owns cognition execution only
      |      returns WorldAction
      |
      +--> [action executor]
      |      applies WorldAction immediately
      |      mutates world state
      |
      +--> [reproduction helper]
             handles child draft + mutation + spawn validity gate
             during reproduce action application
```

Top-level action ordering/arbitration is owned by tick orchestration, not by
runtime backends or reproduction internals.

---

## 3. Canonical Per-Tick Flow

```text
[Phase 0 world updates]
  -> [Build turn queue from living CreatureIds after Phase 0]
  -> [Stable sort IDs]
  -> [Shuffle with tick RNG]
  -> [Phase 1: Batch cognition — all creatures see frozen post-Phase-0 world snapshot]
       For each creature in queue (parallel-safe):
         [gather inputs from frozen world snapshot]
         [execute cognition runtime -> WorldAction]
       Collect all (CreatureId, WorldAction, ComputeCostReport)
  -> [Phase 2: Sequential action execution]
       For each decision in queue order:
         [apply action to current world state]
  -> [Phase 2.5: Reward-modulated learning pass]
       For each creature with reward-modulated plasticity nodes:
         [compute OutcomeSignalBank from Phase 0 snapshot + Phase 2 outcomes]
         [apply reward-modulated weight updates: dw = lr * outcome * trace]
  -> [tick ends; newborns eligible next tick]
```

Phase 0 sub-steps (canonical order):
1. Occupancy depletion update for food regrowth memory.
2. Food growth (per-cell, world-level), using the updated depletion layer.
3. Creature aging (`age_ticks += 1` for each living creature).
4. Energy decay (`energy -= energy_decay_per_tick` for each living creature).
5. Death removal: remove all creatures with `energy <= 0.0`.

Canonical food growth behavior is owned by `v3-world-grid-spec.md`.
Canonical `energy_decay_per_tick` default is owned by
`v3-runtime-config-spec.md`.
Canonical occupancy depletion behavior and defaults are owned by
`v3-world-grid-spec.md`.

Only creatures surviving Phase 0 (including death removal) are eligible for the
current tick queue.

---

## 4. Queue Contract

- Source set: creatures alive after Phase 0 world updates.
- Pre-shuffle order: stable ascending `CreatureId` sort.
- Randomization: one in-place shuffle using tick RNG.
- Execution order: exactly the shuffled queue order.

No re-shuffle occurs mid-tick.

If a queued creature no longer exists/alive when reached (for example, removed
by earlier action outcomes), its turn is skipped and processing continues.

---

## 5. Two-Phase Action Contract

### Phase 1: Batch Cognition

All creatures resolve sensor inputs from the frozen post-Phase-0 world snapshot
(before any creature actions this tick). Cognition executes for each creature and
produces a `WorldAction`. Cognition only mutates each creature's private state
(energy, memory, graph_state) — it does not modify world state.

Phase 1 is parallel-safe: each creature's cognition is independent with no
shared mutable state. The implementation uses Rayon `par_iter_mut` for
multi-core execution.

### Phase 2: Sequential Action Execution

Decisions are applied in queue order. Each action is applied immediately to
current world state, and resulting mutations persist before the next action.

This is the canonical first-processed-wins model.

### Phase 2.5: Reward-Modulated Learning Pass

After all actions are executed, creatures with reward-modulated plasticity
nodes receive weight updates based on tick outcomes. For each such creature:

1. Compute `OutcomeSignalBank` from Phase 0 energy snapshot and Phase 2
   action results (energy delta, action success rate, damage received,
   offspring spawned).
2. For each reward-modulated edge, apply:
   `dw = learning_rate * outcome_signal[channel] * eligibility_trace[edge]`.
3. Clamp updated weight to `[-weight_clamp, weight_clamp]`.

Phase 0 calls `graph_runtime.begin_tick(&genome.nodes)` to decay initialized
eligibility once and freeze its base. Each successful Phase 1 graph visit
replaces activity from that base, using actual evaluation inputs; activity
contains no learning-rate factor. Skipped modules retain decayed credit,
and failed visits preserve the last successful contribution (or base).
Phase 2.5 applies rewards once per initialized edge even on skipped ticks,
without clearing credit. Zero-delta updates retain their configured charge
and work count. Clock decay initializes no weights and adds no charge.
See the graph backend reference for the exact four activity rules.

Creatures without reward-modulated nodes skip Phase 2.5 entirely.
Newborns spawned during Phase 2 have no outcome accumulator entry and
are naturally excluded.

---

## 6. Action Application Semantics

Each world action is applied at the acting creature's turn per Section 5.

### NoOp

- No world effects.
- Deduct `energy.costs.noop_cost` from creature energy.

### Eat

- Consume food from creature's current cell using selected-type `consume_food_type` semantics
  from `v3-world-grid-spec.md`.
- Gain energy using the shared reward for every ordinary type:
  `energy += consumed_amount * energy.costs.eat_reward_per_food`.
- Clamp energy to `energy.lifecycle.max_energy` before charging the action cost.
- Deduct `energy.costs.eat_cost` from creature energy.

### Move

- Resolve target neighbor using validity primitives from
  `v3-world-grid-spec.md`.
- If valid: update creature position and world occupancy.
- If invalid: no position change.
- Deduct `energy.costs.move_cost` from creature energy regardless of move
  success.

### Reproduce

- Full flow per `v3-reproduction-spec.md` and `v3-runtime-config-spec.md`
  Section 4.
- `energy.costs.reproduce_cost` is deducted within reproduction transfer
  sequencing (not as a separate post-action step).

### Energy after action

Energy may become zero or negative after action application. The creature is
not removed until the next tick's Phase 0 death removal step (Section 3).

Config key defaults for all action costs are canonical in
`v3-runtime-config-spec.md`.

---

## 7. Conflict Resolution Contract

All conflict-prone actions resolve against current world state at the moment the
action is applied (during Phase 2).

Because cognition sees the frozen post-Phase-0 world snapshot (Phase 1), multiple
creatures may independently decide to target the same cell or food during
cognition. Conflicts are resolved during Phase 2 action execution via
first-processed-wins in queue order.

Canonical move/spawn target-validity primitives (edge handling, occupancy, and
barrier checks) are owned by `v3-world-grid-spec.md`.

Implications:
- `Move`: first processed successful move claims destination occupancy.
- `Eat`: first processed successful eat consumes the targeted food/resource
  (including the requested ordinary-food type). Later creatures that decided
  to eat the same cell during cognition will find no food.
- `Reproduce`: first processed valid spawn into a target cell succeeds;
  later reproduce actions targeting now-occupied cells fail the invalid-target
  gate.

No additional global arbitration pass runs after Phase 2.

---

## 8. Newborn Eligibility

Creatures spawned during tick `T`:
- are inserted into world state immediately on successful reproduction action,
- are not added to tick `T` turn queue,
- become eligible for execution starting tick `T + 1`.

---

## 9. Test-Mode Reproducibility Notes (Tick Arbitration)

Project-level determinism scope is canonical in root `AGENTS.md`: production runtime determinism is not a
product requirement.

For deterministic tests that depend on action-order outcomes, pin:
- Queue source filter (alive after Phase 0 updates only).
- Pre-shuffle ordering rule (stable sorted `CreatureId`).
- Shuffle algorithm and RNG seed/stream for tick queue.
- Two-phase sequencing: batch cognition (Phase 1) then sequential action
  execution (Phase 2) in queue order.
- Newborn eligibility rule (next tick only).

Parallel cognition (Phase 1) is deterministic: `execute_creature_mesh` is a
pure function of its inputs (no RNG, no shared mutable state). Results are
collected in original queue order via Rayon's `IndexedParallelIterator::collect`,
which preserves input ordering. Action execution (Phase 2) is sequential and
deterministic given the same decision order.

Runtime cognition reproducibility controls are canonical in
`v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).

---

## 10. Cross-Reference Map

- Cognition runtime internals: `v3-mesh-execution-spec.md`
- Reproduce action internals: `v3-reproduction-spec.md`
- Mutation event pipeline: `v3-mutation-spec.md`
- Required counters/reasons: `v3-evolution-observability-spec.md`
- Tick/runtime config defaults: `v3-runtime-config-spec.md`
- World/grid semantics and validity primitives: `v3-world-grid-spec.md`
- Startup seeding and founder baseline policy: `v3-startup-seeding-spec.md`
- External lifecycle transport contract: `v3-server-api-protocol-spec.md`
