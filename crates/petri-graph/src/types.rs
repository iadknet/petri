use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerPalette {
    NeuralOnly,
    LogicOnly,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct SensorInputs {
    pub food_here: f32,
    pub energy: f32,
    pub random: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ActionOutputs {
    pub move_x: f32,
    pub move_y: f32,
    pub eat: f32,
    pub reproduce: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeKind {
    InputFoodHere,
    InputEnergy,
    InputRandom,
    Constant(f32),
    Add,
    Multiply,
    Threshold(f32),
    GreaterThan,
    Sigmoid,
    Tanh,
    Select,
    OutputMoveX,
    OutputMoveY,
    OutputEat,
    OutputReproduce,
}
