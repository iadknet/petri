use criterion::{black_box, criterion_group, criterion_main, Criterion};
use v3_core::config::SimulationConfig;
use v3_core::simulation::seeding::seed_simulation;
use v3_server::handlers::lifecycle::build_ws_frame;
use v3_server::query::cache::build_food_fertility_u8;
use v3_server::state::{SimHandle, SimulationStatus};

fn transport_stress_simulation(seed: u64) -> v3_core::simulation::Simulation {
    let mut config = SimulationConfig::default();
    config.world.width = 320;
    config.world.height = 320;
    config.population.initial_creatures = 1_000;
    config.runtime.perception.vision_radius = 5;
    seed_simulation(config, seed)
}

fn transport_handle(seed: u64) -> SimHandle {
    let sim = transport_stress_simulation(seed);
    let cached_fertility_u8 = build_food_fertility_u8(sim.world.food());
    SimHandle {
        sim,
        status: SimulationStatus::Running,
        active_trace: None,
        cached_fertility_u8,
    }
}

fn bench_monolithic_frame_build(c: &mut Criterion) {
    c.bench_function("monolithic_frame_build", |b| {
        b.iter_with_setup(
            || transport_handle(42),
            |handle| {
                let frame = build_ws_frame(black_box(&handle));
                black_box(frame);
            },
        );
    });
}

fn bench_monolithic_frame_encode(c: &mut Criterion) {
    c.bench_function("monolithic_frame_encode", |b| {
        b.iter_with_setup(
            || build_ws_frame(&transport_handle(42)),
            |frame| {
                let bytes = rmp_serde::to_vec_named(black_box(&frame)).expect("msgpack encode");
                black_box(bytes);
            },
        );
    });
}

criterion_group!(
    transport_benches,
    bench_monolithic_frame_build,
    bench_monolithic_frame_encode
);
criterion_main!(transport_benches);
