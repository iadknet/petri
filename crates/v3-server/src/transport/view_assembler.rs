//! View payload assembly.

use std::collections::BTreeMap;

use crate::query::projection::ProjectionSnapshot;
use crate::query::spatial_index::ViewRect;
use crate::state::FoodCell;
use crate::transport::protocol::{
    OverviewFoodCellPayload, ViewDetailPayload, ViewOverviewPayload, ViewRectPayload,
};
use crate::transport::session::ViewSubscription;

#[must_use]
pub fn assemble_detail_payload(
    snapshot: &ProjectionSnapshot,
    subscription: &ViewSubscription,
) -> ViewDetailPayload {
    let rect = subscription_rect(snapshot, subscription);
    let mut creatures = Vec::new();
    for index in snapshot.creature_tile_index.query_indices(rect) {
        let creature = &snapshot.ws_frame.frame.creatures[index];
        if rect.contains(creature.x, creature.y) {
            creatures.push(creature.clone());
        }
    }

    let world_width = usize::from(snapshot.ws_frame.frame.width);
    let type_count = snapshot.ws_frame.frame.food_types.len();
    let cell_count = world_width * usize::from(snapshot.ws_frame.frame.height);
    let mut food = Vec::new();
    for type_idx in 0..type_count {
        let Ok(type_idx_u16) = u16::try_from(type_idx) else {
            break;
        };
        let plane_start = type_idx * cell_count;
        for y in rect.y..rect.y.saturating_add(rect.height) {
            let row_offset = usize::from(y) * world_width;
            for x in rect.x..rect.x.saturating_add(rect.width) {
                let idx = plane_start + row_offset + usize::from(x);
                let density = snapshot
                    .food_density_planes
                    .get(idx)
                    .copied()
                    .unwrap_or(0.0);
                if density <= 0.0 {
                    continue;
                }
                food.push(FoodCell {
                    x,
                    y,
                    type_idx: type_idx_u16,
                    density,
                });
            }
        }
    }
    let predation_events = snapshot
        .ws_frame
        .predation_events
        .iter()
        .filter(|event| {
            rect.contains(event.attacker_x, event.attacker_y)
                || rect.contains(event.victim_x, event.victim_y)
        })
        .cloned()
        .collect();

    ViewDetailPayload {
        rect: ViewRectPayload::from(rect),
        width: rect.width,
        height: rect.height,
        food,
        creatures,
        predation_events,
    }
}

#[must_use]
pub fn assemble_overview_payload(
    snapshot: &ProjectionSnapshot,
    subscription: &ViewSubscription,
) -> ViewOverviewPayload {
    let rect = subscription_rect(snapshot, subscription);
    let grid_width = rect.width.min(128) as usize;
    let grid_height = rect.height.min(128) as usize;
    let grid_cells = grid_width * grid_height;
    let mut food_by_bucket: BTreeMap<(u16, u16, u16), f32> = BTreeMap::new();
    let mut creature_count_u16 = vec![0u16; grid_cells];

    let world_width = usize::from(snapshot.ws_frame.frame.width);
    let type_count = snapshot.ws_frame.frame.food_types.len();
    let cell_count = world_width * usize::from(snapshot.ws_frame.frame.height);
    for type_idx in 0..type_count {
        let Ok(type_idx_u16) = u16::try_from(type_idx) else {
            break;
        };
        let plane_start = type_idx * cell_count;
        for y in rect.y..rect.y.saturating_add(rect.height) {
            let row_offset = usize::from(y) * world_width;
            for x in rect.x..rect.x.saturating_add(rect.width) {
                let idx = plane_start + row_offset + usize::from(x);
                let density = snapshot
                    .food_density_planes
                    .get(idx)
                    .copied()
                    .unwrap_or(0.0);
                if density <= 0.0 {
                    continue;
                }
                let bucket_x = ((x - rect.x) as usize * grid_width) / rect.width as usize;
                let bucket_y = ((y - rect.y) as usize * grid_height) / rect.height as usize;
                let key = (
                    bucket_x.min(grid_width.saturating_sub(1)) as u16,
                    bucket_y.min(grid_height.saturating_sub(1)) as u16,
                    type_idx_u16,
                );
                *food_by_bucket.entry(key).or_insert(0.0) += density;
            }
        }
    }

    for index in snapshot.creature_tile_index.query_indices(rect) {
        let creature = &snapshot.ws_frame.frame.creatures[index];
        if !rect.contains(creature.x, creature.y) {
            continue;
        }
        let bucket_x = ((creature.x - rect.x) as usize * grid_width) / rect.width as usize;
        let bucket_y = ((creature.y - rect.y) as usize * grid_height) / rect.height as usize;
        let dst = bucket_y.min(grid_height - 1) * grid_width + bucket_x.min(grid_width - 1);
        creature_count_u16[dst] = creature_count_u16[dst].saturating_add(1);
    }

    let food = food_by_bucket
        .into_iter()
        .map(
            |((bucket_x, bucket_y, type_idx), density)| OverviewFoodCellPayload {
                bucket_x,
                bucket_y,
                type_idx,
                density,
            },
        )
        .collect();

    ViewOverviewPayload {
        rect: ViewRectPayload::from(rect),
        grid_width: grid_width as u16,
        grid_height: grid_height as u16,
        food,
        creature_count_u16,
    }
}

