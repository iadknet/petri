use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use v3_core::config::SimulationConfig;
use v3_core::simulation::{seed_simulation, Simulation};

use crate::query::cache::DirtyRect;
use crate::query::projection::ProjectionStore;
use crate::transport::session::ProjectionNotice;
use crate::transport::session_registry::SessionRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationStatus {
    Idle,
    Running,
    Paused,
}

pub struct SimHandle {
    pub sim: Simulation,
    pub status: SimulationStatus,
    pub active_trace: Option<v3_core::runtime::trace::recording::ActiveTrace>,
}

impl SimHandle {
    pub fn new_default() -> Self {
        let config = SimulationConfig::default();
        let sim = seed_simulation(config, 0);
        Self {
            sim,
            status: SimulationStatus::Idle,
            active_trace: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LastTickActions {
    #[serde(rename = "move")]
    pub move_count: u32,
    pub eat: u32,
    pub reproduce: u32,
    pub noop: u32,
    pub steal: u32,
    pub predation_kills: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredationEventSnapshot {
    pub attacker_x: u16,
    pub attacker_y: u16,
    pub victim_x: u16,
    pub victim_y: u16,
    pub energy_stolen: f32,
    pub killed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusPayload {
    pub state: SimulationStatus,
    pub population: usize,
    pub mean_energy: f32,
    pub last_tick_actions: LastTickActions,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub predation_actions_attempted_total: u64,
    pub predation_actions_transferred_total: u64,
    pub predation_actions_rejected_total: u64,
    pub predation_kills_total: u64,
    pub last_tick_compute_total_mean: f32,
    pub last_tick_compute_total_min: f32,
    pub last_tick_compute_total_max: f32,
    pub last_tick_compute_vm_mean: f32,
    pub last_tick_compute_graph_mean: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub energy: f32,
    pub generation: u64,
    pub phenotype_rgb: [u8; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodCell {
    pub x: u16,
    pub y: u16,
    pub density: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BarrierCell {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FramePayload {
    pub width: u16,
    pub height: u16,
    pub creatures: Vec<CreatureSnapshot>,
    pub food: Vec<FoodCell>,
    pub barriers: Vec<BarrierCell>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthPayload {
    pub population: usize,
    pub mean_energy: f32,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
    pub mutation_events_attempted_total_by_domain: HashMap<String, u64>,
    pub mutation_events_applied_total_by_domain: HashMap<String, u64>,
    pub mutation_events_attempted_total_by_operator: HashMap<String, u64>,
    pub mutation_events_applied_total_by_operator: HashMap<String, u64>,
    pub mutation_events_skipped_total_by_operator: HashMap<String, u64>,
    pub mutation_operator_funnel_total_by_operator: HashMap<String, MutationOperatorFunnelPayload>,
    pub mutation_skip_reasons_total_by_operator: HashMap<String, HashMap<String, u64>>,
    pub mutation_events_applied_total_semantic_noop: u64,
    pub mutation_events_applied_total_semantic_change: u64,
    pub mutation_target_reachability_total: MutationTargetReachabilityTotalPayload,
    pub mutation_value_totals_by_operator: HashMap<String, MutationOperatorValueTotalsPayload>,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub reproduction_actions_rejected_total_by_reason: HashMap<String, u64>,
    pub reproduction_actions_rejected_invalid_target_total_by_cause: HashMap<String, u64>,
    pub reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
        HashMap<String, u64>,
    pub mutation_events_skipped_total_by_reason: HashMap<String, u64>,
    pub move_actions_blocked_total_by_cause: HashMap<String, u64>,
    pub move_actions_blocked_avoidable_total_by_reader_state: HashMap<String, u64>,
    pub move_attempts_with_barrier_neighbor_total_by_reader_state: HashMap<String, u64>,
    pub move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: HashMap<String, u64>,
    pub reproduction_attempts_with_barrier_neighbor_total_by_reader_state: HashMap<String, u64>,
    pub reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state:
        HashMap<String, u64>,
    pub predation_actions_attempted_total: u64,
    pub predation_actions_transferred_total: u64,
    pub predation_actions_rejected_total: u64,
    pub predation_kills_total: u64,
    pub predation_actions_by_result: HashMap<String, u64>,
    pub genome_complexity_mean: f32,
    pub genome_complexity_min: u32,
    pub genome_complexity_max: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MutationTargetReachabilityTotalPayload {
    pub reachable: u64,
    pub unreachable: u64,
    pub not_applicable: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MutationOperatorFunnelPayload {
    pub attempted: u64,
    pub applicable: u64,
    pub structurally_valid: u64,
    pub applied: u64,
    pub semantic_change: u64,
    pub skipped: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MutationOperatorValueTotalsPayload {
    pub carriers_observed_total: u64,
    pub survival_ticks_sum: u64,
    pub offspring_spawned_sum: u64,
    pub final_energy_sum: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsFrame {
    pub tick: u64,
    pub status: StatusPayload,
    pub frame: FramePayload,
    pub health: HealthPayload,
    pub predation_events: Vec<PredationEventSnapshot>,
}

#[derive(Clone)]
pub struct AppState {
    pub sim: Arc<Mutex<SimHandle>>,
    pub projection: Arc<RwLock<ProjectionStore>>,
    pub perf: Arc<RwLock<TransportPerfSnapshot>>,
    pub sessions: Arc<RwLock<SessionRegistry>>,
    pub ws_tx: broadcast::Sender<ProjectionNotice>,
}

#[derive(Clone, Debug, Default)]
pub struct TransportPerfSnapshot {
    pub projection_publish_ms: f64,
    pub ws_frame_publish_ms: f64,
}

impl AppState {
    pub fn new() -> Self {
        let handle = SimHandle::new_default();
        let (ws_tx, _) = broadcast::channel(16);
        Self {
            projection: Arc::new(RwLock::new(ProjectionStore::from_handle(&handle))),
            perf: Arc::new(RwLock::new(TransportPerfSnapshot::default())),
            sessions: Arc::new(RwLock::new(SessionRegistry::default())),
            sim: Arc::new(Mutex::new(handle)),
            ws_tx,
        }
    }

    pub fn publish_ws_frame(&self, frame: WsFrame) {
        self.publish_ws_frame_update(frame, None, false);
    }

    pub fn publish_ws_frame_update(
        &self,
        frame: WsFrame,
        dirty_rect: Option<DirtyRect>,
        world_static_changed: bool,
    ) {
        let started = Instant::now();
        let snapshot = self
            .projection
            .write()
            .expect("projection lock poisoned")
            .publish_ws_frame(frame);
        let projection_publish_ms = started.elapsed().as_secs_f64() * 1_000.0;
        // Perf timing is diagnostic telemetry only. Readers may briefly observe the
        // new projection revision with the prior wall-clock timing until this write
        // follows, which is acceptable for non-authoritative transport metrics.
        let mut perf = self.perf.write().expect("perf lock poisoned");
        perf.projection_publish_ms = projection_publish_ms;
        perf.ws_frame_publish_ms = started.elapsed().as_secs_f64() * 1_000.0;
        let _ = self.ws_tx.send(ProjectionNotice {
            projection_revision: snapshot.projection_revision,
            dirty_rect,
            world_static_changed,
        });
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
