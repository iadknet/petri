use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use slotmap::Key;
use tokio::sync::{broadcast, Mutex};
use v3_core::config::SimulationConfig;
use v3_core::mutation::phenotype::channels_to_rgb;
use v3_core::simulation::{seed_simulation, Simulation};

use crate::query::projection::ProjectionStore;
use crate::transport::session::ProjectionNotice;
use crate::transport::session_registry::SessionRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationStatus {
    Idle,
    Running,
    Paused,
}

pub struct SimHandle {
    pub sim: Simulation,
    pub status: SimulationStatus,
    pub active_trace: Option<v3_core::runtime::trace::recording::ActiveTrace>,
    /// Cached quantized fertility grid. Computed once at startup/restart; fertility
    /// is static and does not change between resets.
    pub cached_fertility_u8: Arc<[u8]>,
}

impl SimHandle {
    pub(crate) fn new(config: SimulationConfig, seed: u64) -> Self {
        let sim = seed_simulation(config, seed);
        let cached_fertility_u8 =
            crate::query::cache::build_primary_food_fertility_u8(sim.world.food());
        Self {
            sim,
            status: SimulationStatus::Idle,
            active_trace: None,
            cached_fertility_u8,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LastTickActions {
    #[serde(rename = "move")]
    pub move_count: u32,
    pub eat: u32,
    pub reproduce: u32,
    pub noop: u32,
    pub steal: u32,
    pub predation_kills: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredationEventSnapshot {
    pub attacker_x: u16,
    pub attacker_y: u16,
    pub victim_x: u16,
    pub victim_y: u16,
    pub energy_stolen: f32,
    pub killed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusPayload {
    pub state: SimulationStatus,
    pub population: usize,
    pub mean_energy: f32,
    pub last_tick_actions: LastTickActions,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub predation_actions_attempted_total: u64,
    pub predation_actions_transferred_total: u64,
    pub predation_actions_rejected_total: u64,
    pub predation_kills_total: u64,
    pub last_tick_compute_total_mean: f32,
    pub last_tick_compute_total_min: f32,
    pub last_tick_compute_total_max: f32,
    pub last_tick_compute_vm_mean: f32,
    pub last_tick_compute_graph_mean: f32,
    pub last_tick_food_occupancy_depletion_mean: f32,
    pub last_tick_food_occupancy_depletion_occupied_cells: u32,
    pub last_tick_food_growth_suppressed_by_occupancy_depletion: f32,
    pub last_tick_food_cells_with_type_inhibition: u32,
    pub last_tick_food_growth_suppressed_by_type_inhibition: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub energy: f32,
    pub reproductive_reserve: f32,
    pub generation: u64,
    pub phenotype_rgb: [u8; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodCell {
    pub x: u16,
    pub y: u16,
    pub type_idx: u16,
    pub density: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoodTypeSnapshot {
    pub type_idx: u16,
    pub name: String,
    pub color: String,
    pub growth_inhibitor: f32,
    pub metabolic_energy_yield: f32,
    pub reproductive_reserve_yield: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BarrierCell {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FramePayload {
    pub width: u16,
    pub height: u16,
    pub creatures: Vec<CreatureSnapshot>,
    pub food_types: Vec<FoodTypeSnapshot>,
    pub food: Vec<FoodCell>,
    pub barriers: Vec<BarrierCell>,
    pub food_fertility_u8: Arc<[u8]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthPayload {
    pub population: usize,
    pub mean_energy: f32,
    pub last_tick_food_occupancy_depletion_mean: f32,
    pub last_tick_food_occupancy_depletion_occupied_cells: u32,
    pub last_tick_food_growth_suppressed_by_occupancy_depletion: f32,
    pub last_tick_food_cells_with_type_inhibition: u32,
    pub last_tick_food_growth_suppressed_by_type_inhibition: f32,
    pub mutation_events_attempted_total: u64,
    pub mutation_events_applied_total: u64,
    pub mutation_events_skipped_total: u64,
    pub mutation_events_attempted_total_by_domain: HashMap<String, u64>,
    pub mutation_events_applied_total_by_domain: HashMap<String, u64>,
    pub mutation_events_attempted_total_by_operator: HashMap<String, u64>,
    pub mutation_events_applied_total_by_operator: HashMap<String, u64>,
    pub mutation_events_skipped_total_by_operator: HashMap<String, u64>,
    pub mutation_operator_funnel_total_by_operator: HashMap<String, MutationOperatorFunnelPayload>,
    pub mutation_skip_reasons_total_by_operator: HashMap<String, HashMap<String, u64>>,
    pub mutation_added_node_input_classes_total_by_operator: HashMap<String, HashMap<String, u64>>,
    pub mutation_added_node_world_inputs_total_by_operator: HashMap<String, HashMap<String, u64>>,
    pub vm_live_read_world_inputs_current: HashMap<String, u64>,
    pub mutation_events_applied_total_semantic_noop: u64,
    pub mutation_events_applied_total_semantic_change: u64,
    pub mutation_target_reachability_total: MutationTargetReachabilityTotalPayload,
    pub mutation_value_totals_by_operator: HashMap<String, MutationOperatorValueTotalsPayload>,
    pub mutation_outcome_summary: MutationOperatorValueTotalsPayload,
    pub reproduction_actions_attempted_total: u64,
    pub reproduction_actions_spawned_total: u64,
    pub reproduction_actions_rejected_total: u64,
    pub reproduction_actions_rejected_total_by_reason: HashMap<String, u64>,
    pub reproduction_actions_rejected_invalid_target_total_by_cause: HashMap<String, u64>,
    pub reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
        HashMap<String, u64>,
    pub mutation_events_skipped_total_by_reason: HashMap<String, u64>,
    pub move_actions_blocked_total_by_cause: HashMap<String, u64>,
    pub move_actions_blocked_avoidable_total_by_reader_state: HashMap<String, u64>,
    pub move_attempts_with_barrier_neighbor_total_by_reader_state: HashMap<String, u64>,
    pub move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: HashMap<String, u64>,
    pub reproduction_attempts_with_barrier_neighbor_total_by_reader_state: HashMap<String, u64>,
    pub reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state:
        HashMap<String, u64>,
    pub predation_actions_attempted_total: u64,
    pub predation_actions_transferred_total: u64,
    pub predation_actions_rejected_total: u64,
    pub predation_kills_total: u64,
    pub predation_actions_by_result: HashMap<String, u64>,
    pub genome_complexity_mean: f32,
    pub genome_complexity_min: u32,
    pub genome_complexity_max: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MutationTargetReachabilityTotalPayload {
    pub reachable: u64,
    pub unreachable: u64,
    pub not_applicable: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MutationOperatorFunnelPayload {
    pub attempted: u64,
    pub applicable: u64,
    pub structurally_valid: u64,
    pub applied: u64,
    pub semantic_change: u64,
    pub skipped: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MutationOperatorValueTotalsPayload {
    pub carriers_observed_total: u64,
    pub survival_ticks_sum: u64,
    pub offspring_spawned_sum: u64,
    pub final_energy_sum: f64,
    pub helpful_total: u64,
    pub neutral_total: u64,
    pub detrimental_total: u64,
    pub confidence_low_total: u64,
    pub confidence_medium_total: u64,
    pub confidence_high_total: u64,
    pub viability_score_sum: f64,
    pub viability_score_delta_sum: f64,
    pub survived_short_horizon_total: u64,
    pub survived_long_horizon_total: u64,
    pub reproduced_once_total: u64,
    pub mean_lifetime_energy_sum: f64,
    pub action_attempted_total: u64,
    pub blocked_move_total: u64,
    pub invalid_reproduce_total: u64,
    pub invalid_action_total: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsFrame {
    pub tick: u64,
    pub status: StatusPayload,
    pub frame: FramePayload,
    pub health: HealthPayload,
    pub predation_events: Vec<PredationEventSnapshot>,
}

#[derive(Clone)]
pub struct AppState {
    pub sim: Arc<Mutex<SimHandle>>,
    pub projection: Arc<RwLock<ProjectionStore>>,
    pub perf: Arc<RwLock<TransportPerfSnapshot>>,
    pub sessions: Arc<RwLock<SessionRegistry>>,
    pub ws_tx: broadcast::Sender<ProjectionNotice>,
    pub(crate) startup_defaults: Arc<SimulationConfig>,
}

#[derive(Clone, Debug, Default)]
pub struct TransportPerfSnapshot {
    pub projection_publish_ms: f64,
    pub ws_frame_publish_ms: f64,
}

pub fn build_ws_frame(handle: &SimHandle) -> WsFrame {
    let sim = &handle.sim;
    let stats = &sim.stats;

    let status = StatusPayload {
        state: handle.status,
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        last_tick_actions: LastTickActions {
            move_count: stats.last_tick_move,
            eat: stats.last_tick_eat,
            reproduce: stats.last_tick_reproduce,
            noop: stats.last_tick_noop,
            steal: stats.last_tick_steal,
            predation_kills: stats.last_tick_predation_kills,
        },
        reproduction_actions_attempted_total: stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: stats.reproduction_actions_rejected_total,
        predation_actions_attempted_total: stats.predation_actions_attempted_total,
        predation_actions_transferred_total: stats.predation_actions_transferred_total,
        predation_actions_rejected_total: stats.predation_actions_rejected_total,
        predation_kills_total: stats.predation_kills_total,
        last_tick_compute_total_mean: stats.last_tick_compute_total_mean,
        last_tick_compute_total_min: stats.last_tick_compute_total_min,
        last_tick_compute_total_max: stats.last_tick_compute_total_max,
        last_tick_compute_vm_mean: stats.last_tick_compute_vm_mean,
        last_tick_compute_graph_mean: stats.last_tick_compute_graph_mean,
        last_tick_food_occupancy_depletion_mean: stats.last_tick_food_occupancy_depletion_mean,
        last_tick_food_occupancy_depletion_occupied_cells: stats
            .last_tick_food_occupancy_depletion_occupied_cells,
        last_tick_food_growth_suppressed_by_occupancy_depletion: stats
            .last_tick_food_growth_suppressed_by_occupancy_depletion,
        last_tick_food_cells_with_type_inhibition: stats.last_tick_food_cells_with_type_inhibition,
        last_tick_food_growth_suppressed_by_type_inhibition: stats
            .last_tick_food_growth_suppressed_by_type_inhibition,
    };

    let mut creatures = Vec::with_capacity(sim.creatures.len());
    let mut complexity_sum: u64 = 0;
    let mut complexity_min: u32 = u32::MAX;
    let mut complexity_max: u32 = 0;
    let mut vm_live_read_world_inputs_current = std::collections::HashMap::new();
    for (id, creature) in &sim.creatures {
        let c = creature.cached_complexity;
        complexity_sum += c as u64;
        complexity_min = complexity_min.min(c);
        complexity_max = complexity_max.max(c);
        for (key, count) in creature.cached_live_vm_world_inputs.iter().copied() {
            *vm_live_read_world_inputs_current.entry(key).or_insert(0u64) += u64::from(count);
        }
        creatures.push(CreatureSnapshot {
            id: id.data().as_ffi(),
            x: creature.position.x,
            y: creature.position.y,
            energy: creature.energy,
            reproductive_reserve: creature.reproductive_reserve,
            generation: creature.generation,
            phenotype_rgb: channels_to_rgb(creature.phenotype_channels),
        });
    }
    let creature_count = creatures.len();
    let complexity_mean = if creature_count > 0 {
        complexity_sum as f32 / creature_count as f32
    } else {
        0.0
    };
    if creature_count == 0 {
        complexity_min = 0;
    }

    let food_types = sim
        .world
        .food()
        .food_types()
        .iter()
        .map(|food_type| FoodTypeSnapshot {
            type_idx: food_type.id.get(),
            name: food_type.config.name.clone(),
            color: food_type.config.color.clone(),
            growth_inhibitor: food_type.config.growth_inhibitor,
            metabolic_energy_yield: food_type.config.metabolic_energy_yield,
            reproductive_reserve_yield: food_type.config.reproductive_reserve_yield,
        })
        .collect();

    let mut food = Vec::new();
    sim.world
        .food()
        .for_each_food_cell(|x, y, type_idx, density| {
            food.push(FoodCell {
                x,
                y,
                type_idx: type_idx.get(),
                density,
            });
        });

    let mut barriers = Vec::new();
    for y in 0..sim.world.height {
        for x in 0..sim.world.width {
            let pos = v3_core::contracts::Position::new(x, y);
            if sim.world.is_barrier(pos) {
                barriers.push(BarrierCell { x, y });
            }
        }
    }

    let food_fertility_u8 = Arc::clone(&handle.cached_fertility_u8);

    let frame = FramePayload {
        width: sim.world.width,
        height: sim.world.height,
        creatures,
        food_types,
        food,
        barriers,
        food_fertility_u8,
    };

    let map_mutation_value_totals = |totals: &v3_core::simulation::stats::MutationValueTotals| {
        MutationOperatorValueTotalsPayload {
            carriers_observed_total: totals.carriers_observed_total,
            survival_ticks_sum: totals.survival_ticks_sum,
            offspring_spawned_sum: totals.offspring_spawned_sum,
            final_energy_sum: totals.final_energy_sum,
            helpful_total: totals.helpful_total,
            neutral_total: totals.neutral_total,
            detrimental_total: totals.detrimental_total,
            confidence_low_total: totals.confidence_low_total,
            confidence_medium_total: totals.confidence_medium_total,
            confidence_high_total: totals.confidence_high_total,
            viability_score_sum: totals.viability_score_sum,
            viability_score_delta_sum: totals.viability_score_delta_sum,
            survived_short_horizon_total: totals.survived_short_horizon_total,
            survived_long_horizon_total: totals.survived_long_horizon_total,
            reproduced_once_total: totals.reproduced_once_total,
            mean_lifetime_energy_sum: totals.mean_lifetime_energy_sum,
            action_attempted_total: totals.action_attempted_total,
            blocked_move_total: totals.blocked_move_total,
            invalid_reproduce_total: totals.invalid_reproduce_total,
            invalid_action_total: totals.invalid_action_total,
        }
    };

    let health = HealthPayload {
        population: sim.creatures.len(),
        mean_energy: sim.mean_energy(),
        last_tick_food_occupancy_depletion_mean: stats.last_tick_food_occupancy_depletion_mean,
        last_tick_food_occupancy_depletion_occupied_cells: stats
            .last_tick_food_occupancy_depletion_occupied_cells,
        last_tick_food_growth_suppressed_by_occupancy_depletion: stats
            .last_tick_food_growth_suppressed_by_occupancy_depletion,
        last_tick_food_cells_with_type_inhibition: stats.last_tick_food_cells_with_type_inhibition,
        last_tick_food_growth_suppressed_by_type_inhibition: stats
            .last_tick_food_growth_suppressed_by_type_inhibition,
        mutation_events_attempted_total: stats.mutation_events_attempted_total,
        mutation_events_applied_total: stats.mutation_events_applied_total,
        mutation_events_skipped_total: stats.mutation_events_skipped_total,
        mutation_events_attempted_total_by_domain: stats
            .mutation_events_attempted_total_by_domain
            .iter()
            .map(|(domain, count)| (domain.as_key().to_string(), *count))
            .collect(),
        mutation_events_applied_total_by_domain: stats
            .mutation_events_applied_total_by_domain
            .iter()
            .map(|(domain, count)| (domain.as_key().to_string(), *count))
            .collect(),
        mutation_events_attempted_total_by_operator: stats
            .mutation_events_attempted_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_events_applied_total_by_operator: stats
            .mutation_events_applied_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_events_skipped_total_by_operator: stats
            .mutation_events_skipped_total_by_operator
            .iter()
            .map(|(operator, count)| (operator.as_key().to_string(), *count))
            .collect(),
        mutation_operator_funnel_total_by_operator: stats
            .mutation_operator_funnel_total_by_operator
            .iter()
            .map(|(operator, funnel)| {
                (
                    operator.as_key().to_string(),
                    MutationOperatorFunnelPayload {
                        attempted: funnel.attempted,
                        applicable: funnel.applicable,
                        structurally_valid: funnel.structurally_valid,
                        applied: funnel.applied,
                        semantic_change: funnel.semantic_change,
                        skipped: funnel.skipped,
                    },
                )
            })
            .collect(),
        mutation_skip_reasons_total_by_operator: stats
            .mutation_skip_reasons_total_by_operator
            .iter()
            .map(|(operator, reasons)| {
                (
                    operator.as_key().to_string(),
                    reasons
                        .iter()
                        .map(|(reason, count)| (reason.as_key().to_string(), *count))
                        .collect(),
                )
            })
            .collect(),
        mutation_added_node_input_classes_total_by_operator: stats
            .mutation_added_node_input_classes_total_by_operator
            .iter()
            .map(|(operator, classes)| {
                (
                    operator.as_key().to_string(),
                    classes
                        .iter()
                        .map(|(class, count)| (class.as_key().to_string(), *count))
                        .collect(),
                )
            })
            .collect(),
        mutation_added_node_world_inputs_total_by_operator: stats
            .mutation_added_node_world_inputs_total_by_operator
            .iter()
            .map(|(operator, inputs)| {
                (
                    operator.as_key().to_string(),
                    inputs
                        .iter()
                        .map(|(input, count)| (input.as_key().to_string(), *count))
                        .collect(),
                )
            })
            .collect(),
        vm_live_read_world_inputs_current: vm_live_read_world_inputs_current
            .into_iter()
            .map(|(key, count)| (key.as_key().to_string(), count))
            .collect(),
        mutation_events_applied_total_semantic_noop: stats
            .mutation_events_applied_total_semantic_noop,
        mutation_events_applied_total_semantic_change: stats
            .mutation_events_applied_total_semantic_change,
        mutation_target_reachability_total: MutationTargetReachabilityTotalPayload {
            reachable: stats.mutation_reachable_target_total,
            unreachable: stats.mutation_unreachable_target_total,
            not_applicable: stats.mutation_not_applicable_target_total,
        },
        mutation_value_totals_by_operator: stats
            .mutation_value_totals_by_operator
            .iter()
            .map(|(operator, totals)| {
                (
                    operator.as_key().to_string(),
                    map_mutation_value_totals(totals),
                )
            })
            .collect(),
        mutation_outcome_summary: map_mutation_value_totals(&stats.mutation_outcome_summary),
        reproduction_actions_attempted_total: stats.reproduction_actions_attempted_total,
        reproduction_actions_spawned_total: stats.reproduction_actions_spawned_total,
        reproduction_actions_rejected_total: stats.reproduction_actions_rejected_total,
        reproduction_actions_rejected_total_by_reason: stats
            .reproduction_actions_rejected_by_reason
            .iter()
            .map(|(reason, count)| (reason.as_key().to_string(), *count))
            .collect(),
        reproduction_actions_rejected_invalid_target_total_by_cause: stats
            .reproduction_actions_rejected_invalid_target_total_by_cause
            .iter()
            .map(|(cause, count)| (cause.as_key().to_string(), *count))
            .collect(),
        reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state: stats
            .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        mutation_events_skipped_total_by_reason: stats
            .mutation_events_skipped_by_reason
            .iter()
            .map(|(reason, count)| (reason.as_key().to_string(), *count))
            .collect(),
        move_actions_blocked_total_by_cause: stats
            .move_actions_blocked_total_by_cause
            .iter()
            .map(|(cause, count)| (cause.as_key().to_string(), *count))
            .collect(),
        move_actions_blocked_avoidable_total_by_reader_state: stats
            .move_actions_blocked_avoidable_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        move_attempts_with_barrier_neighbor_total_by_reader_state: stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: stats
            .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        reproduction_attempts_with_barrier_neighbor_total_by_reader_state: stats
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state: stats
            .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
            .iter()
            .map(|(state, count)| (state.as_key().to_string(), *count))
            .collect(),
        predation_actions_attempted_total: stats.predation_actions_attempted_total,
        predation_actions_transferred_total: stats.predation_actions_transferred_total,
        predation_actions_rejected_total: stats.predation_actions_rejected_total,
        predation_kills_total: stats.predation_kills_total,
        predation_actions_by_result: stats
            .predation_actions_by_result
            .iter()
            .map(|(result, count)| (result.as_key().to_string(), *count))
            .collect(),
        genome_complexity_mean: complexity_mean,
        genome_complexity_min: complexity_min,
        genome_complexity_max: complexity_max,
    };

    let predation_events: Vec<PredationEventSnapshot> = stats
        .last_tick_predation_events
        .iter()
        .map(|event| PredationEventSnapshot {
            attacker_x: event.attacker_x,
            attacker_y: event.attacker_y,
            victim_x: event.victim_x,
            victim_y: event.victim_y,
            energy_stolen: event.energy_stolen,
            killed: event.killed,
        })
        .collect();

    WsFrame {
        tick: sim.tick,
        status,
        frame,
        health,
        predation_events,
    }
}
