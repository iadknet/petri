//! Projection-backed read model for transitional server refactors.

use crate::handlers::lifecycle::build_ws_frame;
use crate::query::cache::{build_barrier_mask, build_food_density_u8};
use crate::query::spatial_index::{CreatureTileIndex, DEFAULT_TILE_SIZE};
use crate::state::{SimHandle, WsFrame};

/// Published query-side snapshot used by transitional projection-backed reads.
#[derive(Clone, Debug)]
pub struct ProjectionSnapshot {
    pub projection_revision: u64,
    pub world_static_revision: u64,
    pub ws_frame: WsFrame,
    pub food_density_u8: Box<[u8]>,
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

    /// Publish a new projection by rebuilding a websocket frame from the simulation handle.
    pub fn publish_from_handle(&mut self, handle: &SimHandle) -> ProjectionSnapshot {
        self.publish_ws_frame(build_ws_frame(handle))
    }

    /// Publish a new projection from an already-built websocket frame.
    pub fn publish_ws_frame(&mut self, frame: WsFrame) -> ProjectionSnapshot {
        self.projection_revision += 1;
        let mut next_snapshot = ProjectionSnapshot::from_ws_frame(
            self.projection_revision,
            self.world_static_revision,
            frame,
        );
        if !same_world_static(&self.snapshot, &next_snapshot) {
            self.world_static_revision += 1;
            next_snapshot.world_static_revision = self.world_static_revision;
        }
        self.snapshot = next_snapshot;

        self.snapshot.clone()
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
        let food_density_u8 = build_food_density_u8(&ws_frame.frame);
        let food_fertility_u8 = ws_frame.frame.food_fertility_u8.clone();
        let barrier_mask = build_barrier_mask(&ws_frame.frame);
        let creature_tile_index =
            CreatureTileIndex::build(DEFAULT_TILE_SIZE, &ws_frame.frame.creatures);

        Self {
            projection_revision,
            world_static_revision,
            food_density_u8,
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
        && current.barrier_mask == next.barrier_mask
        && current.food_fertility_u8 == next.food_fertility_u8
}
