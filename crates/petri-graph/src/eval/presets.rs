use crate::types::{ControllerPalette, Edge, NodeKind};

use super::ComputationGraph;

pub(super) fn neural_only() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::NeuralOnly,
        nodes: vec![
            NodeKind::InputFoodHere,             // 0
            NodeKind::InputEnergy,               // 1
            NodeKind::InputRandom,               // 2
            NodeKind::InputMemoryRead,           // 3
            NodeKind::Add,                       // 4
            NodeKind::Tanh,                      // 5
            NodeKind::Add,                       // 6
            NodeKind::Tanh,                      // 7
            NodeKind::OutputMoveX,               // 8
            NodeKind::OutputMoveY,               // 9
            NodeKind::Sigmoid,                   // 10
            NodeKind::OutputEat,                 // 11
            NodeKind::Sigmoid,                   // 12
            NodeKind::OutputReproduce,           // 13
            NodeKind::OutputMemoryWriteValue,    // 14
            NodeKind::InputMemoryAddressNorm,    // 15
            NodeKind::OutputMemoryWriteEnable,   // 16
            NodeKind::OutputMemoryAddressSelect, // 17
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 4,
                weight: 0.6,
            },
            Edge {
                from: 2,
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
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 6,
                weight: -0.5,
            },
            Edge {
                from: 6,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 9,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 10,
                weight: 2.0,
            },
            Edge {
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 12,
                weight: 2.0,
            },
            Edge {
                from: 12,
                to: 13,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 14,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 15,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 17,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn logic_only() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::LogicOnly,
        nodes: vec![
            NodeKind::InputFoodHere,             // 0
            NodeKind::InputEnergy,               // 1
            NodeKind::InputRandom,               // 2
            NodeKind::InputMemoryRead,           // 3
            NodeKind::Threshold(0.4),            // 4
            NodeKind::Threshold(0.8),            // 5
            NodeKind::OutputEat,                 // 6
            NodeKind::OutputReproduce,           // 7
            NodeKind::Threshold(0.0),            // 8
            NodeKind::Constant(-1.0),            // 9
            NodeKind::Constant(1.0),             // 10
            NodeKind::Select,                    // 11
            NodeKind::OutputMoveX,               // 12
            NodeKind::OutputMoveY,               // 13
            NodeKind::OutputMemoryWriteValue,    // 14
            NodeKind::InputMemoryAddressNorm,    // 15
            NodeKind::OutputMemoryWriteEnable,   // 16
            NodeKind::OutputMemoryAddressSelect, // 17
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 5,
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
                weight: 1.0,
            },
            Edge {
                from: 11,
                to: 13,
                weight: -1.0,
            },
            Edge {
                from: 3,
                to: 14,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 15,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 17,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn hybrid() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputFoodHere,             // 0
            NodeKind::InputEnergy,               // 1
            NodeKind::InputRandom,               // 2
            NodeKind::InputMemoryRead,           // 3
            NodeKind::Add,                       // 4
            NodeKind::Tanh,                      // 5
            NodeKind::Add,                       // 6
            NodeKind::Tanh,                      // 7
            NodeKind::Threshold(0.25),           // 8
            NodeKind::Sigmoid,                   // 9
            NodeKind::OutputMoveX,               // 10
            NodeKind::OutputMoveY,               // 11
            NodeKind::OutputEat,                 // 12
            NodeKind::OutputReproduce,           // 13
            NodeKind::OutputMemoryWriteValue,    // 14
            NodeKind::InputMemoryAddressNorm,    // 15
            NodeKind::OutputMemoryWriteEnable,   // 16
            NodeKind::OutputMemoryAddressSelect, // 17
        ],
        edges: vec![
            Edge {
                from: 2,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 4,
                weight: 0.5,
            },
            Edge {
                from: 4,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 5,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 6,
                weight: -1.0,
            },
            Edge {
                from: 1,
                to: 6,
                weight: 0.7,
            },
            Edge {
                from: 6,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 8,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 9,
                weight: 1.4,
            },
            Edge {
                from: 0,
                to: 9,
                weight: 0.6,
            },
            Edge {
                from: 9,
                to: 13,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 14,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 15,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 17,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn founder_neural_only() -> ComputationGraph {
    let mut nodes = vec![
        NodeKind::InputFoodHere,          // 0
        NodeKind::InputEnergy,            // 1
        NodeKind::InputRandom,            // 2
        NodeKind::InputMemoryRead,        // 3
        NodeKind::Constant(-2.5),         // 4
        NodeKind::Add,                    // 5
        NodeKind::Sigmoid,                // 6
        NodeKind::OutputEat,              // 7
        NodeKind::Constant(-7.0),         // 8
        NodeKind::Add,                    // 9
        NodeKind::Sigmoid,                // 10
        NodeKind::OutputReproduce,        // 11
        NodeKind::Tanh,                   // 12
        NodeKind::OutputMoveX,            // 13
        NodeKind::Tanh,                   // 14
        NodeKind::OutputMoveY,            // 15
        NodeKind::OutputMemoryWriteValue, // 16
    ];
    nodes.push(NodeKind::InputMemoryAddressNorm);
    nodes.push(NodeKind::OutputMemoryWriteEnable);
    nodes.push(NodeKind::OutputMemoryAddressSelect);
    nodes.push(NodeKind::OutputInventoryPickup);
    nodes.push(NodeKind::OutputInventoryPut);
    nodes.push(NodeKind::OutputInventorySlotSelect);
    nodes.push(NodeKind::OutputInventoryDirectionSelect);

    ComputationGraph {
        palette: ControllerPalette::NeuralOnly,
        nodes,
        edges: vec![
            Edge {
                from: 0,
                to: 5,
                weight: 6.0,
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
                from: 6,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 9,
                weight: 8.0,
            },
            Edge {
                from: 0,
                to: 9,
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
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 12,
                weight: 0.4,
            },
            Edge {
                from: 12,
                to: 13,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 14,
                weight: -0.4,
            },
            Edge {
                from: 14,
                to: 15,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 16,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 18,
                weight: 1.0,
            },
            Edge {
                from: 17,
                to: 18,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 19,
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn founder_logic_only() -> ComputationGraph {
    let mut nodes = vec![
        NodeKind::InputFoodHere,          // 0
        NodeKind::InputEnergy,            // 1
        NodeKind::InputRandom,            // 2
        NodeKind::InputMemoryRead,        // 3
        NodeKind::Threshold(0.05),        // 4
        NodeKind::OutputEat,              // 5
        NodeKind::Threshold(0.9),         // 6
        NodeKind::Multiply,               // 7
        NodeKind::OutputReproduce,        // 8
        NodeKind::Threshold(0.7),         // 9
        NodeKind::Constant(0.0),          // 10
        NodeKind::Constant(1.0),          // 11
        NodeKind::Select,                 // 12
        NodeKind::OutputMoveX,            // 13
        NodeKind::OutputMoveY,            // 14
        NodeKind::OutputMemoryWriteValue, // 15
    ];
    nodes.push(NodeKind::InputMemoryAddressNorm);
    nodes.push(NodeKind::OutputMemoryWriteEnable);
    nodes.push(NodeKind::OutputMemoryAddressSelect);
    nodes.push(NodeKind::OutputInventoryPickup);
    nodes.push(NodeKind::OutputInventoryPut);
    nodes.push(NodeKind::OutputInventorySlotSelect);
    nodes.push(NodeKind::OutputInventoryDirectionSelect);

    ComputationGraph {
        palette: ControllerPalette::LogicOnly,
        nodes,
        edges: vec![
            Edge {
                from: 0,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 9,
                weight: 1.0,
            },
            Edge {
                from: 9,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 10,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 11,
                to: 12,
                weight: 1.0,
            },
            Edge {
                from: 12,
                to: 13,
                weight: 0.35,
            },
            Edge {
                from: 12,
                to: 14,
                weight: 0.0,
            },
            Edge {
                from: 3,
                to: 15,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 17,
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
                weight: 1.0,
            },
        ],
    }
}

pub(super) fn founder_hybrid() -> ComputationGraph {
    let mut nodes = vec![
        NodeKind::InputFoodHere,            // 0
        NodeKind::InputEnergy,              // 1
        NodeKind::InputRandom,              // 2
        NodeKind::InputFoodDirection,       // 3
        NodeKind::InputFoodDistance,        // 4
        NodeKind::InputCreatureDirection,   // 5
        NodeKind::InputCreatureDistance,    // 6
        NodeKind::InputLocalDensity,        // 7
        NodeKind::InputMoveBlockedLastTick, // 8
        NodeKind::InputMemoryRead,          // 9
        NodeKind::Threshold(0.08),          // 10
        NodeKind::OutputEat,                // 11
        NodeKind::Constant(-7.0),           // 12
        NodeKind::Add,                      // 13
        NodeKind::Sigmoid,                  // 14
        NodeKind::OutputReproduce,          // 15
        NodeKind::Add,                      // 16
        NodeKind::Tanh,                     // 17
        NodeKind::OutputMoveX,              // 18
        NodeKind::Add,                      // 19
        NodeKind::Tanh,                     // 20
        NodeKind::OutputMoveY,              // 21
        NodeKind::OutputMemoryWriteValue,   // 22
        NodeKind::InputBarrierDirection,    // 23
        NodeKind::InputBarrierDistance,     // 24
        NodeKind::InputMemoryAddressNorm,   // 25
    ];
    for index in 0_u8..5 {
        nodes.push(NodeKind::InputTouchExists(index));
        nodes.push(NodeKind::InputTouchFoodValue(index));
        nodes.push(NodeKind::InputTouchHasBarrier(index));
        nodes.push(NodeKind::InputTouchOccupied(index));
    }
    for index in 0_u8..12 {
        nodes.push(NodeKind::InputSlotExists(index));
        nodes.push(NodeKind::InputSlotIsEmpty(index));
        nodes.push(NodeKind::InputSlotIsBarrier(index));
        nodes.push(NodeKind::InputSlotFoodValue(index));
    }
    nodes.push(NodeKind::OutputMemoryWriteEnable);
    nodes.push(NodeKind::OutputMemoryAddressSelect);
    nodes.push(NodeKind::OutputInventoryPickup);
    nodes.push(NodeKind::OutputInventoryPut);
    nodes.push(NodeKind::OutputInventorySlotSelect);
    nodes.push(NodeKind::OutputInventoryDirectionSelect);

    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes,
        edges: vec![
            Edge {
                from: 0,
                to: 10,
                weight: 1.0,
            },
            Edge {
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 13,
                weight: 8.0,
            },
            Edge {
                from: 0,
                to: 13,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 13,
                weight: 1.0,
            },
            Edge {
                from: 7,
                to: 13,
                weight: -2.0,
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
                from: 14,
                to: 15,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 16,
                weight: 0.4,
            },
            Edge {
                from: 3,
                to: 16,
                weight: 0.35,
            },
            Edge {
                from: 5,
                to: 16,
                weight: -0.25,
            },
            Edge {
                from: 8,
                to: 16,
                weight: -0.8,
            },
            Edge {
                from: 16,
                to: 17,
                weight: 1.0,
            },
            Edge {
                from: 17,
                to: 18,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 19,
                weight: -0.4,
            },
            Edge {
                from: 4,
                to: 19,
                weight: -0.35,
            },
            Edge {
                from: 6,
                to: 19,
                weight: 0.25,
            },
            Edge {
                from: 8,
                to: 19,
                weight: 0.8,
            },
            Edge {
                from: 19,
                to: 20,
                weight: 1.0,
            },
            Edge {
                from: 20,
                to: 21,
                weight: 1.0,
            },
            Edge {
                from: 23,
                to: 16,
                weight: -0.4,
            },
            Edge {
                from: 24,
                to: 19,
                weight: -0.6,
            },
            Edge {
                from: 9,
                to: 22,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 94,
                weight: 1.0,
            },
            Edge {
                from: 25,
                to: 94,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 95,
                weight: 1.0,
            },
        ],
    }
}
