//! The bundle operators (T25.F03, T25.F08, T25.F11, T25.F12): the one birth
//! policy behind the four creation operators, four that change an existing
//! bundle through its record, and one that detaches a member.

use rand::Rng;

use crate::contracts::votes::{DirectedVoteKind, VOTE_DIRECTION_COUNT};
use crate::contracts::{FieldSet, InputReference, SensorContext, SensorDescriptor, MEMORY_BANK};
use crate::genome::cgp::{
    self, BundleBirth, BundleId, BundleSource, CgpGraphBackendDef, LearningSlot, Wiring,
};
use crate::genome::{BackendDef, Learning, MemberPolicy, NodeGenome};
use crate::mutation::event::{wiring_index, BirthEdgeClass, BundleEdits, MutationSkipReason};

use super::hebbian::{draw_hebbian_config, draw_modulation, is_member, select_eligible, Target};
use super::operators::perturb_weight;

/// How a creation operator draws its new bundle's member policies and
/// `credits` (T25.F08, T25.F12): `Uniform` one policy and one coin for every
/// member, `Randomized` a policy and a coin per member. Both draw the two
/// slots first; neither draws the pattern or the scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreationForm {
    Uniform,
    Randomized,
}

/// One learning mode `gen_range(0..3u8)` (0 static, 1 Hebbian, 2
/// reward-modulated) and its config: a learning mode draws one config by
/// `EnableHebbian`'s law, then `EnableRewardModulation`'s for mode 2.
fn draw_mode(rng: &mut impl Rng) -> Option<Learning> {
    let mode = rng.gen_range(0..3u8);
    (mode > 0).then(|| {
        let mut config = draw_hebbian_config(rng);
        if mode == 2 {
            config.modulation = Some(draw_modulation(rng));
        }
        config
    })
}

/// A member policy draw (T25.F12 invariant 10): `gen_bool(0.5)` true
/// `Inherit`, else [`draw_mode`]: absent `Static`, present `Own`.
fn draw_policy(rng: &mut impl Rng) -> MemberPolicy {
    if rng.gen_bool(0.5) {
        MemberPolicy::Inherit
    } else {
        draw_mode(rng).map_or(MemberPolicy::Static, MemberPolicy::Own)
    }
}

/// Learned creation (T25.F12 invariant 10), drawn after a new bundle's
/// shape draws: `scale_learning` then `member_default` by [`draw_mode`];
/// then `Uniform` one policy and one `credits` coin `gen_bool(0.5)` written
/// to every member, `Randomized` per member in canonical order a policy
/// then a coin. Modifiers stay 0.
fn draw_learning(
    def: &mut CgpGraphBackendDef,
    id: BundleId,
    form: CreationForm,
    rng: &mut impl Rng,
) {
    let scale_learning = draw_mode(rng);
    let member_default = draw_mode(rng);
    let mut record = def.bundle_mut(id);
    record.set_slot(LearningSlot::Scale, scale_learning);
    record.set_slot(LearningSlot::MemberDefault, member_default);
    let members: Vec<_> = def.members(id).collect();
    let uniform = (form == CreationForm::Uniform).then(|| (draw_policy(rng), rng.gen_bool(0.5)));
    for member in members {
        let (policy, credits) = uniform.unwrap_or_else(|| (draw_policy(rng), rng.gen_bool(0.5)));
        let mut edge = def.edge_mut(member);
        edge.set_policy(policy);
        edge.set_credits(credits);
    }
}

/// The birth law of a creation operator (T25.F11): a live bundle's scale is
/// one `gen_range(-1.0..=1.0)` draw, a silent bundle's is 0 with no draw.
/// Live and silent are separate operators, never a draw inside one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BirthLaw {
    Live,
    Silent,
}

