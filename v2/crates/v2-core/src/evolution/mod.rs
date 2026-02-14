pub mod config;
pub mod operators;
pub mod validation;

use crate::mesh::CreatureGenome;

pub use config::{MutationConfig, MutationConfigError, MutationWeights};
pub use operators::{MutationOperatorKind, apply_birth_mutations, apply_operator};
pub use validation::{
    MutationInvariantError, repair_genome, repair_or_discard, validate_mutation_invariants,
};

pub const MEMORY_BYTES: usize = 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Offspring {
    pub genome: CreatureGenome,
    pub memory_bytes: [u8; MEMORY_BYTES],
}

#[must_use]
pub fn inherit_memory_bytes(parent_memory: &[u8; MEMORY_BYTES]) -> [u8; MEMORY_BYTES] {
    *parent_memory
}

#[must_use]
pub fn reproduce_asexual(
    parent_genome: &CreatureGenome,
    parent_memory: &[u8; MEMORY_BYTES],
    config: &MutationConfig,
    seed: u64,
) -> Option<Offspring> {
    let candidate = apply_birth_mutations(parent_genome, config, seed);
    let genome = repair_or_discard(candidate, config, 3)?;

    Some(Offspring {
        genome,
        memory_bytes: inherit_memory_bytes(parent_memory),
    })
}
