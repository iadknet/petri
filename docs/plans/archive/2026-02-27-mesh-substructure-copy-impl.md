# Mesh Substructure Copy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add mesh-level backward/forward slice analysis functions and two new topology mutation operators that duplicate multi-node substructures with ID remapping.

**Architecture:** Pure analysis functions in `creature/genome/analysis.rs` returning `DetectedGene` index sets. Two new topology mutation operators in `mutation/topology/mod.rs` that consume the analysis functions to clone connected node chains with fresh IDs and internal target remapping. Follows the same pattern established for VM and graph gene slicing.

**Tech Stack:** Rust, rand crate (Rng trait), existing `DetectedGene` type, existing `next_node_id` helper.

**See also:** `docs/plans/2026-02-27-mesh-substructure-copy-design.md` (design rationale)

**Goal IDs:** GP-01, GP-02

**Scope:**
- Included: 4 analysis functions, 2 mutation operators, unit tests for all
- Excluded: Wiring graph slice into graph mutations, pruning operators, engine weight changes

**Docs Impact:**
- New: this implementation plan
- Updated: none
- Retired: none

**Supersedes:** none
**Superseded-By:** none

---

## Goal Alignment

| Work Item | Goal ID | Rationale |
|-----------|---------|-----------|
| Mesh slice analysis functions | GP-01 | Enables detection of multi-node functional units for duplication |
| Mesh copy mutation operators | GP-01 | Gives evolution the tool to duplicate processing pipelines |
| Analysis in shared module | GP-02 | Pure analysis separated from mutation, reusable across domains |

## Boundary Impact

- `creature/genome/analysis.rs`: 4 new pub functions, reuses existing `DetectedGene` and `CreatureGenome` imports
- `mutation/topology/mod.rs`: 2 new enum variants, 2 operator functions, new import from analysis module
- Dependency direction: `mutation` → `creature::genome::analysis` (already established)

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `creature/genome/analysis.rs` | keep | Already hosts VM, graph, and mesh analysis; mesh slicing is natural addition |
| `mutation/topology/mod.rs` | keep | Topology operators own mesh mutations; new copy operators follow existing CopyNode pattern |

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| max_size cap for mesh slicing | Hardcoded 8 (mesh structures typically 2-5 nodes) | agent | resolved |
| Should cloned nodes always be reachable | No; 50% backlink probability matches CopyNode pattern | agent | resolved |

## Workflow Constraints

- **Worktree:** All work MUST be done in a new git worktree via `superpowers:using-git-worktrees`
- **Code review:** After implementation, run `superpowers:requesting-code-review` with `rust-skills` focus. Review is **recursive** — continue fixing and re-reviewing until all Critical + Important issues are resolved. Only Minor/Nitpick may be deferred.
- **Skills:** `rust-skills` must be active during all Rust code writing

---

## Task 1: Set up worktree

**Step 1: Create worktree**

Use `superpowers:using-git-worktrees` to create an isolated worktree for this work.

**Step 2: Verify clean state**

Run: `cd v3 && cargo check --workspace`
Expected: compiles cleanly

---

## Task 2: Add `mesh_backward_slice` analysis function

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/analysis.rs`

**Step 1: Write failing tests**

Add these tests to the `#[cfg(test)] mod tests` block in `analysis.rs`, after the existing mesh reachability tests. Use the existing `simple_vm_node` helper already defined in tests.

