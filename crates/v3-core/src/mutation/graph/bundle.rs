//! The edge-bundle operators (T25.F03, T25.F04, T25.F08, T25.F09): four
//! create a silent bundle (`Ring`, `DirectionPair`, each in the uniform and
//! the randomized form), four change an existing bundle through its record,
//! three move its learned scale's retention, and one detaches a member.

use rand::Rng;

use crate::contracts::votes::{DirectedVoteKind, VOTE_DIRECTION_COUNT};
use crate::contracts::{FieldSet, InputReference, SensorContext};
use crate::genome::cgp::{
    self, BundleBirth, BundleId, BundleSource, CgpGraphBackendDef, Scope, Wiring,
};
use crate::genome::{retention_decays, BackendDef, NodeGenome, PlasticityConfig};
use crate::mutation::types::{BundleEdits, MutationSkipReason};

use super::hebbian::{
    draw_hebbian_config, draw_modulation, draw_retention, is_member, select_eligible,
    step_retention,
};
use super::operators::perturb_weight;

/// How a creation operator draws its new bundle's learning attributes
/// (T25.F08): `Uniform` one mode and one scope for the whole bundle (F07),
/// `Randomized` a mode and a scope per member. Both draw one
/// `inherit_scale` for the bundle; neither draws the pattern or the scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreationForm {
    Uniform,
    Randomized,
}

/// One learning mode `gen_range(0..3u8)` (0 static, 1 Hebbian, 2
/// reward-modulated) and its config: a learning mode draws one config by
/// `EnableHebbian`'s law, then `EnableRewardModulation`'s for mode 2.
fn draw_mode(rng: &mut impl Rng) -> Option<PlasticityConfig> {
    let mode = rng.gen_range(0..3u8);
    (mode > 0).then(|| {
        let mut config = draw_hebbian_config(rng);
        if mode == 2 {
            config.modulation = Some(draw_modulation(rng));
        }
        config
    })
}

/// The scope coin: true `Bundle`.
fn draw_scope(rng: &mut impl Rng) -> Scope {
    if rng.gen_bool(0.5) {
        Scope::Group
    } else {
        Scope::Edge
    }
}

/// Learned creation (T25.F07 invariant 7, T25.F08 invariant 2), drawn after
/// a new bundle's shape draws. `Uniform`: one mode with its config, then one
/// scope coin, written to every member. `Randomized`: per member in
/// canonical order, its own mode, config and scope coin. Then one
/// `inherit_scale` coin.
fn draw_learning(
    def: &mut CgpGraphBackendDef,
    id: BundleId,
    form: CreationForm,
    rng: &mut impl Rng,
) {
    let members: Vec<_> = def.members(id).collect();
    let uniform = (form == CreationForm::Uniform).then(|| (draw_mode(rng), draw_scope(rng)));
    for member in members {
        let (config, scope) = uniform.unwrap_or_else(|| (draw_mode(rng), draw_scope(rng)));
        let mut edge = def.edge_mut(member);
        edge.set_plasticity(config);
        edge.set_scope(scope);
    }
    let inherit_scale = rng.gen_bool(0.5);
    def.bundle_mut(id).set_inherit_scale(inherit_scale);
}

// Commit 1 of T25.F11: the birth policy exists and is tested; the four
// creation operators of commit 3 call it.
/// The birth law of a creation operator (T25.F11): a live bundle's scale is
/// one `gen_range(-1.0..=1.0)` draw, a silent bundle's is 0 with no draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "T25.F11 commit 1: called from commit 3")
)]
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
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "T25.F11 commit 1: called from commit 3")
)]
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
                        .filter(|&kind| admissible.iter().any(|plan| plan_kind(plan) == Some(kind)))
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

/// The kind a directed plan votes for.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "T25.F11 commit 1: called from commit 3")
)]
const fn plan_kind(wiring: &Wiring) -> Option<DirectedVoteKind> {
    match *wiring {
        Wiring::Ring { kind, .. } | Wiring::DirectionPair { kind, .. } => Some(kind),
        Wiring::Plain { .. } => None,
    }
}

/// The birth policy's learning draws (T25.F11 invariant 6), after every
/// bundle of `ids` is materialized: per bundle in order, the form coin
/// `gen_bool(0.5)` (true `Randomized`), then [`draw_learning`] in that
/// form. Returns the creations, by wiring and form.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "T25.F11 commit 1: called from commit 3")
)]
pub(crate) fn draw_birth_learning(
    def: &mut CgpGraphBackendDef,
    ids: &[BundleId],
    wirings: &[Wiring],
    rng: &mut impl Rng,
) -> BundleEdits {
    let mut edits = BundleEdits::default();
    for (&id, wiring) in ids.iter().zip(wirings) {
        let randomized = rng.gen_bool(0.5);
        let form = if randomized {
            CreationForm::Randomized
        } else {
            CreationForm::Uniform
        };
        draw_learning(def, id, form, rng);
        let created = BundleEdits::creation(wiring, randomized).created;
        for (row, add) in edits.created.iter_mut().zip(created) {
            for (count, add) in row.iter_mut().zip(add) {
                *count += add;
            }
        }
    }
    edits
}

