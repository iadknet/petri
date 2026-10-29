//! The population census: the means, extremes and sums over the living
//! population that the benchmark sample, the NDJSON stream, the telemetry
//! gauges and the server's status and health payloads carry.
//!
//! Each quantity has one definition, computed here once: integer sums
//! divided in `f64`, in the population's order as given. Means are `0` for an
//! empty population except mean energy, which is `None` there: each output
//! applies its own extinction value, and an `f32` output only casts.
//!
//! Observation only: it reads creature state the tick already wrote.

use std::collections::BTreeMap;

use crate::contracts::WorldInputKey;
use crate::creature::state::CreatureState;
use crate::genome::BackendDef;
use crate::simulation::Simulation;

/// The living population, in the order a census reads it.
#[derive(Clone)]
pub struct PopulationCensus<'a> {
    creatures: Vec<&'a CreatureState>,
}

/// Means of the living population's structure and lineage depth: total
/// genome size (junk included), mesh node count, generation, and wired
/// output sinks (Graph sinks with at least one edge). All `0.0` for an
/// empty population.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StructureMeans {
    pub genome_size: f64,
    pub mesh_nodes: f64,
    pub generation: f64,
    /// T25.F02: the sinks the unit rule stopped charging a unit for.
    pub wired_sinks: f64,
}

/// Functional complexity over the living population: the `f32` mean the
/// server reports, the minimum and the maximum, all `0` when empty.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ComplexityRange {
    pub mean: f32,
    pub min: u32,
    pub max: u32,
}

/// The litters carried at one tick: their count and summed energy.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PendingBirths {
    pub count: u64,
    pub litter_energy: f64,
}

impl<'a> PopulationCensus<'a> {
    /// The census of `creatures`, read in the order given.
    pub fn new(creatures: impl IntoIterator<Item = &'a CreatureState>) -> Self {
        Self {
            creatures: creatures.into_iter().collect(),
        }
    }

    /// The census of `sim`'s living population, in its iteration order.
    #[must_use]
    pub fn of(sim: &'a Simulation) -> Self {
        Self::new(sim.creatures.values())
    }

    #[must_use]
    pub fn population(&self) -> usize {
        self.creatures.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.creatures.is_empty()
    }

    fn mean(&self, total: u64) -> f64 {
        total as f64 / self.creatures.len() as f64
    }

    /// Mean energy: an `f64` sum over the `f32` energies in population order
    /// over the count, the one definition the benchmark sample, the NDJSON
    /// stream, the server payloads and `petri.tick.mean_energy` read. `None`
    /// for an empty population.
    #[must_use]
    pub fn mean_energy(&self) -> Option<f64> {
        if self.creatures.is_empty() {
            return None;
        }
        let total: f64 = self.creatures.iter().map(|c| f64::from(c.energy)).sum();
        Some(total / self.creatures.len() as f64)
    }

    /// The structure and lineage-depth means.
    #[must_use]
    pub fn structure_means(&self) -> StructureMeans {
        if self.creatures.is_empty() {
            return StructureMeans::default();
        }
        let [mut genome_size, mut mesh_nodes, mut generation, mut wired_sinks] = [0u64; 4];
        for creature in &self.creatures {
            genome_size += u64::from(creature.genome_size());
            mesh_nodes += creature.genome().nodes.len() as u64;
            generation += creature.generation;
            for node in &creature.genome().nodes {
                if let BackendDef::Graph(graph) = node.backend_def() {
                    wired_sinks += graph
                        .output_sinks()
                        .iter()
                        .filter(|sink| !sink.inputs.is_empty())
                        .count() as u64;
                }
            }
        }
        StructureMeans {
            genome_size: self.mean(genome_size),
            mesh_nodes: self.mean(mesh_nodes),
            generation: self.mean(generation),
            wired_sinks: self.mean(wired_sinks),
        }
    }

    /// The deepest generation alive, `0` when empty.
    #[must_use]
    pub fn generation_max(&self) -> u64 {
        self.creatures
            .iter()
            .map(|c| c.generation)
            .max()
            .unwrap_or(0)
    }