#[must_use]
pub fn subscription_rect(
    snapshot: &ProjectionSnapshot,
    subscription: &ViewSubscription,
) -> ViewRect {
    let world_width = snapshot.ws_frame.frame.width;
    let world_height = snapshot.ws_frame.frame.height;
    let max_x = world_width.saturating_sub(1);
    let max_y = world_height.saturating_sub(1);
    let x = subscription.x.min(max_x);
    let y = subscription.y.min(max_y);
    let width = subscription.width.min(world_width.saturating_sub(x)).max(1);
    let height = subscription
        .height
        .min(world_height.saturating_sub(y))
        .max(1);

    ViewRect {
        x,
        y,
        width,
        height,
    }
}

impl From<ViewRect> for ViewRectPayload {
    fn from(value: ViewRect) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::query::projection::ProjectionSnapshot;
    use crate::state::{
        BarrierCell, CreatureSnapshot, FoodCell, FoodTypeSnapshot, FramePayload, HealthPayload,
        LastTickActions, SimulationStatus, StatusPayload, WsFrame,
    };
    use crate::transport::protocol::ZoomTier;
    use crate::transport::session::ViewSubscription;

    use super::{assemble_detail_payload, assemble_overview_payload};

    fn sample_snapshot() -> ProjectionSnapshot {
        ProjectionSnapshot::from_ws_frame(
            7,
            3,
            WsFrame {
                tick: 12,
                status: StatusPayload {
                    state: SimulationStatus::Paused,
                    population: 2,
                    mean_energy: 5.0,
                    last_tick_actions: LastTickActions {
                        move_count: 0,
                        eat: 0,
                        reproduce: 0,
                        noop: 0,
                        steal: 0,
                        predation_kills: 0,
                    },
                    reproduction_actions_attempted_total: 0,
                    reproduction_actions_spawned_total: 0,
                    reproduction_actions_rejected_total: 0,
                    predation_actions_attempted_total: 0,
                    predation_actions_transferred_total: 0,
                    predation_actions_rejected_total: 0,
                    predation_kills_total: 0,
                    last_tick_compute_total_mean: 0.0,
                    last_tick_compute_total_min: 0.0,
                    last_tick_compute_total_max: 0.0,
                    last_tick_compute_vm_mean: 0.0,
                    last_tick_compute_graph_mean: 0.0,
                    last_tick_food_occupancy_depletion_mean: 0.0,
                    last_tick_food_occupancy_depletion_occupied_cells: 0,
                    last_tick_food_growth_suppressed_by_occupancy_depletion: 0.0,
                    last_tick_food_cells_with_type_inhibition: 0,
                    last_tick_food_growth_suppressed_by_type_inhibition: 0.0,
                },
                frame: FramePayload {
                    width: 256,
                    height: 256,
                    creatures: vec![
                        CreatureSnapshot {
                            id: 1,
                            x: 2,
                            y: 3,
                            energy: 10.0,
                            generation: 1,
                            phenotype_rgb: [1, 2, 3],
                            reproductive_reserve: 0.0,
                        },
                        CreatureSnapshot {
                            id: 2,
                            x: 200,
                            y: 200,
                            energy: 20.0,
                            generation: 2,
                            phenotype_rgb: [4, 5, 6],
                            reproductive_reserve: 0.0,
                        },
                    ],
                    food_types: vec![
                        FoodTypeSnapshot {
                            type_idx: 0,
                            name: "Primary Food".to_string(),
                            color: "#22c55e".to_string(),
                            growth_inhibitor: 0.2,
                            metabolic_energy_yield: 10.0,
                            reproductive_reserve_yield: 0.0,
                        },
                        FoodTypeSnapshot {
                            type_idx: 1,
                            name: "Secondary Food".to_string(),
                            color: "#0ea5e9".to_string(),
                            growth_inhibitor: 0.3,
                            metabolic_energy_yield: 0.0,
                            reproductive_reserve_yield: 1.0,
                        },
                    ],
                    food: vec![
                        FoodCell {
                            x: 2,
                            y: 2,
                            type_idx: 0,
                            density: 1.0,
                        },
                        FoodCell {
                            x: 2,
                            y: 2,
                            type_idx: 1,
                            density: 0.5,
                        },
                        FoodCell {
                            x: 3,
                            y: 3,
                            type_idx: 0,
                            density: 0.25,
                        },
                    ],
                    barriers: vec![BarrierCell { x: 3, y: 3 }, BarrierCell { x: 180, y: 180 }],
                    food_fertility_u8: vec![128u8; 256 * 256].into(),
                },
                health: HealthPayload {
                    population: 2,
                    mean_energy: 5.0,
                    last_tick_food_occupancy_depletion_mean: 0.0,
                    last_tick_food_occupancy_depletion_occupied_cells: 0,
                    last_tick_food_growth_suppressed_by_occupancy_depletion: 0.0,
                    last_tick_food_cells_with_type_inhibition: 0,
                    last_tick_food_growth_suppressed_by_type_inhibition: 0.0,
                    mutation_events_attempted_total: 0,
                    mutation_events_applied_total: 0,
                    mutation_events_skipped_total: 0,
                    mutation_events_attempted_total_by_domain: Default::default(),
                    mutation_events_applied_total_by_domain: Default::default(),
                    mutation_events_attempted_total_by_operator: Default::default(),
                    mutation_events_applied_total_by_operator: Default::default(),
                    mutation_events_skipped_total_by_operator: Default::default(),
                    mutation_operator_funnel_total_by_operator: Default::default(),
                    mutation_skip_reasons_total_by_operator: Default::default(),
                    mutation_added_node_input_classes_total_by_operator: Default::default(),
                    mutation_added_node_world_inputs_total_by_operator: Default::default(),
                    vm_live_read_world_inputs_current: Default::default(),
                    mutation_events_applied_total_semantic_noop: 0,
                    mutation_events_applied_total_semantic_change: 0,
                    mutation_target_reachability_total: Default::default(),
                    mutation_value_totals_by_operator: Default::default(),
                    mutation_outcome_summary: Default::default(),
                    reproduction_actions_attempted_total: 0,
                    reproduction_actions_spawned_total: 0,
                    reproduction_actions_rejected_total: 0,
                    reproduction_actions_rejected_total_by_reason: Default::default(),
                    reproduction_actions_rejected_invalid_target_total_by_cause: Default::default(),
                    reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state:
                        Default::default(),
                    mutation_events_skipped_total_by_reason: Default::default(),
                    move_actions_blocked_total_by_cause: Default::default(),
                    move_actions_blocked_avoidable_total_by_reader_state: Default::default(),
                    move_attempts_with_barrier_neighbor_total_by_reader_state: Default::default(),
                    move_blocked_barrier_with_barrier_neighbor_total_by_reader_state:
                        Default::default(),
                    reproduction_attempts_with_barrier_neighbor_total_by_reader_state:
                        Default::default(),
                    reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state:
                        Default::default(),
                    predation_actions_attempted_total: 0,
                    predation_actions_transferred_total: 0,
                    predation_actions_rejected_total: 0,
                    predation_kills_total: 0,
                    predation_actions_by_result: Default::default(),
                    genome_complexity_mean: 0.0,
                    genome_complexity_min: 0,
                    genome_complexity_max: 0,
                },
                predation_events: Vec::new(),
            },
        )
    }

