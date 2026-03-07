# V3 Mutation Spec

Reference specification for mutation responsibilities, operator domains, and
event processing policy in V3.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-reproduction-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-runtime-config-spec.md`
- `v3-phenotype-spec.md`

---

## 1. Purpose and Scope

This document defines:
- Mutation ownership boundaries.
- Mutation event pipeline.
- Structural validity checks after mutation events.
- Invalid-event handling policy.
- Required mutation telemetry semantics.
- Randomized mutation selection semantics (trigger, event count, domain/operator
  sampling).

This document does not define:
- Runtime mesh execution semantics.
- Reproduction action cost rules.
- Storage or transport format for telemetry.
- Canonical mutation config field defaults (see `v3-runtime-config-spec.md`).

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
- `SwapNodeBackend`
- `RewriteNodeId`
- `CopyNode`
- `CopyMeshBackwardSlice`
- `CopyMeshForwardSlice`
- `SpliceNode`
- `SwapRouteTargets`

### VM domain

- `VmInstructionMutation` (insert/delete/replace opcode, mutate operands)
- `VmConstantMutation`
- `VmRegisterCountMutation`
- `VmInstructionRawFieldMutation` (raw representable-field mutation for
  tolerant runtime decoders, including fields such as `action_type` and
  immediate memory addresses)
- `VmInsertReadStoreMotif` (insert ReadInput + StoreSlotImm instruction pair)
- `VmInsertLoadCompareMotif` (insert LoadSlotImm + CmpGt instruction pair)
- `VmMutateSlotAddress` (mutate slot_idx on an existing shared-memory slot
  opcode)
- `VmMutatePairedSlotAddress` (co-mutate all LoadSlotImm/StoreSlotImm
  instructions sharing the same slot_idx to a new random slot)

### Graph domain

- `AlterGraphEdgeWeight`
- `SwapGraphOperator`
- `MutateGraphOperatorParam`
- `AddInternalGraphNode(kind)`
- `RemoveInternalGraphNode`
- `AddGraphEdge`
- `RetargetGraphEdge`
- `RemoveGraphEdge`
- `GraphRawFieldMutation` (raw representable-field mutation for tolerant graph
  encodings, including `InputRef { ref_idx, sub_idx }`, `CustomOutput(u8)`,
  `ReadSlot(u8)`, `ReadSlotPrev(u8)`, `WriteSlot(u8)`, `ClearSlot(u8)`,
  and edge source indices)
- `CopyInternalNode`
- `CopySubgraph`
- `CopyEdgeBundle`
- `EnableHebbian` (add `PlasticityConfig` to a non-plasticity node)
- `DisableHebbian` (remove `PlasticityConfig` from a plasticity node)
- `MutateHebbianRule` (change the `HebbianRule` variant)
- `MutateHebbianRate` (perturb the `learning_rate`)
- `ToggleHebbianLamarckian` (flip the `lamarckian` inheritance flag)
- `EnableRewardModulation` (add `RewardModulationConfig` to a pure Hebbian node)
- `DisableRewardModulation` (remove reward modulation from a modulated node)
- `MutateRewardSource` (change the `OutcomeChannel` a modulated node listens to)
- `MutateTraceDecay` (perturb the `trace_decay` rate on a modulated node)

### InputRef domain

- `Add`
- `Remove`
- `Swap`
- `RawFieldMutation` (raw representable-field mutation for tolerant
  `UpstreamSlot(usize)` values)

Phenotype mutation is not a mutation engine domain. It is a separate pathway
triggered by genome mutation; see `v3-phenotype-spec.md`.

Implementations may add operators, but they must preserve structural
parseability or be rolled back/skipped under policy below.

---

## 4. Event Pipeline

### 4.1 Engine contract

```text
apply_mutations(genome, mutation_config, parent_reachable_nodes, rng_ctx) -> MutationSummary
```

Call semantics:
- `MutationEngine` is called unconditionally for every offspring. Internally,
  it rolls `mutation_probability` to decide whether any mutation events are
  attempted. If the probability gate fails, it returns a zero-event summary
  immediately. Callers never skip the `MutationEngine` call.
- `parent_reachable_nodes` is a sorted ascending slice of node indices that
  were reachable in the parent's genome (computed via BFS from entry node).
  Used for reachability-biased target selection (see Section 4.3).

`MutationSummary` minimum fields:
- `attempted_events: u32`
- `applied_events: u32`
- `skipped_events: u32`
- `skip_reasons: map<MutationSkipReason, u32>`
- `attempted_by_domain: map<MutationDomain, u32>`
- `applied_by_domain: map<MutationDomain, u32>`
- `attempted_by_operator: map<MutationOperator, u32>`
- `applied_by_operator: map<MutationOperator, u32>`
- `applied_semantic_noop_events: u32`
- `applied_semantic_change_events: u32`
- `reachable_target_events: u32`
- `unreachable_target_events: u32`
- `not_applicable_events: u32`

Accounting invariant:
- `attempted_events = applied_events + skipped_events`.

### 4.2 Event processing sequence

```text
for each selected event:
  1) choose mutation domain
  2) choose operator (may fail under complexity restriction if the
     domain has no eligible operators — skip with NoApplicableTarget)
  2b) select mutation target with reachability bias (see Section 4.3)
  3) run domain pre-guards (construction constraints)
  4) snapshot local mutation target (or full genome)
  5) apply candidate mutation
  6) run ParseabilityGate
  7) if parseability fails:
       rollback event
       mark skipped(ParseabilityViolation)
       continue
  8) commit event; classify target as Reachable/Unreachable/NotApplicable
