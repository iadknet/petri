use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::graph_mutator::{GraphMutator, GraphOperator};
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::{MutationSkipReason, MutationSummary};
use crate::mutation::vm_mutator::{VmMutator, VmOperator};

/// Orchestrates genome mutation events for offspring.
pub struct MutationEngine;

impl MutationEngine {
    /// Apply mutation events to a child genome and return a summary.
    ///
    /// Accounting invariant: `summary.attempted_events == summary.applied_events + summary.skipped_events`.
    pub fn apply_mutations(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        rng: &mut impl Rng,
    ) -> MutationSummary {
        // Probability gate.
        if !rng.gen_bool(config.mutation_probability) {
            return MutationSummary::zero();
        }

        // Determine number of mutation events this birth.
        let event_count = rng
            .gen_range(config.per_birth_mutation_events_min..=config.per_birth_mutation_events_max);

        let mut summary = MutationSummary::zero();
        for _ in 0..event_count {
            summary.attempted_events += 1;

            // Pick domain uniformly: 0=Topology, 1=VM, 2=Graph.
            let domain = rng.gen_range(0u8..3);
            let result = match domain {
                0 => apply_topology_event(genome, rng),
                1 => apply_vm_event(genome, rng),
                _ => apply_graph_event(genome, rng),
            };

            match result {
                Ok(()) => summary.applied_events += 1,
                Err(reason) => {
                    summary.skipped_events += 1;
                    *summary.skip_reasons.entry(reason).or_insert(0) += 1;
                }
            }
        }

        summary
    }
}

/// Apply one topology mutation event with parseability gate.
fn apply_topology_event(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let op = TopologyOperator::random(rng);
    let snapshot = genome.clone();
    match TopologyMutator::apply(genome, op, rng) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

/// Apply one VM mutation event with parseability gate.
fn apply_vm_event(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let op = VmOperator::random(rng);
    let snapshot = genome.clone();
    match VmMutator::apply(genome, op, rng) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

/// Apply one graph mutation event with parseability gate.
fn apply_graph_event(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let op = GraphOperator::random(rng);
    let snapshot = genome.clone();
    match GraphMutator::apply(genome, op, rng) {
        Ok(()) => {
            if ParseabilityGate::validate(genome).is_ok() {
                Ok(())
            } else {
                *genome = snapshot;
                Err(MutationSkipReason::ParseabilityViolation)
            }
        }
        Err(reason) => {
            *genome = snapshot;
            Err(reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::creature::founder::v3alpha1_founder_genome;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn engine_accounting_invariant_always_holds() {
        let mut config = SimulationConfig::default().mutation;
        config.mutation_probability = 1.0;
        config.per_birth_mutation_events_min = 1;
        config.per_birth_mutation_events_max = 5;

        for seed in 0u64..100 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
            assert_eq!(
                summary.attempted_events,
                summary.applied_events + summary.skipped_events,
                "accounting invariant violated at seed {}",
                seed
            );
        }
    }

    #[test]
    fn engine_with_probability_zero_returns_zero_summary() {
        let mut config = SimulationConfig::default().mutation;
        config.mutation_probability = 0.0;
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(42);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
        assert_eq!(summary.attempted_events, 0);
        assert_eq!(summary.applied_events, 0);
        assert_eq!(summary.skipped_events, 0);
    }

    #[test]
    fn engine_with_probability_one_applies_events() {
        let mut config = SimulationConfig::default().mutation;
        config.mutation_probability = 1.0;
        config.per_birth_mutation_events_min = 3;
        config.per_birth_mutation_events_max = 3;
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(7);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
        assert_eq!(summary.attempted_events, 3, "must attempt exactly 3 events");
    }

    #[test]
    fn engine_mutations_preserve_parseability() {
        let mut config = SimulationConfig::default().mutation;
        config.mutation_probability = 1.0;
        config.per_birth_mutation_events_min = 1;
        config.per_birth_mutation_events_max = 4;

        for seed in 0u64..50 {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(seed);
            MutationEngine::apply_mutations(&mut genome, &config, &mut r);
            assert!(
                ParseabilityGate::validate(&genome).is_ok(),
                "parseability violated at seed {}",
                seed
            );
        }
    }

    #[test]
    fn engine_with_founder_genome_does_not_panic() {
        let mut config = SimulationConfig::default().mutation;
        config.mutation_probability = 1.0;
        config.per_birth_mutation_events_min = 1;
        config.per_birth_mutation_events_max = 4;

        let mut genome = v3alpha1_founder_genome();
        for seed in 0u64..1000 {
            let mut r = rng(seed);
            let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut r);
            assert_eq!(
                summary.attempted_events,
                summary.applied_events + summary.skipped_events
            );
        }
    }
}