    #[test]
    fn detail_frame_is_clipped_to_subscription_rect() {
        let snapshot = sample_snapshot();
        let subscription = ViewSubscription {
            request_id: 1,
            x: 0,
            y: 0,
            width: 8,
            height: 8,
            canvas_width: 320,
            canvas_height: 240,
            zoom_tier: ZoomTier::Detail,
        };

        let detail = assemble_detail_payload(&snapshot, &subscription);

        assert_eq!(detail.width, 8);
        assert_eq!(detail.height, 8);
        assert_eq!(detail.creatures.len(), 1);
        assert_eq!(detail.creatures[0].id, 1);
        assert_eq!(detail.food.len(), 3);
        assert_eq!(
            detail
                .food
                .iter()
                .filter(|cell| cell.x == 2 && cell.y == 2)
                .count(),
            2
        );
        assert!(detail.food.iter().any(|cell| cell.type_idx == 0));
        assert!(detail.food.iter().any(|cell| cell.type_idx == 1));
    }

    #[test]
    fn overview_grid_is_capped() {
        let snapshot = sample_snapshot();
        let subscription = ViewSubscription {
            request_id: 1,
            x: 0,
            y: 0,
            width: 256,
            height: 256,
            canvas_width: 800,
            canvas_height: 600,
            zoom_tier: ZoomTier::Overview,
        };

        let overview = assemble_overview_payload(&snapshot, &subscription);

        assert_eq!(overview.grid_width, 128);
        assert_eq!(overview.grid_height, 128);
        assert_eq!(overview.food.len(), 2);
        assert!(overview.food.iter().any(|cell| cell.bucket_x == 1
            && cell.bucket_y == 1
            && cell.type_idx == 0
            && cell.density > 1.0));
        assert!(overview
            .food
            .iter()
            .any(|cell| cell.bucket_x == 1 && cell.bucket_y == 1 && cell.type_idx == 1));
        assert_eq!(overview.creature_count_u16.len(), 128 * 128);
    }
}
