//! The per-sample `pair_groups` block (T25.F04, `pair-groups-v1`): the
//! `DirectionPair` edge groups the living population carries on reachable
//! nodes, by `(family, kind, offset)`, with the two run counters that read
//! whether evolution keeps such a group and moves its scale off zero.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use v3_core::creature::genome::cgp::{GroupId, Shape};
use v3_core::creature::genome::vote::DirectedVoteKind;
use v3_core::creature::genome::{BackendDef, CreatureGenome};
use v3_core::mutation::types::MutationOperator;

/// The block's definition version.
pub const PAIR_GROUPS_VERSION: &str = "pair-groups-v1";

/// A scale off zero: `|scale| > 1e-3`, the census weight floor.
const NONZERO_FLOOR: f32 = v3_core::creature::drives::PATH_WEIGHT_FLOOR;

/// One `(family, kind, offset)` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairGroupRow {
    pub family: String,
    pub kind: DirectedVoteKind,
    pub offset: u8,
    pub count: u64,
    pub nonzero: u64,
    pub positive: u64,
    pub negative: u64,
    pub members: u64,
}

/// One sample's `pair_groups` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairGroups {
    pub version: String,
    /// Living creatures.
    pub creatures: u64,
    /// Creatures carrying a `DirectionPair` group on a reachable node.
    pub carriers: u64,
    /// `DirectionPair` groups on reachable nodes.
    pub groups: u64,
    /// Of those, groups with `|scale| > 1e-3`.
    pub groups_nonzero: u64,
    /// `DirectionPair` groups on unreachable nodes.
    pub groups_unreachable: u64,
    /// Members present across the reachable groups.
    pub members: u64,
    /// Cumulative applied `Graph.AddDirectionPairGroup` events.
    pub add_applied_total: u64,
    /// Cumulative births witnessing an inherited nonzero pair group.
    pub inherited_nonzero_births_total: u64,
    pub rows: Vec<PairGroupRow>,
}

impl PairGroups {
    /// Census `creatures`, each a genome and its reachable node indices,
    /// with the run's two counters.
    pub fn census<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize])>,
        add_applied_total: u64,
        inherited_nonzero_births_total: u64,
    ) -> Self {
        let mut block = Self {
            version: PAIR_GROUPS_VERSION.to_string(),
            creatures: 0,
            carriers: 0,
            groups: 0,
            groups_nonzero: 0,
            groups_unreachable: 0,
            members: 0,
            add_applied_total,
            inherited_nonzero_births_total,
            rows: Vec::new(),
        };
        let mut rows: BTreeMap<(&'static str, DirectedVoteKind, u8), PairGroupRow> =
            BTreeMap::new();
        for (genome, reachable) in creatures {
            block.creatures += 1;
            let mut carried = false;
            for (index, node) in genome.nodes.iter().enumerate() {
                let BackendDef::Graph(def) = &node.backend_def else {
                    continue;
                };
                for (id, group) in def.groups().iter().enumerate() {
                    let Shape::DirectionPair { ref_idx, .. } = group.shape() else {
                        continue;
                    };
                    if !reachable.contains(&index) {
                        block.groups_unreachable += 1;
                        continue;
                    }
                    carried = true;
                    let members = def
                        .members(GroupId::new(
                            u16::try_from(id).expect("groups below u16::MAX"),
                        ))
                        .count() as u64;
                    let nonzero = group.scale().abs() > NONZERO_FLOOR;
                    block.groups += 1;
                    block.groups_nonzero += u64::from(nonzero);
                    block.members += members;
                    let family = node.input_refs.get(usize::from(ref_idx)).map_or(
                        "unresolved",
                        |reference| match reference {
                            v3_core::contracts::InputReference::World(key) => key.as_key(),
                            _ => "unresolved",
                        },
                    );
                    let row = rows
                        .entry((family, group.kind(), group.offset()))
                        .or_insert_with(|| PairGroupRow {
                            family: family.to_string(),
                            kind: group.kind(),
                            offset: group.offset(),
                            count: 0,
                            nonzero: 0,
                            positive: 0,
                            negative: 0,
                            members: 0,
                        });
                    row.count += 1;
                    row.nonzero += u64::from(nonzero);
                    row.positive += u64::from(nonzero && group.scale() > 0.0);
                    row.negative += u64::from(nonzero && group.scale() < 0.0);
                    row.members += members;
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
            .get(&MutationOperator::GraphAddDirectionPairGroup)
            .copied()
            .unwrap_or(0);
        Some(Self::census(
            sim.creatures
                .values()
                .map(|creature| (&creature.genome, &*creature.cached_reachable_nodes)),
            add_applied_total,
            sim.stats.pair_group_inherited_nonzero_births_total,
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

    /// The founder with an occupancy summary on every Graph node and, per
    /// `(node, kind, offset, scale, drop_one)`, one pair group there.
    fn grouped(groups: &[(usize, DirectedVoteKind, u8, f32, bool)]) -> CreatureGenome {
        let mut genome = v3alpha1_founder_genome();
        let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
        for &(index, kind, offset, scale, drop_one) in groups {
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
            let id = def.add_pair_group(ref_idx, 0, area.direction_pairs()[0], kind, offset);
            def.group_mut(id).set_scale(scale);
            if drop_one {
                let member = def.members(id).next().unwrap();
                def.remove_edge(member);
            }
        }
        genome
    }

    #[test]
    fn the_block_counts_reachable_groups_by_family_kind_and_offset() {
        let move_ = DirectedVoteKind::Move;
        let carrier = grouped(&[
            (0, move_, 2, 0.5, false),
            (0, move_, 2, -0.25, true),
            (0, DirectedVoteKind::StealEnergy, 0, 1e-4, false),
            (1, move_, 2, 0.5, false),
        ]);
        let plain = v3alpha1_founder_genome();
        // Node 1 is unreachable for the carrier: its group is counted apart.
        let carrier_reach = [0usize];
        let plain_reach = [0usize, 1];
        let block = PairGroups::census(
            [(&carrier, &carrier_reach[..]), (&plain, &plain_reach[..])],
            7,
            3,
        );
        assert_eq!(block.version, PAIR_GROUPS_VERSION);
        assert_eq!((block.creatures, block.carriers, block.groups), (2, 1, 3));
        assert_eq!((block.groups_nonzero, block.groups_unreachable), (2, 1));
        assert_eq!(block.members, 12 + 11 + 12);
        assert_eq!(
            (
                block.add_applied_total,
                block.inherited_nonzero_births_total
            ),
            (7, 3)
        );
        let row = |kind, offset, count, nonzero, positive, negative, members| PairGroupRow {
            family: "AreaOccupancySummary".to_string(),
            kind,
            offset,
            count,
            nonzero,
            positive,
            negative,
            members,
        };
        assert_eq!(
            block.rows,
            vec![
                row(move_, 2, 2, 2, 1, 1, 23),
                row(DirectedVoteKind::StealEnergy, 0, 1, 0, 0, 0, 12),
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
        let block = PairGroups::observe(&sim).expect("a living population");
        assert_eq!((block.creatures, block.groups), (2, 0));
        sim.creatures.clear();
        assert_eq!(PairGroups::observe(&sim), None);
    }
}
