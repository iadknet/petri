use serde::{Deserialize, Serialize};

use crate::types::{ControllerPalette, Edge, NodeKind};

mod evaluate;
mod mutate;
mod node_utils;
mod presets;

use self::presets::{
    founder_hybrid, founder_logic_only, founder_neural_only, hybrid, logic_only, neural_only,
};

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
            weight_mutation_rate: 0.08,
            weight_mutation_magnitude: 0.18,
            logic_node_mutation_rate: 0.01,
            structural_mutation_rate: 0.02,
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
}
