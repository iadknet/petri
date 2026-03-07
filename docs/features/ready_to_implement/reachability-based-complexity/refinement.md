---
title: "Reachability-Based Complexity: Separate Genome Size from Functional Complexity"
tags: [simulation, core, genome]
size: M
depends-on: []
status: needs-review
---

## Problem Statement

The current `complexity()` metric counts every discrete genome component
(nodes, edges, VM instructions, graph internal nodes, etc.) with equal weight,
regardless of whether those components are reachable from the entry node. This
means creatures are penalized via complexity-scaled energy costs for carrying
inert genetic material (junk DNA) that has zero effect on their behavior.

This creates a perverse evolutionary pressure: creatures that accumulate
exploratory genetic material through mutation are punished with higher action
costs even though their functional behavior hasn't changed. The result is that
evolution is biased toward small, lean genomes rather than genomes with a rich
reserve of dormant structure that could become useful through reconnection.

The fix is to split the current single metric into two:

1. **`genome_size()`** — matches the current `complexity()` algorithm. Counts all
   components across the entire genome. Used to cap total genetic material a
   creature can carry (mutation pressure gate).

2. **`complexity()`** (refactored) — counts only components that are functionally
   reachable. Used for complexity-scaled costs (energy upkeep, action costs,
   predation kill bonus). This metric operates at three levels:
   - **Mesh level**: only nodes reachable from `entry_node_id` via BFS
     (using existing `mesh_reachable_nodes()`)
   - **VM level**: within reachable VM nodes, only instructions that contribute
     to output (backward slices from output instructions)
   - **Graph level**: within reachable Graph nodes, only internal nodes that
     contribute to output (backward slices from output node kinds)

The existing reachability analysis utilities in `creature/genome/analysis.rs`
(`mesh_reachable_nodes`, `vm_backward_slice`, `graph_backward_slice`) provide
all the building blocks needed.

## User Stories / Acceptance Criteria

- As a simulation runner, I want creatures to not be penalized for carrying junk
  DNA so that evolution can accumulate exploratory genetic material without
  paying energy costs for inert structure.

- As a simulation runner, I want complexity-scaled action costs to reflect only
  the creature's functional mesh so that two creatures with the same active
  circuitry but different amounts of junk DNA pay the same energy costs.

- As a simulation runner, I want a `genome_size()` metric that preserves the
  current total-component counting so that the mutation pressure cap still
  bounds total genetic material.

- As a simulation runner, I want the predation kill bonus to use functional
  complexity so that killing a creature with lots of junk DNA is not artificially
  rewarded.

- As a simulation runner, I want to see functional complexity in the creature
  inspector and evolution stats so that the UI reflects actual behavioral
  complexity.

### Metric definitions

**`genome_size()`** — identical to current `complexity()`:
- Per node: +1 (node itself), +len(input_refs), +len(targets)
- Per VM backend: +len(program), +len(constants)
- Per Graph backend: +len(internal_nodes), +sum(len(internal_node.inputs))
- Counts ALL nodes in the genome, reachable or not

**`complexity()`** — reachability-aware:
- Only counts mesh nodes reachable from `entry_node_id` (BFS via targets)
- Within each reachable VM node: only counts instructions reachable from output
  instructions via backward register-dependency tracing
- Within each reachable Graph node: only counts internal nodes reachable from
  output node kinds (CustomOutput, RouterOutput, WriteActionMeta, PushAction,
  PopAction, ExecuteActionQueue) via backward `source_idx` tracing
- Input refs: only counts input_refs on reachable nodes that are actually read
  by live instructions/internal nodes
- Targets: only counts targets on reachable nodes

### Call site migration

| Call site | Current metric | New metric | Rationale |
|-----------|---------------|------------|-----------|
| `ComplexityEnergyCostConfig::multiplier()` | complexity | complexity (functional) | Cost scaling should reflect active behavior |
| Action cost calculations (tick.rs) | complexity | complexity (functional) | Same — energy costs for functional behavior |
| `PredationConfig::kill_complexity_bonus_multiplier` | complexity | complexity (functional) | Kill reward reflects functional value |
| `MutationConfig::complexity_cap` pressure gate | complexity | genome_size | Cap bounds total genetic material |
| API response (`GET /creature/:id`) | complexity | complexity (functional) + genome_size | Frontend gets both |
| Frontend stats (EvolutionTab) | complexity mean/min/max | complexity (functional) | UI shows behavioral complexity |
| Frontend inspector (InspectorHeader) | complexity | complexity (functional) | UI shows behavioral complexity |

### Config renames

| Current name | New name | Reason |
|--------------|----------|--------|
| `MutationConfig::complexity_cap` | `MutationConfig::genome_size_cap` | Now gates genome_size, not complexity |
| `ComplexityEnergyCostConfig` | (keep name) | Still scales by complexity, but now functional complexity |

### Intra-node dead code analysis

**VM nodes**: For each reachable VM node:
1. Identify all output instructions: WriteInternalPayload, WriteRouteTarget,
   PushAction, PopAction, ExecuteActionQueue, SetPriorityBid, StoreMem8,
   StoreMemF32
2. Run `vm_backward_slice` from each output instruction
3. Union all backward slices — instructions in the union are "live"
4. Count only live instructions toward complexity
5. Constants referenced only by live instructions count; others don't

**Graph nodes**: For each reachable Graph node:
1. Identify all output internal nodes: those with kind CustomOutput,
   RouterOutput, WriteActionMeta, PushAction, PopAction, ExecuteActionQueue
2. Run `graph_backward_slice` from each output node
3. Union all backward slices — internal nodes in the union are "live"
4. Count only live internal nodes and their inputs toward complexity

**Input refs**: For each reachable node:
1. Determine which input_ref indices are actually consumed by live
   backend components (VM register loads from input slots, Graph internal
   nodes referencing upstream slot indices)
2. Count only consumed input_refs toward complexity

## Out of Scope

- **Caching the reachable set on CreatureState** — the reachability-aware
  mutation feature will add this cache. This feature computes functional
  complexity on-demand (or can use the cache when it exists).
- **Reachability-aware mutation biasing** — separate feature that depends on
  this one (already tracked as `reachability-aware-mutation`).
- **New mutation operators** (SplitLiveEdge, ActivateDormantSlice, etc.) —
  future work building on reachability infrastructure.
- **Genome viewer junk DNA fading** — frontend visualization idea that could
  consume reachability data but is a separate feature.
- **Active mesh observability stats panel** — separate feature that builds on
  reachability metrics.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should memory operations (StoreMem8/StoreMemF32/LoadMem8/LoadMemF32) count as "output" for VM dead code analysis? Memory persists across ticks, so writes are functionally alive even without reaching a payload slot. | Treat Store* as output instructions (functionally live). Load* are not outputs — they are inputs to downstream computation. | - | open |
| Should we cache functional complexity on CreatureState at birth to avoid recomputing on every action cost check? | Likely yes — complexity doesn't change after birth. But this overlaps with the reachable-set caching in the mutation feature. Need to coordinate. | - | open |
| Should the frontend display genome_size anywhere, or only functional complexity? | Only functional complexity in existing displays. genome_size could appear in a future genome stats card. | - | open |
| Should we rename `complexity_cap` to `genome_size_cap` in the config, or keep the old name for config compatibility? | Rename — the config is not versioned and breaking changes are acceptable. | - | open |
| Performance: is computing functional complexity per-creature acceptable, or does it need sampling/batching for population-wide stats? | For individual creature actions: compute once at birth, cache. For population stats: existing stats already sample periodically. | - | open |
