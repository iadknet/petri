use rand::seq::index::sample;
use rand::Rng;

use super::helpers::wrap_axis;
use super::*;

impl World {
    pub fn seed_food_density(&mut self, density: f32) {
        let total_cells = self.cells.len();
        if total_cells == 0 {
            return;
        }

        for cell in &mut self.cells {
            cell.food = 0.0;
        }

        let clamped = density.clamp(0.0, 1.0);
        let target = ((clamped * total_cells as f32).round() as usize).min(total_cells);
        if target == 0 {
            return;
        }

        let sampled = sample(&mut self.rng, total_cells, target);
        for idx in sampled.iter() {
            self.cells[idx].food = self.config.food_max_density;
        }
    }
    pub(super) fn update_food(&mut self) {
        let total_cells = self.cells.len();
        if total_cells == 0 {
            return;
        }

        let growth_rate = self.config.food_growth_rate.max(0.0);
        let max_density = self.config.food_max_density.max(0.0);
        if max_density <= 0.0 {
            return;
        }

        let spread_threshold = max_density * self.config.food_spread_threshold.clamp(0.0, 1.0);
        let spawn_floor_density = self.config.food_spawn_floor_density.clamp(0.0, 1.0);
        let source_food = self
            .cells
            .iter()
            .map(|cell| cell.food.clamp(0.0, max_density))
            .collect::<Vec<_>>();
        let total_food: f32 = source_food.iter().sum();
        let average_density = (total_food / (total_cells as f32 * max_density)).clamp(0.0, 1.0);
        let width = self.config.width as i32;
        let height = self.config.height as i32;
        const NEIGHBOR_OFFSETS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

        for (idx, source) in source_food.iter().copied().enumerate() {
            if self.creature_at[idx].is_some() || self.cells[idx].barrier {
                continue;
            }

            let delta = source * growth_rate;
            if delta > 0.0 {
                self.cells[idx].food = (self.cells[idx].food + delta).min(max_density);
            }

            if source < spread_threshold || delta <= 0.0 {
                continue;
            }

            let x = (idx as u32 % self.config.width) as i32;
            let y = (idx as u32 / self.config.width) as i32;
            let mut neighbors = [0_usize; 4];
            let mut neighbor_count = 0_usize;

            for (dx, dy) in NEIGHBOR_OFFSETS {
                let raw_x = x + dx;
                let raw_y = y + dy;
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
                let neighbor_idx = self.idx(nx, ny);
                if self.creature_at[neighbor_idx].is_some() || self.cells[neighbor_idx].barrier {
                    continue;
                }
                neighbors[neighbor_count] = neighbor_idx;
                neighbor_count += 1;
            }

            if neighbor_count == 0 {
                continue;
            }

            let target_idx = neighbors[self.rng.gen_range(0..neighbor_count)];
            self.cells[target_idx].food = (self.cells[target_idx].food + delta).min(max_density);
        }

        if average_density >= spawn_floor_density {
            return;
        }

        let spawn_attempts = (total_cells as f32 * self.config.food_spawn_rate).round() as usize;
        let spawn_delta = max_density * growth_rate;
        if spawn_attempts == 0 || spawn_delta <= 0.0 {
            return;
        }

        for _ in 0..spawn_attempts {
            let idx = self.rng.gen_range(0..total_cells);
            if self.creature_at[idx].is_some() || self.cells[idx].barrier {
                continue;
            }
            let cell = &mut self.cells[idx];
            cell.food = (cell.food + spawn_delta).min(max_density);
        }
    }
}
