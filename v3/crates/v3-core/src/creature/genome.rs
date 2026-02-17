/// Minimal creature genome for Stage 3B.
///
/// Contains a numeric constants pool that is heritable and mutable.
/// Future stages will expand this into per-node constant pools within
/// a multi-node mesh architecture (see v3-genome-sensor-spec.md).
#[derive(Clone, Debug, PartialEq)]
pub struct CreatureGenome {
    /// Numeric constants pool. Copied from parent to offspring,
    /// then mutated according to MutationConfig.
    pub constants: Vec<f32>,
}

impl CreatureGenome {
    /// Deterministic founder genome used for seed creatures and tests.
    /// All seed creatures share this identical genome.
    pub fn simple_founder() -> Self {
        Self {
            constants: vec![0.5, 0.3, 0.7, 0.1],
        }
    }
}
