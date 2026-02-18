use crate::kernel::types::Position;

/// Sensory data for a single neighbor cell.
///
/// `occupied` and `barrier` are stored separately so the VM's
/// `ReadNeighborCell` opcodes can read each flag independently.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeighborSense {
    pub food_density: u8,
    /// True if a creature occupies this cell.
    pub occupied: bool,
    /// True if this cell is a barrier (wall).
    pub barrier: bool,
}

impl NeighborSense {
    /// Convenience: a cell is passable if it is neither occupied nor a barrier.
    #[inline]
    pub fn passable(&self) -> bool {
        !self.occupied && !self.barrier
    }
}

/// Environmental perception inputs.
/// Stage 3C: food_density_self + 8 neighbor senses with separate occupied/barrier.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentalInputs {
    pub food_density_self: u8,
    /// Neighbor senses indexed by `Direction::ALL` order (N, NE, E, SE, S, SW, W, NW).
    pub neighbors: [NeighborSense; 8],
}

/// Introspection inputs.
#[derive(Clone, Debug, Default)]
pub struct IntrospectionInputs {
    pub energy: u32,
    pub position: Position,
    pub generation: u32,
    pub age_ticks: u64,
}

/// Combined inputs for creature execution.
#[derive(Clone, Debug, Default)]
pub struct CreatureInputs {
    pub environmental: EnvironmentalInputs,
    pub introspection: IntrospectionInputs,
}
