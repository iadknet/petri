use rand::Rng;

use crate::types::NodeKind;

pub(super) fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

pub(super) fn is_input_node(node: &NodeKind) -> bool {
    matches!(
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
    )
}

pub(super) fn is_output_node(node: &NodeKind) -> bool {
    matches!(
        node,
        NodeKind::OutputMoveX
            | NodeKind::OutputMoveY
            | NodeKind::OutputEat
            | NodeKind::OutputReproduce
            | NodeKind::OutputMemoryWrite
            | NodeKind::OutputInventoryPickup
            | NodeKind::OutputInventoryPut
            | NodeKind::OutputInventorySlotSelect
            | NodeKind::OutputInventoryDirectionSelect
    )
}

pub(super) fn is_hidden_node(node: &NodeKind) -> bool {
    !is_input_node(node) && !is_output_node(node)
}

pub(super) fn random_hidden_node<R: Rng>(rng: &mut R) -> NodeKind {
    match rng.gen_range(0..13) {
        0 => NodeKind::Add,
        1 => NodeKind::Multiply,
        2 => NodeKind::Negate,
        3 => NodeKind::Abs,
        4 => NodeKind::Min,
        5 => NodeKind::Max,
        6 => NodeKind::Threshold(rng.gen_range(0.0..=1.0)),
        7 => NodeKind::GreaterThan,
        8 => NodeKind::Sigmoid,
        9 => NodeKind::Tanh,
        10 => NodeKind::Relu,
        11 => NodeKind::Select,
        _ => NodeKind::Constant(rng.gen_range(-1.0..=1.0)),
    }
}
