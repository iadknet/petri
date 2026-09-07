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
- Domain mutators own their payloads; paired topology growth also writes its source backend gate.
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
- `CopyNode`
- `CopyMeshBackwardSlice`
- `CopyMeshForwardSlice`
- `SpliceNode`
- `SwapRouteTargets`
- `MutateGateBias`

Topology connection semantics (T11.F15):

- `AddNode` and `SpliceNode` split a valid existing edge through a minimal VM
  `Halt` detour, preserving that edge's slot and bias. The detour forwards to
  the old successor and preserves incoming output slots, queued actions,
  priority and shared memory. Attachment is atomic; no valid edge means skip.
- `AddRouteTarget` selects a node with exactly one valid non-self successor,
  appends a tied-bias branch to a fresh Halt detour forwarding to that successor,
  and writes the branch's free gate slot in the same event. Graphs gain one
  weight-1 edge from `random_graph_source` (including full sensor sub-values).
  VMs gain a `WriteRouteGate` from a uniformly sampled existing register before
  the first Halt/ExecuteActionQueue or at the end, with reference repair.
  Missing gate sinks, zero-register VMs and failed insertion skip atomically.
  Existing orphan gate writes remain. Equal bids retain the old earlier target;
  a varying new bid can immediately select the equivalent detour.
- `CopyNode` faithfully copies inputs, backend and outgoing targets. Self IDs
  follow the copy; external destinations, memory and output addresses stay.
  Attach as a tied later alternative at an existing predecessor whose static
  incumbent points to the original. Incumbent and new slots must have no gate
  writes, the incumbent bias must be finite, and no original-to-predecessor
  path may exist. Select the first slot unused by both targets and backend
  writes. These conditions prove dormancy even with single-visit filtering.
- `SwapNodeBackend` uses the same attachment proof to grow an alternate copy
  with the other blank backend, preserving the original. `SwapRouteTargets`
  exchanges only destination IDs to activate an alternative; slots, biases
  and vector positions remain. Activation may change behavior.
- Mesh-slice copies retain their copy semantics but preflight an attachment
  before appending anything. Every growth operator allocates unused IDs,
  including at integer wrap. General copy and inheritance qualification remain
  T11.F08/F09's scope.
- `RetargetNodeTarget` samples uniformly from the deduplicated union of the
  old successor's successors and the source's other destinations, excluding
  source, current and missing IDs. No global fallback; empty local choices skip.
- `RemoveRouteTarget` requires two targets and preserves the earliest
  highest-bias static incumbent and all surviving slots/biases. A removed
  branch may still have been a runtime winner.
- `RemoveNode` protects entry and last node, prefers currently unreachable
  nodes, otherwise requires a valid non-self static successor. Redirect every
  incoming ID through that successor, preserving route fields/order. For an
  unreachable node without a bypass, delete its incoming entries. Eligibility
  uses fresh traversal, while selection/classification retain the parent cache.
- `RewriteNodeId` is retired from the live catalog. `ChangeEntryNode` remains
  an explicit whole-brain macro with weight 1. Other weights remain unchanged;
  topology total weight is 22 across 13 operators. Eligibility skips are
  reported as skips, never counted as applied silent mutations.
- The former `topology_new_node_birth` configuration is removed: inline
  detours have fixed Halt births. Both backend kinds remain available through
  alternate growth. No other supply or pressure setting changes.

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

VM structural-edit contract:

- Every instruction insertion, deletion, replacement, or copy resolves old
  jump targets using the VM runtime's signed relative-target rule before the
  edit and re-encodes surviving references afterward. Insertion preserves old
  instruction identities; replacement preserves its incoming position; a
  deleted target follows the next survivor, wrapping at the tail.
- A copied jump follows a copied selected target and otherwise follows the
  surviving original target. This is based on source indices, never instruction
  equality, so repeated identical instructions remain distinct. Newly authored
  instructions retain their authored offsets.
