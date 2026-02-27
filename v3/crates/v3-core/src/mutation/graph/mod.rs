use rand::Rng;

use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphInput, GraphInternalNode, GraphNodeKind,
};
use crate::mutation::types::MutationSkipReason;

/// Graph mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    CopyInternalNode,
    CopySubgraph,
    CopyEdgeBundle,
}

impl GraphOperator {
    pub const ALL: [Self; 12] = [
        Self::AlterGraphEdgeWeight,
        Self::SwapGraphOperator,
        Self::MutateGraphOperatorParam,
        Self::AddInternalGraphNode,
        Self::RemoveInternalGraphNode,
        Self::AddGraphEdge,
        Self::RetargetGraphEdge,
        Self::RemoveGraphEdge,
        Self::GraphRawFieldMutation,
        Self::CopyInternalNode,
        Self::CopySubgraph,
        Self::CopyEdgeBundle,
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::AlterGraphEdgeWeight => 4,
            Self::SwapGraphOperator => 2,
            Self::MutateGraphOperatorParam => 4,
            Self::AddInternalGraphNode => 1,
            Self::RemoveInternalGraphNode => 1,
            Self::AddGraphEdge => 2,
            Self::RetargetGraphEdge => 2,
            Self::RemoveGraphEdge => 2,
            Self::GraphRawFieldMutation => 4,
            Self::CopyInternalNode => 1,
            Self::CopySubgraph => 1,
            Self::CopyEdgeBundle => 2,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 12,
            "ALL must cover every GraphOperator variant"
        );
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            sum += Self::ALL[i].weight() as u16;
            i += 1;
        }
        sum
    };

    /// Pick a random graph operator weighted by impact tier.
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
            GraphOperator::CopyInternalNode => apply_copy_internal_node(genome, node_idx, rng),
            GraphOperator::CopySubgraph => apply_copy_subgraph(genome, node_idx, rng),
            GraphOperator::CopyEdgeBundle => apply_copy_edge_bundle(genome, node_idx, rng),
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
        let w = &mut g.internal_nodes[int_idx].inputs[edge_idx].weight;
        if w.abs() > 0.01 {
            *w *= 1.0 + rng.gen_range(-0.2f32..=0.2);
        } else {
            *w += rng.gen_range(-0.1f32..=0.1);
        }
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
        match &mut g.internal_nodes[int_idx].kind {
            GraphNodeKind::CustomOutput(ref mut slot) | GraphNodeKind::InputRef(ref mut slot) => {
                if rng.gen_bool(0.5) {
                    *slot = slot.wrapping_add(1);
                } else {
                    *slot = slot.wrapping_sub(1);
                }
            }
            GraphNodeKind::Constant(ref mut p)
            | GraphNodeKind::Threshold(ref mut p)
            | GraphNodeKind::DecayIntegrator(ref mut p)
            | GraphNodeKind::Momentum(ref mut p)
            | GraphNodeKind::Oscillator(ref mut p) => {
                *p += rng.gen_range(-0.1f32..=0.1);
            }
            _ => unreachable!("is_parameterized filter should prevent reaching here"),
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
        let existing_count = g.internal_nodes.len();
        let kind = random_graph_node_kind(rng);
        let inputs = if existing_count > 0 && rng.gen_bool(0.5) {
            vec![GraphInput {
                source_idx: rng.gen_range(0..existing_count) as u16,
                weight: rng.gen_range(-1.0f32..=1.0),
            }]
        } else {
            vec![]
        };
        g.internal_nodes.push(GraphInternalNode { kind, inputs });
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
            | GraphNodeKind::CustomOutput(_)
            | GraphNodeKind::InputRef(_)
    )
}

fn apply_copy_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let source_idx = rng.gen_range(0..g.internal_nodes.len());
        let mut copy = g.internal_nodes[source_idx].clone();
        // Coin flip: clear inputs or keep.
        if rng.gen_bool(0.5) {
            copy.inputs.clear();
        }
        let new_idx = g.internal_nodes.len();
        g.internal_nodes.push(copy);
        // Coin flip: add backlink edge from random existing node to the copy.
        if rng.gen_bool(0.5) {
            let target = rng.gen_range(0..new_idx);
            g.internal_nodes[target].inputs.push(GraphInput {
                source_idx: new_idx as u16,
                weight: rng.gen_range(-1.0f32..=1.0),
            });
        }
    }
    Ok(())
}

