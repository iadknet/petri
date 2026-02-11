use super::helpers::{sensor_from_best, wrap_axis};
use super::*;

impl World {
    pub(super) fn scan_perception(
        &self,
        x: u32,
        y: u32,
        self_id: Option<CreatureId>,
    ) -> PerceptionScan {
        let mut food_best: Option<(i32, i32, i32)> = None;
        let mut creature_best: Option<(i32, i32, i32)> = None;
        let mut scanned_cells = 0_usize;
        let mut occupied_cells = 0_usize;
        let width = self.config.width as i32;
        let height = self.config.height as i32;

        for dy in -FOOD_SENSOR_RADIUS..=FOOD_SENSOR_RADIUS {
            for dx in -FOOD_SENSOR_RADIUS..=FOOD_SENSOR_RADIUS {
                let raw_x = x as i32 + dx;
                let raw_y = y as i32 + dy;
                let Some((nx, ny)) = (if self.config.world_wrap {
                    Some((
                        wrap_axis(raw_x, self.config.width),
                        wrap_axis(raw_y, self.config.height),
                    ))
                } else if raw_x < 0 || raw_x >= width || raw_y < 0 || raw_y >= height {
                    None
                } else {
                    Some((raw_x as u32, raw_y as u32))
                }) else {
                    continue;
                };
                let idx = self.idx(nx, ny);

                if dx != 0 || dy != 0 {
                    scanned_cells += 1;
                }

                if self.cells[idx].food > 0.0 {
                    let dist_sq = dx * dx + dy * dy;
                    match food_best {
                        Some((best_dist_sq, _, _)) if dist_sq >= best_dist_sq => {}
                        _ => food_best = Some((dist_sq, dx, dy)),
                    }
                }

                if let Some(other_id) = self.creature_at[idx] {
                    if Some(other_id) != self_id {
                        if dx != 0 || dy != 0 {
                            occupied_cells += 1;
                        }
                        let dist_sq = dx * dx + dy * dy;
                        match creature_best {
                            Some((best_dist_sq, _, _)) if dist_sq >= best_dist_sq => {}
                            _ => creature_best = Some((dist_sq, dx, dy)),
                        }
                    }
                }
            }
        }

        let (food_direction, food_distance) = sensor_from_best(food_best);
        let (creature_direction, creature_distance) = sensor_from_best(creature_best);
        let local_density = if scanned_cells == 0 {
            0.0
        } else {
            (occupied_cells as f32 / scanned_cells as f32).clamp(0.0, 1.0)
        };

        PerceptionScan {
            food_direction,
            food_distance,
            creature_direction,
            creature_distance,
            local_density,
        }
    }

    #[cfg(test)]
    pub(super) fn nearest_food_sensor(&self, x: u32, y: u32) -> (f32, f32) {
        let scan = self.scan_perception(x, y, None);
        (scan.food_direction, scan.food_distance)
    }
}
