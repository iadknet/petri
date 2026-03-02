use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use v3_core::config::{RuntimeConfig, SimulationConfig};
use v3_core::contracts::InputReference;
use v3_core::creature::genome::{BackendDef, GraphBackendDef, VmBackendDef};
use v3_core::creature::state::{CreatureState, GraphRuntimeState};
use v3_core::runtime::graph::execute_graph_node;
use v3_core::runtime::types::MeshSideOutputs;
use v3_core::runtime::vm::execute_vm_node;
use v3_core::sensors::perception::{
    genome_uses_extended_perception, PerceptionConfig, PerceptionSnapshot, SensorSnapshot,
};
use v3_core::sensors::reducers::assemble_perception;
use v3_core::sensors::static_inputs::assemble_static_inputs;
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

fn build_sensor_snapshot(sim: &Simulation, creature: &CreatureState) -> SensorSnapshot {
    let local = assemble_static_inputs(&sim.world, creature);
    let perception = if genome_uses_extended_perception(&creature.genome) {
        let config = PerceptionConfig::from_sim_config(&sim.config);
        let mut visible_scratch = VisibilityScratch::default();
        let visible = compute_visible_cells_into(
            creature.position,
            &sim.world,
            get_visibility_table(config.vision_radius),
            &mut visible_scratch,
        );
        assemble_perception(
            creature.id,
            creature,
            visible,
            &sim.world,
            &sim.creatures,
            &config,
        )
    } else {
        PerceptionSnapshot::zero()
    };

    SensorSnapshot { local, perception }
}

struct VmBenchFixture {
    def: VmBackendDef,
    input_refs: Vec<InputReference>,
    sensors: SensorSnapshot,
    runtime_config: RuntimeConfig,
    energy: f32,
    memory: [u8; 1024],
}

fn build_vm_fixture() -> VmBenchFixture {
    let sim = runtime_stress_simulation(42);
    let creature = sim
        .creatures
        .values()
        .next()
        .expect("runtime stress sim has creatures");
    let sensors = build_sensor_snapshot(&sim, creature);
    let (def, input_refs) = creature
        .genome
        .nodes
        .iter()
        .find_map(|node| match &node.backend_def {
            BackendDef::Vm(def) => Some((def.clone(), node.input_refs.clone())),
            BackendDef::Graph(_) => None,
        })
        .expect("founder genome contains a VM node");

    VmBenchFixture {
        def,
        input_refs,
        sensors,
        runtime_config: sim.config.runtime.clone(),
        energy: creature.energy.max(1.0),
        memory: creature.memory,
    }
}

struct GraphBenchFixture {
    def: GraphBackendDef,
    input_refs: Vec<InputReference>,
    sensors: SensorSnapshot,
    runtime_config: RuntimeConfig,
    energy: f32,
    node_idx: usize,
    graph_runtime: GraphRuntimeState,
}

fn build_graph_fixture() -> GraphBenchFixture {
    let sim = runtime_stress_simulation(42);
    let creature = sim
        .creatures
        .values()
        .next()
        .expect("runtime stress sim has creatures");
    let sensors = build_sensor_snapshot(&sim, creature);
    let (node_idx, def, input_refs) = creature
        .genome
        .nodes
        .iter()
        .enumerate()
        .find_map(|(node_idx, node)| match &node.backend_def {
            BackendDef::Graph(def) => Some((node_idx, def.clone(), node.input_refs.clone())),
            BackendDef::Vm(_) => None,
        })
        .expect("founder genome contains a graph node");

    GraphBenchFixture {
        def,
        input_refs,
        sensors,
        runtime_config: sim.config.runtime.clone(),
        energy: creature.energy.max(1.0),
        node_idx,
        graph_runtime: GraphRuntimeState::new(),
    }
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

fn bench_vm_execute_stress(c: &mut Criterion) {
    c.bench_function("vm_execute_stress", |b| {
        b.iter_batched(
            build_vm_fixture,
            |mut fixture| {
                let mut side_outputs =
                    MeshSideOutputs::new(fixture.runtime_config.max_actions_per_turn);
                let _ = black_box(execute_vm_node(
                    &fixture.def,
                    &fixture.input_refs,
                    &[0.0; 12],
                    &mut fixture.energy,
                    0.0,
                    &mut fixture.memory,
                    &fixture.sensors,
                    &fixture.runtime_config,
                    &mut side_outputs,
                ));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_graph_execute_stress(c: &mut Criterion) {
    c.bench_function("graph_execute_stress", |b| {
        b.iter_batched(
            build_graph_fixture,
            |mut fixture| {
                let mut side_outputs =
                    MeshSideOutputs::new(fixture.runtime_config.max_actions_per_turn);
                let _ = black_box(execute_graph_node(
                    &fixture.def,
                    &fixture.input_refs,
                    &[0.0; 12],
                    &mut fixture.energy,
                    0.0,
                    fixture.node_idx,
                    &mut fixture.graph_runtime,
                    &fixture.sensors,
                    &fixture.runtime_config,
                    &mut side_outputs,
                ));
            },
            BatchSize::SmallInput,
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
    bench_vm_execute_stress,
    bench_graph_execute_stress,
    bench_full_tick_large_population
);
criterion_main!(benches);
