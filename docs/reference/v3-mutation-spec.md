# V3 Mutation Spec

Reference specification for mutation responsibilities, operator domains, and
event processing policy in V3.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-reproduction-spec.md`
- `v3-evolution-observability-spec.md`

---

## 1. Purpose and Scope

This document defines:
- Mutation ownership boundaries.
- Mutation event pipeline.
- Structural validity checks after mutation events.
- Invalid-event handling policy.
- Required mutation telemetry semantics.

This document does not define:
- Runtime mesh execution semantics.
- Reproduction action cost rules.
- Storage or transport format for telemetry.

---

## 2. Responsibility Boundaries

```text
[MutationEngine]
  owns:
    - whether mutation triggers
    - event count selection
    - domain/operator selection
    - event sequencing
    - rollback/skip policy
    - mutation summary aggregation
      |
      +--> [TopologyMutator]
      |      node/target/entry structure mutations
      |
      +--> [VmMutator]
      |      VM instruction/operand/constant mutations
      |
      +--> [GraphMutator]
      |      graph internal node/edge/operator/param mutations
      |
      +--> [PhenotypeMutator] (optional when enabled)
      |      phenotype trait/color mutations
      |
      +--> [ParseabilityGate]
             enforces structural validity only
             (no behavioral viability guarantees)
```

Core rule:
- `MutationEngine` orchestrates mutation events.
- Domain mutators only mutate their own domain-specific payloads.
- `ParseabilityGate` evaluates structural validity after each event.

---

## 3. Mutation Domains and Operator Families

### Topology domain

- `AddNode`
- `RemoveNode`
- `RetargetNodeTarget`
- `AddRouteTarget`
- `RemoveRouteTarget`
- `ChangeEntryNode`

### VM domain

- `VmInstructionMutation` (insert/delete/replace opcode, mutate operands)
- `VmConstantMutation`

### Graph domain

- `AddInternalGraphNode(kind)`
- `RemoveInternalGraphNode`
- `AlterGraphEdgeWeight`
- `SwapGraphOperator`
- `MutateGraphOperatorParam`

### Phenotype domain (optional)

- Domain-specific phenotype mutation operations, when enabled for the stage.

Implementations may add operators, but they must preserve structural
parseability or be rolled back/skipped under policy below.

---

## 4. Event Pipeline

### 4.1 Engine contract

```text
apply_mutations(genome, mutation_config, rng_ctx) -> MutationSummary
```

`MutationSummary` minimum fields:
- `attempted_events: u32`
- `applied_events: u32`
- `skipped_events: u32`
- `skip_reasons: map<MutationSkipReason, u32>`

### 4.2 Event processing sequence

```text
for each selected event:
  1) choose mutation domain + operator
  2) run domain pre-guards (construction constraints)
  3) snapshot local mutation target (or full genome)
  4) apply candidate mutation
  5) run ParseabilityGate
  6) if parseability fails:
       rollback event
       mark skipped(ParseabilityViolation)
       continue
  7) commit event
```

---

## 5. Pre-Guards vs Post-Apply Parseability

Both are required.

### Pre-guards (inside each domain mutator)

Prevent obviously invalid events before apply, for example:
- Do not remove the last genome node.
- Do not choose mutation targets from empty candidate sets.
- Keep type-level payload construction valid for the target domain.

If no target can be selected, event is skipped with `NoApplicableTarget`.

### Post-apply parseability check (global)

After each event, run `ParseabilityGate`.

Minimum required parseability invariants:
- `nodes` is non-empty.
- `node_id` values are unique.
- backend payloads remain decodable.

Not required for parseability:
- `entry_node_id` resolves.
- all route targets resolve.
- all graph internal edges are runtime-valid.

These non-required conditions are handled by runtime soft defaults.

---

## 6. Invalid Event Policy

If an event results in non-parseable genome:
1. rollback that event,
2. mark event skipped with reason `ParseabilityViolation`,
3. continue with remaining events.

No behavioral repair is required.
No global topology rewrite is required.

### `MutationSkipReason` minimum enum

- `ParseabilityViolation`
- `NoApplicableTarget`
- `BudgetExhausted`

---

## 7. Required Telemetry Hooks

Mutation processing must emit sufficient data for the minimal contract in
`v3-evolution-observability-spec.md`.

At minimum:
- Per-offspring mutation summary.
- Per-event outcome (applied or skipped).
- Skip reason when skipped.
- Domain and operator identity for each attempted event.

---

## 8. Test-Mode Reproducibility (Optional)

Production behavior is not required to be deterministic across runs.

For deterministic tests, harnesses may pin:
- RNG seed.
- event-domain/operator traversal order.
- mutation budget settings.
- tie-breaking conventions inside domain mutators.
