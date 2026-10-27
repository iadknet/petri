//! The per-world `drives` block (T11.F30, `drives-v2`): the drive census of
//! the terminal population, projected from `v3_core::neighborhood::drives`, with
//! the canonical founder's fan-outs read by the same code.

use serde::{Deserialize, Serialize};
use v3_core::contracts::votes::DirectedVoteKind;
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::genome::analysis::mesh_reachable_nodes;
use v3_core::neighborhood::drives::{DriveCensus, KindDrives, DRIVES_VERSION};

use super::schema::Indicator;
use crate::UNDEFINED;

pub(super) fn undefined_drives() -> Indicator<Drives> {
    Indicator::Undefined(UNDEFINED.to_string())
}

/// One directed kind's row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveKindRow {
    pub kind: String,
    pub drive_carriers: u64,
    pub drive_edges: u64,
    pub drive_edges_positive: u64,
    pub drive_edges_negative: u64,
    pub drive_vm_paths: u64,
    pub fanout_creatures: u64,
    pub fanouts: u64,
    pub fanout_edges: u64,
}

impl DriveKindRow {
    fn new(kind: DirectedVoteKind, row: &KindDrives) -> Self {
        Self {
            kind: format!("{kind:?}"),
            drive_carriers: row.drive_carriers,
            drive_edges: row.drive_edges,
            drive_edges_positive: row.drive_edges_positive,
            drive_edges_negative: row.drive_edges_negative,
            drive_vm_paths: row.drive_vm_paths,
            fanout_creatures: row.fanout_creatures,
            fanouts: row.fanouts,
            fanout_edges: row.fanout_edges,
        }
    }
}

/// One world's `drives` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drives {
    pub version: String,
    pub creatures: u64,
    pub kinds: Vec<DriveKindRow>,
    pub founder_fanouts: u64,
    pub founder_fanout_edges: u64,
}

impl Drives {
    fn new(census: &DriveCensus, founder: &DriveCensus) -> Self {
        let (founder_fanouts, founder_fanout_edges) = founder.fanout_totals();
        Self {
            version: DRIVES_VERSION.to_string(),
            creatures: census.creatures,
            kinds: DirectedVoteKind::ALL
                .iter()
                .zip(&census.kinds)
                .map(|(&kind, row)| DriveKindRow::new(kind, row))
                .collect(),
            founder_fanouts,
            founder_fanout_edges,
        }
    }
}

/// Census the living population from post-run state, beside the founder the
/// run seeds.
pub(super) fn observe_world(sim: &v3_core::simulation::Simulation) -> Drives {
    let census = DriveCensus::observe(
        sim.creatures
            .values()
            .map(|creature| (creature.genome(), creature.reachable_nodes())),
    );
    let founder = founder_genome_with_age_gate(
        sim.config.population.founder_profile,
        &sim.config.energy.lifecycle,
    );
    let reachable = mesh_reachable_nodes(&founder);
    Drives::new(
        &census,
        &DriveCensus::observe([(&founder, reachable.as_slice())]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::config::SimulationConfig;
    use v3_core::simulation::seed_simulation;

    #[test]
    fn a_founder_population_carries_no_drive_and_the_founder_fanouts() {
        let mut config = SimulationConfig::default();
        config.population.initial_creatures = 3;
        let sim = seed_simulation(config, 1);
        let block = observe_world(&sim);
        assert_eq!(block.version, "drives-v2");
        assert_eq!(block.creatures, sim.creatures.len() as u64);
        assert_eq!((block.founder_fanouts, block.founder_fanout_edges), (4, 16));
        let kinds: Vec<_> = block.kinds.iter().map(|row| row.kind.as_str()).collect();
        assert_eq!(kinds, ["Move", "Reproduce", "StealEnergy", "GiveEnergy"]);
        assert!(block.kinds.iter().all(|row| row.drive_carriers == 0));
        assert_eq!(block.kinds[0].fanout_creatures, block.creatures);
        assert_eq!(block.kinds[0].fanouts, 3 * block.creatures);
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(serde_json::from_value::<Drives>(json).unwrap(), block);
    }
}
