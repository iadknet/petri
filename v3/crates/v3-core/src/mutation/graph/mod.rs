pub mod hebbian;

use rand::Rng;

use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphInput, GraphInternalNode, GraphNodeKind,
};
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

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
    EnableHebbian,
    DisableHebbian,
    MutateHebbianRule,
    MutateHebbianRate,
    ToggleHebbianLamarckian,
    EnableRewardModulation,
    DisableRewardModulation,
    MutateRewardSource,
    MutateTraceDecay,
}

impl GraphOperator {
    pub const ALL: [Self; 21] = [
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
        Self::EnableHebbian,
        Self::DisableHebbian,
        Self::MutateHebbianRule,
        Self::MutateHebbianRate,
        Self::ToggleHebbianLamarckian,
        Self::EnableRewardModulation,
        Self::DisableRewardModulation,
        Self::MutateRewardSource,
        Self::MutateTraceDecay,
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
            Self::EnableHebbian => 1,
            Self::DisableHebbian => 1,
            Self::MutateHebbianRule => 2,
            Self::MutateHebbianRate => 4,
            Self::ToggleHebbianLamarckian => 2,
            Self::EnableRewardModulation => 1,
            Self::DisableRewardModulation => 1,
            Self::MutateRewardSource => 2,
            Self::MutateTraceDecay => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 21,
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

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::AddInternalGraphNode
            | Self::AddGraphEdge
            | Self::CopyInternalNode
            | Self::CopySubgraph
            | Self::CopyEdgeBundle
            | Self::EnableHebbian
            | Self::EnableRewardModulation => ComplexityEffect::Increasing,
            Self::RemoveInternalGraphNode
            | Self::RemoveGraphEdge
            | Self::DisableHebbian
            | Self::DisableRewardModulation => ComplexityEffect::Decreasing,
            Self::AlterGraphEdgeWeight
            | Self::SwapGraphOperator
            | Self::MutateGraphOperatorParam
            | Self::RetargetGraphEdge
            | Self::GraphRawFieldMutation
            | Self::MutateHebbianRule
            | Self::MutateHebbianRate
            | Self::ToggleHebbianLamarckian
            | Self::MutateRewardSource
            | Self::MutateTraceDecay => ComplexityEffect::Neutral,
        }
    }

    const NON_INCREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if !Self::ALL[i].complexity_effect().is_increasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    const DECREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if Self::ALL[i].complexity_effect().is_decreasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    /// Pick a random Decreasing-only operator weighted by impact tier.
    pub fn random_decreasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::DECREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::DECREASING_WEIGHT);
        for &op in &Self::ALL {
            if !op.complexity_effect().is_decreasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

    /// Pick a random non-increasing operator (Neutral or Decreasing) weighted by impact tier.
    pub fn random_non_increasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::NON_INCREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::NON_INCREASING_WEIGHT);
        for &op in &Self::ALL {
            if op.complexity_effect().is_increasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

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
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason::NoApplicableTarget)`
    /// if the genome has no Graph-backend nodes or no applicable internal target.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: GraphOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
    ) -> Result<TargetReachability, MutationSkipReason> {
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

        let (node_idx, reachability) =
            biased_select_from(&graph_indices, reachable_nodes, bias, rng)
                .ok_or(MutationSkipReason::NoApplicableTarget)?;
        let result = match op {
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
            GraphOperator::EnableHebbian => hebbian::enable_hebbian(genome, node_idx, rng),
            GraphOperator::DisableHebbian => hebbian::disable_hebbian(genome, node_idx, rng),
            GraphOperator::MutateHebbianRule => hebbian::mutate_hebbian_rule(genome, node_idx, rng),
            GraphOperator::MutateHebbianRate => hebbian::mutate_hebbian_rate(genome, node_idx, rng),
            GraphOperator::ToggleHebbianLamarckian => {
                hebbian::toggle_hebbian_lamarckian(genome, node_idx, rng)
            }
            GraphOperator::EnableRewardModulation => {
                hebbian::enable_reward_modulation(genome, node_idx, rng)
            }
            GraphOperator::DisableRewardModulation => {
                hebbian::disable_reward_modulation(genome, node_idx, rng)
            }
            GraphOperator::MutateRewardSource => {
                hebbian::mutate_reward_source(genome, node_idx, rng)
            }
            GraphOperator::MutateTraceDecay => hebbian::mutate_trace_decay(genome, node_idx, rng),
        };
        result.map(|()| reachability)
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
            GraphNodeKind::CustomOutput(ref mut slot) => {
                if rng.gen_bool(0.5) {
                    *slot = slot.wrapping_add(1);
                } else {
                    *slot = slot.wrapping_sub(1);
                }
            }
            GraphNodeKind::InputRef {
                ref mut ref_idx, ..
            } => {
                if rng.gen_bool(0.5) {
                    *ref_idx = ref_idx.wrapping_add(1);
                } else {
                    *ref_idx = ref_idx.wrapping_sub(1);
                }
            }
            GraphNodeKind::Constant(ref mut p)
            | GraphNodeKind::Threshold(ref mut p)
            | GraphNodeKind::DecayIntegrator(ref mut p)
            | GraphNodeKind::Momentum(ref mut p)
            | GraphNodeKind::Oscillator(ref mut p) => {
                *p += rng.gen_range(-0.1f32..=0.1);
            }
            GraphNodeKind::WriteActionMeta(ref mut slot)
            | GraphNodeKind::PushAction(ref mut slot) => {
                if rng.gen_bool(0.5) {
                    *slot = slot.wrapping_add(1);
                } else {
                    *slot = slot.wrapping_sub(1);
                }
            }
            GraphNodeKind::ReadSlot(ref mut slot_idx)
            | GraphNodeKind::ReadSlotPrev(ref mut slot_idx)
            | GraphNodeKind::WriteSlot(ref mut slot_idx)
            | GraphNodeKind::ClearSlot(ref mut slot_idx) => {
                if rng.gen_bool(0.5) {
                    *slot_idx = slot_idx.wrapping_add(1) % 16;
                } else {
                    *slot_idx = slot_idx.wrapping_sub(1) % 16;
                }
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
        g.internal_nodes.push(GraphInternalNode {
            kind,
            inputs,
            plasticity: None,
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
                GraphNodeKind::InputRef { .. }
                    | GraphNodeKind::CustomOutput(_)
                    | GraphNodeKind::WriteActionMeta(_)
                    | GraphNodeKind::PushAction(_)
                    | GraphNodeKind::ReadSlot(_)
                    | GraphNodeKind::ReadSlotPrev(_)
                    | GraphNodeKind::WriteSlot(_)
                    | GraphNodeKind::ClearSlot(_)
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
                GraphNodeKind::InputRef { .. }
                    | GraphNodeKind::CustomOutput(_)
                    | GraphNodeKind::WriteActionMeta(_)
                    | GraphNodeKind::PushAction(_)
                    | GraphNodeKind::ReadSlot(_)
                    | GraphNodeKind::ReadSlotPrev(_)
                    | GraphNodeKind::WriteSlot(_)
                    | GraphNodeKind::ClearSlot(_)
            ) {
                if pick == 0 {
                    match &g.internal_nodes[int_idx].kind {
                        GraphNodeKind::InputRef { .. } => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::InputRef {
                                ref_idx: rng.gen(),
                                sub_idx: 0,
                            };
                        }
                        GraphNodeKind::CustomOutput(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::CustomOutput(rng.gen());
                        }
                        GraphNodeKind::WriteActionMeta(_) => {
                            g.internal_nodes[int_idx].kind =
                                GraphNodeKind::WriteActionMeta(rng.gen());
                        }
                        GraphNodeKind::PushAction(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::PushAction(rng.gen());
                        }
                        GraphNodeKind::ReadSlot(_) => {
                            g.internal_nodes[int_idx].kind =
                                GraphNodeKind::ReadSlot(rng.gen_range(0u8..16));
                        }
                        GraphNodeKind::ReadSlotPrev(_) => {
                            g.internal_nodes[int_idx].kind =
                                GraphNodeKind::ReadSlotPrev(rng.gen_range(0u8..16));
                        }
                        GraphNodeKind::WriteSlot(_) => {
                            g.internal_nodes[int_idx].kind =
                                GraphNodeKind::WriteSlot(rng.gen_range(0u8..16));
                        }
                        GraphNodeKind::ClearSlot(_) => {
                            g.internal_nodes[int_idx].kind =
                                GraphNodeKind::ClearSlot(rng.gen_range(0u8..16));
                        }
                        _ => unreachable!(),
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

/// Return a random GraphNodeKind covering all 30 variants with random initial params.
fn random_graph_node_kind(rng: &mut impl Rng) -> GraphNodeKind {
    match rng.gen_range(0u8..30) {
        0 => GraphNodeKind::InputRef {
            ref_idx: rng.gen(),
            sub_idx: 0,
        },
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
        21 => GraphNodeKind::RouterOutput,
        22 => GraphNodeKind::WriteActionMeta(rng.gen_range(0u8..8)),
        23 => GraphNodeKind::PushAction(rng.gen_range(0u8..5)),
        24 => GraphNodeKind::PopAction,
        25 => GraphNodeKind::ExecuteActionQueue,
        26 => GraphNodeKind::ReadSlot(rng.gen_range(0u8..16)),
        27 => GraphNodeKind::ReadSlotPrev(rng.gen_range(0u8..16)),
        28 => GraphNodeKind::WriteSlot(rng.gen_range(0u8..16)),
        _ => GraphNodeKind::ClearSlot(rng.gen_range(0u8..16)),
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
            | GraphNodeKind::InputRef { .. }
            | GraphNodeKind::WriteActionMeta(_)
            | GraphNodeKind::PushAction(_)
            | GraphNodeKind::ReadSlot(_)
            | GraphNodeKind::ReadSlotPrev(_)
            | GraphNodeKind::WriteSlot(_)
            | GraphNodeKind::ClearSlot(_)
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
mod tests;
