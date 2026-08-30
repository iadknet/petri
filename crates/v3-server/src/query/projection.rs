//! Projection-backed read model for transitional server refactors.

use crate::query::cache::{build_barrier_mask, build_food_density_planes};
use crate::query::spatial_index::{CreatureTileIndex, DEFAULT_TILE_SIZE};
use crate::state::{build_ws_frame, SimHandle, WsFrame};

/// Published query-side snapshot used by transitional projection-backed reads.
#[derive(Clone, Debug)]
pub struct ProjectionSnapshot {
    pub projection_revision: u64,
    pub world_static_revision: u64,
    pub ws_frame: WsFrame,
    pub food_density_planes: Box<[f32]>,
    pub food_fertility_u8: std::sync::Arc<[u8]>,
    pub barrier_mask: Box<[u8]>,
    pub creature_tile_index: CreatureTileIndex,
}

/// Mutable store for the latest published projection snapshot.
#[derive(Debug)]
pub struct ProjectionStore {
    projection_revision: u64,
    world_static_revision: u64,
    snapshot: ProjectionSnapshot,
}

/// Revisions and static-content change information produced by a projection publish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "projection publications must be forwarded to interested transport sessions"]
pub(crate) struct ProjectionPublication {
    pub(crate) projection_revision: u64,
    pub(crate) world_static_changed: bool,
}

impl ProjectionStore {
    /// Build an initial projection from the provided simulation handle.
    #[must_use]
    pub fn from_handle(handle: &SimHandle) -> Self {
        Self::new(build_ws_frame(handle))
    }

    /// Build an initial projection from a materialized websocket frame.
    #[must_use]
    pub fn new(frame: WsFrame) -> Self {
        let snapshot = ProjectionSnapshot::from_ws_frame(1, 1, frame);

        Self {
            projection_revision: snapshot.projection_revision,
            world_static_revision: snapshot.world_static_revision,
            snapshot,
        }
    }

    /// Publish a new projection from an already-built websocket frame.
    pub(crate) fn publish_ws_frame(
        &mut self,
        frame: WsFrame,
        force_world_static_invalidation: bool,
    ) -> ProjectionPublication {
        self.projection_revision += 1;
        let mut next_snapshot = ProjectionSnapshot::from_ws_frame(
            self.projection_revision,
            self.world_static_revision,
            frame,
        );
        let world_static_changed =
            force_world_static_invalidation || !same_world_static(&self.snapshot, &next_snapshot);
        if world_static_changed {
            self.world_static_revision += 1;
            next_snapshot.world_static_revision = self.world_static_revision;
        }
        self.snapshot = next_snapshot;

        ProjectionPublication {
            projection_revision: self.snapshot.projection_revision,
            world_static_changed,
        }
    }

    /// Borrow the currently published snapshot.
    #[must_use]
    pub fn current(&self) -> &ProjectionSnapshot {
        &self.snapshot
    }
}

impl ProjectionSnapshot {
    #[must_use]
    pub fn from_ws_frame(
        projection_revision: u64,
        world_static_revision: u64,
        ws_frame: WsFrame,
    ) -> Self {
        let food_density_planes = build_food_density_planes(&ws_frame.frame);
        let food_fertility_u8 = ws_frame.frame.food_fertility_u8.clone();
        let barrier_mask = build_barrier_mask(&ws_frame.frame);
        let creature_tile_index =
            CreatureTileIndex::build(DEFAULT_TILE_SIZE, &ws_frame.frame.creatures);

        Self {
            projection_revision,
            world_static_revision,
            food_density_planes,
            food_fertility_u8,
            barrier_mask,
            creature_tile_index,
            ws_frame,
        }
    }
}

fn same_world_static(current: &ProjectionSnapshot, next: &ProjectionSnapshot) -> bool {
    current.ws_frame.frame.width == next.ws_frame.frame.width
        && current.ws_frame.frame.height == next.ws_frame.frame.height
        && current.ws_frame.frame.food_types == next.ws_frame.frame.food_types
        && current.barrier_mask == next.barrier_mask
        && current.food_fertility_u8 == next.food_fertility_u8
}

#[cfg(test)]
mod tests {
    use crate::state::{
        BarrierCell, CreatureSnapshot, FoodCell, FoodTypeSnapshot, FramePayload, HealthPayload,
        LastTickActions, SimulationStatus, StatusPayload, WsFrame,
    };

    use super::{ProjectionSnapshot, ProjectionStore};

    fn sample_ws_frame() -> WsFrame {
        WsFrame {
            tick: 12,
            status: StatusPayload {
                state: SimulationStatus::Paused,
                population: 1,
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
                width: 2,
                height: 2,
                creatures: vec![CreatureSnapshot {
                    id: 1,
                    x: 0,
                    y: 0,
                    energy: 10.0,
                    generation: 1,
                    phenotype_rgb: [1, 2, 3],
                }],
                food_types: vec![
                    FoodTypeSnapshot {
                        type_idx: 0,
                        name: "Primary".to_string(),
                        color: "#22c55e".to_string(),
                        growth_inhibitor: 0.2,
                    },
                    FoodTypeSnapshot {
                        type_idx: 1,
                        name: "Secondary".to_string(),
                        color: "#0ea5e9".to_string(),
                        growth_inhibitor: 0.3,
                    },
                ],
                food: vec![
                    FoodCell {
                        x: 0,
                        y: 0,
                        type_idx: 0,
                        density: 0.5,
                    },
                    FoodCell {
                        x: 1,
                        y: 1,
                        type_idx: 1,
                        density: 1.0,
                    },
                ],
                barriers: vec![BarrierCell { x: 1, y: 0 }],
                food_fertility_u8: vec![7u8; 4].into(),
            },
            health: HealthPayload {
                population: 1,
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
                move_blocked_barrier_with_barrier_neighbor_total_by_reader_state: Default::default(
                ),
                reproduction_attempts_with_barrier_neighbor_total_by_reader_state: Default::default(
                ),
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
        }
    }

    #[test]
    fn projection_snapshot_builds_typed_density_planes() {
        let snapshot = ProjectionSnapshot::from_ws_frame(7, 3, sample_ws_frame());

        assert_eq!(snapshot.food_density_planes.len(), 8);
        assert_eq!(&snapshot.food_density_planes[..4], &[0.5, 0.0, 0.0, 0.0]);
        assert_eq!(&snapshot.food_density_planes[4..], &[0.0, 0.0, 0.0, 1.0]);
        assert_eq!(snapshot.barrier_mask.len(), 1);
        assert_eq!(snapshot.food_fertility_u8.len(), 4);
    }

    #[test]
    fn world_static_revision_changes_when_growth_inhibitor_changes() {
        let initial = sample_ws_frame();
        let mut updated = sample_ws_frame();
        updated.frame.food_types[1].growth_inhibitor = 0.6;

        let mut store = ProjectionStore::new(initial);
        let publication = store.publish_ws_frame(updated, false);

        assert_eq!(publication.projection_revision, 2);
        assert!(publication.world_static_changed);
        assert_eq!(store.current().world_static_revision, 2);
    }

    #[test]
    fn forced_world_static_invalidation_advances_static_revision_for_identical_content() {
        // Arrange
        let mut store = ProjectionStore::new(sample_ws_frame());

        // Act
        let publication = store.publish_ws_frame(sample_ws_frame(), true);

        // Assert
        assert_eq!(publication.projection_revision, 2);
        assert_eq!(store.current().world_static_revision, 2);
        assert!(publication.world_static_changed);
    }
}
