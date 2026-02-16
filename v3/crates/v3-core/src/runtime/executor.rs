use crate::contracts::inputs::CreatureInputs;
use crate::contracts::outputs::CreatureOutputs;

/// Stub executor for Stage 1 (always returns NoOp).
pub fn execute_creature_stub(_inputs: &CreatureInputs) -> CreatureOutputs {
    CreatureOutputs::noop()
}
