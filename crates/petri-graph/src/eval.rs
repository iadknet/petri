use crate::types::{ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs};

#[derive(Clone, Debug)]
pub struct ComputationGraph {
    pub palette: ControllerPalette,
    pub nodes: Vec<NodeKind>,
    pub edges: Vec<Edge>,
}

impl ComputationGraph {
    pub fn from_palette(palette: ControllerPalette) -> Self {
        match palette {
            ControllerPalette::NeuralOnly => neural_only(),
            ControllerPalette::LogicOnly => logic_only(),
            ControllerPalette::Hybrid => hybrid(),
        }
    }

    pub fn evaluate(&self, inputs: SensorInputs) -> ActionOutputs {
        let mut incoming: Vec<Vec<(usize, f32)>> = vec![Vec::new(); self.nodes.len()];
        for edge in &self.edges {
            if edge.to < incoming.len() && edge.from < self.nodes.len() {
                incoming[edge.to].push((edge.from, edge.weight));
            }
        }

        let mut values = vec![0.0_f32; self.nodes.len()];
        let mut outputs = ActionOutputs::default();

        for idx in 0..self.nodes.len() {
            let weighted_inputs = incoming[idx]
                .iter()
                .map(|(src, w)| values[*src] * *w)
                .collect::<Vec<_>>();

            let value = match self.nodes[idx] {
                NodeKind::InputFoodHere => inputs.food_here.clamp(0.0, 1.0),
                NodeKind::InputEnergy => inputs.energy.clamp(0.0, 1.0),
                NodeKind::InputRandom => inputs.random.clamp(-1.0, 1.0),
                NodeKind::Constant(v) => v,
                NodeKind::Add => weighted_inputs.iter().sum(),
                NodeKind::Multiply => {
                    if weighted_inputs.is_empty() {
                        0.0
                    } else {
                        weighted_inputs.iter().copied().product()
                    }
                }
                NodeKind::Threshold(t) => {
                    if weighted_inputs.iter().sum::<f32>() >= t {
                        1.0
                    } else {
                        0.0
                    }
                }
                NodeKind::GreaterThan => {
                    let a = *weighted_inputs.first().unwrap_or(&0.0);
                    let b = *weighted_inputs.get(1).unwrap_or(&0.0);
                    if a > b {
                        1.0
                    } else {
                        0.0
                    }
                }
                NodeKind::Sigmoid => {
                    let x = weighted_inputs.iter().sum::<f32>();
                    1.0 / (1.0 + (-x).exp())
                }
                NodeKind::Tanh => weighted_inputs.iter().sum::<f32>().tanh(),
                NodeKind::Select => {
                    let control = *weighted_inputs.first().unwrap_or(&0.0);
                    let a = *weighted_inputs.get(1).unwrap_or(&0.0);
                    let b = *weighted_inputs.get(2).unwrap_or(&0.0);
                    if control > 0.0 {
                        b
                    } else {
                        a
                    }
                }
                NodeKind::OutputMoveX
                | NodeKind::OutputMoveY
                | NodeKind::OutputEat
                | NodeKind::OutputReproduce => weighted_inputs.iter().sum(),
            };

            values[idx] = value;
            match self.nodes[idx] {
                NodeKind::OutputMoveX => outputs.move_x = value.clamp(-1.0, 1.0),
                NodeKind::OutputMoveY => outputs.move_y = value.clamp(-1.0, 1.0),
                NodeKind::OutputEat => outputs.eat = value.clamp(0.0, 1.0),
                NodeKind::OutputReproduce => outputs.reproduce = value.clamp(0.0, 1.0),
                _ => {}
            }
        }

        outputs
    }
}

fn neural_only() -> ComputationGraph {
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

fn logic_only() -> ComputationGraph {
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

fn hybrid() -> ComputationGraph {
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
