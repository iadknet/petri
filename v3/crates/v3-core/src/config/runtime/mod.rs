pub mod mutation;

pub use mutation::MutationConfig;

/// Runtime configuration (VM execution, mutation).
#[derive(Clone, Debug, Default)]
pub struct RuntimeConfig {
    pub mutation: MutationConfig,
}
