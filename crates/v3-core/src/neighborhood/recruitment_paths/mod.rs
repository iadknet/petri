//! Bounded, replicated task observation. Never used by mutation or simulation.

mod experiment;
mod fixtures;
mod qualification;
mod records;

pub use experiment::observe;
pub use fixtures::{constructed_paths, starting_forms, ConstructedPath, ConstructionStage, Start};
pub use qualification::{
    qualified_paths, search_seeds, CostVerdict, GrowthGap, PathStep, ProductionEvent,
    QualifiedPath, SeedSearch, MAX_PATH_EVENTS, SEARCH_RANGE,
};
pub use records::*;

use crate::config::SimulationConfig;
use crate::contracts::{Direction, Position, WorldAction};
use crate::creature::genome::CreatureGenome;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::runtime::trace::recording::ActiveTrace;
use crate::simulation::{run_tick, Simulation};
use slotmap::SlotMap;

pub const VERSION: &str = "recruitment-paths-v1";

/// Only fixture world initialization differs from production defaults.
#[must_use]
pub fn task_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 12;
    config.world.height = 12;
    config.population.initial_creatures = 0;
    config.world.food.initial_coverage = 0.0;
    config.world.food.growth_rate = 0.0;
    config.world.food.recovery_spawn_rate = 0.0;
    for food in &mut config.world.food.types {
        food.initial_coverage = 0.0;
    }
    config
}

/// Evaluate all eight scenes through one actual tick each, with fresh state.
#[must_use]
pub fn evaluate(genome: &CreatureGenome) -> TaskReading {
    evaluate_with_config(genome, &task_config())
}

fn evaluate_with_config(genome: &CreatureGenome, config: &SimulationConfig) -> TaskReading {
    let centre = Position::new(6, 6);
    let mut scenes = Vec::with_capacity(8);
    for index in 0..8u8 {
        let food = [index & 4 != 0, index & 2 != 0, index & 1 != 0];
        let mut world = WorldState::new(12, 12, config.world.edge_mode);
        world.reconfigure_food(config.world.food.clone());
        for (position, present) in [centre, Position::new(7, 6), Position::new(6, 5)]
            .into_iter()
            .zip(food)
        {
            world.set_food(position, f32::from(present));
        }
        let mut creatures = SlotMap::with_key();
        let id = creatures.insert_with_key(|id| {
            CreatureState::new(
                id,
                genome.clone(),
                centre,
                50.0,
                0,
                [0, 0, 92, 92, 138, 138],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            )
        });
        world.place_creature(centre, id);
        let mut sim = Simulation::new(world, creatures, 0, config.clone(), 41);
        let mut trace = Some(ActiveTrace::new(id, 1));
        run_tick(&mut sim, &mut trace);
        let tick = trace.as_ref().and_then(|trace| trace.ticks.first());
        let creature = sim.creatures.get(id);
        let actions = tick.map_or_else(Vec::new, |tick| tick.final_actions.clone());
        let position = creature.map(|creature| creature.position);
        let correct = |cue| {
            let expected = if cue {
                WorldAction::Move(Direction::E)
            } else {
                WorldAction::NoOp
            };
            actions == [expected]
                && position == Some(if cue { Position::new(7, 6) } else { centre })
        };
        scenes.push(Scene {
            food,
            correct_a: correct(food[0]),
            correct_b: correct(food[1]),
            survived: creature.is_some_and(|creature| creature.energy > 0.0),
            energy: creature.map(|creature| creature.energy),
            maintenance: sim.stats.energy_flows.lifecycle_decay
                + sim.stats.energy_flows.genome_carrying,
            carrying: sim.stats.energy_flows.genome_carrying,
            work: Work {
                mesh_hops: sim.stats.mesh_hops_total,
                vm_steps: sim.stats.vm_steps_total,
                graph_visits: sim.stats.graph_relax_iters_total,
                plasticity: sim.stats.plasticity_updates_total,
            },
            dispatched: tick.map_or_else(Vec::new, |tick| {
                tick.hops.iter().map(|hop| hop.node_id).collect()
            }),
            routing: tick.map_or_else(Vec::new, |tick| {
                tick.hops
                    .iter()
                    .filter_map(|hop| {
                        hop.route
                            .as_ref()
                            .map(|route| (hop.node_id, route.selected_target_id))
                    })
                    .collect()
            }),
            output_slots: tick.map_or_else(Vec::new, |tick| {
                tick.hops
                    .iter()
                    .map(|hop| (hop.node_id, hop.output_slots))
                    .collect()
            }),
            shared_memory: creature.map(|creature| creature.shared_memory),
            actions,
            position,
        });
    }
    TaskReading { scenes }
}

#[cfg(test)]
mod tests;
