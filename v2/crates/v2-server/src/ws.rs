use crate::PROTOCOL_VERSION;
use crate::api::{FrameResponse, HealthPayload, StatusResponse};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum WsPayload {
    Status(StatusResponse),
    Frame(FrameResponse),
    Health(HealthPayload),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WsEventEnvelope {
    pub protocol_version: String,
    pub event: String,
    pub tick: u64,
    pub payload: WsPayload,
}

#[must_use]
pub fn ws_events_for_tick(
    status: &StatusResponse,
    frame: &FrameResponse,
    health: Option<&HealthPayload>,
) -> Vec<WsEventEnvelope> {
    let mut events = vec![
        WsEventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            event: "status".to_string(),
            tick: status.tick,
            payload: WsPayload::Status(status.clone()),
        },
        WsEventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            event: "frame".to_string(),
            tick: frame.tick,
            payload: WsPayload::Frame(frame.clone()),
        },
    ];

    if let Some(health) = health {
        events.push(WsEventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            event: "health".to_string(),
            tick: status.tick,
            payload: WsPayload::Health(health.clone()),
        });
    }
    events
}

pub fn validate_event_payload_shape(event: &WsEventEnvelope) -> Result<(), String> {
    let matches = matches!(
        (&*event.event, &event.payload),
        ("status", WsPayload::Status(_))
            | ("frame", WsPayload::Frame(_))
            | ("health", WsPayload::Health(_))
    );

    if !matches {
        return Err("event/payload mismatch".to_string());
    }
    Ok(())
}
