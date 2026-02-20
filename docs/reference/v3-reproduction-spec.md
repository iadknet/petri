# V3 Reproduction Spec

Reference specification for reproduction flow, offspring drafting, and spawn
commit arbitration in V3.

Status: Active

Related references:
- `v3-creature-lifecycle-spec.md`
- `v3-mutation-spec.md`
- `v3-genome-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-runtime-config-spec.md`

---

## 1. Purpose and Scope

This document defines:
- Reproduction action flow from action execution through spawn commit.
- Offspring draft contract.
- Inheritance rules.
- Spawn arbitration and rejection semantics.
- Required reproduction telemetry semantics.

This document does not define:
- VM/graph cognition internals.
- Mutation operator internals (covered in `v3-mutation-spec.md`).
- Telemetry storage/transport implementation details.

---

## 2. Reproduction Ownership

```text
[tick/actions]
  validates reproduce action + energy transfer
  builds OffspringDraft (no world mutation)
  calls MutationEngine on child genome
  emits SpawnCandidate
         |
         v
[spawn queue]
  deferred commit after action processing
         |
         v
[spawn commit]
  owns authoritative target-cell validity check
  first valid candidate claims target cell
  invalid targets rejected with one failure result
```

---

## 3. OffspringDraft Contract

`OffspringDraft` minimum fields:

- `position` (target spawn position)
- `initial_energy` (post-transfer child energy, scalar `f32`)
- `generation` (`parent.generation + 1`)
- `phenotype` (inherited/mutated according to mutation policy)
- `genome` (parent genome copy after mutation application)
- `memory` (byte-for-byte copy from parent)
- `graph_state` (empty map at spawn)

Constraints:
- Draft creation must not mutate world occupancy state.
- Draft creation must not allocate creature identity key; identity is assigned on
  spawn commit.

---

## 4. Inheritance Rules

### Memory

- Child memory is copied byte-for-byte from parent at draft creation.
- Parent and child memory diverge after spawn.

### Genome

- Child genome starts as parent genome copy.
- Mutation is applied through `MutationEngine` before enqueue.

### Graph state

- Child graph runtime state is not inherited.
- Child starts with empty graph-state map.
- Graph state is lazily allocated at runtime by `NodeId` as nodes execute.

### Inventory

- Child inventory is empty at spawn unless future feature docs specify
  inheritance.

---

## 5. Energy Transfer Contract

Reproduction action semantics:
- Parent pays reproduction action cost according to energy config.
- Requested child transfer is clamped by configured offspring transfer cap.
- If parent cannot satisfy required transfer constraints, reproduction fails and
  no child draft is queued.
- Energy/lifecycle config values are continuous scalar units (`f32`) as defined
  in `v3-runtime-config-spec.md`.

This document defines semantics only; exact field names/constants live in core
runtime config contract: `v3-runtime-config-spec.md`.

---

## 6. Spawn Queue and Commit Arbitration

### Queue semantics

- Spawn candidates are queued during action execution.
- World mutation for child occupancy is deferred until spawn commit phase.

### Commit semantics

Commit uses a single validity gate in queue order:

1. Evaluate `is_valid_spawn_target(position, world_state_now)` at commit time.
2. If valid, spawn candidate and occupy the cell.
3. If invalid, reject `RejectedInvalidTarget`.

`is_valid_spawn_target` returns false when target cell is out of bounds,
barrier-blocked, already occupied, or otherwise not spawnable.

No pre-pass or cascading rejection-order policy is required. Same-tick
contention is handled naturally by commit order: once one candidate claims a
cell, later candidates for that cell fail the same invalid-target gate.

### Ownership boundary

- Spawn target validity is owned by spawn commit.
- Action execution/draft creation may run advisory prechecks for UX/perf, but
  those prechecks are not authoritative.
- Authoritative acceptance/rejection telemetry must be emitted at spawn commit.

### `SpawnCommitResult` minimum enum

- `Spawned`
- `RejectedInvalidTarget`

---

## 7. Required Telemetry Hooks

Reproduction processing must emit required minimal data defined in
`v3-evolution-observability-spec.md`.

At minimum:
- Reproduction attempt count.
- Spawn candidates queued count.
- Spawned count.
- Rejected count by `SpawnCommitResult` reason.
- Optional per-candidate event records for debugging.

---

## 8. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 harness reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
