//! Bounded additive refinement of inferred input-to-vote sets: the ranked
//! slots of a creature-sensor field wired to one vote sink. A `Ring` edge
//! group's scale is its explicit refinement (`AlterGroupScale`, T25.F03), so
//! refinement infers no ring set and never steps a member.
use rand::Rng;
use std::collections::HashMap;

use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, EdgeOccurrence, EdgeSurface, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{BackendDef, NodeGenome};
use crate::mutation::types::{MutationSkipReason, RefinementKey};

/// The key of an inferred set: one creature-sensor field across its ranked
/// slots, into one vote sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SlotKey {
    family: WorldInputKey,
    field: u16,
    sink: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InferredMember {
    position: u16,
    sink: usize,
    edge: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct InferredSet {
    key: SlotKey,
    members: Vec<InferredMember>,
}

fn correspondence(
    family: WorldInputKey,
    channel: u16,
    vote: VoteSink,
    sink: usize,
) -> Option<(SlotKey, u16)> {
    if VoteSink::from_index(vote.index()) != Some(vote) || channel >= family.compound_width() {
        return None;
    }
    match family {
        WorldInputKey::NearbyCreatureCore
        | WorldInputKey::NearbyCreatureVitals
        | WorldInputKey::NearbyCreatureIdentity => {
            // Four ranked slots; field identity, not slot rank, is repeated.
            let width = family.compound_width() / 4;
            Some((
                SlotKey {
                    family,
                    field: channel % width,
                    sink,
                },
                channel / width,
            ))
        }
        _ => None,
    }
}

fn inferred_sets(def: &CgpGraphBackendDef, references: &[InputReference]) -> Vec<InferredSet> {
    // The map is lookup-only: first genomic occurrence fixes selection order.
    let mut indices = HashMap::new();
    let mut candidates: Vec<(InferredSet, bool)> = Vec::new();
    for (sink_index, sink) in def.output_sinks().iter().enumerate() {
        let OutputSinkKind::ActionVote(vote) = sink.kind else {
            continue;
        };
        for (edge_index, edge) in sink.inputs.iter().enumerate() {
            // A group member is its group's to refine (T25.F03).
            if edge.membership().is_some() {
                continue;
            }
            let GraphSource::InputLeaf { ref_idx, sub_idx } = edge.source() else {
                continue;
            };
            let Some(InputReference::World(family)) = references.get(usize::from(ref_idx)) else {
                continue;
            };
            let Some((key, position)) = correspondence(*family, sub_idx, vote, sink_index) else {
                continue;
            };
            let index = *indices.entry(key).or_insert_with(|| {
                candidates.push((
                    InferredSet {
                        key,
                        members: Vec::new(),
                    },
                    true,
                ));
                candidates.len() - 1
            });
            let (group, valid) = &mut candidates[index];
            if !edge.own_weight().is_finite()
                || group
                    .members
                    .iter()
                    .any(|member| member.position == position)
            {
                *valid = false;
            } else {
                group.members.push(InferredMember {
                    position,
                    sink: sink_index,
                    edge: edge_index,
                });
            }
        }
    }
    candidates
        .into_iter()
        .filter_map(|(group, valid)| (valid && group.members.len() >= 2).then_some(group))
        .collect()
}

pub(super) fn has_inferred_set(def: &CgpGraphBackendDef, references: &[InputReference]) -> bool {
    !inferred_sets(def, references).is_empty()
}

/// Step one inferred set; returns the stepped set's `(family, field)`
/// (T25.F04: applications are counted by key).
pub(crate) fn refine(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
) -> Result<RefinementKey, MutationSkipReason> {
    let BackendDef::Graph(def) = &mut node.backend_def else {
        return Err(MutationSkipReason::NoApplicableTarget);
    };
    let sets = inferred_sets(def, &node.input_refs);
    if sets.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let selected = &sets[rng.gen_range(0..sets.len())];
    apply_step(def, selected, rng.gen_range(-0.1f32..=0.1))?;
    Ok(RefinementKey {
        family: selected.key.family,
        field: selected.key.field,
    })
}

fn apply_step(
    def: &mut CgpGraphBackendDef,
    group: &InferredSet,
    scalar: f32,
) -> Result<(), MutationSkipReason> {
    let increment = scalar / (group.members.len() as f32).sqrt();
    let mut proposed = [0.0f32; 8];
    let mut norm_squared = 0.0f64;
    for (member, proposed) in group.members.iter().zip(&mut proposed) {
        let old = def.output_sinks()[member.sink].inputs[member.edge].own_weight();
        *proposed = old + increment;
        if !proposed.is_finite() {
            return Err(MutationSkipReason::NumericProposalRejected);
        }
        norm_squared += (f64::from(*proposed) - f64::from(old)).powi(2);
    }
    if norm_squared.sqrt() > 0.1 + 1e-6 {
        return Err(MutationSkipReason::NumericProposalRejected);
    }
    // No writes occur until every coefficient and the actual vector pass.
    for (member, proposed) in group.members.iter().zip(proposed) {
        def.edge_mut(EdgeOccurrence::new(
            EdgeSurface::SinkInput(member.sink),
            member.edge,
        ))
        .set_weight(proposed);
    }
    Ok(())
}

#[cfg(test)]
#[path = "refinement_tests.rs"]
mod tests;
