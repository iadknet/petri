//! Shared server application-state construction and projection publication.

use std::sync::{Arc, RwLock};
use std::time::Instant;

use tokio::sync::{broadcast, Mutex};
use v3_core::config::SimulationConfig;

use crate::query::cache::DirtyRect;
use crate::query::projection::ProjectionStore;
use crate::state::{SimHandle, TransportPerfSnapshot, WsFrame};
use crate::transport::session::ProjectionNotice;
use crate::transport::session_registry::SessionRegistry;

use crate::state::AppState;

impl AppState {
    pub fn new() -> Self {
        Self::from_config(SimulationConfig::default(), 0)
    }

    pub fn from_config(config: SimulationConfig, seed: u64) -> Self {
        let startup_defaults = Arc::new(config.clone());
        let handle = SimHandle::new(config, seed);
        let (ws_tx, _) = broadcast::channel(16);
        Self {
            projection: Arc::new(RwLock::new(ProjectionStore::from_handle(&handle))),
            perf: Arc::new(RwLock::new(TransportPerfSnapshot::default())),
            sessions: Arc::new(RwLock::new(SessionRegistry::default())),
            sim: Arc::new(Mutex::new(handle)),
            ws_tx,
            startup_defaults,
        }
    }

    pub fn publish_ws_frame(&self, frame: WsFrame) {
        self.publish_ws_frame_with_invalidation(frame, None, false);
    }

    pub(crate) fn publish_startup_ws_frame(&self, frame: WsFrame) {
        self.publish_ws_frame_with_invalidation(frame, None, true);
    }

    pub(crate) fn publish_ws_frame_update(&self, frame: WsFrame, dirty_rect: Option<DirtyRect>) {
        self.publish_ws_frame_with_invalidation(frame, dirty_rect, false);
    }

    fn publish_ws_frame_with_invalidation(
        &self,
        frame: WsFrame,
        dirty_rect: Option<DirtyRect>,
        force_world_static_invalidation: bool,
    ) {
        let started = Instant::now();
        let publication = {
            self.projection
                .write()
                .expect("projection lock poisoned")
                .publish_ws_frame(frame, force_world_static_invalidation)
        };
        let projection_publish_ms = started.elapsed().as_secs_f64() * 1_000.0;
        // Perf timing is diagnostic telemetry only. Readers may briefly observe the
        // new projection revision with the prior wall-clock timing until this write
        // follows, which is acceptable for non-authoritative transport metrics.
        let mut perf = self.perf.write().expect("perf lock poisoned");
        perf.projection_publish_ms = projection_publish_ms;
        perf.ws_frame_publish_ms = started.elapsed().as_secs_f64() * 1_000.0;
        let _ = self.ws_tx.send(ProjectionNotice {
            projection_revision: publication.projection_revision,
            dirty_rect,
            world_static_changed: publication.world_static_changed,
        });
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
