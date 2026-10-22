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
- `AddFeedbackRoute`
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
- `AddRouteTarget` selects a node with exactly one valid successor (the node
  itself qualifies since T19.F02; the detour then forwards to the node, and a
  sole self-target still has no route exit because the resolver always
  selects an existing target), appends a tied-bias branch to a fresh blank
  detour forwarding to that successor,
  and writes the branch's free gate slot in the same event. Graphs gain one
  weight-1 edge from `random_graph_source` (including full sensor sub-values).
  VMs gain a `WriteRouteGate` from a uniformly sampled existing register before
  the first `Halt` or at the end, with reference repair.
  Missing gate sinks, zero-register VMs and failed insertion skip atomically.
  Existing orphan gate writes remain. Equal bids retain the old earlier target;
  a varying new bid can immediately select the equivalent detour.
- `AddFeedbackRoute` (T11.F29) appends a back edge as a losing branch. The
  source's static incumbent must name an existing node with a finite bias and
  an unwritten slot; the branch takes the first slot unused by targets and
  backend writes and needs a possible gate write (VM registers, Graph
  `RouterGate(slot)` sink). Destinations are the source itself plus, when the
  parent executed the source, the executed nodes with a target naming it and
  the executed nodes naming one of those, minus the source's current targets,
  in node-index order; a parent without a dispatch record self-loops only.
  The branch bias is the incumbent's minus 1.0, stepped down one ulp until
  `bias + 1.0 <= incumbent` in `f32`, so a gate value of at most 1.0 on the
  new slot never selects it; larger VM, compute or memory gate values can.
  The gate write follows `AddRouteTarget`'s construction; draws are the
  source, the gate source, then the destination. Ineligible sources skip
  atomically.
- `CopyNode` faithfully copies inputs, backend and outgoing targets. Self IDs
  follow the copy; external destinations, memory and output addresses stay.
  Attach as a tied later alternative at an existing predecessor whose static
  incumbent points to the original. Incumbent and new slots must have no gate
  writes, the incumbent bias must be finite, and no original-to-predecessor
  path may exist. Select the first slot unused by both targets and backend
  writes. These conditions prove dormancy: a tied later alternative never
  wins the static resolver, whether or not the mesh revisits the predecessor
  within a tick (T19.F02 legalized revisits without changing the tie rule).
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
  only the current and missing IDs; the source node itself is a legal
  destination (T19.F02). No global fallback; empty local choices skip.
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
  10), so the macro is drawn in 1 of 221 topology events (211 before T11.F29's
  `AddFeedbackRoute` at weight 10) rather than 1 of 22:
  on a chain-shaped founder an entry moved past the action nodes queues
  nothing, the one topology edit that still reads dead after T11.F15.
  Eligibility skips are reported as skips, never counted as applied silent
  mutations.
- The former `topology_new_node_birth` configuration is removed: inline
  detours have fixed Halt births. Both backend kinds remain available through
  alternate growth. No other supply or pressure setting changes.

### VM domain

- `VmInstructionMutation` (insert/delete/replace opcode, mutate operands);
  the fresh-instruction draw is uniform over all 39 opcodes (T19.F04), which
  include `AddVote { sink in 0..30, src }` (the sink drawn uniformly over the
  vote catalog, the three kind drives included, T11.F30), `WriteActionParam { field_idx, src }`, whose `field_idx` is
  one uniform draw over `ActionParamField::ALL` (`0..4`, T11.F27, T02.F08), and
  `ReadActionQueueParam`, whose `param_slot` is drawn from `0..2`, the queue
  parameter slots that carry a value (T11.F27); the operand nudge moves an
  `AddVote`'s `sink` or `src`, and the raw-field move can still step a
  `field_idx` to `4` or more, a write the VM ignores