- `VmInstructionRawFieldMutation` changes one encoded operand by one bounded
  unit and never replaces the opcode. Fieldless instructions skip. The paired
  slot-address operator remains a linked-address macro; the standalone slot
  address operator moves one address by one unit.
- Register capacity changes only between widths 1 and 32. Raw register fields
  are canonicalized under the old width, and a shrink skips when any read or
  write uses the removed effective register. Widths outside that range skip
  without changing the genome.

### Node-type evolvability contract

Every current or future mesh backend must meet these four requirements:

1. references are stable by identity or remapped on every structural edit;
2. every growth operation preserves function at the moment it fires;
3. persistent state advances once per world tick: graph temporal reads and
   reward eligibility use an explicit boundary. Initialized eligibility decays
   even on skipped visits; successful revisits replace activity from one frozen
   decayed base, using actual evaluation inputs. Reward gain applies eta once
   to activity-only credit; failed visits never overwrite successful activity;
   and
4. mutation supply arrives as small steps: at provisional production defaults,
   80% of triggered births request one event, with a bounded configurable tail
   and uniform opportunity across eligible live and inactive mesh nodes.

T11.F02 establishes the VM reference and operand portions. T11.F03 owns graph
growth, T11.F04 mutation supply, T11.F06 the graph state clock, T11.F07 the
trace/reward clock, T11.F08 duplication, T11.F09 learned-state
correspondence, and T11.F15 the topology connection operators
(`AddRouteTarget`, `MutateGateBias`, `RetargetNodeTarget`,
`RemoveRouteTarget`, `RemoveNode`, `SwapNodeBackend`, `ChangeEntryNode`,
`SwapRouteTargets`) and mesh attachment semantics above. Broader T11.F08/F09
guarantees remain pending.

