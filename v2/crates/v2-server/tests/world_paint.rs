use v2_server::api::{
    PaintAction, PaintPoint, PaintRequest, PaintStrokeTool, ProtocolErrorCode, SimulationApi,
    StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 99,
        world: StartupWorld {
            width: 12,
            height: 10,
            wrap: false,
            sensor_radius: 2,
        },
        population: StartupPopulation {
            initial_creatures: 5,
            max_creatures: 20,
        },
        runtime: StartupRuntime {
            ticks_per_second: 15,
            max_tick_budget_ms: 30,
        },
    }
}

#[test]
fn stroke_paint_mutates_world_and_reports_touched_cells() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let response = api
        .paint(PaintRequest {
            action: PaintAction::Stroke,
            tool: Some(PaintStrokeTool::Food),
            brush_half_extent: Some(1),
            points: vec![PaintPoint { x: 5, y: 5 }],
        })
        .expect("stroke should succeed");

    assert_eq!(response.protocol_version, "v2alpha1");
    assert!(response.paint_result.touched_cells > 0);

    let frame = api.frame();
    assert!(!frame.food.is_empty());
}

#[test]
fn clear_all_clears_food_and_barriers() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    api.paint(PaintRequest {
        action: PaintAction::Stroke,
        tool: Some(PaintStrokeTool::Food),
        brush_half_extent: Some(0),
        points: vec![PaintPoint { x: 2, y: 2 }],
    })
    .expect("food");
    api.paint(PaintRequest {
        action: PaintAction::Stroke,
        tool: Some(PaintStrokeTool::Barrier),
        brush_half_extent: Some(0),
        points: vec![PaintPoint { x: 3, y: 3 }],
    })
    .expect("barrier");

    let before = api.frame();
    assert!(!before.food.is_empty());
    assert!(!before.barriers.is_empty());

    let clear = api
        .paint(PaintRequest {
            action: PaintAction::ClearAll,
            tool: None,
            brush_half_extent: None,
            points: Vec::new(),
        })
        .expect("clear all");
    assert!(clear.paint_result.touched_cells >= 2);

    let after = api.frame();
    assert!(after.food.is_empty());
    assert!(after.barriers.is_empty());
}

#[test]
fn paint_is_rejected_while_running() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    api.start().expect("start");

    let err = api
        .paint(PaintRequest {
            action: PaintAction::Stroke,
            tool: Some(PaintStrokeTool::EraseBarrier),
            brush_half_extent: Some(0),
            points: vec![PaintPoint { x: 1, y: 1 }],
        })
        .expect_err("paint should fail while running");

    assert_eq!(err.status_code(), 409);
    assert_eq!(err.code(), ProtocolErrorCode::InvalidStateTransition);
}
