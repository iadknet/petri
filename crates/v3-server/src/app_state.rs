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

    /// Normalizes `config` before it is stored or seeded, so a server can never
    /// start from an un-normalized config. `PATCH /v3/simulation/config`
    /// rejects any patch that normalization would rewrite; without this, an
    /// un-normalized startup config makes every runtime patch fail, including
    /// the empty one (2026-09-07 config panel apply audit).
    pub fn from_config(config: SimulationConfig, seed: u64) -> Self {
        let mut config = config;
        config.normalize();
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
            #[cfg(feature = "telemetry")]
            telemetry: crate::telemetry::ServerTelemetry::default(),
        }
    }

    /// [`AppState::from_config`] with run telemetry; emits `run.started` for
    /// the initial run.
    #[cfg(feature = "telemetry")]
    pub fn from_config_with_telemetry(
        config: SimulationConfig,
        seed: u64,
        telemetry: v3_telemetry::Telemetry,
    ) -> Self {
        let mut state = Self::from_config(config, seed);
        state.telemetry = crate::telemetry::ServerTelemetry::new(telemetry);
        state.telemetry.begin(&state.startup_defaults, seed);
        state
    }

    /// Stops the simulation, ends the current run as `shutdown` at the tick it
    /// stopped on and flushes, abandoning what is still pending after the flush
    /// bound. Blocks on a worker thread, not on the runtime.
    ///
    /// A running loop exits at its next status check, since the status leaves
    /// `Running`; the process is exiting, so no `run.state` is emitted for it.
    /// The simulation lock stays held through the flush on purpose: it keeps
    /// any in-flight lifecycle handler from ticking or restarting the loop
    /// before the process exits.
    #[cfg(feature = "telemetry")]
    pub async fn shutdown_telemetry(&self) {
        use crate::state::SimulationStatus;
        let mut handle = self.sim.lock().await;
        if handle.status == SimulationStatus::Running {
            handle.status = SimulationStatus::Paused;
        }
        let (sim, samples) = handle.telemetry_parts();
        self.telemetry.end_for_shutdown(sim, samples);
        let telemetry = self.telemetry.clone();
        let _ = tokio::task::spawn_blocking(move || telemetry.flush()).await;
        drop(handle);
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
