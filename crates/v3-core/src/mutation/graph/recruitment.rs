//! One bounded, initially silent afferent addition to an existing vote sink.
use std::collections::HashMap;

use rand::{seq::SliceRandom, Rng};

use crate::config::NeutralInputRecruitment as Arm;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind, RingProjection,
};
use crate::creature::genome::vote::{DirectedVoteKind, VoteSink, VOTE_DIRECTION_COUNT};
use crate::creature::genome::{BackendDef, NodeGenome};
use crate::mutation::compound::sub_value_count;
use crate::mutation::input_ref::input_reference_universe;
use crate::mutation::types::MutationSkipReason;
use crate::runtime::OUTPUT_SLOT_COUNT;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceFamily {
    Input(InputReference),
    Upstream,
    Memory(bool),
}

struct Plan {
    declarations: Vec<InputReference>,
    sources: Vec<GraphSource>,
    /// The ring declared or found, and the drawn channel, when the family is
    /// a neighbor ring (T11.F31).
    ring: Option<(u16, u16)>,
}

impl Plan {
    /// Append the plan at `destination`. A ring family landing on a directed
    /// vote sink `K(d)` becomes one zero-weight projection
    /// `{ref_idx, K, (d − channel) mod 8, 0.0}` in place of its edges
    /// (T11.F31); every other plan appends its edges at weight 0.
    fn append(self, node: &mut NodeGenome, destination: usize) {
        let BackendDef::Graph(def) = &mut node.backend_def else {
            unreachable!("admission requires a Graph backend")
        };
        node.input_refs.extend(self.declarations);
        let directed = match def.output_sinks[destination].kind {
            OutputSinkKind::ActionVote(vote) => DirectedVoteKind::of_sink(vote),
            _ => None,
        };
        if let (Some((ref_idx, channel)), Some((kind, d))) = (self.ring, directed) {
            let channel =
                u8::try_from(channel % u16::from(VOTE_DIRECTION_COUNT)).expect("channel below 8");
            def.projections.push(RingProjection::new(
                ref_idx,
                kind,
                RingProjection::offset_for(d, channel),
                0.0,
            ));
            return;
        }
        def.output_sinks[destination]
            .inputs
            .extend(self.sources.into_iter().map(|source| GraphEdge {
                source,
                weight: 0.0,
            }));
    }
}

/// First representable matching index, and the safe append prefix. A dangling
/// leaf anywhere in the graph must retain its zero fallback, even at weight 0.
struct Admission<'a> {
    existing: HashMap<&'a InputReference, u16>,
    next: usize,
    append_end: usize,
}

impl<'a> Admission<'a> {
    fn new(references: &'a [InputReference], def: &CgpGraphBackendDef) -> Self {
        let mut existing = HashMap::new();
        for (index, reference) in references.iter().enumerate().take(65536) {
            existing
                .entry(reference)
                .or_insert(u16::try_from(index).expect("representable prefix"));
        }
        // A dangling projection reference is a dangling leaf (T11.F31).
        let append_end = def
            .edges()
            .filter_map(|edge| match edge.source {
                GraphSource::InputLeaf { ref_idx, .. } => Some(ref_idx),
                _ => None,
            })
            .chain(def.projections.iter().map(|projection| projection.ref_idx))
            .map(usize::from)
            .filter(|&ref_idx| ref_idx >= references.len())
            .min()
            .unwrap_or(65536);
        Self {
            existing,
            next: references.len(),
            append_end,
        }
    }

