//! The bundle census (T25.F11, T25.F12, `bundles-v2`): the edge bundles a
//! population carries, by wiring (`Plain`, `Ring`, `DirectionPair`), with
//! their genomic and learned scales off zero, their members and members with
//! a nonzero own modifier, their learning (the bundles with each slot, the
//! members by policy and the crediting members), the bundles on unreachable
//! nodes, and rows by wiring and target (a geometric wiring's kind, a
//! `Plain` wiring's sink).
//!
//! One creature's reading ([`CreatureBundles`]) is shared by the population
//! census ([`BundleCensus`]) and the ring-rule census, which takes its `Ring`
//! totals from it. Pure structure: genomes, cached reachability and, when
//! sampled, a runtime's learned state; no brain executes.

use std::collections::BTreeMap;

use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::genome::cgp::{BundleId, Wiring};
use crate::genome::{BackendDef, CreatureGenome, MemberPolicy};
use crate::mutation::types::{wiring_index, WIRING_KEYS};
use crate::neighborhood::PATH_WEIGHT_FLOOR;
use crate::runtime::plasticity::LearnedState;

/// The wirings, indexed by [`wiring_index`]: `Plain`, `Ring`,
/// `DirectionPair`.
pub const WIRING_COUNT: usize = WIRING_KEYS.len();

/// [`wiring_index`] of a `Ring` wiring.
pub const RING: usize = wiring_index(&Wiring::Ring {
    kind: DirectedVoteKind::Move,
    offset: 0,
});

/// One wiring's totals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WiringCounts {
    /// Bundles on reachable nodes.
    pub bundles: u64,
    /// Reachable bundles with `|scale| > PATH_WEIGHT_FLOOR`.
    pub genomic_nonzero: u64,
    /// Reachable bundles whose runtime learned scale is initialized and off
    /// zero; `0` where no runtime is sampled.
    pub learned_nonzero: u64,
    /// Members present on reachable bundles.
    pub members: u64,
    /// Members with `|own_weight| > PATH_WEIGHT_FLOOR`.
    pub own_nonzero: u64,
    /// Bundles on unreachable nodes.
    pub unreachable: u64,
    /// Reachable bundles with `scale_learning` (T25.F12).
    pub scale_learning: u64,
    /// Reachable bundles with `member_default` (T25.F12).
    pub member_default: u64,
    /// Members of reachable bundles by policy, in [`POLICY_KEYS`] order.
    pub members_by_policy: [u64; 3],
    /// Members of reachable bundles with `credits`.
    pub crediting: u64,
}

/// The member policy keys, by [`policy_index`].
pub const POLICY_KEYS: [&str; 3] = [
    MemberPolicy::Inherit.as_key(),
    MemberPolicy::Static.as_key(),
    "own",
];

/// The position of `policy`'s key in [`POLICY_KEYS`].
#[must_use]
pub const fn policy_index(policy: MemberPolicy) -> usize {
    match policy {
        MemberPolicy::Inherit => 0,
        MemberPolicy::Static => 1,
        MemberPolicy::Own(_) => 2,
    }
}

impl WiringCounts {
    fn merge(&mut self, other: &Self) {
        self.bundles += other.bundles;
        self.genomic_nonzero += other.genomic_nonzero;
        self.learned_nonzero += other.learned_nonzero;
        self.members += other.members;
        self.own_nonzero += other.own_nonzero;
        self.unreachable += other.unreachable;
        self.scale_learning += other.scale_learning;
        self.member_default += other.member_default;
        for (total, count) in self
            .members_by_policy
            .iter_mut()
            .zip(other.members_by_policy)
        {
            *total += count;
        }
        self.crediting += other.crediting;
    }
}

/// A row's target: a `Plain` wiring's vote sink or a geometric wiring's
/// kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleTarget {
    Sink(VoteSink),
    Kind(DirectedVoteKind),
}

impl BundleTarget {
    #[must_use]
    pub const fn of(wiring: Wiring) -> Self {
        match wiring {
            Wiring::Plain { sink } => Self::Sink(sink),
            Wiring::Ring { kind, .. } | Wiring::DirectionPair { kind, .. } => Self::Kind(kind),
        }
    }
}

/// One `(wiring, target)` row over reachable bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleRow {
    /// [`wiring_index`] of the row's wiring.
    pub wiring: usize,
    pub target: BundleTarget,
    pub count: u64,
    pub genomic_nonzero: u64,
    pub members: u64,
}

/// Rows in first-seen order; a key appears once.
fn add_row(rows: &mut Vec<BundleRow>, row: BundleRow) {
    match rows
        .iter_mut()
        .find(|seen| seen.wiring == row.wiring && seen.target == row.target)
    {
        Some(seen) => {
            seen.count += row.count;
            seen.genomic_nonzero += row.genomic_nonzero;
            seen.members += row.members;
        }
        None => rows.push(row),
    }
}

fn off_zero(weight: f32) -> bool {
    weight.abs() > PATH_WEIGHT_FLOOR
}

/// One creature's bundles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreatureBundles {
    /// Totals by [`wiring_index`].
    pub wirings: [WiringCounts; WIRING_COUNT],
    /// Rows by `(wiring, target)`, in first-seen order.
    pub rows: Vec<BundleRow>,
}

