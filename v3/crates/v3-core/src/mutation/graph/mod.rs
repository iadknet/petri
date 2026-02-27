use rand::Rng;

use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphInput, GraphInternalNode, GraphNodeKind,
};
use crate::mutation::types::MutationSkipReason;

/// Graph mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphOperator {
    AlterGraphEdgeWeight,
    SwapGraphOperator,
    MutateGraphOperatorParam,
    AddInternalGraphNode,
    RemoveInternalGraphNode,
    AddGraphEdge,
    RetargetGraphEdge,
    RemoveGraphEdge,
    GraphRawFieldMutation,
}

impl GraphOperator {
    /// Pick a random graph operator uniformly.
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u8..9) {
            0 => Self::AlterGraphEdgeWeight,
            1 => Self::SwapGraphOperator,
            2 => Self::MutateGraphOperatorParam,
            3 => Self::AddInternalGraphNode,
            4 => Self::RemoveInternalGraphNode,
            5 => Self::AddGraphEdge,
            6 => Self::RetargetGraphEdge,
            7 => Self::RemoveGraphEdge,
            _ => Self::GraphRawFieldMutation,
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
            GraphOperator::AddGraphEdge => add_graph_edge(genome, node_idx, rng),
            GraphOperator::RetargetGraphEdge => retarget_graph_edge(genome, node_idx, rng),
            GraphOperator::RemoveGraphEdge => remove_graph_edge(genome, node_idx, rng),
            GraphOperator::GraphRawFieldMutation => {
                apply_graph_raw_field_mutation(genome, node_idx, rng)
            }
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
        g.internal_nodes[int_idx].kind = random_graph_node_kind(rng);
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
            GraphNodeKind::Constant(ref mut p)
            | GraphNodeKind::Threshold(ref mut p)
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

fn add_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let int_idx = rng.gen_range(0..g.internal_nodes.len());
        let source_idx = rng.gen_range(0..g.internal_nodes.len() as u16);
        let weight = rng.gen_range(-1.0f32..=1.0);
        g.internal_nodes[int_idx]
            .inputs
            .push(GraphInput { source_idx, weight });
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

fn retarget_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
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
        let new_source = rng.gen_range(0..g.internal_nodes.len() as u16);
        g.internal_nodes[int_idx].inputs[edge_idx].source_idx = new_source;
    }
    Ok(())
}

fn remove_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
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
        g.internal_nodes[int_idx].inputs.remove(edge_idx);
    }
    Ok(())
}

fn apply_graph_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let mut target_count: usize = 0;
        for internal in &g.internal_nodes {
            if matches!(
                &internal.kind,
                GraphNodeKind::InputRef(_) | GraphNodeKind::CustomOutput(_)
            ) {
                target_count += 1;
            }
            target_count += internal.inputs.len();
        }
        if target_count == 0 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let mut pick = rng.gen_range(0..target_count);
        for int_idx in 0..g.internal_nodes.len() {
            if matches!(
                &g.internal_nodes[int_idx].kind,
                GraphNodeKind::InputRef(_) | GraphNodeKind::CustomOutput(_)
            ) {
                if pick == 0 {
                    if matches!(&g.internal_nodes[int_idx].kind, GraphNodeKind::InputRef(_)) {
                        g.internal_nodes[int_idx].kind = GraphNodeKind::InputRef(rng.gen());
                    } else {
                        g.internal_nodes[int_idx].kind = GraphNodeKind::CustomOutput(rng.gen());
                    }
                    return Ok(());
                }
                pick -= 1;
            }

            for edge_idx in 0..g.internal_nodes[int_idx].inputs.len() {
                if pick == 0 {
                    g.internal_nodes[int_idx].inputs[edge_idx].source_idx = rng.gen();
                    return Ok(());
                }
                pick -= 1;
            }
        }
    }
    Ok(())
}

/// Return a random GraphNodeKind covering all 22 variants with random initial params.
fn random_graph_node_kind(rng: &mut impl Rng) -> GraphNodeKind {
    match rng.gen_range(0u8..22) {
        0 => GraphNodeKind::InputRef(rng.gen()),
        1 => GraphNodeKind::Constant(rng.gen_range(-1.0f32..=1.0)),
        2 => GraphNodeKind::Add,
        3 => GraphNodeKind::Multiply,
        4 => GraphNodeKind::Negate,
        5 => GraphNodeKind::Abs,
        6 => GraphNodeKind::Min,
        7 => GraphNodeKind::Max,
        8 => GraphNodeKind::Threshold(rng.gen_range(-1.0f32..=1.0)),
        9 => GraphNodeKind::GreaterThan,
        10 => GraphNodeKind::Sigmoid,
        11 => GraphNodeKind::Tanh,
        12 => GraphNodeKind::Relu,
        13 => GraphNodeKind::Select,
        14 => GraphNodeKind::Clamp01,
        15 => GraphNodeKind::WeightedSum,
        16 => GraphNodeKind::DecayIntegrator(rng.gen_range(0.0f32..=1.0)),
        17 => GraphNodeKind::Momentum(rng.gen_range(0.0f32..=1.0)),
        18 => GraphNodeKind::Oscillator(rng.gen_range(0.01f32..=10.0)),
        19 => GraphNodeKind::AdaptiveGain,
        20 => GraphNodeKind::CustomOutput(rng.gen()),
        _ => GraphNodeKind::RouterOutput,
    }
}