fn apply_copy_subgraph(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.len() < 2 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        // Pick seed node.
        let seed = rng.gen_range(0..g.internal_nodes.len());
        let target_size = rng.gen_range(2..=4).min(g.internal_nodes.len());
        // Random walk to build cluster.
        let mut cluster = vec![seed];
        let mut in_cluster = vec![false; g.internal_nodes.len()];
        in_cluster[seed] = true;
        let mut neighbors = Vec::new();
        for _ in 0..10 {
            if cluster.len() >= target_size {
                break;
            }
            let current = cluster[rng.gen_range(0..cluster.len())];
            // Collect neighbors: inputs of current and nodes that reference current.
            neighbors.clear();
            for edge in &g.internal_nodes[current].inputs {
                let src = edge.source_idx as usize;
                if src < g.internal_nodes.len() && !in_cluster[src] {
                    neighbors.push(src);
                }
            }
            for (i, node) in g.internal_nodes.iter().enumerate() {
                if !in_cluster[i] {
                    for edge in &node.inputs {
                        if edge.source_idx as usize == current {
                            neighbors.push(i);
                            break;
                        }
                    }
                }
            }
            if !neighbors.is_empty() {
                let next = neighbors[rng.gen_range(0..neighbors.len())];
                if !in_cluster[next] {
                    in_cluster[next] = true;
                    cluster.push(next);
                }
            }
        }
        cluster.sort_unstable();
        // Build old→new index map.
        let base = g.internal_nodes.len();
        let old_to_new: std::collections::HashMap<usize, usize> = cluster
            .iter()
            .enumerate()
            .map(|(i, &old)| (old, base + i))
            .collect();
        // Clone nodes and remap intra-cluster edges.
        let mut cloned_nodes: Vec<GraphInternalNode> = cluster
            .iter()
            .map(|&idx| {
                let mut node = g.internal_nodes[idx].clone();
                for edge in &mut node.inputs {
                    let src = edge.source_idx as usize;
                    if let Some(&new_idx) = old_to_new.get(&src) {
                        edge.source_idx = new_idx as u16;
                    }
                    // External edges keep their original source_idx.
                }
                node
            })
            .collect();
        g.internal_nodes.append(&mut cloned_nodes);
    }
    Ok(())
}

