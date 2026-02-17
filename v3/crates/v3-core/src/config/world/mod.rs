pub mod food;

pub use food::FoodConfig;

/// World-level configuration.
#[derive(Clone, Debug, Default)]
pub struct WorldConfig {
    pub food: FoodConfig,
}
