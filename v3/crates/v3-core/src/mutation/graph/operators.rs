use rand::Rng;

use crate::creature::action_log::ACTION_TYPE_COUNT;
use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphInput, GraphInternalNode, GraphNodeKind,
};
use crate::creature::state::SHARED_MEMORY_SLOTS;
use crate::mutation::types::MutationSkipReason;
use crate::runtime::graph_effects::ACTION_META_SLOTS;
use crate::runtime::types::OUTPUT_SLOT_COUNT;

/// Cyclic ±1 step within [0, modulus) for u8.
fn step_bounded_u8(value: &mut u8, modulus: u8, rng: &mut impl Rng) {
    if rng.gen_bool(0.5) {
        *value = value.wrapping_add(1) % modulus;
    } else {
        *value = value.wrapping_add(modulus - 1) % modulus;
    }
}

/// Cyclic ±1 step within [0, modulus) for u16.
fn step_bounded_u16(value: &mut u16, modulus: u16, rng: &mut impl Rng) {
    if rng.gen_bool(0.5) {
        *value = value.wrapping_add(1) % modulus;
    } else {
        *value = value.wrapping_add(modulus - 1) % modulus;
    }
}

pub(super) fn alter_edge_weight(
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

pub(super) fn swap_operator(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let input_ref_count = genome.nodes[node_idx].input_refs.len() as u16;
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        if g.internal_nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let int_idx = rng.gen_range(0..g.internal_nodes.len());
        g.internal_nodes[int_idx].kind = random_graph_node_kind(input_ref_count, rng);
    }
    Ok(())
}

pub(super) fn mutate_operator_param(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let input_ref_count = genome.nodes[node_idx].input_refs.len() as u16;
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
                step_bounded_u8(slot, OUTPUT_SLOT_COUNT as u8, rng);
            }
            GraphNodeKind::InputRef {
                ref mut ref_idx, ..
            } => {
                let n = input_ref_count.max(1);
                step_bounded_u16(ref_idx, n, rng);
            }
            GraphNodeKind::Constant(ref mut p)
            | GraphNodeKind::Threshold(ref mut p)
            | GraphNodeKind::DecayIntegrator(ref mut p)
            | GraphNodeKind::Momentum(ref mut p)
            | GraphNodeKind::Oscillator(ref mut p) => {
                *p += rng.gen_range(-0.1f32..=0.1);
            }
            GraphNodeKind::WriteActionMeta(ref mut slot) => {
                step_bounded_u8(slot, ACTION_META_SLOTS as u8, rng);
            }
            GraphNodeKind::PushAction(ref mut slot) => {
                step_bounded_u8(slot, ACTION_TYPE_COUNT, rng);
            }
            GraphNodeKind::ReadSlot(ref mut slot_idx)
            | GraphNodeKind::ReadSlotPrev(ref mut slot_idx)
            | GraphNodeKind::WriteSlot(ref mut slot_idx)
            | GraphNodeKind::ClearSlot(ref mut slot_idx) => {
                step_bounded_u8(slot_idx, SHARED_MEMORY_SLOTS as u8, rng);
            }
            _ => unreachable!("is_parameterized filter should prevent reaching here"),
        }
    }
    Ok(())
}

pub(super) fn add_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let input_ref_count = genome.nodes[node_idx].input_refs.len() as u16;
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let existing_count = g.internal_nodes.len();
        let kind = random_graph_node_kind(input_ref_count, rng);
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

