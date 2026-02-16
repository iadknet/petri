use v2_server::api::{
    ActionCounts, PaintAction, PaintRequest, PaintStrokeTool, SimulationApi, StartupPopulation,
    StartupRequest, StartupRuntime, StartupWorld,
};
use std::collections::HashSet;

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 101,
        world: StartupWorld {
            width: 24,
            height: 16,
            wrap: true,
            sensor_radius: 4,
        },
        population: StartupPopulation {
            initial_creatures: 32,
            max_creatures: 64,
        },
        runtime: StartupRuntime {
            ticks_per_second: 30,
            max_tick_budget_ms: 16,
        },
    }
}

fn all_zero(counts: &ActionCounts) -> bool {
    counts.r#move == 0
        && counts.eat == 0
        && counts.reproduce == 0
        && counts.inventory_pickup == 0
        && counts.inventory_put == 0
        && counts.noop == 0
}

#[test]
fn running_ticks_mutate_world_state_beyond_tick_counter() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let before = api.frame();
    let before_creatures = before
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y, creature.energy))
        .collect::<Vec<_>>();

    api.start().expect("start");
    for _ in 0..5 {
        assert!(api.tick_running(), "running tick should advance");
    }

    let status = api.status();
    let after = api.frame();

    assert_eq!(status.tick, 5);
    assert!(
        after.food.len() > before.food.len(),
        "food growth should mutate frame state while ticking"
    );
    let any_creature_changed = after.creatures.iter().any(|creature| {
        before_creatures
            .iter()
            .find(|(id, _, _, _)| *id == creature.id)
            .is_none_or(|(_, x, y, energy)| {
                creature.x != *x
                    || creature.y != *y
                    || (creature.energy - *energy).abs() > f32::EPSILON
            })
    });
    assert!(
        any_creature_changed,
        "running ticks should mutate creature positions and/or energy"
    );
}

#[test]
fn action_counts_reflect_applied_creature_tick_actions() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    api.start().expect("start");

    for _ in 0..3 {
        assert!(api.tick_running());
    }

    let status = api.status();
    assert!(
        !all_zero(&status.last_action_counts),
        "running ticks should report non-zero action counts from creature behavior"
    );
}

#[test]
fn frame_food_density_reports_multiple_levels_during_runtime() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    api.start().expect("start");

    for _ in 0..12 {
        assert!(api.tick_running());
    }

    let frame = api.frame();
    let unique = frame.food.iter().map(|cell| cell.density).collect::<HashSet<_>>();
    assert!(
        unique.len() > 1,
        "food density should include more than one level, got {:?}",
        unique
    );
}

#[test]
fn running_ticks_can_produce_births_in_live_server_flow() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    let frame = api.frame();
    let points = frame
        .creatures
        .iter()
        .map(|creature| v2_server::api::PaintPoint {
            x: creature.x,
            y: creature.y,
        })
        .collect::<Vec<_>>();

    api.paint(PaintRequest {
        action: PaintAction::Stroke,
        tool: Some(PaintStrokeTool::Food),
        brush_half_extent: Some(0),
        points,
    })
    .expect("paint food on creature cells");

    api.start().expect("start");
    assert!(api.tick_running());
    let first_tick_status = api.status();
    assert!(
        first_tick_status.last_action_counts.reproduce > 0,
        "first running tick should include reproduce actions in resource-rich setup"
    );
    for _ in 0..7 {
        assert!(api.tick_running());
    }

    let status = api.status();
    assert!(
        status.births_last_window > 0,
        "running ticks should report births after resource-rich startup"
    );
}
