use v2_server::api::{
    HealthPayload, PaintAction, PaintPoint, PaintRequest, PaintStrokeTool, SimulationApi,
    StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};
use v2_server::ws::{validate_event_payload_shape, ws_events_for_tick};

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 7,
        world: StartupWorld {
            width: 8,
            height: 8,
            wrap: true,
            sensor_radius: 1,
        },
        population: StartupPopulation {
            initial_creatures: 3,
            max_creatures: 8,
        },
        runtime: StartupRuntime {
            ticks_per_second: 30,
            max_tick_budget_ms: 20,
        },
    }
}

#[test]
fn ws_order_is_status_then_frame_then_health_for_same_tick() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    api.paint(PaintRequest {
        action: PaintAction::Stroke,
        tool: Some(PaintStrokeTool::Barrier),
        brush_half_extent: Some(0),
        points: vec![PaintPoint { x: 1, y: 1 }],
    })
    .expect("paint succeeds");

    let status = api.status();
    let frame = api.frame();
    let health = HealthPayload {
        population: status.population,
        genome_node_count_p50: 1,
        genome_node_count_p90: 1,
        mean_energy: status.mean_energy,
    };

    let events = ws_events_for_tick(&status, &frame, Some(&health));
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event, "status");
    assert_eq!(events[1].event, "frame");
    assert_eq!(events[2].event, "health");
    assert_eq!(events[0].tick, events[1].tick);
    assert_eq!(events[1].tick, events[2].tick);
}

#[test]
fn event_payload_mismatch_is_rejected() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let status = api.status();
    let frame = api.frame();
    let health = HealthPayload {
        population: status.population,
        genome_node_count_p50: 1,
        genome_node_count_p90: 1,
        mean_energy: status.mean_energy,
    };
    let events = ws_events_for_tick(&status, &frame, Some(&health));

    assert!(validate_event_payload_shape(&events[0]).is_ok());
    assert!(validate_event_payload_shape(&events[1]).is_ok());
    assert!(validate_event_payload_shape(&events[2]).is_ok());

    let mut corrupted = events[0].clone();
    corrupted.event = "frame".to_string();
    assert!(validate_event_payload_shape(&corrupted).is_err());
}
