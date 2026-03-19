//! Transitional view-assembly boundary.

use crate::query::projection::ProjectionSnapshot;
use crate::query::spatial_index::ViewRect;
use crate::transport::protocol::{ViewDetailPayload, ViewOverviewPayload, ViewRectPayload};
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

    let rect_cells = rect.width as usize * rect.height as usize;
    let mut food_density_u8 = Vec::with_capacity(rect_cells);
    let mut food_fertility_u8 = Vec::with_capacity(rect_cells);
    let world_width = snapshot.ws_frame.frame.width;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            let index = y as usize * world_width as usize + x as usize;
            debug_assert!(
                index < snapshot.food_density_u8.len(),
                "detail food density index should stay within the clamped world bounds"
            );
            food_density_u8.push(snapshot.food_density_u8[index]);
            food_fertility_u8.push(snapshot.food_fertility_u8[index]);
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
        food_density_u8,
        food_fertility_u8,
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
    let mut food_density_u8 = vec![0u8; grid_cells];
    let mut food_fertility_u8 = vec![0u8; grid_cells];
    let mut creature_count_u16 = vec![0u16; grid_cells];

    for y in 0..grid_height {
        let world_y = rect.y as usize + y * rect.height as usize / grid_height;
        for x in 0..grid_width {
            let world_x = rect.x as usize + x * rect.width as usize / grid_width;
            let src = world_y * snapshot.ws_frame.frame.width as usize + world_x;
            let dst = y * grid_width + x;
            debug_assert!(
                src < snapshot.food_density_u8.len(),
                "overview food density index should stay within the clamped world bounds"
            );
            food_density_u8[dst] = snapshot.food_density_u8[src];
            food_fertility_u8[dst] = snapshot.food_fertility_u8[src];
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

    ViewOverviewPayload {
        rect: ViewRectPayload::from(rect),
        grid_width: grid_width as u16,
        grid_height: grid_height as u16,
        food_density_u8,
        food_fertility_u8,
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
        BarrierCell, CreatureSnapshot, FoodCell, FramePayload, HealthPayload, LastTickActions,
        SimulationStatus, StatusPayload, WsFrame,
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
                        },
                        CreatureSnapshot {
                            id: 2,
                            x: 200,
                            y: 200,
                            energy: 20.0,
                            generation: 2,
                            phenotype_rgb: [4, 5, 6],
                        },
                    ],
                    food: vec![
                        FoodCell {
                            x: 2,
                            y: 2,
                            density: 1.0,
                        },
                        FoodCell {
                            x: 140,
                            y: 140,
                            density: 0.5,
                        },
                    ],
                    barriers: vec![BarrierCell { x: 3, y: 3 }, BarrierCell { x: 180, y: 180 }],
                    food_fertility_u8: vec![128u8; 256 * 256].into_boxed_slice(),
                },
                health: HealthPayload {
                    population: 2,
                    mean_energy: 5.0,
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
        assert_eq!(
            detail
                .food_density_u8
                .iter()
                .filter(|value| **value > 0)
                .count(),
            1
        );
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
        assert_eq!(overview.food_density_u8.len(), 128 * 128);
        assert_eq!(overview.creature_count_u16.len(), 128 * 128);
    }
}
