use criterion::{black_box, criterion_group, criterion_main, Criterion};
use v3_core::config::SimulationConfig;
use v3_core::simulation::seeding::seed_simulation;
use v3_core::simulation::tick::{run_phase_0, run_tick};

fn bench_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 200;
    cfg.world.height = 200;
    cfg.population.initial_creatures = 200;
    cfg
}

fn bench_phase_0_only(c: &mut Criterion) {
    c.bench_function("phase_0_only_100_ticks", |b| {
        b.iter_with_setup(
            || seed_simulation(bench_config(), 42),
            |mut sim| {
                for _ in 0..100 {
                    run_phase_0(black_box(&mut sim));
                }
            },
        );
    });
}

fn bench_full_tick(c: &mut Criterion) {
    c.bench_function("full_tick_100_ticks", |b| {
        b.iter_with_setup(
            || seed_simulation(bench_config(), 42),
            |mut sim| {
                for _ in 0..100 {
                    run_tick(black_box(&mut sim), &mut None);
                }
            },
        );
    });
}

fn bench_mesh_execution_only(c: &mut Criterion) {
    use v3_core::runtime::mesh::execute_creature_mesh;
    use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use v3_core::sensors::static_inputs::assemble_static_inputs;

    c.bench_function("mesh_execution_200_creatures", |b| {
        b.iter_with_setup(
            || seed_simulation(bench_config(), 42),
            |mut sim| {
                let config = sim.config.runtime.clone();
                let ids: Vec<_> = sim.creatures.keys().collect();
                for id in ids {
                    let local = assemble_static_inputs(&sim.world, &sim.creatures[id]);
                    let ss = SensorSnapshot {
                        local,
                        perception: PerceptionSnapshot::zero(),
                    };
                    let creature = sim.creatures.get_mut(id).unwrap();
                    let _ = black_box(execute_creature_mesh(
                        &creature.genome,
                        &ss,
                        &mut creature.energy,
                        &mut creature.memory,
                        &mut creature.graph_runtime,
                        &config,
                    ));
                }
            },
        );
    });
}

fn bench_full_tick_large_population(c: &mut Criterion) {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 400;
    cfg.world.height = 400;
    cfg.population.initial_creatures = 2000;

    c.bench_function("full_tick_2000_creatures_50_ticks", |b| {
        b.iter_with_setup(
            || seed_simulation(cfg.clone(), 42),
            |mut sim| {
                for _ in 0..50 {
                    run_tick(black_box(&mut sim), &mut None);
                }
            },
        );
    });
}

criterion_group!(
    benches,
    bench_phase_0_only,
    bench_full_tick,
    bench_mesh_execution_only,
    bench_full_tick_large_population
);
criterion_main!(benches);
