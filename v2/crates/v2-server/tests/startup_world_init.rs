use v2_server::api::{
    SimulationApi, StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};

fn startup_request(seed: u64) -> StartupRequest {
    StartupRequest {
        seed,
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
fn startup_seeds_world_with_food_and_creatures() {
    let mut api = SimulationApi::new();
    api.startup(startup_request(5));

    let frame = api.frame();
    assert!(!frame.food.is_empty(), "startup should seed food");
    assert!(!frame.creatures.is_empty(), "startup should seed creatures");
}

#[test]
fn startup_seeding_is_deterministic_for_same_seed() {
    let request = startup_request(5);
    let mut first_api = SimulationApi::new();
    first_api.startup(request.clone());
    let first = first_api.frame();

    let mut second_api = SimulationApi::new();
    second_api.startup(request);
    let second = second_api.frame();

    let first_food = first
        .food
        .iter()
        .map(|cell| (cell.x, cell.y, cell.density))
        .collect::<Vec<_>>();
    let second_food = second
        .food
        .iter()
        .map(|cell| (cell.x, cell.y, cell.density))
        .collect::<Vec<_>>();
    assert_eq!(first_food, second_food);

    let first_creatures = first
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();
    let second_creatures = second
        .creatures
        .iter()
        .map(|creature| (creature.id, creature.x, creature.y))
        .collect::<Vec<_>>();
    assert_eq!(first_creatures, second_creatures);
}

#[test]
fn startup_seeding_varies_for_different_seed() {
    let mut first_api = SimulationApi::new();
    first_api.startup(startup_request(5));
    let first = first_api.frame();

    let mut second_api = SimulationApi::new();
    second_api.startup(startup_request(6));
    let second = second_api.frame();

    let first_food = first
        .food
        .iter()
        .map(|cell| (cell.x, cell.y))
        .collect::<Vec<_>>();
    let second_food = second
        .food
        .iter()
        .map(|cell| (cell.x, cell.y))
        .collect::<Vec<_>>();
    assert_ne!(first_food, second_food);
}
