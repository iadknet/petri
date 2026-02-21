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
  -> loop each id:
       [gather inputs from current world]
       [execute cognition runtime -> WorldAction]
       [apply action immediately to current world]
  -> [tick ends; newborns eligible next tick]
```

Phase 0 sub-steps (canonical order):
1. Food growth (per-cell, world-level).
2. Creature aging (`age_ticks += 1` for each living creature).
3. Energy decay (`energy -= energy_decay_per_tick` for each living creature).
4. Death removal: remove all creatures with `energy <= 0.0`.

Canonical food growth behavior is owned by `v3-world-grid-spec.md`.
Canonical `energy_decay_per_tick` default is owned by
`v3-runtime-config-spec.md`.

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

## 5. Immediate Action-Application Contract

Per queued creature turn:
1. Resolve turn-start inputs from current world state.
2. Execute cognition runtime to produce one `WorldAction`.
3. Apply that action immediately to current world state.
4. Persist resulting world mutations before next creature turn.

This is the canonical first-processed-wins model.

---

## 6. Action Application Semantics

Each world action is applied at the acting creature's turn per Section 5.

### NoOp

- No world effects.
- Deduct `energy.costs.noop_cost` from creature energy.

### Eat

- Consume food from creature's current cell using `consume_food` semantics
  from `v3-world-grid-spec.md`.
- Gain energy: `energy += consumed_amount * energy.costs.eat_reward_per_food`.
- Cap energy at `energy.lifecycle.max_energy`.
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
action is applied.

Canonical move/spawn target-validity primitives (edge handling, occupancy, and
barrier checks) are owned by `v3-world-grid-spec.md`.

Implications:
- `Move`: first processed successful move claims destination occupancy.
- `Eat`: first processed successful eat consumes target food/resource.
- `Reproduce`: first processed valid spawn into a target cell succeeds;
  later reproduce actions targeting now-invalid cells fail the same invalid-target
  gate.

No additional global arbitration pass runs after the per-turn loop.

---

## 8. Newborn Eligibility

Creatures spawned during tick `T`:
- are inserted into world state immediately on successful reproduction action,
- are not added to tick `T` turn queue,
- become eligible for execution starting tick `T + 1`.

---

## 9. Test-Mode Reproducibility Notes (Tick Arbitration)

Project-level determinism scope is canonical in `AGENTS.md`
(`Determinism Scope (Canonical)`): production runtime determinism is not a
product requirement.

For deterministic tests that depend on action-order outcomes, pin:
- Queue source filter (alive after Phase 0 updates only).
- Pre-shuffle ordering rule (stable sorted `CreatureId`).
- Shuffle algorithm and RNG seed/stream for tick queue.
- Per-turn sequencing (`think -> emit action -> apply action`) with no separate
  post-loop commit phase.
- Newborn eligibility rule (next tick only).

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
