//! The ring-rule census (`ring-rules-v2`, T11.F31, T25.F03): which
//! direction-general rules "vote kind `K` toward direction `d` from ring
//! channel `d − offset`" a population carries, through direct edges or one
//! bus hop, and the `Ring` edge bundles it carries.
//!
//! A rule is `(family, kind, offset, sign)`. A path for direction `d` on a
//! reachable Graph node is (i) an `ActionVote` edge into `K(d)` whose genomic
//! weight `w` has `|w| > 1e-3`, from the ring's channel `(d − offset) mod 8`,
//! sign of `w`; or (ii) such an edge reading bus slot `j` (`UpstreamSlot(j)`
//! or `UpstreamBus` field `j`) while some
//! reachable Graph node has an edge with `|w'| > 1e-3` into `CustomOutput(j)`
//! from that ring channel, sign of `w · w'` (execution order and overwrites
//! ignored, so an upper bound). Bundle members are sink edges and take part
//! in both paths at their genomic weight `scale + own_weight`; a member's
//! offset comes from its channel and sink as any edge's does. Compute-node
//! paths are not traced, and no reachable live VM node votes, so VM nodes
//! are not read.
//!
//! Pure structure: it reads genomes and cached reachability and executes no
//! brain.

use std::collections::BTreeMap;

use crate::contracts::votes::{DirectedVoteKind, VOTE_DIRECTION_COUNT};
use crate::contracts::{InputReference, WorldInputKey};
use crate::genome::cgp::{
    ring_offset, BundleSource, CgpGraphBackendDef, GraphSource, OutputSinkKind, Wiring,
};
use crate::genome::{BackendDef, CreatureGenome};
use crate::neighborhood::sensor_census::world_input_key_label;

/// The census version the goal summary records.
pub const RING_RULES_VERSION: &str = "ring-rules-v2";

/// The weight magnitude above which an edge carries a path, and above which
/// a bundle's scale counts as nonzero.
pub const PATH_WEIGHT_FLOOR: f32 = 1e-3;

/// A rule covering this many directions or more is direction-general.
pub const DIRECTION_GENERAL_COVERAGE: u32 = 6;

const DIRECTIONS: u8 = VOTE_DIRECTION_COUNT;

/// One rule: the ring family, the directed kind, the offset, and the sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rule {
    pub family: WorldInputKey,
    pub kind: DirectedVoteKind,
    pub offset: u8,
    pub positive: bool,
}

/// Per-class `Ring` bundle counts on reachable nodes: bundles, those with a
/// positive or negative nonzero scale, and their members present.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RingBundleCounts {
    pub count: u64,
    pub positive: u64,
    pub negative: u64,
    pub members: u64,
}

/// The census of one population.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RingRuleCensus {
    pub creatures: u64,
    /// Creatures with some rule covering at least
    /// [`DIRECTION_GENERAL_COVERAGE`] directions.
    pub direction_general: u64,
    /// The same, counting paths through non-member edges only.
    pub direction_general_edges_only: u64,
    /// Creatures carrying at least one `Ring` bundle on a reachable node.
    pub ring_bundle_carriers: u64,
    /// `Ring` bundles on reachable nodes.
    pub ring_bundles: u64,
    /// `Ring` bundles on reachable nodes with `|scale| > PATH_WEIGHT_FLOOR`.
    pub ring_bundles_nonzero: u64,
    /// `Ring` bundles on unreachable nodes.
    pub ring_bundles_unreachable: u64,
    /// Members present on reachable `Ring` bundles, summed.
    pub ring_bundle_members: u64,
    /// Reachable `Ring` bundles by `(family, kind, offset)`.
    pub ring_bundle_counts: BTreeMap<(WorldInputKey, DirectedVoteKind, u8), RingBundleCounts>,
    /// Creatures by coverage: `coverage[rule][n - 1]` creatures cover the
    /// rule in exactly `n` directions.
    pub coverage: BTreeMap<Rule, [u64; DIRECTIONS as usize]>,
}

