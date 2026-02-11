use std::sync::Arc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio::sync::{broadcast, RwLock};

use petri_core::{CreatureDetail, World, WorldConfig, WorldSnapshot};

mod lifecycle;
mod paint;
mod runtime_patch;
mod startup_draft;
mod status;
mod types;
mod viability;

#[cfg(test)]
mod tests;

pub use startup_draft::{StartupDraft, StartupDraftPatch};
pub use types::{
    AppState, AppStateOptions, IdlePreviewMode, RuntimeConfigPatch, SimulationError,
    SimulationPhase, SimulationState, SimulationStatus, WorldPaintAction, WorldPaintRequest,
    WorldPaintResponse,
};

use self::runtime_patch::apply_runtime_patch;
use self::startup_draft::{build_world_config, startup_draft_from_config};
use self::status::simulation_status_from_locked;
use self::types::SimulationRun;
use self::viability::{evaluate_startup_viability, startup_probe_seed};
impl AppState {
    pub fn new(seed: u64, config: WorldConfig) -> Self {
        Self::new_with_options(seed, config, AppStateOptions::default())
    }

    pub fn new_with_options(seed: u64, config: WorldConfig, options: AppStateOptions) -> Self {
        let (frames_tx, _) = broadcast::channel(256);
        let startup_draft = StartupDraft::viable_default();
        let startup_viability =
            evaluate_startup_viability(&config, &startup_draft, options.viability_probe_enabled);
        let state = SimulationState {
            phase: SimulationPhase::Idle,
            initialization_stage: None,
            viability_probe_enabled: options.viability_probe_enabled,
            run: None,
            startup_draft,
            startup_paint_layer: Default::default(),
            startup_viability,
            pending_restart: false,
            runtime_config: config,
            rng: SmallRng::seed_from_u64(seed),
            next_run_id: 1,
        };
        Self {
            simulation: Arc::new(RwLock::new(state)),
            frames_tx,
        }
    }

    pub fn new_for_tests() -> Self {
        let config = WorldConfig::default();
        Self::new(1, config)
    }

    #[cfg(test)]
    pub fn new_for_tests_fast() -> Self {
        let config = WorldConfig::default();
        Self::new_with_options(
            1,
            config,
            AppStateOptions {
                viability_probe_enabled: false,
            },
        )
    }

    pub async fn simulation_status(&self) -> SimulationStatus {
        let sim = self.simulation.read().await;
        simulation_status_from_locked(&sim)
    }

    pub async fn startup_draft(&self) -> StartupDraft {
        let sim = self.simulation.read().await;
        sim.startup_draft.clone()
    }

    pub async fn patch_startup_draft(
        &self,
        patch: StartupDraftPatch,
    ) -> Result<StartupDraft, SimulationError> {
        let mut sim = self.simulation.write().await;
        let mut updated = sim.startup_draft.clone();
        let changed = updated.apply_patch(patch);
        updated.validate()?;
        sim.startup_draft = updated.clone();
        sim.startup_paint_layer
            .clip_to_bounds(updated.width, updated.height);
        sim.startup_viability = evaluate_startup_viability(
            &sim.runtime_config,
            &sim.startup_draft,
            sim.viability_probe_enabled,
        );
        if changed && (sim.run.is_some() || sim.phase == SimulationPhase::Starting) {
            sim.pending_restart = true;
        }
        Ok(updated)
    }

    pub async fn current_runtime_config(&self) -> WorldConfig {
        let sim = self.simulation.read().await;
        sim.run
            .as_ref()
            .map(|run| run.world.config.clone())
            .unwrap_or_else(|| sim.runtime_config.clone())
    }

    pub async fn patch_runtime_config(&self, patch: RuntimeConfigPatch) -> WorldConfig {
        let mut sim = self.simulation.write().await;

        if let Some(run) = sim.run.as_mut() {
            apply_runtime_patch(&mut run.world.config, &patch);
            let cfg = run.world.config.clone();
            sim.runtime_config = cfg.clone();
            sim.phase = if cfg.paused {
                SimulationPhase::Paused
            } else {
                SimulationPhase::Running
            };
            cfg
        } else {
            apply_runtime_patch(&mut sim.runtime_config, &patch);
            sim.runtime_config.clone()
        }
    }
    pub async fn simulation_snapshot(&self) -> WorldSnapshot {
        let sim = self.simulation.read().await;
        if let Some(run) = sim.run.as_ref() {
            return run.world.snapshot();
        }

        let mut world = World::new(
            build_world_config(&sim.runtime_config, &sim.startup_draft),
            startup_probe_seed(&sim.startup_draft),
        );
        world.seed_food_density(sim.startup_draft.initial_food_density);
        sim.startup_paint_layer.apply_to_world(&mut world);
        world.snapshot()
    }

    pub async fn creature_detail(&self, creature_id: u64) -> Option<CreatureDetail> {
        let sim = self.simulation.read().await;
        sim.run
            .as_ref()
            .and_then(|run| run.world.creature_detail(creature_id))
    }

    pub async fn load_simulation_snapshot(&self, snapshot: WorldSnapshot) -> SimulationStatus {
        let mut sim = self.simulation.write().await;
        let world = World::from_snapshot(snapshot);

        let run_id = sim.next_run_id;
        sim.next_run_id += 1;
        let seed = sim.rng.gen::<u64>();
        sim.run = Some(SimulationRun {
            run_id,
            seed,
            world,
        });
        sim.phase = SimulationPhase::Running;
        sim.initialization_stage = None;
        sim.pending_restart = false;
        sim.startup_paint_layer = Default::default();
        if let Some(run) = sim.run.as_ref() {
            sim.runtime_config = run.world.config.clone();
            sim.startup_draft = startup_draft_from_config(
                &sim.runtime_config,
                sim.startup_draft.initial_food_density,
            );
        }
        sim.startup_viability = evaluate_startup_viability(
            &sim.runtime_config,
            &sim.startup_draft,
            sim.viability_probe_enabled,
        );
        simulation_status_from_locked(&sim)
    }

    pub async fn run_single_iteration(&self) -> (Option<Vec<u8>>, u64) {
        let mut sim = self.simulation.write().await;

        let tps = sim
            .run
            .as_ref()
            .map(|run| run.world.config.ticks_per_second)
            .unwrap_or(sim.runtime_config.ticks_per_second)
            .max(1);
        let delay_ms = (1000 / tps as u64).max(1);

        if sim.phase != SimulationPhase::Running {
            return (None, delay_ms);
        }

        let Some(run) = sim.run.as_mut() else {
            sim.phase = SimulationPhase::Idle;
            return (None, delay_ms);
        };

        run.world.tick();
        let frame = run.world.frame();
        (rmp_serde::to_vec_named(&frame).ok(), delay_ms)
    }
}
