use crate::types::CreatureEvent;

use super::{Creature, CreatureEventKind, EVENT_LOG_CAPACITY, FOOD_SENSOR_RADIUS};

pub(super) fn sensor_from_best(best: Option<(i32, i32, i32)>) -> (f32, f32) {
    let Some((dist_sq, dx, dy)) = best else {
        return (0.0, 1.0);
    };
    if dist_sq == 0 {
        return (0.0, 0.0);
    }

    let distance = (dist_sq as f32).sqrt() / FOOD_SENSOR_RADIUS as f32;
    let direction = (dy as f32).atan2(dx as f32) / std::f32::consts::PI;
    (direction.clamp(-1.0, 1.0), distance.clamp(0.0, 1.0))
}

pub(super) fn axis_step(value: f32) -> i32 {
    if value > 0.25 {
        1
    } else if value < -0.25 {
        -1
    } else {
        0
    }
}

pub(super) fn map_axis(v: i32, max: u32, wrap: bool) -> u32 {
    if max == 0 {
        return 0;
    }
    if wrap {
        wrap_axis(v, max)
    } else {
        v.clamp(0, max as i32 - 1) as u32
    }
}

pub(super) fn wrap_axis(v: i32, max: u32) -> u32 {
    let m = max as i32;
    (((v % m) + m) % m) as u32
}

pub(super) fn quantize_food(food: f32, max_density: f32) -> u8 {
    let max_density = max_density.max(f32::EPSILON);
    let normalized = (food.clamp(0.0, max_density) / max_density).clamp(0.0, 1.0);
    (normalized * 255.0).round() as u8
}

pub(super) fn push_event(creature: &mut Creature, kind: CreatureEventKind, tick: u64) {
    if creature.events.len() == EVENT_LOG_CAPACITY {
        creature.events.pop_front();
    }
    creature.events.push_back(CreatureEvent { kind, tick });
}
