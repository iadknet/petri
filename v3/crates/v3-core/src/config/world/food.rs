/// Food growth and seeding configuration.
#[derive(Clone, Debug)]
pub struct FoodConfig {
    /// Probability per non-barrier cell per tick of gaining +1 food.
    pub growth_rate: f32,
    /// Food density assigned to cells during initial world seeding.
    pub initial_density: u8,
    /// Fraction of cells seeded with food during world initialization.
    pub initial_coverage: f32,
}

impl Default for FoodConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.02,
            initial_density: 80,
            initial_coverage: 0.3,
        }
    }
}
