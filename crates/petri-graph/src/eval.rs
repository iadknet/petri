use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::types::{ActionOutputs, ControllerPalette, Edge, NodeKind, SensorInputs};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MutationConfig {
    pub weight_mutation_rate: f32,
    pub weight_mutation_magnitude: f32,
    pub logic_node_mutation_rate: f32,
    pub structural_mutation_rate: f32,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            weight_mutation_rate: 0.26,
            weight_mutation_magnitude: 0.18,
            logic_node_mutation_rate: 0.04,
            structural_mutation_rate: 0.08,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn founder(palette: ControllerPalette) -> Self {
        match palette {
            ControllerPalette::NeuralOnly => founder_neural_only(),
            ControllerPalette::LogicOnly => founder_logic_only(),
            ControllerPalette::Hybrid => founder_hybrid(),
        }
    }

    pub fn mutate_weights<R: Rng>(&mut self, rng: &mut R, rate: f32, magnitude: f32) -> bool {
        let rate = rate.clamp(0.0, 1.0);
        let magnitude = magnitude.abs();
        let mut changed = false;

        for edge in &mut self.edges {
            if rng.gen_bool(rate as f64) {
                let before = edge.weight;
                edge.weight = clamp(
                    edge.weight + rng.gen_range(-magnitude..=magnitude),
                    -8.0,
                    8.0,
                );
                changed |= (edge.weight - before).abs() > f32::EPSILON;
            }
        }

        for node in &mut self.nodes {
            if rng.gen_bool(rate as f64) {
                match node {
                    NodeKind::Constant(v) => {
                        let before = *v;
                        *v = clamp(*v + rng.gen_range(-magnitude..=magnitude), -8.0, 8.0);
                        changed |= (*v - before).abs() > f32::EPSILON;
                    }
                    NodeKind::Threshold(t) => {
                        let before = *t;
                        *t = clamp(*t + rng.gen_range(-magnitude..=magnitude), 0.0, 1.0);
                        changed |= (*t - before).abs() > f32::EPSILON;
                    }
                    _ => {}
                }
            }
        }

        changed
    }

    pub fn mutate_with_config<R: Rng>(&mut self, rng: &mut R, cfg: MutationConfig) -> bool {
        let mut changed = false;
        changed |=
            self.mutate_weights(rng, cfg.weight_mutation_rate, cfg.weight_mutation_magnitude);

        let structural_rate = cfg.structural_mutation_rate.clamp(0.0, 1.0);
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.add_hidden_node_by_splicing_edge(rng);
        }
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.add_edge_mutation(rng);
        }
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.remove_edge_mutation(rng);
        }
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.remove_disconnected_hidden_nodes() > 0;
        }

        let logic_rate = cfg.logic_node_mutation_rate.clamp(0.0, 1.0);
        if rng.gen_bool(logic_rate as f64) {
            changed |= self.change_hidden_node_type(rng);
        }

        changed
    }

    pub fn add_hidden_node_by_splicing_edge<R: Rng>(&mut self, rng: &mut R) -> bool {
        let candidate_indices = self
            .edges
            .iter()
            .enumerate()
            .filter_map(
                |(idx, edge)| {
                    if edge.from < edge.to {
                        Some(idx)
                    } else {
                        None
                    }
                },
            )
            .collect::<Vec<_>>();
        if candidate_indices.is_empty() {
            return false;
        }

        let edge_idx = candidate_indices[rng.gen_range(0..candidate_indices.len())];
        let edge = self.edges.remove(edge_idx);
        let insert_at = edge.to;

        for existing in &mut self.edges {
            if existing.from >= insert_at {
                existing.from += 1;
            }
            if existing.to >= insert_at {
                existing.to += 1;
            }
        }

        self.nodes.insert(insert_at, random_hidden_node(rng));
        let shifted_to = edge.to + 1;

        self.edges.push(Edge {
            from: edge.from,
            to: insert_at,
            weight: 1.0,
        });
        self.edges.push(Edge {
            from: insert_at,
            to: shifted_to,
            weight: edge.weight,
        });

        true
    }

    pub fn add_edge_mutation<R: Rng>(&mut self, rng: &mut R) -> bool {
        let mut candidates = Vec::new();

        for from in 0..self.nodes.len() {
            if is_output_node(&self.nodes[from]) {
                continue;
            }
            for to in (from + 1)..self.nodes.len() {
                if is_input_node(&self.nodes[to]) || self.has_edge(from, to) {
                    continue;
                }
                candidates.push((from, to));
            }
        }

        if candidates.is_empty() {
            return false;
        }

        let (from, to) = candidates[rng.gen_range(0..candidates.len())];
        self.edges.push(Edge {
            from,
            to,
            weight: rng.gen_range(-1.0..=1.0),
        });
        true
    }

    pub fn remove_edge_mutation<R: Rng>(&mut self, rng: &mut R) -> bool {
        if self.edges.is_empty() {
            return false;
        }
        let idx = rng.gen_range(0..self.edges.len());
        self.edges.swap_remove(idx);
        true
    }

    pub fn remove_disconnected_hidden_nodes(&mut self) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }

        let mut incoming = vec![0_usize; self.nodes.len()];
        let mut outgoing = vec![0_usize; self.nodes.len()];
        for edge in &self.edges {
            if edge.to < incoming.len() {
                incoming[edge.to] += 1;
            }
            if edge.from < outgoing.len() {
                outgoing[edge.from] += 1;
            }
        }

        let mut removable = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if is_hidden_node(node) && incoming[idx] == 0 && outgoing[idx] == 0 {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        removable.sort_unstable();
        let removed = removable.len();
        for idx in removable.into_iter().rev() {
            self.remove_node(idx);
        }
        removed
    }

    pub fn change_hidden_node_type<R: Rng>(&mut self, rng: &mut R) -> bool {
        let hidden_indices = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if is_hidden_node(node) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if hidden_indices.is_empty() {
            return false;
        }

        let idx = hidden_indices[rng.gen_range(0..hidden_indices.len())];
        let original = self.nodes[idx].clone();
        let mut replacement = random_hidden_node(rng);
        for _ in 0..8 {
            if replacement != original {
                break;
            }
            replacement = random_hidden_node(rng);
        }
        if replacement == original {
            return false;
        }
        self.nodes[idx] = replacement;
        true
    }

    pub fn compute_node_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| {
                !matches!(
                    node,
                    NodeKind::InputFoodHere
                        | NodeKind::InputEnergy
                        | NodeKind::InputRandom
                        | NodeKind::InputFoodDirection
                        | NodeKind::InputFoodDistance
                        | NodeKind::OutputMoveX
                        | NodeKind::OutputMoveY
                        | NodeKind::OutputEat
                        | NodeKind::OutputReproduce
                )
            })
            .count()
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
                NodeKind::InputFoodDirection => inputs.food_direction.clamp(-1.0, 1.0),
                NodeKind::InputFoodDistance => inputs.food_distance.clamp(0.0, 1.0),
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
                NodeKind::Relu => weighted_inputs.iter().sum::<f32>().max(0.0),
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

    fn has_edge(&self, from: usize, to: usize) -> bool {
        self.edges
            .iter()
            .any(|edge| edge.from == from && edge.to == to)
    }

    fn remove_node(&mut self, idx: usize) {
        self.nodes.remove(idx);
        self.edges.retain(|edge| edge.from != idx && edge.to != idx);
        for edge in &mut self.edges {
            if edge.from > idx {
                edge.from -= 1;
            }
            if edge.to > idx {
                edge.to -= 1;
            }
        }
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

fn is_input_node(node: &NodeKind) -> bool {
    matches!(
        node,
        NodeKind::InputFoodHere
            | NodeKind::InputEnergy
            | NodeKind::InputRandom
            | NodeKind::InputFoodDirection
            | NodeKind::InputFoodDistance
    )
}

fn is_output_node(node: &NodeKind) -> bool {
    matches!(
        node,
        NodeKind::OutputMoveX
            | NodeKind::OutputMoveY
            | NodeKind::OutputEat
            | NodeKind::OutputReproduce
    )
}

fn is_hidden_node(node: &NodeKind) -> bool {
    !is_input_node(node) && !is_output_node(node)
}

fn random_hidden_node<R: Rng>(rng: &mut R) -> NodeKind {
    match rng.gen_range(0..9) {
        0 => NodeKind::Add,
        1 => NodeKind::Multiply,
        2 => NodeKind::Threshold(rng.gen_range(0.0..=1.0)),
        3 => NodeKind::GreaterThan,
        4 => NodeKind::Sigmoid,
        5 => NodeKind::Tanh,
        6 => NodeKind::Relu,
        7 => NodeKind::Select,
        _ => NodeKind::Constant(rng.gen_range(-1.0..=1.0)),
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

fn founder_neural_only() -> ComputationGraph {
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

fn founder_logic_only() -> ComputationGraph {
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

fn founder_hybrid() -> ComputationGraph {
    ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![
            NodeKind::InputFoodHere,      // 0
            NodeKind::InputEnergy,        // 1
            NodeKind::InputRandom,        // 2
            NodeKind::Threshold(0.08),    // 3
            NodeKind::OutputEat,          // 4
            NodeKind::Threshold(0.9),     // 5
            NodeKind::Multiply,           // 6
            NodeKind::OutputReproduce,    // 7
            NodeKind::Tanh,               // 8
            NodeKind::OutputMoveX,        // 9
            NodeKind::Tanh,               // 10
            NodeKind::OutputMoveY,        // 11
            NodeKind::InputFoodDirection, // 12
            NodeKind::InputFoodDistance,  // 13
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
                weight: 0.4,
            },
            Edge {
                from: 8,
                to: 9,
                weight: 1.0,
            },
            Edge {
                from: 2,
                to: 10,
                weight: -0.4,
            },
            Edge {
                from: 10,
                to: 11,
                weight: 1.0,
            },
            Edge {
                from: 12,
                to: 8,
                weight: 0.35,
            },
            Edge {
                from: 13,
                to: 10,
                weight: -0.35,
            },
        ],
    }
}
