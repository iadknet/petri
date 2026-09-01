use criterion::{black_box, criterion_group, criterion_main, Criterion};
use v3_core::config::SimulationConfig;
use v3_core::sensors::perception::{
    genome_uses_extended_perception, PerceptionConfig, PerceptionSnapshot, SensorSnapshot,
};
use v3_core::sensors::reducers::assemble_perception;
use v3_core::sensors::static_inputs::assemble_static_inputs;
use v3_core::sensors::typed_food::assemble_typed_food_local_snapshot;
use v3_core::sensors::visibility::{
    compute_visible_cells_into, get_visibility_table, VisibilityScratch,
};
use v3_core::simulation::seeding::{seed_simulation, seed_simulation_with_perception_mix};
use v3_core::simulation::tick::{run_phase_0, run_tick};
use v3_core::simulation::Simulation;

const RUNTIME_STRESS_PERCEPTION_CREATURES: usize = 200;
const LARGE_TICK_PERCEPTION_CREATURES: usize = 500;

fn runtime_stress_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 240;
    config.world.height = 240;
    config.population.initial_creatures = 400;
    config.runtime.perception.vision_radius = 5;
    config
}

fn runtime_stress_simulation(seed: u64) -> Simulation {
    seed_simulation(runtime_stress_config(), seed)
}

fn runtime_stress_perception_simulation(seed: u64) -> Simulation {
    seed_simulation_with_perception_mix(
        runtime_stress_config(),
        seed,
        RUNTIME_STRESS_PERCEPTION_CREATURES,
    )
}

fn bench_config() -> SimulationConfig {
    let mut config = runtime_stress_config();
    config.world.width = 200;
    config.world.height = 200;
    config.population.initial_creatures = 200;
    config
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

    c.bench_function("mesh_execution_200_creatures", |b| {
        b.iter_with_setup(
            || seed_simulation(bench_config(), 42),
            |mut sim| {
                let config = sim.config.runtime.clone();
                let ids: Vec<_> = sim.creatures.keys().collect();
                for id in ids {
                    let local = assemble_static_inputs(&sim.world, &sim.creatures[id]);
                    let typed_local_food =
                        assemble_typed_food_local_snapshot(&sim.world, sim.creatures[id].position);
                    let ss = SensorSnapshot {
                        local,
                        perception: PerceptionSnapshot::zero(),
                        typed_local_food,
                    };
                    let creature = sim.creatures.get_mut(id).unwrap();
                    let _ = black_box(execute_creature_mesh(
                        &creature.genome,
                        &ss,
                        &mut creature.energy,
                        creature.reproductive_reserve,
                        &mut creature.shared_memory,
                        &creature.prev_shared_memory,
                        &mut creature.graph_runtime,
                        &config,
                    ));
                }
            },
        );
    });
}

fn bench_full_tick_stress(c: &mut Criterion) {
    c.bench_function("full_tick_stress", |b| {
        b.iter_with_setup(
            || runtime_stress_simulation(42),
            |mut sim| {
                for _ in 0..50 {
                    run_tick(black_box(&mut sim), &mut None);
                }
            },
        );
    });
}

fn bench_perception_assembly_stress(c: &mut Criterion) {
    c.bench_function("perception_assembly_stress", |b| {
        b.iter_with_setup(
            || {
                let sim = runtime_stress_perception_simulation(42);
                let extended_count = sim
                    .creatures
                    .values()
                    .filter(|creature| genome_uses_extended_perception(&creature.genome))
                    .count();
                assert_eq!(
                    extended_count, RUNTIME_STRESS_PERCEPTION_CREATURES,
                    "fixture must contain a stable mixed-perception population"
                );
                sim
            },
            |sim| {
                let config = PerceptionConfig::from_sim_config(&sim.config);
                let table = get_visibility_table(config.vision_radius);
                let mut visible_scratch = VisibilityScratch::default();
                for (id, creature) in sim.creatures.iter() {
                    if !genome_uses_extended_perception(&creature.genome) {
                        continue;
                    }
                    let visible = compute_visible_cells_into(
                        creature.position,
                        &sim.world,
                        table,
                        &mut visible_scratch,
                    );
                    let perception = assemble_perception(
                        id,
                        creature,
                        visible,
                        &sim.world,
                        &sim.creatures,
                        &config,
                    );
                    black_box(perception);
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

    c.bench_function("full_tick_2000_creatures_50_ticks_mixed_perception", |b| {
        b.iter_with_setup(
            || {
                let sim = seed_simulation_with_perception_mix(
                    cfg.clone(),
                    42,
                    LARGE_TICK_PERCEPTION_CREATURES,
                );
                let extended_count = sim
                    .creatures
                    .values()
                    .filter(|creature| genome_uses_extended_perception(&creature.genome))
                    .count();
                assert_eq!(
                    extended_count, LARGE_TICK_PERCEPTION_CREATURES,
                    "fixture must contain a stable mixed-perception population"
                );
                sim
            },
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
    bench_full_tick_stress,
    bench_perception_assembly_stress,
    bench_full_tick_large_population
);
criterion_main!(benches);