```rust
// ── Mesh backward slice tests ───────────────────────────────────────

#[test]
fn mesh_backward_slice_finds_feeding_pipeline() {
    // Node 0 -> Node 1 -> Node 2, entry at 0
    // Backward from node 2: nodes 0 and 1 target-reach node 2's predecessors
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 2, 8).unwrap();
    // Node 1 targets node 2, node 0 targets node 1 -> all three included
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_backward_slice_excludes_unconnected_nodes() {
    // Node 0 -> Node 1, Node 2 -> Node 1, Node 3 is isolated
    // Backward from node 1: nodes 0 and 2 feed into it
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![]),
            simple_vm_node(2, vec![1]),
            simple_vm_node(3, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 1, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]); // node 3 excluded
}

#[test]
fn mesh_backward_slice_anchor_only_when_nothing_targets_it() {
    // Node 0 -> Node 1, backward from node 0: nothing targets node 0
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0]); // anchor only
}

#[test]
fn mesh_backward_slice_respects_max_size() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 2, 2).unwrap();
    assert_eq!(gene.indices.len(), 2);
}

#[test]
fn mesh_backward_slice_out_of_bounds_returns_none() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![])],
    };
    assert_eq!(mesh_backward_slice(&genome, 5, 8), None);
}

#[test]
fn mesh_backward_slice_empty_genome_returns_none() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![],
    };
    assert_eq!(mesh_backward_slice(&genome, 0, 8), None);
}

#[test]
fn mesh_backward_slice_handles_cycle() {
    // Node 0 -> Node 1 -> Node 0 (cycle)
    // Backward from node 0: node 1 targets node 0
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![0]),
        ],
    };
    let gene = mesh_backward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1]);
}
```

**Step 2: Run tests to verify they fail**

Run: `cd v3 && cargo test -p v3-core -- analysis::tests::mesh_backward_slice 2>&1 | tail -5`
Expected: compilation error — `mesh_backward_slice` not found

**Step 3: Implement `mesh_backward_slice` and `mesh_backward_slice_random`**

Add to `analysis.rs` after the `mesh_reachable_nodes` function, before the `#[cfg(test)]` block:

```rust
/// Backward-slice from an anchor node at the mesh level.
///
/// Finds all nodes whose `targets` transitively reach the anchor node
/// (the "feeding pipeline"). Always includes the anchor itself. Returns
/// sorted indices capped at `max_size`.
#[must_use]
pub fn mesh_backward_slice(
    genome: &CreatureGenome,
    anchor_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    if anchor_idx >= genome.nodes.len() {
        return None;
    }

    let mut included = vec![false; genome.nodes.len()];
    included[anchor_idx] = true;
    let mut count = 1usize;

    // Fixpoint: find nodes whose targets contain any included node's node_id
    loop {
        let mut changed = false;
        for i in 0..genome.nodes.len() {
            if included[i] || count >= max_size {
                continue;
            }
            let targets_included = genome.nodes[i].targets.iter().any(|target_id| {
                genome
                    .nodes
                    .iter()
                    .enumerate()
                    .any(|(j, n)| included[j] && n.node_id == *target_id)
            });
            if targets_included {
                included[i] = true;
                count += 1;
                changed = true;
                if count >= max_size {
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }

    let indices: Vec<usize> = included
        .iter()
        .enumerate()
        .filter(|(_, &inc)| inc)
        .map(|(i, _)| i)
        .collect();
    Some(DetectedGene { indices })
}

/// Backward-slice from a randomly chosen mesh node.
#[must_use]
pub fn mesh_backward_slice_random(
    genome: &CreatureGenome,
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    if genome.nodes.is_empty() {
        return None;
    }
    let anchor_idx = rng.gen_range(0..genome.nodes.len());
    mesh_backward_slice(genome, anchor_idx, max_size)
}
```

**Step 4: Run tests to verify they pass**

Run: `cd v3 && cargo test -p v3-core -- analysis::tests::mesh_backward_slice -v`
Expected: all 7 tests pass

**Step 5: Commit**

```bash
git add v3/crates/v3-core/src/creature/genome/analysis.rs
git commit -m "feat: add mesh_backward_slice analysis function"
```

---

## Task 3: Add `mesh_forward_slice` analysis function

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome/analysis.rs`

**Step 1: Write failing tests**

Add after the mesh backward slice tests:

```rust
// ── Mesh forward slice tests ────────────────────────────────────────

