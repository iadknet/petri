/// Energy lifecycle parameters for creatures.
#[derive(Clone, Debug)]
pub struct EnergyLifecycle {
    /// Starting energy for newly spawned/seeded creatures.
    pub initial_energy: u32,
    /// Maximum energy a creature can hold.
    pub max_energy: u32,
    /// Energy lost per tick from passive decay.
    pub energy_decay_per_tick: u32,
    /// Minimum energy a parent must have after paying reproduce cost.
    pub min_reproduce_energy: u32,
    /// Default energy transferred from parent to offspring.
    pub default_offspring_energy: u32,
}

impl Default for EnergyLifecycle {
    fn default() -> Self {
        Self {
            initial_energy: 20,
            max_energy: 100,
            energy_decay_per_tick: 1,
            min_reproduce_energy: 24,
            default_offspring_energy: 12,
        }
    }
}
