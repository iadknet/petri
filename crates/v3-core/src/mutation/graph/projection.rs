//! The ring projection operators (T11.F31): one adds a projection born
//! silent at weight 0, four change an existing projection as a unit.

use rand::Rng;

use crate::contracts::InputReference;
use crate::creature::genome::cgp::{CgpGraphBackendDef, RingProjection};
use crate::creature::genome::vote::{DirectedVoteKind, VOTE_DIRECTION_COUNT};
use crate::creature::genome::{BackendDef, NodeGenome};
use crate::mutation::types::MutationSkipReason;

use super::operators::perturb_weight;

/// The node's ring indices a projection can name, ascending: neighbor rings
/// at an index representable as `u16`.
fn ring_indices(input_refs: &[InputReference]) -> Vec<u16> {
    input_refs
        .iter()
        .take(usize::from(u16::MAX) + 1)
        .enumerate()
        .filter(|(_, reference)| reference.is_ring())
        .map(|(idx, _)| u16::try_from(idx).expect("index within u16"))
        .collect()
}

/// Whether `AddRingProjection` has a site: a ring at an index ≤ 65,535.
pub(super) fn can_add(input_refs: &[InputReference]) -> bool {
    input_refs
        .iter()
        .take(usize::from(u16::MAX) + 1)
        .any(InputReference::is_ring)
}

/// Whether the unit operators have a site: at least one projection.
pub(super) fn has_projection(def: &CgpGraphBackendDef) -> bool {
    !def.projections.is_empty()
}

fn graph_node(
    node: &mut NodeGenome,
) -> Result<(&mut CgpGraphBackendDef, &[InputReference]), MutationSkipReason> {
    match &mut node.backend_def {
        BackendDef::Graph(def) => Ok((def, &node.input_refs)),
        BackendDef::Vm(_) => Err(MutationSkipReason::NoApplicableTarget),
    }
}

/// `AddRingProjection`: three draws in order — the ring uniformly over the
/// ring indices ascending, the kind over `DirectedVoteKind::ALL`, the offset
/// in `0..8` — then push `{ring, kind, offset, 0.0}`. Duplicates are allowed.
pub(crate) fn add(node: &mut NodeGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    let (def, input_refs) = graph_node(node)?;
    let rings = ring_indices(input_refs);
    if rings.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let ref_idx = rings[rng.gen_range(0..rings.len())];
    let kind = DirectedVoteKind::ALL[rng.gen_range(0..DirectedVoteKind::ALL.len())];
    let offset = rng.gen_range(0..VOTE_DIRECTION_COUNT);
    def.projections
        .push(RingProjection::new(ref_idx, kind, offset, 0.0));
    Ok(())
}

/// The first draw of every unit operator: one projection, uniformly.
fn pick(def: &mut CgpGraphBackendDef, rng: &mut impl Rng) -> Result<usize, MutationSkipReason> {
    if def.projections.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(rng.gen_range(0..def.projections.len()))
}

/// `AlterRingProjectionWeight`: `AlterGraphEdgeWeight`'s law on one weight.
pub(crate) fn alter_weight(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = pick(def, rng)?;
    perturb_weight(&mut def.projections[idx].weight, rng);
    Ok(())
}

/// `ShiftRingProjectionOffset`: `gen_bool(0.5)` true shifts +1, false −1,
/// mod 8.
pub(crate) fn shift_offset(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = pick(def, rng)?;
    let n = VOTE_DIRECTION_COUNT;
    let step = if rng.gen_bool(0.5) { 1 } else { n - 1 };
    let projection = &mut def.projections[idx];
    projection.offset = (projection.offset % n + step) % n;
    Ok(())
}

/// `RetargetRingProjectionKind`: `gen_range(0..2)` over the other two kinds
/// in catalog order.
pub(crate) fn retarget_kind(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = pick(def, rng)?;
    let projection = &mut def.projections[idx];
    let others: Vec<DirectedVoteKind> = DirectedVoteKind::ALL
        .into_iter()
        .filter(|&kind| kind != projection.kind)
        .collect();
    projection.kind = others[rng.gen_range(0..others.len())];
    Ok(())
}

/// `RemoveRingProjection`: `Vec::remove`, order kept.
pub(crate) fn remove(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = pick(def, rng)?;
    def.projections.remove(idx);
    Ok(())
}