/// The one birth policy's plan draws (T25.F11 invariant 6): for each field
/// set in order, its wiring among `plans` — a `Plain` set's sink one
/// `gen_range` over its admissible sinks in catalog order; a `Ring` or
/// `Pair` set's kind one `gen_range` over the hostable kinds in
/// `DirectedVoteKind::ALL` order, then a `gen_range(0..8)` offset — and its
/// scale by `law`. A set with no admissible plan skips the event before its
/// draws (the earlier sets' draws already taken).
pub(crate) fn draw_births(
    def: &CgpGraphBackendDef,
    sets: &[(u8, FieldSet)],
    law: BirthLaw,
    rng: &mut impl Rng,
) -> Result<Vec<BundleBirth>, MutationSkipReason> {
    sets.iter()
        .map(|&(field_set, set)| {
            let admissible = cgp::plans(def, &set);
            if admissible.is_empty() {
                return Err(MutationSkipReason::NoApplicableTarget);
            }
            let wiring = match set {
                FieldSet::Plain { .. } => admissible[rng.gen_range(0..admissible.len())],
                FieldSet::Ring | FieldSet::Pair(_) => {
                    let kinds: Vec<DirectedVoteKind> = DirectedVoteKind::ALL
                        .into_iter()
                        .filter(|&kind| admissible.iter().any(|plan| plan.kind() == Some(kind)))
                        .collect();
                    let kind = kinds[rng.gen_range(0..kinds.len())];
                    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
                    match set {
                        FieldSet::Ring => Wiring::Ring { kind, offset },
                        _ => Wiring::DirectionPair { kind, offset },
                    }
                }
            };
            let scale = match law {
                BirthLaw::Live => rng.gen_range(-1.0f32..=1.0),
                BirthLaw::Silent => 0.0,
            };
            Ok(BundleBirth {
                field_set,
                wiring,
                scale,
            })
        })
        .collect()
}

/// The birth policy's learning draws (T25.F11 invariant 6), after every
/// bundle of `ids` is materialized: per bundle in order, the form coin
/// `gen_bool(0.5)` (true `Randomized`), then [`draw_learning`] in that
/// form. Returns the creations, by each bundle's wiring and form.
pub(crate) fn draw_birth_learning(
    def: &mut CgpGraphBackendDef,
    ids: &[BundleId],
    rng: &mut impl Rng,
) -> BundleEdits {
    let mut edits = BundleEdits::default();
    for &id in ids {
        let wiring = def.bundles()[id.index()].wiring();
        let randomized = rng.gen_bool(0.5);
        let form = if randomized {
            CreationForm::Randomized
        } else {
            CreationForm::Uniform
        };
        draw_learning(def, id, form, rng);
        edits.created[wiring_index(&wiring)][usize::from(randomized)] += 1;
    }
    edits
}

/// The edge class a creation reports (T11.F30, T25.F11 invariant 8): the
/// class of the vote sink when it made exactly one bundle and that bundle is
/// `Plain`; `None` otherwise.
pub(crate) fn creation_class(wirings: &[Wiring]) -> Option<BirthEdgeClass> {
    match wirings {
        [Wiring::Plain { sink }] => BirthEdgeClass::of_sink(*sink),
        _ => None,
    }
}

/// The node's admissible bundle sources for `AddBundle*`, in order (T25.F11
/// invariant 7): declared sensors by `ref_idx` (at an index representable
/// as `u16`), field sets ascending, then `Memory { previous: false }` and
/// `Memory { previous: true }`; each when its set has an admissible plan.
/// Lazy, so an applicability check stops at the first source.
pub(crate) fn bundle_sources<'a>(
    def: &'a CgpGraphBackendDef,
    input_refs: &'a [InputReference],
    context: &'a SensorContext,
) -> impl Iterator<Item = (BundleSource, FieldSet)> + 'a {
    let sensors = input_refs
        .iter()
        .take(usize::from(u16::MAX) + 1)
        .enumerate()
        .flat_map(|(idx, reference)| {
            let ref_idx = u16::try_from(idx).expect("index within u16");
            SensorDescriptor::of(reference, context)
                .indexed_field_sets()
                .map(move |(field_set, fields)| {
                    (BundleSource::Sensor { ref_idx, field_set }, fields)
                })
        });
    let banks = [false, true].map(|previous| (BundleSource::Memory { previous }, MEMORY_BANK));
    sensors
        .chain(banks)
        .filter(|(_, set)| !cgp::plans(def, set).is_empty())
}

