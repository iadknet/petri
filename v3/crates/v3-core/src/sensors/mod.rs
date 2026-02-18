use crate::contracts::inputs::{
    CreatureInputs, EnvironmentalInputs, IntrospectionInputs, NeighborSense,
};
use crate::creature::state::CreatureState;
use crate::kernel::types::Direction;
use crate::kernel::world_state::WorldState;

/// Gather all inputs for a creature's decision-making.
/// Stage 3C: food_density_self + 8 neighbor senses (occupied + barrier separately).
pub fn gather_inputs(creature: &CreatureState, world: &WorldState) -> CreatureInputs {
    let pos = creature.position;

    let mut neighbors = [NeighborSense::default(); 8];
    for (i, &dir) in Direction::ALL.iter().enumerate() {
        neighbors[i] = match world.resolve_neighbor(pos, dir) {
            Some(npos) => NeighborSense {
                food_density: world.get_food_density(npos),
                occupied: world.is_occupied(npos),
                barrier: world.is_barrier(npos),
            },
            None => NeighborSense {
                food_density: 0,
                occupied: false,
                barrier: true, // out-of-bounds treated as barrier
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
            generation: creature.generation,
            age_ticks: creature.age,
        },
    }
}
