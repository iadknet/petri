pub mod action_queue;
mod actions;
mod direction;
mod food_type_id;
mod hash;
mod ids;
mod inputs;
mod position;
pub mod routing;

pub use action_queue::ActionQueue;
pub use actions::{ActionType, WorldAction, ACTION_TYPE_COUNT};
pub use direction::{compass_x, compass_y, Direction};
pub use food_type_id::OrdinaryFoodTypeId;
pub use hash::splitmix64;
pub use ids::{CreatureId, NodeId};
#[cfg(test)]
pub(crate) use inputs::world_key_strategy;
pub use inputs::{
    barrier_idx, food_idx, nearby_idx, occupancy_idx, DirectionPair, DynamicIntrospectionKey,
    InputReference, NearbyField, PerceptionRequirements, StaticIntrospectionKey, WorldInputKey,
};
pub use position::Position;
pub use routing::{RouteTarget, MAX_GATE_SLOTS};

/// Number of f32 slots in a creature's shared memory, read and written by
/// both VM and Graph backends.
pub const SHARED_MEMORY_SLOTS: usize = 16;
