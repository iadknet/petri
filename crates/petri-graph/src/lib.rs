mod eval;
mod types;

pub use eval::{ComputationGraph, MutationConfig};
pub use types::{ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn hidden_node_count(graph: &ComputationGraph) -> usize {
        graph
            .nodes
            .iter()
            .filter(|node| {
                !matches!(
                    node,
                    NodeKind::InputFoodHere
                        | NodeKind::InputEnergy
                        | NodeKind::InputRandom
                        | NodeKind::InputFoodDirection
                        | NodeKind::InputFoodDistance
                        | NodeKind::InputCreatureDirection
                        | NodeKind::InputCreatureDistance
                        | NodeKind::InputLocalDensity
                        | NodeKind::InputMoveBlockedLastTick
                        | NodeKind::OutputMoveX
                        | NodeKind::OutputMoveY
                        | NodeKind::OutputEat
                        | NodeKind::OutputReproduce
                )
            })
            .count()
    }

    fn parameter_checksum(graph: &ComputationGraph) -> f32 {
        let node_sum = graph
            .nodes
            .iter()
            .map(|node| match node {
                NodeKind::Constant(v) => *v,
                NodeKind::Threshold(t) => *t,
                _ => 0.0,
            })
            .sum::<f32>();
        let edge_sum = graph.edges.iter().map(|e| e.weight).sum::<f32>();
        node_sum + edge_sum
    }

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
        });
        let impulse = outputs.move_x.abs() + outputs.move_y.abs();
        assert!(impulse < 0.6);
    }

    #[test]
    fn mutation_changes_parameters_without_changing_topology() {
        let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
        let before_nodes = graph.nodes.len();
        let before_edges = graph.edges.len();
        let before_checksum = parameter_checksum(&graph);

        let mut rng = SmallRng::seed_from_u64(7);
        graph.mutate_weights(&mut rng, 1.0, 0.2);

        let after_checksum = parameter_checksum(&graph);
        assert_eq!(graph.nodes.len(), before_nodes);
        assert_eq!(graph.edges.len(), before_edges);
        assert_ne!(before_checksum, after_checksum);

        let outputs = graph.evaluate(SensorInputs {
            food_here: 1.0,
            energy: 1.0,
            random: 0.25,
            food_direction: 0.0,
            food_distance: 1.0,
            creature_direction: 0.0,
            creature_distance: 1.0,
            local_density: 0.0,
            move_blocked_last_tick: 0.0,
        });
        assert!((-1.0..=1.0).contains(&outputs.move_x));
        assert!((-1.0..=1.0).contains(&outputs.move_y));
        assert!((0.0..=1.0).contains(&outputs.eat));
        assert!((0.0..=1.0).contains(&outputs.reproduce));
    }

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
            move_blocked_last_tick: 0.0,
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
            move_blocked_last_tick: 0.0,
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
            move_blocked_last_tick: 3.0,
        });

        assert_eq!(outputs.move_x, 1.0);
        assert_eq!(outputs.eat, 0.0);
        assert_eq!(outputs.reproduce, 1.0);
        assert_eq!(outputs.move_y, -1.0);
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
            move_blocked_last_tick: 0.0,
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
            move_blocked_last_tick: 0.0,
        });

        assert_eq!(outputs.eat, 0.0);
        assert_eq!(outputs.reproduce, 0.4);
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
        });

        let delta = (clear_path.move_x - blocked_and_crowded.move_x).abs()
            + (clear_path.move_y - blocked_and_crowded.move_y).abs()
            + (clear_path.reproduce - blocked_and_crowded.reproduce).abs();
        assert!(
            delta > 0.05,
            "expected new sensors to influence founder hybrid outputs"
        );
    }

    #[test]
    fn add_hidden_node_mutation_splices_existing_edge() {
        let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
        let before_nodes = graph.nodes.len();
        let before_edges = graph.edges.len();
        let mut rng = SmallRng::seed_from_u64(13);

        let changed = graph.add_hidden_node_by_splicing_edge(&mut rng);
        assert!(changed);
        assert_eq!(graph.nodes.len(), before_nodes + 1);
        assert_eq!(graph.edges.len(), before_edges + 1);
        assert!(hidden_node_count(&graph) >= 1);
    }

    #[test]
    fn add_edge_mutation_inserts_new_connection_when_possible() {
        let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
        let before_edges = graph.edges.len();
        let mut rng = SmallRng::seed_from_u64(17);

        let mut changed = false;
        for _ in 0..32 {
            changed = graph.add_edge_mutation(&mut rng);
            if changed {
                break;
            }
        }

        assert!(changed);
        assert_eq!(graph.edges.len(), before_edges + 1);
    }

    #[test]
    fn remove_edge_mutation_removes_one_edge() {
        let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
        let before_edges = graph.edges.len();
        let mut rng = SmallRng::seed_from_u64(19);

        let changed = graph.remove_edge_mutation(&mut rng);
        assert!(changed);
        assert_eq!(graph.edges.len(), before_edges - 1);
    }

    #[test]
    fn remove_disconnected_hidden_nodes_prunes_orphan_nodes() {
        let mut graph = ComputationGraph {
            palette: ControllerPalette::Hybrid,
            nodes: vec![
                NodeKind::InputFoodHere,
                NodeKind::OutputEat,
                NodeKind::Tanh, // disconnected hidden node
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            }],
        };

        let removed = graph.remove_disconnected_hidden_nodes();
        assert_eq!(removed, 1);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn change_hidden_node_type_mutation_retypes_hidden_node() {
        let mut graph = ComputationGraph {
            palette: ControllerPalette::Hybrid,
            nodes: vec![NodeKind::InputFoodHere, NodeKind::Add, NodeKind::OutputEat],
            edges: vec![
                Edge {
                    from: 0,
                    to: 1,
                    weight: 1.0,
                },
                Edge {
                    from: 1,
                    to: 2,
                    weight: 1.0,
                },
            ],
        };
        let mut rng = SmallRng::seed_from_u64(23);

        let changed = graph.change_hidden_node_type(&mut rng);
        assert!(changed);
        assert!(!matches!(graph.nodes[1], NodeKind::Add));
    }

    #[test]
    fn mutate_with_config_runs_structural_and_logic_operators() {
        let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
        let before_nodes = graph.nodes.len();
        let before_edges = graph.edges.len();
        let mut rng = SmallRng::seed_from_u64(29);

        let changed = graph.mutate_with_config(
            &mut rng,
            MutationConfig {
                weight_mutation_rate: 0.0,
                weight_mutation_magnitude: 0.0,
                structural_mutation_rate: 1.0,
                logic_node_mutation_rate: 1.0,
            },
        );

        assert!(changed);
        assert!(graph.nodes.len() != before_nodes || graph.edges.len() != before_edges);
    }
}
