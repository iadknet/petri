/// Mutation parameters applied during offspring genome creation.
#[derive(Clone, Debug)]
pub struct MutationConfig {
    /// Probability that any mutation occurs during reproduction (0.0–1.0).
    pub mutation_probability: f64,
    /// Minimum number of mutation events per birth (when mutation triggers).
    pub per_birth_mutation_events_min: u32,
    /// Maximum number of mutation events per birth (when mutation triggers).
    pub per_birth_mutation_events_max: u32,
    /// Maximum magnitude of constant jitter per mutation event.
    pub constant_jitter_magnitude: f32,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_probability: 0.01,
            per_birth_mutation_events_min: 1,
            per_birth_mutation_events_max: 4,
            constant_jitter_magnitude: 0.1,
        }
    }
}
