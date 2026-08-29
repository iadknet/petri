use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::contracts::{CreatureId, Position};
use crate::kernel::WorldState;

/// Tool to apply during a paint stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaintTool {
    Food,
    Barrier,
    EraseFood,
    EraseBarrier,
}

/// A single paint point in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintPoint {
    pub x: u16,
    pub y: u16,
}

/// Statistics returned after applying a paint stroke.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaintStats {
    pub affected_cells: usize,
    pub food_set_cells: usize,
    pub food_cleared_cells: usize,
    pub barrier_set_cells: usize,
    pub barrier_cleared_cells: usize,
    pub creatures_removed: usize,
}

impl WorldState {
    /// Apply a paint stroke across the given points, expanding each by `brush_half_extent`.
    ///
    /// Returns paint statistics and a list of creature IDs evicted by barrier placement.
    /// The caller is responsible for removing evicted creatures from any slotmap.
    pub fn apply_paint_stroke(
        &mut self,
        tool: PaintTool,
        brush_half_extent: u8,
        points: &[PaintPoint],
        max_density: f32,
    ) -> (PaintStats, Vec<CreatureId>) {
        let mut stats = PaintStats::default();
        let mut evicted = Vec::new();

        // Expand all points by brush radius and deduplicate.
        let cells = expand_brush(points, brush_half_extent, self.width, self.height);
        stats.affected_cells = cells.len();

        for (x, y) in cells {
            let pos = Position::new(x, y);
            match tool {
                PaintTool::Food => {
                    if !self.is_barrier(pos) {
                        self.set_food(pos, max_density);
                        stats.food_set_cells += 1;
                    }
                }
                PaintTool::Barrier => {
                    // Clear food on the cell.
                    if self.food_at(pos) > 0.0 {
                        self.set_food(pos, 0.0);
                        stats.food_cleared_cells += 1;
                    }
                    // Evict creature if present.
                    if let Some(id) = self.creature_at(pos) {
                        evicted.push(id);
                        self.remove_creature(pos);
                        stats.creatures_removed += 1;
                    }
                    self.set_barrier(pos, true);
                    stats.barrier_set_cells += 1;
                }
                PaintTool::EraseFood => {
                    if self.food_at(pos) > 0.0 {
                        self.set_food(pos, 0.0);
                        stats.food_cleared_cells += 1;
                    }
                }
                PaintTool::EraseBarrier => {
                    if self.is_barrier(pos) {
                        self.set_barrier(pos, false);
                        stats.barrier_cleared_cells += 1;
                    }
                }
            }
        }

        (stats, evicted)
    }
}

