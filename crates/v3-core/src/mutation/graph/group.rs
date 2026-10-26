//! The edge-group operators (T25.F03, T25.F04, T25.F08, T25.F09): four
//! create a silent group (`Ring`, `DirectionPair`, each in the uniform and
//! the randomized form), four change an existing group through its record,
//! three move its learned scale's retention, and one detaches a member.

use rand::Rng;

use crate::contracts::InputReference;
use crate::genome::cgp::{self, CgpGraphBackendDef, GroupId, Scope, Shape};
use crate::genome::vote::{DirectedVoteKind, VOTE_DIRECTION_COUNT};
use crate::genome::{retention_decays, BackendDef, NodeGenome, PlasticityConfig};
use crate::mutation::types::MutationSkipReason;

use super::hebbian::{
    draw_hebbian_config, draw_modulation, draw_retention, is_member, select_eligible,
    step_retention,
};
use super::operators::perturb_weight;

/// How a creation operator draws its new group's learning attributes
/// (T25.F08): `Uniform` one mode and one scope for the whole group (F07),
/// `Randomized` a mode and a scope per member. Both draw one
/// `inherit_scale` for the group; neither draws the pattern or the scale.
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

/// The scope coin: true `Group`.
fn draw_scope(rng: &mut impl Rng) -> Scope {
    if rng.gen_bool(0.5) {
        Scope::Group
    } else {
        Scope::Edge
    }
}

/// Learned creation (T25.F07 invariant 7, T25.F08 invariant 2), drawn after
/// a new group's shape draws. `Uniform`: one mode with its config, then one
/// scope coin, written to every member. `Randomized`: per member in
/// canonical order, its own mode, config and scope coin. Then one
/// `inherit_scale` coin.
fn draw_learning(
    def: &mut CgpGraphBackendDef,
    id: GroupId,
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
    def.group_mut(id).set_inherit_scale(inherit_scale);
}

/// The node's ring indices a group can name, ascending: neighbor rings at an
/// index representable as `u16`.
fn ring_indices(input_refs: &[InputReference]) -> impl Iterator<Item = u16> + '_ {
    input_refs
        .iter()
        .take(usize::from(u16::MAX) + 1)
        .enumerate()
        .filter(|(_, reference)| reference.is_ring())
        .map(|(idx, _)| u16::try_from(idx).expect("index within u16"))
}

