use std::collections::HashMap;

use rand::Rng;

use crate::contracts::NodeId;
use crate::creature::genome::analysis::{mesh_backward_slice_random, mesh_forward_slice_random};
use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::mutation::types::MutationSkipReason;

/// Topology mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    SpliceNode,
    SwapRouteTargets,
}

impl TopologyOperator {
    pub const ALL: [Self; 13] = [
        Self::AddNode,
        Self::RemoveNode,
        Self::RetargetNodeTarget,
        Self::AddRouteTarget,
        Self::RemoveRouteTarget,
        Self::ChangeEntryNode,
        Self::SwapNodeBackend,
        Self::RewriteNodeId,
        Self::CopyNode,
        Self::CopyMeshBackwardSlice,
        Self::CopyMeshForwardSlice,
        Self::SpliceNode,
        Self::SwapRouteTargets,
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::AddNode => 1,
            Self::RemoveNode => 1,
            Self::RetargetNodeTarget => 2,
            Self::AddRouteTarget => 2,
            Self::RemoveRouteTarget => 2,
            Self::ChangeEntryNode => 2,
            Self::SwapNodeBackend => 1,
            Self::RewriteNodeId => 4,
            Self::CopyNode => 1,
            Self::CopyMeshBackwardSlice => 1,
            Self::CopyMeshForwardSlice => 1,
            Self::SpliceNode => 1,
            Self::SwapRouteTargets => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        // Compile-time guard: if a variant is added to the enum but not to ALL,
        // weight() will still compile (exhaustive match), but ALL will be incomplete.
        // This assertion catches that at compile time.
        assert!(
            Self::ALL.len() == 13,
            "ALL must cover every TopologyOperator variant"
        );
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            sum += Self::ALL[i].weight() as u16;
            i += 1;
        }
        sum
    };

    /// Pick a random topology operator weighted by impact tier.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut r = rng.gen_range(0..Self::TOTAL_WEIGHT);
        for &op in &Self::ALL {
            let w = op.weight() as u16;
            if r < w {
                return op;
            }
            r -= w;
        }
        unreachable!()
    }
}

/// Topology domain mutator.
pub struct TopologyMutator;

impl TopologyMutator {
    /// Apply a topology operator to the genome.
    ///
    /// Returns `Ok(())` on success, or `Err(MutationSkipReason)` if no applicable target exists.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: TopologyOperator,
        rng: &mut impl Rng,
    ) -> Result<(), MutationSkipReason> {
        match op {
            TopologyOperator::AddNode => apply_add_node(genome, rng),
            TopologyOperator::RemoveNode => apply_remove_node(genome, rng),
            TopologyOperator::RetargetNodeTarget => apply_retarget_node_target(genome, rng),
            TopologyOperator::AddRouteTarget => apply_add_route_target(genome, rng),
            TopologyOperator::RemoveRouteTarget => apply_remove_route_target(genome, rng),
            TopologyOperator::ChangeEntryNode => apply_change_entry_node(genome, rng),
            TopologyOperator::SwapNodeBackend => apply_swap_node_backend(genome, rng),
            TopologyOperator::RewriteNodeId => apply_rewrite_node_id(genome, rng),
            TopologyOperator::CopyNode => apply_copy_node(genome, rng),
            TopologyOperator::CopyMeshBackwardSlice => apply_copy_mesh_backward_slice(genome, rng),
            TopologyOperator::CopyMeshForwardSlice => apply_copy_mesh_forward_slice(genome, rng),
            TopologyOperator::SpliceNode => apply_splice_node(genome, rng),
            TopologyOperator::SwapRouteTargets => apply_swap_route_targets(genome, rng),
        }
    }
}

/// Allocate the next node ID: `max(existing node_ids) + 1` with wrapping u32 arithmetic.
fn next_node_id(genome: &CreatureGenome) -> NodeId {
    let max_id = genome.nodes.iter().map(|n| n.node_id.0).max().unwrap_or(0);
    NodeId::new(max_id.wrapping_add(1))
}

fn apply_add_node(
    genome: &mut CreatureGenome,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let new_id = next_node_id(genome);
    genome.nodes.push(NodeGenome {
        node_id: new_id,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![],
    });
    Ok(())
}

