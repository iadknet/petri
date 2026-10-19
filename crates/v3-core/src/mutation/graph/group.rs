//! The edge-group operators (T25.F03, T25.F04): two create a silent group
//! (`Ring`, `DirectionPair`),
//! four change an existing group through its record.

use rand::Rng;

use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, GroupId, Scope, Shape};
use crate::creature::genome::vote::{DirectedVoteKind, VOTE_DIRECTION_COUNT};
use crate::creature::genome::{BackendDef, NodeGenome};
use crate::mutation::types::MutationSkipReason;

use super::hebbian::{draw_hebbian_config, draw_modulation};
use super::operators::perturb_weight;

/// Learned creation (T25.F07 invariant 7), drawn after a new group's shape
/// draws: one learning mode `gen_range(0..3u8)` (0 static, 1 Hebbian, 2
/// reward-modulated); a learning mode draws one config by
/// `EnableHebbian`'s law, then `EnableRewardModulation`'s for mode 2,
/// written to every member; then one scope coin (true `Group`) written to
/// every member; then one `inherit_scale` coin.
fn draw_learning(def: &mut CgpGraphBackendDef, id: GroupId, rng: &mut impl Rng) {
    let mode = rng.gen_range(0..3u8);
    let config = (mode > 0).then(|| {
        let mut config = draw_hebbian_config(rng);
        if mode == 2 {
            config.modulation = Some(draw_modulation(rng));
        }
        config
    });
    let scope = if rng.gen_bool(0.5) {
        Scope::Group
    } else {
        Scope::Edge
    };
    let inherit_scale = rng.gen_bool(0.5);
    let members: Vec<_> = def.members(id).collect();
    for member in members {
        let mut edge = def.edge_mut(member);
        edge.set_plasticity(config);
        edge.set_scope(scope);
    }
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

/// `AddDirectionPairGroup` (T25.F04): three draws in order — one pair
/// uniformly over the flat list `(ref_idx ascending, pair ascending)`, the
/// kind over `DirectedVoteKind::ALL`, the offset in `0..8` — then a silent
/// `DirectionPair` group: twelve members at own weight 0 and scale 0, with
/// learned creation's draws ([`draw_learning`]). Duplicates are allowed.
pub(crate) fn add_pair(
    node: &mut NodeGenome,
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
    let id = def.add_pair_group(ref_idx, pair, layout, kind, offset);
    draw_learning(def, id, rng);
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

/// `AddRingGroup`: three draws in order — the ring uniformly over the ring
/// indices ascending, the kind over `DirectedVoteKind::ALL`, the offset in
/// `0..8` — then a silent `Ring` group: eight members at own weight 0 and
/// scale 0, with learned creation's draws ([`draw_learning`]). Duplicates
/// are allowed.
pub(crate) fn add(node: &mut NodeGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let rings: Vec<u16> = ring_indices(input_refs).collect();
    if rings.is_empty() || !def.can_host_group() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let ring_ref = rings[rng.gen_range(0..rings.len())];
    let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
    let id = def.add_group(Shape::Ring { ring_ref }, kind, offset);
    draw_learning(def, id, rng);
    Ok(())
}

/// The first draw of every unit operator: one group, uniformly.
fn pick(def: &CgpGraphBackendDef, rng: &mut impl Rng) -> Result<GroupId, MutationSkipReason> {
    let count = def.groups().len();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let index = rng.gen_range(0..count);
    Ok(GroupId::new(
        u16::try_from(index).expect("groups below u16::MAX"),
    ))
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

/// `RetargetGroupKind`: `gen_range(0..2)` over the other two kinds in
/// catalog order; every member moves to its new sink.
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
        .expect("two other kinds");
    def.relocate_group(id, kind, group.offset());
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
