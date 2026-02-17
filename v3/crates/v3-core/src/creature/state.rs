use crate::kernel::types::Position;

/// Energy value with safe operations (non-negative, integer).
/// Uses u32 to avoid floating-point flakiness in tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Energy(u32);

impl Energy {
    pub fn new(value: u32) -> Self {
        Energy(value)
    }

    pub fn drain(&mut self, amount: u32) -> bool {
        if self.0 >= amount {
            self.0 -= amount;
            true
        } else {
            false
        }
    }

    /// Always deducts energy, clamping to 0. Used for action costs that
    /// are paid regardless of whether the action succeeds.
    pub fn drain_saturating(&mut self, amount: u32) {
        self.0 = self.0.saturating_sub(amount);
    }

    pub fn charge(&mut self, amount: u32, max: u32) {
        self.0 = (self.0 + amount).min(max);
    }

    pub fn is_alive(&self) -> bool {
        self.0 > 0
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Minimal creature state for Stage 1.
/// Note: no `id` field — the SlotMap key IS the creature's identity.
#[derive(Clone, Debug)]
pub struct CreatureState {
    pub position: Position,
    pub energy: Energy,
    pub age: u64,
    pub generation: u32,
    pub phenotype_r: u8,
    pub phenotype_g: u8,
    pub phenotype_b: u8,
}

impl CreatureState {
    pub fn new(
        position: Position,
        initial_energy: u32,
        generation: u32,
        phenotype_rgb: [u8; 3],
    ) -> Self {
        Self {
            position,
            energy: Energy::new(initial_energy),
            age: 0,
            generation,
            phenotype_r: phenotype_rgb[0],
            phenotype_g: phenotype_rgb[1],
            phenotype_b: phenotype_rgb[2],
        }
    }
}
