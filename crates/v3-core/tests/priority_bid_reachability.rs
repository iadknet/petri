use rand::rngs::SmallRng;
use rand::SeedableRng;

use v3_core::config::SimulationConfig;
use v3_core::contracts::WorldAction;
use v3_core::creature::founder::v3alpha1_founder_genome;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use v3_core::creature::state::GraphRuntimeState;
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::MutationEngine;
use v3_core::runtime::execute_creature_mesh;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;

const SEARCH_SEEDS: u64 = 512;
const SEARCH_GENERATIONS: usize = 12;

#[derive(Debug, Clone)]
struct ProbeCase {
    energy: f32,
    sensors: SensorSnapshot,
}

fn search_config() -> v3_core::config::MutationConfig {
    let mut config = SimulationConfig::default().mutation;
    config.per_unit_supply_enabled = false;
    config.mutation_probability = 1.0;
    config
}

fn rich_perception_snapshot() -> PerceptionSnapshot {
    PerceptionSnapshot {
        area_food: [0.9, 0.4, -0.2, 0.2, -0.3, 0.6, 1.0],
        typed_area_food: vec![[0.9, 0.4, -0.2, 0.2, -0.3, 0.6, 1.0]],
        area_barrier: [0.1, 0.8, -0.4, 0.5, 0.2, -0.1, 0.7],
        area_occupancy: [0.6, -0.2, 0.3, 0.1, -0.4, 0.5, 0.9],
        nearby_core: [
            1.0, -1.0, 0.0, 0.2, 1.0, 1.0, 0.0, 0.3, 1.0, 0.0, -1.0, 0.4, 0.0, 0.0, 0.0, 0.0,
        ],
        nearby_vitals: [0.8, 1.0, 0.4, 0.0, 0.6, 1.0, 0.0, 0.0],
        nearby_identity: [1.0, 1.0, 0.9, 0.4, 0.0, 0.3, 0.2, 0.0, 0.1, 0.0, 0.0, 0.0],
    }
}

fn probe_cases() -> [ProbeCase; 4] {
    [
        ProbeCase {
            energy: 100.0,
            sensors: SensorSnapshot {
                local: StaticInputs {
                    food_here: 1.0,
                    neighbor_food: [0.9, 0.1, 0.8, 0.2, 0.7, 0.3, 0.6, 0.4],
                    neighbor_barrier: [0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
                    neighbor_occupied: [0.0; 8],
                    max_energy: 200.0,
                    age_ticks: 5.0,
                    previous_outcome: [0.0; 4],
                },
                typed_local_food: TypedFoodLocalSnapshot {
                    food_here_by_type: vec![1.0],
                    neighbor_food_by_type: vec![[0.9, 0.1, 0.8, 0.2, 0.7, 0.3, 0.6, 0.4]],
                },
                perception: rich_perception_snapshot(),
            },
        },
        ProbeCase {
            energy: 24.0,
            sensors: SensorSnapshot {
                local: StaticInputs {
                    food_here: 0.0,
                    neighbor_food: [1.0, 0.0, 0.8, 0.0, 0.6, 0.0, 0.4, 0.0],
                    neighbor_barrier: [0.0; 8],
                    neighbor_occupied: [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
                    max_energy: 200.0,
                    age_ticks: 40.0,
                    previous_outcome: [0.0; 4],
                },
                typed_local_food: TypedFoodLocalSnapshot {
                    food_here_by_type: vec![0.0],
                    neighbor_food_by_type: vec![[1.0, 0.0, 0.8, 0.0, 0.6, 0.0, 0.4, 0.0]],
                },
                perception: rich_perception_snapshot(),
            },
        },
        ProbeCase {
            energy: 8.0,
            sensors: SensorSnapshot {
                local: StaticInputs {
                    food_here: 0.2,
                    neighbor_food: [0.0, 0.4, 0.0, 0.6, 0.0, 0.8, 0.0, 1.0],
                    neighbor_barrier: [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
                    neighbor_occupied: [0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
                    max_energy: 200.0,
                    age_ticks: 120.0,
                    previous_outcome: [0.0; 4],
                },
                typed_local_food: TypedFoodLocalSnapshot {
                    food_here_by_type: vec![0.2],
                    neighbor_food_by_type: vec![[0.0, 0.4, 0.0, 0.6, 0.0, 0.8, 0.0, 1.0]],
                },
                perception: rich_perception_snapshot(),
            },
        },
        ProbeCase {
            energy: 100.0,
            sensors: SensorSnapshot {
                local: StaticInputs {
                    food_here: 0.0,
                    neighbor_food: [0.0; 8],
                    neighbor_barrier: [0.0; 8],
                    neighbor_occupied: [0.0; 8],
                    max_energy: 200.0,
                    age_ticks: 0.0,
                    previous_outcome: [0.0; 4],
                },
                typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
                perception: PerceptionSnapshot::zeroed(1),
            },
        },
    ]
}

fn genome_contains_reachable_priority_bid(genome: &CreatureGenome) -> bool {
    mesh_reachable_nodes(genome).into_iter().any(|node_idx| {
        let Some(node) = genome.nodes.get(node_idx) else {
            return false;
        };
        match &node.backend_def {
            BackendDef::Vm(vm) => vm
                .program
                .iter()
                .any(|instr| matches!(instr, VmInstruction::SetPriorityBid { .. })),
            BackendDef::Graph(_) => false,
        }
    })
}

fn genome_emits_positive_priority_bid_with_real_action(genome: &CreatureGenome) -> bool {
    let runtime = SimulationConfig::default().runtime;
    for case in probe_cases() {
        let mut energy = case.energy;
        let mut shared_memory = [0.0; 16];
        let prev_shared_memory = [0.0; 16];
        let mut graph_runtime = GraphRuntimeState::new();
        let output = execute_creature_mesh(
            genome,
            &case.sensors,
            &mut energy,
            &mut shared_memory,
            &prev_shared_memory,
            &mut graph_runtime,
            &runtime,
        );
        if output.priority_bid > 0.0
            && output
                .actions
                .iter()
                .any(|action| !matches!(action, WorldAction::NoOp))
        {
            return true;
        }
    }
    false
}

fn lineage_match_exists(predicate: impl Fn(&CreatureGenome) -> bool) -> bool {
    let config = search_config();

    for seed in 0..SEARCH_SEEDS {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut genome = v3alpha1_founder_genome();

        for _generation in 1..=SEARCH_GENERATIONS {
            let reachable_nodes = mesh_reachable_nodes(&genome);
            MutationEngine::apply_mutations_with_food_type_count(
                &mut genome,
                &config,
                &reachable_nodes,
                ParentExecuted::NONE,
                &mut rng,
                1,
            );

            if predicate(&genome) {
                return true;
            }
        }
    }

    false
}

#[test]
fn founder_lineage_can_mutate_reachable_set_priority_bid_opcode() {
    assert!(
        lineage_match_exists(genome_contains_reachable_priority_bid),
        "searched {SEARCH_SEEDS} seeds x {SEARCH_GENERATIONS} generations and never introduced SetPriorityBid into a reachable node from the founder lineage"
    );
}

#[test]
fn founder_lineage_can_execute_positive_priority_bid() {
    assert!(
        lineage_match_exists(genome_emits_positive_priority_bid_with_real_action),
        "searched {SEARCH_SEEDS} seeds x {SEARCH_GENERATIONS} generations and never found a founder-descended genome that emits a positive priority bid alongside a real action"
    );
}
