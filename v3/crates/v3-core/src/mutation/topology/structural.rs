use std::collections::HashMap;

use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::analysis::{mesh_backward_slice, mesh_forward_slice};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

use super::birth;

/// Allocate the next node ID: `max(existing node_ids) + 1` with wrapping u32 arithmetic.
pub(super) fn next_node_id(genome: &CreatureGenome) -> NodeId {
    let max_id = genome.nodes.iter().map(|n| n.node_id.0).max().unwrap_or(0);
    NodeId::new(max_id.wrapping_add(1))
}

pub(super) fn apply_add_node(
    genome: &mut CreatureGenome,
    config: &MutationConfig,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let new_id = next_node_id(genome);
    genome.nodes.push(birth::new_topology_birth_node(
        new_id,
        Vec::new(),
        config,
        rng,
    ));
    Ok(())
}

pub(super) fn apply_remove_node(
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

pub(super) fn apply_swap_node_backend(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let node = &mut genome.nodes[idx];
    node.backend_def = match &node.backend_def {
        BackendDef::Vm(_) => birth::blank_graph_backend(config),
        BackendDef::Graph(_) => birth::minimal_vm_backend(),
    };
    Ok(reachability)
}

pub(super) fn apply_rewrite_node_id(
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
            if target.target_id == old_id {
                target.target_id = new_id;
            }
        }
    }

    Ok(reachability)
}

pub(super) fn apply_copy_node(
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
    let backlink_slot = genome.nodes[source_idx].targets.len() as u8;
    genome.nodes[source_idx].targets.push(RouteTarget {
        target_id: new_id,
        slot: backlink_slot,
        gate_bias: 0.0,
    });
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
    let backlink_slot = genome.nodes[source_idx].targets.len() as u8;
    genome.nodes[source_idx].targets.push(RouteTarget {
        target_id: link_target,
        slot: backlink_slot,
        gate_bias: 0.0,
    });
}

pub(super) fn apply_copy_mesh_backward_slice(
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

pub(super) fn apply_copy_mesh_forward_slice(
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

pub(super) fn apply_splice_node(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
    config: &MutationConfig,
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
    let b_id = genome.nodes[a_idx].targets[target_slot].target_id;
    let c_id = next_node_id(genome);
    genome.nodes.push(birth::new_topology_birth_node(
        c_id,
        vec![RouteTarget {
            target_id: b_id,
            slot: 0,
            gate_bias: 0.0,
        }],
        config,
        rng,
    ));
    genome.nodes[a_idx].targets[target_slot].target_id = c_id;
    Ok(reachability)
}
