use std::collections::HashSet;

use rand::Rng;

use crate::contracts::NodeId;
use crate::genome::analysis::{mesh_backward_slice, mesh_forward_slice};
use crate::genome::mesh_edit::Attachment;
use crate::genome::{BackendDef, CreatureGenome};
use crate::mutation::event::{MutationSkipReason, TargetReachability};
use crate::mutation::reachability::TargetSelector;

use super::birth;
use super::routing::{free_gate_slot, lowest_unused_slot, static_incumbent, writes_gate};

/// Find an unused identity even when the largest existing ID wraps.
pub(super) fn next_node_id(genome: &CreatureGenome) -> NodeId {
    let used: HashSet<_> = genome.nodes.iter().map(|n| n.node_id).collect();
    unused_id(
        &used,
        genome
            .nodes
            .iter()
            .map(|n| n.node_id.0)
            .max()
            .unwrap_or(0)
            .wrapping_add(1),
    )
}

fn unused_id(used: &HashSet<NodeId>, mut candidate: u32) -> NodeId {
    while used.contains(&NodeId::new(candidate)) {
        candidate = candidate.wrapping_add(1);
    }
    NodeId::new(candidate)
}

fn bypass_successor(genome: &CreatureGenome, idx: usize) -> Option<NodeId> {
    let node = &genome.nodes[idx];
    let target = node
        .targets
        .get(static_incumbent(&node.targets)?)?
        .target_id;
    (target != node.node_id && genome.nodes.iter().any(|n| n.node_id == target)).then_some(target)
}

pub(super) fn apply_remove_node(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    if genome.nodes.len() <= 1 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let currently_reachable = crate::genome::analysis::mesh_reachable_nodes(genome);
    let unreachable: Vec<_> = (0..genome.nodes.len())
        .filter(|&i| {
            genome.nodes[i].node_id != genome.entry_node_id
                && currently_reachable.binary_search(&i).is_err()
        })
        .collect();
    let eligible = if unreachable.is_empty() {
        (0..genome.nodes.len())
            .filter(|&i| {
                genome.nodes[i].node_id != genome.entry_node_id
                    && bypass_successor(genome, i).is_some()
            })
            .collect()
    } else {
        unreachable
    };
    let (idx, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let successor = bypass_successor(genome, idx);
    genome.remove_node(idx, successor);
    Ok(reachability)
}

pub(super) fn apply_change_entry_node(
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

/// A tied, unwritten alternative loses to the original by position (strict `>`).
fn safe_predecessors(genome: &CreatureGenome, source: usize) -> Vec<(usize, u8, f32)> {
    let source_id = genome.nodes[source].node_id;
    let mut candidates: Vec<_> = genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| {
            let incumbent = node.targets.get(static_incumbent(&node.targets)?)?;
            if incumbent.target_id != source_id
                || !incumbent.gate_bias.is_finite()
                || writes_gate(node.backend_def(), incumbent.slot)
            {
                return None;
            }
            let slot = free_gate_slot(node)?;
            Some((idx, slot, incumbent.gate_bias))
        })
        .collect();
    if !candidates.is_empty() {
        let downstream =
            mesh_forward_slice(genome, source, genome.nodes.len()).expect("existing source");
        candidates.retain(|(idx, _, _)| downstream.indices.binary_search(idx).is_err());
    }
    candidates
}

fn copy_attached(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    swap_backend: bool,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<_> = (0..genome.nodes.len())
        .filter(|&i| !safe_predecessors(genome, i).is_empty())
        .collect();
    let (source, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let predecessors = safe_predecessors(genome, source);
    let (predecessor, slot, gate_bias) = predecessors[rng.gen_range(0..predecessors.len())];
    let new_id = next_node_id(genome);
    let backend = swap_backend.then(|| match genome.nodes[source].backend_def() {
        BackendDef::Vm(_) => birth::blank_graph_backend(),
        BackendDef::Graph(_) => birth::minimal_vm_backend(),
    });
    genome.copy_node_attached(
        source,
        new_id,
        backend,
        Attachment {
            from: predecessor,
            slot,
            gate_bias,
        },
    );
    Ok(reachability)
}

pub(super) fn apply_swap_node_backend(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    copy_attached(genome, targets, rng, true)
}

pub(super) fn apply_copy_node(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    copy_attached(genome, targets, rng, false)
}

/// Maximum number of nodes in a mesh slice for copy operators.
const MESH_SLICE_MAX_SIZE: usize = 8;

/// Clone a set of nodes identified by `gene_indices`, remapping internal
/// target references to fresh NodeIds. Appends cloned nodes to the genome.
/// Attachment to a pre-existing node is required before any copy is appended.
fn clone_and_remap_slice(
    genome: &mut CreatureGenome,
    gene_indices: &[usize],
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let attachable: Vec<_> = genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| lowest_unused_slot(&n.targets).map(|slot| (i, slot)))
        .collect();
    if attachable.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (source_idx, slot) = attachable[rng.gen_range(0..attachable.len())];
    let mut used: HashSet<_> = genome.nodes.iter().map(|n| n.node_id).collect();
    // `new_ids` records the fresh ids in `gene_indices` order (ascending,
    // since the slice functions return sorted indices) so the backlink target
    // the seeded draw picks below is a function of genome content alone
    // rather than of a map's hash order (T10.F11).
    let mut new_ids: Vec<NodeId> = Vec::with_capacity(gene_indices.len());
    let mut next_id = next_node_id(genome);
    for _ in gene_indices {
        new_ids.push(next_id);
        used.insert(next_id);
        next_id = unused_id(&used, next_id.0.wrapping_add(1));
    }
    // Attach through the preflighted free slot. The link draw reads only the
    // slice length, so it is taken before the clone, keeping the draw order.
    let link = rng.gen_range(0..new_ids.len());
    genome.clone_slice(gene_indices, &new_ids, source_idx, slot, link);
    Ok(())
}

pub(super) fn apply_copy_mesh_backward_slice(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (anchor_idx, reachability) = targets
        .select(&all_indices, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let gene = mesh_backward_slice(genome, anchor_idx, MESH_SLICE_MAX_SIZE)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    clone_and_remap_slice(genome, &gene.indices, rng)?;
    Ok(reachability)
}

pub(super) fn apply_copy_mesh_forward_slice(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    if genome.nodes.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (seed_idx, reachability) = targets
        .select(&all_indices, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let gene = mesh_forward_slice(genome, seed_idx, MESH_SLICE_MAX_SIZE)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    clone_and_remap_slice(genome, &gene.indices, rng)?;
    Ok(reachability)
}

pub(super) fn apply_splice_node(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let valid_targets = |idx: usize| -> Vec<usize> {
        genome.nodes[idx]
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| genome.nodes.iter().any(|n| n.node_id == t.target_id))
            .map(|(i, _)| i)
            .collect()
    };
    let eligible: Vec<_> = (0..genome.nodes.len())
        .filter(|&i| !valid_targets(i).is_empty())
        .collect();
    let (idx, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let targets = valid_targets(idx);
    let position = targets[rng.gen_range(0..targets.len())];
    let successor = genome.nodes[idx].targets[position].target_id;
    let new_id = next_node_id(genome);
    genome.append_node(birth::detour(new_id, successor, rng));
    genome.nodes[idx].targets[position].target_id = new_id;
    Ok(reachability)
}
