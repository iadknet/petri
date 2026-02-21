# Petri V3 Creature Lifecycle Reference

Overview/index reference for lifecycle phases and high-level invariants in V3
mesh architecture.

Status: Active

Authoritative detailed contracts:
- `v3-tick-orchestration-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-genome-spec.md`
- `v3-mesh-execution-spec.md`

---

## 1. Lifecycle Phases

At a high level per tick:
1. Phase 0 world updates run (food growth, aging, energy decay).
2. Turn queue is built from creatures alive after Phase 0, then stable-sorted
   by `CreatureId` and shuffled with tick RNG.
3. Each queued creature executes one turn in order:
   - gather turn-start inputs from current world state,
   - execute cognition runtime to emit one `WorldAction`,
   - apply action immediately and mutate world state.
4. Tick ends; any newborn spawned mid-tick becomes eligible on next tick only.

Canonical phase order, queue contract, and arbitration semantics are specified in
`v3-tick-orchestration-spec.md`.

---

## 2. Mutation Policy Summary

V3 uses a junk-DNA-friendly mutation policy.

Core rule:
- Mutation does not perform global cascading cleanup to force behavioral
  validity.

Examples of allowed degraded topology:
- Deleting node `A` does not require removing `A` from all target lists.
- `entry_node_id` may be left dangling.
- Graph internals may contain invalid edges.

Runtime soft defaults (see `v3-mesh-execution-spec.md`) map these conditions to
safe behaviors (`NoOp` or `0.0`) rather than panics.

Detailed mutation ownership, event pipeline, and invalid-event policy are
specified in `v3-mutation-spec.md`.

---

## 3. Structural Validity Summary

Mutation processing enforces structural parseability, not behavioral viability.

Canonical parseability invariants are specified in `v3-genome-spec.md` (Section
4) and applied by mutation `ParseabilityGate` in `v3-mutation-spec.md`.

Canonical runtime fallback behavior for degraded-but-parseable genomes is
specified in `v3-mesh-execution-spec.md` (Section 4, authoritative soft-default
matrix).

---

## 4. Reproduction Inheritance Summary

### Memory

- Offspring memory is copied byte-for-byte from parent at draft creation.
- Parent and child memory diverge after spawn.

### Graph state

- Offspring does not inherit parent graph runtime state.
- Offspring starts with empty graph-state map.
- Graph state is lazily allocated per `NodeId` as nodes execute.

### Inventory

- Offspring inventory is empty at spawn unless future features define
  inheritance.

### Phenotype

- On no mutation event, offspring phenotype matches parent.
- On mutation event, phenotype changes follow mutation policy.

Detailed inheritance and immediate reproduce-action spawn semantics are specified
in `v3-reproduction-spec.md`.

---

## 5. Observability Summary

Minimal required counters/events/reason enums for mutation skips and
reproduction action outcomes are specified in
`v3-evolution-observability-spec.md`.

---

## 6. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick queue/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
