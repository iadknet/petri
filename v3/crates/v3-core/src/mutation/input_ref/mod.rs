use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::creature::genome::{BackendDef, CreatureGenome, GraphNodeKind, VmInstruction};
use crate::mutation::compound;
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

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

/// After removing `input_refs[removed_ref_idx]` from `genome.nodes[node_idx]`,
/// update all internal references (graph InputRef nodes and VM ReadInput instructions)
/// so they stay consistent:
/// - `ref_idx == removed_ref_idx` → set to `u16::MAX` (invalidated → resolves to 0.0)
/// - `ref_idx > removed_ref_idx` → decrement by 1
/// - `ref_idx < removed_ref_idx` → unchanged
///
/// `sub_idx` is unaffected — it indexes within a compound input, not across `input_refs`.
pub(crate) fn reindex_after_removal(
    genome: &mut CreatureGenome,
    node_idx: usize,
    removed_ref_idx: u16,
) {
    debug_assert!(node_idx < genome.nodes.len(), "node_idx out of bounds");
    let node = &mut genome.nodes[node_idx];
    match &mut node.backend_def {
        BackendDef::Graph(gd) => {
            for internal in &mut gd.internal_nodes {
                if let GraphNodeKind::InputRef { ref_idx, .. } = &mut internal.kind {
                    if *ref_idx == removed_ref_idx {
                        *ref_idx = u16::MAX;
                    } else if *ref_idx > removed_ref_idx {
                        *ref_idx -= 1;
                    }
                }
            }
        }
        BackendDef::Vm(vm) => {
            for instr in &mut vm.program {
                if let VmInstruction::ReadInput { ref_idx, .. } = instr {
                    if *ref_idx == removed_ref_idx {
                        *ref_idx = u16::MAX;
                    } else if *ref_idx > removed_ref_idx {
                        *ref_idx -= 1;
                    }
                }
            }
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
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (node_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let new_ref = random_input_reference(rng);
    let count = compound::sub_value_count(&new_ref, config);
    let ref_idx = genome.nodes[node_idx].input_refs.len() as u16;
    genome.nodes[node_idx].input_refs.push(new_ref);
    compound::create_and_connect_input_leaves(
        &mut genome.nodes[node_idx],
        ref_idx,
        count,
        config.input_auto_connect_chance,
        rng,
    );
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
    reindex_after_removal(genome, node_idx, ref_idx as u16);
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
    let new_ref = random_input_reference(rng);
    let count = compound::sub_value_count(&new_ref, config);
    genome.nodes[node_idx].input_refs[ref_idx] = new_ref;
    // Swap intentionally uses create_fan_out_nodes (not create_and_connect_input_leaves):
    // - Swap is a neutral mutation; adding bootstrap edges would make it structural.
    // - For scalar→scalar swaps, existing InputRef internal nodes already read
    //   the new input via resolve_input at the same ref_idx.
    // - Compound-widening swaps leave new sub-value leaves disconnected;
    //   a follow-up can address this if it proves problematic.
    if count > 1 {
        compound::create_fan_out_nodes(&mut genome.nodes[node_idx], ref_idx as u16, count);
    }
    Ok(reachability)
}

/// Generate a random input reference from the full set of 23 possible values.
///
/// Distribution: FoodHere (1) + Ring sensors (3) + StaticIntrospection (2) +
/// DynamicIntrospection (2) + ActionQueue (1) + Area summaries (3) +
/// Nearby creature (3) + UpstreamSlot (8 weighted slots) = 23 total.
fn random_input_reference(rng: &mut impl Rng) -> InputReference {
    let idx = rng.gen_range(0u8..23);
    match idx {
        0 => InputReference::World(WorldInputKey::FoodHere),
        1 => InputReference::World(WorldInputKey::NeighborFoodRing),
        2 => InputReference::World(WorldInputKey::NeighborBarrierRing),
        3 => InputReference::World(WorldInputKey::NeighborOccupiedRing),
        4 => InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
        5 => InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        6 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        7 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
        8 => InputReference::ActionQueue,
        9 => InputReference::World(WorldInputKey::AreaFoodSummary),
        10 => InputReference::World(WorldInputKey::AreaBarrierSummary),
        11 => InputReference::World(WorldInputKey::AreaOccupancySummary),
        12 => InputReference::World(WorldInputKey::NearbyCreatureCore),
        13 => InputReference::World(WorldInputKey::NearbyCreatureVitals),
        14 => InputReference::World(WorldInputKey::NearbyCreatureIdentity),
        _ => InputReference::UpstreamSlot(rng.gen_range(0..12_usize)),
    }
}

fn apply_raw_field_mutation(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<TargetReachability, MutationSkipReason> {
    // Count eligible targets: UpstreamSlot input_refs + InputRef graph nodes (for sub_idx mutation).
    let mut upstream_count: usize = 0;
    let mut graph_input_ref_count: usize = 0;
    for node in &genome.nodes {
        upstream_count += node
            .input_refs
            .iter()
            .filter(|r| matches!(r, InputReference::UpstreamSlot(_)))
            .count();
        if let BackendDef::Graph(ref gd) = node.backend_def {
            graph_input_ref_count += gd
                .internal_nodes
                .iter()
                .filter(|n| matches!(n.kind, GraphNodeKind::InputRef { .. }))
                .count();
        }
    }
    let total = upstream_count + graph_input_ref_count;
    if total == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let mut pick = rng.gen_range(0..total);

    // First pool: UpstreamSlot input_refs.
    if pick < upstream_count {
        for node in &mut genome.nodes {
            for input_ref in &mut node.input_refs {
                if matches!(input_ref, InputReference::UpstreamSlot(_)) {
                    if pick == 0 {
                        *input_ref = InputReference::UpstreamSlot(rng.gen_range(0..12_usize));
                        return Ok(TargetReachability::NotApplicable);
                    }
                    pick -= 1;
                }
            }
        }
    }

    // Second pool: InputRef graph node sub_idx mutation.
    pick -= upstream_count;
    for node in &mut genome.nodes {
        if let BackendDef::Graph(ref mut gd) = node.backend_def {
            for internal in &mut gd.internal_nodes {
                if let GraphNodeKind::InputRef { ref_idx, sub_idx } = &mut internal.kind {
                    if pick == 0 {
                        let width = node
                            .input_refs
                            .get(*ref_idx as usize)
                            .map(|r| compound::sub_value_count(r, config))
                            .unwrap_or(1);
                        *sub_idx = rng.gen_range(0..width);
                        return Ok(TargetReachability::NotApplicable);
                    }
                    pick -= 1;
                }
            }
        }
    }
    Ok(TargetReachability::NotApplicable)
}

#[cfg(test)]
mod tests;
