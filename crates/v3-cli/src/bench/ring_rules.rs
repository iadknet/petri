//! The per-world `ring_rules` block (T11.F31, T25.F03, `ring-rules-v2`): the
//! ring-rule census of the terminal population, projected from
//! `v3_core::neighborhood::ring_rules`. Rows are space-separated strings, as in
//! `input_use`, so the committed summary stays inside its budget.

use serde::{Deserialize, Serialize};
use v3_core::neighborhood::ring_rules::{RingRuleCensus, RING_RULES_VERSION};

use super::schema::Indicator;
use crate::UNDEFINED;

/// The fields of one `ring_group_rows` string, in order.
pub const RING_GROUP_ROW_FORMAT: &str = "family kind offset count positive negative members";
/// The fields of one `rule_rows` string, in order: creatures covering the
/// rule in exactly 1 to 8 directions.
pub const RULE_ROW_FORMAT: &str = "family kind offset sign n1 n2 n3 n4 n5 n6 n7 n8";

pub(super) fn undefined_ring_rules() -> Indicator<RingRules> {
    Indicator::Undefined(UNDEFINED.to_string())
}

/// One world's `ring_rules` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingRules {
    pub version: String,
    pub creatures: u64,
    pub direction_general: u64,
    pub direction_general_edges_only: u64,
    pub ring_group_carriers: u64,
    pub ring_groups: u64,
    pub ring_groups_nonzero: u64,
    pub ring_groups_unreachable: u64,
    pub ring_group_members: u64,
    pub ring_group_row_format: String,
    pub ring_group_rows: Vec<String>,
    pub rule_row_format: String,
    pub rule_rows: Vec<String>,
}

impl From<&RingRuleCensus> for RingRules {
    fn from(census: &RingRuleCensus) -> Self {
        Self {
            version: RING_RULES_VERSION.to_string(),
            creatures: census.creatures,
            direction_general: census.direction_general,
            direction_general_edges_only: census.direction_general_edges_only,
            ring_group_carriers: census.ring_group_carriers,
            ring_groups: census.ring_groups,
            ring_groups_nonzero: census.ring_groups_nonzero,
            ring_groups_unreachable: census.ring_groups_unreachable,
            ring_group_members: census.ring_group_members,
            ring_group_row_format: RING_GROUP_ROW_FORMAT.to_string(),
            ring_group_rows: census.ring_group_rows(),
            rule_row_format: RULE_ROW_FORMAT.to_string(),
            rule_rows: census.rule_rows(),
        }
    }
}

/// Census the living population from post-run state.
pub(super) fn observe_world(sim: &v3_core::simulation::Simulation) -> RingRules {
    let census = RingRuleCensus::observe(
        sim.creatures
            .values()
            .map(|creature| (creature.genome(), creature.reachable_nodes())),
    );
    RingRules::from(&census)
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::config::SimulationConfig;
    use v3_core::simulation::seed_simulation;

    #[test]
    fn a_founder_population_reads_one_rule_at_coverage_four() {
        let mut config = SimulationConfig::default();
        config.population.initial_creatures = 3;
        let sim = seed_simulation(config, 1);
        let block = observe_world(&sim);
        assert_eq!(block.version, "ring-rules-v2");
        assert_eq!(block.creatures, sim.creatures.len() as u64);
        assert_eq!(
            block.rule_rows,
            vec![format!(
                "NeighborFoodRing:0 Move 0 + 0 0 0 {} 0 0 0 0",
                block.creatures
            )]
        );
        assert!(block.ring_group_rows.is_empty());
        assert_eq!(block.ring_groups, 0);
        assert_eq!(block.direction_general, 0);
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(serde_json::from_value::<RingRules>(json).unwrap(), block);
    }
}