/// Returns true if the kind has a mutable parameter.
fn is_parameterized(kind: &GraphNodeKind) -> bool {
    matches!(
        kind,
        GraphNodeKind::Constant(_)
            | GraphNodeKind::Threshold(_)
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
    use crate::creature::genome::{
        BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, NodeGenome,
        VmBackendDef, VmInstruction,
    };
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

    fn graph_only_genome(internal_nodes: Vec<GraphInternalNode>) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(GraphBackendDef { internal_nodes }),
                targets: vec![],
            }],
        }
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
    fn add_graph_edge_increases_input_count() {
        let mut genome = v3alpha1_founder_genome();
        // Count total inputs across all graph internal nodes before.
        let before: usize = genome
            .nodes
            .iter()
            .filter_map(|n| {
                if let BackendDef::Graph(ref g) = n.backend_def {
                    Some(
                        g.internal_nodes
                            .iter()
                            .map(|i| i.inputs.len())
                            .sum::<usize>(),
                    )
                } else {
                    None
                }
            })
            .sum();
        let mut r = rng(0);
        GraphMutator::apply(&mut genome, GraphOperator::AddGraphEdge, &mut r).unwrap();
        let after: usize = genome
            .nodes
            .iter()
            .filter_map(|n| {
                if let BackendDef::Graph(ref g) = n.backend_def {
                    Some(
                        g.internal_nodes
                            .iter()
                            .map(|i| i.inputs.len())
                            .sum::<usize>(),
                    )
                } else {
                    None
                }
            })
            .sum();
        assert_eq!(
            after,
            before + 1,
            "add_graph_edge must add exactly one input"
        );
    }

    #[test]
    fn add_graph_edge_on_empty_internals_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        // Clear all internal nodes from graph backends.
        for node in &mut genome.nodes {
            if let BackendDef::Graph(ref mut g) = node.backend_def {
                g.internal_nodes.clear();
            }
        }
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::AddGraphEdge, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn constant_is_parameterized() {
        assert!(
            is_parameterized(&GraphNodeKind::Constant(1.0)),
            "Constant must be parameterized"
        );
    }

    #[test]
    fn mutate_operator_param_changes_constant_value() {
        let mut genome = v3alpha1_founder_genome();
        // Add a Constant internal node to the graph node.
        if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
            g.internal_nodes.push(GraphInternalNode {
                kind: GraphNodeKind::Constant(0.5),
                inputs: vec![],
            });
        }
        let original = 0.5f32;
        let mut changed = false;
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok()
            {
                if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    // Check last internal node (the Constant we added).
                    let last = gd.internal_nodes.last().unwrap();
                    if let GraphNodeKind::Constant(p) = last.kind {
                        if (p - original).abs() > 1e-7 {
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
        assert!(changed, "mutate_operator_param must change Constant value");
    }

    #[test]
    fn swap_operator_can_produce_parameterized_kinds() {
        // Over many seeds, swap must sometimes produce parameterized kinds
        // (Threshold, DecayIntegrator, Momentum, Oscillator, Constant).
        let mut found_parameterized = false;
        for seed in 0u64..200 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut genome, GraphOperator::SwapGraphOperator, &mut r).is_ok() {
                if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                    for node in &g.internal_nodes {
                        if is_parameterized(&node.kind) {
                            found_parameterized = true;
                            break;
                        }
                    }
                }
            }
            if found_parameterized {
                break;
            }
        }
        assert!(
            found_parameterized,
            "swap must sometimes produce parameterized kinds"
        );
    }

    #[test]
    fn random_graph_node_kind_covers_all_22_variants() {
        use std::collections::HashSet;
        let mut discriminants: HashSet<std::mem::Discriminant<GraphNodeKind>> = HashSet::new();
        for seed in 0u64..2000 {
            let mut r = rng(seed);
            let kind = random_graph_node_kind(&mut r);
            discriminants.insert(std::mem::discriminant(&kind));
        }
        assert_eq!(
            discriminants.len(),
            22,
            "all 22 GraphNodeKind variants must be reachable; got {}",
            discriminants.len()
        );
    }

    #[test]
    fn random_graph_node_kind_reaches_out_of_range_input_ref_and_custom_output() {
        let mut saw_out_of_range_input_ref = false;
        let mut saw_out_of_range_custom_output = false;
        for seed in 0u64..20_000 {
            let mut r = rng(seed);
            match random_graph_node_kind(&mut r) {
                GraphNodeKind::InputRef(idx) if idx > 11 => saw_out_of_range_input_ref = true,
                GraphNodeKind::CustomOutput(idx) if idx > 11 => {
                    saw_out_of_range_custom_output = true
                }
                _ => {}
            }
            if saw_out_of_range_input_ref && saw_out_of_range_custom_output {
                break;
            }
        }
        assert!(
            saw_out_of_range_input_ref,
            "InputRef mutation surface must include out-of-range u8 values"
        );
        assert!(
            saw_out_of_range_custom_output,
            "CustomOutput mutation surface must include out-of-range u8 values"
        );
    }

    #[test]
    fn retarget_graph_edge_changes_source_idx() {
        let genome = v3alpha1_founder_genome();
        // Founder graph node 0 has internal nodes with inputs.
        let mut changed = false;
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::RetargetGraphEdge, &mut r).is_ok() {
                // Check if any edge source_idx differs from original.
                if let (BackendDef::Graph(ref orig), BackendDef::Graph(ref mutated)) =
                    (&genome.nodes[0].backend_def, &g.nodes[0].backend_def)
                {
                    for (o, m) in orig
                        .internal_nodes
                        .iter()
                        .zip(mutated.internal_nodes.iter())
                    {
                        for (oi, mi) in o.inputs.iter().zip(m.inputs.iter()) {
                            if oi.source_idx != mi.source_idx {
                                changed = true;
                                break;
                            }
                        }
                        if changed {
                            break;
                        }
                    }
                }
            }
            if changed {
                break;
            }
        }
        assert!(changed, "retarget must change a source_idx");
    }

    #[test]
    fn retarget_graph_edge_no_edges_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        // Clear all inputs from graph internal nodes.
        if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
            for node in &mut g.internal_nodes {
                node.inputs.clear();
            }
        }
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::RetargetGraphEdge, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn remove_graph_edge_decreases_input_count() {
        let genome = v3alpha1_founder_genome();
        let before: usize = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            g.internal_nodes.iter().map(|n| n.inputs.len()).sum()
        } else {
            panic!("expected graph backend");
        };
        assert!(before > 0, "founder graph must have edges");
        let mut success = false;
        for seed in 0u64..100 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::RemoveGraphEdge, &mut r).is_ok() {
                let after: usize = if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    gd.internal_nodes.iter().map(|n| n.inputs.len()).sum()
                } else {
                    before
                };
                assert_eq!(after, before - 1, "remove_graph_edge must remove one input");
                success = true;
                break;
            }
        }
        assert!(success, "remove_graph_edge must succeed on founder genome");
    }

    #[test]
    fn remove_graph_edge_no_edges_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        if let BackendDef::Graph(ref mut g) = genome.nodes[0].backend_def {
            for node in &mut g.internal_nodes {
                node.inputs.clear();
            }
        }
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::RemoveGraphEdge, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn raw_field_mutation_can_set_input_ref_out_of_range() {
        let mut found_out_of_range = false;
        for seed in 0u64..512 {
            let mut genome = graph_only_genome(vec![GraphInternalNode {
                kind: GraphNodeKind::InputRef(0),
                inputs: vec![],
            }]);
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::GraphRawFieldMutation, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                if let GraphNodeKind::InputRef(idx) = g.internal_nodes[0].kind {
                    if idx > 11 {
                        found_out_of_range = true;
                        break;
                    }
                }
            }
        }
        assert!(
            found_out_of_range,
            "raw graph mutation must reach InputRef values above output slot range"
        );
    }

    #[test]
    fn raw_field_mutation_can_set_custom_output_out_of_range() {
        let mut found_out_of_range = false;
        for seed in 0u64..512 {
            let mut genome = graph_only_genome(vec![GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(0),
                inputs: vec![],
            }]);
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::GraphRawFieldMutation, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                if let GraphNodeKind::CustomOutput(slot) = g.internal_nodes[0].kind {
                    if slot > 11 {
                        found_out_of_range = true;
                        break;
                    }
                }
            }
        }
        assert!(
            found_out_of_range,
            "raw graph mutation must reach CustomOutput values above output slot range"
        );
    }

    #[test]
    fn raw_field_mutation_can_set_edge_source_out_of_range() {
        let mut found_out_of_range = false;
        for seed in 0u64..512 {
            let mut genome = graph_only_genome(vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(1.0),
                    inputs: vec![],
                },
                GraphInternalNode {
                    kind: GraphNodeKind::Add,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                },
            ]);
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::GraphRawFieldMutation, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let source_idx = g.internal_nodes[1].inputs[0].source_idx as usize;
                if source_idx >= g.internal_nodes.len() {
                    found_out_of_range = true;
                    break;
                }
            }
        }
        assert!(
            found_out_of_range,
            "raw graph mutation must reach edge source indices outside internal node bounds"
        );
    }

    #[test]
    fn graph_after_mutation_passes_parseability_gate() {
        let operators = [
            GraphOperator::AlterGraphEdgeWeight,
            GraphOperator::SwapGraphOperator,
            GraphOperator::MutateGraphOperatorParam,
            GraphOperator::AddInternalGraphNode,
            GraphOperator::RemoveInternalGraphNode,
            GraphOperator::AddGraphEdge,
            GraphOperator::RetargetGraphEdge,
            GraphOperator::RemoveGraphEdge,
            GraphOperator::GraphRawFieldMutation,
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