- `VmConstantMutation` — one constant `c` of the pool, drawn uniformly, moves
  by `u × max(|c|, 1)` with `u` uniform in [−0.1, 0.1] (the graph parameter
  step; T11.F23): a unit-scale constant moves as a graph parameter does, a
  raw-scale constant by at most a tenth of its magnitude; the step may cross
  zero and is not clamped. An empty pool gains one constant drawn from
  [−1, 1]
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
  program's last instruction is not `Halt`, a
  newly authored `Halt` guard is spliced immediately before the copied span in
  the same event, so fall-through halts exactly where running past the old
  program's end used to halt. A later jump mutation — an offset stepped by one
  unit, or an inserted or replaced jump — is the only way the span becomes
  reachable, short of a mutation removing or replacing the guard or the
  program's final `Halt`, and it then runs in its original's place. Neutrality here is
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
3. persistent state advances once per visit (T19.F02): a graph visit reads
   the last committed operator state and outputs and commits its own; visits
   per tick are bounded by the per-pass hop cap and priced by the per-tick hop
   ramp; unvisited modules hold; initialized eligibility decays once per world
   tick, even on skipped visits, and every successful visit adds its activity
   from actual evaluation inputs. Reward gain applies eta once to
   activity-only credit; failed visits never overwrite successful activity;
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
split exclusion (retired by T25.F02), and the mesh post-activation qualification. A copy is
required to be neutral when it fires *and* to reproduce its original when a
later mutation runs it in the original's place. Learned-weight correspondence
through such a copy follows T11.F09: birth-local available values move with each
edge occurrence on either surface (T25.F06) through graph/mesh splices and
faithful copies, including edge bundles. Rollback restores them with the genome. New/replaced backends or
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
when both executions have enough energy. Growth
operators: `AddComputeNode` (all three forms below), `CopyComputeNode`,
`CopySubgraph`, `InputRef.Add` on a VM node, the VM `VmCopyInstructionBlock`,
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
`MutateGraphOperatorParam`, `GraphRawFieldMutation`, `InputRef.Add` on a
Graph node (T20.F16), `InputRef.Swap`, and `InputRef.RawFieldMutation` are all
in this class. Topology connection operators
are `RetargetNodeTarget`, `RemoveRouteTarget`, `RemoveNode`, `MutateGateBias`,
`SwapRouteTargets`, and the explicit macro `ChangeEntryNode`, all owned by
T11.F15, and `AddFeedbackRoute`, owned by T11.F29. The retired identity rename is outside this live taxonomy. `CopyEdgeBundle` remains an
explicit multi-edge macro, as the VM's paired-slot-address operator is. A
grown node still costs `graph_node_base_cost` per visit; growth neutrality
does not extend to energy exhaustion.

### Graph domain