/// The node's ring indices a bundle can name, ascending: neighbor rings at an
/// index representable as `u16`.
fn ring_indices(input_refs: &[InputReference]) -> impl Iterator<Item = u16> + '_ {
    input_refs
        .iter()
        .take(usize::from(u16::MAX) + 1)
        .enumerate()
        .filter(|(_, reference)| reference.is_ring())
        .map(|(idx, _)| u16::try_from(idx).expect("index within u16"))
}

/// Whether `AddRingBundle` has a site: a ring at an index ≤ 65,535 and a
/// graph that can host a bundle ([`CgpGraphBackendDef::can_host_bundle`]).
pub(super) fn can_add(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> bool {
    def.can_host_bundle() && ring_indices(input_refs).next().is_some()
}

/// The node's declared pairs a bundle can name (T25.F04), the flat list
/// `(ref_idx ascending, pair ascending)`: every pair of every reference at
/// an index representable as `u16`.
fn pair_sites(input_refs: &[InputReference]) -> impl Iterator<Item = (u16, u8)> + '_ {
    input_refs
        .iter()
        .take(usize::from(u16::MAX) + 1)
        .enumerate()
        .flat_map(|(idx, reference)| {
            let ref_idx = u16::try_from(idx).expect("index within u16");
            (0..reference.direction_pairs().len())
                .map(move |pair| (ref_idx, u8::try_from(pair).expect("few pairs")))
        })
}

/// Whether `AddDirectionPairBundle` has a site: a pair-declaring reference at
/// an index ≤ 65,535 and a graph that can host a bundle.
pub(super) fn can_add_pair(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> bool {
    def.can_host_bundle() && pair_sites(input_refs).next().is_some()
}

/// `AddDirectionPairBundle` (T25.F04) and `AddRandomizedDirectionPairBundle`
/// (T25.F08): three draws in order — one pair uniformly over the flat list
/// `(ref_idx ascending, pair ascending)`, the kind over
/// `DirectedVoteKind::ALL`, the offset in `0..8` — then a silent
/// `DirectionPair` bundle: twelve members at own weight 0 and scale 0, with
/// learned creation's draws in `form` ([`draw_learning`]). Duplicates are
/// allowed.
pub(crate) fn add_pair_in_form(
    node: &mut NodeGenome,
    form: CreationForm,
    context: &SensorContext,
    rng: &mut impl Rng,
) -> Result<BundleEdits, MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let sites: Vec<(u16, u8)> = pair_sites(input_refs).collect();
    if sites.is_empty() || !def.can_host_bundle() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (ref_idx, pair) = sites[rng.gen_range(0..sites.len())];
    let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
    let source = BundleSource::Sensor {
        ref_idx,
        field_set: pair,
    };
    let wiring = Wiring::DirectionPair { kind, offset };
    let birth = BundleBirth {
        field_set: pair,
        wiring,
        scale: 0.0,
    };
    let id = node
        .add_bundle(source, birth, context)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    let (def, _) = graph_node(node)?;
    // Commit 1 of T25.F11: this route keeps its twelve direction members;
    // the fact members the transaction wires leave again, in reverse.
    let facts: Vec<_> = def
        .members(id)
        .filter(|&member| def.is_fact_member(member))
        .collect();
    for member in facts.into_iter().rev() {
        def.remove_edge(member);
    }
    draw_learning(def, id, form, rng);
    Ok(BundleEdits::creation(
        &wiring,
        form == CreationForm::Randomized,
    ))
}

/// Whether the unit operators have a site: at least one bundle.
pub(super) fn has_bundle(def: &CgpGraphBackendDef) -> bool {
    !def.bundles().is_empty()
}

fn graph_node(
    node: &mut NodeGenome,
) -> Result<(&mut CgpGraphBackendDef, &[InputReference]), MutationSkipReason> {
    match &mut node.backend_def {
        BackendDef::Graph(def) => Ok((def, &node.input_refs)),
        BackendDef::Vm(_) => Err(MutationSkipReason::NoApplicableTarget),
    }
}

/// `AddRingBundle` and `AddRandomizedRingBundle` (T25.F08): three draws in
/// order — the ring uniformly over the ring indices ascending, the kind
/// over `DirectedVoteKind::ALL`, the offset in `0..8` — then a silent `Ring`
/// bundle: eight members at own weight 0 and scale 0, with learned
/// creation's draws in `form` ([`draw_learning`]). Duplicates are allowed.
pub(crate) fn add_in_form(
    node: &mut NodeGenome,
    form: CreationForm,
    context: &SensorContext,
    rng: &mut impl Rng,
) -> Result<BundleEdits, MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let rings: Vec<u16> = ring_indices(input_refs).collect();
    if rings.is_empty() || !def.can_host_bundle() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let ring_ref = rings[rng.gen_range(0..rings.len())];
    let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
    let source = BundleSource::Sensor {
        ref_idx: ring_ref,
        field_set: 0,
    };
    let wiring = Wiring::Ring { kind, offset };
    let birth = BundleBirth {
        field_set: 0,
        wiring,
        scale: 0.0,
    };
    let id = node
        .add_bundle(source, birth, context)
        .map_err(|_| MutationSkipReason::NoApplicableTarget)?;
    let (def, _) = graph_node(node)?;
    draw_learning(def, id, form, rng);
    Ok(BundleEdits::creation(
        &wiring,
        form == CreationForm::Randomized,
    ))
}

