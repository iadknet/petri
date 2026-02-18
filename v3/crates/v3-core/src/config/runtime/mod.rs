pub mod mutation;
pub mod vm;

pub use mutation::MutationConfig;
pub use vm::VmConfig;

/// Runtime configuration (VM execution, mutation).
#[derive(Clone, Debug, Default)]
pub struct RuntimeConfig {
    pub mutation: MutationConfig,
    pub vm: VmConfig,
}
