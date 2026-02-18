use crate::config::SimulationConfig;
use crate::creature::founders;
use crate::creature::state::CreatureState;
use crate::kernel::types::Position;
use crate::kernel::world_state::WorldState;
use crate::SimulationState;
use rand::Rng;

/// Fixed phenotype for seed creatures (warm red).
const SEED_PHENOTYPE: [u8; 3] = [204, 61, 61];

/// Seed creatures into the simulation at random empty positions,
/// and seed initial food on the world.
/// Best-effort: if the world is too full, fewer creatures are placed.
pub fn seed_creatures(
    state: &mut SimulationState,
    count: usize,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) {
    // Seed food first
    state.world.seed_food(&config.world.food, rng);

    let initial_energy = config.energy.lifecycle.initial_energy;
    let founder_genome = founders::get("simple");
    for _ in 0..count {
        if let Some(pos) = find_empty_position(&state.world, rng) {
            let creature = CreatureState::new(
                pos,
                initial_energy,
                0,
                SEED_PHENOTYPE,
                founder_genome.clone(),
            );
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