/// `AddBundleLive` / `AddBundleSilent` (T25.F11 invariant 7): one
/// `gen_range` over the node's admissible sources ([`bundle_sources`]),
/// then the birth policy for that one set under `law`
/// ([`draw_births`], [`NodeGenome::add_bundle`],
/// [`draw_birth_learning`]). Duplicates are allowed.
pub(crate) fn add_bundle_by_law(
    node: &mut NodeGenome,
    law: BirthLaw,
    context: &SensorContext,
    rng: &mut impl Rng,
) -> Result<(BundleEdits, Option<BirthEdgeClass>), MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let sources: Vec<_> = bundle_sources(def, input_refs, context).collect();
    if sources.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (source, set) = sources[rng.gen_range(0..sources.len())];
    let births = draw_births(def, &[(source.field_set(), set)], law, rng)?;
    let id = node
        .add_bundle(source, births[0], context)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    let (def, _) = graph_node(node)?;
    let edits = draw_birth_learning(def, &[id], rng);
    Ok((edits, creation_class(&[births[0].wiring])))
}

/// `AddSensorLive` / `AddSensorSilent` on a node, after the sensor draw
/// (T25.F11 invariant 7): on a Graph node the birth policy for every field
/// set of `reference` in order, then [`NodeGenome::add_sensor`] with every
/// birth, then the learning draws per bundle; on a VM node the declaration
/// alone. A set without an admissible plan skips before its draws; an
/// `Err` from `add_sensor` skips after them.
pub(crate) fn add_sensor_by_law(
    node: &mut NodeGenome,
    reference: InputReference,
    law: BirthLaw,
    context: &SensorContext,
    rng: &mut impl Rng,
) -> Result<(BundleEdits, Option<BirthEdgeClass>), MutationSkipReason> {
    let BackendDef::Graph(def) = node.backend_def() else {
        node.add_sensor(reference, &[], context)
            .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
        return Ok((BundleEdits::default(), None));
    };
    let sets: Vec<(u8, FieldSet)> = SensorDescriptor::of(&reference, context)
        .indexed_field_sets()
        .collect();
    let births = draw_births(def, &sets, law, rng)?;
    let first = def.bundles().len();
    node.add_sensor(reference, &births, context)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    let (def, _) = graph_node(node)?;
    let ids: Vec<BundleId> = def
        .indexed_bundles()
        .skip(first)
        .map(|(id, _)| id)
        .collect();
    let edits = draw_birth_learning(def, &ids, rng);
    let wirings: Vec<Wiring> = births.iter().map(|birth| birth.wiring).collect();
    Ok((edits, creation_class(&wirings)))
}

/// Whether the unit operators have a site: at least one bundle.
pub(super) fn has_bundle(def: &CgpGraphBackendDef) -> bool {
    !def.bundles().is_empty()
}

fn graph_node(
    node: &mut NodeGenome,
) -> Result<(&mut CgpGraphBackendDef, &[InputReference]), MutationSkipReason> {
    match node.backend_edit() {
        (input_refs, BackendDef::Graph(def)) => Ok((def, input_refs)),
        (_, BackendDef::Vm(_)) => Err(MutationSkipReason::NoApplicableTarget),
    }
}

/// `DetachGraphEdge` (T25.F08): one member uniformly in canonical order,
/// one draw and no other, then the F03 detach ([`cgp::detach`]): its own
/// weight becomes `compose(own, scale, pattern)` bitwise, it leaves its
/// bundle with its effective learning (T25.F12 invariant 8), its seed
/// composes (T25.F07 invariant 6; every member alike, T25.F11 invariant 4),
/// and a last member's detach removes the bundle.
pub(crate) fn detach_member(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
) -> Result<BundleEdits, MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let Target::Edge(picked) = select_eligible(def, is_member, rng)? else {
        return Err(MutationSkipReason::NoApplicableTarget);
    };
    let (_, removed) = cgp::detach(def, picked, input_refs);
    Ok(BundleEdits::detach(removed))
}

