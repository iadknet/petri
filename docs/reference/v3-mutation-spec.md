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

Topology connection semantics (T11.F15, T11.F18):

- `AddNode` and `SpliceNode` split a valid existing edge through a blank
  Graph or minimal VM `Halt` detour, preserving that edge's slot and bias. The detour forwards to
  the old successor and preserves incoming output slots, queued actions,
  priority and shared memory. Attachment is atomic; no valid edge means skip.
  All three detour growth operators choose Graph or VM with probability 0.5,
  independently of the source backend, using the existing mutation RNG after
  eligibility and attachment preparation. Graph detours have configured unwired
  fixed outputs and no compute nodes; VM detours have one register and `Halt`.
  Both add a mesh hop when visited. Unwired Graph dispatch has no compute
  charge; a zero-compute Graph with a wired effect surface is entered and pays
  `graph_node_base_cost * max(compute_nodes.len(), 1)`. VM dispatch retains its
  Halt charge. Their genome sizes add two and three
  units respectively. State neutrality excludes exhausted budgets and downstream
  live energy introspection; node-local metadata stays local.
- `AddRouteTarget` selects a node with exactly one valid non-self successor,
  appends a tied-bias branch to a fresh blank detour forwarding to that successor,
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
  an explicit whole-brain macro with weight 1. Since 2026-09-18 every other
  topology weight is scaled by ten (refinement 40, moderate 20, structural
  10), so the macro is drawn in 1 of 211 topology events rather than 1 of 22:
  on a chain-shaped founder an entry moved past the action nodes queues
  nothing, the one topology edit that still reads dead after T11.F15.
  Eligibility skips are reported as skips, never counted as applied silent
  mutations.
- The former `topology_new_node_birth` configuration is removed: inline
  detours have fixed Halt births. Both backend kinds remain available through
  alternate growth. No other supply or pressure setting changes.

### VM domain

- `VmInstructionMutation` (insert/delete/replace opcode, mutate operands);
  the fresh-instruction draw is uniform over the 42-opcode catalog, which
  includes `WriteDirectionBid { direction in 0..8, src }` (T11.F21), and the
  operand nudge moves its `direction` or `src`; a new `PushAction` draws
  `action_type` in `0..=4`, the range
  `decode_world_action` admits (T13.F05); existing values above 4 stay in
  the genome and the raw-field unit step is unchanged
- `VmConstantMutation`
- `VmRegisterCountMutation`
- `VmInstructionRawFieldMutation` (raw representable-field mutation for
  tolerant runtime decoders, including fields such as `action_type` and
  immediate memory addresses)
- `VmInsertReadStoreMotif` (insert ReadInput + StoreSlotImm instruction pair)
- `VmInsertLoadCompareMotif` (insert LoadSlotImm + CmpGt instruction pair)
- `VmCopyInstructionBlock` — a growth operator; copies a contiguous block of
  2 to 32 instructions, clamped to the program, into the dormant tail span
  below
- `VmCopyGeneBackwardSlice` — a growth operator; copies the backward
  dependency slice of one instruction into the dormant tail span
- `VmCopyGeneForwardSlice` — a growth operator; copies the forward
  dependency slice of one instruction into the dormant tail span
- `VmCopyInstructionBlockRemapped` — an explicit macro, not a growth
  operator: it copies a block inline at a random position with every register
  field cyclically shifted, so it changes behavior at the moment it fires and
  is measured as a behavior-changing operator
- `VmCopyConstantBlock` — an append-only copy of a constant-pool block, and
  outside the growth class: `LoadConst` resolves
  `const_idx.rem_euclid(constants.len())` (`runtime/vm.rs`), so a `const_idx`
  at or above the old pool length resolves to a different constant after the
  append. It is silent when it fires unless a `const_idx` wraps the pool
- `VmMutateSlotAddress` (mutate slot_idx on an existing shared-memory slot
  opcode)
- `VmMutatePairedSlotAddress` (co-mutate all LoadSlotImm/StoreSlotImm
  instructions sharing the same slot_idx to a new random slot)

VM structural-edit contract:

- Applicable targets (T13.F03): `VmDeleteInstruction` needs a program of more
  than one instruction; the block and constant copies need a non-empty
  program or constant pool; the gene slices need an output instruction
  (backward) or a register-writing instruction (forward); the `ReadInput`
  motifs need an input reference; `VmMutateSlotAddress` needs a slot
  instruction and `VmMutatePairedSlotAddress` a slot carrying both a load and
  a store. `VmConstantMutation`, `VmInstructionMutation` and
  `VmInsertLoadCompareMotif` apply to any VM module, including a dormant one.

