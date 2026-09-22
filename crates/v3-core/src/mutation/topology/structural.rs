use std::collections::{HashMap, HashSet};

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::analysis::{mesh_backward_slice, mesh_forward_slice};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

use super::birth;
use super::routing::{lowest_unused_slot, static_incumbent, writes_gate};
use crate::contracts::MAX_GATE_SLOTS;

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
    let currently_reachable = crate::creature::genome::analysis::mesh_reachable_nodes(genome);
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
    let removed = genome.nodes[idx].node_id;
    let successor = bypass_successor(genome, idx);
    genome.nodes.remove(idx);
    for node in &mut genome.nodes {
        if let Some(successor) = successor {
            for target in &mut node.targets {
                if target.target_id == removed {
                    target.target_id = successor;
                }
            }
        } else {
            node.targets.retain(|target| target.target_id != removed);
        }
    }
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
                || writes_gate(&node.backend_def, incumbent.slot)
            {
                return None;
            }
            let slot = (0..MAX_GATE_SLOTS as u8).find(|&slot| {
                !node.targets.iter().any(|t| t.slot == slot)
                    && !writes_gate(&node.backend_def, slot)
            })?;
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
    alternate: Option<&MutationConfig>,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<_> = (0..genome.nodes.len())
        .filter(|&i| !safe_predecessors(genome, i).is_empty())
        .collect();
    let (source, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let predecessors = safe_predecessors(genome, source);
    let (predecessor, slot, gate_bias) = predecessors[rng.gen_range(0..predecessors.len())];
    let old_id = genome.nodes[source].node_id;
    let new_id = next_node_id(genome);
    let mut clone = genome.nodes[source].clone();
    clone.node_id = new_id;
    for target in &mut clone.targets {
        if target.target_id == old_id {
            target.target_id = new_id;
        }
    }
    if let Some(config) = alternate {
        clone.backend_def = match clone.backend_def {
            BackendDef::Vm(_) => birth::blank_graph_backend(config),
            BackendDef::Graph(_) => birth::minimal_vm_backend(),
        };
    }
    genome.nodes[predecessor].targets.push(RouteTarget {
        target_id: new_id,
        slot,
        gate_bias,
    });
    genome.nodes.push(clone);
    Ok(reachability)
}

pub(super) fn apply_swap_node_backend(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    copy_attached(genome, targets, rng, Some(config))
}

pub(super) fn apply_copy_node(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    copy_attached(genome, targets, rng, None)
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
    // Build old_id -> new_id mapping. `new_ids` records the fresh ids in
    // `gene_indices` order (ascending, since the slice functions return sorted
    // indices) so the backlink target the seeded draw picks below is a function
    // of genome content alone rather than of the map's hash order (T10.F11).
    let mut id_map = HashMap::with_capacity(gene_indices.len());
    let mut new_ids: Vec<NodeId> = Vec::with_capacity(gene_indices.len());
    let mut next_id = next_node_id(genome);
    for &idx in gene_indices {
        let old_id = genome.nodes[idx].node_id;
        id_map.insert(old_id, next_id);
        new_ids.push(next_id);
        used.insert(next_id);
        next_id = unused_id(&used, next_id.0.wrapping_add(1));
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
                        if let Some(&new_id) = id_map.get(&t.target_id) {
                            RouteTarget {
                                target_id: new_id,
                                slot: t.slot,
                                gate_bias: t.gate_bias,
                            }
                        } else {
                            *t // external target preserved
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    genome.nodes.extend(cloned);

    // Note: previously remapped CustomOutput slots in cloned Graph backends
    // to avoid clobbering. CGP Graph backends have fixed output sinks (not
    // remappable), and VM backends don't have CustomOutput node kinds, so
    // this remapping step is now a no-op and has been removed.

    // Attach through the preflighted free slot, retaining ordered target draws.
    let link_target = new_ids[rng.gen_range(0..new_ids.len())];
    genome.nodes[source_idx].targets.push(RouteTarget {
        target_id: link_target,
        slot,
        gate_bias: 0.0,
    });
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
    config: &MutationConfig,
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
    genome
        .nodes
        .push(birth::detour(new_id, successor, config, rng));
    genome.nodes[idx].targets[position].target_id = new_id;
    Ok(reachability)
}
