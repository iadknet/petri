use v2_server::api::{
    SimulationApi, StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};

fn non_viable_request() -> StartupRequest {
    StartupRequest {
        seed: 123,
        world: StartupWorld {
            width: 0,
            height: 0,
            wrap: true,
            sensor_radius: 4,
        },
        population: StartupPopulation {
            initial_creatures: 0,
            max_creatures: 0,
        },
        runtime: StartupRuntime {
            ticks_per_second: 30,
            max_tick_budget_ms: 16,
        },
    }
}

#[test]
fn startup_sanitizes_non_viable_seed_request() {
    let mut api = SimulationApi::new();
    api.startup(non_viable_request());

    let status = api.status();
    assert!(
        status.population >= 1,
        "startup should guarantee at least one viable seed creature"
    );

    let frame = api.frame();
    assert_eq!(frame.width, 1);
    assert_eq!(frame.height, 1);
    assert!(
        !frame.creatures.is_empty(),
        "startup should emit a viable creature snapshot"
    );
}

#[test]
fn startup_viability_is_deterministic_for_same_request() {
    let request = non_viable_request();
    let mut first_api = SimulationApi::new();
    first_api.startup(request.clone());
    let first = first_api.frame();

    let mut second_api = SimulationApi::new();
    second_api.startup(request);
    let second = second_api.frame();

    assert_eq!(first.width, second.width);
    assert_eq!(first.height, second.height);
    assert_eq!(first.creatures.len(), second.creatures.len());
    assert_eq!(
        first
            .creatures
            .first()
            .map(|creature| creature.phenotype_rgb),
        second
            .creatures
            .first()
            .map(|creature| creature.phenotype_rgb)
    );
}

#[test]
fn startup_non_viable_inputs_normalize_to_deterministic_viable_baseline() {
    let mut api = SimulationApi::new();
    api.startup(non_viable_request());

    let frame = api.frame();
    assert_eq!(frame.width, 1);
    assert_eq!(frame.height, 1);
    assert_eq!(frame.tick, 0);

    assert_eq!(frame.creatures.len(), 1);
    let founder = &frame.creatures[0];
    assert_eq!(founder.id, 1);
    assert_eq!(founder.x, 0);
    assert_eq!(founder.y, 0);

    assert_eq!(frame.food.len(), 1);
    assert_eq!(frame.food[0].x, 0);
    assert_eq!(frame.food[0].y, 0);
}