/// The ring a reference is, when it is one.
fn ring_of(reference: Option<&InputReference>) -> Option<WorldInputKey> {
    match reference {
        Some(InputReference::World(key)) if key.is_neighbor_ring() => Some(*key),
        _ => None,
    }
}

fn carries(weight: f32) -> bool {
    weight.abs() > PATH_WEIGHT_FLOOR
}

/// One creature's coverage: per rule, the direction masks of all paths and
/// of edge paths only.
#[derive(Default)]
struct Coverage(BTreeMap<Rule, (u8, u8)>);

impl Coverage {
    fn add(&mut self, rule: Rule, d: u8, edge: bool) {
        let entry = self.0.entry(rule).or_default();
        entry.0 |= 1 << d;
        if edge {
            entry.1 |= 1 << d;
        }
    }
}

fn graphs<'a>(
    genome: &'a CreatureGenome,
    reachable: &'a [usize],
) -> impl Iterator<Item = (&'a CgpGraphBackendDef, &'a [InputReference])> + 'a {
    reachable.iter().filter_map(|&idx| {
        let node = genome.nodes.get(idx)?;
        match &node.backend_def {
            BackendDef::Graph(def) => Some((def, node.input_refs.as_slice())),
            BackendDef::Vm(_) => None,
        }
    })
}

/// The ring channels each `CustomOutput(j)` carries on the reachable graphs:
/// `(family, channel, sign of w')`.
fn bus(
    genome: &CreatureGenome,
    reachable: &[usize],
) -> BTreeMap<u8, Vec<(WorldInputKey, u8, bool)>> {
    let mut bus: BTreeMap<u8, Vec<(WorldInputKey, u8, bool)>> = BTreeMap::new();
    for (def, refs) in graphs(genome, reachable) {
        for sink in def.output_sinks() {
            let OutputSinkKind::CustomOutput(slot) = sink.kind else {
                continue;
            };
            for edge in sink.inputs.iter() {
                let weight = def.edge_weight(edge, sink.kind, refs);
                if !carries(weight) {
                    continue;
                }
                if let GraphSource::InputLeaf { ref_idx, sub_idx } = edge.source() {
                    if let (Some(family), Ok(channel)) = (
                        ring_of(refs.get(usize::from(ref_idx))),
                        u8::try_from(sub_idx),
                    ) {
                        if channel < DIRECTIONS {
                            bus.entry(slot)
                                .or_default()
                                .push((family, channel, weight > 0.0));
                        }
                    }
                }
            }
        }
    }
    bus
}

