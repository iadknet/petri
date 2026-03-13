//! Public transport protocol types for the v3alpha2 cutover.

use serde::{Deserialize, Serialize};

use crate::query::projection::ProjectionSnapshot;
use crate::state::{
    CreatureSnapshot, HealthPayload, LastTickActions, MutationOperatorFunnelPayload,
    MutationOperatorValueTotalsPayload, MutationTargetReachabilityTotalPayload,
    PredationEventSnapshot, SimulationStatus, TransportPerfSnapshot,
};
pub use crate::types::PROTOCOL_VERSION;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoomTier {
    Overview,
    Detail,
    Inspect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    SubscribeView {
        request_id: u64,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        canvas_width: u16,
        canvas_height: u16,
        zoom_tier: ZoomTier,
    },
    UnsubscribeView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerfPayload {
    pub projection_publish_ms: f64,
    pub ws_frame_publish_ms: f64,
    pub subscriber_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusEventPayload {
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
    pub predation_actions_by_result: std::collections::HashMap<String, u64>,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
    pub mutation_events_attempted_total_by_domain: std::collections::HashMap<String, u64>,
    pub mutation_events_applied_total_by_domain: std::collections::HashMap<String, u64>,
    pub mutation_events_attempted_total_by_operator: std::collections::HashMap<String, u64>,
    pub mutation_events_applied_total_by_operator: std::collections::HashMap<String, u64>,
    pub mutation_events_skipped_total_by_operator: std::collections::HashMap<String, u64>,
    pub mutation_operator_funnel_total_by_operator:
        std::collections::HashMap<String, MutationOperatorFunnelPayload>,
    pub mutation_skip_reasons_total_by_operator:
        std::collections::HashMap<String, std::collections::HashMap<String, u64>>,
    pub mutation_events_applied_total_semantic_noop: u64,
    pub mutation_events_applied_total_semantic_change: u64,
    pub mutation_target_reachability_total: MutationTargetReachabilityTotalPayload,
    pub mutation_value_totals_by_operator:
        std::collections::HashMap<String, MutationOperatorValueTotalsPayload>,
    pub mutation_outcome_summary: MutationOperatorValueTotalsPayload,
    pub move_actions_blocked_total_by_cause: std::collections::HashMap<String, u64>,
    pub move_actions_blocked_avoidable_total_by_reader_state:
        std::collections::HashMap<String, u64>,
    pub move_attempts_with_barrier_neighbor_total_by_reader_state:
        std::collections::HashMap<String, u64>,
    pub move_blocked_barrier_with_barrier_neighbor_total_by_reader_state:
        std::collections::HashMap<String, u64>,
    pub reproduction_attempts_with_barrier_neighbor_total_by_reader_state:
        std::collections::HashMap<String, u64>,
    pub reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state:
        std::collections::HashMap<String, u64>,
    pub reproduction_actions_rejected_invalid_target_total_by_cause:
        std::collections::HashMap<String, u64>,
    pub reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
        std::collections::HashMap<String, u64>,
    pub last_tick_compute_energy_total_mean: f32,
    pub last_tick_compute_energy_total_min: f32,
    pub last_tick_compute_energy_total_max: f32,
    pub last_tick_compute_energy_vm_mean: f32,
    pub last_tick_compute_energy_graph_mean: f32,
    pub perf: PerfPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldStaticPayload {
    pub width: u16,
    pub height: u16,
    pub barrier_mask: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewRectPayload {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewOverviewPayload {
    /// Overview payloads intentionally omit predation events; only detail views
    /// carry exact event locations because overview mode is aggregate-only.
    pub rect: ViewRectPayload,
    pub grid_width: u16,
    pub grid_height: u16,
    pub food_density_u8: Vec<u8>,
    pub creature_count_u16: Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewDetailPayload {
    pub rect: ViewRectPayload,
    pub width: u16,
    pub height: u16,
    pub food_density_u8: Vec<u8>,
    pub creatures: Vec<CreatureSnapshot>,
    pub predation_events: Vec<PredationEventSnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Status {
        protocol_version: String,
        projection_revision: u64,
        world_static_revision: u64,
        tick: u64,
        payload: StatusEventPayload,
    },
    Health {
        protocol_version: String,
        projection_revision: u64,
        world_static_revision: u64,
        tick: u64,
        payload: HealthPayload,
    },
    WorldStatic {
        protocol_version: String,
        projection_revision: u64,
        world_static_revision: u64,
        tick: u64,
        payload: WorldStaticPayload,
    },
    ViewOverview {
        protocol_version: String,
        request_id: u64,
        projection_revision: u64,
        world_static_revision: u64,
        tick: u64,
        payload: ViewOverviewPayload,
    },
    ViewDetail {
        protocol_version: String,
        request_id: u64,
        projection_revision: u64,
        world_static_revision: u64,
        tick: u64,
        payload: ViewDetailPayload,
    },
}

#[must_use]
pub fn build_status_event_payload(
    snapshot: &ProjectionSnapshot,
    perf: &TransportPerfSnapshot,
    subscriber_count: usize,
) -> StatusEventPayload {
    let status = &snapshot.ws_frame.status;
    let health = &snapshot.ws_frame.health;

    StatusEventPayload {
        state: status.state,
        population: status.population,
        mean_energy: status.mean_energy,
        last_tick_actions: status.last_tick_actions.clone(),
        reproduction_actions_attempted_total: status.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: status.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: status.reproduction_actions_rejected_total,
        predation_actions_attempted_total: status.predation_actions_attempted_total,
        predation_actions_transferred_total: status.predation_actions_transferred_total,
        predation_actions_rejected_total: status.predation_actions_rejected_total,
        predation_kills_total: status.predation_kills_total,
        predation_actions_by_result: health.predation_actions_by_result.clone(),
        mutation_events_attempted_total: health.mutation_events_attempted_total,
        mutation_events_applied_total: health.mutation_events_applied_total,
        mutation_events_skipped_total: health.mutation_events_skipped_total,
        mutation_events_attempted_total_by_domain: health
            .mutation_events_attempted_total_by_domain
            .clone(),
        mutation_events_applied_total_by_domain: health
            .mutation_events_applied_total_by_domain
            .clone(),
        mutation_events_attempted_total_by_operator: health
            .mutation_events_attempted_total_by_operator
            .clone(),
        mutation_events_applied_total_by_operator: health
            .mutation_events_applied_total_by_operator
            .clone(),
        mutation_events_skipped_total_by_operator: health
            .mutation_events_skipped_total_by_operator
            .clone(),
        mutation_operator_funnel_total_by_operator: health
            .mutation_operator_funnel_total_by_operator
            .clone(),
        mutation_skip_reasons_total_by_operator: health
            .mutation_skip_reasons_total_by_operator
            .clone(),
        mutation_events_applied_total_semantic_noop: health
            .mutation_events_applied_total_semantic_noop,
        mutation_events_applied_total_semantic_change: health
            .mutation_events_applied_total_semantic_change,
        mutation_target_reachability_total: health.mutation_target_reachability_total.clone(),
        mutation_value_totals_by_operator: health.mutation_value_totals_by_operator.clone(),
        mutation_outcome_summary: health.mutation_outcome_summary.clone(),
        move_actions_blocked_total_by_cause: health.move_actions_blocked_total_by_cause.clone(),
        move_actions_blocked_avoidable_total_by_reader_state: health
            .move_actions_blocked_avoidable_total_by_reader_state
            .clone(),
        move_attempts_with_barrier_neighbor_total_by_reader_state: health
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .clone(),
        move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: health
            .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
            .clone(),
        reproduction_attempts_with_barrier_neighbor_total_by_reader_state: health
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .clone(),
        reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state: health
            .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
            .clone(),
        reproduction_actions_rejected_invalid_target_total_by_cause: health
            .reproduction_actions_rejected_invalid_target_total_by_cause
            .clone(),
        reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state: health
            .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
            .clone(),
        last_tick_compute_energy_total_mean: status.last_tick_compute_total_mean,
        last_tick_compute_energy_total_min: status.last_tick_compute_total_min,
        last_tick_compute_energy_total_max: status.last_tick_compute_total_max,
        last_tick_compute_energy_vm_mean: status.last_tick_compute_vm_mean,
        last_tick_compute_energy_graph_mean: status.last_tick_compute_graph_mean,
        perf: PerfPayload {
            projection_publish_ms: perf.projection_publish_ms,
            ws_frame_publish_ms: perf.ws_frame_publish_ms,
            subscriber_count,
        },
    }
}

#[must_use]
pub fn build_world_static_payload(snapshot: &ProjectionSnapshot) -> WorldStaticPayload {
    WorldStaticPayload {
        width: snapshot.ws_frame.frame.width,
        height: snapshot.ws_frame.frame.height,
        barrier_mask: snapshot.barrier_mask.to_vec(),
    }
}

pub fn decode_client_message_text(text: &str) -> Result<ClientMessage, serde_json::Error> {
    serde_json::from_str(text)
}

pub fn decode_client_message_binary(
    bytes: &[u8],
) -> Result<ClientMessage, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}

pub fn encode_server_message(message: &ServerMessage) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(message)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::state::{
        CreatureSnapshot, LastTickActions, PredationEventSnapshot, SimulationStatus,
    };

    use super::{
        ClientMessage, ServerMessage, ViewDetailPayload, ViewRectPayload, ZoomTier,
        PROTOCOL_VERSION,
    };

    #[test]
    fn subscribe_view_json_decodes_to_typed_message() {
        let value = json!({
            "type": "subscribe_view",
            "request_id": 7u64,
            "x": 1u16,
            "y": 2u16,
            "width": 30u16,
            "height": 40u16,
            "canvas_width": 640u16,
            "canvas_height": 480u16,
            "zoom_tier": "detail"
        });

        let decoded: ClientMessage =
            serde_json::from_value(value).expect("client message should deserialize");

        assert_eq!(
            decoded,
            ClientMessage::SubscribeView {
                request_id: 7,
                x: 1,
                y: 2,
                width: 30,
                height: 40,
                canvas_width: 640,
                canvas_height: 480,
                zoom_tier: ZoomTier::Detail,
            }
        );
    }

    #[test]
    fn server_view_detail_roundtrips_through_msgpack() {
        let message = ServerMessage::ViewDetail {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: 9,
            projection_revision: 11,
            world_static_revision: 3,
            tick: 77,
            payload: ViewDetailPayload {
                rect: ViewRectPayload {
                    x: 1,
                    y: 2,
                    width: 4,
                    height: 5,
                },
                width: 4,
                height: 5,
                food_density_u8: vec![0, 1, 2, 3],
                creatures: vec![CreatureSnapshot {
                    id: 42,
                    x: 3,
                    y: 4,
                    energy: 2.0,
                    generation: 1,
                    phenotype_rgb: [1, 2, 3],
                }],
                predation_events: vec![PredationEventSnapshot {
                    attacker_x: 1,
                    attacker_y: 2,
                    victim_x: 3,
                    victim_y: 4,
                    energy_stolen: 5.0,
                    killed: false,
                }],
            },
        };

        let bytes = rmp_serde::to_vec_named(&message).expect("serialize server message");
        let decoded: ServerMessage =
            rmp_serde::from_slice(&bytes).expect("deserialize server message");

        match decoded {
            ServerMessage::ViewDetail {
                request_id,
                projection_revision,
                world_static_revision,
                tick,
                payload,
                ..
            } => {
                assert_eq!(request_id, 9);
                assert_eq!(projection_revision, 11);
                assert_eq!(world_static_revision, 3);
                assert_eq!(tick, 77);
                assert_eq!(payload.width, 4);
                assert_eq!(payload.creatures.len(), 1);
                assert_eq!(payload.creatures[0].id, 42);
            }
            other => panic!("expected view_detail message, got {other:?}"),
        }
    }

    #[test]
    fn status_payload_keeps_action_shape() {
        let payload = super::StatusEventPayload {
            state: SimulationStatus::Paused,
            population: 1,
            mean_energy: 2.0,
            last_tick_actions: LastTickActions {
                move_count: 1,
                eat: 2,
                reproduce: 3,
                noop: 4,
                steal: 5,
                predation_kills: 6,
            },
            reproduction_actions_attempted_total: 0,
            reproduction_actions_spawned_total: 0,
            reproduction_actions_rejected_total: 0,
            predation_actions_attempted_total: 0,
            predation_actions_transferred_total: 0,
            predation_actions_rejected_total: 0,
            predation_kills_total: 0,
            predation_actions_by_result: Default::default(),
            mutation_events_attempted_total: 0,
            mutation_events_applied_total: 0,
            mutation_events_skipped_total: 0,
            mutation_events_attempted_total_by_domain: Default::default(),
            mutation_events_applied_total_by_domain: Default::default(),
            mutation_events_attempted_total_by_operator: Default::default(),
            mutation_events_applied_total_by_operator: Default::default(),
            mutation_events_skipped_total_by_operator: Default::default(),
            mutation_operator_funnel_total_by_operator: Default::default(),
            mutation_skip_reasons_total_by_operator: Default::default(),
            mutation_events_applied_total_semantic_noop: 0,
            mutation_events_applied_total_semantic_change: 0,
            mutation_target_reachability_total:
                super::MutationTargetReachabilityTotalPayload::default(),
            mutation_value_totals_by_operator: Default::default(),
            mutation_outcome_summary: Default::default(),
            move_actions_blocked_total_by_cause: Default::default(),
            move_actions_blocked_avoidable_total_by_reader_state: Default::default(),
            move_attempts_with_barrier_neighbor_total_by_reader_state: Default::default(),
            move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: Default::default(),
            reproduction_attempts_with_barrier_neighbor_total_by_reader_state: Default::default(),
            reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state:
                Default::default(),
            reproduction_actions_rejected_invalid_target_total_by_cause: Default::default(),
            reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
                Default::default(),
            last_tick_compute_energy_total_mean: 0.0,
            last_tick_compute_energy_total_min: 0.0,
            last_tick_compute_energy_total_max: 0.0,
            last_tick_compute_energy_vm_mean: 0.0,
            last_tick_compute_energy_graph_mean: 0.0,
            perf: super::PerfPayload {
                projection_publish_ms: 0.1,
                ws_frame_publish_ms: 0.2,
                subscriber_count: 3,
            },
        };

        let encoded = serde_json::to_value(payload).expect("serialize status payload");
        assert_eq!(encoded["last_tick_actions"]["move"], 1);
        assert!(encoded.get("last_tick_compute_total_mean").is_none());
    }
}
