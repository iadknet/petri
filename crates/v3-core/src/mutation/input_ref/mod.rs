use rand::seq::SliceRandom;
use rand::Rng;

use crate::config::{MutationConfig, OrdinaryFoodTypeId};
use crate::contracts::votes::{VoteSink, VOTE_SINK_COUNT};
use crate::contracts::{
    InputReference, SensorContext, SensorDescriptor, WorldInputKey, OUTPUT_SLOT_COUNT,
};
use crate::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind};
use crate::genome::{input_ref_kind, BackendDef, CreatureGenome, NodeGenome};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::sampling;
use crate::mutation::types::{BirthEdgeClass, MutationSkipReason, TargetReachability};

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
    pub fn applies_to(self, node: &NodeGenome, context: &SensorContext) -> bool {
        match self {
            Self::Add => true,
            Self::Prune => !prunable_indices(node).is_empty(),
            Self::Swap => !swappable_indices(node, context).is_empty(),
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
        Self::apply_in(
            genome,
            op,
            targets,
            rng,
            config,
            &crate::config::SimulationConfig::default().sensor_context(1),
        )
    }

    /// Apply an input ref operator under the run's sensor context.
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

    /// [`Self::apply_in`] with the class of the vote-sink
    /// edge an `Add` on a Graph node pushed (T11.F30); the same draws.
    pub(crate) fn apply_classified(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        _config: &MutationConfig,
        context: &SensorContext,
    ) -> Result<(TargetReachability, Option<BirthEdgeClass>), MutationSkipReason> {
        if genome.nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        if op == InputRefOperator::RawFieldMutation {
            return apply_raw_field_mutation(genome, rng, context)
                .map(|reachability| (reachability, None));
        }
        let eligible = Self::applicable_indices(genome, op, context);
        if eligible.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }
        let (node_idx, reachability) = targets
            .select(&eligible, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        let class = if op == InputRefOperator::Add {
            apply_add(&mut genome.nodes[node_idx], rng, context)?
        } else {
            Self::apply_to_node(genome, op, node_idx, rng, context)?;
            None
        };
        Ok((reachability, class))
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

    /// Apply a node-targeted operator (`Add`, `Prune`, `Swap`) to
    /// `node_idx`. `RawFieldMutation` draws over the whole genome's table and
    /// has no node target, so it is not routed here.
    pub(crate) fn apply_to_node(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        node_idx: usize,
        rng: &mut impl Rng,
        context: &SensorContext,
    ) -> Result<(), MutationSkipReason> {
        let node = &mut genome.nodes[node_idx];
        match op {
            InputRefOperator::Add => apply_add(node, rng, context).map(|_| ()),
            InputRefOperator::Prune => apply_prune(node, rng),
            InputRefOperator::Swap => apply_swap(node, rng, context),
            InputRefOperator::RawFieldMutation => {
                unreachable!("RawFieldMutation is genome-wide and never node-targeted")
            }
        }
    }
}

/// Declare a new input reference on the target node through
/// [`NodeGenome::add_sensor`].
///
/// On a VM node the reference is wired into nothing: no `ReadInput` is
/// inserted, nothing else is drawn, and the reference becomes addressable
/// only by the VM operators that construct `ReadInput`, so the event is a
/// neutral growth step.
///
/// On a Graph node the event is a connection (T20.F16, born-connected input
/// declaration): the reference is declared already wired into one of the
/// node's `ActionVote` sinks by [`connect_declared_input`], so it is live and
/// exposed to selection at birth (`docs/reference/v3-mutation-spec.md`'s
/// InputRef domain). Returns the class of the vote sink the born edge lands
/// on (T11.F30). A declaration the node cannot hold skips the event after
/// every draw.
fn apply_add(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
    context: &SensorContext,
) -> Result<Option<BirthEdgeClass>, MutationSkipReason> {
    let new_ref = sampling::random_input_reference(rng, context);
    let width = SensorDescriptor::of(&new_ref, context).width();
    let declared = node.add_sensor(new_ref, &[], context);
    let class = match &mut node.backend_def {
        BackendDef::Graph(def) => connect_declared_input(def, declared.ok(), width, rng),
        BackendDef::Vm(_) => None,
    };
    declared.map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    Ok(class)
}

/// Wire the just-declared reference `ref_idx` into one `ActionVote` sink.
///
/// Three draws in this order: the sub-value uniformly under `width`, the
/// sink uniformly by index over all `VOTE_SINK_COUNT`, and the weight from
/// `AddGraphEdge`'s new-edge law (uniform on [-1, 1]). The draws are always
/// taken; the edge is pushed only when the reference was declared (`ref_idx`
/// is `Some`) and the node carries the drawn sink, otherwise the event stays
/// a plain declaration with the same draw stream. Returns the class of the
/// sink the edge was pushed onto (T11.F30), `None` when none was pushed.
pub(crate) fn connect_declared_input(
    def: &mut CgpGraphBackendDef,
    ref_idx: Option<u16>,
    width: u32,
    rng: &mut impl Rng,
) -> Option<BirthEdgeClass> {
    let width = u16::try_from(width).expect("a declared width below u16::MAX");
    debug_assert!(
        width > 0,
        "every input reference has at least one sub-value"
    );
    let sub_idx = rng.gen_range(0..width);
    let sink = VoteSink::from_index(rng.gen_range(0..VOTE_SINK_COUNT));
    let weight = rng.gen_range(-1.0f32..=1.0);
    let (Some(ref_idx), Some(vote)) = (ref_idx, sink) else {
        return None;
    };
    let surface = def.sink_surface(OutputSinkKind::ActionVote(vote))?;
    def.push_edge(
        surface,
        GraphEdge::new(GraphSource::InputLeaf { ref_idx, sub_idx }, weight),
    );
    BirthEdgeClass::of_sink(vote)
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
