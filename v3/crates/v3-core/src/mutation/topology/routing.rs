use rand::Rng;

use crate::contracts::{RouteTarget, MAX_GATE_SLOTS};
use crate::creature::genome::CreatureGenome;
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

const _: () = assert!(MAX_GATE_SLOTS <= 8, "lowest_unused_slot uses u8 bitmask");

/// Returns the lowest slot index not already occupied by any target in `targets`.
/// Returns `None` when all `MAX_GATE_SLOTS` slots are in use.
fn lowest_unused_slot(targets: &[RouteTarget]) -> Option<u8> {
    let used: u8 = targets
        .iter()
        .filter(|t| (t.slot as usize) < MAX_GATE_SLOTS)
        .fold(0u8, |mask, t| mask | (1 << t.slot));
    (0..MAX_GATE_SLOTS as u8).find(|&s| used & (1 << s) == 0)
}

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
    let slot = lowest_unused_slot(&genome.nodes[node_idx].targets)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let target_id = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
    genome.nodes[node_idx].targets.push(RouteTarget {
        target_id,
        slot,
        gate_bias: -1.0,
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
    // Swap only target_id, preserving slot and gate_bias at each position.
    // This maintains positional slot stability per the design spec.
    let a_target = genome.nodes[node_idx].targets[a].target_id;
    let b_target = genome.nodes[node_idx].targets[b].target_id;
    genome.nodes[node_idx].targets[a].target_id = b_target;
    genome.nodes[node_idx].targets[b].target_id = a_target;
    Ok(reachability)
}

pub(super) fn apply_mutate_gate_bias(
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
    let target = &mut genome.nodes[node_idx].targets[target_slot];
    target.gate_bias = (target.gate_bias + rng.gen_range(-0.5f32..0.5)).clamp(-4.0, 4.0);
    Ok(reachability)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::founder::v3alpha1_founder_genome;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    // --- lowest_unused_slot tests ---

    #[test]
    fn lowest_unused_slot_finds_gap() {
        let targets = vec![
            RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 0.0,
            },
            RouteTarget {
                target_id: NodeId::new(2),
                slot: 2,
                gate_bias: 0.0,
            },
        ];
        assert_eq!(lowest_unused_slot(&targets), Some(1));
    }

    #[test]
    fn lowest_unused_slot_empty_returns_zero() {
        assert_eq!(lowest_unused_slot(&[]), Some(0));
    }

    #[test]
    fn lowest_unused_slot_all_used() {
        let targets: Vec<RouteTarget> = (0..MAX_GATE_SLOTS as u8)
            .map(|s| RouteTarget {
                target_id: NodeId::new(s as u32),
                slot: s,
                gate_bias: 0.0,
            })
            .collect();
        assert_eq!(lowest_unused_slot(&targets), None);
    }

    #[test]
    fn lowest_unused_slot_first_slot_taken() {
        let targets = vec![RouteTarget {
            target_id: NodeId::new(1),
            slot: 0,
            gate_bias: 0.0,
        }];
        assert_eq!(lowest_unused_slot(&targets), Some(1));
    }

    // --- apply_add_route_target tests ---

    #[test]
    fn add_route_target_uses_lowest_unused_slot_and_negative_bias() {
        // Founder node 0 has exactly one target at slot 0; new target should get
        // slot 1 (lowest unused) and gate_bias == -1.0.
        let mut genome = v3alpha1_founder_genome();
        assert_eq!(genome.nodes[0].targets.len(), 1);
        assert_eq!(
            genome.nodes[0].targets[0].slot, 0,
            "precondition: existing target at slot 0"
        );

        let reachable = vec![0];
        let mut r = rng(42);
        let result = apply_add_route_target(&mut genome, &reachable, 1.0, &mut r);
        assert!(result.is_ok(), "expected Ok, got {:?}", result);

        // Node 0 must now have two targets.
        assert_eq!(genome.nodes[0].targets.len(), 2);

        // The new target must have slot 1 and gate_bias -1.0.
        let new_target = genome.nodes[0]
            .targets
            .iter()
            .find(|t| t.slot != 0)
            .expect("new target with slot != 0 should exist");
        assert_eq!(
            new_target.slot, 1,
            "new target should occupy slot 1 (lowest unused)"
        );
        assert_eq!(
            new_target.gate_bias, -1.0,
            "new speculative target must have gate_bias -1.0"
        );
    }

    #[test]
    fn add_route_target_noop_when_full() {
        // Fill node 0 with MAX_GATE_SLOTS targets so there is no free slot.
        let mut genome = v3alpha1_founder_genome();
        genome.nodes[0].targets.clear();
        for s in 0..MAX_GATE_SLOTS as u8 {
            genome.nodes[0].targets.push(RouteTarget {
                target_id: NodeId::new(s as u32),
                slot: s,
                gate_bias: 0.0,
            });
        }

        // Force selection of node 0 (which is definitely full).
        let mut r = rng(7);
        let result = apply_add_route_target(&mut genome, &[0], 1.0, &mut r);
        assert_eq!(
            result,
            Err(MutationSkipReason::NoApplicableTarget),
            "fully-occupied node must return NoApplicableTarget"
        );
        assert_eq!(genome.nodes[0].targets.len(), MAX_GATE_SLOTS);
    }

    // --- apply_remove_route_target test ---

    #[test]
    fn remove_route_target_removes_entire_entry() {
        // Give node 0 a second target so remove has something to pick.
        let mut genome = v3alpha1_founder_genome();
        genome.nodes[0].targets.push(RouteTarget {
            target_id: NodeId::new(1),
            slot: 1,
            gate_bias: -1.0,
        });
        assert_eq!(genome.nodes[0].targets.len(), 2);

        let reachable = vec![0];
        let mut r = rng(0);
        let result = apply_remove_route_target(&mut genome, &reachable, 1.0, &mut r);
        assert!(result.is_ok());
        assert_eq!(genome.nodes[0].targets.len(), 1);
    }

    // --- apply_retarget_node_target test ---

    #[test]
    fn retarget_updates_target_id_only() {
        // Override gate_bias on node 0's target to a non-default value so the
        // assertion is meaningful.
        let mut genome = v3alpha1_founder_genome();
        genome.nodes[0].targets[0].gate_bias = 1.5;
        let original_slot = genome.nodes[0].targets[0].slot;
        let original_bias = genome.nodes[0].targets[0].gate_bias;

        let reachable = vec![0];
        let mut r = rng(1);
        apply_retarget_node_target(&mut genome, &reachable, 1.0, &mut r).unwrap();

        // slot and gate_bias must be untouched; only target_id may change.
        assert_eq!(genome.nodes[0].targets[0].slot, original_slot);
        assert_eq!(genome.nodes[0].targets[0].gate_bias, original_bias);
    }

    // --- apply_swap_route_targets test ---

    #[test]
    fn swap_route_targets_swaps_only_target_ids() {
        // Give node 0 two targets with distinct IDs, slots, and biases, then verify
        // that only the target_ids are exchanged.
        let mut genome = v3alpha1_founder_genome();
        genome.nodes[0].targets = vec![
            RouteTarget {
                target_id: NodeId::new(10),
                slot: 0,
                gate_bias: 0.5,
            },
            RouteTarget {
                target_id: NodeId::new(20),
                slot: 1,
                gate_bias: -1.0,
            },
        ];

        let reachable = vec![0];
        let mut r = rng(99);
        apply_swap_route_targets(&mut genome, &reachable, 1.0, &mut r).unwrap();

        let targets = &genome.nodes[0].targets;
        // slot and gate_bias must remain at their original Vec positions.
        assert_eq!(targets[0].slot, 0);
        assert_eq!(targets[0].gate_bias, 0.5);
        assert_eq!(targets[1].slot, 1);
        assert_eq!(targets[1].gate_bias, -1.0);
        // The two target_ids must be the original pair.
        let ids: std::collections::HashSet<u32> = targets.iter().map(|t| t.target_id.0).collect();
        assert!(ids.contains(&10) && ids.contains(&20));
    }
}