T20.F02 qualifies ordinary zero-weight edges to existing `ActionVote` sinks
for F04 recruitment. The [graph consumer contract](v3-graph-backend-spec.md#qualified-zero-vote-connections-t20f02)
owns eligible destinations, finite/context preconditions, independently mutable
occurrences, prune/reindex and copy/serde behavior, address/admission limits,
and native storage/work costs. Existing weight mutation and edge deletion apply
to zero edges. T20.F04 adds the opt-in `RecruitNeutralInput` operator below.

Topology mutations operate on `compute_nodes` only. Fixed structural outputs
(the 98 output sinks, vote and parameter sinks included) are never
added/removed/retyped — only their edges are evolvable.

- `AlterGraphEdgeWeight` (both edge-bearing surfaces: compute inputs and sink
  inputs)
- `SwapGraphOperator` (compute nodes only, 17 `ComputeNodeKind` variants)
- `MutateGraphOperatorParam` (compute params: Constant, Threshold,
  DecayIntegrator, Momentum, Oscillator)
- `AddComputeNode(kind)` — a growth operator; draws one of three
  function-preserving forms with equal probability: (a) disconnected, a
  random kind appended with no inputs; (b) bootstrap, a random kind appended
  with one input edge from `random_graph_source`, read by no surface; (c)
  split, a NEAT-style insertion into one existing edge (either surface,
  source any `GraphSource`) with an identity `Add` node whose
  single input (weight 1.0) reproduces the split edge's prior source exactly
  in f32 (the new node's output passes through `sanitize_output` like every
  compute node's, so exact reproduction holds for values already within its
  NaN→0/±1e9-clamp range), and the old edge retargeted to the new node at its
  old weight. The split draws from the splittable edges only. When the split
  edge's consumer is a compute node at index `c`,
  the new node is inserted at index `c` and every `ComputeNode(i >= c)`
  reference is remapped to `i + 1` across both surfaces
  (`CgpGraphBackendDef::insert_compute_node_at`, the insert-with-remap
  inverse of `remove_compute_node_at`), preserving Gauss-Seidel pass order; a
  sink consumer appends instead. A split of a backward or
  self edge delays that edge's value by one visit. An edge whose source
  is an out-of-range `ComputeNode` (a prior removal's sentinel) is not
  splittable and is excluded from the draw; when the graph has no splittable
  edge the split form is not offered and `AddComputeNode` draws one of the
  other two forms, both of which apply to any graph.
  Every other edge splits, including a sink's direct
  `DynamicIntrospection(EnergyCurrent)` read on a graph carrying plasticity:
  learning is free (T25.F02), so the evaluation and post-evaluation effects
  `ResolveCtx` carry the same energy and an identity node between them
  commits the value the direct edge would. The T11.F08 exclusion of that
  shape went with the plasticity-cost deduction it guarded, and the split's
  neutrality property holds unconditionally.
- `RemoveComputeNode` (removes from `compute_nodes`, remaps
  `GraphSource::ComputeNode` indices across all edge containers)
- Edge operators and edge groups (T25.F03): every edge draw indexes
  `edge_sites`, which enumerates group members as ordinary sink edges. A
  member may be re-weighted (its correction), removed (the last member's
  removal removes the group) or retargeted on its own; an edit that changes a
  member's source outside a group operator (`RetargetGraphEdge` with a changed
  source, every `GraphRawFieldMutation` step on a member, `AddComputeNode`'s
  split) detaches it first: its own weight becomes its genomic weight
  `scale + own_weight`, bitwise, and it moves to the end of its sink's
  independent edges, where the edit continues. A same-source retarget draw
  changes nothing. `DetachGraphEdge` (below) detaches one member and
  nothing else. `AddGraphEdge` and `InputRef.Add` insert before a sink's
  first member and never join a group: the pushed edge carries no
  membership and every group keeps its record and member set (T25.F08).
  Each committed detach and group removal is counted
  (`graph_edges_detached_total`, `groups_removed_total`). Under the T25.F10
  unit rule a member costs nothing, so a detach adds one unit beside the
  operator's own delta, less the record's when its last member goes (a sole
  member's detach adds nothing); removing a member is 0, or −1 with the
  record. Supply stays `Binomial(genome_size(), per_unit_rate)` while every
  edge-targeting operator still picks among all materialized edges, members
  included: a genome of intact `n`-member genes draws about `n + 1` times
  fewer events for the same targets (the intended supply-and-target law).
- `AddGraphEdge` (both edge-bearing surfaces, `pick_random_surface` one
  uniform draw over one surface per compute node and one per sink in vector
  order: all 98 sinks, the 30 vote sinks and the four parameter sinks
  included (T11.F27, T11.F30, T02.F08);
  source sampled by
  `random_graph_source`, which draws a compound `InputLeaf` source's
  `sub_idx` uniformly across the reference's full width via
  `mutation::compound::sub_value_count`, so new edges can reach every
  sub-value, not just index 0)
- `RetargetGraphEdge` (both edge-bearing surfaces; same `sub_idx` sampling
  as `AddGraphEdge`)
- `RemoveGraphEdge` (both edge-bearing surfaces)
- `GraphRawFieldMutation` — selects one parameterized compute node or one
  edge uniformly, then changes exactly one field by one unit and never
  replaces the `GraphSource` variant: a parameter by the existing
  `MutateGraphOperatorParam` step; `ComputeNode(idx)` by ±1 inward at the
  bounds `0..compute_count`; `InputLeaf.ref_idx` by ±1 inward within
  `0..input_refs.len()`, offered only when the current `sub_idx` stays within
  the candidate reference's width; `InputLeaf.sub_idx` by ±1 inward within the
  reference's width; `SharedMemory.slot` by ±1 modulo 16; `SharedMemory.previous`
  flipped. Edges with no valid unit move are excluded from
  the draw, so the
  operator selects only a module that has a parameterized compute node or a
  movable edge, and never skips after selecting one.
- `CopyComputeNode` — a growth operator; inserts a faithful copy (kind and
  inputs, each edge with its learning attributes) of one random compute node
  directly after its source
  and nothing else. The copy reads whatever its source read and is read by
  nothing; no backlink is added, and inputs are never coin-flip cleared (a
  copy that should start disconnected is `AddComputeNode`'s disconnected
  form). Placing the copy at `source + 1` rather than at the end is what
  keeps every copied edge on its original's evaluation phase (T11.F08): a
  lower-index source is read from the current visit and a self or
  higher-index source from the last committed outputs (T19.F02), so a source
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
- The nine plasticity operators act on edges (T25.F06): each picks one edge
  uniformly among the edges matching its predicate, in `edge_sites` order over
  both surfaces (count-then-select, one draw), writes through
  `EdgeMut::set_plasticity`, and applies to a module when any edge matches. A
  `ClearSlot` edge never matches; a group member does (T25.F07). After the
  pick a member draws the **scope coin** `gen_bool(0.5)`: false, the change
  lands on the picked member; true, on every member of its group the
  operator's predicate admits, the rest untouched with no draw; a non-member
  draws no coin. Value draws follow the coin and are drawn once, and each
  target applies the law to its own config: `EnableHebbian` writes one
  config to every admitted (static) member; `EnableRewardModulation` sets one
  modulation, each member keeping its rule, rate, clamp and flag;
  `DisableRewardModulation` clears the modulation alone; `MutateHebbianRule`
  and `MutateRewardSource` draw among the others of the picked edge's value
  and write the drawn value; `ToggleHebbianLamarckian` writes the picked
  edge's flipped flag; `MutateHebbianRate` and `MutateTraceDecay` draw one
  `u = gen_range(-1.0..=1.0)` and each target steps its own value by its
  branch (`× (1 + 0.3u)` above 0.01, else `+ 0.05u`, then `[0, 1]`). Weights
  and catalog positions are unchanged; a learning attribute carries no
  genome unit.
  Plasticity travels with its edge: retarget, split (the consumer keeps it,
  the identity edge is static), `CopyComputeNode`, `CopySubgraph` and
  `CopyEdgeBundle` copy it; `AddGraphEdge`, recruitment and group creation
  push static edges; `RemoveGraphEdge` removes it with the edge.
- `EnableHebbian` (a static, eligible edge: rule, rate `0.01..0.2`, clamp
  `1.0..5.0`, Lamarckian coin)
- `DisableHebbian` (a plastic edge: remove its `PlasticityConfig`)
- `MutateHebbianRule` (a plastic edge: one of the other three rules)
- `MutateHebbianRate` (a plastic edge: perturb `learning_rate`, kept in `[0, 1]`)
- `ToggleHebbianLamarckian` (a plastic edge: flip the inheritance flag)
- `EnableRewardModulation` (a pure Hebbian edge: a channel and decay `0.5..0.99`)
- `DisableRewardModulation` (a modulated edge: remove its modulation)
- `MutateRewardSource` (a modulated edge: one of the other three channels)
- `MutateTraceDecay` (a modulated edge: perturb `trace_decay`, kept in `[0, 1]`)
- `SetScope` (T25.F07, key `Graph.SetScope`, weight 2, Neutral, after
  `RemoveGroup` in the catalog; applies to a Graph node with a member): one
  member uniformly in canonical order, the scope coin, then the other of the
  picked member's scope written to it alone or to every member of its group;
  no value draw. Scope carries no genome unit.
  Each committed change of the nine plasticity operators, `SetScope` and
  the three edge retention operators counts once at the scope it landed on,
  `Group` for a member with a true coin, else `Edge`
  (`attribute_changes_by_scope`, T25.F08, T25.F09).
- **Retention** (T25.F09): an edge's `retention` and a group's (its learned
  scale's) set how far a learned quantity relaxes toward its birth value
  each tick (graph backend spec Section 13); `EnableHebbian` and both
  creation forms write 1 with no draw. Six operators after
  `DetachGraphEdge`, keys `Graph.<name>`, all Neutral, `Δgenome_size` 0,
  none admitted by restricted size pressure:

  | Operator | Weight | Applies to | Draws, in order |
  | --- | ---: | --- | --- |
  | `EnableRetention` | 1 | a Graph node with a plastic edge at retention 1 | one such edge uniformly in canonical order; the scope coin for a member (a group-wide change reaches every admitted member); one `u = gen_range(-4.0..=0.0)`; retention `1 − 10ᵘ` on every target |
  | `DisableRetention` | 1 | a Graph node with a plastic edge below retention 1 | the edge and coin; no value draw; retention 1 |
  | `MutateRetention` | 4 | as `DisableRetention` | the edge and coin; one `u = gen_range(-1.0..=1.0)`; each target's deficit `d = 1 − retention` becomes `(d · (1 + 0.3u)).clamp(10⁻⁴, 1)` |
  | `EnableGroupRetention` | 1 | a Graph node with a group record at retention 1 | one such record uniformly in `GroupId` order (a record without a scale slot admitted, its retention inert); `EnableRetention`'s draw |
  | `DisableGroupRetention` | 1 | a Graph node with a group record below retention 1 | the record; retention 1 |
  | `MutateGroupRetention` | 4 | as `DisableGroupRetention` | the record; `MutateRetention`'s `u` and step |

  A scope-`Group` member's retention is inert but editable, as its
  `lamarckian` flag; the group three report no scope.
- `RecruitNeutralInput` (T20.F04, structural weight 1) — enabled by
  `mutation.neutral_input_recruitment = "SingleChannel"` or `"WholeFamily"`;
  `"WholeFamily"` is the default since 2026-10-12.
  Select one Graph node through the existing applicable-node and target-bias
  rules, then one admissible source family and one present valid `ActionVote`
  sink occurrence, empty or wired. SingleChannel appends one ordinary edge of
  weight `0.0`; WholeFamily appends each canonical finite channel once to that
  one sink. Only missing input declarations are appended. Existing coefficients,
  duplicate occurrences, references, compute nodes, routes and backend kinds
  are preserved.

  Sources cover the existing input-reference universe, every configured food
  identity, all 24 upstream slots and both 16-slot shared-memory banks. Fixed
  compounds use their actual widths and scalars use channel 0. SingleChannel
  samples ActionQueue across all `u16` channels; WholeFamily excludes it because
  it has no small complete fixed-width family. WholeFamily groups upstream into
  24 channels/declarations and each memory bank into 16 channels with no
  declaration. The event appends at most 30 edges and 24 declarations, to one
  sink, and counts as one mutation event. Typed families name one food type.

  Source admission reuses the first representable matching declaration, or
  preflights every append before sampling. Appends cannot bind any incumbent
  dangling InputLeaf, and new indices must fit `u16`; there is no truncation,
  overwrite or partial bundle. A full declaration table still admits existing
  sources and shared memory, so a node with a valid vote sink always has a
  feasible source. VM-only nodes and absent or invalid vote sinks are excluded.
  Direct unavailable constructions leave the genome unchanged.

  Off (also the omitted default) removes recruitment before sampling and keeps
  existing operator order, weights and RNG draws. Size-pressure restriction
  excludes recruitment as growth. The existing operator funnel and selected
  node record use `Graph.RecruitNeutralInput`; observation draws no RNG. The
  neighborhood schema includes a zero-valued row when Off. Ordinary
  `InputRef.Add`, `AddGraphEdge`, retargeting and VM paths remain separate, and
  the old ActionQueue source draw remains 12 channels. Independent weight
  mutation, pruning and copy/serialization operate on the ordinary new edges.

  Exact finite-source vote neutrality requires the F02 execution/input context
  and adequate budget. New declarations may activate sensor assembly; added
  units increase maintenance, replication and later size-derived mutation
  supply. An already active graph has no separate physiological per-edge price;
  a dormant graph's first edge adds its minimum charged visit. Silent signaling
  does not imply physiological neutrality or discovered usefulness.

  Ring case (T25.F03): a ring family (`NeighborFoodRing`, `NeighborBarrierRing`,
  `NeighborOccupiedRing`) landing on a directed sink `K(d)` is declared as
  above, then creates one silent `Ring` edge group on that reference into `K`
  at offset `(d − channel) mod 8` (`channel` 0 under WholeFamily): eight
  members at own weight 0 and scale 0, in place of the edge or column, with no
  new draw. Other families and destinations are unchanged: a ring family
  landing on a kind drive (T11.F30) appends its zero-weight edges as on `Eat`,
  never a group. The family universe drops ring families when the graph
  cannot host a group (`can_host_group`: all 24 direction sinks and
  fewer than `u16::MAX` groups), so a drawn ring plan always completes. A
  dangling member reference bounds admission as any dangling leaf does.

  Pair case (T25.F04): under WholeFamily, a pair-declaring family (sensor
  spec Section 5.7) landing on a directed sink `K(d)` draws one pair
  `gen_range(0..direction_pairs().len())` after the destination (the only new
  draw) and creates one silent `DirectionPair` gene at offset `d` through
  one container primitive: twelve direction members plus one born-0 fact
  member into `Drive K` per fact of the pair, in facts order (T25.F10): 15
  members for an area family, 20 for a creature slot (eight facts since
  T27.F01), one `genome_size()` unit beside its declarations.
  The reference is declared once and shared. SingleChannel keeps its
  one-edge plan and draws, an area family on an undirected destination its
  whole-family edges. `NearbyCreature`, whose pairs are its slots
  (`slot_pairs()`, T25.F05), takes the pair draw on every destination: on a
  directed sink the pair case above wires a whole neighbor, its offset to the
  eight direction votes and its eight other fields to the drive; on any other
  sink the drawn slot's ten channels `10k..10k+10`, ascending and born 0,
  never the family's 40. The WholeFamily universe
  drops pair-declaring families when the graph cannot host a group or any
  kind's `Drive` sink is absent, so a drawn pair plan always completes; ring
  admission is unchanged.

- Edge-group operators (T25.F03), at the catalog positions and weights of the
  T11.F31 ring projection operators they replace, after
  `RecruitNeutralInput`, with `AddDirectionPairGroup` (T25.F04) directly
  after `AddRingGroup`, `SetScope` (T25.F07) after `RemoveGroup`, and
  `AddRandomizedRingGroup`, `AddRandomizedDirectionPairGroup` and
  `DetachGraphEdge` (T25.F08) after `SetScope`, and the six retention
  operators (T25.F09) after `DetachGraphEdge`. The Graph catalog holds 38
  operators; the default-arm total weight (every operator but
  `RecruitNeutralInput`) is 78, and the enabled total at the default config
  79. A group is a record
  `{shape: Ring {ring_ref} | DirectionPair {ref_idx, pair}, kind, offset,
  scale, inherit_scale, retention}` plus its member sink edges, each with a `scope`
  (graph backend spec, "Ring edge groups" and "Direction-pair edge groups");
  the unit operators pick one group uniformly over both shapes, edit the
  record and move members with it, scope included.

  **Learned creation** (T25.F07): after its shape draws a creation operator
  draws one mode `gen_range(0..3u8)` (0 static, 1 Hebbian, 2
  reward-modulated); a learning mode draws one config by `EnableHebbian`'s
  law (then `EnableRewardModulation`'s for mode 2) written to every member;
  then one scope coin (`gen_bool(0.5)`, true `Group`) written to every
  member; then one `inherit_scale` coin (`gen_bool(0.5)`). The pattern and
  the scale are never randomized: a new group is silent.

  **Randomized creation** (T25.F08): `AddRandomizedRingGroup` and
  `AddRandomizedDirectionPairGroup` apply where their uniform twins do, make
  the twin's shape draws in the same order and create the silent group
  through the same constructor; then, per member in canonical order, its
  own mode `gen_range(0..3u8)`, that mode's config by the same laws and its
  own scope coin; then one `inherit_scale` coin for the group. A mutation in
  the uniform form changes every member it builds; the randomized form
  gives each member its own learning attributes.

  **Detach** (T25.F08): `DetachGraphEdge` applies to a Graph node with a
  member. It draws one member uniformly in canonical order (one draw, no
  coin, no value draw) and detaches it as a retarget does: its own weight
  becomes `compose(own, scale, pattern)` bitwise, its membership and scope
  clear, its plasticity stays, it moves to the end of its sink's
  independent prefix, and its seed composes (T25.F07 invariant 6). A fact
  member (T25.F10) detaches own-only: its own weight bits and its seed entry
  stay exactly as stored. A last
  member's detach removes the record and renumbers higher `GroupId`s. A
  committed detach counts one `graph_edges_detached`, and one
  `groups_removed` when the record went.

  | Operator (key `Graph.<name>`) | Weight | Applies to | Draws, in order | Complexity |
  | --- | ---: | --- | --- | --- |
  | `AddRingGroup` | 2 | a Graph node with a ring at an index ≤ 65,535 that can host a group | the ring uniformly over those indices ascending; kind `gen_range(0..3)` over `[Move, Reproduce, StealEnergy]`; offset `gen_range(0..8)`; creates a group at scale 0 with eight members at own weight 0 (duplicates allowed); then learned creation's draws; `Δgenome_size` +1 | Increasing |
  | `AddDirectionPairGroup` | 2 | a Graph node with a pair-declaring reference at an index ≤ 65,535 that can host a group | one pair uniformly over the flat list (`ref_idx` ascending, `pair` ascending) of declared pairs; kind `gen_range(0..3)`; offset `gen_range(0..8)`; creates a group at scale 0 with twelve members at own weight 0 and no fact members (duplicates allowed); then learned creation's draws; `Δgenome_size` +1 | Increasing |
  | `AlterGroupScale` | 4 | a Graph node with a group | the group uniformly; `AlterGraphEdgeWeight`'s law on the scale (`\|w\| > 0.01` scales by `1 + U[−0.2, 0.2]`, else adds `U[−0.1, 0.1]`) | Neutral |
  | `ShiftGroupOffset` | 2 | same | the group; `gen_bool(0.5)` true +1, false −1, mod 8; every member moves to its new sink | Neutral |
  | `RetargetGroupKind` | 2 | same | the group; `gen_range(0..2)` over the other two kinds in catalog order; every member moves to its new sink, a fact member to the new kind's drive; a gene with a fact member skips after its draws when that drive is absent | Neutral |
  | `RemoveGroup` | 2 | same | the group; removes the record and every member of either role, renumbering higher `GroupId`s; `Δgenome_size` −1 | Decreasing |
  | `AddRandomizedRingGroup` | 2 | as `AddRingGroup` | `AddRingGroup`'s shape draws; creates the same silent group; then randomized creation's per-member draws and one `inherit_scale` coin; `Δgenome_size` +1 | Increasing |
  | `AddRandomizedDirectionPairGroup` | 2 | as `AddDirectionPairGroup` | `AddDirectionPairGroup`'s shape draws; creates the same silent group; then randomized creation's per-member draws and one `inherit_scale` coin; `Δgenome_size` +1 | Increasing |
  | `DetachGraphEdge` | 2 | a Graph node with a member | one member uniformly in canonical order; detaches it (a fact member own-only); `Δgenome_size` +1, or 0 when the record goes | Neutral |

  Restricted size pressure admits only `RemoveGroup` of these nine. A shift or
  retarget re-inserts each member at its new sink in the I10 position, so it
  and its inverse return an equal genome. A born-connected `InputRef.Add`
  ring landing on a direction sink stays one ordinary edge (T20.F16
  unchanged, draw for draw).

### InputRef domain

- `Add` — pushes one reference to `input_refs`. On a VM node it is a growth
  operator and wires nothing (no `ReadInput` auto-insertion, no further draw);
  the VM `ReadInput { ref_idx }` reference becomes addressable only by the VM
  operators that construct `ReadInput`. On a Graph node it is a connection
  operator (T20.F16, born-connected input declaration): after the reference
  draw it draws `sub_idx = gen_range(0..width)` (`width` the reference's
  sub-value count), a vote sink `VoteSink::from_index(gen_range(0..30))` and a
  weight `gen_range(-1.0f32..=1.0)` (`AddGraphEdge`'s new-edge law), in that
  order, and appends one `GraphEdge { source: InputLeaf { ref_idx, sub_idx },
  weight }` to that `ActionVote` sink's inputs, `ref_idx` the appended index.
  The draws are always taken; when the index does not fit `u16` or the node
  lacks the drawn sink, no edge is pushed and the event stays a plain
  declaration. `Add` and `AddGraphEdge` report the class of the vote sink
  they push onto (`drive` or `direction`, T11.F30); a birth whose only
  applied event is one of them stores that class, and `SimStats` keys its
  carrier outcome by it (`mutation_value_totals_by_birth_edge_class`). The born edge is an ordinary edge with no provenance; the
  connected reference is not prunable while the edge consumes it. A ring
  landing on a direction sink stays this one ordinary edge, never joining an
  edge group (T25.F03).

  The new reference is one `gen_range(0..30)` key draw into the one bucket
  table, `sampling::INPUT_BUCKETS` (T27.F01); `input_reference_universe`
  (the `Swap` partition, recruitment's enumeration) is the same table
  expanded over the food types and the 24 upstream slots, deduplicated in
  table order. Recruitment's family weights (`universe_entries`) are a
  separate declaration.

  | Index | Reference | Extra draw |
  | --- | --- | --- |
  | 0, 1 | `FoodHere`, `NeighborFoodRing` | food type, with more than one food type |
  | 2, 3 | `NeighborBarrierRing`, `NeighborOccupiedRing` | none |
  | 4 | `StaticIntrospection(AgeTicks)` | none |
  | 5 to 7 | `StaticIntrospection(OwnRed)`, `(OwnGreen)`, `(OwnBlue)` (T27.F01) | none |
  | 8, 9 | `DynamicIntrospection(EnergyCurrent)`, `(EnergyConsumedThisTick)` | none |
  | 10 | `ActionQueue` | none |
  | 11 | `AreaFoodSummary` | food type, with more than one food type |
  | 12 to 16 | `AreaBarrierSummary`, `NearbyCreature` (13, 14 and 15, T25.F05), `AreaOccupancySummary` | none |
  | 17 to 21 | `ActionVotes`, `PreviousPassVotes`, `CommitCounts`, `DynamicIntrospection(HopsThisTick)`, `PreviousOutcome` | none |
  | 22 to 29 | `UpstreamSlot` (eight buckets, the slot over all 24) | the slot |
- `Prune` (T11.F22) — deletes one entry that no consumer addresses: no
  `GraphSource::InputLeaf { ref_idx }` on any edge container (a group's
  members address its ring), and no VM `ReadInput { ref_idx }` anywhere
  in the program, live or not. Higher `ref_idx` values are decremented on both
  backends, group shape references included, so every consumer
  resolves to the same reference afterwards and none is removed or rewritten;
  the event is neutral at birth. A node with no unreferenced entry is not a
  target.
- `Swap` (T11.F22) — replaces one entry with another member of its kind,
  drawn uniformly from the others. The kind is the entry's mesh read class
  (`Food`, `Barrier`, `Occupancy`, `Neighbor`, `Introspection`, `Upstream`,
  `ActionQueue`, `Decision`) and compound width, so a food ring may become
  another type's food ring, an energy scalar an age or hop-count scalar, an
  upstream slot another slot, `ActionVotes` `PreviousPassVotes` and back,
  never a scalar for a ring or a barrier ring for a food ring. Rings,
  summaries, neighbor blocks, the action queue, `CommitCounts` (`Decision`,
  width 4) and `PreviousOutcome` (`Introspection`, width 4) are the only
  members of their kind and are never swapped; typed food is swappable only
  with more than one food type. The width never changes, so no edge falls out of range.
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
  it draws the requested event count from the one supply rule: one
  Bernoulli trial per unit at `per_unit_rate` (T11.F19, T11.F20), on the
  parent's own `genome_size()` for every production birth. If
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
  6) run ParseabilityGate, then validate_graph on every Graph backend
  7) if parseability fails:
       rollback event
       mark skipped(ParseabilityViolation)
       continue
     if validate_graph fails (T25.F03, every domain):
       rollback event
       mark skipped(GraphInvariantViolation)
       continue
  8) commit event; classify target as Reachable/Unreachable/NotApplicable
```

Selection randomization rules (internal to `MutationEngine`):
- Supply (T11.F19, T11.F20), the only mutation-count rule: read the unit
  count once, draw one Bernoulli trial at `per_unit_rate` per unit, and
  request one event per success, so the count is
  `Binomial(units, per_unit_rate)` with no trigger, minimum, maximum, or
  continuation. Every production birth and reading uses the parent's own
  `genome_size()`, so a genome pays its size in exposure. Rate 0.0 requests
  nothing (every birth is a clone); rate 1.0 requests one event per unit. The
  default 0.0060625 (`0.485 / 80`, T25.F02) keeps the 80-unit V3Alpha1
  founder at 0.485 requested events per birth. Natural analog: per-base copy error. If the count is
  zero, `MutationEngine` returns `MutationSummary { attempted_events: 0,
  applied_events: 0, skipped_events: 0, ... }` immediately.
- The two instruments, the drift walk and the recruitment-paths legacy
  panel, draw the same rule on a fixed unit count: the canonical V3Alpha1
  founder's `genome_size()` (`FOUNDER_GENOME_SIZE_UNITS`, 80), whatever the
  walked genome's size, so their exposure is an instrument constant
  (`Binomial(80, per_unit_rate)`, 0.485 requested events per birth at
  the default rate). The recruitment-paths S0 panel draws on the child's own
  size, as production does.
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
   sets are resolved once per birth in the parent's node indices, and at
   every draw of the birth a child node is a member exactly when it has been
   present since the birth began, no event of this birth created it, and its
   parent index is in the set (T11.F24). Carried nodes keep their membership
   when a mid-birth `RemoveNode` shifts their index, and a node added during
   the birth never counts, even on a removed node's `NodeId` or former index.
   This holds for every engine caller, the drift walk and other `neighborhood`
   readings included.
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

After each event, run `ParseabilityGate`, then `validate_graph` on every Graph
backend (T25.F03): the edge-group invariants I2–I10 of the graph backend
spec, checked after every applied event of any domain, since `InputRef.Prune`
reindexes shape references and topology events clone and swap backends.

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

If an event results in non-parseable genome or a Graph backend failing
`validate_graph`:
1. rollback that event (the genome, birth seeds included, is restored),
2. mark event skipped with reason `ParseabilityViolation` or
   `GraphInvariantViolation`,
3. continue with remaining events. A rolled-back event counts no detach and no
   group removal.

No behavioral repair is required.
No global topology rewrite is required.

### `MutationSkipReason` minimum enum

- `ParseabilityViolation`
- `NoApplicableTarget`
- `GraphInvariantViolation` (T25.F03)

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

### Applied graph temporal clock (T11.F06, per visit since T19.F02)

The node-type persistent-state contract uses the last committed operator
state and compute outputs as the read base of every visit. Ordered
combinational paths compute within one visit; self/higher-index edges read
the previous visit's commit, whether that visit was this tick or an earlier
one. Repeated visits within a tick advance temporal state once per visit,
bounded by the per-pass hop cap and priced by the per-tick hop ramp;
unvisited modules hold. Disconnected node growth cannot change an existing
node's clock. See `v3-graph-backend-spec.md`.
