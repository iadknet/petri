use std::time::Instant;

use rand::Rng;
use tracing::{info, warn};

use petri_core::World;

use super::startup_draft::build_world_config;
use super::status::simulation_status_from_locked;
use super::types::SimulationRun;
use super::viability::{ensure_viable_start, startup_probe_seed};
use super::{AppState, SimulationError, SimulationPhase, SimulationStatus};

impl AppState {
    pub async fn start_simulation(&self) -> Result<SimulationStatus, SimulationError> {
        let (seed, run_id, config, initial_food_density, probe_seed, viability_probe_enabled) = {
            let mut sim = self.simulation.write().await;
            if sim.run.is_some() || sim.phase == SimulationPhase::Starting {
                return Err(SimulationError::AlreadyRunning);
            }

            let seed = sim.rng.gen::<u64>();
            let run_id = sim.next_run_id;
            sim.next_run_id += 1;
            let config = build_world_config(&sim.runtime_config, &sim.startup_draft);
            let initial_food_density = sim.startup_draft.initial_food_density;
            let probe_seed = startup_probe_seed(&sim.startup_draft);
            sim.phase = SimulationPhase::Starting;
            sim.initialization_stage = Some(if sim.viability_probe_enabled {
                "viability_probe"
            } else {
                "world_build"
            });
            (
                seed,
                run_id,
                config,
                initial_food_density,
                probe_seed,
                sim.viability_probe_enabled,
            )
        };

        let startup_begin = Instant::now();
        info!(
            run_id,
            seed,
            width = config.width,
            height = config.height,
            initial_creatures = config.initial_creatures,
            max_creatures = config.max_creatures,
            "simulation startup initiated"
        );

        if viability_probe_enabled {
            let probe_begin = Instant::now();
            let survivors = match ensure_viable_start(&config, initial_food_density, probe_seed) {
                Ok(survivors) => survivors,
                Err(err) => {
                    let mut sim = self.simulation.write().await;
                    sim.phase = SimulationPhase::Idle;
                    sim.initialization_stage = None;
                    warn!(run_id, "simulation startup failed viability probe");
                    return Err(err);
                }
            };
            info!(
                run_id,
                survivors,
                probe_ms = probe_begin.elapsed().as_millis(),
                "simulation startup viability probe completed"
            );
        } else {
            info!(run_id, "simulation startup viability probe disabled");
        }

        {
            let mut sim = self.simulation.write().await;
            sim.initialization_stage = Some("world_build");
        }

        let world_build_begin = Instant::now();
        let mut world = World::new(config.clone(), seed);
        world.seed_food_density(initial_food_density);
        info!(
            run_id,
            world_build_ms = world_build_begin.elapsed().as_millis(),
            "simulation startup world build completed"
        );

        let mut sim = self.simulation.write().await;
        sim.run = Some(SimulationRun {
            run_id,
            seed,
            world,
        });
        sim.phase = SimulationPhase::Running;
        sim.initialization_stage = None;
        sim.pending_restart = false;
        sim.runtime_config = config;
        info!(
            run_id,
            total_startup_ms = startup_begin.elapsed().as_millis(),
            "simulation startup completed"
        );

        Ok(simulation_status_from_locked(&sim))
    }

    pub async fn restart_simulation(&self) -> Result<SimulationStatus, SimulationError> {
        let (
            seed,
            run_id,
            config,
            initial_food_density,
            probe_seed,
            fallback_phase,
            viability_probe_enabled,
        ) = {
            let mut sim = self.simulation.write().await;
            if sim.run.is_none() {
                return Err(SimulationError::NoActiveRun);
            }
            if sim.phase == SimulationPhase::Starting {
                return Err(SimulationError::AlreadyRunning);
            }

            let seed = sim.rng.gen::<u64>();
            let run_id = sim.next_run_id;
            sim.next_run_id += 1;
            let config = build_world_config(&sim.runtime_config, &sim.startup_draft);
            let initial_food_density = sim.startup_draft.initial_food_density;
            let probe_seed = startup_probe_seed(&sim.startup_draft);
            let fallback_phase = if sim.runtime_config.paused {
                SimulationPhase::Paused
            } else {
                SimulationPhase::Running
            };
            sim.phase = SimulationPhase::Starting;
            sim.initialization_stage = Some(if sim.viability_probe_enabled {
                "viability_probe"
            } else {
                "world_build"
            });
            (
                seed,
                run_id,
                config,
                initial_food_density,
                probe_seed,
                fallback_phase,
                sim.viability_probe_enabled,
            )
        };

        let restart_begin = Instant::now();
        info!(run_id, seed, "simulation restart initiated");

        if viability_probe_enabled {
            let probe_begin = Instant::now();
            let survivors = match ensure_viable_start(&config, initial_food_density, probe_seed) {
                Ok(survivors) => survivors,
                Err(err) => {
                    let mut sim = self.simulation.write().await;
                    sim.phase = fallback_phase;
                    sim.initialization_stage = None;
                    warn!(run_id, "simulation restart failed viability probe");
                    return Err(err);
                }
            };
            info!(
                run_id,
                survivors,
                probe_ms = probe_begin.elapsed().as_millis(),
                "simulation restart viability probe completed"
            );
        } else {
            info!(run_id, "simulation restart viability probe disabled");
        }

        {
            let mut sim = self.simulation.write().await;
            sim.initialization_stage = Some("world_build");
        }

        let world_build_begin = Instant::now();
        let mut world = World::new(config.clone(), seed);
        world.seed_food_density(initial_food_density);
        info!(
            run_id,
            world_build_ms = world_build_begin.elapsed().as_millis(),
            "simulation restart world build completed"
        );

        let mut sim = self.simulation.write().await;
        sim.run = Some(SimulationRun {
            run_id,
            seed,
            world,
        });
        sim.phase = SimulationPhase::Running;
        sim.initialization_stage = None;
        sim.pending_restart = false;
        sim.runtime_config = config;
        info!(
            run_id,
            total_restart_ms = restart_begin.elapsed().as_millis(),
            "simulation restart completed"
        );

        Ok(simulation_status_from_locked(&sim))
    }
}
