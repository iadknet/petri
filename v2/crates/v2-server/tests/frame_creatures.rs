use v2_server::api::{
    SimulationApi, StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 19,
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

#[test]
fn frame_creatures_keep_ids_and_positions_across_no_tick_reads() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());

    let first = api
        .frame()
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();

    let second = api
        .frame()
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();

    assert_eq!(first, second, "no-tick frame reads should be stable");

    let startup_status = api.status();
    assert_eq!(startup_status.tick, 0);

    api.start().expect("idle -> running");
    api.pause().expect("running -> paused");

    let third = api
        .frame()
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();

    assert_eq!(first, third, "lifecycle transitions without ticks must not reshuffle creatures");
}

#[test]
fn frame_creatures_are_deterministic_for_same_startup_seed() {
    let request = startup_request();
    let mut first_api = SimulationApi::new();
    first_api.startup(request.clone());
    let first = first_api
        .frame()
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();

    let mut second_api = SimulationApi::new();
    second_api.startup(request);
    let second = second_api
        .frame()
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();

    assert_eq!(first, second);
}
