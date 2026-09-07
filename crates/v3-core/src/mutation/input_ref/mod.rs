use rand::Rng;

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::genome::CreatureGenome;
use crate::mutation::compound::sub_value_count;
use crate::mutation::reachability::TargetSelector;
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
    #[cfg(test)]
    pub fn apply(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        Self::apply_with_food_type_count(genome, op, targets, rng, config, 1)
    }

    /// Apply an input ref operator with typed-food mutation context.
    pub fn apply_with_food_type_count(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
        food_type_count: usize,
    ) -> Result<TargetReachability, MutationSkipReason> {
        if genome.nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        match op {
            InputRefOperator::Add => apply_add(genome, targets, rng, food_type_count),
            InputRefOperator::Remove => apply_remove(genome, targets, rng),
            InputRefOperator::Swap => apply_swap(genome, targets, rng, config, food_type_count),
            InputRefOperator::RawFieldMutation => {
                apply_raw_field_mutation(genome, rng, config, food_type_count)
            }
        }
    }
}

/// Push a new input reference onto the target node. It is wired into
/// nothing on either backend: a fresh `InputReference` is a growth step, not
/// a connection, so it must be neutral at the moment it fires
/// (`docs/reference/v3-mutation-spec.md`'s node-type contract). The
/// reference becomes addressable by later connection operators
/// (`AddGraphEdge`, `RetargetGraphEdge` on the graph backend; VM operators
/// that reference `input_refs` on the VM backend).
fn apply_add(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    food_type_count: usize,
) -> Result<TargetReachability, MutationSkipReason> {
    let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
    let (node_idx, reachability) = targets
        .select(&all_indices, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let new_ref = if food_type_count <= 1 {
        sampling::random_input_reference(rng)
    } else {
        sampling::random_input_reference_for_food_types(rng, food_type_count)
    };
    genome.nodes[node_idx].input_refs.push(new_ref);
    Ok(reachability)
}

fn apply_remove(
    genome: &mut CreatureGenome,
    targets: &mut TargetSelector<'_>,
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
    let (node_idx, reachability) = targets
        .select(&eligible, rng)
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
    targets: &mut TargetSelector<'_>,
    rng: &mut impl Rng,
    config: &MutationConfig,
    food_type_count: usize,
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
    let (node_idx, reachability) = targets
        .select(&eligible, rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let ref_idx = rng.gen_range(0..genome.nodes[node_idx].input_refs.len());
    let new_ref = if food_type_count <= 1 {
        sampling::random_input_reference(rng)
    } else {
        sampling::random_input_reference_for_food_types(rng, food_type_count)
    };
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
    food_type_count: usize,
) -> Result<TargetReachability, MutationSkipReason> {
    // Count eligible targets: UpstreamSlot refs + typed food world refs.
    let mut upstream_count = 0usize;
    let mut food_ref_count = 0usize;
    for node in &genome.nodes {
        upstream_count += node
            .input_refs
            .iter()
            .filter(|r| matches!(r, InputReference::UpstreamSlot(_)))
            .count();
        food_ref_count += node
            .input_refs
            .iter()
            .filter(|r| food_type_idx_ref(r).is_some())
            .count();
    }
    let total = upstream_count + food_ref_count;
    if total == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let mut pick = rng.gen_range(0..total);

    for node in &mut genome.nodes {
        for input_ref in &mut node.input_refs {
            if matches!(input_ref, InputReference::UpstreamSlot(_)) {
                if pick == 0 {
                    *input_ref = InputReference::UpstreamSlot(rng.gen_range(0..OUTPUT_SLOT_COUNT));
                    return Ok(TargetReachability::NotApplicable);
                }
                pick -= 1;
                continue;
            }

            if let Some(type_idx) = food_type_idx_mut(input_ref) {
                if pick == 0 {
                    *type_idx = sample_new_food_type_idx(*type_idx, food_type_count, rng);
                    return Ok(TargetReachability::NotApplicable);
                }
                pick -= 1;
            }
        }
    }
    Ok(TargetReachability::NotApplicable)
}

fn food_type_idx_ref(input_ref: &InputReference) -> Option<OrdinaryFoodTypeId> {
    match input_ref {
        InputReference::World(WorldInputKey::FoodHere { type_idx })
        | InputReference::World(WorldInputKey::NeighborFoodRing { type_idx })
        | InputReference::World(WorldInputKey::AreaFoodSummary { type_idx }) => Some(*type_idx),
        _ => None,
    }
}

fn food_type_idx_mut(input_ref: &mut InputReference) -> Option<&mut OrdinaryFoodTypeId> {
    match input_ref {
        InputReference::World(WorldInputKey::FoodHere { type_idx })
        | InputReference::World(WorldInputKey::NeighborFoodRing { type_idx })
        | InputReference::World(WorldInputKey::AreaFoodSummary { type_idx }) => Some(type_idx),
        _ => None,
    }
}

fn sample_new_food_type_idx(
    current: OrdinaryFoodTypeId,
    food_type_count: usize,
    rng: &mut impl Rng,
) -> OrdinaryFoodTypeId {
    let capped = food_type_count.clamp(1, usize::from(u16::MAX) + 1);
    if capped == 1 {
        return OrdinaryFoodTypeId::default();
    }

    let current_idx = usize::from(current.get());
    if current_idx >= capped {
        return OrdinaryFoodTypeId::new(rng.gen_range(0..capped) as u16);
    }

    let draw = rng.gen_range(0..(capped - 1));
    let remapped = if draw >= current_idx { draw + 1 } else { draw };
    OrdinaryFoodTypeId::new(remapped as u16)
}

#[cfg(test)]
mod tests;
