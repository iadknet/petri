use rand::seq::SliceRandom;
use rand::Rng;

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::{InputReference, SensorContext, WorldInputKey, OUTPUT_SLOT_COUNT};
use crate::genome::{input_ref_kind, CreatureGenome, NodeGenome};
use crate::mutation::graph::bundle::{add_sensor_by_law, BirthLaw};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::sampling;
use crate::mutation::types::{EventReport, MutationSkipReason, TargetReachability};

/// Every `InputReference` a genome can carry in a world with
/// `context.food_type_count` ordinary food types: `sampling::INPUT_BUCKETS`
/// expanded over the food types and every upstream slot, deduplicated in
/// table order (T27.F01).
pub(crate) fn input_reference_universe(context: &SensorContext) -> Vec<InputReference> {
    sampling::bucket_references(context.food_type_count)
}

/// The members of `universe` that share `entry`'s kind, `entry` excluded.
fn alternatives_in<'a>(
    universe: &'a [InputReference],
    entry: &'a InputReference,
    context: &'a SensorContext,
) -> impl Iterator<Item = &'a InputReference> + 'a {
    let kind = input_ref_kind(entry, context);
    universe
        .iter()
        .filter(move |candidate| *candidate != entry && input_ref_kind(candidate, context) == kind)
}

/// The other members of `entry`'s kind: what a `Swap` may replace it with.
/// Empty when the entry is the only member of its kind under `context`
/// (every ring, summary, neighbor and queue reference, `CommitCounts`
/// and `PreviousOutcome`; typed food at
/// one food type), so such an entry is not swappable.
#[must_use]
pub fn swap_alternatives(entry: &InputReference, context: &SensorContext) -> Vec<InputReference> {
    alternatives_in(&input_reference_universe(context), entry, context)
        .cloned()
        .collect()
}

/// Indices of the node's swappable entries. The universe is enumerated once
/// per node, not once per entry.
fn swappable_indices(node: &NodeGenome, context: &SensorContext) -> Vec<usize> {
    let universe = input_reference_universe(context);
    node.input_refs
        .iter()
        .enumerate()
        .filter(|(_, entry)| alternatives_in(&universe, entry, context).next().is_some())
        .map(|(idx, _)| idx)
        .collect()
}

/// Indices of the node's entries no consumer addresses
/// ([`NodeGenome::addressed_input_refs`]): what a `Prune` may delete
/// without any consumer losing its sensor.
#[must_use]
pub fn prunable_indices(node: &NodeGenome) -> Vec<usize> {
    let addressed = node.addressed_input_refs();
    (0..node.input_refs.len())
        .filter(|&idx| u16::try_from(idx).is_ok_and(|idx| !addressed.contains(&idx)))
        .collect()
}

/// Input reference mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputRefOperator {
    /// Declare a sampled sensor and wire it whole, every bundle live
    /// (T25.F11).
    AddSensorLive,
    /// Declare a sampled sensor and wire it whole, every bundle silent
    /// (T25.F11).
    AddSensorSilent,
    /// Delete one entry no consumer addresses (T11.F22; formerly `Remove`,
    /// which deleted any entry and dropped its consumers).
    Prune,
    /// Replace one entry with another member of its kind (T11.F22).
    Swap,
    RawFieldMutation,
}

impl InputRefOperator {
    pub const ALL: [Self; 5] = [
        Self::AddSensorLive,
        Self::AddSensorSilent,
        Self::Prune,
        Self::Swap,
        Self::RawFieldMutation,
    ];

    /// Whether the operator has a site on `node` (T13.F03 applicability
    /// predicate; the single source of truth `apply` draws from).
    #[must_use]
    pub fn applies_to(self, node: &NodeGenome, context: &SensorContext) -> bool {
        match self {
            Self::AddSensorLive | Self::AddSensorSilent => true,
            Self::Prune => !prunable_indices(node).is_empty(),
            Self::Swap => !swappable_indices(node, context).is_empty(),
            Self::RawFieldMutation => node.input_refs.iter().any(is_raw_mutable),
        }
    }

