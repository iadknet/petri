use petri_graph::{ComputationGraph, ControllerPalette, Edge, NodeKind, SensorInputs};

#[test]
fn relu_and_food_distance_direction_inputs_are_supported() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputFoodDirection, // 0
            NodeKind::InputFoodDistance,  // 1
            NodeKind::Add,                // 2
            NodeKind::Relu,               // 3
            NodeKind::OutputMoveX,        // 4
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 2,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 2,
                weight: -1.0,
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
        ],
    };

    let positive = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.0,
        random: 0.0,
        food_direction: 0.9,
        food_distance: 0.2,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        barrier_direction: 0.0,
        barrier_distance: 1.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
        ..SensorInputs::default()
    });
    assert!(positive.move_x > 0.0);

    let zeroed = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.0,
        random: 0.0,
        food_direction: -0.4,
        food_distance: 0.6,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        barrier_direction: 0.0,
        barrier_distance: 1.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
        ..SensorInputs::default()
    });
    assert_eq!(zeroed.move_x, 0.0);
}

#[test]
fn new_input_nodes_are_supported_and_clamped() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputCreatureDirection,   // 0
            NodeKind::OutputMoveX,              // 1
            NodeKind::InputCreatureDistance,    // 2
            NodeKind::OutputEat,                // 3
            NodeKind::InputLocalDensity,        // 4
            NodeKind::OutputReproduce,          // 5
            NodeKind::InputMoveBlockedLastTick, // 6
            NodeKind::OutputMoveY,              // 7
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 7,
                weight: -1.0,
            },
        ],
    };

    let outputs = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.0,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 2.0,
        creature_distance: -0.5,
        local_density: 1.8,
        barrier_direction: 0.0,
        barrier_distance: 1.0,
        move_blocked_last_tick: 3.0,
        memory_read: 0.0,
        ..SensorInputs::default()
    });

    assert_eq!(outputs.move_x, 1.0);
    assert_eq!(outputs.eat, 0.0);
    assert_eq!(outputs.reproduce, 1.0);
    assert_eq!(outputs.move_y, -1.0);
}

#[test]
fn barrier_input_nodes_are_supported_and_clamped() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputBarrierDirection, // 0
            NodeKind::OutputMoveX,           // 1
            NodeKind::InputBarrierDistance,  // 2
            NodeKind::OutputEat,             // 3
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 1.0,
            },
        ],
    };

    let outputs = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.0,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        barrier_direction: 2.3,
        barrier_distance: -0.4,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
        ..SensorInputs::default()
    });

    assert_eq!(outputs.move_x, 1.0);
    assert_eq!(outputs.eat, 0.0);
}

#[test]
fn negate_abs_min_and_max_node_kinds_are_supported() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::Constant(2.0),   // 0
            NodeKind::Constant(-3.0),  // 1
            NodeKind::Negate,          // 2
            NodeKind::Abs,             // 3
            NodeKind::Min,             // 4
            NodeKind::Max,             // 5
            NodeKind::OutputMoveX,     // 6
            NodeKind::OutputMoveY,     // 7
            NodeKind::OutputEat,       // 8
            NodeKind::OutputReproduce, // 9
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 2,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 2,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 4,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 6,
                weight: 1.0,
            },
            Edge {
                from: 3,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 4,
                to: 8,
                weight: 1.0,
            },
            Edge {
                from: 5,
                to: 9,
                weight: 1.0,
            },
        ],
    };

    let outputs = graph.evaluate(SensorInputs {
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
        ..SensorInputs::default()
    });

    assert_eq!(outputs.move_x, 1.0);
    assert_eq!(outputs.move_y, 1.0);
    assert_eq!(outputs.eat, 0.0);
    assert_eq!(outputs.reproduce, 1.0);
}

