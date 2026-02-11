use crate::types::{ControllerPalette, Edge, NodeKind};

use super::ComputationGraph;

pub(super) fn neural_only() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::NeuralOnly,
        nodes: vec![
            NodeKind::InputFoodHere,   // 0
            NodeKind::InputEnergy,     // 1
            NodeKind::InputRandom,     // 2
            NodeKind::Add,             // 3
            NodeKind::Tanh,            // 4
            NodeKind::Add,             // 5
            NodeKind::Tanh,            // 6
            NodeKind::OutputMoveX,     // 7
            NodeKind::OutputMoveY,     // 8
            NodeKind::Sigmoid,         // 9
            NodeKind::OutputEat,       // 10
            NodeKind::Sigmoid,         // 11
            NodeKind::OutputReproduce, // 12
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 3,
                weight: 0.6,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 5,
                weight: -0.5,
            },
            Edge {
                from: 5,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 9,
                weight: 2.0,
            },
            Edge {
                from: 9,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 11,
                weight: 2.0,
            },
            Edge {
                from: 11,
                to: 12,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn logic_only() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::LogicOnly,
        nodes: vec![
            NodeKind::InputFoodHere,   // 0
            NodeKind::InputEnergy,     // 1
            NodeKind::InputRandom,     // 2
            NodeKind::Threshold(0.4),  // 3
            NodeKind::Threshold(0.8),  // 4
            NodeKind::OutputEat,       // 5
            NodeKind::OutputReproduce, // 6
            NodeKind::Threshold(0.0),  // 7
            NodeKind::Constant(-1.0),  // 8
            NodeKind::Constant(1.0),   // 9
            NodeKind::Select,          // 10
            NodeKind::OutputMoveX,     // 11
            NodeKind::OutputMoveY,     // 12
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 8,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 9,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 10,
                to: 12,
                weight: -1.0,
            },
        ],
    }
}

pub(super) fn hybrid() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputFoodHere,   // 0
            NodeKind::InputEnergy,     // 1
            NodeKind::InputRandom,     // 2
            NodeKind::Add,             // 3
            NodeKind::Tanh,            // 4
            NodeKind::Add,             // 5
            NodeKind::Tanh,            // 6
            NodeKind::Threshold(0.25), // 7
            NodeKind::Sigmoid,         // 8
            NodeKind::OutputMoveX,     // 9
            NodeKind::OutputMoveY,     // 10
            NodeKind::OutputEat,       // 11
            NodeKind::OutputReproduce, // 12
        ],
        edges: vec![
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 3,
                weight: 0.5,
            },
            Edge {
                from: 3,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 9,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 5,
                weight: -1.0,
            },
            Edge {
                from: 1,
                to: 5,
                weight: 0.7,
            },
            Edge {
                from: 5,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 8,
                weight: 1.4,
            },
            Edge {
                from: 0,
                to: 8,
                weight: 0.6,
            },
            Edge {
                from: 8,
                to: 12,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn founder_neural_only() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::NeuralOnly,
        nodes: vec![
            NodeKind::InputFoodHere,   // 0
            NodeKind::InputEnergy,     // 1
            NodeKind::InputRandom,     // 2
            NodeKind::Constant(-2.5),  // 3
            NodeKind::Add,             // 4
            NodeKind::Sigmoid,         // 5
            NodeKind::OutputEat,       // 6
            NodeKind::Constant(-7.0),  // 7
            NodeKind::Add,             // 8
            NodeKind::Sigmoid,         // 9
            NodeKind::OutputReproduce, // 10
            NodeKind::Tanh,            // 11
            NodeKind::OutputMoveX,     // 12
            NodeKind::Tanh,            // 13
            NodeKind::OutputMoveY,     // 14
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 4,
                weight: 6.0,
            },
            Edge {
                from: 3,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 5,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 8,
                weight: 8.0,
            },
            Edge {
                from: 0,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 8,
                to: 9,
                weight: 1.0,
            },
            Edge {
                from: 9,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 11,
                weight: 0.4,
            },
            Edge {
                from: 11,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 13,
                weight: -0.4,
            },
            Edge {
                from: 13,
                to: 14,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn founder_logic_only() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::LogicOnly,
        nodes: vec![
            NodeKind::InputFoodHere,   // 0
            NodeKind::InputEnergy,     // 1
            NodeKind::InputRandom,     // 2
            NodeKind::Threshold(0.05), // 3
            NodeKind::OutputEat,       // 4
            NodeKind::Threshold(0.9),  // 5
            NodeKind::Multiply,        // 6
            NodeKind::OutputReproduce, // 7
            NodeKind::Threshold(0.7),  // 8
            NodeKind::Constant(0.0),   // 9
            NodeKind::Constant(1.0),   // 10
            NodeKind::Select,          // 11
            NodeKind::OutputMoveX,     // 12
            NodeKind::OutputMoveY,     // 13
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 5,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 8,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 9,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 11,
                to: 12,
                weight: 0.35,
            },
            Edge {
                from: 11,
                to: 13,
                weight: 0.0,
            },
        ],
    }
}

pub(super) fn founder_hybrid() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputFoodHere,            // 0
            NodeKind::InputEnergy,              // 1
            NodeKind::InputRandom,              // 2
            NodeKind::InputFoodDirection,       // 3
            NodeKind::InputFoodDistance,        // 4
            NodeKind::InputCreatureDirection,   // 5
            NodeKind::InputCreatureDistance,    // 6
            NodeKind::InputLocalDensity,        // 7
            NodeKind::InputMoveBlockedLastTick, // 8
            NodeKind::Threshold(0.08),          // 9
            NodeKind::OutputEat,                // 10
            NodeKind::Constant(-7.0),           // 11
            NodeKind::Add,                      // 12
            NodeKind::Sigmoid,                  // 13
            NodeKind::OutputReproduce,          // 14
            NodeKind::Add,                      // 15
            NodeKind::Tanh,                     // 16
            NodeKind::OutputMoveX,              // 17
            NodeKind::Add,                      // 18
            NodeKind::Tanh,                     // 19
            NodeKind::OutputMoveY,              // 20
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 9,
                weight: 1.0,
            },
            Edge {
                from: 9,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 12,
                weight: 8.0,
            },
            Edge {
                from: 0,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 12,
                weight: -2.0,
            },
            Edge {
                from: 11,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 12,
                to: 13,
                weight: 1.0,
            },
            Edge {
                from: 13,
                to: 14,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 15,
                weight: 0.4,
            },
            Edge {
                from: 3,
                to: 15,
                weight: 0.35,
            },
            Edge {
                from: 5,
                to: 15,
                weight: -0.25,
            },
            Edge {
                from: 8,
                to: 15,
                weight: -0.8,
            },
            Edge {
                from: 15,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 16,
                to: 17,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 18,
                weight: -0.4,
            },
            Edge {
                from: 4,
                to: 18,
                weight: -0.35,
            },
            Edge {
                from: 6,
                to: 18,
                weight: 0.25,
            },
            Edge {
                from: 8,
                to: 18,
                weight: 0.8,
            },
            Edge {
                from: 18,
                to: 19,
                weight: 1.0,
            },
            Edge {
                from: 19,
                to: 20,
                weight: 1.0,
            },
        ],
    }
}