    /// The operator's stable key, `<Domain>.<Name>`: benchmark summary
    /// keys, server payloads and the serde form of `MutationOperator`.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::AddSensorLive => "InputRef.AddSensorLive",
            Self::AddSensorSilent => "InputRef.AddSensorSilent",
            Self::Prune => "InputRef.Prune",
            Self::Swap => "InputRef.Swap",
            Self::RawFieldMutation => "InputRef.RawFieldMutation",
        }
    }

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            // T25.F11: the live and silent pair split `Add`'s 2 evenly.
            Self::AddSensorLive | Self::AddSensorSilent => 1,
            Self::Prune => 2,
            Self::Swap => 4,
            Self::RawFieldMutation => 4,
        }
    }

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::AddSensorLive | Self::AddSensorSilent => ComplexityEffect::Increasing,
            Self::Prune => ComplexityEffect::Decreasing,
            Self::Swap | Self::RawFieldMutation => ComplexityEffect::Neutral,
        }
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
        Self::apply_in(
            genome,
            op,
            targets,
            rng,
            config,
            &crate::config::SimulationConfig::default().sensor_context(1),
        )
    }

    /// [`Self::apply_classified`], reachability only: the tests' entry under
    /// a chosen sensor context.
    #[cfg(test)]
    pub fn apply_in(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<TargetReachability, MutationSkipReason> {
        Self::apply_classified(genome, op, targets, rng, config, context)
            .map(|(reachability, _)| reachability)
    }

    /// Apply an input ref operator under the run's sensor context, with what
    /// the event reports ([`EventReport`]: the vote-sink class of an
    /// `AddSensor*` that made exactly one `Plain` bundle, T11.F30 and T25.F11
    /// invariant 8, and its bundle edits).
    pub(crate) fn apply_classified(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        _config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<(TargetReachability, EventReport), MutationSkipReason> {
        if genome.nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        if op == InputRefOperator::RawFieldMutation {
            return apply_raw_field_mutation(genome, rng, context)
                .map(|reachability| (reachability, EventReport::default()));
        }
        let eligible = Self::applicable_indices(genome, op, context);
        if eligible.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let (node_idx, reachability) = targets
            .select(&eligible, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        let report = Self::apply_to_node(genome, op, node_idx, rng, context)?;
        Ok((reachability, report))
    }

    /// The nodes `op`'s predicate accepts, in index order.
    #[must_use]
    pub(crate) fn applicable_indices(
        genome: &CreatureGenome,
        op: InputRefOperator,
        context: &SensorContext,
    ) -> Vec<usize> {
        genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| op.applies_to(node, context))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Apply a node-targeted operator (`AddSensor*`, `Prune`, `Swap`) to
    /// `node_idx`. `RawFieldMutation` draws over the whole genome's table and
    /// has no node target, so it is not routed here.
    pub(crate) fn apply_to_node(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        node_idx: usize,
        rng: &mut impl Rng,
        context: &SensorContext,
    ) -> Result<EventReport, MutationSkipReason> {
        let node = &mut genome.nodes[node_idx];
        let unreported =
            |result: Result<(), MutationSkipReason>| result.map(|()| EventReport::default());
        match op {
            InputRefOperator::AddSensorLive => apply_add_sensor(node, BirthLaw::Live, rng, context),
            InputRefOperator::AddSensorSilent => {
                apply_add_sensor(node, BirthLaw::Silent, rng, context)
            }
            InputRefOperator::Prune => unreported(apply_prune(node, rng)),
            InputRefOperator::Swap => unreported(apply_swap(node, rng, context)),
            InputRefOperator::RawFieldMutation => {
                unreachable!("RawFieldMutation is genome-wide and never node-targeted")
            }
        }
    }
}

/// `AddSensorLive` / `AddSensorSilent` (T25.F11 invariant 7): one sensor
/// from the 30-bucket sampler under `context` (food-typed where the world
/// has more than one food type), then on a Graph node the birth policy for
/// every field set and one `add_sensor` transaction; on a VM node the
/// declaration alone, wired into nothing (a neutral growth step).
fn apply_add_sensor(
    node: &mut NodeGenome,
    law: BirthLaw,
    rng: &mut impl Rng,
    context: &SensorContext,
) -> Result<EventReport, MutationSkipReason> {
    let reference = sampling::random_input_reference(rng, context);
    let (bundle_edits, class) = add_sensor_by_law(node, reference, law, context, rng)?;
    Ok(EventReport {
        class,
        bundle_edits,
        ..EventReport::default()
    })
}

/// Delete one entry drawn uniformly among those no consumer addresses
/// ([`NodeGenome::prune`] renumbers the higher indices). Every consumer
/// resolves to the same `InputReference` afterwards and none is removed or
/// rewritten, so the event is neutral at birth.
fn apply_prune(node: &mut NodeGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    let ref_idx = *prunable_indices(node)
        .choose(rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let ref_idx = u16::try_from(ref_idx).expect("a prunable index fits u16");
    node.prune(ref_idx)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)
}

/// Replace one entry, drawn uniformly among the node's swappable entries,
/// with another member of its kind drawn uniformly from the others
/// ([`NodeGenome::swap`]). The width is unchanged by construction, so no
/// edge falls out of range.
fn apply_swap(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
    context: &SensorContext,
) -> Result<(), MutationSkipReason> {
    let ref_idx = *swappable_indices(node, context)
        .choose(rng)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let new_ref = swap_alternatives(&node.input_refs[ref_idx], context)
        .choose(rng)
        .cloned()
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let ref_idx = u16::try_from(ref_idx).map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    node.swap(ref_idx, new_ref, context)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)
}

/// One raw-mutable entry, uniformly over the genome's table in node then
/// index order, replaced through [`NodeGenome::swap`]: an `UpstreamSlot`
/// by a fresh slot draw, a food-typed key by another food type.
fn apply_raw_field_mutation(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
    context: &SensorContext,
) -> Result<TargetReachability, MutationSkipReason> {
    let sites: Vec<(usize, usize)> = genome
        .nodes
        .iter()
        .enumerate()
        .flat_map(|(node_idx, node)| {
            node.input_refs
                .iter()
                .enumerate()
                .filter(|(_, reference)| is_raw_mutable(reference))
                .map(move |(ref_idx, _)| (node_idx, ref_idx))
        })
        .collect();
    if sites.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (node_idx, ref_idx) = sites[rng.gen_range(0..sites.len())];
    let node = &mut genome.nodes[node_idx];
    let replacement = match &node.input_refs[ref_idx] {
        InputReference::UpstreamSlot(_) => {
            InputReference::UpstreamSlot(rng.gen_range(0..OUTPUT_SLOT_COUNT))
        }
        other => {
            let mut replacement = other.clone();
            let type_idx = food_type_idx_mut(&mut replacement).expect("a raw-mutable food key");
            *type_idx = sample_new_food_type_idx(*type_idx, context.food_type_count, rng);
            replacement
        }
    };
    let ref_idx = u16::try_from(ref_idx).map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    node.swap(ref_idx, replacement, context)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
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
