mod eval;
mod types;

pub use eval::{ComputationGraph, MutationConfig};
pub use types::{
    ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs, ACTION_CONFIDENCE_COUNT,
    SLOT_COUNT_MAX, TOUCH_DIRECTION_COUNT,
};
