use rand::Rng;

use crate::creature::genome::{BackendDef, CreatureGenome, GraphInternalNode, GraphNodeKind};
use crate::mutation::types::MutationSkipReason;

/// Graph mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphOperator {
    AlterGraphEdgeWeight,
    SwapGraphOperator,
    MutateGraphOperatorParam,
    AddInternalGraphNode,
    RemoveInternalGraphNode,
}

impl GraphOperator {
    /// Pick a random graph operator uniformly.
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u8..5) {
            0 => Self::AlterGraphEdgeWeight,
            1 => Self::SwapGraphOperator,
            2 => Self::MutateGraphOperatorParam,
            3 => Self::AddInternalGraphNode,
            _ => Self::RemoveInternalGraphNode,
        }
    }
}

/// Graph domain mutator.
pub struct GraphMutator;

impl GraphMutator {
    /// Apply a graph operator to the genome.
    ///
    /// Returns `Ok(())` on success, or `Err(MutationSkipReason::NoApplicableTarget)` if the
    /// genome has no Graph-backend nodes or no applicable internal target.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        rng: &mut impl Rng,
    ) -> Result<(), MutationSkipReason> {
        // Pre-guard: must have at least one Graph-backend node.
        let graph_indices: Vec<usize> = genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.backend_def, BackendDef::Graph(_)))
            .map(|(i, _)| i)
            .collect();
        if graph_indices.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let node_idx = graph_indices[rng.gen_range(0..graph_indices.len())];
        match op {
            GraphOperator::AlterGraphEdgeWeight => alter_edge_weight(genome, node_idx, rng),
            GraphOperator::SwapGraphOperator => swap_operator(genome, node_idx, rng),
            GraphOperator::MutateGraphOperatorParam => mutate_operator_param(genome, node_idx, rng),
            GraphOperator::AddInternalGraphNode => add_internal_node(genome, node_idx, rng),
            GraphOperator::RemoveInternalGraphNode => remove_internal_node(genome, node_idx, rng),
        }
    }
}

fn alter_edge_weight(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        // Find internal nodes with non-empty inputs.
        let eligible: Vec<usize> = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.inputs.is_empty())
            .map(|(i, _)| i)
            .collect();
        if eligible.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let int_idx = eligible[rng.gen_range(0..eligible.len())];
        let edge_idx = rng.gen_range(0..g.internal_nodes[int_idx].inputs.len());
        g.internal_nodes[int_idx].inputs[edge_idx].weight += rng.gen_range(-0.5f32..=0.5);
    }
    Ok(())
}

fn swap_operator(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let int_idx = rng.gen_range(0..g.internal_nodes.len());
        g.internal_nodes[int_idx].kind = random_non_parameterized_kind(rng);
    }
    Ok(())
}

fn mutate_operator_param(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        // Find internal nodes with parameterized kinds.
        let eligible: Vec<usize> = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| is_parameterized(&n.kind))
            .map(|(i, _)| i)
            .collect();
        if eligible.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let int_idx = eligible[rng.gen_range(0..eligible.len())];
        let delta = rng.gen_range(-0.1f32..=0.1);
        match &mut g.internal_nodes[int_idx].kind {
            GraphNodeKind::Threshold(ref mut p)
            | GraphNodeKind::DecayIntegrator(ref mut p)
            | GraphNodeKind::Momentum(ref mut p)
            | GraphNodeKind::Oscillator(ref mut p) => *p += delta,
            _ => {}
        }
    }
    Ok(())
}

fn add_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        g.internal_nodes.push(GraphInternalNode {
            kind: GraphNodeKind::Constant(rng.gen_range(-1.0f32..=1.0)),
            inputs: vec![],
        });
    }
    Ok(())
}

fn remove_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let int_idx = rng.gen_range(0..g.internal_nodes.len());
        g.internal_nodes.remove(int_idx);
    }
    Ok(())
}

/// Return a random non-parameterized GraphNodeKind (safe to swap without context).
fn random_non_parameterized_kind(rng: &mut impl Rng) -> GraphNodeKind {
    match rng.gen_range(0u8..15) {
        0 => GraphNodeKind::Add,
        1 => GraphNodeKind::Multiply,
        2 => GraphNodeKind::Negate,
        3 => GraphNodeKind::Abs,
        4 => GraphNodeKind::Min,
        5 => GraphNodeKind::Max,
        6 => GraphNodeKind::Relu,
        7 => GraphNodeKind::Sigmoid,
        8 => GraphNodeKind::Tanh,
        9 => GraphNodeKind::GreaterThan,
        10 => GraphNodeKind::Select,
        11 => GraphNodeKind::Clamp01,
        12 => GraphNodeKind::WeightedSum,
        13 => GraphNodeKind::AdaptiveGain,
        _ => GraphNodeKind::RouterOutput,
    }
}

