//! The per-sample `pair_bundles` block (T25.F04, `pair-bundles-v2` since
//! T25.F10): the `DirectionPair` genes the living population carries on
//! reachable nodes, by `(family, kind, offset)`, their direction members,
//! fact members and members with a nonzero own correction, with the two run
//! counters that read whether evolution keeps such a gene and moves its
//! scale off zero.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use v3_core::contracts::votes::DirectedVoteKind;
use v3_core::genome::cgp::{BundleId, Shape};
use v3_core::genome::{BackendDef, CreatureGenome};
use v3_core::mutation::types::MutationOperator;

/// The block's definition version.
pub const PAIR_BUNDLES_VERSION: &str = "pair-bundles-v2";

/// A scale or own correction off zero: `|w| > 1e-3`, the census weight
/// floor.
const NONZERO_FLOOR: f32 = v3_core::neighborhood::drives::PATH_WEIGHT_FLOOR;

/// One `(family, kind, offset)` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairBundleRow {
    pub family: String,
    pub kind: DirectedVoteKind,
    pub offset: u8,
    pub count: u64,
    pub nonzero: u64,
    pub positive: u64,
    pub negative: u64,
    /// Direction members present.
    pub members: u64,
    /// Fact members present (T25.F10).
    pub facts: u64,
    /// Members of either role with `|own_weight| > 1e-3` (T25.F10).
    pub own_nonzero: u64,
}

/// One sample's `pair_bundles` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairBundles {
    pub version: String,
    /// Living creatures.
    pub creatures: u64,
    /// Creatures carrying a `DirectionPair` bundle on a reachable node.
    pub carriers: u64,
    /// `DirectionPair` bundles on reachable nodes.
    pub bundles: u64,
    /// Of those, bundles with `|scale| > 1e-3`.
    pub bundles_nonzero: u64,
    /// `DirectionPair` bundles on unreachable nodes.
    pub bundles_unreachable: u64,
    /// Direction members present across the reachable bundles.
    pub members: u64,
    /// Fact members present across the reachable bundles (T25.F10).
    pub facts: u64,
    /// Members of either role across the reachable bundles with
    /// `|own_weight| > 1e-3` (T25.F10).
    pub own_nonzero: u64,
    /// Cumulative applied `Graph.AddDirectionPairBundle` events.
    pub add_applied_total: u64,
    /// Cumulative births witnessing an inherited nonzero pair bundle.
    pub inherited_nonzero_births_total: u64,
    pub rows: Vec<PairBundleRow>,
}