/// Whether `AddRingGroup` has a site: a ring at an index ≤ 65,535 and a
/// graph that can host a group ([`CgpGraphBackendDef::can_host_group`]).
pub(super) fn can_add(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> bool {
    def.can_host_group() && ring_indices(input_refs).next().is_some()
}

/// The node's declared pairs a group can name (T25.F04), the flat list
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

/// Whether `AddDirectionPairGroup` has a site: a pair-declaring reference at
/// an index ≤ 65,535 and a graph that can host a group.
pub(super) fn can_add_pair(def: &CgpGraphBackendDef, input_refs: &[InputReference]) -> bool {
    def.can_host_group() && pair_sites(input_refs).next().is_some()
}

/// `AddDirectionPairGroup` (T25.F04) and `AddRandomizedDirectionPairGroup`
/// (T25.F08): three draws in order — one pair uniformly over the flat list
/// `(ref_idx ascending, pair ascending)`, the kind over
/// `DirectedVoteKind::ALL`, the offset in `0..8` — then a silent
/// `DirectionPair` group: twelve members at own weight 0 and scale 0, with
/// learned creation's draws in `form` ([`draw_learning`]). Duplicates are
/// allowed.
pub(crate) fn add_pair_in_form(
    node: &mut NodeGenome,
    form: CreationForm,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let sites: Vec<(u16, u8)> = pair_sites(input_refs).collect();
    if sites.is_empty() || !def.can_host_group() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let (ref_idx, pair) = sites[rng.gen_range(0..sites.len())];
    let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
    let layout = input_refs[usize::from(ref_idx)].direction_pairs()[usize::from(pair)];
    let id = def.add_pair_group(ref_idx, pair, layout, kind, offset, &[]);
    draw_learning(def, id, form, rng);
    Ok(())
}

/// Whether the unit operators have a site: at least one group.
pub(super) fn has_group(def: &CgpGraphBackendDef) -> bool {
    !def.groups().is_empty()
}

fn graph_node(
    node: &mut NodeGenome,
) -> Result<(&mut CgpGraphBackendDef, &[InputReference]), MutationSkipReason> {
    match &mut node.backend_def {
        BackendDef::Graph(def) => Ok((def, &node.input_refs)),
        BackendDef::Vm(_) => Err(MutationSkipReason::NoApplicableTarget),
    }
}

/// `AddRingGroup` and `AddRandomizedRingGroup` (T25.F08): three draws in
/// order — the ring uniformly over the ring indices ascending, the kind
/// over `DirectedVoteKind::ALL`, the offset in `0..8` — then a silent `Ring`
/// group: eight members at own weight 0 and scale 0, with learned
/// creation's draws in `form` ([`draw_learning`]). Duplicates are allowed.
pub(crate) fn add_in_form(
    node: &mut NodeGenome,
    form: CreationForm,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let rings: Vec<u16> = ring_indices(input_refs).collect();
    if rings.is_empty() || !def.can_host_group() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let ring_ref = rings[rng.gen_range(0..rings.len())];
    let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
    let id = def.add_group(Shape::Ring { ring_ref }, kind, offset);
    draw_learning(def, id, form, rng);
    Ok(())
}

/// `DetachGraphEdge` (T25.F08): one member uniformly in canonical order,
/// one draw and no other, then the F03 detach ([`cgp::detach`]): its own
/// weight becomes `compose(own, scale, pattern)` bitwise, it leaves its
/// group keeping its plasticity, its seed composes (T25.F07 invariant 6;
/// a fact member detaches own-only, T25.F10), and a last member's detach
/// removes the group.
pub(crate) fn detach_member(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let picked = select_eligible(def, is_member, rng)?;
    cgp::detach(def, picked, input_refs);
    Ok(())
}

/// The first draw of every unit operator: one group, uniformly.
fn pick(def: &CgpGraphBackendDef, rng: &mut impl Rng) -> Result<GroupId, MutationSkipReason> {
    pick_where(def, |_| true, rng)
}

/// `AlterGroupScale`: `AlterGraphEdgeWeight`'s law on the group's scale.
pub(crate) fn alter_scale(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let mut scale = def.groups()[id.index()].scale();
    perturb_weight(&mut scale, rng);
    def.group_mut(id).set_scale(scale);
    Ok(())
}

/// `ShiftGroupOffset`: `gen_bool(0.5)` true shifts +1, false −1, mod 8;
/// every member moves to its new sink.
pub(crate) fn shift_offset(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let n = VOTE_DIRECTION_COUNT;
    let step = if rng.gen_bool(0.5) { 1 } else { n - 1 };
    let group = def.groups()[id.index()];
    def.relocate_group(id, group.kind(), (group.offset() + step) % n);
    Ok(())
}

/// `RetargetGroupKind`: `gen_range(0..3)` over the other three kinds in
/// catalog order; every member moves to its new sink, a fact member to the
/// new kind's drive. A gene with a fact member is not retargeted to a kind
/// that cannot host facts ([`CgpGraphBackendDef::can_host_facts`],
/// T25.F10): the event skips after its draws.
pub(crate) fn retarget_kind(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    let group = def.groups()[id.index()];
    let pick = rng.gen_range(0..DirectedVoteKind::ALL.len() - 1);
    let kind = DirectedVoteKind::ALL
        .into_iter()
        .filter(|&kind| kind != group.kind())
        .nth(pick)
        .expect("another kind");
    let has_facts = def.members(id).any(|member| def.is_fact_member(member));
    if has_facts && !def.can_host_facts(kind) {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    def.relocate_group(id, kind, group.offset());
    Ok(())
}

/// A group-record predicate of the retention operators.
pub(super) type GroupPredicate = fn(&cgp::Group) -> bool;

/// `EnableGroupRetention`: a record whose retention is exactly 1.
pub(super) fn retains(group: &cgp::Group) -> bool {
    !retention_decays(group.retention())
}

/// `DisableGroupRetention`, `MutateGroupRetention`: a record whose
/// retention is below 1.
pub(super) fn is_decaying_group(group: &cgp::Group) -> bool {
    retention_decays(group.retention())
}

/// Whether any group record matches `predicate`.
pub(super) fn any_group(def: &CgpGraphBackendDef, predicate: GroupPredicate) -> bool {
    def.groups().iter().any(predicate)
}

/// One group uniformly among the records `predicate` admits, in `GroupId`
/// order: count-then-select, one draw. A record without a scale slot is
/// admitted; its retention is inert.
fn pick_where(
    def: &CgpGraphBackendDef,
    predicate: GroupPredicate,
    rng: &mut impl Rng,
) -> Result<GroupId, MutationSkipReason> {
    let matching = || {
        def.groups()
            .iter()
            .enumerate()
            .filter(|(_, group)| predicate(group))
    };
    let count = matching().count();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let pick = rng.gen_range(0..count);
    let (index, _) = matching()
        .nth(pick)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    Ok(GroupId::new(
        u16::try_from(index).expect("groups below u16::MAX"),
    ))
}

/// `EnableGroupRetention` (T25.F09): one retaining group, then
/// `EnableRetention`'s draw on its record.
pub(crate) fn enable_retention(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, retains, rng)?;
    let retention = draw_retention(rng);
    def.group_mut(id).set_retention(retention);
    Ok(())
}

/// `DisableGroupRetention` (T25.F09): one decaying group's retention
/// returns to 1; no value draw.
pub(crate) fn disable_retention(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, is_decaying_group, rng)?;
    def.group_mut(id).set_retention(1.0);
    Ok(())
}

/// `MutateGroupRetention` (T25.F09): one decaying group, one `u ∈ [−1, 1]`,
/// `MutateRetention`'s step on its deficit.
pub(crate) fn mutate_retention(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick_where(def, is_decaying_group, rng)?;
    let u = rng.gen_range(-1.0f32..=1.0);
    let retention = step_retention(def.groups()[id.index()].retention(), u);
    def.group_mut(id).set_retention(retention);
    Ok(())
}

/// `RemoveGroup`: the record and every member.
pub(crate) fn remove(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let id = pick(def, rng)?;
    def.remove_group(id);
    Ok(())
}
