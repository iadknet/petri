//! One bounded, initially silent afferent addition to an existing vote sink.
use std::collections::HashMap;

use rand::{seq::SliceRandom, Rng};

use crate::config::NeutralInputRecruitment as Arm;
use crate::contracts::{DirectionPair, InputReference};
use crate::creature::genome::cgp::{
    ring_offset, CgpGraphBackendDef, EdgeSurface, GraphEdge, GraphSource, OutputSinkKind, Shape,
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
    /// a neighbor ring.
    ring: Option<(u16, u16)>,
    /// The pair plan, when the family declares direction pairs and the arm
    /// is `WholeFamily` (T25.F04).
    pairs: Option<PairPlan>,
}

/// The reference declared or found and the pairs it declares. A family whose
/// pairs are its slots ([`InputReference::slot_pairs`], T25.F05) carries no
/// whole-family sources: its plan resolves to one drawn slot on every
/// destination.
#[derive(Clone, Copy)]
struct PairPlan {
    ref_idx: u16,
    pairs: &'static [DirectionPair],
    slots: bool,
}

/// The `(kind, direction)` of the directed vote sink at `destination`.
fn directed(def: &CgpGraphBackendDef, destination: usize) -> Option<(DirectedVoteKind, u8)> {
    def.output_sinks()[destination].kind.direction()
}

impl Plan {
    /// How many pairs the pair case draws over at `destination`: the
    /// family's declared pairs when the plan is a pair plan landing on a
    /// directed sink, or a slot plan on any destination (T25.F05); `None`
    /// otherwise (no draw).
    fn pair_count(&self, def: &CgpGraphBackendDef, destination: usize) -> Option<usize> {
        let plan = self.pairs?;
        (plan.slots || directed(def, destination).is_some()).then_some(plan.pairs.len())
    }

    /// Append the plan at `destination`. A ring family landing on a directed
    /// vote sink `K(d)` becomes one silent `Ring` group on `ref_idx` into `K`
    /// at offset `(d − channel) mod 8`, eight members at own weight 0 and
    /// scale 0, in place of its edges (T25.F03). A pair plan landing on
    /// `K(d)` with drawn pair `pair` becomes one silent `DirectionPair` group
    /// at offset `d` (twelve members) plus one born-0 edge into `Drive K` per
    /// fact of the pair, in `facts` order (T25.F04). A slot plan landing
    /// elsewhere appends the drawn slot's channels ascending, born 0
    /// (T25.F05). Every other plan appends its edges at weight 0.
    fn append(self, node: &mut NodeGenome, destination: usize, pair: Option<u8>) {
        let BackendDef::Graph(def) = &mut node.backend_def else {
            unreachable!("admission requires a Graph backend")
        };
        node.input_refs.extend(self.declarations);
        let directed = directed(def, destination);
        if let (Some(PairPlan { ref_idx, pairs, .. }), Some((kind, d)), Some(pair)) =
            (self.pairs, directed, pair)
        {
            let layout = pairs[usize::from(pair)];
            def.add_pair_group(ref_idx, pair, layout, kind, d);
            let drive = def
                .sink_surface(OutputSinkKind::ActionVote(kind.drive()))
                .expect("the preflight admits a pair family only beside every drive sink");
            for &sub_idx in layout.facts {
                def.push_edge(
                    drive,
                    GraphEdge::new(GraphSource::InputLeaf { ref_idx, sub_idx }, 0.0),
                );
            }
            return;
        }
        if let Some(PairPlan {
            ref_idx,
            pairs,
            slots: true,
        }) = self.pairs
        {
            let layout = pairs[usize::from(pair.expect("a slot plan always draws its slot"))];
            let mut channels: Vec<u16> = [layout.x, layout.y]
                .into_iter()
                .chain(layout.facts.iter().copied())
                .collect();
            channels.sort_unstable();
            for sub_idx in channels {
                def.push_edge(
                    EdgeSurface::SinkInput(destination),
                    GraphEdge::new(GraphSource::InputLeaf { ref_idx, sub_idx }, 0.0),
                );
            }
            return;
        }
        if let (Some((ref_idx, channel)), Some((kind, d))) = (self.ring, directed) {
            let channel =
                u8::try_from(channel % u16::from(VOTE_DIRECTION_COUNT)).expect("channel below 8");
            def.add_group(
                Shape::Ring { ring_ref: ref_idx },
                kind,
                ring_offset(d, channel),
            );
            return;
        }
        for source in self.sources {
            def.push_edge(
                EdgeSurface::SinkInput(destination),
                GraphEdge::new(source, 0.0),
            );
        }
    }
}