/// A bundle-record predicate.
pub(super) type BundlePredicate = fn(&cgp::Bundle) -> bool;

/// Whether any bundle record matches `predicate`.
pub(super) fn any_bundle(def: &CgpGraphBackendDef, predicate: BundlePredicate) -> bool {
    def.bundles().iter().any(predicate)
}

/// One bundle uniformly among the records `predicate` admits, in `BundleId`
/// order: count-then-select, one draw.
fn pick_where(
    def: &CgpGraphBackendDef,
    predicate: BundlePredicate,
    rng: &mut impl Rng,
) -> Result<BundleId, MutationSkipReason> {
    let matching = || {
        def.indexed_bundles()
            .filter(|(_, bundle)| predicate(bundle))
    };
    let count = matching().count();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let pick = rng.gen_range(0..count);
    let (id, _) = matching()
        .nth(pick)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    Ok(id)
}

/// The first draw of every unit operator: one bundle, uniformly.
fn pick(def: &CgpGraphBackendDef, rng: &mut impl Rng) -> Result<BundleId, MutationSkipReason> {
    pick_where(def, |_| true, rng)
}

/// `AlterBundleScale`: `AlterGraphEdgeWeight`'s law on the bundle's scale.
pub(crate) fn alter_scale(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let mut scale = def.bundles()[id.index()].scale();
    perturb_weight(&mut scale, rng);
    def.bundle_mut(id).set_scale(scale);
    Ok(())
}

/// Whether a bundle is wired geometrically (`Ring`, `DirectionPair`): the
/// records `ShiftBundleOffset` rotates.
pub(super) fn is_geometric(bundle: &cgp::Bundle) -> bool {
    bundle.wiring().offset().is_some()
}

/// `ShiftBundleOffset` (geometric bundles only): one geometric bundle, then
/// `gen_bool(0.5)` true shifts +1, false −1, mod 8; every member moves to
/// its new site by coordinate.
pub(crate) fn shift_offset(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, is_geometric, rng)?;
    let n = VOTE_DIRECTION_COUNT;
    let step = if rng.gen_bool(0.5) { 1 } else { n - 1 };
    let wiring = def.bundles()[id.index()].wiring();
    let (Some(kind), Some(offset)) = (wiring.kind(), wiring.offset()) else {
        unreachable!("a geometric bundle");
    };
    def.relocate_bundle(id, wiring.rotated(kind, (offset + step) % n));
    Ok(())
}

/// `RetargetBundle` (T25.F11 invariant 7): one bundle; a `Plain` bundle
/// moves to another present vote sink, `gen_range` over the others in
/// catalog order (`NoApplicableTarget` before that draw when there is
/// none); a geometric bundle to another kind, `gen_range(0..3)` over
/// `DirectedVoteKind::ALL` minus its own, skipping after the draw when the
/// graph lacks a site the new kind needs. Every member moves by coordinate.
pub(crate) fn retarget(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let wiring = def.bundles()[id.index()].wiring();
    let target = match wiring {
        Wiring::Plain { sink } => {
            let others: Vec<Wiring> = cgp::plans(def, &MEMORY_BANK)
                .into_iter()
                .filter(|plan| *plan != Wiring::Plain { sink })
                .collect();
            if others.is_empty() {
                return Err(MutationSkipReason::NoApplicableTarget);
            }
            others[rng.gen_range(0..others.len())]
        }
        Wiring::Ring { kind, offset } | Wiring::DirectionPair { kind, offset } => {
            let pick = rng.gen_range(0..DirectedVoteKind::ALL.len() - 1);
            let other = DirectedVoteKind::ALL
                .into_iter()
                .filter(|&candidate| candidate != kind)
                .nth(pick)
                .expect("another kind");
            wiring.rotated(other, offset)
        }
    };
    if !cgp::hosts(def, &target) {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    def.relocate_bundle(id, target);
    Ok(())
}

/// `RemoveBundle`: the record and every member.
pub(crate) fn remove(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<BundleEdits, MutationSkipReason> {
    let id = pick(def, rng)?;
    def.remove_bundle(id);
    Ok(BundleEdits::removal(true))
}
