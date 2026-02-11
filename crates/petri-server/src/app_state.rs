use std::sync::Arc;
use std::time::Instant;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use petri_core::{CreatureDetail, World, WorldConfig, WorldSnapshot};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SimulationPhase {
    Idle,
    Starting,
    Running,
    Paused,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartupDraft {
    pub initial_creatures: usize,
    pub max_creatures: usize,
    pub width: u32,
    pub height: u32,
    pub initial_food_density: f32,
    pub energy_initial: f32,
    pub food_spawn_rate: f32,
    pub food_growth_rate: f32,
    pub food_spread_threshold: f32,
    pub food_spawn_floor_density: f32,
    pub energy_per_tick_decay: f32,
    pub energy_per_move: f32,
    pub world_wrap: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StartupDraftPatch {
    pub initial_creatures: Option<usize>,
    pub max_creatures: Option<usize>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub initial_food_density: Option<f32>,
    pub energy_initial: Option<f32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
    pub food_spread_threshold: Option<f32>,
    pub food_spawn_floor_density: Option<f32>,
    pub energy_per_tick_decay: Option<f32>,
    pub energy_per_move: Option<f32>,
    pub world_wrap: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfigPatch {
    pub paused: Option<bool>,
    pub ticks_per_second: Option<u32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
    pub food_spread_threshold: Option<f32>,
    pub food_spawn_floor_density: Option<f32>,
    pub food_max_density: Option<f32>,
    pub food_energy_value: Option<f32>,
    pub energy_per_tick_decay: Option<f32>,
    pub energy_per_move: Option<f32>,
    pub energy_per_compute_node: Option<f32>,
    pub energy_per_reproduce: Option<f32>,
    pub energy_max: Option<f32>,
    pub min_reproduce_energy: Option<f32>,
    pub offspring_energy_fraction: Option<f32>,
    pub max_creatures: Option<usize>,
    pub weight_mutation_rate: Option<f32>,
    pub weight_mutation_magnitude: Option<f32>,
    pub logic_node_mutation_rate: Option<f32>,
    pub structural_mutation_rate: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationStatus {
    pub phase: SimulationPhase,
    pub initialization_stage: Option<&'static str>,
    pub viability_probe_enabled: bool,
    pub run_id: Option<u64>,
    pub seed: Option<u64>,
    pub tick: u64,
    pub population: usize,
    pub average_energy: f32,
    pub pending_restart: bool,
    pub startup_viable: bool,
    pub startup_viability_code: Option<&'static str>,
    pub startup_viability_message: Option<String>,
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

#[derive(Clone, Debug)]
struct StartupViability {
    is_viable: bool,
    code: Option<&'static str>,
    message: Option<String>,
}

impl StartupViability {
    fn viable() -> Self {
        Self {
            is_viable: true,
            code: None,
            message: None,
        }
    }

    fn from_error(err: SimulationError) -> Self {
        Self {
            is_viable: false,
            code: Some(err.code()),
            message: Some(err.message()),
        }
    }
}

pub struct SimulationState {
    phase: SimulationPhase,
    initialization_stage: Option<&'static str>,
    viability_probe_enabled: bool,
    run: Option<SimulationRun>,
    startup_draft: StartupDraft,
    startup_viability: StartupViability,
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

#[derive(Clone, Copy, Debug)]
pub struct AppStateOptions {
    pub viability_probe_enabled: bool,
}

impl Default for AppStateOptions {
    fn default() -> Self {
        Self {
            viability_probe_enabled: true,
        }
    }
}

impl StartupDraft {
    pub fn viable_default() -> Self {
        Self {
            initial_creatures: 300,
            max_creatures: 500_000,
            width: 400,
            height: 400,
            initial_food_density: 0.15,
            energy_initial: 0.7,
            food_spawn_rate: 0.05,
            food_growth_rate: 0.10,
            food_spread_threshold: 0.75,
            food_spawn_floor_density: 0.03,
            energy_per_tick_decay: 0.01,
            energy_per_move: 0.02,
            world_wrap: true,
        }
    }

    fn apply_patch(&mut self, patch: StartupDraftPatch) -> bool {
        let mut changed = false;

        if let Some(v) = patch.initial_creatures {
            changed |= self.initial_creatures != v;
            self.initial_creatures = v;
        }
        if let Some(v) = patch.max_creatures {
            changed |= self.max_creatures != v;
            self.max_creatures = v;
        }
        if let Some(v) = patch.width {
            changed |= self.width != v;
            self.width = v;
        }
        if let Some(v) = patch.height {
            changed |= self.height != v;
            self.height = v;
        }
        if let Some(v) = patch.initial_food_density {
            changed |= (self.initial_food_density - v).abs() > f32::EPSILON;
            self.initial_food_density = v;
        }
        if let Some(v) = patch.energy_initial {
            changed |= (self.energy_initial - v).abs() > f32::EPSILON;
            self.energy_initial = v;
        }
        if let Some(v) = patch.food_spawn_rate {
            changed |= (self.food_spawn_rate - v).abs() > f32::EPSILON;
            self.food_spawn_rate = v;
        }
        if let Some(v) = patch.food_growth_rate {
            changed |= (self.food_growth_rate - v).abs() > f32::EPSILON;
            self.food_growth_rate = v;
        }
        if let Some(v) = patch.food_spread_threshold {
            changed |= (self.food_spread_threshold - v).abs() > f32::EPSILON;
            self.food_spread_threshold = v;
        }
        if let Some(v) = patch.food_spawn_floor_density {
            changed |= (self.food_spawn_floor_density - v).abs() > f32::EPSILON;
            self.food_spawn_floor_density = v;
        }
        if let Some(v) = patch.energy_per_tick_decay {
            changed |= (self.energy_per_tick_decay - v).abs() > f32::EPSILON;
            self.energy_per_tick_decay = v;
        }
        if let Some(v) = patch.energy_per_move {
            changed |= (self.energy_per_move - v).abs() > f32::EPSILON;
            self.energy_per_move = v;
        }
        if let Some(v) = patch.world_wrap {
            changed |= self.world_wrap != v;
            self.world_wrap = v;
        }

        changed
    }

    fn validate(&self) -> Result<(), SimulationError> {
        validate_range(
            "initial_creatures",
            self.initial_creatures as f64,
            1.0,
            20_000.0,
        )?;
        validate_range("max_creatures", self.max_creatures as f64, 1.0, 500_000.0)?;
        validate_range("width", self.width as f64, 50.0, 2_000.0)?;
        validate_range("height", self.height as f64, 50.0, 2_000.0)?;
        validate_range("energy_initial", self.energy_initial as f64, 0.01, 20.0)?;
        validate_range(
            "initial_food_density",
            self.initial_food_density as f64,
            0.0,
            1.0,
        )?;
        validate_range("food_spawn_rate", self.food_spawn_rate as f64, 0.0, 1.0)?;
        validate_range("food_growth_rate", self.food_growth_rate as f64, 0.0, 1.0)?;
        validate_range(
            "food_spread_threshold",
            self.food_spread_threshold as f64,
            0.0,
            1.0,
        )?;
        validate_range(
            "food_spawn_floor_density",
            self.food_spawn_floor_density as f64,
            0.0,
            1.0,
        )?;
        validate_range(
            "energy_per_tick_decay",
            self.energy_per_tick_decay as f64,
            0.0,
            0.50,
        )?;
        validate_range("energy_per_move", self.energy_per_move as f64, 0.0, 0.50)?;
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

fn build_world_config(base: &WorldConfig, draft: &StartupDraft) -> WorldConfig {
    let mut cfg = base.clone();
    cfg.width = draft.width;
    cfg.height = draft.height;
    cfg.world_wrap = draft.world_wrap;
    cfg.initial_creatures = draft.initial_creatures;
    cfg.max_creatures = draft.max_creatures;
    cfg.energy_initial = draft.energy_initial;
    cfg.food_spawn_rate = draft.food_spawn_rate;
    cfg.food_growth_rate = draft.food_growth_rate;
    cfg.food_spread_threshold = draft.food_spread_threshold;
    cfg.food_spawn_floor_density = draft.food_spawn_floor_density;
    cfg.energy_per_tick_decay = draft.energy_per_tick_decay;
    cfg.energy_per_move = draft.energy_per_move;
    cfg.paused = false;
    cfg
}

fn startup_draft_from_config(config: &WorldConfig, initial_food_density: f32) -> StartupDraft {
    StartupDraft {
        initial_creatures: config.initial_creatures,
        max_creatures: config.max_creatures,
        width: config.width,
        height: config.height,
        initial_food_density,
        energy_initial: config.energy_initial,
        food_spawn_rate: config.food_spawn_rate,
        food_growth_rate: config.food_growth_rate,
        food_spread_threshold: config.food_spread_threshold,
        food_spawn_floor_density: config.food_spawn_floor_density,
        energy_per_tick_decay: config.energy_per_tick_decay,
        energy_per_move: config.energy_per_move,
        world_wrap: config.world_wrap,
    }
}

fn ensure_viable_start(
    config: &WorldConfig,
    initial_food_density: f32,
    seed: u64,
) -> Result<usize, SimulationError> {
    let mut probe = World::new(config.clone(), seed);
    probe.seed_food_density(initial_food_density);
    for _ in 0..100 {
        probe.tick();
        if probe.creature_count() == 0 {
            break;
        }
    }

    let survivors = probe.creature_count();
    if survivors == 0 {
        return Err(SimulationError::NonViableStartupConfig);
    }
    Ok(survivors)
}

fn apply_runtime_patch(config: &mut WorldConfig, patch: &RuntimeConfigPatch) {
    if let Some(paused) = patch.paused {
        config.paused = paused;
    }
    if let Some(tps) = patch.ticks_per_second {
        config.ticks_per_second = tps.max(1);
    }
    if let Some(rate) = patch.food_spawn_rate {
        config.food_spawn_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(rate) = patch.food_growth_rate {
        config.food_growth_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(threshold) = patch.food_spread_threshold {
        config.food_spread_threshold = threshold.clamp(0.0, 1.0);
    }
    if let Some(floor) = patch.food_spawn_floor_density {
        config.food_spawn_floor_density = floor.clamp(0.0, 1.0);
    }
    if let Some(density) = patch.food_max_density {
        config.food_max_density = density.max(0.01);
    }
    if let Some(value) = patch.food_energy_value {
        config.food_energy_value = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_tick_decay {
        config.energy_per_tick_decay = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_move {
        config.energy_per_move = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_compute_node {
        config.energy_per_compute_node = value.max(0.0);
    }
    if let Some(value) = patch.energy_per_reproduce {
        config.energy_per_reproduce = value.max(0.0);
    }
    if let Some(value) = patch.energy_max {
        config.energy_max = value.max(0.01);
    }
    if let Some(value) = patch.min_reproduce_energy {
        config.min_reproduce_energy = value.max(0.0);
    }
    if let Some(value) = patch.offspring_energy_fraction {
        config.offspring_energy_fraction = value.clamp(0.0, 1.0);
    }
    if let Some(max_creatures) = patch.max_creatures {
        config.max_creatures = max_creatures.max(1);
    }
    if let Some(rate) = patch.weight_mutation_rate {
        config.weight_mutation_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(magnitude) = patch.weight_mutation_magnitude {
        config.weight_mutation_magnitude = magnitude.max(0.0);
    }
    if let Some(rate) = patch.logic_node_mutation_rate {
        config.logic_node_mutation_rate = rate.clamp(0.0, 1.0);
    }
    if let Some(rate) = patch.structural_mutation_rate {
        config.structural_mutation_rate = rate.clamp(0.0, 1.0);
    }
}

fn evaluate_startup_viability(
    base: &WorldConfig,
    draft: &StartupDraft,
    viability_probe_enabled: bool,
) -> StartupViability {
    if let Err(err) = draft.validate() {
        return StartupViability::from_error(err);
    }
    if !viability_probe_enabled {
        return StartupViability::viable();
    }

    let config = build_world_config(base, draft);
    match ensure_viable_start(
        &config,
        draft.initial_food_density,
        startup_probe_seed(draft),
    ) {
        Ok(_) => StartupViability::viable(),
        Err(err) => StartupViability::from_error(err),
    }
}

fn startup_probe_seed(draft: &StartupDraft) -> u64 {
    let mut seed = 0xA11C_E5EED_u64;
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.initial_creatures as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.max_creatures as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.width as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.height as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.initial_food_density.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_initial.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_spawn_rate.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_growth_rate.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_spread_threshold.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.food_spawn_floor_density.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_per_tick_decay.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(draft.energy_per_move.to_bits() as u64);
    seed = seed
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(u64::from(draft.world_wrap));
    seed
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
        initialization_stage: sim.initialization_stage,
        viability_probe_enabled: sim.viability_probe_enabled,
        run_id,
        seed,
        tick,
        population,
        average_energy,
        pending_restart: sim.pending_restart,
        startup_viable: sim.startup_viability.is_viable,
        startup_viability_code: sim.startup_viability.code,
        startup_viability_message: sim.startup_viability.message.clone(),
        startup_draft: sim.startup_draft.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::StartupDraft;

    #[test]
    fn startup_draft_validation_allows_expanded_upper_bounds() {
        let mut draft = StartupDraft::viable_default();
        draft.initial_creatures = 12_000;
        draft.max_creatures = 450_000;
        draft.width = 1_400;
        draft.height = 1_400;
        draft.energy_initial = 12.0;
        draft.energy_per_tick_decay = 0.25;
        draft.energy_per_move = 0.25;

        assert!(draft.validate().is_ok());
    }
}