- Every instruction insertion, deletion, replacement, or copy resolves old
  jump targets using the VM runtime's signed relative-target rule before the
  edit and re-encodes surviving references afterward. Insertion preserves old
  instruction identities; replacement preserves its incoming position; a
  deleted target follows the next survivor, wrapping at the tail.
- A copied jump follows a copied selected target and otherwise follows the
  surviving original target. This is based on source indices, never instruction
  equality, so repeated identical instructions remain distinct. Newly authored
  instructions retain their authored offsets.
- `VmCopyInstructionBlock`, `VmCopyGeneBackwardSlice`, and
  `VmCopyGeneForwardSlice` place their copy in a dormant span at the program
  tail (T11.F08). Every surviving jump keeps its old resolved target through
  the repair above, so nothing outside the span jumps into it; when the
  program's last instruction is neither `Halt` nor `ExecuteActionQueue`, a
  newly authored `Halt` guard is spliced immediately before the copied span in
  the same event, so fall-through halts exactly where running past the old
  program's end used to halt. A later jump mutation — an offset stepped by one
  unit, or an inserted or replaced jump — is the only way the span becomes
  reachable, short of a mutation removing or replacing the guard or the
  program's final terminal, and it then runs in its original's place. Neutrality here is
  behavioral and holds under ample budget: executing the guard costs one
  `Halt` step, and the longer program can reach the step cap or exhaust energy
  where the original did not.
- `VmInstructionRawFieldMutation` changes one encoded operand by one bounded
  unit and never replaces the opcode. Fieldless instructions are excluded
  from the draw, and a program of nothing but fieldless instructions is not
  an applicable target. The paired
  slot-address operator remains a linked-address macro; the standalone slot
  address operator moves one address by one unit.
- Register capacity changes only between widths 1 and 32. Raw register fields
  are canonicalized under the old width. The direction is drawn among the
  feasible moves only: a program in which some read or write uses the
  register a shrink would remove can still grow, and grows instead of
  skipping. A def with neither move available — a width outside 1..=32, or
  width 32 with the shrink blocked (growth is never blocked at width 1) —
  is not an applicable target.

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
   80% of triggered births request one event, with a bounded configurable tail,
   and each event's target is drawn with `mutation.executed_bias` toward the
   mesh nodes the parent's brain dispatched within `executed_window_ticks`,
   leaving a uniform residual across every other eligible live and inactive
   node.

T11.F02 establishes the VM reference and operand portions. T11.F03 owns graph
growth, T11.F04 mutation supply, T11.F06 the graph state clock, T11.F07 the
trace/reward clock, T11.F09 learned-state correspondence, and T11.F15 the
topology connection operators (`AddRouteTarget`, `MutateGateBias`,
`RetargetNodeTarget`, `RemoveRouteTarget`, `RemoveNode`, `SwapNodeBackend`,
`ChangeEntryNode`, `SwapRouteTargets`) and mesh attachment semantics above.
T11.F17 owns the target draw of requirement 4: the former "uniform opportunity
across eligible live and inactive mesh nodes" is now a draw biased toward the
nodes the parent's brain executed in recent ticks, with a uniform residual
(Section 4.3). The per-birth event count, operator weights, and operator
semantics are unchanged.

