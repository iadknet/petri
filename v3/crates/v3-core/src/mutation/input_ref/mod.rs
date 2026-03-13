use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::{CreatureGenome, VmInstruction};
use crate::mutation::compound::sub_value_count;
use crate::mutation::reachability::biased_select_from;
use crate::mutation::sampling;
use crate::mutation::types::{MutationSkipReason, TargetReachability};
use crate::runtime::OUTPUT_SLOT_COUNT;

/// Input reference mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputRefOperator {
    Add,
    Remove,
    Swap,
    RawFieldMutation,
}

impl InputRefOperator {
    pub const ALL: [Self; 4] = [Self::Add, Self::Remove, Self::Swap, Self::RawFieldMutation];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::Add => 2,
            Self::Remove => 2,
            Self::Swap => 4,
            Self::RawFieldMutation => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 4,
            "ALL must cover every InputRefOperator variant"
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
            Self::Add => ComplexityEffect::Increasing,
            Self::Remove => ComplexityEffect::Decreasing,
            Self::Swap | Self::RawFieldMutation => ComplexityEffect::Neutral,
        }
    }

    /// Pick a random input ref operator weighted by impact tier.
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

/// Input reference domain mutator.
pub struct InputRefMutator;

impl InputRefMutator {
    /// Apply an input ref operator to the genome.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        if genome.nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        match op {
            InputRefOperator::Add => apply_add(genome, reachable_nodes, bias, rng, config),
            InputRefOperator::Remove => apply_remove(genome, reachable_nodes, bias, rng),
            InputRefOperator::Swap => apply_swap(genome, reachable_nodes, bias, rng, config),
            InputRefOperator::RawFieldMutation => apply_raw_field_mutation(genome, rng, config),
        }
    }
}

fn apply_add(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    use crate::creature::genome::cgp::{GraphEdge, GraphSource};
    use crate::creature::genome::BackendDef;
    use crate::mutation::graph::operators::{get_edge_vec_mut, pick_random_surface};

    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (node_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let new_ref = sampling::random_input_reference(rng);
    genome.nodes[node_idx].input_refs.push(new_ref);

    // Compute width before mutable borrow of backend_def (own-borrow-over-clone)
    let new_ref_idx = (genome.nodes[node_idx].input_refs.len() - 1) as u16;
    let width = sub_value_count(
        &genome.nodes[node_idx].input_refs[new_ref_idx as usize],
        config,
    );

    // Auto-wire into the backend so the new input is immediately connected.
    match genome.nodes[node_idx].backend_def {
        BackendDef::Graph(ref mut def) => {
            // Wire all sub-indices into random graph surfaces.
            for sub_idx in 0..width {
                if let Some(surface) = pick_random_surface(def, rng) {
                    let edges = get_edge_vec_mut(def, surface);
                    edges.push(GraphEdge {
                        source: GraphSource::InputLeaf {
                            ref_idx: new_ref_idx,
                            sub_idx,
                        },
                        weight: rng.gen_range(-1.0f32..=1.0),
                    });
                }
            }
        }
        BackendDef::Vm(ref mut vm) => {
            // Insert a ReadInput instruction so the new ref is actually read.
            // Without this, the ref sits disconnected — no VM instruction
            // references it — creating junk DNA that wastes complexity budget.
            let rc = vm.register_count.max(1);
            let dst = rng.gen_range(0..rc);
            let sub_idx = if width > 1 {
                rng.gen_range(0..width)
            } else {
                0
            };
            let read = VmInstruction::ReadInput {
                dst,
                ref_idx: new_ref_idx,
                sub_idx,
            };
            let pos = rng.gen_range(0..=vm.program.len());
            vm.program.insert(pos, read);
        }
    }

    Ok(reachability)
}

fn apply_remove(
    genome: &mut CreatureGenome,
    reachable_nodes: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Result<TargetReachability, MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.input_refs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let ref_idx = rng.gen_range(0..genome.nodes[node_idx].input_refs.len());
    genome.nodes[node_idx].input_refs.remove(ref_idx);
    debug_assert!(ref_idx <= u16::MAX as usize, "input_refs index exceeds u16");
    genome.nodes[node_idx]
        .backend_def
        .reindex_input_refs_after_removal(ref_idx as u16);
    Ok(reachability)
}

fn apply_swap(
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
        .filter(|(_, n)| !n.input_refs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let ref_idx = rng.gen_range(0..genome.nodes[node_idx].input_refs.len());
    let new_ref = sampling::random_input_reference(rng);
    genome.nodes[node_idx].input_refs[ref_idx] = new_ref;
    let new_width = sub_value_count(&genome.nodes[node_idx].input_refs[ref_idx], config);
    debug_assert!(ref_idx <= u16::MAX as usize, "input_refs index exceeds u16");
    genome.nodes[node_idx]
        .backend_def
        .clamp_sub_idx_after_swap(ref_idx as u16, new_width);
    Ok(reachability)
}

fn apply_raw_field_mutation(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
    _config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    // Count eligible targets: UpstreamSlot input_refs only.
    // (The old second pool — InputRef graph nodes for sub_idx mutation — no longer
    // exists in the CGP graph backend. CGP edges use GraphSource::InputLeaf and
    // sub_idx is mutated via mutation/graph/operators.rs.)
    let mut upstream_count: usize = 0;
    for node in &genome.nodes {
        upstream_count += node
            .input_refs
            .iter()
            .filter(|r| matches!(r, InputReference::UpstreamSlot(_)))
            .count();
    }
    if upstream_count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let mut pick = rng.gen_range(0..upstream_count);

    for node in &mut genome.nodes {
        for input_ref in &mut node.input_refs {
            if matches!(input_ref, InputReference::UpstreamSlot(_)) {
                if pick == 0 {
                    *input_ref = InputReference::UpstreamSlot(rng.gen_range(0..OUTPUT_SLOT_COUNT));
                    return Ok(TargetReachability::NotApplicable);
                }
                pick -= 1;
            }
        }
    }
    Ok(TargetReachability::NotApplicable)
}

#[cfg(test)]
mod tests;
