use std::sync::Arc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use petri_core::{World, WorldConfig};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SimulationPhase {
    Idle,
    Running,
    Paused,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartupDraft {
    pub initial_creatures: usize,
    pub initial_food_density: f32,
    pub food_spawn_rate: f32,
    pub food_growth_rate: f32,
    pub energy_per_tick_decay: f32,
    pub energy_per_move: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StartupDraftPatch {
    pub initial_creatures: Option<usize>,
    pub initial_food_density: Option<f32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
    pub energy_per_tick_decay: Option<f32>,
    pub energy_per_move: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfigPatch {
    pub paused: Option<bool>,
    pub ticks_per_second: Option<u32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationStatus {
    pub phase: SimulationPhase,
    pub run_id: Option<u64>,
    pub seed: Option<u64>,
    pub tick: u64,
    pub population: usize,
    pub average_energy: f32,
    pub pending_restart: bool,
    pub startup_draft: StartupDraft,
}

#[derive(Debug)]
pub enum SimulationError {
    InvalidStartupRange {
        field: &'static str,
        min: f64,
        max: f64,
        actual: f64,
    },
    AlreadyRunning,
    NoActiveRun,
    NonViableStartupConfig,
}

impl SimulationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidStartupRange { .. } => "startup_value_out_of_range",
            Self::AlreadyRunning => "simulation_already_running",
            Self::NoActiveRun => "no_active_simulation",
            Self::NonViableStartupConfig => "non_viable_startup_config",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidStartupRange {
                field,
                min,
                max,
                actual,
            } => format!("{field} must be in range [{min}, {max}] but received {actual}",),
            Self::AlreadyRunning => "simulation is already active; use restart instead".into(),
            Self::NoActiveRun => "cannot restart because no simulation has been started yet".into(),
            Self::NonViableStartupConfig => {
                "startup configuration failed viability probe (100 ticks ended with extinction)"
                    .into()
            }
        }
    }
}

struct SimulationRun {
    run_id: u64,
    seed: u64,
    world: World,
}

pub struct SimulationState {
    phase: SimulationPhase,
    run: Option<SimulationRun>,
    startup_draft: StartupDraft,
    pending_restart: bool,
    runtime_config: WorldConfig,
    rng: SmallRng,
    next_run_id: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub simulation: Arc<RwLock<SimulationState>>,
    pub frames_tx: broadcast::Sender<Vec<u8>>,
}

impl StartupDraft {
    pub fn viable_default() -> Self {
        Self {
            initial_creatures: 300,
            initial_food_density: 0.25,
            food_spawn_rate: 0.10,
            food_growth_rate: 0.20,
            energy_per_tick_decay: 0.01,
            energy_per_move: 0.02,
        }
    }

    fn apply_patch(&mut self, patch: StartupDraftPatch) -> bool {
        let mut changed = false;

        if let Some(v) = patch.initial_creatures {
            changed |= self.initial_creatures != v;
            self.initial_creatures = v;
        }
        if let Some(v) = patch.initial_food_density {
            changed |= (self.initial_food_density - v).abs() > f32::EPSILON;
            self.initial_food_density = v;
        }
        if let Some(v) = patch.food_spawn_rate {
            changed |= (self.food_spawn_rate - v).abs() > f32::EPSILON;
            self.food_spawn_rate = v;
        }
        if let Some(v) = patch.food_growth_rate {
            changed |= (self.food_growth_rate - v).abs() > f32::EPSILON;
            self.food_growth_rate = v;
        }
        if let Some(v) = patch.energy_per_tick_decay {
            changed |= (self.energy_per_tick_decay - v).abs() > f32::EPSILON;
            self.energy_per_tick_decay = v;
        }
        if let Some(v) = patch.energy_per_move {
            changed |= (self.energy_per_move - v).abs() > f32::EPSILON;
            self.energy_per_move = v;
        }

        changed
    }

    fn validate(&self) -> Result<(), SimulationError> {
        validate_range(
            "initial_creatures",
            self.initial_creatures as f64,
            1.0,
            5_000.0,
        )?;
        validate_range(
            "initial_food_density",
            self.initial_food_density as f64,
            0.0,
            1.0,
        )?;
        validate_range("food_spawn_rate", self.food_spawn_rate as f64, 0.0, 1.0)?;
        validate_range("food_growth_rate", self.food_growth_rate as f64, 0.0, 1.0)?;
        validate_range(
            "energy_per_tick_decay",
            self.energy_per_tick_decay as f64,
            0.0,
            0.10,
        )?;
        validate_range("energy_per_move", self.energy_per_move as f64, 0.0, 0.10)?;
        Ok(())
    }
}

fn validate_range(
    field: &'static str,
    value: f64,
    min: f64,
    max: f64,
) -> Result<(), SimulationError> {
    if value < min || value > max {
        return Err(SimulationError::InvalidStartupRange {
            field,
            min,
            max,
            actual: value,
        });
    }
    Ok(())
}

impl AppState {
    pub fn new(seed: u64, config: WorldConfig) -> Self {
        let (frames_tx, _) = broadcast::channel(256);
        let state = SimulationState {
            phase: SimulationPhase::Idle,
            run: None,
            startup_draft: StartupDraft::viable_default(),
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
        if changed && sim.run.is_some() {
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

        if sim.run.is_some() {
            let (cfg, paused_update, tps_update, spawn_update, growth_update) = {
                let run = sim.run.as_mut().expect("checked above");
                let mut paused_update: Option<bool> = None;
                let mut tps_update: Option<u32> = None;
                let mut spawn_update: Option<f32> = None;
                let mut growth_update: Option<f32> = None;

                if let Some(paused) = patch.paused {
                    run.world.config.paused = paused;
                    paused_update = Some(paused);
                }
                if let Some(tps) = patch.ticks_per_second {
                    let tps = tps.max(1);
                    run.world.config.ticks_per_second = tps;
                    tps_update = Some(tps);
                }
                if let Some(rate) = patch.food_spawn_rate {
                    run.world.config.food_spawn_rate = rate.clamp(0.0, 1.0);
                    spawn_update = Some(run.world.config.food_spawn_rate);
                }
                if let Some(rate) = patch.food_growth_rate {
                    run.world.config.food_growth_rate = rate.clamp(0.0, 1.0);
                    growth_update = Some(run.world.config.food_growth_rate);
                }

                (
                    run.world.config.clone(),
                    paused_update,
                    tps_update,
                    spawn_update,
                    growth_update,
                )
            };

            if let Some(paused) = paused_update {
                sim.phase = if paused {
                    SimulationPhase::Paused
                } else {
                    SimulationPhase::Running
                };
                sim.runtime_config.paused = paused;
            }
            if let Some(tps) = tps_update {
                sim.runtime_config.ticks_per_second = tps;
            }
            if let Some(rate) = spawn_update {
                sim.runtime_config.food_spawn_rate = rate;
            }
            if let Some(rate) = growth_update {
                sim.runtime_config.food_growth_rate = rate;
            }
            cfg
        } else {
            if let Some(paused) = patch.paused {
                sim.runtime_config.paused = paused;
            }
            if let Some(tps) = patch.ticks_per_second {
                sim.runtime_config.ticks_per_second = tps.max(1);
            }
            if let Some(rate) = patch.food_spawn_rate {
                sim.runtime_config.food_spawn_rate = rate.clamp(0.0, 1.0);
            }
            if let Some(rate) = patch.food_growth_rate {
                sim.runtime_config.food_growth_rate = rate.clamp(0.0, 1.0);
            }
            sim.runtime_config.clone()
        }
    }

    pub async fn start_simulation(&self) -> Result<SimulationStatus, SimulationError> {
        let mut sim = self.simulation.write().await;
        if sim.run.is_some() {
            return Err(SimulationError::AlreadyRunning);
        }

        let seed = sim.rng.gen::<u64>();
        let config = build_world_config(&sim.runtime_config, &sim.startup_draft);
        ensure_viable_start(&config, sim.startup_draft.initial_food_density, seed)?;

        let mut world = World::new(config.clone(), seed);
        world.seed_food_density(sim.startup_draft.initial_food_density);

        let run_id = sim.next_run_id;
        sim.next_run_id += 1;
        sim.run = Some(SimulationRun {
            run_id,
            seed,
            world,
        });
        sim.phase = SimulationPhase::Running;
        sim.pending_restart = false;
        sim.runtime_config = config;

        Ok(simulation_status_from_locked(&sim))
    }

    pub async fn restart_simulation(&self) -> Result<SimulationStatus, SimulationError> {
        let mut sim = self.simulation.write().await;
        if sim.run.is_none() {
            return Err(SimulationError::NoActiveRun);
        }

        let seed = sim.rng.gen::<u64>();
        let config = build_world_config(&sim.runtime_config, &sim.startup_draft);
        ensure_viable_start(&config, sim.startup_draft.initial_food_density, seed)?;

        let mut world = World::new(config.clone(), seed);
        world.seed_food_density(sim.startup_draft.initial_food_density);

        let run_id = sim.next_run_id;
        sim.next_run_id += 1;
        sim.run = Some(SimulationRun {
            run_id,
            seed,
            world,
        });
        sim.phase = SimulationPhase::Running;
        sim.pending_restart = false;
        sim.runtime_config = config;

        Ok(simulation_status_from_locked(&sim))
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

fn build_world_config(base: &WorldConfig, draft: &StartupDraft) -> WorldConfig {
    let mut cfg = base.clone();
    cfg.width = 200;
    cfg.height = 200;
    cfg.initial_creatures = draft.initial_creatures;
    cfg.food_spawn_rate = draft.food_spawn_rate;
    cfg.food_growth_rate = draft.food_growth_rate;
    cfg.energy_per_tick_decay = draft.energy_per_tick_decay;
    cfg.energy_per_move = draft.energy_per_move;
    cfg.paused = false;
    cfg
}

fn ensure_viable_start(
    config: &WorldConfig,
    initial_food_density: f32,
    seed: u64,
) -> Result<(), SimulationError> {
    let mut probe = World::new(config.clone(), seed);
    probe.seed_food_density(initial_food_density);
    for _ in 0..100 {
        probe.tick();
        if probe.creature_count() == 0 {
            break;
        }
    }

    if probe.creature_count() == 0 {
        return Err(SimulationError::NonViableStartupConfig);
    }
    Ok(())
}

fn simulation_status_from_locked(sim: &SimulationState) -> SimulationStatus {
    let (run_id, seed, tick, population, average_energy) = if let Some(run) = sim.run.as_ref() {
        (
            Some(run.run_id),
            Some(run.seed),
            run.world.tick_count(),
            run.world.creature_count(),
            run.world.average_energy(),
        )
    } else {
        (None, None, 0, 0, 0.0)
    };

    SimulationStatus {
        phase: sim.phase,
        run_id,
        seed,
        tick,
        population,
        average_energy,
        pending_restart: sim.pending_restart,
        startup_draft: sim.startup_draft.clone(),
    }
}
