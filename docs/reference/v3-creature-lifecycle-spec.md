# Petri V3 Creature Lifecycle Reference

Reference specification for mutation, reproduction, and phenotype inheritance in
V3 mesh architecture.

Status: Active

---

## 1. Lifecycle Phases

At a high level per tick:
1. Existing creatures act.
2. Reproduction requests are generated.
3. Offspring are created and queued.
4. Spawn queue is committed after action processing.

This preserves stable keys while actions are executing.

---

## 2. Mutation Policy

V3 uses a **junk-DNA-friendly mutation policy**.

Core rule:
- Mutation operators do not perform global cascading cleanup to force logical
  validity.

Examples:
- Deleting node `A` does not require removing `A` from every other node's
  `targets`.
- A mutation may leave `entry_node_id` dangling.
- Graph internals may contain invalid edges.

Runtime soft-default behavior (see `v3-mesh-execution-spec.md`) safely maps these
cases to `NoOp` or `0.0` instead of panicking.

Rationale:
- Preserve dormant and partially broken structures for long-term evolutionary
  novelty.
- Let selection pressure remove non-viable offspring behaviorally.

---

## 3. Mutation Operator Families

### Mesh-topology operators

- `AddNode`
- `RemoveNode`
- `RetargetNodeTarget`
- `AddRouteTarget`
- `RemoveRouteTarget`
- `ChangeEntryNode`

### VM operators

- `VmInstructionMutation` (insert/delete/replace opcode, mutate operands)
- `VmConstantMutation`

### Graph-backend operators

- `AddInternalGraphNode(kind)`
- `RemoveInternalGraphNode`
- `AlterGraphEdgeWeight`
- `SwapGraphOperator`
- `MutateGraphOperatorParam`

### Genome-parameter operators

- `GenomeParamMutation` (for evolvable top-level knobs)

Implementation can extend this set, but operators must keep data structures
parseable.

---

## 4. Structural Invariants

Mutation pipeline must enforce minimal parseability invariants:
- `nodes` remains non-empty
- `node_id` values remain unique
- backend payloads remain decodable

Intentionally not required:
- `entry_node_id` resolves
- all targets resolve
- all graph edges are valid

These are runtime-handled conditions under soft defaults.

---

## 5. Repair Strategy

V3 avoids aggressive repair that rewrites offspring topology semantics.

Allowed repairs are structural only (for example resolving duplicate ids).
Behavioral/topological cleanup (like forced target rewiring) is not required.

If mutation cannot produce a parseable genome representation, the mutation event
may be skipped or retried according to implementation policy.

---

## 6. State Inheritance on Reproduction

### Memory

- Offspring memory is copied byte-for-byte from parent at reproduction commit.
- Parent and child memory diverge after spawn.

### Graph State

- Offspring does not inherit parent graph runtime state.
- Offspring starts with empty graph-state map.
- Graph state is lazily allocated per `NodeId` as nodes execute.

### Inventory

- Offspring inventory is empty at spawn unless future features define
  inheritance.

---

## 7. Phenotype Inheritance

Founder baseline:
- `rgb = [204, 61, 61]`
- `channel_weights = [1.0, 1.0, 1.0]`
- `positive_increment = [true, true, true]`

On no mutation event, offspring phenotype matches parent exactly.
On mutation event, channel-level mutation applies with deterministic RNG for a
fixed simulation seed.

---

## 8. Determinism Requirements

- Mutation event ordering is deterministic for a fixed RNG seed.
- Operator selection and parameter perturbation are deterministic for that seed.
- Reproduction commit order is deterministic within tick orchestration.