T11.F08 owns duplication on all three backends: VM dormant-tail placement and
its terminal guard, graph copy placement and the copy self-edge rule, the
split exclusion, and the mesh post-activation qualification. A copy is
required to be neutral when it fires *and* to reproduce its original when a
later mutation runs it in the original's place. Learned-weight correspondence
through such a copy follows T11.F09: birth-local available values move with each
compute input occurrence through graph/mesh splices and faithful copies, including
edge bundles. Rollback restores them with the genome. New/replaced backends or
edges have no parent origin, even at a reused mesh ID or index. Source deletion,
actual direct retargeting (including raw source-field changes), and explicit
edge-weight mutation clear that edge's origin. Pure index repair and copy remaps
preserve it; identity split preserves the consumer origin and creates a new
identity edge at its genomic weight. Compute kind/parameter, input-reference
meaning, routing, and plasticity changes preserve surviving edge occurrences.
Tracking consumes no RNG and does not affect selection, semantic counts,
parseability, serialization, or genome identity. The final child flag and
[reproduction inheritance rules](v3-reproduction-spec.md#graph-state) determine
whether tracked values materialize as runtime weights.

Growth-versus-connection taxonomy (requirement 2, established for the graph
and InputRef domains by T11.F03; the VM insert/copy families are T11.F02's
and T11.F08's): a growth operator adds structure and must be neutral at the
moment it fires — identical action, output-slot, and shared-memory behavior
when both executions have enough energy and relaxation passes. Growth
operators: `AddComputeNode` (all three forms below), `CopyComputeNode`,
`CopySubgraph`, `InputRef.Add`, the VM `VmCopyInstructionBlock`,
`VmCopyGeneBackwardSlice`, and `VmCopyGeneForwardSlice`,
and the topology `AddNode`, `CopyNode`, mesh slices, `SpliceNode`,
`AddRouteTarget`, and `SwapNodeBackend`. `VmCopyInstructionBlockRemapped` is
not in this class: the register-renamed inline copy is an explicit
behavior-changing macro, as `CopyEdgeBundle` and the paired slot-address
operator are. `VmCopyConstantBlock` is not in this class either: it is
append-only and silent in practice, but a `const_idx` that wraps the constant
pool resolves differently after the append, so it is not neutral by
construction. T11.F15 owns the topology attachment and paired-routing
guarantees; F08 owns copy placement and post-activation qualification on
every backend. A connection or parameter operator may change
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

- `AlterGraphEdgeWeight` (all 6 edge-bearing surfaces: compute inputs, sink
  inputs, action gate, action param, action direction bank (T11.F21), execute
  gate)
- `SwapGraphOperator` (compute nodes only, 17 `ComputeNodeKind` variants)
- `MutateGraphOperatorParam` (compute params: Constant, Threshold,
  DecayIntegrator, Momentum, Oscillator)
- `MutateActionSlotBehavior` (mutates `action_bank[i].behavior` between `Pop`
  and `Emit(WorldActionKind)` variants)
- `AddComputeNode(kind)` — a growth operator; draws one of three
  function-preserving forms with equal probability: (a) disconnected, a
  random kind appended with no inputs; (b) bootstrap, a random kind appended
  with one input edge from `random_graph_source`, read by no surface; (c)
  split, a NEAT-style insertion into one existing edge (any of the six
  surfaces, source any `GraphSource`) with an identity `Add` node whose
  single input (weight 1.0) reproduces the split edge's prior source exactly
  in f32 (the new node's output passes through `sanitize_output` like every
  compute node's, so exact reproduction holds for values already within its
  NaN→0/±1e9-clamp range), and the old edge retargeted to the new node at its
  old weight. The split draws from the splittable edges only. When the split
  edge's consumer is a compute node at index `c`,
  the new node is inserted at index `c` and every `ComputeNode(i >= c)`
  reference is remapped to `i + 1` across all six surfaces
  (`CgpGraphBackendDef::insert_compute_node_at`, the insert-with-remap
  inverse of `remove_compute_node_at`), preserving Gauss-Seidel pass order; a
  sink/action/execute-gate consumer appends instead. A split of a backward or
  self edge may extend convergence by at most one pass. An edge whose source
  is an out-of-range `ComputeNode` (a prior removal's sentinel) is not
  splittable and is excluded from the draw; when the graph has no splittable
  edge the split form is not offered and `AddComputeNode` draws one of the
  other two forms, both of which apply to any graph.
  **Split exclusion** (T11.F08, replacing T11.F03's documented exception): an
  edge is also unsplittable when the graph carries plasticity, the edge's
  consumer is a sink, action slot, or execute gate, and its source is an
  `InputLeaf` resolving to
  `DynamicIntrospection(EnergyCurrent)`. An identity node between them caches
  the value during evaluation, while the direct edge resolves it in the
  post-convergence effects context after the plasticity-cost deduction, so the
  two can differ by `plasticity_cost * weight` (no observable effect under the
  production default `plasticity_update_cost = 0.0`, but the split is not
  function-preserving in general). `EnergyCurrent` is the only excluded key:
  in `runtime/cgp/execute.rs` the effects `ResolveCtx` differs from the
  evaluation `ResolveCtx` only in `energy`, so `EnergyConsumedThisTick`
  resolves identically either side of the deduction and its edges split normally. With the exclusion in place, the
  split's neutrality property holds unconditionally.
- `RemoveComputeNode` (removes from `compute_nodes`, remaps
  `GraphSource::ComputeNode` indices across all edge containers)
- `AddGraphEdge` (all 6 edge-bearing surfaces, `pick_random_surface` uniform
  over one surface per compute node and sink, three per action slot (gate,
  param, direction bank), and the execute gate; a direction-bank edge draws
  its `direction` uniformly in `0..8`; source sampled by
  `random_graph_source`, which draws a compound `InputLeaf` source's
  `sub_idx` uniformly across the reference's full width via
  `mutation::compound::sub_value_count`, so new edges can reach every
  sub-value, not just index 0)
- `RetargetGraphEdge` (all 6 edge-bearing surfaces; same `sub_idx` sampling
  as `AddGraphEdge`)
- `RemoveGraphEdge` (all 6 edge-bearing surfaces)
- `GraphRawFieldMutation` — selects one parameterized compute node or one
  edge uniformly, then changes exactly one field by one unit and never
  replaces the `GraphSource` variant: a parameter by the existing
  `MutateGraphOperatorParam` step; `ComputeNode(idx)` by ±1 inward at the
  bounds `0..compute_count`; `InputLeaf.ref_idx` by ±1 inward within
  `0..input_refs.len()`, offered only when the current `sub_idx` stays within
  the candidate reference's width; `InputLeaf.sub_idx` by ±1 inward within the
  reference's width; `SharedMemory.slot` by ±1 modulo 16; `SharedMemory.previous`
  flipped; a direction-bank edge's `direction` by ±1 inward within `0..8`
  (one more one-unit field beside its source moves; a `direction >= 8` edge
  offers no direction move). Edges with no valid unit move are excluded from
  the draw, so the
  operator selects only a module that has a parameterized compute node or a
  movable edge, and never skips after selecting one.
- `CopyComputeNode` — a growth operator; inserts a faithful copy (kind,
  inputs, plasticity) of one random compute node directly after its source
  and nothing else. The copy reads whatever its source read and is read by
  nothing; no backlink is added, and inputs are never coin-flip cleared (a
  copy that should start disconnected is `AddComputeNode`'s disconnected
  form). Placing the copy at `source + 1` rather than at the end is what
  keeps every copied edge on its original's evaluation phase (T11.F08): under
  the T11.F06 clock a lower-index source is read from the current visit and a
  self or higher-index source from the frozen tick-start outputs, so a source
  below the original must stay below the copy and a source above must stay
  above. The copy's own inputs take the same index shift as every other
  surviving reference, and a self-edge on the copy reads the copy, so the
  copy's persistent state is its own.
- `CopySubgraph` — a growth operator; copies a random-walk cluster of target
  size 2 to 4 compute nodes, which may be a single node when the seed has no
  compute neighbours, inserting the `i`-th sorted member's copy at final
  index
  `c_i + i + 1` with all final indices computed before any insertion. Edges
  between cluster members, including self-edges, are remapped onto the
  copies; external sources keep their logical target at its shifted index.
  Copied nodes start as dead genes: nothing reads them until a connection
  operator does, and when every edge that read a member from outside the
  cluster is retargeted to that member's copy, the copies reproduce the
  originals. Both copy operators reserve room for their copies before
  selection, so a graph that would produce an index at or above the
  `u16::MAX` dangling-reference sentinel is not an applicable target.
- `CopyEdgeBundle` (copies one compute node's edge set onto another; the
  source is drawn from the nodes that have edges, and a graph with fewer than
  two compute nodes or no edge-bearing node is not an applicable target)
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
- `Prune` (T11.F22) — deletes one entry that no consumer addresses: no
  `GraphSource::InputLeaf { ref_idx }` on any edge container and no VM
  `ReadInput { ref_idx }` anywhere in the program, live or not. Higher
  `ref_idx` values are decremented on both backends, so every consumer
  resolves to the same reference afterwards and none is removed or rewritten;
  the event is neutral at birth. A node with no unreferenced entry is not a
  target.
- `Swap` (T11.F22) — replaces one entry with another member of its kind,
  drawn uniformly from the others. The kind is the entry's mesh read class
  (`Food`, `Barrier`, `Occupancy`, `Neighbor`, `Introspection`, `Upstream`,
  `ActionQueue`) and compound width, so a food ring may become another type's
  food ring, an energy scalar an age scalar, an upstream slot another slot,
  never a scalar for a ring or a barrier ring for a food ring. Rings,
  summaries, neighbor blocks and the action queue are the only members of
  their kind and are never swapped; typed food is swappable only with more
  than one food type. The width never changes, so no edge falls out of range.
  An entry keeps its kind for the node's life and its copies'.
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
apply_mutations(genome, mutation_config, parent_reachable_nodes,
                parent_executed, rng_ctx) -> MutationSummary
```

Call semantics:
- `MutationEngine` is called unconditionally for every offspring. Internally,
  it draws the requested event count from the supply rule (production: one
  Bernoulli trial per `genome_size()` unit at `per_unit_rate`, T11.F19). If
  the count is zero, it returns a zero-event summary immediately. Callers
  never skip the `MutationEngine` call.
- `parent_reachable_nodes` is a sorted ascending slice of node indices that
  were reachable in the parent's genome (computed via BFS from entry node).
  Used for reachability-biased target selection (see Section 4.3).
- `parent_executed` names the parent's recently executed nodes: either an
  explicit sorted index slice (observation harnesses) or the live parent's
  dispatch record read at its current age. It is resolved to indices only
  after the supply draw requests at least one event, so a zero-event birth
  derives nothing.

`MutationSummary` minimum fields:
- `attempted_events: u32`
- `applied_events: u32`
- `skipped_events: u32`
- `skip_reasons: map<MutationSkipReason, u32>`
- `attempted_by_domain: map<MutationDomain, u32>`
- `applied_by_domain: map<MutationDomain, u32>`
- `attempted_by_operator: map<MutationOperator, u32>`
- `applied_by_operator: map<MutationOperator, u32>`
- `reachable_target_events: u32`
- `unreachable_target_events: u32`
- `executed_target_events: u32`
- `not_applicable_events: u32`

Accounting invariant:
- `attempted_events = applied_events + skipped_events`.

### 4.2 Event processing sequence

```text
for each selected event:
  1) choose mutation domain
  2) choose operator (may fail under complexity restriction if the
     domain has no eligible operators — skip with NoApplicableTarget)
  2b) build the operator's applicable set: the nodes carrying a site this
      operator can act on (Section 5 pre-guards). An empty set skips with
      NoApplicableTarget before any draw, so the event records no target
  2c) select mutation target from that applicable set with executed and
      reachability bias (Section 4.3)
  3) run the remaining domain pre-guards (construction constraints)
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
- Production supply (T11.F19, `per_unit_supply_enabled = true`): read the
  parent's `genome_size()` once, draw one Bernoulli trial at `per_unit_rate`
  per unit, and request one event per success, so the count is
  `Binomial(genome_size(), per_unit_rate)` with no trigger, minimum, maximum,
  or continuation. Rate 0.0 requests nothing (every birth is a clone); rate
  1.0 requests one event per unit. The default 0.005 keeps the 111-unit
  founder at about 0.555 requested events per birth; a genome pays its size
  in exposure. Natural analog: per-base copy error. If the count is zero,
  `MutationEngine` returns `MutationSummary { attempted_events: 0,
  applied_events: 0, skipped_events: 0, ... }` immediately.
