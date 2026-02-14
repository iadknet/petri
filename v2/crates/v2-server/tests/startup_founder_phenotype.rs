use v2_core::phenotype::FOUNDER_PHENOTYPE_RGB;
use v2_server::api::{
    SimulationApi, StartupPopulation, StartupRequest, StartupRuntime, StartupWorld,
};

fn startup_request() -> StartupRequest {
    StartupRequest {
        seed: 5,
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
fn startup_seeded_creatures_share_founder_phenotype() {
    let mut api = SimulationApi::new();
    api.startup(startup_request());

    let frame = api.frame();
    assert!(
        !frame.creatures.is_empty(),
        "startup should produce at least one creature snapshot"
    );

    let founder_rgb = frame.creatures[0].phenotype_rgb;
    assert!(
        frame
            .creatures
            .iter()
            .all(|creature| creature.phenotype_rgb == founder_rgb),
        "all startup-seeded creatures should share one founder phenotype"
    );
}

#[test]
fn startup_founder_phenotype_is_deterministic_and_matches_baseline() {
    let mut request = startup_request();
    request.seed = 5;
    let mut first_api = SimulationApi::new();
    first_api.startup(request.clone());
    let first = first_api.frame();
    let first_rgb = first
        .creatures
        .first()
        .expect("startup should seed at least one creature")
        .phenotype_rgb;

    request.seed = 99;
    let mut second_api = SimulationApi::new();
    second_api.startup(request);
    let second = second_api.frame();
    let second_rgb = second
        .creatures
        .first()
        .expect("startup should seed at least one creature")
        .phenotype_rgb;

    assert_eq!(first_rgb, FOUNDER_PHENOTYPE_RGB);
    assert_eq!(second_rgb, FOUNDER_PHENOTYPE_RGB);
    assert_eq!(first_rgb, second_rgb);
}

#[test]
fn startup_keeps_state_idle_after_seeding_founders() {
    let mut api = SimulationApi::new();
    let startup = api.startup(startup_request());
    assert_eq!(startup.state, "idle");
    assert_eq!(api.status().state, "idle");
}