impl CreatureBundles {
    /// Read `genome`'s Graph nodes against its `reachable` node indices and,
    /// when sampled, its runtime's `learned` scales.
    #[must_use]
    pub fn of(
        genome: &CreatureGenome,
        reachable: &[usize],
        learned: Option<&LearnedState>,
    ) -> Self {
        let mut reading = Self::default();
        for (index, node) in genome.nodes.iter().enumerate() {
            let BackendDef::Graph(def) = &node.backend_def else {
                continue;
            };
            if !reachable.contains(&index) {
                for bundle in def.bundles() {
                    reading.wirings[wiring_index(&bundle.wiring())].unreachable += 1;
                }
                continue;
            }
            let scales: BTreeMap<BundleId, Option<f32>> = learned
                .map(|learned| {
                    learned
                        .scales(index)
                        .map(|scale| (scale.bundle, scale.learned))
                        .collect()
                })
                .unwrap_or_default();
            for (id, bundle) in def.indexed_bundles() {
                let wiring = bundle.wiring();
                let row = wiring_index(&wiring);
                let members = def.members(id).count() as u64;
                let own_nonzero = def
                    .members(id)
                    .filter(|&member| off_zero(def.edge_at(member).own_weight()))
                    .count() as u64;
                let genomic = off_zero(bundle.scale());
                let learned_nonzero = scales.get(&id).copied().flatten().is_some_and(off_zero);
                let total = &mut reading.wirings[row];
                total.scale_learning += u64::from(bundle.scale_learning().is_some());
                total.member_default += u64::from(bundle.member_default().is_some());
                for member in def.members(id) {
                    let membership = def.edge_at(member).membership().expect("a member");
                    total.members_by_policy[policy_index(membership.policy)] += 1;
                    total.crediting += u64::from(membership.credits);
                }
                total.bundles += 1;
                total.genomic_nonzero += u64::from(genomic);
                total.learned_nonzero += u64::from(learned_nonzero);
                total.members += members;
                total.own_nonzero += own_nonzero;
                add_row(
                    &mut reading.rows,
                    BundleRow {
                        wiring: row,
                        target: BundleTarget::of(wiring),
                        count: 1,
                        genomic_nonzero: u64::from(genomic),
                        members,
                    },
                );
            }
        }
        reading
    }

    /// Whether the creature carries a bundle on a reachable node.
    #[must_use]
    pub fn carries(&self) -> bool {
        self.wirings.iter().any(|wiring| wiring.bundles > 0)
    }
}

/// The bundle census of one population.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BundleCensus {
    pub creatures: u64,
    /// Creatures carrying a bundle on a reachable node.
    pub carriers: u64,
    /// Whether any creature's runtime was sampled: the learned counts are
    /// readings only then.
    pub learned_sampled: bool,
    /// Totals by [`wiring_index`].
    pub wirings: [WiringCounts; WIRING_COUNT],
    /// Rows by `(wiring, target)`, in first-seen order.
    pub rows: Vec<BundleRow>,
}

impl BundleCensus {
    /// Census `creatures`, each a genome, its reachable node indices and,
    /// when sampled, its runtime's learned state.
    pub fn observe<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize], Option<&'a LearnedState>)>,
    ) -> Self {
        let mut census = Self::default();
        for (genome, reachable, learned) in creatures {
            census.creatures += 1;
            census.learned_sampled |= learned.is_some();
            let reading = CreatureBundles::of(genome, reachable, learned);
            census.carriers += u64::from(reading.carries());
            for (total, wiring) in census.wirings.iter_mut().zip(&reading.wirings) {
                total.merge(wiring);
            }
            for row in reading.rows {
                add_row(&mut census.rows, row);
            }
        }
        census
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{FieldSet, SensorContext, MEMORY_BANK};
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::genome::cgp::{BundleBirth, BundleSource};

    /// The ring-rule census takes its `Ring` totals from this census, which
    /// counts every `Ring` wiring, while it counts only `Ring` wirings on a
    /// sensor source: the two agree because a `Ring` wiring always has one.
    /// A memory bank resolves to a `Plain` set, which no `Ring` wiring fits,
    /// so the graph refuses the bundle and `validate_graph` rejects a stored
    /// one.
    #[test]
    fn a_ring_wiring_never_sits_on_a_memory_source() {
        assert!(matches!(MEMORY_BANK, FieldSet::Plain { .. }));
        for previous in [false, true] {
            assert_eq!(
                BundleSource::Memory { previous }.resolve(&[]),
                Some(MEMORY_BANK)
            );
        }
        let context = SensorContext::WIDEST;
        let mut genome = v3alpha1_founder_genome();
        let node = genome
            .nodes
            .iter_mut()
            .find(|node| matches!(node.backend_def, BackendDef::Graph(_)))
            .expect("the founder carries a Graph node");
        for kind in DirectedVoteKind::ALL {
            for offset in 0..8 {
                for previous in [false, true] {
                    let wiring = Wiring::Ring { kind, offset };
                    assert!(!wiring.fits(&MEMORY_BANK));
                    assert!(node
                        .add_bundle(
                            BundleSource::Memory { previous },
                            BundleBirth {
                                field_set: 0,
                                wiring,
                                scale: 1.0,
                            },
                            &context,
                        )
                        .is_err());
                }
            }
        }
    }
}