```

Selection randomization rules (internal to `MutationEngine`):
- Mutation trigger rolls global `mutation_probability` internally. If the roll
  fails, `MutationEngine` returns `MutationSummary { attempted_events: 0,
  applied_events: 0, skipped_events: 0, ... }` immediately.
- If triggered, event count is sampled from configured inclusive min/max bounds.
- For each event, domain is sampled uniformly across enabled domains
  (`Topology`, `Vm`, `Graph`, `InputRef`).
- For each event, operator is sampled uniformly across the selected domain's
  operators.
- Operator-specific mutation fields are randomized per event according to that
  operator's mutator implementation.
- Canonical owner for mutation config keys/defaults: `v3-runtime-config-spec.md`.

Complexity pressure gate:
- When `genome_size_pressure_enabled` is true and the genome's total structural
  size (`genome_size()`) exceeds `genome_size_cap`, the engine restricts mutation
  domains to decreasing-only operators.
- The pressure gate uses `genome_size()` (total structural size including
  unreachable/dead code), not `complexity()` (functional reachability-aware).
  This prevents runaway structural bloat even when junk DNA does not affect
  action energy costs.

### 4.3 Reachability bias

Mutation target selection is biased toward reachable (functional) mesh nodes
using per-domain probability thresholds from `ReachableBiasConfig`.

Algorithm (`biased_select_from`):
1. Build eligible set for the operator (domain-specific filtering).
2. Roll RNG against the domain's bias probability.
3. On success: compute intersection of eligible and reachable sets via sorted
   two-pointer merge. If intersection is non-empty, pick uniformly from it
   (target is `Reachable`). If empty, fall through to uniform selection.
4. On failure (or fallthrough): pick uniformly from eligible set. Classify
   picked index via binary search in reachable set → `Reachable` or
   `Unreachable`.

Per-domain bias defaults: topology=0.7, vm=0.7, graph=0.7, input_ref=0.5.
See `v3-runtime-config-spec.md` for config fields.

Exempt operators:
- Topology `AddNode` and `ChangeEntryNode` do not select a target node. They
  return `NotApplicable` for reachability classification.

`TargetReachability` classification:
- `Reachable`: selected target was in the parent's reachable set.
- `Unreachable`: selected target was not in the parent's reachable set.
- `NotApplicable`: operator does not perform target selection.

Design note: The parent's cached reachable set is used to bias mutations on
the offspring. After mutations, the offspring's actual reachable set may differ.
This is intentional — mutations are biased toward what was functional in the
parent. The offspring receives a fresh BFS computation at birth.

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

Required parseability invariants are defined authoritatively in
`v3-genome-spec.md` (Section 4) and must be applied verbatim by
`ParseabilityGate`.

Not required for parseability:
- `entry_node_id` resolves.
- all route targets resolve.
- all graph internal edges are runtime-valid.

These non-required conditions are handled by runtime soft defaults; canonical
chain-level fallback behavior is in
`v3-mesh-execution-spec.md` (Section 4, authoritative soft-default matrix).

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
- Semantic category identity (`SemanticNoop` vs `SemanticChange`) for each
  applied event.

Semantic category rule:
- `Topology.RewriteNodeId` is semantic-noop-capable and must be counted under
  `SemanticNoop` when applied.
- Applied semantic-noop events still count as applied mutation events and remain
  eligible to trigger phenotype mutation through reproduction policy.

---

## 8. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