#[test]
fn mesh_forward_slice_finds_downstream_subtree() {
    // Node 0 -> Node 1 -> Node 2
    // Forward from node 0: follows targets to 1, then to 2
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_forward_slice_excludes_upstream_nodes() {
    // Node 0 -> Node 1 -> Node 2
    // Forward from node 1: only nodes 1 and 2
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 1, 8).unwrap();
    assert_eq!(gene.indices, vec![1, 2]); // node 0 excluded
}

#[test]
fn mesh_forward_slice_seed_only_when_no_targets() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 1, 8).unwrap();
    assert_eq!(gene.indices, vec![1]); // seed only, no targets
}

#[test]
fn mesh_forward_slice_handles_branching() {
    // Node 0 -> [Node 1, Node 2]
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1, 2]),
            simple_vm_node(1, vec![]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1, 2]);
}

#[test]
fn mesh_forward_slice_respects_max_size() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![2]),
            simple_vm_node(2, vec![]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 2).unwrap();
    assert_eq!(gene.indices.len(), 2);
}

#[test]
fn mesh_forward_slice_out_of_bounds_returns_none() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![])],
    };
    assert_eq!(mesh_forward_slice(&genome, 5, 8), None);
}

#[test]
fn mesh_forward_slice_handles_dangling_target() {
    // Node 0 targets node 99 which doesn't exist
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![simple_vm_node(0, vec![99])],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0]); // seed only, dangling target ignored
}

#[test]
fn mesh_forward_slice_handles_cycle() {
    // Node 0 -> Node 1 -> Node 0 (cycle)
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            simple_vm_node(0, vec![1]),
            simple_vm_node(1, vec![0]),
        ],
    };
    let gene = mesh_forward_slice(&genome, 0, 8).unwrap();
    assert_eq!(gene.indices, vec![0, 1]);
}
```

**Step 2: Run tests to verify they fail**

Run: `cd v3 && cargo test -p v3-core -- analysis::tests::mesh_forward_slice 2>&1 | tail -5`
Expected: compilation error — `mesh_forward_slice` not found

**Step 3: Implement `mesh_forward_slice` and `mesh_forward_slice_random`**

Add after `mesh_backward_slice_random`:

```rust
/// Forward-slice from a seed node at the mesh level.
///
/// BFS forward from the seed through `targets` edges, collecting the
/// downstream subtree. Always includes the seed itself. Returns sorted
/// indices capped at `max_size`.
#[must_use]
pub fn mesh_forward_slice(
    genome: &CreatureGenome,
    seed_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    if seed_idx >= genome.nodes.len() {
        return None;
    }

    let mut visited = vec![false; genome.nodes.len()];
    visited[seed_idx] = true;
    let mut queue = VecDeque::new();
    queue.push_back(seed_idx);
    let mut count = 1usize;

    while let Some(idx) = queue.pop_front() {
        for target_id in &genome.nodes[idx].targets {
            if count >= max_size {
                break;
            }
            if let Some(target_idx) = genome.nodes.iter().position(|n| n.node_id == *target_id) {
                if !visited[target_idx] {
                    visited[target_idx] = true;
                    queue.push_back(target_idx);
                    count += 1;
                }
            }
        }
        if count >= max_size {
            break;
        }
    }

    let indices: Vec<usize> = visited
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect();
    Some(DetectedGene { indices })
}