- Legacy per-birth supply (`per_unit_supply_enabled = false`, disabled in
  production; the drift walk, the recruitment-paths experiment, and T11.F13
  use it as the fixed-count control): roll `mutation_probability`; if the roll
  fails, return the zero summary. If triggered, request the configured
  minimum, then continue with the configured
  `per_birth_mutation_event_continuation_probability` until the first failed
  draw or the maximum count. Defaults are trigger 0.44, continuation 0.2,
  bounds 1–10: approximately 0.55 requested events per birth, with 80% of
  triggered births requesting exactly one. Its RNG consumption is unchanged
  from before T11.F19, so the walk's rows are reproducible byte for byte.
- Every requested event becomes one attempted event; a skip does not cause an
  extra requested event. `attempted_events = applied_events + skipped_events`.
- For each event, select topology with `mesh_layer_probability`; otherwise
  select VM, graph, or input-reference mutation with equal probability.
- Select operators with their existing weights, preserving eligibility and
  pressure handling.
- A node-internal operator (VM, Graph) selects from the nodes on which it can
  actually apply, not from every node of its backend: its applicability
  predicate is the same site enumeration the operator draws from, so a
  selected node never reports `NoApplicableTarget` afterwards (T13.F03). An
  operator with no applicable node anywhere in the genome is discarded for
  the event with no target recorded, and the engine retries the domain's
  other operators as before.
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

