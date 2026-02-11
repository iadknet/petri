mod eval;
mod types;

pub use eval::ComputationGraph;
pub use types::{ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs};

#[cfg(test)]
mod tests {
    use super::*;

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
}
