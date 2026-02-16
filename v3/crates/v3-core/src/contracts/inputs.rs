use crate::kernel::types::Position;

/// Environmental perception inputs (stub for Stage 1).
#[derive(Clone, Debug, Default)]
pub struct EnvironmentalInputs {
    pub food_density_self: u8,
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
