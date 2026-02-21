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
- Telemetry storage/transport implementation details (owned by
  `v3-evolution-observability-spec.md` semantics only).

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

Phase 0 includes:
- Food growth.
- Creature aging.
- Energy decay.

Only creatures alive after Phase 0 world updates are eligible for the current
tick queue.

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

## 6. Conflict Resolution Contract

All conflict-prone actions resolve against current world state at the moment the
action is applied.

Implications:
- `Move`: first processed successful move claims destination occupancy.
- `Eat`: first processed successful eat consumes target food/resource.
- `Reproduce`: first processed valid spawn into a target cell succeeds;
  later reproduce actions targeting now-invalid cells fail the same invalid-target
  gate.

No deferred global arbitration pass is required.

---

## 7. Newborn Eligibility

Creatures spawned during tick `T`:
- are inserted into world state immediately on successful reproduction action,
- are not added to tick `T` turn queue,
- become eligible for execution starting tick `T + 1`.

---

## 8. Test-Mode Reproducibility Notes (Tick Arbitration)

Project-level determinism scope is canonical in `AGENTS.md`
(`Determinism Scope (Canonical)`): production runtime determinism is not a
product requirement.

For deterministic tests that depend on action-order outcomes, pin:
- Queue source filter (alive after Phase 0 updates only).
- Pre-shuffle ordering rule (stable sorted `CreatureId`).
- Shuffle algorithm and RNG seed/stream for tick queue.
- Per-turn sequencing (`think -> emit action -> apply action`) with no deferred
  commit phases.
- Newborn eligibility rule (next tick only).

Runtime cognition reproducibility controls are canonical in
`v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).

---

## 9. Cross-Reference Map

- Cognition runtime internals: `v3-mesh-execution-spec.md`
- Reproduce action internals: `v3-reproduction-spec.md`
- Mutation event pipeline: `v3-mutation-spec.md`
- Required counters/reasons: `v3-evolution-observability-spec.md`
- Tick/runtime config defaults: `v3-runtime-config-spec.md`
