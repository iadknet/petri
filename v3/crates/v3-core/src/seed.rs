use crate::creature::state::CreatureState;
use crate::kernel::types::Position;
use crate::kernel::world_state::WorldState;
use crate::SimulationState;
use rand::Rng;

/// Seed creatures into the simulation at random empty positions.
/// Best-effort: if the world is too full, fewer creatures are placed.
pub fn seed_creatures(
    state: &mut SimulationState,
    count: usize,
    initial_energy: u32,
    rng: &mut impl Rng,
) {
    for _ in 0..count {
        if let Some(pos) = find_empty_position(&state.world, rng) {
            let creature = CreatureState::new(pos, initial_energy, 0, random_phenotype(rng));
            state.spawn_creature(creature);
        }
    }
}

fn find_empty_position(world: &WorldState, rng: &mut impl Rng) -> Option<Position> {
    let max_attempts = 100;
    for _ in 0..max_attempts {
        let pos = Position {
            x: rng.gen_range(0..world.width),
            y: rng.gen_range(0..world.height),
        };
        if !world.is_occupied(pos) && !world.is_barrier(pos) {
            return Some(pos);
        }
    }
    None
}

fn random_phenotype(rng: &mut impl Rng) -> [u8; 3] {
    [rng.gen(), rng.gen(), rng.gen()]
}
