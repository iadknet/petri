use std::collections::BTreeSet;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::creature::genome::cgp::GraphSource;
use crate::creature::genome::mesh_annotations::{classify_input_ref, MeshReadClass};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmInstruction};
use crate::creature::sensor_census::world_input_key_universe;
use crate::mutation::compound::sub_value_count;
use crate::mutation::reachability::TargetSelector;
use crate::mutation::sampling;
use crate::mutation::types::{MutationSkipReason, TargetReachability};
use crate::runtime::OUTPUT_SLOT_COUNT;

/// The kind of an `input_refs` entry: its mesh read class and compound
/// width. A `Swap` only ever replaces an entry with another member of the
/// same kind, so a consumer keeps reading the same modality at the same
/// width for the node's life and its copies' (T11.F22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputRefKind {
    pub class: MeshReadClass,
    pub width: u16,
}

/// The kind partition every within-kind rule reads: the operator, its
/// applicability predicate, and the probes all call this one function.
#[must_use]
pub fn input_ref_kind(reference: &InputReference, config: &MutationConfig) -> InputRefKind {
    InputRefKind {
        class: classify_input_ref(reference),
        width: sub_value_count(reference, config),
    }
}

/// Every `InputReference` a genome can carry in a world with
/// `food_type_count` ordinary food types: the sampling pool of
/// `sampling::random_input_reference_for_food_types`, enumerated.
fn input_reference_universe(food_type_count: usize) -> Vec<InputReference> {
    let capped = food_type_count.clamp(1, usize::from(u16::MAX) + 1);
    let food_types = (0..capped).map(|idx| OrdinaryFoodTypeId::new(idx as u16));
    world_input_key_universe(food_types)
        .into_iter()
        .map(InputReference::World)
        .chain([
            InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            InputReference::ActionQueue,
        ])
        .chain((0..OUTPUT_SLOT_COUNT).map(InputReference::UpstreamSlot))
        .collect()
}

/// The members of `universe` that share `entry`'s kind, `entry` excluded.
fn alternatives_in<'a>(
    universe: &'a [InputReference],
    entry: &'a InputReference,
    config: &'a MutationConfig,
) -> impl Iterator<Item = &'a InputReference> + 'a {
    let kind = input_ref_kind(entry, config);
    universe
        .iter()
        .filter(move |candidate| *candidate != entry && input_ref_kind(candidate, config) == kind)
}

/// The other members of `entry`'s kind: what a `Swap` may replace it with.
/// Empty when the entry is the only member of its kind at this food type
/// count (every ring, summary, neighbor and queue reference; typed food at
/// one food type), so such an entry is not swappable.
#[must_use]
pub fn swap_alternatives(
    entry: &InputReference,
    config: &MutationConfig,
    food_type_count: usize,
) -> Vec<InputReference> {
    alternatives_in(&input_reference_universe(food_type_count), entry, config)
        .cloned()
        .collect()
}

/// Indices of the node's swappable entries. The universe is enumerated once
/// per node, not once per entry.
fn swappable_indices(
    node: &NodeGenome,
    config: &MutationConfig,
    food_type_count: usize,
) -> Vec<usize> {
    let universe = input_reference_universe(food_type_count);
    node.input_refs
        .iter()
        .enumerate()
        .filter(|(_, entry)| alternatives_in(&universe, entry, config).next().is_some())
        .map(|(idx, _)| idx)
        .collect()
}

/// The `input_refs` indices some consumer on the node's backend addresses:
/// a `GraphSource::InputLeaf` on any edge container, or a VM `ReadInput`
/// anywhere in the program, live or not.
fn referenced_indices(node: &NodeGenome) -> BTreeSet<u16> {
    match &node.backend_def {
        BackendDef::Graph(def) => def
            .edges()
            .filter_map(|edge| match edge.source {
                GraphSource::InputLeaf { ref_idx, .. } => Some(ref_idx),
                GraphSource::SharedMemory { .. } | GraphSource::ComputeNode(_) => None,
            })
            .collect(),
        BackendDef::Vm(vm) => vm
            .program
            .iter()
            .filter_map(|instruction| match instruction {
                VmInstruction::ReadInput { ref_idx, .. } => Some(*ref_idx),
                _ => None,
            })
            .collect(),
    }
}

/// Indices of the node's entries no consumer addresses: what a `Prune` may
/// delete without any consumer losing its sensor.
#[must_use]
pub fn prunable_indices(node: &NodeGenome) -> Vec<usize> {
    let referenced = referenced_indices(node);
    (0..node.input_refs.len())
        .filter(|&idx| u16::try_from(idx).is_ok_and(|idx| !referenced.contains(&idx)))
        .collect()
}

/// Input reference mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputRefOperator {
    Add,
    /// Delete one entry no consumer addresses (T11.F22; formerly `Remove`,
    /// which deleted any entry and dropped its consumers).
    Prune,
    /// Replace one entry with another member of its kind (T11.F22).
    Swap,
    RawFieldMutation,
}

