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
  -> [Phase 2.5: Outcome store and reward-modulated learning pass]
       For each living creature:
         [compute OutcomeSignalBank from Phase 0 snapshot + Phase 2 outcomes]
         [store it as the creature's previous outcome; newborns store zeros]
         if it has a reward-modulated edge:
           [apply reward-modulated weight updates: dw = lr * outcome * trace]
  -> [tick ends; newborns eligible next tick]
```

Phase 0 sub-steps (canonical order):
1. Occupancy depletion update for food regrowth memory, then the grazing
   fertility modifier recovery step on every non-barrier cell of every food
   type.
2. Food growth (per-cell, world-level), using the updated depletion layer and
   the recovered grazing modifiers.
3. Creature aging (`age_ticks += 1` for each living creature).
4. Energy decay, genome carrying cost and capacity upkeep (T26.F02), as one
   combined `f32` subtraction for each living creature:
   `base = energy_decay_per_tick + genome_carry_cost_per_unit * genome_size`,
   `upkeep = upkeep_per_unit * (capacity - reference)` (signed: negative for
   a body below the reference), `energy -= max(base + upkeep, 0)`, where
   `genome_size` is the creature's total genome size cached at birth (junk
   included). No term is scaled by the complexity or age action multipliers.
   At `capacity = reference` the upkeep is `0.0` and the charge is `base` bit
   for bit. Booking: `lifecycle_decay` and `genome_carrying` take the
   decay-first partition of `before - base`; `capacity_upkeep` takes the
   applied debit's signed remainder, so the zero floor's shortfall lands
   there. Attribution uses the charge's rounded `f32` intermediates:
   `LifecycleDecay` when `before - decay <= 0`, else `GenomeCarrying` when
   `before - base <= 0`, else `CapacityUpkeep`; a body at the reference is
   never attributed to upkeep.
5. Death removal: remove all creatures with `energy <= 0.0`. A creature the
   combined charge in sub-step 4 takes to or below zero is removed here, in
   the same tick.

Canonical food growth behavior is owned by `v3-world-grid-spec.md`.
Canonical `energy_decay_per_tick`, `genome_carry_cost_per_unit` and
`energy.capacity` defaults are owned by `v3-runtime-config-spec.md`.
Canonical occupancy depletion and grazing behavior and defaults are owned by
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

### Phase 2.5: Outcome Store and Reward-Modulated Learning Pass

After all actions are executed, every living creature stores its tick
outcome, and creatures with a reward-modulated edge on either surface
(T25.F06) receive weight updates based on it. The pass iterates the creatures
directly. For each creature:

1. Compute `OutcomeSignalBank` from Phase 0 energy snapshot and Phase 2
   action results (energy delta, action success rate, damage received,
   offspring spawned), before the Phase 2.5 zero floor (learning is free, T25.F02).
2. Store the bank on the creature (`CreatureState::previous_outcome`,
   T19.F05). The next tick's sensor assembly reads it into
   `StaticInputs::previous_outcome` for the `PreviousOutcome` input
   (`v3-sensor-spec.md` Section 3.7). A creature born this tick has no
   accumulator entry and stores zeros; nothing is inherited.
3. For each initialized reward-modulated edge (its backend visited, or its
   slot seeded at birth), apply:
   `dw = learning_rate * outcome_signal[channel] * eligibility_trace[edge]`.
4. Clamp updated weight to `[-weight_clamp, weight_clamp]`.
5. Floor the creature's energy at `0.0`, crediting `zero_floor_credit`; this
   reaches exactly the creatures that have a bank and a reward-modulated edge.

Phase 0 calls `graph_runtime.begin_tick(age)` to record the dispatch
record's age and decay every trace once over the learned state's modulated
slots; it snapshots no internal state (T19.F02). Each successful Phase 1 graph visit adds its
activity to the trace, using actual evaluation inputs, so a module the pass
dispatches twice adds twice; activity contains no learning-rate factor.
Skipped modules retain decayed credit, and failed visits leave the trace
untouched. Phase 2.5 applies rewards once per initialized edge even on
skipped ticks, without clearing credit. Zero-delta updates retain their
configured charge and work count. Clock decay initializes no weights and
adds no charge. See the graph backend reference for the exact four activity
rules.

Creatures without reward-modulated nodes skip the reward steps (3 and 4).
Newborns spawned during Phase 2 have no outcome accumulator entry and are
excluded from the reward steps.

---

## 6. Action Application Semantics

Each world action is applied at the acting creature's turn per Section 5.

### Action effort

Every committed action charges its effort on every attempt, whatever the
outcome (T16.F02): `effort = adjusted_action_cost(max(base, noop_cost),
complexity, age)` (`EnergyConfig::action_effort`). The actions share the
multiplier, so no action's effort is below `NoOp`'s, and `NoOp`'s is above
zero (`noop_cost > 0`); a base at or above `noop_cost` is charged exactly
`adjusted_action_cost(base, …)`. A failed attempt (a blocked move, an eat
finding no food, a rejected reproduce, a steal with no victim or nothing to
take) pays its effort and nothing more: there is no failed-action penalty. The debit lands in
`energy_flows.action_charges.<action>`, and on a failed attempt also in its
subset `energy_flows.failed_action_effort.<action>`.

### NoOp

- No world effects.
- Deduct the effort of base `energy.costs.noop_cost`.

### Eat

- Consume food from creature's current cell using selected-type `consume_food_type` semantics
  from `v3-world-grid-spec.md`.
- Gain energy using the selected type’s `energy_per_unit`, inheriting the shared
  `energy.costs.eat_reward_per_food` when absent/null:
  `energy += consumed_amount * effective_type_reward`.
- Clamp energy to `energy.lifecycle.max_energy` before charging the action cost.
- Deduct the effort of base `energy.costs.eat_cost`, whether or not food was
  found.

### Move

- Resolve target neighbor using validity primitives from
  `v3-world-grid-spec.md`.
- If valid: update creature position and world occupancy.
- If invalid: no position change.
- Deduct the effort of base `energy.costs.move_cost` regardless of move
  success.

### Reproduce

- Full flow per `v3-reproduction-spec.md` and `v3-runtime-config-spec.md`
  Section 4.
- Step 0 deducts the effort of base `energy.costs.reproduce_attempt_cost`
  before any gate; the replication cost `energy.costs.reproduce_cost` is
  deducted within reproduction transfer sequencing only on a conception (not
  as a separate post-action step).

### StealEnergy

- The intensity is `i = min(1, steal_share × effort)`: the attacker's own
  heritable share times the decoded effort modifier (T27.F03).
- Deduct the effort of base `exchange.effort_rate × i × max(attacker
  energy, 0)` before the target resolves, floored at `NoOp`'s effort.
  Transfer, kill and bonus rules are owned by the predation action.
- The amount is `i × victim.energy` when the victim's energy is positive,
  else 0; a kill happens only at `i = 1` or by rounding. A steal transfers only when the victim's stored energy
  falls; one that moves nothing (a zero amount, an exhausted or `NaN` store,
  an amount `f32` rounding absorbs) fails as `RejectedNothingToTake`: its
  effort is booked as failed, and it kills nothing, credits no bonus and
  records no predation event.

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
