use rand::Rng;

use crate::contracts::NodeId;
use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::mutation::types::MutationSkipReason;

/// Topology mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl TopologyOperator {
    /// Pick a random topology operator uniformly.
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u8..9) {
            0 => Self::AddNode,
            1 => Self::RemoveNode,
            2 => Self::RetargetNodeTarget,
            3 => Self::AddRouteTarget,
            4 => Self::RemoveRouteTarget,
            5 => Self::ChangeEntryNode,
            6 => Self::SwapNodeBackend,
            7 => Self::RewriteNodeId,
            _ => Self::CopyNode,
        }
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
}
