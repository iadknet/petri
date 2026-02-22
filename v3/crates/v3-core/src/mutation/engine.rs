use crate::config::MutationConfig;
use crate::creature::genome::CreatureGenome;
use crate::mutation::types::MutationSummary;

/// Orchestrates genome mutation events for offspring.
///
/// Stage 4 stub: always returns a zero-event summary.
/// The `mutation_probability` gate always fails in this stub because no dice are rolled.
/// Stage 5 will replace this with real mutation operators.
pub struct MutationEngine;

impl MutationEngine {
    /// Apply mutation events to a child genome and return a summary.
    ///
    /// Stage 4 stub: no mutations are applied; returns `MutationSummary::zero()`.
    pub fn apply_mutations(
        genome: &mut CreatureGenome,
        config: &MutationConfig,
        rng: &mut impl rand::Rng,
    ) -> MutationSummary {
        // Stage 4 stub: no mutations applied.
        let _ = (genome, config, rng);
        MutationSummary::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::creature::founder::v3alpha1_founder_genome;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn stub_engine_returns_zero_summary() {
        let mut genome = v3alpha1_founder_genome();
        let config = SimulationConfig::default().mutation;
        let mut rng = SmallRng::seed_from_u64(42);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut rng);
        assert_eq!(summary.attempted_events, 0);
        assert_eq!(summary.applied_events, 0);
        assert_eq!(summary.skipped_events, 0);
    }

    #[test]
    fn stub_engine_accounting_invariant_holds() {
        let mut genome = v3alpha1_founder_genome();
        let config = SimulationConfig::default().mutation;
        let mut rng = SmallRng::seed_from_u64(99);
        let summary = MutationEngine::apply_mutations(&mut genome, &config, &mut rng);
        assert_eq!(
            summary.attempted_events,
            summary.applied_events + summary.skipped_events
        );
    }
}
