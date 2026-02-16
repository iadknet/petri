use crate::contracts::inputs::{CreatureInputs, EnvironmentalInputs, IntrospectionInputs};
use crate::creature::state::CreatureState;
use crate::kernel::world_state::WorldState;

/// Stub sensors for Stage 1 (returns minimal inputs).
pub fn gather_inputs_stub(creature: &CreatureState, _world: &WorldState) -> CreatureInputs {
    CreatureInputs {
        environmental: EnvironmentalInputs::default(),
        introspection: IntrospectionInputs {
            energy: creature.energy.value(),
            position: creature.position,
        },
    }
}