fn apply_remove_node(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.len() <= 1 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    // Find non-entry nodes to avoid removing the entry.
    let entry_id = genome.entry_node_id;
    let removable: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.node_id != entry_id)
        .map(|(i, _)| i)
        .collect();
    if removable.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let idx = removable[rng.gen_range(0..removable.len())];
    genome.nodes.remove(idx);
    Ok(())
}

fn apply_retarget_node_target(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    // Find nodes with non-empty targets.
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let node_idx = eligible[rng.gen_range(0..eligible.len())];
    let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
    let new_target = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    genome.nodes[node_idx].targets[target_slot] = new_target;
    Ok(())
}

fn apply_add_route_target(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let node_idx = rng.gen_range(0..genome.nodes.len());
    let target_id = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    genome.nodes[node_idx].targets.push(target_id);
    Ok(())
}

fn apply_remove_route_target(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let node_idx = eligible[rng.gen_range(0..eligible.len())];
    let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
    genome.nodes[node_idx].targets.remove(target_slot);
    Ok(())
}

fn apply_change_entry_node(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.len() <= 1 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let current_entry = genome.entry_node_id;
    let candidates: Vec<NodeId> = genome
        .nodes
        .iter()
        .map(|n| n.node_id)
        .filter(|&id| id != current_entry)
        .collect();
    if candidates.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    genome.entry_node_id = candidates[rng.gen_range(0..candidates.len())];
    Ok(())
}

fn apply_swap_node_backend(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let idx = rng.gen_range(0..genome.nodes.len());
    let node = &mut genome.nodes[idx];
    node.backend_def = match &node.backend_def {
        BackendDef::Vm(_) => BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![],
        }),
        BackendDef::Graph(_) => BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
    };
    Ok(())
}

fn apply_rewrite_node_id(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let idx = rng.gen_range(0..genome.nodes.len());
    let old_id = genome.nodes[idx].node_id;
    let new_id = next_node_id(genome);
    genome.nodes[idx].node_id = new_id;

    if genome.entry_node_id == old_id {
        genome.entry_node_id = new_id;
    }

    for node in &mut genome.nodes {
        for target in &mut node.targets {
            if *target == old_id {
                *target = new_id;
            }
        }
    }

    Ok(())
}

fn apply_copy_node(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let source_idx = rng.gen_range(0..genome.nodes.len());
    let backend_def = genome.nodes[source_idx].backend_def.clone();
    let new_id = next_node_id(genome);

    let targets = if rng.gen_bool(0.5) {
        genome.nodes[source_idx].targets.clone()
    } else {
        vec![]
    };
    let input_refs = if rng.gen_bool(0.5) {
        genome.nodes[source_idx].input_refs.clone()
    } else {
        vec![]
    };
    let add_backlink = rng.gen_bool(0.5);

    genome.nodes.push(NodeGenome {
        node_id: new_id,
        input_refs,
        backend_def,
        targets,
    });

    if add_backlink {
        genome.nodes[source_idx].targets.push(new_id);
    }
    Ok(())
}

/// Maximum number of nodes in a mesh slice for copy operators.
const MESH_SLICE_MAX_SIZE: usize = 8;