#[test]
fn min_and_max_default_missing_inputs_to_zero() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::Constant(0.4),   // 0
            NodeKind::Min,             // 1
            NodeKind::Max,             // 2
            NodeKind::OutputEat,       // 3
            NodeKind::OutputReproduce, // 4
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 0,
                to: 2,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 3,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 4,
                weight: 1.0,
            },
        ],
    };

    let outputs = graph.evaluate(SensorInputs {
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
        ..SensorInputs::default()
    });

    assert_eq!(outputs.eat, 0.0);
    assert_eq!(outputs.reproduce, 0.4);
}

#[test]
fn memory_io_nodes_are_supported_and_clamped() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputMemoryRead,   // 0
            NodeKind::OutputMemoryWrite, // 1
        ],
        edges: vec![Edge {
            from: 0,
            to: 1,
            weight: 1.0,
        }],
    };

    let high = graph.evaluate(SensorInputs {
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
        memory_read: 2.0,
        ..SensorInputs::default()
    });
    assert_eq!(high.memory_write, 1.0);

    let low = graph.evaluate(SensorInputs {
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
        memory_read: -1.0,
        ..SensorInputs::default()
    });
    assert_eq!(low.memory_write, 0.0);
}

#[test]
fn touch_slot_inputs_and_inventory_outputs_are_supported_and_clamped() {
    let graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputTouchExists(1),            // 0
            NodeKind::OutputInventoryPickup,          // 1
            NodeKind::InputTouchFoodValue(2),         // 2
            NodeKind::OutputInventorySlotSelect,      // 3
            NodeKind::InputTouchHasBarrier(3),        // 4
            NodeKind::OutputInventoryPut,             // 5
            NodeKind::InputTouchOccupied(4),          // 6
            NodeKind::OutputMoveX,                    // 7
            NodeKind::InputSlotExists(0),             // 8
            NodeKind::OutputMoveY,                    // 9
            NodeKind::InputSlotIsEmpty(0),            // 10
            NodeKind::OutputEat,                      // 11
            NodeKind::InputSlotIsBarrier(0),          // 12
            NodeKind::OutputInventoryDirectionSelect, // 13
            NodeKind::InputSlotFoodValue(0),          // 14
            NodeKind::OutputReproduce,                // 15
            NodeKind::InputSlotFoodValue(99),         // 16
            NodeKind::OutputMemoryWrite,              // 17
        ],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 2.0,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 2.0,
            },
            Edge {
                from: 4,
                to: 5,
                weight: 1.0,
            },
            Edge {
                from: 6,
                to: 7,
                weight: 1.0,
            },
            Edge {
                from: 8,
                to: 9,
                weight: -1.0,
            },
            Edge {
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 12,
                to: 13,
                weight: -2.0,
            },
            Edge {
                from: 14,
                to: 15,
                weight: 1.0,
            },
            Edge {
                from: 16,
                to: 17,
                weight: 1.0,
            },
        ],
    };

    let outputs = graph.evaluate(SensorInputs {
        touch_exists: [0.0, 2.0, 0.0, 0.0, 0.0],
        touch_food_value: [0.0, 0.0, 1.3, 0.0, 0.0],
        touch_has_barrier: [0.0, 0.0, 0.0, -1.0, 0.0],
        touch_occupied: [0.0, 0.0, 0.0, 0.0, 1.0],
        slot_exists: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        slot_is_empty: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        slot_is_barrier: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        slot_food_value: [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ..SensorInputs::default()
    });

    assert_eq!(outputs.inventory_pickup, 1.0);
    assert_eq!(outputs.inventory_slot_select, 1.0);
    assert_eq!(outputs.inventory_put, 0.0);
    assert_eq!(outputs.move_x, 1.0);
    assert_eq!(outputs.move_y, -1.0);
    assert_eq!(outputs.eat, 1.0);
    assert_eq!(outputs.inventory_direction_select, -1.0);
    assert_eq!(outputs.reproduce, 1.0);
    assert_eq!(outputs.memory_write, 0.0);
}
