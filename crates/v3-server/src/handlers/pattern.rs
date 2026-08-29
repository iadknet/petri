use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use v3_core::kernel::paint::{PaintPoint, PaintStats, PaintTool};
use v3_core::patterns::{generate_pattern_seeded, PatternBounds, PatternParams};

use crate::error::{AppError, FieldError};
use crate::query::cache::{world_static_changed, DirtyRect};
use crate::state::build_ws_frame;
use crate::state::{AppState, SimulationStatus};
use crate::types::PROTOCOL_VERSION;

const MAX_PATTERN_AREA: u32 = 1_000_000;

#[derive(Debug, Deserialize)]
pub struct PatternRequest {
    pub params: PatternParams,
    pub bounds: PatternBounds,
    pub seed: u64,
}

#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub protocol_version: &'static str,
    pub bounds: PatternBounds,
    pub bitmap: String,
    pub cell_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ApplyResponse {
    pub protocol_version: &'static str,
    pub stats: PaintStats,
    pub dirty_rect: DirtyRect,
    pub world_static_changed: bool,
}

fn validate_request(
    req: &PatternRequest,
    world_width: u16,
    world_height: u16,
) -> Result<PatternBounds, AppError> {
    if req.bounds.width == 0 || req.bounds.height == 0 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "bounds".into(),
                reason: "width and height must be > 0".into(),
            }],
            endpoint: "pattern",
        });
    }

    // Clip bounds to world dimensions.
    let x = req.bounds.x.min(world_width.saturating_sub(1));
    let y = req.bounds.y.min(world_height.saturating_sub(1));
    let width = req.bounds.width.min(world_width.saturating_sub(x));
    let height = req.bounds.height.min(world_height.saturating_sub(y));

    let area = width as u32 * height as u32;
    if area > MAX_PATTERN_AREA {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "bounds".into(),
                reason: format!("area ({area}) exceeds maximum ({MAX_PATTERN_AREA})"),
            }],
            endpoint: "pattern",
        });
    }

    Ok(PatternBounds {
        x,
        y,
        width,
        height,
    })
}

/// Pack generated paint points into a row-major bitmap within bounds.
///
/// One bit per cell (MSB-first), base64-encoded. Bit = 1 means barrier cell.
fn encode_bitmap(bounds: &PatternBounds, points: &[PaintPoint]) -> String {
    let total_bits = bounds.width as usize * bounds.height as usize;
    let byte_count = total_bits.div_ceil(8);
    let mut bytes = vec![0u8; byte_count];

    for p in points {
        let col = (p.x - bounds.x) as usize;
        let row = (p.y - bounds.y) as usize;
        let bit_idx = row * bounds.width as usize + col;
        bytes[bit_idx / 8] |= 1 << (7 - (bit_idx % 8)); // MSB-first
    }

    BASE64.encode(&bytes)
}

pub async fn preview(
    State(app): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let req: PatternRequest = serde_json::from_slice(&body)
        .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?;

    let handle = app.sim.lock().await;
    let (w, h) = (handle.sim.world.width, handle.sim.world.height);
    drop(handle);

    let clipped = validate_request(&req, w, h)?;
    let points = generate_pattern_seeded(clipped, &req.params, req.seed);
    let bitmap = encode_bitmap(&clipped, &points);

    Ok(Json(PreviewResponse {
        protocol_version: PROTOCOL_VERSION,
        bounds: clipped,
        bitmap,
        cell_count: points.len() as u32,
    }))
}

pub async fn apply(
    State(app): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let req: PatternRequest = serde_json::from_slice(&body)
        .map_err(|e| AppError::InvalidRequest(format!("invalid JSON: {e}")))?;

    let mut handle = app.sim.lock().await;

    if handle.status == SimulationStatus::Running {
        return Err(AppError::InvalidStateTransition {
            expected: Some("idle or paused".into()),
            current: "running".into(),
        });
    }

    let (w, h) = (handle.sim.world.width, handle.sim.world.height);
    let clipped = validate_request(&req, w, h)?;
    let points = generate_pattern_seeded(clipped, &req.params, req.seed);

    if points.is_empty() {
        return Ok(Json(ApplyResponse {
            protocol_version: PROTOCOL_VERSION,
            stats: PaintStats::default(),
            dirty_rect: DirtyRect::empty(),
            world_static_changed: false,
        }));
    }

    // Apply pattern as barriers via Simulation::apply_paint.
    // brush_half_extent=0 since pattern cells are already at final positions.
    let stats = handle.sim.apply_paint(PaintTool::Barrier, 0, &points);
    let static_changed = world_static_changed(PaintTool::Barrier, &stats);

    // Construct dirty rect directly from the bounds (no need to iterate points).
    let dirty_rect = DirtyRect {
        x: clipped.x,
        y: clipped.y,
        width: clipped.width,
        height: clipped.height,
    };

    let frame = build_ws_frame(&handle);
    drop(handle);
    app.publish_ws_frame_update(frame, Some(dirty_rect), static_changed);

    Ok(Json(ApplyResponse {
        protocol_version: PROTOCOL_VERSION,
        stats,
        dirty_rect,
        world_static_changed: static_changed,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_bitmap_single_cell() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 8,
            height: 1,
        };
        // Cell at (3, 0) = bit index 3, MSB-first = 0b00010000 = 0x10
        let points = vec![PaintPoint { x: 3, y: 0 }];
        let encoded = encode_bitmap(&bounds, &points);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], 0b0001_0000);
    }

    #[test]
    fn encode_bitmap_multiple_cells() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 4,
            height: 2,
        };
        // 8 total bits = 1 byte
        // (0,0) = bit 0, (1,0) = bit 1, (0,1) = bit 4
        let points = vec![
            PaintPoint { x: 0, y: 0 },
            PaintPoint { x: 1, y: 0 },
            PaintPoint { x: 0, y: 1 },
        ];
        let encoded = encode_bitmap(&bounds, &points);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        // bit 0 (MSB) + bit 1 + bit 4 = 0b11001000
        assert_eq!(decoded[0], 0b1100_1000);
    }

    #[test]
    fn encode_bitmap_with_offset_bounds() {
        let bounds = PatternBounds {
            x: 10,
            y: 20,
            width: 4,
            height: 1,
        };
        // Cell at (12, 20) = column 2 within bounds = bit 2
        let points = vec![PaintPoint { x: 12, y: 20 }];
        let encoded = encode_bitmap(&bounds, &points);
        let decoded = BASE64.decode(&encoded).unwrap();
        // bit 2 MSB-first = 0b00100000 = 0x20
        assert_eq!(decoded[0], 0b0010_0000);
    }

    #[test]
    fn encode_bitmap_empty_points() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        };
        let encoded = encode_bitmap(&bounds, &[]);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 8); // 64 bits = 8 bytes
        assert!(decoded.iter().all(|&b| b == 0));
    }
}