pub(super) fn add_graph_edge(
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

pub(super) fn remove_internal_node(
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

pub(super) fn retarget_graph_edge(
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

pub(super) fn remove_graph_edge(
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

pub(super) fn apply_graph_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let input_ref_count = genome.nodes[node_idx].input_refs.len() as u16;
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
                                ref_idx: rng.gen_range(0..input_ref_count.max(1)),
                                sub_idx: 0,
                            };
                        }
                        GraphNodeKind::CustomOutput(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::CustomOutput(
                                rng.gen_range(0..OUTPUT_SLOT_COUNT as u8),
                            );
                        }
                        GraphNodeKind::WriteActionMeta(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::WriteActionMeta(
                                rng.gen_range(0..ACTION_META_SLOTS as u8),
                            );
                        }
                        GraphNodeKind::PushAction(_) => {
                            g.internal_nodes[int_idx].kind =
                                GraphNodeKind::PushAction(rng.gen_range(0..ACTION_TYPE_COUNT));
                        }
                        GraphNodeKind::ReadSlot(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::ReadSlot(
                                rng.gen_range(0..SHARED_MEMORY_SLOTS as u8),
                            );
                        }
                        GraphNodeKind::ReadSlotPrev(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::ReadSlotPrev(
                                rng.gen_range(0..SHARED_MEMORY_SLOTS as u8),
                            );
                        }
                        GraphNodeKind::WriteSlot(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::WriteSlot(
                                rng.gen_range(0..SHARED_MEMORY_SLOTS as u8),
                            );
                        }
                        GraphNodeKind::ClearSlot(_) => {
                            g.internal_nodes[int_idx].kind = GraphNodeKind::ClearSlot(
                                rng.gen_range(0..SHARED_MEMORY_SLOTS as u8),
                            );
                        }
                        _ => unreachable!(),
                    }
                    return Ok(());
                }
                pick -= 1;
            }

            for edge_idx in 0..g.internal_nodes[int_idx].inputs.len() {
                if pick == 0 {
                    g.internal_nodes[int_idx].inputs[edge_idx].source_idx =
                        rng.gen_range(0..g.internal_nodes.len() as u16);
                    return Ok(());
                }
                pick -= 1;
            }
        }
    }
    Ok(())
}

/// Return a random GraphNodeKind covering all 30 variants with random initial params.
pub(super) fn random_graph_node_kind(input_ref_count: u16, rng: &mut impl Rng) -> GraphNodeKind {
    match rng.gen_range(0u8..30) {
        0 => GraphNodeKind::InputRef {
            ref_idx: rng.gen_range(0..input_ref_count.max(1)),
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
        20 => GraphNodeKind::CustomOutput(rng.gen_range(0..OUTPUT_SLOT_COUNT as u8)),
        21 => GraphNodeKind::RouterOutput,
        22 => GraphNodeKind::WriteActionMeta(rng.gen_range(0..ACTION_META_SLOTS as u8)),
        23 => GraphNodeKind::PushAction(rng.gen_range(0..ACTION_TYPE_COUNT)),
        24 => GraphNodeKind::PopAction,
        25 => GraphNodeKind::ExecuteActionQueue,
        26 => GraphNodeKind::ReadSlot(rng.gen_range(0..SHARED_MEMORY_SLOTS as u8)),
        27 => GraphNodeKind::ReadSlotPrev(rng.gen_range(0..SHARED_MEMORY_SLOTS as u8)),
        28 => GraphNodeKind::WriteSlot(rng.gen_range(0..SHARED_MEMORY_SLOTS as u8)),
        _ => GraphNodeKind::ClearSlot(rng.gen_range(0..SHARED_MEMORY_SLOTS as u8)),
    }
}

/// Returns true if the kind has a mutable parameter.
pub(super) fn is_parameterized(kind: &GraphNodeKind) -> bool {
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

pub(super) fn apply_copy_internal_node(
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
        // Always add backlink edge from random existing node to the copy.
        let target = rng.gen_range(0..new_idx);
        g.internal_nodes[target].inputs.push(GraphInput {
            source_idx: new_idx as u16,
            weight: rng.gen_range(-1.0f32..=1.0),
        });
    }
    Ok(())
}

pub(super) fn apply_copy_subgraph(
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
        // Build old->new index map.
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

pub(super) fn apply_copy_edge_bundle(
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