/// `DetachGraphEdge` (T25.F08): one member uniformly in canonical order,
/// one draw and no other, then the F03 detach ([`cgp::detach`]): its own
/// weight becomes `compose(own, scale, pattern)` bitwise, it leaves its
/// bundle keeping its plasticity, its seed composes (T25.F07 invariant 6;
/// a fact member detaches own-only, T25.F10), and a last member's detach
/// removes the bundle.
pub(crate) fn detach_member(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
) -> Result<BundleEdits, MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let picked = select_eligible(def, is_member, rng)?;
    let (_, removed) = cgp::detach(def, picked, input_refs);
    Ok(BundleEdits::detach(removed))
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

/// `ShiftBundleOffset`: `gen_bool(0.5)` true shifts +1, false −1, mod 8;
/// every member moves to its new sink.
pub(crate) fn shift_offset(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let n = VOTE_DIRECTION_COUNT;
    let step = if rng.gen_bool(0.5) { 1 } else { n - 1 };
    let bundle = def.bundles()[id.index()];
    def.relocate_bundle(id, bundle.kind(), (bundle.offset() + step) % n);
    Ok(())
}

/// `RetargetBundleKind`: `gen_range(0..3)` over the other three kinds in
/// catalog order; every member moves to its new sink, a fact member to the
/// new kind's drive. A gene with a fact member is not retargeted to a kind
/// that cannot host facts ([`CgpGraphBackendDef::can_host_facts`],
/// T25.F10): the event skips after its draws.
pub(crate) fn retarget_kind(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let bundle = def.bundles()[id.index()];
    let pick = rng.gen_range(0..DirectedVoteKind::ALL.len() - 1);
    let kind = DirectedVoteKind::ALL
        .into_iter()
        .filter(|&kind| kind != bundle.kind())
        .nth(pick)
        .expect("another kind");
    let has_facts = def.members(id).any(|member| def.is_fact_member(member));
    if has_facts && !def.can_host_facts(kind) {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    def.relocate_bundle(id, kind, bundle.offset());
    Ok(())
}

/// A bundle-record predicate of the retention operators.
pub(super) type BundlePredicate = fn(&cgp::Bundle) -> bool;

/// `EnableBundleRetention`: a record whose retention is exactly 1.
pub(super) fn retains(bundle: &cgp::Bundle) -> bool {
    !retention_decays(bundle.retention())
}

/// `DisableBundleRetention`, `MutateBundleRetention`: a record whose
/// retention is below 1.
pub(super) fn is_decaying_bundle(bundle: &cgp::Bundle) -> bool {
    retention_decays(bundle.retention())
}

/// Whether any bundle record matches `predicate`.
pub(super) fn any_bundle(def: &CgpGraphBackendDef, predicate: BundlePredicate) -> bool {
    def.bundles().iter().any(predicate)
}

/// One bundle uniformly among the records `predicate` admits, in `BundleId`
/// order: count-then-select, one draw. A record without a scale slot is
/// admitted; its retention is inert.
fn pick_where(
    def: &CgpGraphBackendDef,
    predicate: BundlePredicate,
    rng: &mut impl Rng,
) -> Result<BundleId, MutationSkipReason> {
    let matching = || {
        def.bundles()
            .iter()
            .enumerate()
            .filter(|(_, bundle)| predicate(bundle))
    };
    let count = matching().count();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let pick = rng.gen_range(0..count);
    let (index, _) = matching()
        .nth(pick)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    Ok(BundleId::new(
        u16::try_from(index).expect("bundles below u16::MAX"),
    ))
}

/// `EnableBundleRetention` (T25.F09): one retaining bundle, then
/// `EnableRetention`'s draw on its record.
pub(crate) fn enable_retention(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, retains, rng)?;
    let retention = draw_retention(rng);
    def.bundle_mut(id).set_retention(retention);
    Ok(())
}

/// `DisableBundleRetention` (T25.F09): one decaying bundle's retention
/// returns to 1; no value draw.
pub(crate) fn disable_retention(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, is_decaying_bundle, rng)?;
    def.bundle_mut(id).set_retention(1.0);
    Ok(())
}

/// `MutateBundleRetention` (T25.F09): one decaying bundle, one `u ∈ [−1, 1]`,
/// `MutateRetention`'s step on its deficit.
pub(crate) fn mutate_retention(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, is_decaying_bundle, rng)?;
    let u = rng.gen_range(-1.0f32..=1.0);
    let retention = step_retention(def.bundles()[id.index()].retention(), u);
    def.bundle_mut(id).set_retention(retention);
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
