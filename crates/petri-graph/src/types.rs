use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerPalette {
    NeuralOnly,
    LogicOnly,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SensorInputs {
    pub food_here: f32,
    pub energy: f32,
    pub random: f32,
    pub food_direction: f32,
    pub food_distance: f32,
    pub creature_direction: f32,
    pub creature_distance: f32,
    pub local_density: f32,
    pub move_blocked_last_tick: f32,
    pub memory_read: f32,
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
            move_blocked_last_tick: 0.0,
            memory_read: 0.0,
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
    InputMoveBlockedLastTick,
    InputMemoryRead,
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
}