/// Expand paint points by brush radius and deduplicate, bounds-checking against world size.
fn expand_brush(
    points: &[PaintPoint],
    half_extent: u8,
    width: u16,
    height: u16,
) -> Vec<(u16, u16)> {
    let r = half_extent as i32;
    let mut seen = HashSet::with_capacity(points.len() * ((2 * r + 1) * (2 * r + 1)) as usize);
    let mut cells = Vec::with_capacity(seen.capacity());

    for p in points {
        for dy in -r..=r {
            for dx in -r..=r {
                let cx = p.x as i32 + dx;
                let cy = p.y as i32 + dy;
                if cx >= 0 && cx < width as i32 && cy >= 0 && cy < height as i32 {
                    let coord = (cx as u16, cy as u16);
                    if seen.insert(coord) {
                        cells.push(coord);
                    }
                }
            }
        }
    }

    cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldEdgeMode;
    use slotmap::SlotMap;

    fn small_world() -> WorldState {
        WorldState::new(10, 10, WorldEdgeMode::Wrap)
    }

    #[test]
    fn food_tool_sets_max_density_on_empty_cells() {
        let mut w = small_world();
        let points = vec![PaintPoint { x: 3, y: 3 }];
        let (stats, evicted) = w.apply_paint_stroke(PaintTool::Food, 0, &points, 1.0);

        assert_eq!(stats.affected_cells, 1);
        assert_eq!(stats.food_set_cells, 1);
        assert!(evicted.is_empty());
        assert!((w.food_at(Position::new(3, 3)) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn food_tool_skips_barrier_cells() {
        let mut w = small_world();
        let pos = Position::new(5, 5);
        w.set_barrier(pos, true);
        let points = vec![PaintPoint { x: 5, y: 5 }];
        let (stats, _) = w.apply_paint_stroke(PaintTool::Food, 0, &points, 1.0);

        assert_eq!(stats.food_set_cells, 0);
        assert!((w.food_at(pos)).abs() < f32::EPSILON);
    }

    #[test]
    fn barrier_tool_evicts_creature_and_clears_food() {
        let mut w = small_world();
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let pos = Position::new(4, 4);
        w.place_creature(pos, id);
        w.set_food(pos, 0.5);

        let points = vec![PaintPoint { x: 4, y: 4 }];
        let (stats, evicted) = w.apply_paint_stroke(PaintTool::Barrier, 0, &points, 1.0);

        assert!(w.is_barrier(pos));
        assert_eq!(evicted, vec![id]);
        assert_eq!(stats.creatures_removed, 1);
        assert_eq!(stats.food_cleared_cells, 1);
        assert!(w.creature_at(pos).is_none());
        assert!((w.food_at(pos)).abs() < f32::EPSILON);
    }

    #[test]
    fn erase_food_clears_food() {
        let mut w = small_world();
        let pos = Position::new(2, 2);
        w.set_food(pos, 0.8);
        let points = vec![PaintPoint { x: 2, y: 2 }];
        let (stats, _) = w.apply_paint_stroke(PaintTool::EraseFood, 0, &points, 1.0);

        assert_eq!(stats.food_cleared_cells, 1);
        assert!((w.food_at(pos)).abs() < f32::EPSILON);
    }

    #[test]
    fn erase_barrier_clears_barrier() {
        let mut w = small_world();
        let pos = Position::new(7, 7);
        w.set_barrier(pos, true);
        let points = vec![PaintPoint { x: 7, y: 7 }];
        let (stats, _) = w.apply_paint_stroke(PaintTool::EraseBarrier, 0, &points, 1.0);

        assert_eq!(stats.barrier_cleared_cells, 1);
        assert!(!w.is_barrier(pos));
    }

    #[test]
    fn brush_expands_3x3() {
        let mut w = small_world();
        let points = vec![PaintPoint { x: 5, y: 5 }];
        let (stats, _) = w.apply_paint_stroke(PaintTool::Barrier, 1, &points, 1.0);

        assert_eq!(stats.affected_cells, 9);
        assert_eq!(stats.barrier_set_cells, 9);
        // Check all 9 cells are barriers.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let pos = Position::new((5 + dx) as u16, (5 + dy) as u16);
                assert!(w.is_barrier(pos));
            }
        }
    }

    #[test]
    fn brush_clamps_at_world_edge() {
        let mut w = small_world();
        let points = vec![PaintPoint { x: 0, y: 0 }];
        let (stats, _) = w.apply_paint_stroke(PaintTool::Barrier, 1, &points, 1.0);

        // Corner: only 4 cells in bounds (0,0), (1,0), (0,1), (1,1).
        assert_eq!(stats.affected_cells, 4);
        assert_eq!(stats.barrier_set_cells, 4);
    }

    #[test]
    fn duplicate_points_are_deduplicated() {
        let mut w = small_world();
        let points = vec![
            PaintPoint { x: 3, y: 3 },
            PaintPoint { x: 3, y: 3 },
            PaintPoint { x: 3, y: 3 },
        ];
        let (stats, _) = w.apply_paint_stroke(PaintTool::Barrier, 0, &points, 1.0);

        assert_eq!(stats.affected_cells, 1);
        assert_eq!(stats.barrier_set_cells, 1);
    }

    #[test]
    fn brush_5x5_center_of_world() {
        let mut w = small_world();
        let points = vec![PaintPoint { x: 5, y: 5 }];
        let (stats, _) = w.apply_paint_stroke(PaintTool::Food, 2, &points, 1.0);

        assert_eq!(stats.affected_cells, 25);
        assert_eq!(stats.food_set_cells, 25);
    }
}
