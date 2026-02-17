use crate::contracts::inputs::{
    CreatureInputs, EnvironmentalInputs, IntrospectionInputs, NeighborSense,
};
use crate::creature::state::CreatureState;
use crate::kernel::types::Direction;
use crate::kernel::world_state::WorldState;

/// Gather all inputs for a creature's decision-making.
/// Stage 2: food_density_self + 8 neighbor senses.
pub fn gather_inputs(creature: &CreatureState, world: &WorldState) -> CreatureInputs {
    let pos = creature.position;

    let mut neighbors = [NeighborSense::default(); 8];
    for (i, &dir) in Direction::ALL.iter().enumerate() {
        neighbors[i] = match world.resolve_neighbor(pos, dir) {
            Some(npos) => NeighborSense {
                food_density: world.get_food_density(npos),
                passable: !world.is_barrier(npos) && !world.is_occupied(npos),
            },
            None => NeighborSense {
                food_density: 0,
                passable: false,
            },
        };
    }

    CreatureInputs {
        environmental: EnvironmentalInputs {
            food_density_self: world.get_food_density(pos),
            neighbors,
        },
        introspection: IntrospectionInputs {
            energy: creature.energy.value(),
            position: creature.position,
        },
    }
}
