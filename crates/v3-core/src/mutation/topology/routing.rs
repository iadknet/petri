use rand::Rng;

use super::{birth, structural::next_node_id};
use crate::config::MutationConfig;
use crate::contracts::{NodeId, RouteTarget, MAX_GATE_SLOTS};
use crate::creature::genome::cgp::{GraphEdge, OutputSinkKind};
use crate::creature::genome::CreatureGenome;
use crate::creature::genome::{BackendDef, VmInstruction};
use crate::mutation::graph::operators::random_graph_source;
use crate::mutation::reachability::TargetSelector;
use crate::mutation::types::{MutationSkipReason, TargetReachability};
use crate::mutation::vm::operators::insert_new_instruction_with_reference_repair;
use crate::runtime::routing::{resolve_gated_route, RouteGateMap};

const _: () = assert!(MAX_GATE_SLOTS <= 8, "lowest_unused_slot uses u8 bitmask");

/// Returns the lowest slot index not already occupied by any target in `targets`.
/// Returns `None` when all `MAX_GATE_SLOTS` slots are in use.
pub(super) fn lowest_unused_slot(targets: &[RouteTarget]) -> Option<u8> {
    let used: u8 = targets
        .iter()
        .filter(|t| (t.slot as usize) < MAX_GATE_SLOTS)
        .fold(0u8, |mask, t| mask | (1 << t.slot));
    (0..MAX_GATE_SLOTS as u8).find(|&s| used & (1 << s) == 0)
}

pub(super) fn static_incumbent(targets: &[RouteTarget]) -> Option<usize> {
    resolve_gated_route(targets, &RouteGateMap::default()).map(|(index, _)| index)
}

pub(super) fn writes_gate(backend: &BackendDef, slot: u8) -> bool {
    match backend {
        BackendDef::Vm(vm) => vm.program.iter().any(|instruction| matches!(instruction, VmInstruction::WriteRouteGate { slot: written, .. } if *written == slot)),
        BackendDef::Graph(graph) => graph.output_sinks.iter().any(|sink| matches!(sink.kind, OutputSinkKind::RouterGate(written) if written == slot) && !sink.inputs.is_empty()),
    }
}

fn local_destinations(genome: &CreatureGenome, node_idx: usize, target_idx: usize) -> Vec<NodeId> {
    let node = &genome.nodes[node_idx];
    let current = node.targets[target_idx].target_id;
    let mut candidates: Vec<_> = genome
        .nodes
        .iter()
        .find(|n| n.node_id == current)
        .into_iter()
        .flat_map(|n| n.targets.iter())
        .chain(node.targets.iter())
        .map(|t| t.target_id)
        // The node itself is a legal destination (T19.F02); only the current
        // target and missing ids are excluded.
        .filter(|id| *id != current && genome.nodes.iter().any(|n| n.node_id == *id))
        .collect();
    candidates.sort_unstable();
    candidates.dedup();
    candidates
}

pub(super) fn apply_retarget_node_target(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<_> = (0..genome.nodes.len())
        .filter(|&i| {
            (0..genome.nodes[i].targets.len()).any(|j| !local_destinations(genome, i, j).is_empty())
        })
        .collect();
    let (idx, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let targets: Vec<_> = (0..genome.nodes[idx].targets.len())
        .filter(|&j| !local_destinations(genome, idx, j).is_empty())
        .collect();
    let target = targets[rng.gen_range(0..targets.len())];
    let candidates = local_destinations(genome, idx, target);
    genome.nodes[idx].targets[target].target_id = candidates[rng.gen_range(0..candidates.len())];
    Ok(reachability)
}

pub(super) fn apply_add_route_target(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<_> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            n.targets.len() == 1
                && genome
                    .nodes
                    .iter()
                    .any(|other| other.node_id == n.targets[0].target_id)
                && lowest_unused_slot(&n.targets).is_some()
        })
        .map(|(i, _)| i)
        .collect();
    let (idx, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let slot = lowest_unused_slot(&genome.nodes[idx].targets)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let mut backend = genome.nodes[idx].backend_def.clone();
    match &mut backend {
        BackendDef::Vm(vm) => {
            if vm.register_count == 0 {
                return Err(MutationSkipReason::NoApplicableTarget);
            }
            let at = vm
                .program
                .iter()
                .position(|i| matches!(i, VmInstruction::Halt))
                .unwrap_or(vm.program.len());
            let instruction = VmInstruction::WriteRouteGate {
                slot,
                src: rng.gen_range(0..vm.register_count),
            };
            insert_new_instruction_with_reference_repair(&mut vm.program, at, instruction)?;
        }
        BackendDef::Graph(graph) => {
            let sink = graph
                .output_sinks
                .iter()
                .position(|s| matches!(s.kind, OutputSinkKind::RouterGate(s) if s == slot))
                .ok_or(MutationSkipReason::NoApplicableTarget)?;
            let source = random_graph_source(
                graph.compute_nodes.len() as u16,
                &genome.nodes[idx].input_refs,
                config,
                rng,
            );
            graph.output_sinks[sink].inputs.push(GraphEdge {
                source,
                weight: 1.0,
            });
        }
    }
    let old = genome.nodes[idx].targets[0];
    let new_id = next_node_id(genome);
    genome.nodes[idx].backend_def = backend;
    genome.nodes[idx].targets.push(RouteTarget {
        target_id: new_id,
        slot,
        gate_bias: old.gate_bias,
    });
    genome.nodes.push(birth::detour(new_id, old.target_id, rng));
    Ok(reachability)
}

