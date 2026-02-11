use petri_core::{PaintStats, World};

use super::startup_draft::build_world_config;
use super::viability::startup_probe_seed;
use super::{
    AppState, IdlePreviewMode, SimulationError, SimulationPhase, SimulationState, WorldPaintAction,
    WorldPaintRequest, WorldPaintResponse,
};

impl AppState {
    pub async fn paint_world(
        &self,
        request: WorldPaintRequest,
    ) -> Result<WorldPaintResponse, SimulationError> {
        let mut sim = self.simulation.write().await;
        match sim.phase {
            SimulationPhase::Idle => apply_idle_paint(&mut sim, request),
            SimulationPhase::Paused => apply_paused_paint(&mut sim, request),
            phase => Err(SimulationError::PaintPhaseNotEditable { phase }),
        }
    }
}

fn apply_idle_paint(
    sim: &mut SimulationState,
    request: WorldPaintRequest,
) -> Result<WorldPaintResponse, SimulationError> {
    let stats = match request.action {
        WorldPaintAction::Stroke => {
            let tool = request
                .tool
                .ok_or_else(|| SimulationError::InvalidPaintRequest {
                    message: "tool is required for stroke action".into(),
                })?;
            let brush_half_extent =
                request
                    .brush_half_extent
                    .ok_or_else(|| SimulationError::InvalidPaintRequest {
                        message: "brush_half_extent is required for stroke action".into(),
                    })?;

            sim.startup_paint_layer.apply_stroke(
                tool,
                brush_half_extent,
                &request.points,
                sim.startup_draft.width,
                sim.startup_draft.height,
            )?
        }
        WorldPaintAction::ClearAll => sim.startup_paint_layer.clear_all(),
        WorldPaintAction::Preview => PaintStats::default(),
    };

    let mode = request
        .idle_preview_mode
        .unwrap_or(IdlePreviewMode::PaintLayer);
    let frame = idle_preview_frame(sim, mode);
    Ok(WorldPaintResponse {
        phase: SimulationPhase::Idle,
        stats,
        frame,
    })
}

fn apply_paused_paint(
    sim: &mut SimulationState,
    request: WorldPaintRequest,
) -> Result<WorldPaintResponse, SimulationError> {
    let run = sim
        .run
        .as_mut()
        .ok_or(SimulationError::PaintPhaseNotEditable {
            phase: SimulationPhase::Idle,
        })?;

    let stats = match request.action {
        WorldPaintAction::Stroke => {
            let tool = request
                .tool
                .ok_or_else(|| SimulationError::InvalidPaintRequest {
                    message: "tool is required for stroke action".into(),
                })?;
            let brush_half_extent =
                request
                    .brush_half_extent
                    .ok_or_else(|| SimulationError::InvalidPaintRequest {
                        message: "brush_half_extent is required for stroke action".into(),
                    })?;

            run.world
                .apply_paint_stroke(tool, brush_half_extent, &request.points)
                .map_err(|err| match err {
                    petri_core::PaintError::InvalidBrushHalfExtent { received, .. } => {
                        SimulationError::InvalidPaintRequest {
                            message: format!(
                                "brush_half_extent must be 0, 1, or 2 but received {received}"
                            ),
                        }
                    }
                })?
        }
        WorldPaintAction::ClearAll => run.world.clear_painted_cells(),
        WorldPaintAction::Preview => PaintStats::default(),
    };

    let frame = run.world.frame();
    Ok(WorldPaintResponse {
        phase: SimulationPhase::Paused,
        stats,
        frame,
    })
}

fn idle_preview_frame(sim: &SimulationState, mode: IdlePreviewMode) -> petri_core::WorldFrame {
    let mut config = build_world_config(&sim.runtime_config, &sim.startup_draft);
    let seed = startup_probe_seed(&sim.startup_draft);

    match mode {
        IdlePreviewMode::PaintLayer => {
            config.initial_creatures = 0;
            let mut world = World::new(config, seed);
            sim.startup_paint_layer.apply_to_world(&mut world);
            world.frame()
        }
        IdlePreviewMode::FullStartup => {
            let mut world = World::new(config, seed);
            world.seed_food_density(sim.startup_draft.initial_food_density);
            sim.startup_paint_layer.apply_to_world(&mut world);
            world.frame()
        }
    }
}
