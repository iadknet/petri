use crate::kernel::types::Position;

/// Sensory data for a single neighbor cell.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeighborSense {
    pub food_density: u8,
    pub passable: bool,
}

/// Environmental perception inputs.
/// Stage 2: food_density_self + 8 neighbor senses.
/// Will be substantially reworked in Stage 3 for full SensorFrame.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentalInputs {
    pub food_density_self: u8,
    /// Neighbor senses indexed by `Direction::ALL` order (N, NE, E, SE, S, SW, W, NW).
    pub neighbors: [NeighborSense; 8],
}

/// Introspection inputs (stub for Stage 1).
#[derive(Clone, Debug, Default)]
pub struct IntrospectionInputs {
    pub energy: u32,
    pub position: Position,
}

/// Combined inputs for creature execution.
#[derive(Clone, Debug, Default)]
pub struct CreatureInputs {
    pub environmental: EnvironmentalInputs,
    pub introspection: IntrospectionInputs,
}