    fn plan(&self, family: &SourceFamily, arm: Arm, channel: u16) -> Option<Plan> {
        if arm == Arm::Off {
            return None;
        }
        let mut plan = Plan {
            declarations: vec![],
            sources: vec![],
            ring: None,
        };
        let mut ring = None;
        let mut declare = |reference: &InputReference| {
            if let Some(&index) = self.existing.get(reference) {
                return Some(index);
            }
            let index = self.next.checked_add(plan.declarations.len())?;
            if index >= self.append_end {
                return None;
            }
            let index = u16::try_from(index).ok()?;
            plan.declarations.push(reference.clone());
            Some(index)
        };
        match family {
            SourceFamily::Input(reference) => {
                let width = sub_value_count(reference);
                if *reference == InputReference::ActionQueue {
                    if arm == Arm::WholeFamily {
                        return None;
                    }
                } else if channel >= width {
                    return None;
                }
                let ref_idx = declare(reference)?;
                if reference.is_ring() {
                    ring = Some((ref_idx, channel));
                }
                if arm == Arm::SingleChannel {
                    plan.sources.push(GraphSource::InputLeaf {
                        ref_idx,
                        sub_idx: channel,
                    });
                } else {
                    plan.sources.extend(
                        (0..width).map(|sub_idx| GraphSource::InputLeaf { ref_idx, sub_idx }),
                    );
                }
            }
            SourceFamily::Upstream => {
                if arm != Arm::WholeFamily {
                    return None;
                }
                for slot in 0..OUTPUT_SLOT_COUNT {
                    let ref_idx = declare(&InputReference::UpstreamSlot(slot))?;
                    plan.sources.push(GraphSource::InputLeaf {
                        ref_idx,
                        sub_idx: 0,
                    });
                }
            }
            SourceFamily::Memory(previous) => {
                if channel >= 16 {
                    return None;
                }
                let channels = if arm == Arm::WholeFamily {
                    0..16
                } else {
                    channel..channel + 1
                };
                plan.sources
                    .extend(channels.map(|slot| GraphSource::SharedMemory {
                        slot: u8::try_from(slot).expect("memory slot"),
                        previous: *previous,
                    }));
            }
        }
        plan.ring = ring;
        Some(plan)
    }

    fn families(&self, arm: Arm, food_type_count: usize) -> Vec<SourceFamily> {
        let mut families: Vec<_> = input_reference_universe(food_type_count)
            .into_iter()
            .filter(|reference| {
                arm != Arm::WholeFamily
                    || !matches!(
                        reference,
                        InputReference::ActionQueue | InputReference::UpstreamSlot(_)
                    )
            })
            .map(SourceFamily::Input)
            .collect();
        if arm == Arm::WholeFamily {
            families.push(SourceFamily::Upstream);
        }
        families.extend([SourceFamily::Memory(false), SourceFamily::Memory(true)]);
        families.retain(|family| self.plan(family, arm, 0).is_some());
        families
    }
}

pub(super) fn destinations(def: &CgpGraphBackendDef) -> Vec<usize> {
    def.output_sinks
        .iter()
        .enumerate()
        .filter_map(|(index, sink)| match sink.kind {
            OutputSinkKind::ActionVote(vote)
                if VoteSink::from_index(vote.index()) == Some(vote) =>
            {
                Some(index)
            }
            _ => None,
        })
        .collect()
}

pub(super) fn recruit(
    node: &mut NodeGenome,
    arm: Arm,
    food_type_count: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let BackendDef::Graph(def) = &node.backend_def else {
        return Err(MutationSkipReason::NoApplicableTarget);
    };
    let sites = destinations(def);
    if arm == Arm::Off || sites.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let admission = Admission::new(&node.input_refs, def);
    let families = admission.families(arm, food_type_count);
    // Both memory banks are always feasible. No retries, partial bundles or
    // sampled capacity failure can turn an eligible node into a local skip.
    let family = families
        .choose(rng)
        .expect("shared memory needs no declaration");
    let channel = if arm == Arm::WholeFamily {
        0
    } else {
        match family {
            SourceFamily::Input(InputReference::ActionQueue) => rng.gen(),
            SourceFamily::Input(reference) => rng.gen_range(0..sub_value_count(reference)),
            SourceFamily::Memory(_) => rng.gen_range(0..16),
            SourceFamily::Upstream => unreachable!("whole-family source"),
        }
    };
    let plan = admission
        .plan(family, arm, channel)
        .expect("preflighted source");
    let destination = *sites.choose(rng).expect("present vote sink");
    plan.append(node, destination);
    Ok(())
}

/// Explicit access fixture using the same preflight and append as mutation.
#[cfg(test)]
pub(crate) fn recruit_source(
    node: &mut NodeGenome,
    family: &SourceFamily,
    arm: Arm,
    channel: u16,
    vote: VoteSink,
) -> Result<(), MutationSkipReason> {
    let BackendDef::Graph(def) = &node.backend_def else {
        return Err(MutationSkipReason::NoApplicableTarget);
    };
    let destination = destinations(def)
        .into_iter()
        .find(|&i| def.output_sinks[i].kind == OutputSinkKind::ActionVote(vote))
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let plan = Admission::new(&node.input_refs, def)
        .plan(family, arm, channel)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    plan.append(node, destination);
    Ok(())
}

#[cfg(test)]
#[path = "recruitment_tests.rs"]
mod tests;
