use v2_server::api::{
    ErrorDetails, ErrorEnvelope, ErrorField, PaintAction, PaintRequest, PaintStrokeTool,
    ProtocolErrorCode, SimulationApi, SimulationError, StartupPopulation, StartupRequest,
    StartupRuntime, StartupWorld,
};

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 11,
        world: StartupWorld {
            width: 20,
            height: 12,
            wrap: false,
            sensor_radius: 2,
        },
        population: StartupPopulation {
            initial_creatures: 10,
            max_creatures: 20,
        },
        runtime: StartupRuntime {
            ticks_per_second: 20,
            max_tick_budget_ms: 12,
        },
    }
}

#[test]
fn success_payloads_include_protocol_version() {
    let mut api = SimulationApi::new();
    let startup = api.startup(startup_request());
    assert_eq!(startup.protocol_version, "v2alpha1");

    let status = api.status();
    assert_eq!(status.protocol_version, "v2alpha1");

    let frame = api.frame();
    assert_eq!(frame.protocol_version, "v2alpha1");

    api.start().expect("start");
    let pause = api.pause().expect("pause");
    assert_eq!(pause.protocol_version, "v2alpha1");

    let step = api.step(Some(2)).expect("step");
    assert_eq!(step.protocol_version, "v2alpha1");
}

#[test]
fn protocol_error_envelope_maps_to_status_code() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let err = api.pause().expect_err("pause in idle");

    assert_eq!(err.status_code(), 409);
    assert_eq!(err.code(), ProtocolErrorCode::InvalidStateTransition);

    let envelope = err.into_envelope();
    assert_eq!(envelope.protocol_version, "v2alpha1");
    assert_eq!(envelope.error.code, "invalid_state_transition");
    assert!(envelope.error.details.expected_state.is_some());
}

#[test]
fn paint_validation_failure_uses_422_validation_rejected() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let err = api
        .paint(PaintRequest {
            action: PaintAction::Stroke,
            tool: Some(PaintStrokeTool::Food),
            brush_half_extent: Some(1),
            points: Vec::new(),
        })
        .expect_err("empty stroke should be rejected");

    assert_eq!(err.status_code(), 422);
    assert_eq!(err.code(), ProtocolErrorCode::ValidationRejected);
}

#[test]
fn paint_validation_error_envelope_includes_endpoint_and_field_reason() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let err = api
        .paint(PaintRequest {
            action: PaintAction::Stroke,
            tool: Some(PaintStrokeTool::Food),
            brush_half_extent: Some(1),
            points: Vec::new(),
        })
        .expect_err("empty stroke should be rejected");

    let envelope = err.into_envelope();
    assert_eq!(envelope.error.code, "validation_rejected");
    assert_eq!(
        envelope.error.details.endpoint.as_deref(),
        Some("/v2/simulation/world/paint")
    );
    assert_eq!(envelope.error.details.field_errors.len(), 1);
    assert_eq!(envelope.error.details.field_errors[0].field, "points");
}

#[test]
fn error_envelope_can_include_field_errors() {
    let envelope = ErrorEnvelope::new(SimulationError::Protocol(Box::new(
        v2_server::api::ProtocolError {
            code: ProtocolErrorCode::InvalidRequest,
            message: "invalid shape".to_string(),
            details: ErrorDetails {
                endpoint: Some("/v2/simulation/world/paint".to_string()),
                field_errors: vec![ErrorField {
                    field: "points".to_string(),
                    reason: "must not be empty".to_string(),
                }],
                expected_state: None,
                current_state: None,
            },
        },
    )));

    assert_eq!(envelope.protocol_version, "v2alpha1");
    assert_eq!(envelope.error.code, "invalid_request");
    assert_eq!(envelope.error.details.field_errors.len(), 1);
}
