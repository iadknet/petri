use crate::config::runtime::mutation::MutationConfig;
use crate::creature::genome::CreatureGenome;
use rand::Rng;

/// Apply mutation to a genome according to config.
///
/// Rolls against `mutation_probability` to determine if any mutation occurs.
/// If triggered, applies `per_birth_mutation_events_min..=max` jitter events
/// to random constants in the genome.
pub fn mutate_genome(genome: &mut CreatureGenome, config: &MutationConfig, rng: &mut impl Rng) {
    if genome.constants.is_empty() {
        return;
    }

    if rng.gen::<f64>() >= config.mutation_probability {
        return;
    }

    let event_count =
        rng.gen_range(config.per_birth_mutation_events_min..=config.per_birth_mutation_events_max);

    for _ in 0..event_count {
        let idx = rng.gen_range(0..genome.constants.len());
        let jitter =
            rng.gen_range(-config.constant_jitter_magnitude..=config.constant_jitter_magnitude);
        genome.constants[idx] += jitter;
        // Clamp to avoid NaN/Inf drift
        genome.constants[idx] = genome.constants[idx].clamp(-1e6, 1e6);
    }
}