pub(super) fn apply_remove_route_target(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<_> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.targets.len() >= 2)
        .map(|(i, _)| i)
        .collect();
    let (idx, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let incumbent = static_incumbent(&genome.nodes[idx].targets).expect("nonempty targets");
    let mut target = rng.gen_range(0..genome.nodes[idx].targets.len() - 1);
    if target >= incumbent {
        target += 1;
    }
    genome.nodes[idx].targets.remove(target);
    Ok(reachability)
}

pub(super) fn apply_swap_route_targets(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
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
    let (node_idx, reachability) = targets
        .select(&eligible, rng)
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
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.targets.is_empty())
        .map(|(i, _)| i)
        .collect();
    let (node_idx, reachability) = targets
        .select(&eligible, rng)
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

    /// Counts the RNG calls an operator spends, so the T19.F02 self-target
    /// relaxation can be shown not to change the draw structure.
    struct CountingRng {
        inner: SmallRng,
        calls: usize,
    }
    impl rand::RngCore for CountingRng {
        fn next_u32(&mut self) -> u32 {
            self.calls += 1;
            self.inner.next_u32()
        }
        fn next_u64(&mut self) -> u64 {
            self.calls += 1;
            self.inner.next_u64()
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.calls += 1;
            self.inner.fill_bytes(dest);
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }
    fn counting(seed: u64) -> CountingRng {
        CountingRng {
            inner: rng(seed),
            calls: 0,
        }
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
    fn add_route_target_uses_lowest_unused_slot_and_tied_bias() {
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
        let result = apply_add_route_target(
            &mut genome,
            &mut TargetSelector::reachable_only(&reachable, 1.0),
            &mut r,
            &MutationConfig::default(),
        );
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
        assert_eq!(new_target.gate_bias, 0.0, "new branch ties the original");
    }

    #[test]
    fn add_route_target_noop_when_all_nodes_full() {
        // Fill ALL nodes with MAX_GATE_SLOTS targets so no node has a free slot.
        let mut genome = v3alpha1_founder_genome();
        for node in &mut genome.nodes {
            node.targets.clear();
            for s in 0..MAX_GATE_SLOTS as u8 {
                node.targets.push(RouteTarget {
                    target_id: NodeId::new(s as u32),
                    slot: s,
                    gate_bias: 0.0,
                });
            }
        }

        let mut r = rng(7);
        let result = apply_add_route_target(
            &mut genome,
            &mut TargetSelector::reachable_only(&[0, 1], 1.0),
            &mut r,
            &MutationConfig::default(),
        );
        assert_eq!(
            result,
            Err(MutationSkipReason::NoApplicableTarget),
            "all nodes fully-occupied must return NoApplicableTarget"
        );
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
        let result = apply_remove_route_target(
            &mut genome,
            &mut TargetSelector::reachable_only(&reachable, 1.0),
            &mut r,
        );
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

        let mut third = genome.nodes[1].clone();
        third.node_id = NodeId::new(2);
        genome.nodes.push(third);
        genome.nodes[1].targets.push(RouteTarget {
            target_id: NodeId::new(2),
            slot: 0,
            gate_bias: 0.0,
        });
        let reachable = vec![0];
        let mut r = rng(1);
        apply_retarget_node_target(
            &mut genome,
            &mut TargetSelector::reachable_only(&reachable, 1.0),
            &mut r,
        )
        .unwrap();

        // slot and gate_bias must be untouched; only target_id may change.
        assert_eq!(genome.nodes[0].targets[0].slot, original_slot);
        assert_eq!(genome.nodes[0].targets[0].gate_bias, original_bias);
    }

    /// T19.F02: a node may be retargeted onto itself when the current
    /// destination routes back to it. The two-node cycle `0 -> 1 -> 0` leaves
    /// node 0 exactly one candidate (itself), so the draw is forced whatever
    /// the seed; the RNG spends the same three draws as any retarget.
    #[test]
    fn retarget_draws_a_self_target_from_the_neighborhood() {
        let mut genome = v3alpha1_founder_genome();
        genome.nodes[1].targets = vec![RouteTarget {
            target_id: NodeId::new(0),
            slot: 0,
            gate_bias: 0.0,
        }];
        let reachable = vec![0];
        for seed in 0..8 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            apply_retarget_node_target(
                &mut g,
                &mut TargetSelector::reachable_only(&reachable, 1.0),
                &mut r,
            )
            .unwrap();
            assert_eq!(
                g.nodes[0].targets[0].target_id,
                NodeId::new(0),
                "seed {seed}"
            );
            assert_eq!(g.nodes[0].targets[0].slot, genome.nodes[0].targets[0].slot);
        }
    }

    /// T19.F02: the draw structure is unchanged by the self-target
    /// relaxation. A retarget whose only candidate is the node itself spends
    /// exactly the RNG calls of a retarget whose only candidate is another
    /// node, and an `AddRouteTarget` on a self-looping node spends exactly
    /// the calls of one on a forward-routing node, seed for seed.
    #[test]
    fn self_target_draws_spend_the_same_rng_calls_as_ordinary_draws() {
        let reachable = vec![0];
        let mut self_retarget = v3alpha1_founder_genome();
        self_retarget.nodes[1].targets = vec![RouteTarget {
            target_id: NodeId::new(0),
            slot: 0,
            gate_bias: 0.0,
        }];
        let mut forward_retarget = v3alpha1_founder_genome();
        let mut third = forward_retarget.nodes[1].clone();
        third.node_id = NodeId::new(2);
        third.targets = vec![];
        forward_retarget.nodes.push(third);
        forward_retarget.nodes[1].targets = vec![RouteTarget {
            target_id: NodeId::new(2),
            slot: 0,
            gate_bias: 0.0,
        }];
        let mut self_add = v3alpha1_founder_genome();
        self_add.nodes[0].targets[0].target_id = NodeId::new(0);
        let forward_add = v3alpha1_founder_genome();
        for seed in 0..8 {
            let mut calls = [0usize; 4];
            for (slot, genome) in [&self_retarget, &forward_retarget].into_iter().enumerate() {
                let mut g = genome.clone();
                let mut r = counting(seed);
                apply_retarget_node_target(
                    &mut g,
                    &mut TargetSelector::reachable_only(&reachable, 1.0),
                    &mut r,
                )
                .unwrap();
                calls[slot] = r.calls;
            }
            for (slot, genome) in [&self_add, &forward_add].into_iter().enumerate() {
                let mut g = genome.clone();
                let mut r = counting(seed);
                apply_add_route_target(
                    &mut g,
                    &mut TargetSelector::reachable_only(&reachable, 1.0),
                    &mut r,
                    &MutationConfig::default(),
                )
                .unwrap();
                calls[2 + slot] = r.calls;
            }
            assert_eq!(calls[0], calls[1], "retarget, seed {seed}");
            assert_eq!(calls[2], calls[3], "add route target, seed {seed}");
        }
    }

    /// T19.F02: a sole self-target is a valid single successor for
    /// `AddRouteTarget`; the detour forwards to the old successor, the node
    /// itself.
    #[test]
    fn add_route_target_accepts_a_self_looping_node_and_forwards_the_detour_to_it() {
        let mut genome = v3alpha1_founder_genome();
        genome.nodes[0].targets[0].target_id = NodeId::new(0);
        let reachable = vec![0];
        let mut r = rng(3);
        apply_add_route_target(
            &mut genome,
            &mut TargetSelector::reachable_only(&reachable, 1.0),
            &mut r,
            &MutationConfig::default(),
        )
        .unwrap();
        assert_eq!(genome.nodes[0].targets.len(), 2);
        assert_eq!(genome.nodes[0].targets[0].target_id, NodeId::new(0));
        let detour_id = genome.nodes[0].targets[1].target_id;
        let detour = genome
            .nodes
            .iter()
            .find(|n| n.node_id == detour_id)
            .unwrap();
        assert_eq!(detour.targets.len(), 1);
        assert_eq!(detour.targets[0].target_id, NodeId::new(0));
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
        apply_swap_route_targets(
            &mut genome,
            &mut TargetSelector::reachable_only(&reachable, 1.0),
            &mut r,
        )
        .unwrap();

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
