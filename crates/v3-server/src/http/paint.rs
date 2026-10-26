use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use v3_core::kernel::paint::{PaintPoint, PaintStats, PaintTool};

use crate::error::{AppError, FieldError};
use crate::query::cache::{paint_dirty_rect, world_static_changed, DirtyRect};
use crate::state::build_ws_frame;
use crate::state::{AppState, SimulationStatus};
use crate::types::PROTOCOL_VERSION;

/// The tool a paint stroke applies (G22: the wire form of `PaintTool`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum PaintToolRequest {
    Food,
    Barrier,
    EraseFood,
    EraseBarrier,
}

impl From<PaintToolRequest> for PaintTool {
    fn from(tool: PaintToolRequest) -> Self {
        match tool {
            PaintToolRequest::Food => Self::Food,
            PaintToolRequest::Barrier => Self::Barrier,
            PaintToolRequest::EraseFood => Self::EraseFood,
            PaintToolRequest::EraseBarrier => Self::EraseBarrier,
        }
    }
}

/// What a paint stroke or a pattern apply changed (G22: the wire form of
/// `PaintStats`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PaintStatsResponse {
    pub affected_cells: usize,
    pub food_set_cells: usize,
    pub food_cleared_cells: usize,
    pub barrier_set_cells: usize,
    pub barrier_cleared_cells: usize,
    pub creatures_removed: usize,
}

impl From<&PaintStats> for PaintStatsResponse {
    fn from(stats: &PaintStats) -> Self {
        Self {
            affected_cells: stats.affected_cells,
            food_set_cells: stats.food_set_cells,
            food_cleared_cells: stats.food_cleared_cells,
            barrier_set_cells: stats.barrier_set_cells,
            barrier_cleared_cells: stats.barrier_cleared_cells,
            creatures_removed: stats.creatures_removed,
        }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PaintRequest {
    pub tool: PaintToolRequest,
    pub brush_half_extent: u8,
    pub points: Vec<PaintPointDto>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PaintPointDto {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct PaintResponse {
    protocol_version: &'static str,
    stats: PaintStatsResponse,
    dirty_rect: DirtyRect,
    world_static_changed: bool,
}

pub async fn paint(
    State(app): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let req: PaintRequest = serde_json::from_slice(&body)
        .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?;

    if req.brush_half_extent > 2 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "brush_half_extent".into(),
                reason: "must be 0, 1, or 2".into(),
            }],
            endpoint: "paint",
        });
    }

    const MAX_PAINT_POINTS: usize = 10_000;
    if req.points.len() > MAX_PAINT_POINTS {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "points".into(),
                reason: format!("must contain at most {MAX_PAINT_POINTS} points"),
            }],
            endpoint: "paint",
        });
    }

    if req.points.is_empty() {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "points".into(),
                reason: "must contain at least one point".into(),
            }],
            endpoint: "paint",
        });
    }

    let mut handle = app.sim.lock().await;

    if handle.status == SimulationStatus::Running {
        return Err(AppError::InvalidStateTransition {
            expected: Some("idle or paused".into()),
            current: "running".into(),
        });
    }

    let (w, h) = (handle.sim.world.width, handle.sim.world.height);
    let points: Vec<PaintPoint> = req
        .points
        .iter()
        .filter(|p| p.x < w && p.y < h)
        .map(|p| PaintPoint { x: p.x, y: p.y })
        .collect();

    if points.is_empty() {
        return Ok(Json(PaintResponse {
            protocol_version: PROTOCOL_VERSION,
            stats: PaintStatsResponse::default(),
            dirty_rect: DirtyRect::empty(),
            world_static_changed: false,
        }));
    }

    let dirty_rect =
        paint_dirty_rect(w, h, req.brush_half_extent, &points).unwrap_or_else(DirtyRect::empty);
    let tool = PaintTool::from(req.tool);
    let stats = handle.sim.apply_paint(tool, req.brush_half_extent, &points);
    let static_changed = world_static_changed(tool, &stats);

    let frame = build_ws_frame(&handle);
    drop(handle);
    app.publish_ws_frame_update(frame, Some(dirty_rect));

    Ok(Json(PaintResponse {
        protocol_version: PROTOCOL_VERSION,
        stats: PaintStatsResponse::from(&stats),
        dirty_rect,
        world_static_changed: static_changed,
    }))
}
