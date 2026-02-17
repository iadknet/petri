use crate::kernel::types::Direction;

/// World actions a creature can attempt.
#[derive(Clone, Debug, PartialEq)]
pub enum WorldAction {
    NoOp,
    Eat,
    Move {
        direction: Direction,
    },
    Reproduce {
        direction: Direction,
        energy_amount: u32,
    },
}

/// Internal outputs (stub for Stage 1).
#[derive(Clone, Debug, Default)]
pub struct InternalOutputs {
    // Memory writes will be added later
}

/// Complete creature execution output.
#[derive(Clone, Debug)]
pub struct CreatureOutputs {
    pub world_action: WorldAction,
    pub internal: InternalOutputs,
    pub energy_consumed: u32,
}

impl CreatureOutputs {
    pub fn noop() -> Self {
        Self {
            world_action: WorldAction::NoOp,
            internal: InternalOutputs::default(),
            energy_consumed: 0,
        }
    }
}
