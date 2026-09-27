//! Bounded additive refinement of existing semantic input-to-vote groups.
use rand::Rng;
use rand::{rngs::SmallRng, SeedableRng};
use serde::Serialize;
use std::collections::HashMap;

use crate::contracts::{Direction, InputReference, WorldInputKey};
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::{VoteKind, VoteSink};
use crate::creature::genome::{BackendDef, NodeGenome};
use crate::mutation::types::MutationSkipReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroupKey {
    Ring {
        family: WorldInputKey,
        kind: VoteKind,
        offset: u16,
    },
    Slots {
        family: WorldInputKey,
        field: u16,
        sink: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Member {
    position: u16,
    sink: usize,
    edge: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Group {
    key: GroupKey,
    members: Vec<Member>,
}

fn correspondence(
    family: WorldInputKey,
    channel: u16,
    vote: VoteSink,
    sink: usize,
) -> Option<(GroupKey, u16)> {
    if VoteSink::from_index(vote.index()) != Some(vote) || channel >= family.compound_width() {
        return None;
    }
    match family {
        WorldInputKey::NeighborFoodRing { .. }
        | WorldInputKey::NeighborBarrierRing
        | WorldInputKey::NeighborOccupiedRing => {
            let (kind, direction) = match vote {
                VoteSink::Move(d) => (VoteKind::Move, d),
                VoteSink::Reproduce(d) => (VoteKind::Reproduce, d),
                VoteSink::StealEnergy(d) => (VoteKind::StealEnergy, d),
                _ => return None,
            };
            let directions = Direction::ALL.len() as u16;
            Some((
                GroupKey::Ring {
                    family,
                    kind,
                    offset: (u16::from(direction) + directions - channel) % directions,
                },
                channel,
            ))
        }
        WorldInputKey::NearbyCreatureCore
        | WorldInputKey::NearbyCreatureVitals
        | WorldInputKey::NearbyCreatureIdentity => {
            // Four ranked slots; field identity, not slot rank, is repeated.
            let width = family.compound_width() / 4;
            Some((
                GroupKey::Slots {
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

fn groups(def: &CgpGraphBackendDef, references: &[InputReference]) -> Vec<Group> {
    // The map is lookup-only: first genomic occurrence fixes selection order.
    let mut indices = HashMap::new();
    let mut candidates: Vec<(Group, bool)> = Vec::new();
    for (sink_index, sink) in def.output_sinks.iter().enumerate() {
        let OutputSinkKind::ActionVote(vote) = sink.kind else {
            continue;
        };
        for (edge_index, edge) in sink.inputs.iter().enumerate() {
            let GraphSource::InputLeaf { ref_idx, sub_idx } = edge.source else {
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
                    Group {
                        key,
                        members: Vec::new(),
                    },
                    true,
                ));
                candidates.len() - 1
            });
            let (group, valid) = &mut candidates[index];
            if !edge.weight.is_finite()
                || group
                    .members
                    .iter()
                    .any(|member| member.position == position)
            {
                *valid = false;
            } else {
                group.members.push(Member {
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

pub(super) fn has_group(def: &CgpGraphBackendDef, references: &[InputReference]) -> bool {
    !groups(def, references).is_empty()
}

pub(crate) fn refine(node: &mut NodeGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    refine_observed(node, rng, None)
}

/// Assay-only event observer/intervention. Not a production mutation setting.
pub(crate) struct RefinementDiagnostic {
    matched: bool,
    signs: SmallRng,
    pub(crate) steps: Vec<RefinementStep>,
    pub(crate) access: Vec<AccessStep>,
    pub(crate) event: u32,
}

impl RefinementDiagnostic {
    pub(crate) fn new(matched: bool, seed: u64) -> Self {
        Self {
            matched,
            signs: SmallRng::seed_from_u64(seed),
            steps: Vec::new(),
            access: Vec::new(),
            event: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessStep {
    pub event: u32,
    pub node: crate::contracts::NodeId,
    pub channels: Vec<(InputReference, u16)>,
}

pub(crate) fn eligible_focal_groups(
    genome: &crate::creature::genome::CreatureGenome,
    focal: WorldInputKey,
) -> usize {
    genome
        .nodes
        .iter()
        .map(|node| match &node.backend_def {
            BackendDef::Graph(graph) => groups(graph, &node.input_refs)
                .iter()
                .filter(|group| match group.key {
                    GroupKey::Ring { family, .. } | GroupKey::Slots { family, .. } => {
                        family == focal
                    }
                })
                .count(),
            BackendDef::Vm(_) => 0,
        })
        .sum()
}

#[derive(Debug, Clone, Serialize)]
pub struct RefinementCoefficient {
    pub channel: u16,
    pub sink: usize,
    pub edge: usize,
    pub before: f32,
    pub requested: f32,
    pub after: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefinementStep {
    pub event: u32,
    pub node: crate::contracts::NodeId,
    pub family: WorldInputKey,
    pub eligible_groups: usize,
    pub matched: bool,
    pub scalar: f32,
    pub coefficients: Vec<RefinementCoefficient>,
    pub actual_norm: f64,
    pub applied: bool,
}

pub(crate) fn refine_observed(
    node: &mut NodeGenome,
    rng: &mut impl Rng,
    diagnostic: Option<&mut RefinementDiagnostic>,
) -> Result<(), MutationSkipReason> {
    let BackendDef::Graph(def) = &mut node.backend_def else {
        return Err(MutationSkipReason::NoApplicableTarget);
    };
    let groups = groups(def, &node.input_refs);
    if groups.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let selected = &groups[rng.gen_range(0..groups.len())];
    let scalar = rng.gen_range(-0.1f32..=0.1);
    let Some(diagnostic) = diagnostic else {
        return apply_step(def, selected, scalar);
    };
    let increment = scalar / (selected.members.len() as f32).sqrt();
    let coefficients: Vec<_> = selected
        .members
        .iter()
        .map(|member| {
            let before = def.output_sinks[member.sink].inputs[member.edge].weight;
            let requested = if diagnostic.matched {
                increment.abs()
                    * if diagnostic.signs.gen_bool(0.5) {
                        1.0
                    } else {
                        -1.0
                    }
            } else {
                increment
            };
            RefinementCoefficient {
                channel: member.position,
                sink: member.sink,
                edge: member.edge,
                before,
                requested,
                after: before + requested,
            }
        })
        .collect();
    let actual_norm = coefficients
        .iter()
        .map(|coefficient| (f64::from(coefficient.after) - f64::from(coefficient.before)).powi(2))
        .sum::<f64>()
        .sqrt();
    let valid = coefficients
        .iter()
        .all(|coefficient| coefficient.after.is_finite())
        && actual_norm <= 0.1 + 1e-6;
    let family = match selected.key {
        GroupKey::Ring { family, .. } | GroupKey::Slots { family, .. } => family,
    };
    let mut step = RefinementStep {
        event: diagnostic.event,
        node: node.node_id,
        family,
        eligible_groups: groups.len(),
        matched: diagnostic.matched,
        scalar,
        coefficients,
        actual_norm,
        applied: valid,
    };
    if valid {
        for coefficient in &step.coefficients {
            def.output_sinks[coefficient.sink].inputs[coefficient.edge].weight = coefficient.after;
        }
    } else {
        for coefficient in &mut step.coefficients {
            coefficient.after = coefficient.before;
        }
        step.actual_norm = 0.0;
    }
    diagnostic.steps.push(step);
    if valid {
        Ok(())
    } else {
        Err(MutationSkipReason::NumericProposalRejected)
    }
}

fn apply_step(
    def: &mut CgpGraphBackendDef,
    group: &Group,
    scalar: f32,
) -> Result<(), MutationSkipReason> {
    let increment = scalar / (group.members.len() as f32).sqrt();
    let mut proposed = [0.0f32; 8];
    let mut norm_squared = 0.0f64;
    for (member, proposed) in group.members.iter().zip(&mut proposed) {
        let old = def.output_sinks[member.sink].inputs[member.edge].weight;
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
        def.output_sinks[member.sink].inputs[member.edge].weight = proposed;
    }
    Ok(())
}

#[cfg(test)]
#[path = "refinement_tests.rs"]
mod tests;