Growth-versus-connection taxonomy (requirement 2, established for the graph
and InputRef domains by T11.F03; the VM insert/copy families are T11.F02's
and T11.F08's): a growth operator adds structure and must be neutral at the
moment it fires — identical action, output-slot, and shared-memory behavior
when both executions have enough energy and relaxation passes. Growth
operators: `AddComputeNode` (all three forms below), `CopyComputeNode`,
`CopySubgraph`, `InputRef.Add`, and the topology `AddNode`, `CopyNode`, mesh
slices, `SpliceNode`, `AddRouteTarget`, and `SwapNodeBackend`. T11.F15 owns
the topology attachment and paired-routing guarantees; F08 retains general
copy qualification. A connection or parameter operator may change
behavior, and must do so in one small step: `AddGraphEdge`,
`RetargetGraphEdge`, `RemoveGraphEdge`, `CopyEdgeBundle`, `SwapGraphOperator`,
`MutateGraphOperatorParam`, `GraphRawFieldMutation`, `InputRef.Swap`, and
`InputRef.RawFieldMutation` are all in this class. Topology connection operators
are `RetargetNodeTarget`, `RemoveRouteTarget`, `RemoveNode`, `MutateGateBias`,
`SwapRouteTargets`, and the explicit macro `ChangeEntryNode`, all owned by
T11.F15. The retired identity rename is outside this live taxonomy. `CopyEdgeBundle` remains an
explicit multi-edge macro, as the VM's paired-slot-address operator is. A
grown node still costs `graph_node_base_cost` per relaxation pass; growth
neutrality does not extend to energy exhaustion.

### Graph domain

Topology mutations operate on `compute_nodes` only. Fixed structural outputs
(output sinks, action bank, execute gate) are never added/removed/retyped —
only their edges are evolvable.

- `AlterGraphEdgeWeight` (all 5 edge-bearing surfaces: compute inputs, sink
  inputs, action gate, action param, execute gate)
- `SwapGraphOperator` (compute nodes only, 17 `ComputeNodeKind` variants)
- `MutateGraphOperatorParam` (compute params: Constant, Threshold,
  DecayIntegrator, Momentum, Oscillator)
- `MutateActionSlotBehavior` (mutates `action_bank[i].behavior` between `Pop`
  and `Emit(WorldActionKind)` variants)
- `AddComputeNode(kind)` — a growth operator; draws one of three
  function-preserving forms with equal probability: (a) disconnected, a
  random kind appended with no inputs; (b) bootstrap, a random kind appended
  with one input edge from `random_graph_source`, read by no surface; (c)
  split, a NEAT-style insertion into one existing edge (any of the five
  surfaces, source any `GraphSource`) with an identity `Add` node whose
  single input (weight 1.0) reproduces the split edge's prior source exactly
  in f32 (the new node's output passes through `sanitize_output` like every
  compute node's, so exact reproduction holds for values already within its
  NaN→0/±1e9-clamp range), and the old edge retargeted to the new node at its
  old weight. When the split edge's consumer is a compute node at index `c`,
  the new node is inserted at index `c` and every `ComputeNode(i >= c)`
  reference is remapped to `i + 1` across all five surfaces
  (`CgpGraphBackendDef::insert_compute_node_at`, the insert-with-remap
  inverse of `remove_compute_node_at`), preserving Gauss-Seidel pass order; a
  sink/action/execute-gate consumer appends instead. A split of a backward or
  self edge may extend convergence by at most one pass. Skips with
  `NoApplicableTarget` when the graph has no edge, or when the picked edge's
  source is an out-of-range `ComputeNode` (a prior removal's sentinel).
  **Documented exception**: an append-branch split retargeting an
  `InputLeaf(DynamicIntrospection(EnergyCurrent))` edge, on a graph carrying
  plasticity, is not neutral — the new node's cached value is read at effects
  time before the post-convergence plasticity-cost deduction, while a direct
  edge on the same non-compute surface reads energy after it, differing by
  `plasticity_cost * weight`. No observable effect under the production
  default `plasticity_update_cost = 0.0`; see the T11.F03 spec.
- `RemoveComputeNode` (removes from `compute_nodes`, remaps
  `GraphSource::ComputeNode` indices across all edge containers)
- `AddGraphEdge` (all 5 edge-bearing surfaces; source sampled by
  `random_graph_source`, which draws a compound `InputLeaf` source's
  `sub_idx` uniformly across the reference's full width via
  `mutation::compound::sub_value_count`, so new edges can reach every
  sub-value, not just index 0)
- `RetargetGraphEdge` (all 5 edge-bearing surfaces; same `sub_idx` sampling
  as `AddGraphEdge`)
- `RemoveGraphEdge` (all 5 edge-bearing surfaces)
- `GraphRawFieldMutation` — selects one parameterized compute node or one
  edge uniformly, then changes exactly one field by one unit and never
  replaces the `GraphSource` variant: a parameter by the existing
  `MutateGraphOperatorParam` step; `ComputeNode(idx)` by ±1 inward at the
  bounds `0..compute_count`; `InputLeaf.ref_idx` by ±1 inward within
  `0..input_refs.len()`, offered only when the current `sub_idx` stays within
  the candidate reference's width; `InputLeaf.sub_idx` by ±1 inward within the
  reference's width; `SharedMemory.slot` by ±1 modulo 16; `SharedMemory.previous`
  flipped. Skips with `NoApplicableTarget` when the picked target has no valid
  unit move.
- `CopyComputeNode` — a growth operator; pushes a faithful copy (kind,
  inputs, plasticity) of one random compute node and nothing else. The copy
  reads whatever its source read and is read by nothing; no backlink is
  added, and inputs are never coin-flip cleared (a copy that should start
  disconnected is `AddComputeNode`'s disconnected form).
- `CopySubgraph` (copies compute node cluster; internal edges remapped,
  external edges preserved; copied nodes start as dead genes)
- `CopyEdgeBundle` (copies edge set between surfaces)
- `EnableHebbian` (add `PlasticityConfig` to a non-plasticity compute node)
- `DisableHebbian` (remove `PlasticityConfig` from a plasticity compute node)
- `MutateHebbianRule` (change the `HebbianRule` variant)
- `MutateHebbianRate` (perturb the `learning_rate`)
- `ToggleHebbianLamarckian` (flip the `lamarckian` inheritance flag)
- `EnableRewardModulation` (add `RewardModulationConfig` to a pure Hebbian node)
- `DisableRewardModulation` (remove reward modulation from a modulated node)
- `MutateRewardSource` (change the `OutcomeChannel` a modulated node listens to)
- `MutateTraceDecay` (perturb the `trace_decay` rate on a modulated node)

### InputRef domain

- `Add` — a growth operator; pushes to `input_refs` and wires nothing on
  either backend (no VM `ReadInput` auto-insertion, no graph edge). The new
  `GraphSource::InputLeaf` source, or VM `ReadInput { ref_idx }` reference,
  becomes addressable only by a later connection operator (`AddGraphEdge` or
  `RetargetGraphEdge` on the graph backend; the VM operators that construct
  `ReadInput` on the VM backend).
- `Remove` (removes from `input_refs`; walks all edge containers to remove
  edges where `ref_idx == removed` and decrements `ref_idx` for edges where
  `ref_idx > removed`)
- `Swap` (replaces an input_ref variant; updates edges where `sub_idx` exceeds
  new width across all edge containers)
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
- If triggered, request the configured minimum, then continue with the configured
  `per_birth_mutation_event_continuation_probability` until the first failed
  draw or the maximum count. Defaults are trigger 0.44, continuation 0.2,
  bounds 1–10: approximately 0.55 requested events per birth, with 80% of
  triggered births requesting exactly one. This is a provisional comparison
  baseline, not an optimum or an inherited-policy minimum (T11.F13, T08.F05).
- Every requested event becomes one attempted event; a skip does not cause an
  extra requested event. `attempted_events = applied_events + skipped_events`.
- For each event, select topology with `mesh_layer_probability`; otherwise
  select VM, graph, or input-reference mutation with equal probability.
- Select operators with their existing weights, preserving eligibility and
  pressure handling.
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

Per-domain bias defaults: topology=0.0, vm=0.0, graph=0.0, input_ref=0.0.
Selection is uniform over eligible mesh nodes, including inactive scaffold;
there is no fixed quota for the reachable and unreachable classes. Existing
neutral growth operators supply scaffold without artificial founder bloat.
See `v3-runtime-config-spec.md` for config fields.

Exempt operators:
- Topology `ChangeEntryNode` does not select a target node and returns
  `NotApplicable`. `AddNode` now selects an edge source and is classified.

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
- All live operators are `SemanticChange` families after identity rename
  retirement. This label does not imply a changed phenotype: the neighborhood
  battery separately measures actual silent/changed/dead outcomes.
- Applied semantic-noop events still count as applied mutation events and remain
  eligible to trigger phenotype mutation through reproduction policy.

---

## 8. Policy References

- Project-level determinism scope is canonical in root `AGENTS.md`.
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).

## 9. Mutation Supply Observations

The neighborhood report records birth counts by requested (`attempted_events`)
count in `by_requested_events`, including zero, separately from `by_events`
bucketed by applied count. Their weighted sums give requested and applied
supply; the difference is skipped supply. `any_events` contains conditional
outcomes among births with applied events. Divide its silent, changed, and dead
counts by `births_total` for absolute mutated outcomes per all births; add
`zero_event_births` to silent counts when reporting all behavior-identical births.
Lower conditional harm alone does not establish fewer dead births overall.

### Applied graph temporal clock (T11.F06)

The node-type persistent-state contract now uses frozen world-tick operator
state and compute outputs. Ordered combinational paths compute within one
visit; self/higher-index edges cross the tick boundary. Repeated visits cannot
accelerate temporal state and unvisited modules hold it. Disconnected node
growth cannot change an existing node's clock. See `v3-graph-backend-spec.md`.
