//! Censuses of a run's final living population: how many creatures carry
//! each structural memory companion, and how many act differently when
//! their memory is zeroed or scrambled (the memory-sensitivity probes of
//! `simulation::tick`).

use crate::genome::{structural_companions, CreatureGenome};
use crate::simulation::{
    FinalActionObservation, TemporalActionObservation, TemporalMemorySubstrate,
};

/// Creatures carrying each structural companion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompanionCounts {
    pub creatures: u64,
    pub reads_shared_memory: u64,
    pub writes_shared_memory: u64,
    pub has_stateful_compute_node: u64,
    pub has_plasticity: u64,
}

impl CompanionCounts {
    pub fn of<'a>(genomes: impl IntoIterator<Item = &'a CreatureGenome>) -> Self {
        let mut counts = Self::default();
        for genome in genomes {
            let companions = structural_companions(genome);
            counts.creatures += 1;
            counts.reads_shared_memory += u64::from(companions.reads_shared_memory);
            counts.writes_shared_memory += u64::from(companions.writes_shared_memory);
            counts.has_stateful_compute_node += u64::from(companions.has_stateful_compute_node);
            counts.has_plasticity += u64::from(companions.has_plasticity);
        }
        counts
    }
}

/// Creatures whose actions differ from the zeroed, the scrambled, or either
/// memory intervention.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DifferingCounts {
    pub creatures: u64,
    pub from_zeroed: u64,
    pub from_scrambled: u64,
    pub from_either: u64,
}

impl DifferingCounts {
    pub fn of<'a>(observations: impl IntoIterator<Item = &'a FinalActionObservation>) -> Self {
        let mut counts = Self::default();
        for observation in observations {
            let zeroed = observation.intact != observation.zeroed;
            let scrambled = observation.intact != observation.scrambled;
            counts.creatures += 1;
            counts.from_zeroed += u64::from(zeroed);
            counts.from_scrambled += u64::from(scrambled);
            counts.from_either += u64::from(zeroed || scrambled);
        }
        counts
    }

    /// The counts over the observations of one temporal memory `substrate`.
    #[must_use]
    pub fn of_substrate(
        observations: &[TemporalActionObservation],
        substrate: TemporalMemorySubstrate,
    ) -> Self {
        Self::of(
            observations
                .iter()
                .filter(|observation| observation.substrate == substrate)
                .map(|observation| &observation.actions),
        )
    }
}