### 4.3 Executed and reachability bias

Mutation target selection runs two layers over the operator's eligible set:
an executed layer (T11.F17) at `mutation.executed_bias`, then the established
reachability layer at the domain's `ReachableBiasConfig` threshold.

For VM and Graph operators the eligible set is the operator's applicable set
— the backend's nodes filtered by the operator's applicability predicate
(T13.F03) — so both biases apply within the modules that carry a suitable
site. For Topology and for InputRef `Add` the eligible set is unchanged;
InputRef `Swap` and `Prune` filter per entry (a node with a swappable entry, a
node with an unreferenced entry) before the draw, so neither can select a
node and then find no site.

Executed layer (`TargetSelector::select`), applied in all four domains:
1. An empty eligible set selects nothing.
2. When `executed_bias` is `0.0`, or when every eligible node is executed, the
   layer cannot change the outcome: it consumes no RNG and the draw is exactly
   the reachability layer's. A draw whose eligible set is entirely executed is
   therefore byte-identical to the pre-feature draw, so single-event founder
   births are identical, while a multi-event founder birth whose earlier event
   adds a node diverges from that draw on. Both the executed and reachable
   sets are fixed once per birth in the parent's node indices; after a
   mid-birth `RemoveNode`, later indices in both sets are stale by one for the
   rest of that birth (the drift harness maps by `NodeId` and is not affected).
