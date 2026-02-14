use v2_server::api::{
    ProtocolErrorCode, SimulationApi, SimulationError, StartupPopulation, StartupRequest,
    StartupRuntime, StartupWorld,
};

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 42,
        world: StartupWorld {
            width: 32,
            height: 24,
            wrap: true,
            sensor_radius: 4,
        },
        population: StartupPopulation {
            initial_creatures: 100,
            max_creatures: 250,
        },
        runtime: StartupRuntime {
            ticks_per_second: 30,
            max_tick_budget_ms: 16,
        },
    }
}

#[test]
fn startup_resets_state_and_tick() {
    let mut api = SimulationApi::new();
    let startup = api.startup(startup_request());
    assert_eq!(startup.state, "idle");
    assert_eq!(startup.tick, 0);
    assert_eq!(startup.protocol_version, "v2alpha1");

    let running = api.start().expect("idle -> running");
    assert_eq!(running.state, "running");
    let paused = api.pause().expect("running -> paused");
    assert_eq!(paused.state, "paused");
    let stepped = api.step(Some(5)).expect("paused step");
    assert_eq!(stepped.tick, 5);

    let reset = api.startup(startup_request());
    assert_eq!(reset.state, "idle");
    assert_eq!(reset.tick, 0);
}

#[test]
fn start_and_pause_are_idempotent() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());

    let first_start = api.start().expect("start");
    let second_start = api.start().expect("idempotent start");
    assert_eq!(first_start.state, "running");
    assert_eq!(second_start.state, "running");
    assert_eq!(first_start.tick, second_start.tick);

    let first_pause = api.pause().expect("pause");
    let second_pause = api.pause().expect("idempotent pause");
    assert_eq!(first_pause.state, "paused");
    assert_eq!(second_pause.state, "paused");
}

#[test]
fn pause_from_idle_is_invalid_transition() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let error = api.pause().expect_err("idle pause should fail");
    assert_eq!(error.status_code(), 409);
    assert_eq!(error.code(), ProtocolErrorCode::InvalidStateTransition);
}

#[test]
fn step_requires_paused_and_valid_range() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());

    let idle_error = api.step(None).expect_err("idle step should fail");
    assert_eq!(idle_error.status_code(), 409);
    assert_eq!(idle_error.code(), ProtocolErrorCode::InvalidStateTransition);

    api.start().expect("start");
    let running_error = api.step(Some(1)).expect_err("running step should fail");
    assert_eq!(running_error.status_code(), 409);
    assert_eq!(
        running_error.code(),
        ProtocolErrorCode::InvalidStateTransition
    );

    api.pause().expect("pause");
    let default_step = api.step(None).expect("default steps");
    assert_eq!(default_step.tick, 1);

    let zero = api.step(Some(0)).expect_err("0 is out of range");
    assert_eq!(zero.status_code(), 400);
    assert_eq!(zero.code(), ProtocolErrorCode::InvalidRequest);

    let too_large = api.step(Some(1001)).expect_err("1001 is out of range");
    assert_eq!(too_large.status_code(), 400);
    assert_eq!(too_large.code(), ProtocolErrorCode::InvalidRequest);
}

#[test]
fn protocol_errors_report_expected_and_current_state() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let error = api.pause().expect_err("pause should fail in idle");
    let SimulationError::Protocol(protocol_error) = error;
    assert_eq!(
        protocol_error.details.expected_state.as_deref(),
        Some("running")
    );
    assert_eq!(
        protocol_error.details.current_state.as_deref(),
        Some("idle")
    );
}