    /// Distinct world inputs each creature's last mesh run read, averaged;
    /// `0.0` when empty.
    #[must_use]
    pub fn mean_world_inputs_read(&self) -> f64 {
        if self.creatures.is_empty() {
            return 0.0;
        }
        self.mean(
            self.creatures
                .iter()
                .map(|c| c.last_mesh_world_reads.len() as u64)
                .sum(),
        )
    }

    /// The functional complexity range.
    #[must_use]
    pub fn complexity(&self) -> ComplexityRange {
        if self.creatures.is_empty() {
            return ComplexityRange::default();
        }
        let mut sum = 0u64;
        let mut min = u32::MAX;
        let mut max = 0u32;
        for creature in &self.creatures {
            let c = creature.complexity();
            sum += u64::from(c);
            min = min.min(c);
            max = max.max(c);
        }
        ComplexityRange {
            mean: sum as f32 / self.creatures.len() as f32,
            min,
            max,
        }
    }

    /// The live VM world-input reads, summed per key over the population.
    #[must_use]
    pub fn live_vm_world_inputs(&self) -> BTreeMap<WorldInputKey, u64> {
        let mut reads = BTreeMap::new();
        for creature in &self.creatures {
            for &(key, count) in creature.live_vm_world_inputs() {
                *reads.entry(key).or_insert(0) += u64::from(count);
            }
        }
        reads
    }

    /// The litters still carried, their energy summed in `f64` in
    /// population order.
    #[must_use]
    pub fn pending_births(&self) -> PendingBirths {
        let (count, litter_energy) = self
            .creatures
            .iter()
            .filter_map(|creature| creature.pending_birth)
            .fold((0, 0.0), |(count, energy), pending| {
                (count + 1, energy + f64::from(pending.litter_energy))
            });
        PendingBirths {
            count,
            litter_energy,
        }
    }

    /// The creatures' lineage ids, in population order.
    pub fn lineage_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.creatures.iter().map(|c| c.lineage_id)
    }

    /// The creatures, in population order.
    pub fn creatures(&self) -> impl Iterator<Item = &'a CreatureState> + '_ {
        self.creatures.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::simulation::seed_simulation;

    fn sim(creatures: u32) -> Simulation {
        let mut config = SimulationConfig::default();
        config.world.width = 32;
        config.world.height = 32;
        config.population.initial_creatures = creatures;
        seed_simulation(config, 42)
    }

    #[test]
    fn an_empty_population_reports_zero_means_and_no_benchmark_energy() {
        let sim = sim(0);
        let census = PopulationCensus::of(&sim);
        assert_eq!(census.structure_means(), StructureMeans::default());
        assert_eq!(census.mean_energy(), None);
        assert_eq!(census.complexity(), ComplexityRange::default());
        assert_eq!(census.generation_max(), 0);
        assert_eq!(census.pending_births(), PendingBirths::default());
    }

    #[test]
    fn founders_carry_two_mesh_nodes_generation_zero_and_seventeen_wired_sinks() {
        let sim = sim(4);
        let census = PopulationCensus::of(&sim);
        let expected_genome_size =
            f64::from(sim.creatures.values().map(|c| c.genome_size()).sum::<u32>()) / 4.0;
        assert_eq!(
            census.structure_means(),
            StructureMeans {
                genome_size: expected_genome_size,
                mesh_nodes: 2.0,
                generation: 0.0,
                wired_sinks: 17.0,
            }
        );
    }

    /// Mean energy sums in `f64`: energies whose `f32` sum would swallow
    /// the small terms keep them.
    #[test]
    fn mean_energy_sums_the_f32_energies_in_f64() {
        let mut sim = sim(3);
        for (creature, energy) in sim.creatures.values_mut().zip([16_777_216.0_f32, 1.0, 1.0]) {
            creature.energy = energy;
        }
        let census = PopulationCensus::of(&sim);
        assert_eq!(census.mean_energy(), Some(16_777_218.0 / 3.0));
    }
}