3. Otherwise roll RNG once against `executed_bias`. On success, and when
   `eligible ∩ executed` is non-empty, pick uniformly from that intersection.
   On a failed roll or an empty intersection, fall through to the
   reachability layer with no further executed-layer draws.
4. While genome-size pressure restricts a birth the executed bias is `0.0`,
   so the inverted reachability bias keeps pruning unreachable structure and
   the executed core is never targeted for removal.

A node counts as executed when the parent's dispatch record holds a dispatch
for that mesh node index less than `mutation.executed_window_ticks` ticks
before the parent's current age. The record is written once per mesh hop from
the shared mesh executor, so traced and untraced execution agree and
observation clones (which run on cloned per-creature state) never write it. A
newborn starts with an empty record: indices belong to its own genome.
Observation harnesses (`neighborhood`) substitute the node ids the subject
dispatches across the fixed battery.

Reachability layer (`biased_select_from`), unchanged:
1. Roll RNG against the domain's bias probability.
2. On success: compute intersection of eligible and reachable sets via sorted
   two-pointer merge. If intersection is non-empty, pick uniformly from it
   (target is `Reachable`). If empty, fall through to uniform selection.
3. On failure (or fallthrough): pick uniformly from eligible set. Classify
   picked index via binary search in reachable set → `Reachable` or
   `Unreachable`.

Defaults: `executed_bias` 0.9; per-domain reachable bias topology=0.0,
vm=0.0, graph=0.0, input_ref=0.0. The 0.1 residual keeps drawing uniformly
over every eligible mesh node, including inactive scaffold, so neutral
scaffold keeps drifting; there is no fixed quota for any class. See
`v3-runtime-config-spec.md` for config fields.

Every picked target is classified against the reachable set, whichever layer
picked it, and a picked index that is in the executed set also increments
`executed_target_events` — a membership count parallel to the reachable
classification, not a count of layer firings.

Exempt operators:
- Topology `ChangeEntryNode` does not select a target node and returns
  `NotApplicable`. `AddNode` now selects an edge source and is classified.
- InputRef `RawFieldMutation` draws uniformly across every node's input
  references rather than selecting a mesh node, and returns `NotApplicable`.
  Neither layer covers it.

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

Every VM and Graph operator's pre-guard is an applicability predicate over one
node, evaluated before selection: it names the sites the operator will draw
from, and application draws only from those sites (T13.F03). The predicate is
the single source of truth for an operator's sites on a node — a new or
extended operator adds its sites to the predicate rather than failing after
the draw. InputRef `Swap` and `Prune` carry the same kind of predicate over
the node's `input_refs` entries: a swappable entry (another member of its
kind exists) and an unreferenced entry (no edge or `ReadInput` addresses it)
respectively (T11.F22).

If no node is applicable, the event is skipped with `NoApplicableTarget` and
records no target.

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
