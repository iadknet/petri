mod eval;
mod types;

pub use eval::{ComputationGraph, MutationConfig};
pub use types::{
    ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs, SLOT_COUNT_MAX,
    TOUCH_DIRECTION_COUNT,
};
