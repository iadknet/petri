use rand::Rng;

use crate::contracts::RouteTarget;
use crate::creature::genome::CreatureGenome;
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

pub(super) fn apply_retarget_node_target(
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
    genome.nodes[node_idx].targets[target_slot].target_id = new_target;
    Ok(reachability)
}

pub(super) fn apply_add_route_target(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (node_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_id = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    let idx = genome.nodes[node_idx].targets.len();
    genome.nodes[node_idx].targets.push(RouteTarget {
        target_id,
        slot: idx as u8,
        gate_bias: 0.0,
    });
    Ok(reachability)
}

pub(super) fn apply_remove_route_target(
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

pub(super) fn apply_swap_route_targets(
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