/// Forward-slice from a randomly chosen mesh node.
#[must_use]
pub fn mesh_forward_slice_random(
    genome: &CreatureGenome,
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    if genome.nodes.is_empty() {
        return None;
    }
    let seed_idx = rng.gen_range(0..genome.nodes.len());
    mesh_forward_slice(genome, seed_idx, max_size)
}
```

**Step 4: Run tests to verify they pass**

Run: `cd v3 && cargo test -p v3-core -- analysis::tests::mesh_forward_slice -v`
Expected: all 8 tests pass

**Step 5: Run full analysis test suite**

Run: `cd v3 && cargo test -p v3-core -- analysis::tests -v`
Expected: all tests pass (existing + new)

**Step 6: Commit**

```bash
git add v3/crates/v3-core/src/creature/genome/analysis.rs
git commit -m "feat: add mesh_forward_slice analysis function"
```

---

## Task 4: Add `CopyMeshBackwardSlice` and `CopyMeshForwardSlice` topology operators

**Files:**
- Modify: `v3/crates/v3-core/src/mutation/topology/mod.rs`

**Step 1: Write failing tests**

Add to the existing `mod tests` block in `topology/mod.rs`:

```rust
#[test]
fn copy_mesh_backward_slice_duplicates_feeding_pipeline() {
    // Build a 3-node chain: 0 -> 1 -> 2
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(2)],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let before = genome.nodes.len();
    // Try many seeds to find one that produces a multi-node slice
    let mut found_multi = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result =
            TopologyMutator::apply(&mut g, TopologyOperator::CopyMeshBackwardSlice, &mut r);
        if result.is_ok() && g.nodes.len() > before + 1 {
            found_multi = true;
            // Verify all new nodes have unique IDs
            let ids: Vec<NodeId> = g.nodes.iter().map(|n| n.node_id).collect();
            let unique: std::collections::HashSet<_> = ids.iter().collect();
            assert_eq!(ids.len(), unique.len(), "all node IDs must be unique");
            break;
        }
    }
    assert!(
        found_multi,
        "must find a seed that copies multiple nodes"
    );
}

#[test]
fn copy_mesh_backward_slice_remaps_internal_targets() {
    // Node 0 -> Node 1; backward from node 1 includes both.
    // After copy: cloned nodes should reference each other, not originals.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    // Find a seed where anchor_idx=1, producing a 2-node slice
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result =
            TopologyMutator::apply(&mut g, TopologyOperator::CopyMeshBackwardSlice, &mut r);
        if result.is_ok() && g.nodes.len() == 4 {
            // Two new nodes added. The cloned "node 0" should target the cloned "node 1",
            // not the original NodeId(1).
            let original_ids: std::collections::HashSet<NodeId> =
                genome.nodes.iter().map(|n| n.node_id).collect();
            let new_nodes: Vec<&NodeGenome> = g
                .nodes
                .iter()
                .filter(|n| !original_ids.contains(&n.node_id))
                .collect();
            assert_eq!(new_nodes.len(), 2);
            // Find the cloned "node 0" (the one with targets)
            if let Some(cloned_with_targets) = new_nodes.iter().find(|n| !n.targets.is_empty()) {
                // Its targets should reference the OTHER new node, not original NodeId(1)
                for target in &cloned_with_targets.targets {
                    if original_ids.contains(target) {
                        // External target — allowed (e.g. targets pointing outside the slice)
                    } else {
                        // Internal remapped target — must match one of the new node IDs
                        let new_ids: Vec<NodeId> = new_nodes.iter().map(|n| n.node_id).collect();
                        assert!(
                            new_ids.contains(target),
                            "remapped target {:?} must be in new node IDs {:?}",
                            target,
                            new_ids
                        );
                    }
                }
            }
            return;
        }
    }
    panic!("could not find a seed that produces a 2-node backward slice copy");
}

#[test]
fn copy_mesh_forward_slice_duplicates_downstream_subtree() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1)],
            },
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(2)],
            },
            NodeGenome {
                node_id: NodeId::new(2),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            },
        ],
    };
    let before = genome.nodes.len();
    let mut found_multi = false;
    for seed in 0u64..200 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        let result =
            TopologyMutator::apply(&mut g, TopologyOperator::CopyMeshForwardSlice, &mut r);
        if result.is_ok() && g.nodes.len() > before + 1 {
            found_multi = true;
            let ids: Vec<NodeId> = g.nodes.iter().map(|n| n.node_id).collect();
            let unique: std::collections::HashSet<_> = ids.iter().collect();
            assert_eq!(ids.len(), unique.len(), "all node IDs must be unique");
            break;
        }
    }
    assert!(
        found_multi,
        "must find a seed that copies multiple nodes via forward slice"
    );
}