/// First representable matching index, and the safe append prefix. A dangling
/// leaf anywhere in the graph must retain its zero fallback, even at weight 0.
struct Admission<'a> {
    existing: HashMap<&'a InputReference, u16>,
    next: usize,
    append_end: usize,
    /// Whether a ring family's group can be created
    /// ([`CgpGraphBackendDef::can_host_group`]); when not, ring families
    /// leave the universe, so a drawn ring plan always completes.
    hosts_ring_group: bool,
    /// Whether a pair family's group and facts edges can be placed: a group
    /// can be hosted and every kind's `Drive` sink is present (T25.F04);
    /// when not, pair-declaring families leave the `WholeFamily` universe.
    hosts_pair_plan: bool,
}

impl<'a> Admission<'a> {
    fn new(references: &'a [InputReference], def: &CgpGraphBackendDef) -> Self {
        let hosts_ring_group = def.can_host_group();
        let hosts_pair_plan = hosts_ring_group
            && DirectedVoteKind::ALL.into_iter().all(|kind| {
                def.sink_surface(OutputSinkKind::ActionVote(kind.drive()))
                    .is_some()
            });
        let mut existing = HashMap::new();
        for (index, reference) in references.iter().enumerate().take(65536) {
            existing
                .entry(reference)
                .or_insert(u16::try_from(index).expect("representable prefix"));
        }
        // A group's members are edges, so a dangling member reference is a
        // dangling leaf like any other (T25.F03).
        let append_end = def
            .edges()
            .filter_map(|edge| match edge.source() {
                GraphSource::InputLeaf { ref_idx, .. } => Some(ref_idx),
                _ => None,
            })
            .map(usize::from)
            .filter(|&ref_idx| ref_idx >= references.len())
            .min()
            .unwrap_or(65536);
        Self {
            existing,
            next: references.len(),
            append_end,
            hosts_ring_group,
            hosts_pair_plan,
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
            pairs: None,
        };
        let mut ring = None;
        let mut pairs = None;
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
                if arm == Arm::WholeFamily && !reference.direction_pairs().is_empty() {
                    pairs = Some(PairPlan {
                        ref_idx,
                        pairs: reference.direction_pairs(),
                        slots: reference.slot_pairs(),
                    });
                }
                if arm == Arm::SingleChannel {
                    plan.sources.push(GraphSource::InputLeaf {
                        ref_idx,
                        sub_idx: channel,
                    });
                } else if !reference.slot_pairs() {
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
        plan.pairs = pairs;
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
            .filter(|reference| self.hosts_ring_group || !reference.is_ring())
            .filter(|reference| {
                arm != Arm::WholeFamily
                    || self.hosts_pair_plan
                    || reference.direction_pairs().is_empty()
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
    def.output_sinks()
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
    // The pair case's only draw (T25.F04), after the destination: on a
    // directed sink, or on every destination for a slot family (T25.F05).
    let pair = plan
        .pair_count(def, destination)
        .map(|count| u8::try_from(rng.gen_range(0..count)).expect("few pairs"));
    plan.append(node, destination, pair);
    Ok(())
}

/// Explicit access fixture using the same preflight and append as mutation;
/// a pair plan that draws a pair takes pair 0.
#[cfg(test)]
pub(crate) fn recruit_source(
    node: &mut NodeGenome,
    family: &SourceFamily,
    arm: Arm,
    channel: u16,
    vote: VoteSink,
) -> Result<(), MutationSkipReason> {
    place(node, family, arm, channel, vote, 0)
}

/// [`recruit_source`] under `WholeFamily` choosing the pair the pair case
/// draws (T25.F04).
#[cfg(test)]
pub(crate) fn recruit_source_with_pair(
    node: &mut NodeGenome,
    family: &SourceFamily,
    arm: Arm,
    vote: VoteSink,
    pair: u8,
) -> Result<(), MutationSkipReason> {
    place(node, family, arm, 0, vote, pair)
}

#[cfg(test)]
fn place(
    node: &mut NodeGenome,
    family: &SourceFamily,
    arm: Arm,
    channel: u16,
    vote: VoteSink,
    pair: u8,
) -> Result<(), MutationSkipReason> {
    let BackendDef::Graph(def) = &node.backend_def else {
        return Err(MutationSkipReason::NoApplicableTarget);
    };
    let destination = destinations(def)
        .into_iter()
        .find(|&i| def.output_sinks()[i].kind == OutputSinkKind::ActionVote(vote))
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let plan = Admission::new(&node.input_refs, def)
        .plan(family, arm, channel)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    let pair = plan.pair_count(def, destination).map(|_| pair);
    plan.append(node, destination, pair);
    Ok(())
}

#[cfg(test)]
#[path = "recruitment_tests.rs"]
mod tests;
