---
title: Reachability-Aware Mutation Pathways
tags: [simulation, core, genome]
size: M
depends-on: [reachability-based-complexity]
status: ready
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

The completed `reachability-based-complexity` feature established the
infrastructure this feature builds on:

- `mesh_reachable_nodes()` in `creature/genome/analysis.rs` — BFS from entry
  node returning reachable node indices
- `functional_complexity()` — full reachability-aware complexity metric
  (mesh-level + intra-node dead code analysis)
- `cached_complexity: u32` on `CreatureState` — functional complexity cached at
  birth since genomes are immutable post-creation
- `genome_size()` vs `complexity()` split — total vs functional metrics

However, no mutation operator currently uses reachability information for target
selection. The mutation engine selects domains (topology, VM, graph, input_ref)
and then operators pick targets uniformly within the genome. This feature adds
a probabilistic bias layer so that mutation operators preferentially target
reachable structure while preserving some drift on unreachable material.

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
  event as targeting a `reachable`, `unreachable`, or `cross_boundary` node so
  I can observe whether evolution is actually investing in live structure.
- As a simulation runner, I want per-domain `reachable_bias` config knobs
  (one each for topology, VM, graph, input_ref) that control the probability
  of selecting a reachable node as the mutation target.

### Biasing mechanics

- When a domain mutator needs to select a target mesh node, it rolls against its
  `reachable_bias` probability:
  - On success: select uniformly from reachable nodes (that match the domain's
    backend filter, e.g., VM-backend nodes for VM domain).
  - On failure: select uniformly from all nodes (standard uniform behavior).
- This means `reachable_bias = 0.0` exactly reproduces current uniform selection
  behavior — backwards-compatible default during rollout.
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
  `mesh_reachable_nodes()` once and store the result as a `Vec<usize>` on
  `CreatureState` (new field: `pub cached_reachable_nodes: Vec<usize>`).
- This cached set is used:
  - At reproduction time, to bias the offspring's mutation target selection
    (using the parent's cached set, since the offspring starts as a copy of the
    parent's genome before mutation).
  - By observability/stats systems for mesh health reporting.
- The set is computed from the *post-mutation* genome, reflecting the offspring's
  actual reachable structure.
- The parent's cached reachable set is used to bias mutations *on the offspring*
  (since the offspring's genome starts as a copy of the parent's, the parent's
  reachable set is valid for biasing before mutation modifies topology).
- For founder creatures (seeding), compute `mesh_reachable_nodes()` at creation
  time and store directly.
- Follow the same caching pattern as `cached_complexity`:
  - Compute inside `CreatureState::new()` from the genome parameter.
  - Add a `CreatureState::new_with_cached_fields()` variant (or extend the
    existing no-mutation fast path) that accepts pre-computed values when the
    parent's genome was not mutated.

### Telemetry

- Extend `MutationSummary` with:
  - `reachable_target_events: u32` — events where the target node was reachable.
  - `unreachable_target_events: u32` — events where the target was unreachable.
  - `cross_boundary_events: u32` — events that span reachable/unreachable
    boundaries (e.g., topology operators that reconnect dormant structure).
  - `no_reachability_context_events: u32` — events where reachability info was
    unavailable or not applicable (e.g., `AddNode` creates a new disconnected
    node).

### Coordination with unified-shared-memory

The unified-shared-memory feature (in progress) replaces `memory: [u8; 1024]`
with `shared_memory: [f32; 16]` on `CreatureState`, adds new VM slot opcodes
and graph node kinds, and adds 4 VM mutation motif operators. The interaction
with this feature is minimal:

- **`CreatureState` fields**: Both features add independent fields to
  `CreatureState`. Merge-order awareness needed but no functional conflict.
- **New mutation motifs**: The shared-memory motif operators are VM-internal
  (operate within a single node's instruction list). Mesh-level reachability
  biasing applies normally — the mesh node is selected with reachability bias,
  then the motif operates inside that node. No special handling required.
- **Analysis functions**: Shared memory adds new opcodes to
  `vm_is_output_instruction` and `graph_is_output_node`. This feature consumes
  `mesh_reachable_nodes()` which operates at the mesh level, not opcode level.
  No conflict.
- **Merge order**: Whichever feature lands second handles routine merge
  conflicts (exhaustive match arms, `CreatureState::new()` signature). The
  shared-memory plan already notes: "Motifs and reachability: Independent of
  reachability-aware mutation feature."

## Out of Scope

- **New mutation operators** (SplitLiveEdge, DuplicateLiveSliceWithReconnect,
  ActivateDormantSlice, PruneLiveBranch) — these are a planned fast-follow
  feature that builds on the biasing infrastructure from this feature.
- **Intra-node reachability analysis** — analyzing dead code within VM programs
  or unused graph internal nodes within a reachable mesh node. This is already
  handled by `functional_complexity()` in the completed reachability-based-
  complexity feature.
- **Two-lane mutation system** — a strict functional-lane vs junk-lane split
  with separate operator pools. The idea doc cautions against this as too rigid
  for a first implementation. This feature uses soft probabilistic biasing
  instead.
- **Reachability-based complexity cost scaling** — already completed in the
  reachability-based-complexity feature.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should "failure" branch of bias roll select from all nodes or unreachable-only? | All nodes — preserves current uniform behavior as baseline at bias=0.0, simpler implementation, avoids edge cases when unreachable set is small. | user | resolved |
| What should the default `reachable_bias` values be per domain? | Needs experimentation. Likely 0.6-0.8 for topology/VM/graph, possibly lower for input_ref. Start conservative and tune. Decide during planning phase. | - | open |
| Should the parent's reachable set or the offspring's pre-mutation genome be used for biasing? | Parent's cached set — available without extra BFS, valid since offspring starts as parent copy before mutation. | user | resolved |
| How should `ChangeEntryNode` interact with reachability? | Exempt from biasing — it fundamentally redefines what is reachable, so biasing toward "currently reachable" nodes is not meaningful. | user | resolved |
| Should reachable_bias of 0.0 exactly reproduce current uniform behavior? | Yes — 0.0 bias means always select from all nodes uniformly. This is the backwards-compatible default during rollout. | - | resolved |
