use petri_graph::{ComputationGraph, ControllerPalette, NodeKind, SensorInputs};

#[test]
fn neural_palette_outputs_are_bounded() {
    let graph = ComputationGraph::from_palette(ControllerPalette::NeuralOnly);
    let outputs = graph.evaluate(SensorInputs {
        food_here: 1.0,
        energy: 0.8,
        random: 0.2,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });

    assert!((-1.0..=1.0).contains(&outputs.move_x));
    assert!((-1.0..=1.0).contains(&outputs.move_y));
    assert!((0.0..=1.0).contains(&outputs.eat));
    assert!((0.0..=1.0).contains(&outputs.reproduce));
}

#[test]
fn logic_palette_gates_eat_and_reproduce() {
    let graph = ComputationGraph::from_palette(ControllerPalette::LogicOnly);

    let high = graph.evaluate(SensorInputs {
        food_here: 1.0,
        energy: 1.0,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    assert!(high.eat > 0.5);
    assert!(high.reproduce > 0.5);

    let low = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.2,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    assert!(low.eat <= 0.5);
    assert!(low.reproduce <= 0.5);
}

#[test]
fn hybrid_palette_motion_changes_with_random_sensor() {
    let graph = ComputationGraph::from_palette(ControllerPalette::Hybrid);

    let a = graph.evaluate(SensorInputs {
        food_here: 0.4,
        energy: 0.6,
        random: -0.9,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    let b = graph.evaluate(SensorInputs {
        food_here: 0.4,
        energy: 0.6,
        random: 0.9,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });

    let delta = (a.move_x - b.move_x).abs() + (a.move_y - b.move_y).abs();
    assert!(delta > 0.05);
}

#[test]
fn founder_graph_forages_and_delays_reproduction() {
    let graph = ComputationGraph::founder(ControllerPalette::Hybrid);

    let food_low_energy = graph.evaluate(SensorInputs {
        food_here: 1.0,
        energy: 0.2,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    assert!(food_low_energy.eat > 0.5);
    assert!(food_low_energy.reproduce <= 0.5);

    let food_high_energy = graph.evaluate(SensorInputs {
        food_here: 1.0,
        energy: 1.0,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    assert!(food_high_energy.reproduce > 0.5);

    let no_food = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 1.0,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    assert!(no_food.eat <= 0.5);
}

#[test]
fn founder_graph_has_low_default_motion_cost() {
    let graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let outputs = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.5,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    let impulse = outputs.move_x.abs() + outputs.move_y.abs();
    assert!(impulse < 0.6);
}

#[test]
fn founder_hybrid_references_food_distance_and_direction_sensors() {
    let graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| matches!(node, NodeKind::InputFoodDirection)),
        "founder graph should include food-direction sensor node"
    );
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| matches!(node, NodeKind::InputFoodDistance)),
        "founder graph should include food-distance sensor node"
    );
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| matches!(node, NodeKind::InputCreatureDistance)),
        "founder graph should include creature-distance sensor node"
    );
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| matches!(node, NodeKind::InputMoveBlockedLastTick)),
        "founder graph should include movement-feedback sensor node"
    );
}

#[test]
fn founder_hybrid_outputs_change_when_new_sensors_change() {
    let graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let clear_path = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.6,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
    });
    let blocked_and_crowded = graph.evaluate(SensorInputs {
        food_here: 0.0,
        energy: 0.6,
        random: 0.0,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 0.0,
        local_density: 1.0,
        move_blocked_last_tick: 1.0,
        memory_read: 0.0,
    });

    let delta = (clear_path.move_x - blocked_and_crowded.move_x).abs()
        + (clear_path.move_y - blocked_and_crowded.move_y).abs()
        + (clear_path.reproduce - blocked_and_crowded.reproduce).abs();
    assert!(
        delta > 0.05,
        "expected new sensors to influence founder hybrid outputs"
    );
}