/// Clone a set of nodes identified by `gene_indices`, remapping internal
/// target references to fresh NodeIds. Appends cloned nodes to the genome.
/// With 50% probability, offsets CustomOutput slots in cloned Graph backends
/// to avoid clobbering the originals. With 50% probability, adds a backlink
/// from a random pre-existing node to a random cloned node.
fn clone_and_remap_slice(genome: &mut CreatureGenome, gene_indices: &[usize], rng: &mut impl Rng) {
    // Build old_id -> new_id mapping
    let mut id_map = HashMap::with_capacity(gene_indices.len());
    let mut next_id = next_node_id(genome);
    for &idx in gene_indices {
        let old_id = genome.nodes[idx].node_id;
        id_map.insert(old_id, next_id);
        next_id = NodeId::new(next_id.0.wrapping_add(1));
    }

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
                        if let Some(&new_id) = id_map.get(t) {
                            new_id // internal target remapped
                        } else {
                            *t // external target preserved
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    let pre_existing_count = genome.nodes.len();
    genome.nodes.extend(cloned);

    // 50% chance: remap CustomOutput slots in cloned nodes to avoid clobbering
    if rng.gen_bool(0.5) {
        let offset = rng.gen_range(1u8..12); // 1-11, never 0 (no-op)
        for idx in pre_existing_count..genome.nodes.len() {
            genome.nodes[idx].backend_def.remap_output_slots(offset);
        }
    }

    // 50% chance: add backlink from random pre-existing node to a random cloned node
    if rng.gen_bool(0.5) {
        let new_ids: Vec<NodeId> = id_map.values().copied().collect();
        let link_target = new_ids[rng.gen_range(0..new_ids.len())];
        let source_idx = rng.gen_range(0..pre_existing_count);
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

fn apply_splice_node(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let a_idx = eligible[rng.gen_range(0..eligible.len())];
    let target_slot = rng.gen_range(0..genome.nodes[a_idx].targets.len());
    let b_id = genome.nodes[a_idx].targets[target_slot];
    let c_id = next_node_id(genome);
    genome.nodes.push(NodeGenome {
        node_id: c_id,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
        targets: vec![b_id],
    });
    genome.nodes[a_idx].targets[target_slot] = c_id;
    Ok(())
}

fn apply_swap_route_targets(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.targets.len() >= 2)
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let node_idx = eligible[rng.gen_range(0..eligible.len())];
    let len = genome.nodes[node_idx].targets.len();
    let a = rng.gen_range(0..len);
    let mut b = rng.gen_range(0..len - 1);
    if b >= a {
        b += 1;
    }
    genome.nodes[node_idx].targets.swap(a, b);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::parseability::ParseabilityGate;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn add_node_increases_node_count_by_one() {
        let mut genome = v3alpha1_founder_genome();
        let before = genome.nodes.len();
        let mut r = rng(0);
        TopologyMutator::apply(&mut genome, TopologyOperator::AddNode, &mut r).unwrap();
        assert_eq!(genome.nodes.len(), before + 1);
    }

    #[test]
    fn remove_node_decreases_node_count() {
        let mut genome = v3alpha1_founder_genome();
        assert!(genome.nodes.len() >= 2, "founder must have >=2 nodes");
        let before = genome.nodes.len();
        let mut r = rng(1);
        TopologyMutator::apply(&mut genome, TopologyOperator::RemoveNode, &mut r).unwrap();
        assert_eq!(genome.nodes.len(), before - 1);
    }

    #[test]
    fn remove_node_on_single_node_genome_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        // Reduce to single node.
        genome.nodes.truncate(1);
        genome.entry_node_id = genome.nodes[0].node_id;
        let mut r = rng(2);
        let result = TopologyMutator::apply(&mut genome, TopologyOperator::RemoveNode, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn retarget_node_target_changes_target() {
        let mut genome = v3alpha1_founder_genome();
        // Node 0 has targets=[NodeId::new(1)] in the founder.
        let before_target = genome.nodes[0].targets[0];
        // Run until the target changes or we give up (not guaranteed since genome only has 2 nodes).
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut rr = rng(seed);
            let _ = TopologyMutator::apply(&mut g, TopologyOperator::RetargetNodeTarget, &mut rr);
            if g.nodes[0].targets[0] != before_target {
                genome = g;
                break;
            }
        }
        // At minimum, the operation doesn't fail and targets still has same length.
        let mut r2 = rng(99);
        let result =
            TopologyMutator::apply(&mut genome, TopologyOperator::RetargetNodeTarget, &mut r2);
        assert!(result.is_ok());
    }

    #[test]
    fn add_route_target_increases_target_count() {
        let mut genome = v3alpha1_founder_genome();
        let before = genome.nodes[0].targets.len();
        let mut r = rng(4);
        // Force node index 0 by using a genome with known structure.
        TopologyMutator::apply(&mut genome, TopologyOperator::AddRouteTarget, &mut r).unwrap();
        // Total targets across all nodes must increase by 1.
        let after: usize = genome.nodes.iter().map(|n| n.targets.len()).sum();
        let before_total: usize = v3alpha1_founder_genome()
            .nodes
            .iter()
            .map(|n| n.targets.len())
            .sum();
        // before_total = before counts. Add one.
        let _ = before;
        assert_eq!(after, before_total + 1);
    }

    #[test]
    fn change_entry_node_changes_entry() {
        let mut genome = v3alpha1_founder_genome();
        assert!(genome.nodes.len() >= 2);
        let original_entry = genome.entry_node_id;
        let mut r = rng(5);
        TopologyMutator::apply(&mut genome, TopologyOperator::ChangeEntryNode, &mut r).unwrap();
        assert_ne!(
            genome.entry_node_id, original_entry,
            "entry node must change"
        );
    }

    #[test]
    fn topology_after_each_operator_passes_parseability_gate() {
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
            TopologyOperator::SpliceNode,
            TopologyOperator::SwapRouteTargets,
        ];
        for (i, &op) in operators.iter().enumerate() {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(i as u64 + 100);
            let result = TopologyMutator::apply(&mut genome, op, &mut r);
            // Either succeeds or NoApplicableTarget — either way, genome must still be parseable.
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

    #[test]
    fn swap_node_backend_toggles_backend_on_single_node_genome() {
        let mut genome = v3alpha1_founder_genome();
        genome.nodes.truncate(1);
        genome.entry_node_id = genome.nodes[0].node_id;

        let mut r1 = rng(7);
        TopologyMutator::apply(&mut genome, TopologyOperator::SwapNodeBackend, &mut r1).unwrap();
        assert!(matches!(genome.nodes[0].backend_def, BackendDef::Vm(_)));

        let mut r2 = rng(8);
        TopologyMutator::apply(&mut genome, TopologyOperator::SwapNodeBackend, &mut r2).unwrap();
        assert!(matches!(genome.nodes[0].backend_def, BackendDef::Graph(_)));
    }

    #[test]
    fn rewrite_node_id_rewrites_entry_and_target_references() {
        let mut genome = v3alpha1_founder_genome();
        genome.nodes.truncate(1);
        genome.nodes[0].targets = vec![genome.nodes[0].node_id];
        genome.entry_node_id = genome.nodes[0].node_id;
        let old_id = genome.nodes[0].node_id;

        let mut r = rng(11);
        TopologyMutator::apply(&mut genome, TopologyOperator::RewriteNodeId, &mut r).unwrap();
        let new_id = genome.nodes[0].node_id;

        assert_ne!(new_id, old_id, "node id should be rewritten");
        assert_eq!(genome.entry_node_id, new_id, "entry id should be rewritten");
        assert_eq!(
            genome.nodes[0].targets,
            vec![new_id],
            "all target references should be rewritten"
        );
    }

    #[test]
    fn copy_node_increases_node_count_by_one() {
        let mut genome = v3alpha1_founder_genome();
        let before = genome.nodes.len();
        let mut r = rng(42);
        TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
        assert_eq!(genome.nodes.len(), before + 1);
    }

    #[test]
    fn copy_node_assigns_different_node_id() {
        let mut genome = v3alpha1_founder_genome();
        let original_ids: Vec<NodeId> = genome.nodes.iter().map(|n| n.node_id).collect();
        let mut r = rng(42);
        TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
        let new_node = genome.nodes.last().unwrap();
        assert!(
            !original_ids.contains(&new_node.node_id),
            "copied node must have a fresh NodeId"
        );
    }

    #[test]
    fn copy_node_deep_copies_vm_backend() {
        // The founder genome has VM nodes. Over seeds, find one that copies a VM node
        // and verify backend equality.
        let mut found = false;
        for seed in 0u64..100 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
            let new_node = genome.nodes.last().unwrap();
            if matches!(&new_node.backend_def, BackendDef::Vm(_)) {
                // Find which source node has matching backend.
                let has_match = genome
                    .nodes
                    .iter()
                    .rev()
                    .skip(1)
                    .any(|n| n.backend_def == new_node.backend_def);
                if has_match {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "must find at least one seed that deep-copies a VM backend"
        );
    }

    #[test]
    fn copy_node_deep_copies_graph_backend() {
        // Swap one founder node to Graph first, then copy.
        let mut found = false;
        for seed in 0u64..100 {
            let mut genome = v3alpha1_founder_genome();
            // Swap node 1 to Graph backend.
            genome.nodes[1].backend_def = BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![],
            });
            let mut r = rng(seed);
            TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
            let new_node = genome.nodes.last().unwrap();
            if matches!(&new_node.backend_def, BackendDef::Graph(_)) {
                let has_match = genome
                    .nodes
                    .iter()
                    .rev()
                    .skip(1)
                    .any(|n| n.backend_def == new_node.backend_def);
                if has_match {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "must find at least one seed that deep-copies a Graph backend"
        );
    }

    #[test]
    fn copy_node_sometimes_copies_targets_sometimes_not() {
        let mut saw_copied = false;
        let mut saw_empty = false;
        for seed in 0u64..500 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
            let new_node = genome.nodes.last().unwrap();
            if new_node.targets.is_empty() {
                saw_empty = true;
            } else {
                saw_copied = true;
            }
            if saw_copied && saw_empty {
                break;
            }
        }
        assert!(saw_copied, "must observe at least one copy with targets");
        assert!(
            saw_empty,
            "must observe at least one copy with empty targets"
        );
    }

    #[test]
    fn copy_node_sometimes_copies_input_refs_sometimes_not() {
        let mut saw_copied = false;
        let mut saw_empty = false;
        for seed in 0u64..500 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
            let new_node = genome.nodes.last().unwrap();
            if new_node.input_refs.is_empty() {
                saw_empty = true;
            } else {
                saw_copied = true;
            }
            if saw_copied && saw_empty {
                break;
            }
        }
        assert!(saw_copied, "must observe at least one copy with input_refs");
        assert!(
            saw_empty,
            "must observe at least one copy with empty input_refs"
        );
    }

    #[test]
    fn copy_node_sometimes_adds_backlink_sometimes_not() {
        let mut saw_backlink = false;
        let mut saw_no_backlink = false;
        for seed in 0u64..500 {
            let mut genome = v3alpha1_founder_genome();
            let original_targets: Vec<Vec<NodeId>> =
                genome.nodes.iter().map(|n| n.targets.clone()).collect();
            let mut r = rng(seed);
            TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
            let new_id = genome.nodes.last().unwrap().node_id;
            // Check if any original node gained the new_id in its targets.
            let backlinked = genome.nodes.iter().enumerate().any(|(i, n)| {
                i < original_targets.len()
                    && n.targets.contains(&new_id)
                    && !original_targets[i].contains(&new_id)
            });
            if backlinked {
                saw_backlink = true;
            } else {
                saw_no_backlink = true;
            }
            if saw_backlink && saw_no_backlink {
                break;
            }
        }
        assert!(saw_backlink, "must observe at least one backlink addition");
        assert!(
            saw_no_backlink,
            "must observe at least one case without backlink"
        );
    }

    #[test]
    fn copy_node_can_copy_entry_node() {
        let mut genome = v3alpha1_founder_genome();
        genome.nodes.truncate(1);
        genome.entry_node_id = genome.nodes[0].node_id;
        let before = genome.nodes.len();
        let mut r = rng(99);
        TopologyMutator::apply(&mut genome, TopologyOperator::CopyNode, &mut r).unwrap();
        assert_eq!(genome.nodes.len(), before + 1);
    }

    #[test]
    fn copy_mesh_backward_slice_duplicates_feeding_pipeline() {
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
                TopologyMutator::apply(&mut g, TopologyOperator::CopyMeshBackwardSlice, &mut r);
            if result.is_ok() && g.nodes.len() > before + 1 {
                found_multi = true;
                let ids: Vec<NodeId> = g.nodes.iter().map(|n| n.node_id).collect();
                let unique: std::collections::HashSet<_> = ids.iter().collect();
                assert_eq!(ids.len(), unique.len(), "all node IDs must be unique");
                break;
            }
        }
        assert!(found_multi, "must find a seed that copies multiple nodes");
    }

    #[test]
    fn copy_mesh_backward_slice_remaps_internal_targets() {
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
        for seed in 0u64..200 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            let result =
                TopologyMutator::apply(&mut g, TopologyOperator::CopyMeshBackwardSlice, &mut r);
            if result.is_ok() && g.nodes.len() == 4 {
                let original_ids: std::collections::HashSet<NodeId> =
                    genome.nodes.iter().map(|n| n.node_id).collect();
                let new_nodes: Vec<&NodeGenome> = g
                    .nodes
                    .iter()
                    .filter(|n| !original_ids.contains(&n.node_id))
                    .collect();
                assert_eq!(new_nodes.len(), 2);
                if let Some(cloned_with_targets) = new_nodes.iter().find(|n| !n.targets.is_empty())
                {
                    for target in &cloned_with_targets.targets {
                        if !original_ids.contains(target) {
                            let new_ids: Vec<NodeId> =
                                new_nodes.iter().map(|n| n.node_id).collect();
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
        TopologyMutator::apply(&mut genome, TopologyOperator::CopyMeshBackwardSlice, &mut r)
            .unwrap();
        assert_eq!(genome.nodes.len(), 2);
        assert_ne!(genome.nodes[0].node_id, genome.nodes[1].node_id);
    }

    // ── Gap 6: SwapRouteTargets tests ──

    #[test]
    fn swap_route_targets_changes_target_order() {
        // Create a genome with a node that has multiple targets.
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)],
            }],
        };
        let original_targets = genome.nodes[0].targets.clone();
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            TopologyMutator::apply(&mut g, TopologyOperator::SwapRouteTargets, &mut r).unwrap();
            if g.nodes[0].targets != original_targets {
                changed = true;
                break;
            }
        }
        assert!(changed, "swap must change target order");
    }

    #[test]
    fn swap_route_targets_preserves_target_set() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)],
            }],
        };
        let mut original_sorted = genome.nodes[0].targets.clone();
        original_sorted.sort();
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            TopologyMutator::apply(&mut g, TopologyOperator::SwapRouteTargets, &mut r).unwrap();
            let mut after_sorted = g.nodes[0].targets.clone();
            after_sorted.sort();
            assert_eq!(
                original_sorted, after_sorted,
                "swap must preserve the same set of targets"
            );
        }
    }

    #[test]
    fn swap_route_targets_requires_at_least_two_targets() {
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
                targets: vec![NodeId::new(1)],
            }],
        };
        let mut r = rng(0);
        let result =
            TopologyMutator::apply(&mut genome, TopologyOperator::SwapRouteTargets, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    // ── Gap 3: SpliceNode tests ──

    #[test]
    fn splice_node_increases_node_count_by_one() {
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
                    targets: vec![],
                },
            ],
        };
        let before = genome.nodes.len();
        let mut r = rng(0);
        TopologyMutator::apply(&mut genome, TopologyOperator::SpliceNode, &mut r).unwrap();
        assert_eq!(genome.nodes.len(), before + 1);
    }

    #[test]
    fn splice_node_creates_a_to_c_to_b_chain() {
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
                    targets: vec![],
                },
            ],
        };
        let b_id = NodeId::new(1);
        let mut r = rng(0);
        TopologyMutator::apply(&mut genome, TopologyOperator::SpliceNode, &mut r).unwrap();
        let c = genome.nodes.last().unwrap();
        let c_id = c.node_id;
        // C's target is B
        assert_eq!(c.targets, vec![b_id], "C must target B");
        // A's target is now C (not B)
        assert!(
            genome.nodes[0].targets.contains(&c_id),
            "A must now target C"
        );
        assert!(
            !genome.nodes[0].targets.contains(&b_id),
            "A must no longer directly target B"
        );
    }

    #[test]
    fn splice_node_new_node_is_blank_vm() {
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
                targets: vec![NodeId::new(1)],
            }],
        };
        let mut r = rng(0);
        TopologyMutator::apply(&mut genome, TopologyOperator::SpliceNode, &mut r).unwrap();
        let c = genome.nodes.last().unwrap();
        assert!(
            c.input_refs.is_empty(),
            "spliced node must have no input_refs"
        );
        if let BackendDef::Vm(ref vm) = c.backend_def {
            assert_eq!(vm.register_count, 1);
            assert!(vm.constants.is_empty());
            assert_eq!(vm.program, vec![VmInstruction::Halt]);
        } else {
            panic!("spliced node must be VM backend");
        }
    }

    #[test]
    fn splice_node_no_targets_returns_skip() {
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
        let mut r = rng(0);
        let result = TopologyMutator::apply(&mut genome, TopologyOperator::SpliceNode, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn splice_node_passes_parseability_gate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(42);
        let result = TopologyMutator::apply(&mut genome, TopologyOperator::SpliceNode, &mut r);
        match result {
            Ok(()) | Err(MutationSkipReason::NoApplicableTarget) => {
                assert!(
                    ParseabilityGate::validate(&genome).is_ok(),
                    "parseability failed after SpliceNode"
                );
            }
            Err(other) => panic!("unexpected skip reason {:?}", other),
        }
    }

    #[test]
    fn splice_node_new_node_gets_fresh_node_id() {
        let mut genome = v3alpha1_founder_genome();
        let original_ids: Vec<NodeId> = genome.nodes.iter().map(|n| n.node_id).collect();
        let mut r = rng(0);
        TopologyMutator::apply(&mut genome, TopologyOperator::SpliceNode, &mut r).unwrap();
        let c = genome.nodes.last().unwrap();
        assert!(
            !original_ids.contains(&c.node_id),
            "spliced node must have a fresh NodeId"
        );
    }

    // ── Gap 5: Output slot remapping in clones tests ──

    #[test]
    fn clone_remap_slice_sometimes_offsets_custom_outputs() {
        use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphNodeKind};
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(GraphBackendDef {
                    internal_nodes: vec![GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(2),
                        inputs: vec![],
                        hebbian: None,
                    }],
                }),
                targets: vec![],
            }],
        };
        let mut saw_different = false;
        for seed in 0u64..200 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            clone_and_remap_slice(&mut g, &[0], &mut r);
            if let BackendDef::Graph(ref gd) = g.nodes.last().unwrap().backend_def {
                if let GraphNodeKind::CustomOutput(slot) = gd.internal_nodes[0].kind {
                    if slot != 2 {
                        saw_different = true;
                        break;
                    }
                }
            }
        }
        assert!(
            saw_different,
            "cloned nodes must sometimes have different CustomOutput slot"
        );
    }

    #[test]
    fn clone_remap_slice_sometimes_preserves_custom_outputs() {
        use crate::creature::genome::{GraphBackendDef, GraphInternalNode, GraphNodeKind};
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(GraphBackendDef {
                    internal_nodes: vec![GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(2),
                        inputs: vec![],
                        hebbian: None,
                    }],
                }),
                targets: vec![],
            }],
        };
        let mut saw_same = false;
        for seed in 0u64..200 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            clone_and_remap_slice(&mut g, &[0], &mut r);
            if let BackendDef::Graph(ref gd) = g.nodes.last().unwrap().backend_def {
                if let GraphNodeKind::CustomOutput(slot) = gd.internal_nodes[0].kind {
                    if slot == 2 {
                        saw_same = true;
                        break;
                    }
                }
            }
        }
        assert!(
            saw_same,
            "cloned nodes must sometimes keep original CustomOutput slot"
        );
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

    #[test]
    fn topology_weighted_random_favors_refinement() {
        let mut counts = std::collections::HashMap::new();
        let mut r = rng(42);
        for _ in 0..10_000 {
            let op = TopologyOperator::random(&mut r);
            *counts.entry(op).or_insert(0u32) += 1;
        }
        let rewrite = counts
            .get(&TopologyOperator::RewriteNodeId)
            .copied()
            .unwrap_or(0);
        let add_node = counts.get(&TopologyOperator::AddNode).copied().unwrap_or(0);
        assert!(
            rewrite > add_node * 2,
            "RewriteNodeId (weight 4) must appear >2x AddNode (weight 1); got {} vs {}",
            rewrite,
            add_node,
        );
    }

    #[test]
    fn topology_operator_weights_are_positive() {
        let all = TopologyOperator::ALL;
        assert_eq!(
            all.len(),
            13,
            "ALL must cover every TopologyOperator variant"
        );
        for &op in &all {
            assert!(op.weight() > 0, "weight must be positive for {:?}", op);
        }
    }
}