impl PairBundles {
    /// Census `creatures`, each a genome and its reachable node indices,
    /// with the run's two counters.
    pub fn census<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize])>,
        add_applied_total: u64,
        inherited_nonzero_births_total: u64,
    ) -> Self {
        let mut block = Self {
            version: PAIR_BUNDLES_VERSION.to_string(),
            creatures: 0,
            carriers: 0,
            bundles: 0,
            bundles_nonzero: 0,
            bundles_unreachable: 0,
            members: 0,
            facts: 0,
            own_nonzero: 0,
            add_applied_total,
            inherited_nonzero_births_total,
            rows: Vec::new(),
        };
        let mut rows: BTreeMap<(&'static str, DirectedVoteKind, u8), PairBundleRow> =
            BTreeMap::new();
        for (genome, reachable) in creatures {
            block.creatures += 1;
            let mut carried = false;
            for (index, node) in genome.nodes.iter().enumerate() {
                let BackendDef::Graph(def) = &node.backend_def else {
                    continue;
                };
                for (id, bundle) in def.bundles().iter().enumerate() {
                    let Shape::DirectionPair { ref_idx, .. } = bundle.shape() else {
                        continue;
                    };
                    if !reachable.contains(&index) {
                        block.bundles_unreachable += 1;
                        continue;
                    }
                    carried = true;
                    let id = BundleId::new(u16::try_from(id).expect("bundles below u16::MAX"));
                    let (mut members, mut facts, mut own_nonzero) = (0u64, 0u64, 0u64);
                    for member in def.members(id) {
                        if def.is_fact_member(member) {
                            facts += 1;
                        } else {
                            members += 1;
                        }
                        own_nonzero +=
                            u64::from(def.edge_at(member).own_weight().abs() > NONZERO_FLOOR);
                    }
                    let nonzero = bundle.scale().abs() > NONZERO_FLOOR;
                    block.bundles += 1;
                    block.bundles_nonzero += u64::from(nonzero);
                    block.members += members;
                    block.facts += facts;
                    block.own_nonzero += own_nonzero;
                    let family = node.input_refs.get(usize::from(ref_idx)).map_or(
                        "unresolved",
                        |reference| match reference {
                            v3_core::contracts::InputReference::World(key) => key.as_key(),
                            _ => "unresolved",
                        },
                    );
                    let row = rows
                        .entry((family, bundle.kind(), bundle.offset()))
                        .or_insert_with(|| PairBundleRow {
                            family: family.to_string(),
                            kind: bundle.kind(),
                            offset: bundle.offset(),
                            count: 0,
                            nonzero: 0,
                            positive: 0,
                            negative: 0,
                            members: 0,
                            facts: 0,
                            own_nonzero: 0,
                        });
                    row.count += 1;
                    row.nonzero += u64::from(nonzero);
                    row.positive += u64::from(nonzero && bundle.scale() > 0.0);
                    row.negative += u64::from(nonzero && bundle.scale() < 0.0);
                    row.members += members;
                    row.facts += facts;
                    row.own_nonzero += own_nonzero;
                }
            }
            block.carriers += u64::from(carried);
        }
        block.rows = rows.into_values().collect();
        block
    }

    /// The living population's block from post-tick state; `None` at
    /// extinction.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Option<Self> {
        if sim.creatures.is_empty() {
            return None;
        }
        let add_applied_total = sim
            .stats
            .mutation_events_applied_total_by_operator
            .get(&MutationOperator::GraphAddDirectionPairBundle)
            .copied()
            .unwrap_or(0);
        Some(Self::census(
            sim.creatures
                .values()
                .map(|creature| (creature.genome(), creature.reachable_nodes())),
            add_applied_total,
            sim.stats.pair_bundle_inherited_nonzero_births_total,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::config::SimulationConfig;
    use v3_core::contracts::{InputReference, WorldInputKey};
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::simulation::seed_simulation;

    /// How a fixture gene is built: `Plain` the twelve direction members,
    /// `DropOne` minus its first direction member, `Facts` with its fact
    /// members, one of them removed (a fact hole), and own corrections on
    /// one direction member (0.5), one fact member (-0.25) and one more
    /// direction member exactly at the floor (not counted).
    #[derive(Clone, Copy)]
    enum Gene {
        Plain,
        DropOne,
        Facts,
    }

    /// The founder with an occupancy summary on every Graph node and, per
    /// `(node, kind, offset, scale, gene)`, one pair gene there.
    fn grouped(bundles: &[(usize, DirectedVoteKind, u8, f32, Gene)]) -> CreatureGenome {
        let mut genome = v3alpha1_founder_genome();
        let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
        for &(index, kind, offset, scale, gene) in bundles {
            let node = &mut genome.nodes[index];
            let ref_idx = match node.input_refs.iter().position(|r| *r == area) {
                Some(found) => u16::try_from(found).unwrap(),
                None => {
                    node.input_refs.push(area.clone());
                    u16::try_from(node.input_refs.len() - 1).unwrap()
                }
            };
            let BackendDef::Graph(def) = &mut node.backend_def else {
                panic!("a Graph node");
            };
            let layout = area.direction_pairs()[0];
            let facts = if matches!(gene, Gene::Facts) {
                layout.facts
            } else {
                &[]
            };
            let id = def.add_pair_bundle(ref_idx, 0, layout, kind, offset, facts);
            def.bundle_mut(id).set_scale(scale);
            match gene {
                Gene::Plain => {}
                Gene::DropOne => {
                    let member = def.members(id).next().unwrap();
                    def.remove_edge(member);
                }
                Gene::Facts => {
                    let hole = def
                        .members(id)
                        .filter(|&m| def.is_fact_member(m))
                        .last()
                        .unwrap();
                    def.remove_edge(hole);
                    let fact = def.members(id).find(|&m| def.is_fact_member(m)).unwrap();
                    def.edge_mut(fact).set_weight(-0.25);
                    let directions: Vec<_> = def
                        .members(id)
                        .filter(|&m| !def.is_fact_member(m))
                        .collect();
                    def.edge_mut(directions[0]).set_weight(0.5);
                    def.edge_mut(directions[1]).set_weight(NONZERO_FLOOR);
                }
            }
        }
        genome
    }

    #[test]
    fn the_block_counts_reachable_bundles_by_family_kind_and_offset() {
        let move_ = DirectedVoteKind::Move;
        let carrier = grouped(&[
            (0, move_, 2, 0.5, Gene::Plain),
            (0, move_, 2, -0.25, Gene::DropOne),
            // Exactly at the floor: not nonzero.
            (
                0,
                DirectedVoteKind::StealEnergy,
                0,
                NONZERO_FLOOR,
                Gene::Plain,
            ),
            // A lone positive bundle: the sign split is not symmetric.
            (0, move_, 0, 0.5, Gene::Plain),
            // A gene with facts and a fact hole (T25.F10).
            (0, DirectedVoteKind::Reproduce, 4, 0.0, Gene::Facts),
            (1, move_, 2, 0.5, Gene::Plain),
        ]);
        let plain = v3alpha1_founder_genome();
        // Node 1 is unreachable for the carrier: its bundle is counted apart.
        let carrier_reach = [0usize];
        let plain_reach = [0usize, 1];
        let block = PairBundles::census(
            [(&carrier, &carrier_reach[..]), (&plain, &plain_reach[..])],
            7,
            3,
        );
        assert_eq!(block.version, PAIR_BUNDLES_VERSION);
        assert_eq!((block.creatures, block.carriers, block.bundles), (2, 1, 5));
        assert_eq!((block.bundles_nonzero, block.bundles_unreachable), (3, 1));
        assert_eq!(block.members, 12 + 11 + 12 + 12 + 12);
        assert_eq!((block.facts, block.own_nonzero), (2, 2));
        assert_eq!(
            (
                block.add_applied_total,
                block.inherited_nonzero_births_total
            ),
            (7, 3)
        );
        let row =
            |kind, offset, count, nonzero, positive, negative, members, facts| PairBundleRow {
                family: "AreaOccupancySummary".to_string(),
                kind,
                offset,
                count,
                nonzero,
                positive,
                negative,
                members,
                facts,
                own_nonzero: 0,
            };
        assert_eq!(
            block.rows,
            vec![
                row(move_, 0, 1, 1, 1, 0, 12, 0),
                row(move_, 2, 2, 2, 1, 1, 23, 0),
                PairBundleRow {
                    own_nonzero: 2,
                    ..row(DirectedVoteKind::Reproduce, 4, 1, 0, 0, 0, 12, 2)
                },
                row(DirectedVoteKind::StealEnergy, 0, 1, 0, 0, 0, 12, 0),
            ]
        );
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["rows"][0]["kind"], "Move");
    }

    #[test]
    fn the_block_is_null_at_extinction() {
        let mut config = SimulationConfig::default();
        config.population.initial_creatures = 2;
        let mut sim = seed_simulation(config, 3);
        let block = PairBundles::observe(&sim).expect("a living population");
        assert_eq!((block.creatures, block.bundles), (2, 0));
        sim.creatures.clear();
        assert_eq!(PairBundles::observe(&sim), None);
    }
}
