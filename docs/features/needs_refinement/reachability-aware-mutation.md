---
title: Reachability-Aware Mutation Pathways
tags: [simulation, core, genome]
size: M
depends-on: [reachability-based-complexity]
status: needs-review
---

## Problem Statement

Mutation currently selects target mesh nodes uniformly across the entire genome,
treating reachable (functional) and unreachable (junk DNA) nodes identically.
This means most mutations land on whatever structure happens to be most abundant
rather than preferentially improving the creature's active decision-making mesh.

The result is that evolution spends significant mutation budget on inert genetic
material — polishing dead code, copying junk, or rewiring unreachable nodes —
while the small live core that actually drives behavior receives proportionally
fewer targeted mutations. This slows the evolution of richer creature cognition
and makes it harder for active mesh structure to grow and reorganize.

The existing `mesh_reachable_nodes()` BFS analysis (in `creature/genome/analysis.rs`)
already identifies which nodes are reachable from the entry node, but no mutation
operator currently uses this information for target selection.

## User Stories / Acceptance Criteria

- As a simulation runner, I want mutations to preferentially target reachable
  mesh nodes so that evolution improves active behavior more efficiently.
- As a simulation runner, I want a configurable per-domain bias (topology, VM,
  graph, input_ref) so I can tune how strongly each mutation domain favors
  reachable vs unreachable targets.
- As a simulation runner, I want unreachable nodes to still receive some
  mutations so that dormant structure can drift, simplify, or become useful
  when reconnected.
- As a simulation runner, I want the reachable node set cached on
  `CreatureState` (as a sorted `Vec<usize>`) so it is computed once at birth
  (after mutation) and reused without per-event BFS overhead.
- As a simulation runner, I want mutation telemetry to categorize each applied
  event as targeting a `reachable`, `unreachable`, or `cross-boundary` node so
  I can observe whether evolution is actually investing in live structure.
- As a simulation runner, I want per-domain `reachable_bias` config knobs
  (one each for topology, VM, graph, input_ref) that control the probability
  of selecting a reachable node as the mutation target.

### Biasing mechanics

- When a domain mutator needs to select a target mesh node, it rolls against its
  `reachable_bias` probability:
  - On success: select uniformly from reachable nodes (that match the domain's
    backend filter, e.g., VM-backend nodes for VM domain).
  - On failure: select uniformly from all nodes (or unreachable-only nodes —
    design decision for planning phase).
- If no reachable node matches the domain filter (e.g., all reachable nodes are
  Graph-backend but a VM mutation was selected), fall back to uniform selection
  across all eligible nodes.
- Topology operators that don't target a specific node (e.g., `AddNode`,
  `ChangeEntryNode`) are unaffected by biasing.
- Topology operators that select structural targets (e.g., `RetargetNodeTarget`,
  `SpliceNode`, `CopyMeshBackwardSlice`) should bias their anchor/seed selection
  toward reachable nodes.

### Cached reachable set

- After `MutationEngine::apply_mutations()` completes for an offspring, compute
  `mesh_reachable_nodes()` once and store the result as a `Vec<usize>` on the
  creature's state.
- This cached set is used:
  - At reproduction time, to bias the offspring's mutation target selection.
  - By the future reachability-based complexity feature for cost scaling.
  - By observability/stats systems for mesh health reporting.
- The set is computed from the *post-mutation* genome, reflecting the offspring's
  actual reachable structure.
- The parent's cached reachable set is used to bias mutations *on the offspring*
  (since the offspring's genome starts as a copy of the parent's, the parent's
  reachable set is valid for biasing before mutation modifies topology).

### Telemetry

- Extend `MutationSummary` with:
  - `reachable_target_events: u32` — events where the target node was reachable.
  - `unreachable_target_events: u32` — events where the target was unreachable.
  - `cross_boundary_events: u32` — events that span reachable/unreachable
    boundaries (e.g., topology operators that reconnect dormant structure).
  - `no_reachability_context_events: u32` — events where reachability info was
    unavailable or not applicable (e.g., `AddNode` creates a new disconnected
    node).

## Out of Scope

- **New mutation operators** (SplitLiveEdge, DuplicateLiveSliceWithReconnect,
  ActivateDormantSlice, PruneLiveBranch) — these are a planned fast-follow
  feature that builds on the biasing infrastructure from this feature.
- **Intra-node reachability analysis** — analyzing dead code within VM programs
  or unused graph internal nodes within a reachable mesh node. This is part of
  the reachability-based complexity feature, not this feature.
- **Two-lane mutation system** — a strict functional-lane vs junk-lane split
  with separate operator pools. The idea doc cautions against this as too rigid
  for a first implementation. This feature uses soft probabilistic biasing
  instead.
- **Reachability-based complexity cost scaling** — depends on the
  reachability-based-complexity feature. This feature only caches the reachable
  set; it does not change how complexity costs are computed.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should "failure" branch of bias roll select from all nodes or unreachable-only? | All nodes preserves current uniform behavior as baseline; unreachable-only creates a strict partition. Need to decide during planning. | - | open |
| What should the default `reachable_bias` values be per domain? | Likely 0.6-0.8 for topology/VM/graph, maybe lower for input_ref. Needs experimentation. | - | open |
| Should the parent's reachable set or the offspring's pre-mutation genome be used for biasing? | Parent's cached set is available and valid (offspring starts as parent copy). Using it avoids an extra BFS before mutation. | - | open |
| How should `ChangeEntryNode` interact with reachability? | It fundamentally changes what is reachable. Not clear if biasing applies. Probably exempt. | - | open |
| Should reachable_bias of 0.0 exactly reproduce current uniform behavior? | Yes — 0.0 bias means always select from all nodes uniformly. This is the backwards-compatible default during rollout. | - | open |
