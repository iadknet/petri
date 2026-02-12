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
                        | NodeKind::InputBarrierDirection
                        | NodeKind::InputBarrierDistance
                        | NodeKind::InputMoveBlockedLastTick
                        | NodeKind::InputMemoryRead
                        | NodeKind::InputTouchExists(_)
                        | NodeKind::InputTouchFoodValue(_)
                        | NodeKind::InputTouchHasBarrier(_)
                        | NodeKind::InputTouchOccupied(_)
                        | NodeKind::InputSlotExists(_)
                        | NodeKind::InputSlotIsEmpty(_)
                        | NodeKind::InputSlotIsBarrier(_)
                        | NodeKind::InputSlotFoodValue(_)
                        | NodeKind::OutputMoveX
                        | NodeKind::OutputMoveY
                        | NodeKind::OutputEat
                        | NodeKind::OutputReproduce
                        | NodeKind::OutputMemoryWrite
                        | NodeKind::OutputInventoryPickup
                        | NodeKind::OutputInventoryPut
                        | NodeKind::OutputInventorySlotSelect
                        | NodeKind::OutputInventoryDirectionSelect
                )
            })
            .count()
    }

    pub fn phenotype_color(&self) -> [u8; 3] {
        let fingerprint = self.phenotype_fingerprint();
        let hue = ((fingerprint & 0xFFFF) as f32 / 65_535.0) * 360.0;
        let saturation = 0.55 + (((fingerprint >> 16) & 0xFF) as f32 / 255.0) * 0.35;
        let value = 0.65 + (((fingerprint >> 24) & 0xFF) as f32 / 255.0) * 0.30;
        hsv_to_rgb(hue, saturation, value)
    }

    fn phenotype_fingerprint(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        hash = fnv1a_mix(
            hash,
            match self.palette {
                ControllerPalette::NeuralOnly => 1,
                ControllerPalette::LogicOnly => 2,
                ControllerPalette::Hybrid => 3,
            },
        );
        hash = fnv1a_mix(hash, self.nodes.len() as u64);
        for (index, node) in self.nodes.iter().enumerate() {
            let (kind, payload) = node_signature(node);
            hash = fnv1a_mix(hash, index as u64);
            hash = fnv1a_mix(hash, kind);
            hash = fnv1a_mix(hash, payload);
        }

        let mut edges = self
            .edges
            .iter()
            .map(|edge| {
                (
                    edge.from as u64,
                    edge.to as u64,
                    edge.weight.to_bits() as u64,
                )
            })
            .collect::<Vec<_>>();
        edges.sort_unstable();
        hash = fnv1a_mix(hash, edges.len() as u64);
        for (from, to, weight_bits) in edges {
            hash = fnv1a_mix(hash, from);
            hash = fnv1a_mix(hash, to);
            hash = fnv1a_mix(hash, weight_bits);
        }
        hash
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

fn fnv1a_mix(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV_PRIME)
}

fn node_signature(node: &NodeKind) -> (u64, u64) {
    match node {
        NodeKind::InputFoodHere => (1, 0),
        NodeKind::InputEnergy => (2, 0),
        NodeKind::InputRandom => (3, 0),
        NodeKind::InputFoodDirection => (4, 0),
        NodeKind::InputFoodDistance => (5, 0),
        NodeKind::InputCreatureDirection => (6, 0),
        NodeKind::InputCreatureDistance => (7, 0),
        NodeKind::InputLocalDensity => (8, 0),
        NodeKind::InputBarrierDirection => (9, 0),
        NodeKind::InputBarrierDistance => (10, 0),
        NodeKind::InputMoveBlockedLastTick => (11, 0),
        NodeKind::InputMemoryRead => (12, 0),
        NodeKind::InputTouchExists(index) => (13, *index as u64),
        NodeKind::InputTouchFoodValue(index) => (14, *index as u64),
        NodeKind::InputTouchHasBarrier(index) => (15, *index as u64),
        NodeKind::InputTouchOccupied(index) => (16, *index as u64),
        NodeKind::InputSlotExists(index) => (17, *index as u64),
        NodeKind::InputSlotIsEmpty(index) => (18, *index as u64),
        NodeKind::InputSlotIsBarrier(index) => (19, *index as u64),
        NodeKind::InputSlotFoodValue(index) => (20, *index as u64),
        NodeKind::Constant(value) => (21, value.to_bits() as u64),
        NodeKind::Add => (22, 0),
        NodeKind::Multiply => (23, 0),
        NodeKind::Negate => (24, 0),
        NodeKind::Abs => (25, 0),
        NodeKind::Min => (26, 0),
        NodeKind::Max => (27, 0),
        NodeKind::Threshold(value) => (28, value.to_bits() as u64),
        NodeKind::GreaterThan => (29, 0),
        NodeKind::Sigmoid => (30, 0),
        NodeKind::Tanh => (31, 0),
        NodeKind::Relu => (32, 0),
        NodeKind::Select => (33, 0),
        NodeKind::OutputMoveX => (34, 0),
        NodeKind::OutputMoveY => (35, 0),
        NodeKind::OutputEat => (36, 0),
        NodeKind::OutputReproduce => (37, 0),
        NodeKind::OutputMemoryWrite => (38, 0),
        NodeKind::OutputInventoryPickup => (39, 0),
        NodeKind::OutputInventoryPut => (40, 0),
        NodeKind::OutputInventorySlotSelect => (41, 0),
        NodeKind::OutputInventoryDirectionSelect => (42, 0),
    }
}

fn hsv_to_rgb(hue_degrees: f32, saturation: f32, value: f32) -> [u8; 3] {
    let hue = if hue_degrees.is_finite() {
        hue_degrees.rem_euclid(360.0)
    } else {
        0.0
    };
    let saturation = saturation.clamp(0.0, 1.0);
    let value = value.clamp(0.0, 1.0);

    let chroma = value * saturation;
    let hue_section = hue / 60.0;
    let x = chroma * (1.0 - ((hue_section.rem_euclid(2.0)) - 1.0).abs());
    let (r1, g1, b1) = if hue_section < 1.0 {
        (chroma, x, 0.0)
    } else if hue_section < 2.0 {
        (x, chroma, 0.0)
    } else if hue_section < 3.0 {
        (0.0, chroma, x)
    } else if hue_section < 4.0 {
        (0.0, x, chroma)
    } else if hue_section < 5.0 {
        (x, 0.0, chroma)
    } else {
        (chroma, 0.0, x)
    };
    let m = value - chroma;

    [to_rgb_u8(r1 + m), to_rgb_u8(g1 + m), to_rgb_u8(b1 + m)]
}

fn to_rgb_u8(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}