impl RingRuleCensus {
    /// Census `creatures`, each a genome and its reachable node indices.
    pub fn observe<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize])>,
    ) -> Self {
        let mut census = Self::default();
        for (genome, reachable) in creatures {
            census.observe_one(genome, reachable);
        }
        census
    }

    fn observe_one(&mut self, genome: &CreatureGenome, reachable: &[usize]) {
        self.creatures += 1;
        let bus = bus(genome, reachable);
        let mut coverage = Coverage::default();
        let mut carried = 0u64;
        for (def, refs) in graphs(genome, reachable) {
            for sink in def.output_sinks() {
                let OutputSinkKind::ActionVote(vote) = sink.kind else {
                    continue;
                };
                let Some((kind, d)) = DirectedVoteKind::of_sink(vote) else {
                    continue;
                };
                if d >= DIRECTIONS {
                    continue;
                }
                for edge in sink.inputs.iter() {
                    let weight = def.edge_weight(edge, sink.kind, refs);
                    if !carries(weight) {
                        continue;
                    }
                    let independent = edge.membership().is_none();
                    let GraphSource::InputLeaf { ref_idx, sub_idx } = edge.source() else {
                        continue;
                    };
                    let reference = refs.get(usize::from(ref_idx));
                    if let Some(family) = ring_of(reference) {
                        if let Ok(channel @ 0..DIRECTIONS) = u8::try_from(sub_idx) {
                            let rule = Rule {
                                family,
                                kind,
                                offset: ring_offset(d, channel),
                                positive: weight > 0.0,
                            };
                            coverage.add(rule, d, independent);
                        }
                    } else if let Some(slot) = reference.and_then(|r| r.bus_slot(sub_idx)) {
                        let Some(sources) = u8::try_from(slot).ok().and_then(|j| bus.get(&j))
                        else {
                            continue;
                        };
                        for &(family, channel, positive) in sources {
                            let rule = Rule {
                                family,
                                kind,
                                offset: ring_offset(d, channel),
                                positive: positive == (weight > 0.0),
                            };
                            coverage.add(rule, d, independent);
                        }
                    }
                }
            }
            for (index, bundle) in def.bundles().iter().enumerate() {
                let (
                    Wiring::Ring { kind, offset },
                    BundleSource::Sensor {
                        ref_idx: ring_ref, ..
                    },
                ) = (bundle.wiring(), bundle.source())
                else {
                    continue;
                };
                let members = def
                    .members(crate::genome::cgp::BundleId::new(
                        u16::try_from(index).expect("bundles below u16::MAX"),
                    ))
                    .count() as u64;
                carried += 1;
                self.ring_bundles += 1;
                self.ring_bundle_members += members;
                let nonzero = carries(bundle.scale());
                self.ring_bundles_nonzero += u64::from(nonzero);
                let Some(family) = ring_of(refs.get(usize::from(ring_ref))) else {
                    continue;
                };
                let counts = self
                    .ring_bundle_counts
                    .entry((family, kind, offset))
                    .or_default();
                counts.count += 1;
                counts.positive += u64::from(nonzero && bundle.scale() > 0.0);
                counts.negative += u64::from(nonzero && bundle.scale() < 0.0);
                counts.members += members;
            }
        }
        self.ring_bundles_unreachable += genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(idx, _)| !reachable.contains(idx))
            .filter_map(|(_, node)| match &node.backend_def {
                BackendDef::Graph(def) => Some(
                    def.bundles()
                        .iter()
                        .filter(|bundle| matches!(bundle.wiring(), Wiring::Ring { .. }))
                        .count() as u64,
                ),
                BackendDef::Vm(_) => None,
            })
            .sum::<u64>();
        self.ring_bundle_carriers += u64::from(carried > 0);
        let general = |mask: u8| mask.count_ones() >= DIRECTION_GENERAL_COVERAGE;
        self.direction_general += u64::from(coverage.0.values().any(|&(all, _)| general(all)));
        self.direction_general_edges_only +=
            u64::from(coverage.0.values().any(|&(_, edges)| general(edges)));
        for (rule, (all, _)) in coverage.0 {
            let n = all.count_ones() as usize;
            if n > 0 {
                self.coverage.entry(rule).or_default()[n - 1] += 1;
            }
        }
    }

    /// `family kind offset count positive negative members`, one per
    /// reachable `Ring` bundle class, in key order.
    #[must_use]
    pub fn ring_bundle_rows(&self) -> Vec<String> {
        self.ring_bundle_counts
            .iter()
            .map(|(&(family, kind, offset), counts)| {
                format!(
                    "{} {kind:?} {offset} {} {} {} {}",
                    world_input_key_label(family),
                    counts.count,
                    counts.positive,
                    counts.negative,
                    counts.members
                )
            })
            .collect()
    }

    /// `family kind offset sign n1 … n8`, one per rule some creature covers,
    /// in rule order.
    #[must_use]
    pub fn rule_rows(&self) -> Vec<String> {
        self.coverage
            .iter()
            .map(|(rule, counts)| {
                let counts: Vec<String> = counts.iter().map(u64::to_string).collect();
                format!(
                    "{} {:?} {} {} {}",
                    world_input_key_label(rule.family),
                    rule.kind,
                    rule.offset,
                    if rule.positive { "+" } else { "-" },
                    counts.join(" ")
                )
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "ring_rules_tests.rs"]
mod tests;
