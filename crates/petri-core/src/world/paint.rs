use std::collections::HashSet;

use super::*;

impl World {
    pub fn apply_paint_stroke(
        &mut self,
        tool: PaintTool,
        brush_half_extent: u8,
        points: &[PaintPoint],
    ) -> Result<PaintStats, PaintError> {
        if brush_half_extent > MAX_BRUSH_HALF_EXTENT {
            return Err(PaintError::InvalidBrushHalfExtent {
                received: brush_half_extent,
                max: MAX_BRUSH_HALF_EXTENT,
            });
        }

        let mut touched = HashSet::new();
        let radius = brush_half_extent as i32;
        for point in points {
            let px = point.x as i32;
            let py = point.y as i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let x = px + dx;
                    let y = py + dy;
                    if x < 0
                        || y < 0
                        || x >= self.config.width as i32
                        || y >= self.config.height as i32
                    {
                        continue;
                    }
                    touched.insert(self.idx(x as u32, y as u32));
                }
            }
        }

        let mut stats = PaintStats::default();
        for idx in touched {
            stats.affected_cells += 1;
            self.apply_paint_to_cell(idx, tool, &mut stats);
        }
        Ok(stats)
    }

    pub fn clear_painted_cells(&mut self) -> PaintStats {
        let mut stats = PaintStats::default();
        for cell in &mut self.cells {
            let mut changed = false;
            if cell.food > 0.0 {
                cell.food = 0.0;
                stats.food_cleared_cells += 1;
                changed = true;
            }
            if cell.barrier {
                cell.barrier = false;
                stats.barrier_cleared_cells += 1;
                changed = true;
            }
            if changed {
                stats.affected_cells += 1;
            }
        }
        stats
    }

    fn apply_paint_to_cell(&mut self, idx: usize, tool: PaintTool, stats: &mut PaintStats) {
        match tool {
            PaintTool::Food => {
                let target = self.config.food_max_density.max(0.0);
                if (self.cells[idx].food - target).abs() > f32::EPSILON {
                    self.cells[idx].food = target;
                    stats.food_set_cells += 1;
                }
            }
            PaintTool::Barrier => {
                if let Some(id) = self.creature_at[idx] {
                    self.creature_at[idx] = None;
                    self.creatures.remove(id);
                    stats.creatures_removed += 1;
                }
                if !self.cells[idx].barrier {
                    self.cells[idx].barrier = true;
                    stats.barrier_set_cells += 1;
                }
            }
            PaintTool::EraseFood => {
                if self.cells[idx].food > 0.0 {
                    self.cells[idx].food = 0.0;
                    stats.food_cleared_cells += 1;
                }
            }
            PaintTool::EraseBarrier => {
                if self.cells[idx].barrier {
                    self.cells[idx].barrier = false;
                    stats.barrier_cleared_cells += 1;
                }
            }
        }
    }
}