/// Returns true if the kind has a mutable parameter.
fn is_parameterized(kind: &GraphNodeKind) -> bool {
    matches!(
        kind,
        GraphNodeKind::Threshold(_)
            | GraphNodeKind::DecayIntegrator(_)
            | GraphNodeKind::Momentum(_)
            | GraphNodeKind::Oscillator(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};
    use crate::creature::parseability::ParseabilityGate;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    /// Founder genome has a Graph node (node 0) with internal nodes including `Threshold(24.0)`.
    fn graph_node_internal_count(genome: &CreatureGenome) -> usize {
        genome
            .nodes
            .iter()
            .filter_map(|n| {
                if let BackendDef::Graph(ref g) = n.backend_def {
                    Some(g.internal_nodes.len())
                } else {
                    None
                }
            })
            .sum()
    }

    #[test]
    fn alter_graph_edge_weight_changes_weight() {
        let genome = v3alpha1_founder_genome();
        // Founder node 0 graph has internal nodes with inputs (e.g., idx 2 Threshold has inputs).
        let original_weight = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            g.internal_nodes[2].inputs[0].weight
        } else {
            panic!()
        };
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            let result = GraphMutator::apply(&mut g, GraphOperator::AlterGraphEdgeWeight, &mut r);
            if result.is_ok() {
                let new_weight = if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    gd.internal_nodes[2].inputs[0].weight
                } else {
                    panic!()
                };
                if (new_weight - original_weight).abs() > 1e-7 {
                    changed = true;
                    break;
                }
            }
        }
        assert!(changed, "edge weight must change");
    }

    #[test]
    fn swap_graph_operator_changes_node_kind() {
        let genome = v3alpha1_founder_genome();
        let mut swapped = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::SwapGraphOperator, &mut r).is_ok() {
                swapped = true;
                break;
            }
        }
        assert!(swapped, "swap must succeed on founder genome");
    }

    #[test]
    fn mutate_graph_operator_param_changes_param() {
        let genome = v3alpha1_founder_genome();
        // Node 0 graph has Threshold(24.0) at internal index 2.
        let original_param = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            if let GraphNodeKind::Threshold(p) = g.internal_nodes[2].kind {
                p
            } else {
                panic!("expected Threshold")
            }
        } else {
            panic!()
        };
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok()
            {
                let new_param = if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    if let GraphNodeKind::Threshold(p) = gd.internal_nodes[2].kind {
                        p
                    } else {
                        original_param
                    }
                } else {
                    original_param
                };
                if (new_param - original_param).abs() > 1e-7 {
                    changed = true;
                    break;
                }
            }
        }
        assert!(changed, "parameter must change after mutation");
    }

    #[test]
    fn add_internal_graph_node_increases_internal_node_count() {
        let mut genome = v3alpha1_founder_genome();
        let before = graph_node_internal_count(&genome);
        let mut r = rng(0);
        GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
        let after = graph_node_internal_count(&genome);
        assert_eq!(after, before + 1);
    }

    #[test]
    fn remove_internal_graph_node_decreases_count() {
        let mut genome = v3alpha1_founder_genome();
        let before = graph_node_internal_count(&genome);
        assert!(before > 0, "founder graph node must have internal nodes");
        let mut r = rng(0);
        GraphMutator::apply(&mut genome, GraphOperator::RemoveInternalGraphNode, &mut r).unwrap();
        let after = graph_node_internal_count(&genome);
        assert_eq!(after, before - 1);
    }

    #[test]
    fn graph_mutator_on_vm_only_genome_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        // Replace all nodes with VM-only nodes.
        genome.nodes = vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }];
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::AlterGraphEdgeWeight, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn graph_after_mutation_passes_parseability_gate() {
        let operators = [
            GraphOperator::AlterGraphEdgeWeight,
            GraphOperator::SwapGraphOperator,
            GraphOperator::MutateGraphOperatorParam,
            GraphOperator::AddInternalGraphNode,
            GraphOperator::RemoveInternalGraphNode,
        ];
        for (i, &op) in operators.iter().enumerate() {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(i as u64 + 300);
            let _ = GraphMutator::apply(&mut genome, op, &mut r);
            assert!(
                ParseabilityGate::validate(&genome).is_ok(),
                "parseability failed after {:?}",
                op
            );
        }
    }
}
