pub mod energy;
pub mod runtime;
pub mod world;

pub use energy::EnergyConfig;
pub use runtime::RuntimeConfig;
pub use world::WorldConfig;

/// Top-level simulation configuration.
/// Pure data — no dependencies on simulation state.
#[derive(Clone, Debug, Default)]
pub struct SimulationConfig {
    pub world: WorldConfig,
    pub energy: EnergyConfig,
    pub runtime: RuntimeConfig,
}
