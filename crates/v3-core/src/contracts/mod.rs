pub mod action_queue;
mod actions;
mod brain_io;
mod direction;
mod food_type_id;
mod hash;
mod ids;
mod inputs;
mod position;
pub mod routing;
pub mod votes;

pub use action_queue::ActionQueue;
pub use actions::{ActionOutcome, ActionResult, ActionType, WorldAction, ACTION_TYPE_COUNT};
pub use brain_io::{OutcomeChannel, OUTCOME_CHANNEL_COUNT, OUTPUT_SLOT_COUNT};
pub use direction::{compass_x, compass_y, Direction};
pub use food_type_id::OrdinaryFoodTypeId;
pub use hash::splitmix64;
pub use ids::{CreatureId, NodeId};
pub use inputs::{
    barrier_idx, food_idx, nearby_idx, occupancy_idx, DirectionPair, DynamicIntrospectionKey,
    FieldSet, InputReference, NearbyField, PerceptionRequirements, SensorContext, SensorDescriptor,
    StaticIntrospectionKey, WorldInputKey, MEMORY_BANK,
};
#[cfg(test)]
pub(crate) use inputs::{pair_layout, world_key_strategy};
pub use position::Position;
pub use routing::{RouteTarget, MAX_GATE_SLOTS};

/// Number of f32 slots in a creature's shared memory, read and written by
/// both VM and Graph backends.
pub const SHARED_MEMORY_SLOTS: usize = 16;
