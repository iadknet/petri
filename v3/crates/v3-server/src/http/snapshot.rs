//! Projection-backed bootstrap snapshot handler.

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::transport::protocol::{
    build_status_event_payload, build_world_static_payload, ZoomTier,
};
use crate::transport::session::ViewSubscription;
use crate::transport::view_assembler::{assemble_detail_payload, assemble_overview_payload};
use crate::types::PROTOCOL_VERSION;

#[derive(Debug, Default, Deserialize)]
pub struct SnapshotQuery {
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub canvas_width: Option<u16>,
    pub canvas_height: Option<u16>,
    pub zoom_tier: Option<ZoomTier>,
}

pub async fn get_snapshot(
    State(app): State<AppState>,
    Query(query): Query<SnapshotQuery>,
) -> impl IntoResponse {
    let snapshot = {
        app.projection
            .read()
            .expect("projection lock poisoned")
            .current()
            .clone()
    };
    let perf = app.perf.read().expect("perf lock poisoned").clone();
    let frame = &snapshot.ws_frame.frame;

    let subscription = ViewSubscription {
        request_id: 0,
        x: query.x.unwrap_or(0),
        y: query.y.unwrap_or(0),
        width: query.width.unwrap_or(frame.width),
        height: query.height.unwrap_or(frame.height),
        canvas_width: query.canvas_width.unwrap_or(frame.width),
        canvas_height: query.canvas_height.unwrap_or(frame.height),
        zoom_tier: query.zoom_tier.unwrap_or(ZoomTier::Overview),
    };

    let mut view = match subscription.zoom_tier {
        ZoomTier::Overview => {
            serde_json::to_value(assemble_overview_payload(&snapshot, &subscription))
                .expect("overview payload must serialize")
        }
        ZoomTier::Detail | ZoomTier::Inspect => {
            serde_json::to_value(assemble_detail_payload(&snapshot, &subscription))
                .expect("detail payload must serialize")
        }
    };
    view.as_object_mut()
        .expect("view payload must serialize to an object")
        .insert(
            "kind".into(),
            serde_json::Value::String(
                match subscription.zoom_tier {
                    ZoomTier::Overview => "overview",
                    ZoomTier::Detail | ZoomTier::Inspect => "detail",
                }
                .to_string(),
            ),
        );

    Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "projection_revision": snapshot.projection_revision,
        "world_static_revision": snapshot.world_static_revision,
        "tick": snapshot.ws_frame.tick,
        "status": build_status_event_payload(&snapshot, &perf, app.ws_tx.receiver_count()),
        "health": snapshot.ws_frame.health,
        "world_static": build_world_static_payload(&snapshot),
        "view": view,
    }))
}
