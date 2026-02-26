use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use v3_core::kernel::paint::{PaintPoint, PaintStats, PaintTool};

use crate::error::{AppError, FieldError};
use crate::handlers::lifecycle::build_ws_frame;
use crate::state::{AppState, SimulationStatus, WsFrame};
use crate::types::PROTOCOL_VERSION;

#[derive(Debug, Deserialize)]
pub struct PaintRequest {
    pub tool: PaintTool,
    pub brush_half_extent: u8,
    pub points: Vec<PaintPointDto>,
}

#[derive(Debug, Deserialize)]
pub struct PaintPointDto {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Serialize)]
struct PaintResponse {
    protocol_version: &'static str,
    stats: PaintStats,
    frame: WsFrame,
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
        let frame = build_ws_frame(&handle);
        return Ok(Json(PaintResponse {
            protocol_version: PROTOCOL_VERSION,
            stats: PaintStats::default(),
            frame,
        }));
    }

    let stats = handle
        .sim
        .apply_paint(req.tool, req.brush_half_extent, &points);

    let frame = build_ws_frame(&handle);

    Ok(Json(PaintResponse {
        protocol_version: PROTOCOL_VERSION,
        stats,
        frame,
    }))
}
