# Mesh Substructure Copy — Analysis & Mutation Operators

**Goal:** Add mesh-level backward/forward slice analysis and corresponding topology mutation operators that duplicate multi-node substructures with ID remapping, completing the hierarchical gene duplication pattern (instruction → sub-node graph → mesh).

**Goal IDs:** GP-01 (Evolve Richer Creature Decision-Making), GP-02 (Maintain Clean Architecture Boundaries)

**Scope:**
- Included: Mesh backward/forward slice analysis functions in `analysis.rs`; two new topology mutation operators (`CopyMeshBackwardSlice`, `CopyMeshForwardSlice`) in `topology/mod.rs`; unit tests for all new functions and operators.
- Excluded: Wiring graph slice analysis into graph mutation operators; mesh pruning/compaction operators; changes to mutation engine domain weights.

**Docs Impact:**
- New: `docs/plans/2026-02-27-mesh-substructure-copy-design.md` (this file)
- Updated: none
- Retired: none

**Supersedes:** none
**Superseded-By:** none

---

## Context

VM and Graph backends both have a progression of copy operators:
1. Dumb block/node copy (no remapping)
2. Remapped copy (register shifts / edge remapping)
3. Semantic gene copy via slicing (backward/forward dependency analysis)

The mesh (topology) level only has single-node `CopyNode` — no multi-node substructure operators. This means evolution cannot duplicate multi-node processing pipelines as a unit. If a creature evolves a useful routing chain (sensor-preprocessing → decision → action), there is no mutation pathway to duplicate that chain and let one copy diverge.

Gene duplication is biology's primary engine of functional innovation. Completing this pattern at the mesh level gives evolution the tool to scale up behavioral complexity when fitness pressure rewards multi-stage processing.

The analysis functions follow the established pattern: pure functions in `creature/genome/analysis.rs` returning `DetectedGene` (sorted index sets), consumed by mutation operators but available for future uses (reproduction, visualization, pruning).

---

## Design

### Analysis Functions (`analysis.rs`)

**Backward slice** — "feeding pipeline": which nodes route execution toward the anchor?

`mesh_backward_slice(genome, anchor_idx, max_size) -> Option<DetectedGene>`
- Anchor must be valid (in bounds)
- Scan all nodes: if any node's `targets` list contains the anchor's `node_id`, include it
- Fixpoint: repeat until no new nodes or `max_size` reached
- Always includes the anchor itself
- Returns sorted indices, or `None` if anchor is invalid

`mesh_backward_slice_random(genome, rng, max_size) -> Option<DetectedGene>`
- Pick a random node index as anchor, delegate to deterministic core

**Forward slice** — "downstream subtree": what can this node route to?

`mesh_forward_slice(genome, seed_idx, max_size) -> Option<DetectedGene>`
- BFS forward from seed through `targets` edges
- Always includes the seed itself
- Cap at `max_size`, return sorted indices

`mesh_forward_slice_random(genome, rng, max_size) -> Option<DetectedGene>`
- Pick a random node index as seed, delegate to deterministic core

### Mutation Operators (`topology/mod.rs`)

**`CopyMeshBackwardSlice`**
1. Call `mesh_backward_slice_random` (max_size from config or hardcoded cap)
2. Deep-clone all nodes in the detected gene
3. Assign fresh `NodeId`s via `next_node_id` (incrementing for each cloned node)
4. Build old→new ID mapping; remap all `targets` within the cloned set
5. Append cloned nodes to `genome.nodes`
6. With 50% probability, add a backlink from a random existing node to a random cloned node (making reachable)

**`CopyMeshForwardSlice`**
- Same pattern, using `mesh_forward_slice_random`

**Operator registration:**
- `TopologyOperator` enum grows from 9 to 11 variants
- `random()` range adjusts from `0..9` to `0..11`

### ID Remapping

The critical difference from single-node `CopyNode`: cloned substructures have *internal* target references that must be remapped to the new IDs. External references (targets pointing outside the cloned set) are preserved as-is — they connect the cloned pipeline to the existing mesh.

---

## Goal Alignment

| Work Item | Goal ID | Rationale |
|-----------|---------|-----------|
| Mesh slice analysis functions | GP-01 | Enables detection of multi-node functional units for duplication |
| Mesh copy mutation operators | GP-01 | Gives evolution the tool to duplicate processing pipelines, driving behavioral innovation |
| Analysis in shared module | GP-02 | Pure analysis separated from mutation side-effects, reusable across domains |

---

## Boundary Impact

- **`creature/genome/analysis.rs`:** Adds 4 public functions + `DetectedGene` reuse. No new types needed. Uses existing `CreatureGenome` imports already present for `mesh_reachable_nodes`.
- **`mutation/topology/mod.rs`:** Adds 2 enum variants, 2 operator functions. New import: `use crate::creature::genome::analysis::{mesh_backward_slice_random, mesh_forward_slice_random};`. No changes to `TopologyMutator::apply` signature.
- **Dependency direction:** `mutation` → `creature::genome::analysis` (same direction already established by VM mutation → analysis dependency). No new cross-crate dependencies.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `creature/genome/analysis.rs` | keep | Already hosts VM, graph, and mesh analysis functions; mesh slicing is a natural addition |
| `mutation/topology/mod.rs` | keep | Topology operators own mesh-level mutations; new copy operators follow existing `CopyNode` pattern |
| `mutation/engine/mod.rs` | keep | Domain weight distribution (25% per domain) unchanged; new operators share topology's existing budget |

---

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| What `max_size` cap for mesh slicing? | Use hardcoded cap of 8 nodes (mesh structures are small, typically 2-5 nodes) | agent | resolved |
| Should cloned substructures always be reachable? | No — 50% backlink probability matches existing `CopyNode` pattern; unreachable copies become "junk DNA" available for future reconnection | agent | resolved |
| Should external target references in cloned nodes be preserved or cleared? | Preserved — they connect the cloned pipeline to the existing mesh, maintaining functional relationships | agent | resolved |

---

## Implementation Tasks

1. Write this design doc and commit.
2. Add `mesh_backward_slice` and `mesh_backward_slice_random` to `analysis.rs` with unit tests.
3. Add `mesh_forward_slice` and `mesh_forward_slice_random` to `analysis.rs` with unit tests.
4. Add `CopyMeshBackwardSlice` and `CopyMeshForwardSlice` operators to `topology/mod.rs` with unit tests.
5. Update `TopologyOperator` enum and `random()` dispatch.
6. Run full verification suite.

---

## Verification

```bash
cd v3 && cargo fmt --all -- --check
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
cd v3 && cargo test --workspace
cd v3 && cargo test -p v3-core --test viability
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode warn
scripts/check-architecture-harness.sh --mode warn
```

---

## Workflow Constraints

- **Worktree:** All implementation work MUST be done in a new git worktree via `superpowers:using-git-worktrees`.
- **Code review:** After implementation, run `superpowers:requesting-code-review` with `rust-skills` focus. Review is **recursive** — continue fixing and re-reviewing until all issues (Critical + Important) are resolved. Only Minor/Nitpick issues may be deferred.
- **Skills:** `rust-skills` must be active during all Rust code writing.

**Review cycles:** 1