#[test]
fn copy_mesh_operators_single_node_genome_returns_single_copy() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    };
    let mut r = rng(42);
    TopologyMutator::apply(
        &mut genome,
        TopologyOperator::CopyMeshBackwardSlice,
        &mut r,
    )
    .unwrap();
    assert_eq!(genome.nodes.len(), 2);
    assert_ne!(genome.nodes[0].node_id, genome.nodes[1].node_id);
}

#[test]
fn copy_mesh_operators_pass_parseability_gate() {
    let operators = [
        TopologyOperator::CopyMeshBackwardSlice,
        TopologyOperator::CopyMeshForwardSlice,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 200);
        let result = TopologyMutator::apply(&mut genome, op, &mut r);
        match result {
            Ok(()) | Err(MutationSkipReason::NoApplicableTarget) => {
                assert!(
                    ParseabilityGate::validate(&genome).is_ok(),
                    "parseability failed after {:?}: {:?}",
                    op,
                    ParseabilityGate::validate(&genome)
                );
            }
            Err(other) => panic!("unexpected skip reason {:?} for {:?}", other, op),
        }
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cd v3 && cargo test -p v3-core -- topology::tests::copy_mesh 2>&1 | tail -5`
Expected: compilation error — `CopyMeshBackwardSlice` not found

**Step 3: Add enum variants and operator dispatch**

Update `TopologyOperator` enum in `topology/mod.rs`:

```rust
pub enum TopologyOperator {
    AddNode,
    RemoveNode,
    RetargetNodeTarget,
    AddRouteTarget,
    RemoveRouteTarget,
    ChangeEntryNode,
    SwapNodeBackend,
    RewriteNodeId,
    CopyNode,
    CopyMeshBackwardSlice,
    CopyMeshForwardSlice,
}
```

Update `random()`:

```rust
pub fn random(rng: &mut impl Rng) -> Self {
    match rng.gen_range(0u8..11) {
        0 => Self::AddNode,
        1 => Self::RemoveNode,
        2 => Self::RetargetNodeTarget,
        3 => Self::AddRouteTarget,
        4 => Self::RemoveRouteTarget,
        5 => Self::ChangeEntryNode,
        6 => Self::SwapNodeBackend,
        7 => Self::RewriteNodeId,
        8 => Self::CopyNode,
        9 => Self::CopyMeshBackwardSlice,
        _ => Self::CopyMeshForwardSlice,
    }
}
```

Update `TopologyMutator::apply` match:

```rust
TopologyOperator::CopyMeshBackwardSlice => apply_copy_mesh_backward_slice(genome, rng),
TopologyOperator::CopyMeshForwardSlice => apply_copy_mesh_forward_slice(genome, rng),
```

Add import at the top of the file:

```rust
use crate::creature::genome::analysis::{mesh_backward_slice_random, mesh_forward_slice_random};
```

Also add `std::collections::HashMap` to imports for ID remapping.

**Step 4: Implement the operator functions**

Add after `apply_copy_node`:

```rust
/// Maximum number of nodes in a mesh slice for copy operators.
const MESH_SLICE_MAX_SIZE: usize = 8;

fn clone_and_remap_slice(
    genome: &mut CreatureGenome,
    gene_indices: &[usize],
    rng: &mut impl Rng,
) {
    // Build old_id -> new_id mapping
    let mut id_map = HashMap::new();
    let mut next_id = next_node_id(genome);
    for &idx in gene_indices {
        let old_id = genome.nodes[idx].node_id;
        id_map.insert(old_id, next_id);
        next_id = NodeId::new(next_id.0.wrapping_add(1));
    }

    // Clone nodes with remapped IDs and internal targets
    let old_ids: std::collections::HashSet<NodeId> = gene_indices
        .iter()
        .map(|&idx| genome.nodes[idx].node_id)
        .collect();

    let cloned: Vec<NodeGenome> = gene_indices
        .iter()
        .map(|&idx| {
            let original = &genome.nodes[idx];
            NodeGenome {
                node_id: id_map[&original.node_id],
                input_refs: original.input_refs.clone(),
                backend_def: original.backend_def.clone(),
                targets: original
                    .targets
                    .iter()
                    .map(|t| {
                        if old_ids.contains(t) {
                            id_map[t]
                        } else {
                            *t // external target preserved
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    genome.nodes.extend(cloned);

    // 50% chance: add backlink from random existing node to a random cloned node
    if rng.gen_bool(0.5) {
        let new_ids: Vec<NodeId> = id_map.values().copied().collect();
        let link_target = new_ids[rng.gen_range(0..new_ids.len())];
        let source_idx = rng.gen_range(0..genome.nodes.len() - gene_indices.len());
        genome.nodes[source_idx].targets.push(link_target);
    }
}

fn apply_copy_mesh_backward_slice(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let gene = mesh_backward_slice_random(genome, rng, MESH_SLICE_MAX_SIZE)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    clone_and_remap_slice(genome, &gene.indices, rng);
    Ok(())
}

fn apply_copy_mesh_forward_slice(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let gene = mesh_forward_slice_random(genome, rng, MESH_SLICE_MAX_SIZE)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    clone_and_remap_slice(genome, &gene.indices, rng);
    Ok(())
}
```

**Step 5: Update the parseability gate test to include new operators**

In the existing `topology_after_each_operator_passes_parseability_gate` test, add the two new operators to the `operators` array:

```rust
let operators = [
    TopologyOperator::AddNode,
    TopologyOperator::RemoveNode,
    TopologyOperator::RetargetNodeTarget,
    TopologyOperator::AddRouteTarget,
    TopologyOperator::RemoveRouteTarget,
    TopologyOperator::ChangeEntryNode,
    TopologyOperator::SwapNodeBackend,
    TopologyOperator::RewriteNodeId,
    TopologyOperator::CopyNode,
    TopologyOperator::CopyMeshBackwardSlice,
    TopologyOperator::CopyMeshForwardSlice,
];
```

**Step 6: Run tests to verify they pass**

Run: `cd v3 && cargo test -p v3-core -- topology::tests -v`
Expected: all topology tests pass (existing + new)

**Step 7: Commit**

```bash
git add v3/crates/v3-core/src/mutation/topology/mod.rs
git commit -m "feat: add CopyMeshBackwardSlice and CopyMeshForwardSlice topology operators"
```

---

## Task 5: Full verification

**Step 1: Format check**

Run: `cd v3 && cargo fmt --all -- --check`
Expected: no formatting issues (fix with `cargo fmt --all` if needed)

**Step 2: Clippy**

Run: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings

**Step 3: Full workspace tests**

Run: `cd v3 && cargo test --workspace`
Expected: all tests pass

**Step 4: Viability tests**

Run: `cd v3 && cargo test -p v3-core --test viability`
Expected: all 18 viability tests pass

**Step 5: Harness checks**

Run:
```bash
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode warn
scripts/check-architecture-harness.sh --mode warn
```
Expected: our files pass (pre-existing failures acceptable)

**Step 6: Commit any fixes**

If any verification steps required fixes, commit them.

---

## Task 6: Code review (recursive)

**Step 1: Run code review**

Use `superpowers:requesting-code-review` with `rust-skills` focus. Review all changes since the worktree branched.

**Step 2: Fix all Critical and Important issues**

Apply fixes from the review.

**Step 3: Re-review**

Run the code review again. Repeat Steps 2-3 until the review returns no Critical or Important issues. Minor/Nitpick issues may be noted but do not block.

**Step 4: Commit fixes**

```bash
git add -A
git commit -m "fix: address code review findings"
```

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

**Review cycles:** 1
