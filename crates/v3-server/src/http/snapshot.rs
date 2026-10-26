//! Projection-backed bootstrap snapshot handler.

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::state::{AppState, HealthPayload};
use crate::transport::protocol::{
    build_status_event_payload, build_world_static_payload, StatusEventPayload, ViewDetailPayload,
    ViewOverviewPayload, WorldStaticPayload, ZoomTier,
};
use crate::transport::session::ViewSubscription;
use crate::transport::view_assembler::{assemble_detail_payload, assemble_overview_payload};
use crate::types::PROTOCOL_VERSION;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SnapshotQuery {
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub canvas_width: Option<u16>,
    pub canvas_height: Option<u16>,
    pub zoom_tier: Option<ZoomTier>,
}

/// The snapshot's view: the overview or the detail payload, tagged by `kind`.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SnapshotView {
    Overview(ViewOverviewPayload),
    Detail(ViewDetailPayload),
}

/// The response of `GET /v3/simulation/snapshot`.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct SnapshotResponse {
    protocol_version: &'static str,
    projection_revision: u64,
    world_static_revision: u64,
    tick: u64,
    status: StatusEventPayload,
    health: HealthPayload,
    world_static: WorldStaticPayload,
    view: SnapshotView,
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

    let view = match subscription.zoom_tier {
        ZoomTier::Overview => {
            SnapshotView::Overview(assemble_overview_payload(&snapshot, &subscription))
        }
        ZoomTier::Detail | ZoomTier::Inspect => {
            SnapshotView::Detail(assemble_detail_payload(&snapshot, &subscription))
        }
    };

    Json(SnapshotResponse {
        protocol_version: PROTOCOL_VERSION,
        projection_revision: snapshot.projection_revision,
        world_static_revision: snapshot.world_static_revision,
        tick: snapshot.ws_frame.tick,
        status: build_status_event_payload(&snapshot, &perf, app.ws_tx.receiver_count()),
        health: snapshot.ws_frame.health.clone(),
        world_static: build_world_static_payload(&snapshot),
        view,
    })
}
