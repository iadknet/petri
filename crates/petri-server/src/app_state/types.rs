use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use petri_core::{PaintPoint, PaintStats, PaintTool, World, WorldConfig, WorldFrame};

use super::startup_draft::StartupDraft;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SimulationPhase {
    Idle,
    Starting,
    Running,
    Paused,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfigPatch {
    pub paused: Option<bool>,
    pub ticks_per_second: Option<u32>,
    pub sensor_radius: Option<u32>,
    pub food_spawn_rate: Option<f32>,
    pub food_growth_rate: Option<f32>,
    pub food_spread_threshold: Option<f32>,
    pub food_spawn_floor_density: Option<f32>,
    pub food_max_density: Option<f32>,
    pub food_energy_value: Option<f32>,
    pub energy_per_tick_decay: Option<f32>,
    pub energy_per_move: Option<f32>,
    pub energy_per_inventory_attempt: Option<f32>,
    pub energy_per_compute_node: Option<f32>,
    pub energy_per_reproduce: Option<f32>,
    pub illegal_action_energy_penalty: Option<f32>,
    pub energy_max: Option<f32>,
    pub min_reproduce_energy: Option<f32>,
    pub offspring_energy_fraction: Option<f32>,
    pub max_creatures: Option<usize>,
    pub weight_mutation_rate: Option<f32>,
    pub weight_mutation_magnitude: Option<f32>,
    pub logic_node_mutation_rate: Option<f32>,
    pub structural_mutation_rate: Option<f32>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldPaintAction {
    Stroke,
    ClearAll,
    Preview,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdlePreviewMode {
    PaintLayer,
    FullStartup,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldPaintRequest {
    pub action: WorldPaintAction,
    #[serde(default)]
    pub tool: Option<PaintTool>,
    #[serde(default)]
    pub brush_half_extent: Option<u8>,
    #[serde(default)]
    pub points: Vec<PaintPoint>,
    #[serde(default)]
    pub idle_preview_mode: Option<IdlePreviewMode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldPaintResponse {
    pub phase: SimulationPhase,
    pub stats: PaintStats,
    pub frame: WorldFrame,
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
    PaintPhaseNotEditable {
        phase: SimulationPhase,
    },
    InvalidPaintRequest {
        message: String,
    },
}

impl SimulationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidStartupRange { .. } => "startup_value_out_of_range",
            Self::AlreadyRunning => "simulation_already_running",
            Self::NoActiveRun => "no_active_simulation",
            Self::NonViableStartupConfig => "non_viable_startup_config",
            Self::PaintPhaseNotEditable { .. } => "paint_phase_not_editable",
            Self::InvalidPaintRequest { .. } => "invalid_paint_request",
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
            Self::PaintPhaseNotEditable { phase } => {
                format!(
                    "painting is only allowed in idle or paused phase; current phase is {phase:?}"
                )
            }
            Self::InvalidPaintRequest { message } => message.clone(),
        }
    }
}

pub(super) struct SimulationRun {
    pub(super) run_id: u64,
    pub(super) seed: u64,
    pub(super) world: World,
}

#[derive(Clone, Debug)]
pub(super) struct StartupViability {
    pub(super) is_viable: bool,
    pub(super) code: Option<&'static str>,
    pub(super) message: Option<String>,
}

impl StartupViability {
    pub(super) fn viable() -> Self {
        Self {
            is_viable: true,
            code: None,
            message: None,
        }
    }

    pub(super) fn from_error(err: SimulationError) -> Self {
        Self {
            is_viable: false,
            code: Some(err.code()),
            message: Some(err.message()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct StartupPaintLayer {
    pub(super) food_overrides: HashMap<PaintPoint, bool>,
    pub(super) barrier_overrides: HashMap<PaintPoint, bool>,
}

impl StartupPaintLayer {
    pub(super) fn clip_to_bounds(&mut self, width: u32, height: u32) {
        self.food_overrides
            .retain(|point, _| point.x < width && point.y < height);
        self.barrier_overrides
            .retain(|point, _| point.x < width && point.y < height);
    }

    pub(super) fn apply_stroke(
        &mut self,
        tool: PaintTool,
        brush_half_extent: u8,
        points: &[PaintPoint],
        width: u32,
        height: u32,
    ) -> Result<PaintStats, SimulationError> {
        let touched = touched_points_from_stroke(brush_half_extent, points, width, height)?;
        let mut stats = PaintStats::default();
        for point in touched {
            stats.affected_cells += 1;
            match tool {
                PaintTool::Food => {
                    if self.food_overrides.insert(point, true) != Some(true) {
                        stats.food_set_cells += 1;
                    }
                }
                PaintTool::Barrier => {
                    if self.barrier_overrides.insert(point, true) != Some(true) {
                        stats.barrier_set_cells += 1;
                    }
                }
                PaintTool::EraseFood => {
                    if self.food_overrides.insert(point, false) != Some(false) {
                        stats.food_cleared_cells += 1;
                    }
                }
                PaintTool::EraseBarrier => {
                    if self.barrier_overrides.insert(point, false) != Some(false) {
                        stats.barrier_cleared_cells += 1;
                    }
                }
            }
        }
        Ok(stats)
    }

    pub(super) fn clear_all(&mut self) -> PaintStats {
        let affected_cells = self
            .food_overrides
            .keys()
            .chain(self.barrier_overrides.keys())
            .copied()
            .collect::<HashSet<_>>()
            .len();
        let stats = PaintStats {
            affected_cells,
            food_set_cells: 0,
            food_cleared_cells: self.food_overrides.len(),
            barrier_set_cells: 0,
            barrier_cleared_cells: self.barrier_overrides.len(),
            creatures_removed: 0,
        };
        self.food_overrides.clear();
        self.barrier_overrides.clear();
        stats
    }

    pub(super) fn apply_to_world(&self, world: &mut World) {
        for (point, set_food) in &self.food_overrides {
            let tool = if *set_food {
                PaintTool::Food
            } else {
                PaintTool::EraseFood
            };
            let _ = world.apply_paint_stroke(tool, 0, &[*point]);
        }
        for (point, set_barrier) in &self.barrier_overrides {
            let tool = if *set_barrier {
                PaintTool::Barrier
            } else {
                PaintTool::EraseBarrier
            };
            let _ = world.apply_paint_stroke(tool, 0, &[*point]);
        }
    }
}

fn touched_points_from_stroke(
    brush_half_extent: u8,
    points: &[PaintPoint],
    width: u32,
    height: u32,
) -> Result<HashSet<PaintPoint>, SimulationError> {
    if brush_half_extent > 2 {
        return Err(SimulationError::InvalidPaintRequest {
            message: format!(
                "brush_half_extent must be 0, 1, or 2 but received {brush_half_extent}"
            ),
        });
    }

    let mut touched = HashSet::new();
    let radius = brush_half_extent as i32;
    for point in points {
        let px = point.x as i32;
        let py = point.y as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = px + dx;
                let y = py + dy;
                if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                    continue;
                }
                touched.insert(PaintPoint {
                    x: x as u32,
                    y: y as u32,
                });
            }
        }
    }
    Ok(touched)
}

pub struct SimulationState {
    pub(super) phase: SimulationPhase,
    pub(super) initialization_stage: Option<&'static str>,
    pub(super) viability_probe_enabled: bool,
    pub(super) run: Option<SimulationRun>,
    pub(super) startup_draft: StartupDraft,
    pub(super) startup_paint_layer: StartupPaintLayer,
    pub(super) startup_viability: StartupViability,
    pub(super) pending_restart: bool,
    pub(super) runtime_config: WorldConfig,
    pub(super) rng: SmallRng,
    pub(super) next_run_id: u64,
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