impl InputRefOperator {
    pub const ALL: [Self; 4] = [Self::Add, Self::Prune, Self::Swap, Self::RawFieldMutation];

    /// Whether the operator has a site on `node` (T13.F03 applicability
    /// predicate; the single source of truth `apply` draws from).
    #[must_use]
    pub fn applies_to(
        self,
        node: &NodeGenome,
        config: &MutationConfig,
        food_type_count: usize,
    ) -> bool {
        match self {
            Self::Add => true,
            Self::Prune => !prunable_indices(node).is_empty(),
            Self::Swap => !swappable_indices(node, config, food_type_count).is_empty(),
            Self::RawFieldMutation => node.input_refs.iter().any(is_raw_mutable),
        }
    }

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::Add => 2,
            Self::Prune => 2,
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
            Self::Prune => ComplexityEffect::Decreasing,
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

        if op == InputRefOperator::RawFieldMutation {
            return apply_raw_field_mutation(genome, rng, config, food_type_count);
        }
        let eligible = Self::applicable_indices(genome, op, config, food_type_count);
        if eligible.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let (node_idx, reachability) = targets
            .select(&eligible, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        Self::apply_to_node(genome, op, node_idx, rng, config, food_type_count)?;
        Ok(reachability)
    }

    /// The nodes `op`'s predicate accepts, in index order.
    #[must_use]
    pub(crate) fn applicable_indices(
        genome: &CreatureGenome,
        op: InputRefOperator,
        config: &MutationConfig,
        food_type_count: usize,
    ) -> Vec<usize> {
        genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| op.applies_to(node, config, food_type_count))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Apply a node-targeted operator (`Add`, `Prune`, `Swap`) to
    /// `node_idx`. `RawFieldMutation` draws over the whole genome's table and
    /// has no node target, so it is not routed here.
    pub(crate) fn apply_to_node(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        node_idx: usize,
        rng: &mut impl Rng,
        config: &MutationConfig,
        food_type_count: usize,
    ) -> Result<(), MutationSkipReason> {
        let node = &mut genome.nodes[node_idx];
        match op {
            InputRefOperator::Add => {
                apply_add(node, rng, food_type_count);
                Ok(())
            }
            InputRefOperator::Prune => apply_prune(node, rng),
            InputRefOperator::Swap => apply_swap(node, rng, config, food_type_count),
            InputRefOperator::RawFieldMutation => {
                unreachable!("RawFieldMutation is genome-wide and never node-targeted")
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
fn apply_add(node: &mut NodeGenome, rng: &mut impl Rng, food_type_count: usize) {
    let new_ref = if food_type_count <= 1 {
        sampling::random_input_reference(rng)
    } else {
        sampling::random_input_reference_for_food_types(rng, food_type_count)
    };
    node.input_refs.push(new_ref);
}

/// Delete one entry drawn uniformly among those no consumer addresses, then
/// renumber the higher indices. Every consumer resolves to the same
/// `InputReference` afterwards and none is removed or rewritten, so the
/// event is neutral at birth.
fn apply_prune(node: &mut NodeGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    let ref_idx = *prunable_indices(node)
        .choose(rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    node.input_refs.remove(ref_idx);
    let ref_idx = u16::try_from(ref_idx).expect("input_refs index exceeds u16");
    node.backend_def.reindex_input_refs_after_removal(ref_idx);
    Ok(())
}

/// Replace one entry, drawn uniformly among the node's swappable entries,
/// with another member of its kind drawn uniformly from the others. The
/// width is unchanged by construction, so no edge falls out of range and
/// `clamp_sub_idx_after_swap` has nothing to remove.
fn apply_swap(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
    config: &MutationConfig,
    food_type_count: usize,
) -> Result<(), MutationSkipReason> {
    let ref_idx = *swappable_indices(node, config, food_type_count)
        .choose(rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let old_ref = &node.input_refs[ref_idx];
    let new_ref = swap_alternatives(old_ref, config, food_type_count)
        .choose(rng)
        .cloned()
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    debug_assert_eq!(
        sub_value_count(&new_ref, config),
        sub_value_count(old_ref, config),
        "a within-kind swap keeps the width, so clamp_sub_idx_after_swap removes nothing"
    );
    node.input_refs[ref_idx] = new_ref;
    Ok(())
}

fn apply_raw_field_mutation(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
    _config: &MutationConfig,
    food_type_count: usize,
) -> Result<TargetReachability, MutationSkipReason> {
    // Count eligible targets: UpstreamSlot refs + typed food world refs.
    let total = genome
        .nodes
        .iter()
        .flat_map(|node| &node.input_refs)
        .filter(|r| is_raw_mutable(r))
        .count();
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

fn is_raw_mutable(input_ref: &InputReference) -> bool {
    matches!(input_ref, InputReference::UpstreamSlot(_)) || food_type_idx_ref(input_ref).is_some()
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
