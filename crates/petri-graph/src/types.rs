use serde::{Deserialize, Serialize};

pub const TOUCH_DIRECTION_COUNT: usize = 5;
pub const SLOT_COUNT_MAX: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerPalette {
    NeuralOnly,
    LogicOnly,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SensorInputs {
    pub food_here: f32,
    pub energy: f32,
    pub random: f32,
    pub food_direction: f32,
    pub food_distance: f32,
    pub creature_direction: f32,
    pub creature_distance: f32,
    pub local_density: f32,
    pub barrier_direction: f32,
    pub barrier_distance: f32,
    pub move_blocked_last_tick: f32,
    pub memory_read: f32,
    pub touch_exists: [f32; TOUCH_DIRECTION_COUNT],
    pub touch_food_value: [f32; TOUCH_DIRECTION_COUNT],
    pub touch_has_barrier: [f32; TOUCH_DIRECTION_COUNT],
    pub touch_occupied: [f32; TOUCH_DIRECTION_COUNT],
    pub slot_exists: [f32; SLOT_COUNT_MAX],
    pub slot_is_empty: [f32; SLOT_COUNT_MAX],
    pub slot_is_barrier: [f32; SLOT_COUNT_MAX],
    pub slot_food_value: [f32; SLOT_COUNT_MAX],
}

impl Default for SensorInputs {
    fn default() -> Self {
        Self {
            food_here: 0.0,
            energy: 0.0,
            random: 0.0,
            food_direction: 0.0,
            food_distance: 1.0,
            creature_direction: 0.0,
            creature_distance: 1.0,
            local_density: 0.0,
            barrier_direction: 0.0,
            barrier_distance: 1.0,
            move_blocked_last_tick: 0.0,
            memory_read: 0.0,
            touch_exists: [0.0; TOUCH_DIRECTION_COUNT],
            touch_food_value: [0.0; TOUCH_DIRECTION_COUNT],
            touch_has_barrier: [0.0; TOUCH_DIRECTION_COUNT],
            touch_occupied: [0.0; TOUCH_DIRECTION_COUNT],
            slot_exists: [0.0; SLOT_COUNT_MAX],
            slot_is_empty: [0.0; SLOT_COUNT_MAX],
            slot_is_barrier: [0.0; SLOT_COUNT_MAX],
            slot_food_value: [0.0; SLOT_COUNT_MAX],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ActionOutputs {
    pub move_x: f32,
    pub move_y: f32,
    pub eat: f32,
    pub reproduce: f32,
    pub memory_write: f32,
    pub inventory_pickup: f32,
    pub inventory_put: f32,
    pub inventory_slot_select: f32,
    pub inventory_direction_select: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    InputFoodHere,
    InputEnergy,
    InputRandom,
    InputFoodDirection,
    InputFoodDistance,
    InputCreatureDirection,
    InputCreatureDistance,
    InputLocalDensity,
    InputBarrierDirection,
    InputBarrierDistance,
    InputMoveBlockedLastTick,
    InputMemoryRead,
    InputTouchExists(u8),
    InputTouchFoodValue(u8),
    InputTouchHasBarrier(u8),
    InputTouchOccupied(u8),
    InputSlotExists(u8),
    InputSlotIsEmpty(u8),
    InputSlotIsBarrier(u8),
    InputSlotFoodValue(u8),
    Constant(f32),
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    Threshold(f32),
    GreaterThan,
    Sigmoid,
    Tanh,
    Relu,
    Select,
    OutputMoveX,
    OutputMoveY,
    OutputEat,
    OutputReproduce,
    OutputMemoryWrite,
    OutputInventoryPickup,
    OutputInventoryPut,
    OutputInventorySlotSelect,
    OutputInventoryDirectionSelect,
}