fn apply_copy_edge_bundle(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.len() < 2 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let source = rng.gen_range(0..g.internal_nodes.len());
        if g.internal_nodes[source].inputs.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        // Pick a different target.
        let mut target = rng.gen_range(0..g.internal_nodes.len() - 1);
        if target >= source {
            target += 1;
        }
        let copied_edges = g.internal_nodes[source].inputs.clone();
        g.internal_nodes[target].inputs.extend(copied_edges);
    }
    Ok(())
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
            GraphOperator::CopyInternalNode,
            GraphOperator::CopySubgraph,
            GraphOperator::CopyEdgeBundle,
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

    // ── GraphCopyInternalNode tests ──

    #[test]
    fn copy_internal_node_increases_count() {
        let mut genome = v3alpha1_founder_genome();
        let before = graph_node_internal_count(&genome);
        let mut r = rng(0);
        GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
        let after = graph_node_internal_count(&genome);
        assert_eq!(after, before + 1);
    }

    #[test]
    fn copy_internal_node_on_empty_returns_no_applicable_target() {
        let mut genome = graph_only_genome(vec![]);
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn copy_internal_node_preserves_kind() {
        // The copied node must have the same kind as the source.
        for seed in 0u64..20 {
            let mut genome = graph_only_genome(vec![GraphInternalNode {
                kind: GraphNodeKind::Sigmoid,
                inputs: vec![],
            }]);
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                assert_eq!(g.internal_nodes.len(), 2);
                assert!(
                    matches!(g.internal_nodes[1].kind, GraphNodeKind::Sigmoid),
                    "copied node must preserve kind"
                );
            }
        }
    }

    #[test]
    fn copy_internal_node_sometimes_copies_edges_sometimes_not() {
        let source_node = GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
        };
        let mut saw_with_edges = false;
        let mut saw_without_edges = false;
        for seed in 0u64..200 {
            let mut genome = graph_only_genome(vec![source_node.clone()]);
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let copy = &g.internal_nodes[1];
                if copy.inputs.is_empty() {
                    saw_without_edges = true;
                } else {
                    saw_with_edges = true;
                }
            }
            if saw_with_edges && saw_without_edges {
                break;
            }
        }
        assert!(saw_with_edges, "must sometimes copy edges");
        assert!(saw_without_edges, "must sometimes clear edges");
    }

    #[test]
    fn copy_internal_node_sometimes_adds_backlink_sometimes_not() {
        let source_node = GraphInternalNode {
            kind: GraphNodeKind::Relu,
            inputs: vec![],
        };
        let mut saw_backlink = false;
        let mut saw_no_backlink = false;
        for seed in 0u64..200 {
            let mut genome = graph_only_genome(vec![source_node.clone()]);
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::CopyInternalNode, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                // Check if any existing node got a new input pointing to the copy (index 1).
                let has_backlink = g.internal_nodes.iter().enumerate().any(|(idx, n)| {
                    idx < g.internal_nodes.len() - 1
                        && n.inputs
                            .iter()
                            .any(|e| e.source_idx as usize == g.internal_nodes.len() - 1)
                });
                if has_backlink {
                    saw_backlink = true;
                } else {
                    saw_no_backlink = true;
                }
            }
            if saw_backlink && saw_no_backlink {
                break;
            }
        }
        assert!(saw_backlink, "must sometimes add backlink");
        assert!(saw_no_backlink, "must sometimes skip backlink");
    }

    // ── GraphCopySubgraph tests ──

    #[test]
    fn copy_subgraph_increases_count() {
        let mut genome = v3alpha1_founder_genome();
        let before = graph_node_internal_count(&genome);
        let mut r = rng(0);
        GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r).unwrap();
        let after = graph_node_internal_count(&genome);
        assert!(after > before, "subgraph copy must add nodes");
    }

    #[test]
    fn copy_subgraph_fewer_than_2_returns_no_applicable_target() {
        let mut genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![],
        }]);
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn copy_subgraph_remaps_intra_cluster_edges() {
        // Two nodes with edges between them — clone should remap internal edges.
        let nodes = vec![
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
        ];
        let mut found_remapped = false;
        for seed in 0u64..100 {
            let mut genome = graph_only_genome(nodes.clone());
            let mut r = rng(seed);
            let result = GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r);
            if result.is_ok() {
                if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                    // New nodes start at index 2. Check if any cloned node has an edge
                    // pointing to another cloned node (index >= 2).
                    for node in &g.internal_nodes[2..] {
                        for edge in &node.inputs {
                            if edge.source_idx as usize >= 2 {
                                found_remapped = true;
                            }
                        }
                    }
                }
            }
            if found_remapped {
                break;
            }
        }
        assert!(
            found_remapped,
            "subgraph copy must remap intra-cluster edges"
        );
    }

    #[test]
    fn copy_subgraph_preserves_external_edges() {
        // Node 1 has an edge to node 0. If only node 1 is in the cluster,
        // the edge should stay pointing to node 0 (external).
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::Constant(1.0),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Relu,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 0.5,
                }],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 1,
                    weight: 1.0,
                }],
            },
        ];
        let mut found_external_preserved = false;
        for seed in 0u64..200 {
            let mut genome = graph_only_genome(nodes.clone());
            let mut r = rng(seed);
            if GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r).is_ok() {
                if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                    // Check cloned nodes for edges pointing to external indices (< 3).
                    for node in &g.internal_nodes[3..] {
                        for edge in &node.inputs {
                            if (edge.source_idx as usize) < 3 {
                                found_external_preserved = true;
                            }
                        }
                    }
                }
            }
            if found_external_preserved {
                break;
            }
        }
        assert!(
            found_external_preserved,
            "subgraph copy must preserve external edge targets"
        );
    }

    #[test]
    fn copy_subgraph_preserves_node_kinds() {
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::Sigmoid,
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Tanh,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
            },
        ];
        let mut r = rng(0);
        let mut genome = graph_only_genome(nodes);
        GraphMutator::apply(&mut genome, GraphOperator::CopySubgraph, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let cloned_kinds: Vec<_> = g.internal_nodes[2..]
                .iter()
                .map(|n| std::mem::discriminant(&n.kind))
                .collect();
            // Cloned kinds must be a subset of {Sigmoid, Tanh}.
            let sigmoid_disc = std::mem::discriminant(&GraphNodeKind::Sigmoid);
            let tanh_disc = std::mem::discriminant(&GraphNodeKind::Tanh);
            for k in &cloned_kinds {
                assert!(
                    *k == sigmoid_disc || *k == tanh_disc,
                    "cloned node kind must match original"
                );
            }
        }
    }

    // ── GraphCopyEdgeBundle tests ──

    #[test]
    fn copy_edge_bundle_copies_all_edges() {
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::Constant(1.0),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![
                    GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    },
                    GraphInput {
                        source_idx: 0,
                        weight: 2.0,
                    },
                ],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Relu,
                inputs: vec![],
            },
        ];
        let mut found_copied = false;
        for seed in 0u64..100 {
            let mut genome = graph_only_genome(nodes.clone());
            let mut r = rng(seed);
            if GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r).is_ok() {
                if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                    // The target node (initially empty Relu at idx 2) should now have edges.
                    // Or node 1's edges were copied onto node 0 or 2.
                    let total_edges: usize = g.internal_nodes.iter().map(|n| n.inputs.len()).sum();
                    if total_edges > 2 {
                        found_copied = true;
                        break;
                    }
                }
            }
        }
        assert!(found_copied, "edge bundle must copy edges to target");
    }

    #[test]
    fn copy_edge_bundle_fewer_than_2_returns_no_applicable_target() {
        let mut genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
        }]);
        let mut r = rng(0);
        let result = GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn copy_edge_bundle_source_no_edges_returns_no_applicable_target() {
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::Constant(1.0),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Relu,
                inputs: vec![],
            },
        ];
        let mut all_skip = true;
        for seed in 0u64..100 {
            let mut genome = graph_only_genome(nodes.clone());
            let mut r = rng(seed);
            let result = GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r);
            if result.is_ok() {
                all_skip = false;
                break;
            }
        }
        assert!(
            all_skip,
            "with no edges on any node, all attempts must return NoApplicableTarget"
        );
    }

    // ── Gap 2: Randomize AddInternalGraphNode kind tests ──

    #[test]
    fn add_internal_node_uses_varied_kinds() {
        use std::collections::HashSet;
        let mut discriminants: HashSet<std::mem::Discriminant<GraphNodeKind>> = HashSet::new();
        for seed in 0u64..500 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let last = g.internal_nodes.last().unwrap();
                discriminants.insert(std::mem::discriminant(&last.kind));
            }
        }
        assert!(
            discriminants.len() >= 5,
            "must see ≥5 distinct kinds; got {}",
            discriminants.len()
        );
    }

    #[test]
    fn add_internal_node_sometimes_adds_initial_edge() {
        let mut saw_with_edge = false;
        let mut saw_without_edge = false;
        for seed in 0u64..200 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let last = g.internal_nodes.last().unwrap();
                if last.inputs.is_empty() {
                    saw_without_edge = true;
                } else {
                    saw_with_edge = true;
                }
            }
            if saw_with_edge && saw_without_edge {
                break;
            }
        }
        assert!(saw_with_edge, "must sometimes add initial edge");
        assert!(saw_without_edge, "must sometimes have no initial edge");
    }

    #[test]
    fn add_internal_node_edge_points_to_existing_internal() {
        for seed in 0u64..200 {
            let mut genome = v3alpha1_founder_genome();
            let pre_count = if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                g.internal_nodes.len()
            } else {
                continue;
            };
            let mut r = rng(seed);
            GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let last = g.internal_nodes.last().unwrap();
                for edge in &last.inputs {
                    assert!(
                        (edge.source_idx as usize) < pre_count,
                        "edge source_idx {} must be < pre-existing count {}",
                        edge.source_idx,
                        pre_count
                    );
                }
            }
        }
    }

    #[test]
    fn add_internal_node_skips_edge_when_no_existing_internals() {
        let mut genome = graph_only_genome(vec![]);
        let mut r = rng(42);
        GraphMutator::apply(&mut genome, GraphOperator::AddInternalGraphNode, &mut r).unwrap();
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert_eq!(g.internal_nodes.len(), 1);
            assert!(
                g.internal_nodes[0].inputs.is_empty(),
                "new node in empty graph must have no inputs"
            );
        }
    }

    // ── Gap 7: Proportional edge weight step tests ──

    #[test]
    fn alter_edge_weight_proportional_for_large_weights() {
        // Weight=10.0 should be scaled by ±20%, so result in [8.0, 12.0].
        let genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 10.0,
            }],
        }]);
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            GraphMutator::apply(&mut g, GraphOperator::AlterGraphEdgeWeight, &mut r).unwrap();
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                let w = gd.internal_nodes[0].inputs[0].weight;
                assert!(
                    (8.0..=12.0).contains(&w),
                    "weight {} must be in [8.0, 12.0] for proportional step on 10.0",
                    w
                );
            }
        }
    }

    #[test]
    fn alter_edge_weight_absolute_for_near_zero_weights() {
        // Weight=0.0 should use absolute step, result in [-0.1, 0.1].
        let genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 0.0,
            }],
        }]);
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            GraphMutator::apply(&mut g, GraphOperator::AlterGraphEdgeWeight, &mut r).unwrap();
            if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                let w = gd.internal_nodes[0].inputs[0].weight;
                assert!(
                    (-0.1..=0.1).contains(&w),
                    "weight {} must be in [-0.1, 0.1] for absolute step near zero",
                    w
                );
            }
        }
    }

    // ── Gap 1: CustomOutput/InputRef parameterization tests ──

    #[test]
    fn custom_output_is_parameterized() {
        assert!(
            is_parameterized(&GraphNodeKind::CustomOutput(5)),
            "CustomOutput must be parameterized"
        );
    }

    #[test]
    fn input_ref_is_parameterized() {
        assert!(
            is_parameterized(&GraphNodeKind::InputRef(3)),
            "InputRef must be parameterized"
        );
    }

    #[test]
    fn mutate_operator_param_changes_custom_output_slot() {
        let genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::CustomOutput(5),
            inputs: vec![],
        }]);
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok()
            {
                if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    if let GraphNodeKind::CustomOutput(slot) = gd.internal_nodes[0].kind {
                        if slot != 5 {
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
        assert!(changed, "CustomOutput slot must change by ±1");
    }

    #[test]
    fn mutate_operator_param_changes_input_ref_index() {
        let genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::InputRef(3),
            inputs: vec![],
        }]);
        let mut changed = false;
        for seed in 0u64..50 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok()
            {
                if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    if let GraphNodeKind::InputRef(idx) = gd.internal_nodes[0].kind {
                        if idx != 3 {
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
        assert!(changed, "InputRef index must change by ±1");
    }

    #[test]
    fn mutate_operator_param_wraps_custom_output_at_boundary() {
        // CustomOutput(0) should eventually wrap to 255.
        let genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::CustomOutput(0),
            inputs: vec![],
        }]);
        let mut saw_255 = false;
        for seed in 0u64..200 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok()
            {
                if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    if let GraphNodeKind::CustomOutput(slot) = gd.internal_nodes[0].kind {
                        if slot == 255 {
                            saw_255 = true;
                            break;
                        }
                    }
                }
            }
        }
        assert!(saw_255, "CustomOutput(0) must wrap to 255 via wrapping_sub");
    }

    #[test]
    fn mutate_operator_param_wraps_input_ref_at_boundary() {
        // InputRef(255) should eventually wrap to 0.
        let genome = graph_only_genome(vec![GraphInternalNode {
            kind: GraphNodeKind::InputRef(255),
            inputs: vec![],
        }]);
        let mut saw_0 = false;
        for seed in 0u64..200 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if GraphMutator::apply(&mut g, GraphOperator::MutateGraphOperatorParam, &mut r).is_ok()
            {
                if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                    if let GraphNodeKind::InputRef(idx) = gd.internal_nodes[0].kind {
                        if idx == 0 {
                            saw_0 = true;
                            break;
                        }
                    }
                }
            }
        }
        assert!(saw_0, "InputRef(255) must wrap to 0 via wrapping_add");
    }

    #[test]
    fn copy_edge_bundle_preserves_source_edges() {
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 3.0,
                }],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Relu,
                inputs: vec![],
            },
        ];
        for seed in 0u64..100 {
            let mut genome = graph_only_genome(nodes.clone());
            let mut r = rng(seed);
            if GraphMutator::apply(&mut genome, GraphOperator::CopyEdgeBundle, &mut r).is_ok() {
                if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                    // Source node (idx 0) must still have its original edge.
                    assert!(
                        g.internal_nodes[0]
                            .inputs
                            .iter()
                            .any(|e| (e.weight - 3.0).abs() < 1e-7),
                        "source edges must be preserved"
                    );
                }
            }
        }
    }

    #[test]
    fn graph_weighted_random_favors_refinement() {
        let mut counts = std::collections::HashMap::new();
        let mut r = rng(42);
        for _ in 0..10_000 {
            let op = GraphOperator::random(&mut r);
            *counts.entry(op).or_insert(0u32) += 1;
        }
        let alter = counts
            .get(&GraphOperator::AlterGraphEdgeWeight)
            .copied()
            .unwrap_or(0);
        let add_node = counts
            .get(&GraphOperator::AddInternalGraphNode)
            .copied()
            .unwrap_or(0);
        assert!(
            alter > add_node * 2,
            "AlterGraphEdgeWeight (weight 4) must appear >2x AddInternalGraphNode (weight 1); got {} vs {}",
            alter, add_node,
        );
    }

    #[test]
    fn graph_operator_weights_are_positive() {
        let all = GraphOperator::ALL;
        assert_eq!(all.len(), 12, "ALL must cover every GraphOperator variant");
        for &op in &all {
            assert!(op.weight() > 0, "weight must be positive for {:?}", op);
        }
    }
}
