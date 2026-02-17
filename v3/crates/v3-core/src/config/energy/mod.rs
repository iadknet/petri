pub mod costs;
pub mod lifecycle;

pub use costs::EnergyCosts;
pub use lifecycle::EnergyLifecycle;

/// Energy-related configuration.
#[derive(Clone, Debug, Default)]
pub struct EnergyConfig {
    pub lifecycle: EnergyLifecycle,
    pub costs: EnergyCosts,
}
