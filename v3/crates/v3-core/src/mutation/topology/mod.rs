use std::collections::HashMap;

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::genome::analysis::{mesh_backward_slice, mesh_forward_slice};
use crate::creature::genome::cgp::CgpGraphBackendDef;
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

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

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::AddNode
            | Self::CopyNode
            | Self::CopyMeshBackwardSlice
            | Self::CopyMeshForwardSlice
            | Self::SpliceNode
            | Self::AddRouteTarget => ComplexityEffect::Increasing,
            Self::RemoveNode | Self::RemoveRouteTarget => ComplexityEffect::Decreasing,
            Self::RetargetNodeTarget
            | Self::ChangeEntryNode
            | Self::SwapNodeBackend
            | Self::RewriteNodeId
            | Self::SwapRouteTargets => ComplexityEffect::Neutral,
        }
    }

    const NON_INCREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if !Self::ALL[i].complexity_effect().is_increasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    const DECREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if Self::ALL[i].complexity_effect().is_decreasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    /// Pick a random Decreasing-only operator weighted by impact tier.
    pub fn random_decreasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::DECREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::DECREASING_WEIGHT);
        for &op in &Self::ALL {
            if !op.complexity_effect().is_decreasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

    /// Pick a random non-increasing operator (Neutral or Decreasing) weighted by impact tier.
    pub fn random_non_increasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::NON_INCREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::NON_INCREASING_WEIGHT);
        for &op in &Self::ALL {
            if op.complexity_effect().is_increasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

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
    /// Apply a topology operator to the genome with reachability-biased target selection.
    ///
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason)` if no
    /// applicable target exists. Exempt operators (AddNode, ChangeEntryNode) return
    /// `NotApplicable` since they don't select a target node.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: TopologyOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
    ) -> Result<TargetReachability, MutationSkipReason> {
        match op {
            // Exempt: these don't select a target node for mutation.
            TopologyOperator::AddNode => {
                apply_add_node(genome, rng).map(|()| TargetReachability::NotApplicable)
            }
            TopologyOperator::ChangeEntryNode => {
                apply_change_entry_node(genome, rng).map(|()| TargetReachability::NotApplicable)
            }
            // Biased operators:
            TopologyOperator::RemoveNode => apply_remove_node(genome, reachable_nodes, bias, rng),
            TopologyOperator::RetargetNodeTarget => {
                apply_retarget_node_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::AddRouteTarget => {
                apply_add_route_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::RemoveRouteTarget => {
                apply_remove_route_target(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::SwapNodeBackend => {
                apply_swap_node_backend(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::RewriteNodeId => {
                apply_rewrite_node_id(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::CopyNode => apply_copy_node(genome, reachable_nodes, bias, rng),
            TopologyOperator::CopyMeshBackwardSlice => {
                apply_copy_mesh_backward_slice(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::CopyMeshForwardSlice => {
                apply_copy_mesh_forward_slice(genome, reachable_nodes, bias, rng)
            }
            TopologyOperator::SpliceNode => apply_splice_node(genome, reachable_nodes, bias, rng),
            TopologyOperator::SwapRouteTargets => {
                apply_swap_route_targets(genome, reachable_nodes, bias, rng)
            }
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
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
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
    let (idx, reachability) = biased_select_from(&removable, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    genome.nodes.remove(idx);
    Ok(reachability)
}

fn apply_retarget_node_target(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    // Find nodes with non-empty targets.
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
    let new_target = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    genome.nodes[node_idx].targets[target_slot] = new_target;
    Ok(reachability)
}

fn apply_add_route_target(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (node_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_id = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    genome.nodes[node_idx].targets.push(target_id);
    Ok(reachability)
}

fn apply_remove_route_target(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
    genome.nodes[node_idx].targets.remove(target_slot);
    Ok(reachability)
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
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let node = &mut genome.nodes[idx];
    node.backend_def = match &node.backend_def {
        BackendDef::Vm(_) => BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs(
            &MutationConfig::default(),
        )),
        BackendDef::Graph(_) => BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![VmInstruction::Halt],
        }),
    };
    Ok(reachability)
}

fn apply_rewrite_node_id(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
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

    Ok(reachability)
}

fn apply_copy_node(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (source_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
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
    genome.nodes.push(NodeGenome {
        node_id: new_id,
        input_refs,
        backend_def,
        targets,
    });

    // Always add backlink to ensure the copied node is reachable.
    genome.nodes[source_idx].targets.push(new_id);
    Ok(reachability)
}

/// Maximum number of nodes in a mesh slice for copy operators.
const MESH_SLICE_MAX_SIZE: usize = 8;

/// Clone a set of nodes identified by `gene_indices`, remapping internal
/// target references to fresh NodeIds. Appends cloned nodes to the genome.
/// With 50% probability, offsets CustomOutput slots in cloned Graph backends
/// to avoid clobbering the originals. Always adds a backlink from a random
/// pre-existing node to a random cloned node to ensure reachability.
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

    // Note: previously remapped CustomOutput slots in cloned Graph backends
    // to avoid clobbering. CGP Graph backends have fixed output sinks (not
    // remappable), and VM backends don't have CustomOutput node kinds, so
    // this remapping step is now a no-op and has been removed.

    // Always add backlink from random pre-existing node to a random cloned node
    // to ensure reachability.
    let new_ids: Vec<NodeId> = id_map.values().copied().collect();
    let link_target = new_ids[rng.gen_range(0..new_ids.len())];
    let source_idx = rng.gen_range(0..pre_existing_count);
    genome.nodes[source_idx].targets.push(link_target);
}

fn apply_copy_mesh_backward_slice(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (anchor_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let gene = mesh_backward_slice(genome, anchor_idx, MESH_SLICE_MAX_SIZE)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    clone_and_remap_slice(genome, &gene.indices, rng);
    Ok(reachability)
}

fn apply_copy_mesh_forward_slice(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (seed_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let gene = mesh_forward_slice(genome, seed_idx, MESH_SLICE_MAX_SIZE)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    clone_and_remap_slice(genome, &gene.indices, rng);
    Ok(reachability)
}

fn apply_splice_node(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
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
    let (a_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
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
    Ok(reachability)
}

fn apply_swap_route_targets(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
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
    let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let len = genome.nodes[node_idx].targets.len();
    let a = rng.gen_range(0..len);
    let mut b = rng.gen_range(0..len - 1);
    if b >= a {
        b += 1;
    }
    genome.nodes[node_idx].targets.swap(a, b);
    Ok(reachability)
}

#[cfg(test)]
mod tests;
