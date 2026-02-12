use crate::types::{ActionOutputs, NodeKind, SensorInputs};

use super::*;

impl ComputationGraph {
    pub fn evaluate(&self, inputs: SensorInputs) -> ActionOutputs {
        fn sensor_at(sensor: &[f32], index: u8) -> f32 {
            sensor.get(index as usize).copied().unwrap_or(0.0)
        }

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
                NodeKind::InputCreatureDirection => inputs.creature_direction.clamp(-1.0, 1.0),
                NodeKind::InputCreatureDistance => inputs.creature_distance.clamp(0.0, 1.0),
                NodeKind::InputLocalDensity => inputs.local_density.clamp(0.0, 1.0),
                NodeKind::InputBarrierDirection => inputs.barrier_direction.clamp(-1.0, 1.0),
                NodeKind::InputBarrierDistance => inputs.barrier_distance.clamp(0.0, 1.0),
                NodeKind::InputMoveBlockedLastTick => {
                    if inputs.move_blocked_last_tick > 0.5 {
                        1.0
                    } else {
                        0.0
                    }
                }
                NodeKind::InputMemoryRead => inputs.memory_read.clamp(0.0, 1.0),
                NodeKind::InputTouchExists(index) => {
                    sensor_at(&inputs.touch_exists, index).clamp(0.0, 1.0)
                }
                NodeKind::InputTouchFoodValue(index) => {
                    sensor_at(&inputs.touch_food_value, index).clamp(0.0, 1.0)
                }
                NodeKind::InputTouchHasBarrier(index) => {
                    sensor_at(&inputs.touch_has_barrier, index).clamp(0.0, 1.0)
                }
                NodeKind::InputTouchOccupied(index) => {
                    sensor_at(&inputs.touch_occupied, index).clamp(0.0, 1.0)
                }
                NodeKind::InputSlotExists(index) => {
                    sensor_at(&inputs.slot_exists, index).clamp(0.0, 1.0)
                }
                NodeKind::InputSlotIsEmpty(index) => {
                    sensor_at(&inputs.slot_is_empty, index).clamp(0.0, 1.0)
                }
                NodeKind::InputSlotIsBarrier(index) => {
                    sensor_at(&inputs.slot_is_barrier, index).clamp(0.0, 1.0)
                }
                NodeKind::InputSlotFoodValue(index) => {
                    sensor_at(&inputs.slot_food_value, index).clamp(0.0, 1.0)
                }
                NodeKind::Constant(v) => v,
                NodeKind::Add => weighted_inputs.iter().sum(),
                NodeKind::Multiply => {
                    if weighted_inputs.is_empty() {
                        0.0
                    } else {
                        weighted_inputs.iter().copied().product()
                    }
                }
                NodeKind::Negate => -weighted_inputs.iter().sum::<f32>(),
                NodeKind::Abs => weighted_inputs.iter().sum::<f32>().abs(),
                NodeKind::Min => {
                    let a = *weighted_inputs.first().unwrap_or(&0.0);
                    let b = *weighted_inputs.get(1).unwrap_or(&0.0);
                    a.min(b)
                }
                NodeKind::Max => {
                    let a = *weighted_inputs.first().unwrap_or(&0.0);
                    let b = *weighted_inputs.get(1).unwrap_or(&0.0);
                    a.max(b)
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
                | NodeKind::OutputReproduce
                | NodeKind::OutputMemoryWrite
                | NodeKind::OutputInventoryPickup
                | NodeKind::OutputInventoryPut
                | NodeKind::OutputInventorySlotSelect
                | NodeKind::OutputInventoryDirectionSelect => weighted_inputs.iter().sum(),
            };

            values[idx] = value;
            match self.nodes[idx] {
                NodeKind::OutputMoveX => outputs.move_x = value.clamp(-1.0, 1.0),
                NodeKind::OutputMoveY => outputs.move_y = value.clamp(-1.0, 1.0),
                NodeKind::OutputEat => outputs.eat = value.clamp(0.0, 1.0),
                NodeKind::OutputReproduce => outputs.reproduce = value.clamp(0.0, 1.0),
                NodeKind::OutputMemoryWrite => outputs.memory_write = value.clamp(0.0, 1.0),
                NodeKind::OutputInventoryPickup => outputs.inventory_pickup = value.clamp(0.0, 1.0),
                NodeKind::OutputInventoryPut => outputs.inventory_put = value.clamp(0.0, 1.0),
                NodeKind::OutputInventorySlotSelect => {
                    outputs.inventory_slot_select = value.clamp(-1.0, 1.0)
                }
                NodeKind::OutputInventoryDirectionSelect => {
                    outputs.inventory_direction_select = value.clamp(-1.0, 1.0)
                }
                _ => {}
            }
        }

        outputs
    }
}
