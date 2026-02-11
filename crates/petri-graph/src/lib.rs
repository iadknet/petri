mod eval;
mod types;

pub use eval::ComputationGraph;
pub use types::{ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

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
        });
        assert!(high.eat > 0.5);
        assert!(high.reproduce > 0.5);

        let low = graph.evaluate(SensorInputs {
            food_here: 0.0,
            energy: 0.2,
            random: 0.0,
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
        });
        let b = graph.evaluate(SensorInputs {
            food_here: 0.4,
            energy: 0.6,
            random: 0.9,
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
        });
        assert!(food_low_energy.eat > 0.5);
        assert!(food_low_energy.reproduce <= 0.5);

        let food_high_energy = graph.evaluate(SensorInputs {
            food_here: 1.0,
            energy: 1.0,
            random: 0.0,
        });
        assert!(food_high_energy.reproduce > 0.5);

        let no_food = graph.evaluate(SensorInputs {
            food_here: 0.0,
            energy: 1.0,
            random: 0.0,
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
        });
        assert!((-1.0..=1.0).contains(&outputs.move_x));
        assert!((-1.0..=1.0).contains(&outputs.move_y));
        assert!((0.0..=1.0).contains(&outputs.eat));
        assert!((0.0..=1.0).contains(&outputs.reproduce));
    }
}
