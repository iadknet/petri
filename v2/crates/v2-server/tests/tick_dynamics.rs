use v2_server::api::{
    ActionCounts, SimulationApi, StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};

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
}

#[test]
fn action_counts_are_not_synthetic_when_no_actions_are_applied() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());
    api.start().expect("start");

    for _ in 0..3 {
        assert!(api.tick_running());
    }

    let status = api.status();
    assert!(
        all_zero(&status.last_action_counts),
        "without applied creature actions, action counts should remain zero"
    );
}
