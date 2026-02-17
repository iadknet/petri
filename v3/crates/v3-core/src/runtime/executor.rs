use crate::config::SimulationConfig;
use crate::contracts::inputs::CreatureInputs;
use crate::contracts::outputs::{CreatureOutputs, WorldAction};
use crate::kernel::types::Direction;
use rand::seq::SliceRandom;
use rand::Rng;

/// Heuristic brain for Stage 2.
/// Logic: (1) eat if food at current position, (2) reproduce when energy is
/// high and a passable neighbor exists, (3) move toward highest-food passable
/// neighbor (random tiebreak), (4) move in random passable direction,
/// (5) NoOp if completely trapped.
pub fn execute_heuristic(
    inputs: &CreatureInputs,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> CreatureOutputs {
    let env = &inputs.environmental;

    // If there's food here, eat it
    if env.food_density_self > 0 {
        return CreatureOutputs {
            world_action: WorldAction::Eat,
            ..CreatureOutputs::noop()
        };
    }

    // If energy is high enough, attempt reproduction into a random passable direction.
    let passable_dirs: Vec<Direction> = Direction::ALL
        .iter()
        .enumerate()
        .filter(|(i, _)| env.neighbors[*i].passable)
        .map(|(_, &dir)| dir)
        .collect();
    if inputs.introspection.energy >= config.energy.lifecycle.min_reproduce_energy {
        if let Some(&dir) = passable_dirs.choose(rng) {
            return CreatureOutputs {
                world_action: WorldAction::Reproduce {
                    direction: dir,
                    energy_amount: config.energy.lifecycle.default_offspring_energy,
                },
                ..CreatureOutputs::noop()
            };
        }
    }

    // Find passable neighbors with food, pick the one with highest food (random tiebreak)
    let mut best_food: u8 = 0;
    let mut best_dirs: Vec<Direction> = Vec::new();

    for (i, &dir) in Direction::ALL.iter().enumerate() {
        let n = &env.neighbors[i];
        if n.passable && n.food_density > 0 {
            if n.food_density > best_food {
                best_food = n.food_density;
                best_dirs.clear();
                best_dirs.push(dir);
            } else if n.food_density == best_food {
                best_dirs.push(dir);
            }
        }
    }

    if let Some(&dir) = best_dirs.choose(rng) {
        return CreatureOutputs {
            world_action: WorldAction::Move { direction: dir },
            ..CreatureOutputs::noop()
        };
    }

    // No food visible — move in a random passable direction
    if let Some(&dir) = passable_dirs.choose(rng) {
        return CreatureOutputs {
            world_action: WorldAction::Move { direction: dir },
            ..CreatureOutputs::noop()
        };
    }

    // Completely trapped
    CreatureOutputs::noop()
}
