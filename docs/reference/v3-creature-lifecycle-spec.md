# Petri V3 Creature Lifecycle Reference

Overview/index reference for lifecycle phases and high-level invariants in V3
mesh architecture.

Status: Active

Authoritative detailed contracts:
- `v3-tick-orchestration-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-phenotype-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-genome-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-startup-seeding-spec.md`
- `v3-server-api-protocol-spec.md`
- `v3-cli-contract-spec.md`

---

## 1. Lifecycle Phases

At a high level per tick:
1. Phase 0 world updates run (food growth, aging, energy decay, death removal).
2. Turn queue is built from creatures alive after Phase 0, then stable-sorted
   by `CreatureId` and shuffled with tick RNG.
3. Each queued creature executes one turn in order:
   - gather turn-start inputs from current world state,
   - execute cognition runtime to emit one `WorldAction`,
   - apply action immediately and mutate world state.
4. Tick ends; any newborn spawned mid-tick becomes eligible on next tick only.

Canonical phase order, queue contract, and arbitration semantics are specified in
`v3-tick-orchestration-spec.md`.
Startup/reset seeding behavior and founder baseline policy are specified in
`v3-startup-seeding-spec.md`.

---

## 2. CreatureId Contract

`CreatureId` is a `u64` monotonically incrementing identifier.

Rules:
- Each creature receives a unique `CreatureId` at spawn time (including
  startup-seeded founders).
- IDs are never reused within a simulation run.
- Allocation is monotonically incrementing from `0`.
- `CreatureId` is used for occupancy indexing, turn-queue stable sort, and
  observability event attribution.

---

## 3. Death and Removal

A creature dies when its energy reaches zero or below.

Death removal is processed during Phase 0 of each tick, after energy decay is
applied. Creatures that reach zero or negative energy during cognition or action
application within a tick are not removed until the following tick's Phase 0.

Canonical Phase 0 sub-step ordering (including death removal) is specified in
`v3-tick-orchestration-spec.md` Section 3.

---

## 4. Mutation Policy Summary

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

## 5. Structural Validity Summary

Mutation processing enforces structural parseability, not behavioral viability.

Canonical parseability invariants are specified in `v3-genome-spec.md` (Section
4) and applied by mutation `ParseabilityGate` in `v3-mutation-spec.md`.

Canonical runtime fallback behavior for degraded-but-parseable genomes is
specified in `v3-mesh-execution-spec.md` (Section 4, authoritative soft-default
matrix).

---

## 6. Reproduction Inheritance Summary

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
- Phenotype state model, mutation algorithm, and trigger rules are canonical in
  `v3-phenotype-spec.md`.

Detailed inheritance and immediate reproduce-action spawn semantics are specified
in `v3-reproduction-spec.md`.

---

## 7. Observability Summary

Minimal required counters/events/reason enums for mutation skips and
reproduction action outcomes are specified in
`v3-evolution-observability-spec.md`.
Canonical server/ws transport mapping for observability data is specified in
`v3-server-api-protocol-spec.md`; canonical CLI run-output mapping is specified
in `v3-cli-contract-spec.md`.

---

## 8. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick queue/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
