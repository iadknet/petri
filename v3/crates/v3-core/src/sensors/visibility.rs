//! Visibility tables for extended perception LOS computation.
//!
//! Per v3-sensor-spec.md Section 4: visibility is computed in local-offset space
//! using strict-corner blocking. Barriers are visible and opaque; cells beyond
//! are hidden.
//!
//! Tables are process-wide cached per radius (1..=8) and reused across ticks.

use crate::contracts::Position;
use crate::kernel::WorldState;
use std::sync::OnceLock;

/// Maximum supported vision radius.
pub const MAX_VISION_RADIUS: u8 = 8;

/// A precomputed ray from origin `(0,0)` to a target offset `(dx, dy)`.
///
/// Steps are ordered from closest to farthest. Each step is `(dx, dy)` in
/// local-offset space. The ray starts at step 0 = `(signum_dx, signum_dy)` or
/// the first adjacent step, advancing toward the target.
#[derive(Debug, Clone)]
pub struct Ray {
    pub target_dx: i32,
    pub target_dy: i32,
    /// Ordered intermediate steps (not including origin).
    /// Each step is an absolute local offset from origin.
    pub steps: Vec<(i32, i32)>,
}

/// Precomputed visibility table for a given radius.
#[derive(Debug, Clone)]
pub struct VisibilityTable {
    pub radius: u8,
    /// All rays for offsets in [-r, r]×[-r, r], excluding (0,0).
    pub rays: Vec<Ray>,
}

/// Process-wide cached tables, indexed by `radius - 1`.
static TABLES: [OnceLock<VisibilityTable>; MAX_VISION_RADIUS as usize] = [
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
];

/// Get (or lazily compute) the visibility table for a given radius.
pub fn get_visibility_table(radius: u8) -> &'static VisibilityTable {
    let r = radius.clamp(1, MAX_VISION_RADIUS);
    let idx = (r - 1) as usize;
    TABLES[idx].get_or_init(|| build_visibility_table(r))
}

/// Build a visibility table by precomputing Bresenham-like rays from origin
/// to every offset in [-r, r]×[-r, r].
fn build_visibility_table(radius: u8) -> VisibilityTable {
    let r = radius as i32;
    let mut rays = Vec::new();

    for dy in -r..=r {
        for dx in -r..=r {
            if dx == 0 && dy == 0 {
                continue;
            }
            rays.push(build_ray(dx, dy));
        }
    }

    VisibilityTable { radius, rays }
}

/// Build a single ray from (0,0) to (target_dx, target_dy) using a
/// Bresenham-like line rasterization.
fn build_ray(target_dx: i32, target_dy: i32) -> Ray {
    let mut steps = Vec::new();

    let adx = target_dx.unsigned_abs() as i32;
    let ady = target_dy.unsigned_abs() as i32;
    let sx = if target_dx > 0 {
        1
    } else if target_dx < 0 {
        -1
    } else {
        0
    };
    let sy = if target_dy > 0 {
        1
    } else if target_dy < 0 {
        -1
    } else {
        0
    };

    let mut x = 0i32;
    let mut y = 0i32;

    if adx >= ady {
        let mut err = adx / 2;
        for _ in 0..adx {
            x += sx;
            err -= ady;
            if err < 0 {
                y += sy;
                err += adx;
            }
            steps.push((x, y));
        }
    } else {
        let mut err = ady / 2;
        for _ in 0..ady {
            y += sy;
            err -= adx;
            if err < 0 {
                x += sx;
                err += ady;
            }
            steps.push((x, y));
        }
    }

    Ray {
        target_dx,
        target_dy,
        steps,
    }
}

/// A set of visible cells for a creature at a given position.
///
/// Stores local offsets `(dx, dy)` of cells visible from origin.
pub struct VisibleCells {
    /// Visible offsets and their resolved world positions.
    cells: Vec<VisibleCell>,
}

/// A single visible cell with its local offset and resolved world position.
#[derive(Debug, Clone, Copy)]
pub struct VisibleCell {
    pub dx: i32,
    pub dy: i32,
    pub pos: Position,
}

impl VisibleCells {
    /// Iterate over all visible cells.
    pub fn iter(&self) -> impl Iterator<Item = &VisibleCell> {
        self.cells.iter()
    }

    /// Number of visible cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether the visible set is empty.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

/// Compute the set of visible cells for a creature at `origin` using the
/// given visibility table and world state.
///
/// Per v3-sensor-spec.md Section 4:
/// - Self cell (0,0) is always visible (included for food)
/// - Barriers are visible and opaque (cells beyond are hidden)
/// - Strict-corner rule for diagonal steps
pub fn compute_visible_cells(
    origin: Position,
    world: &WorldState,
    table: &VisibilityTable,
) -> VisibleCells {
    let r = table.radius as i32;
    let total = ((2 * r + 1) * (2 * r + 1)) as usize;

    // Track which offsets have been marked visible to avoid duplicates.
    // Use a flat grid indexed by (dx + r, dy + r).
    let side = (2 * r + 1) as usize;
    let mut seen = vec![false; side * side];
    let mut cells = Vec::with_capacity(total);

    // Self cell is always visible.
    let self_idx = (r as usize) * side + (r as usize);
    seen[self_idx] = true;
    cells.push(VisibleCell {
        dx: 0,
        dy: 0,
        pos: origin,
    });

    for ray in &table.rays {
        let mut prev_x = 0i32;
        let mut prev_y = 0i32;
        let mut blocked = false;

        for &(step_dx, step_dy) in &ray.steps {
            if blocked {
                break;
            }

            // Check strict-corner rule for diagonal steps.
            let is_diagonal = step_dx != prev_x && step_dy != prev_y;
            if is_diagonal {
                // The two orthogonal side cells from the previous position.
                let side1 = world.resolve_offset(origin, step_dx, prev_y);
                let side2 = world.resolve_offset(origin, prev_x, step_dy);

                let side1_blocks = side1.is_none_or(|p| world.is_barrier(p));
                let side2_blocks = side2.is_none_or(|p| world.is_barrier(p));

                if side1_blocks && side2_blocks {
                    break;
                }
            }

            // Resolve the step position.
            let Some(pos) = world.resolve_offset(origin, step_dx, step_dy) else {
                break;
            };

            // Mark as visible if not already seen.
            let grid_x = (step_dx + r) as usize;
            let grid_y = (step_dy + r) as usize;
            let idx = grid_y * side + grid_x;
            if !seen[idx] {
                seen[idx] = true;
                cells.push(VisibleCell {
                    dx: step_dx,
                    dy: step_dy,
                    pos,
                });
            }

            // If this cell is a barrier, it's visible but blocks further rays.
            if world.is_barrier(pos) {
                blocked = true;
            }

            prev_x = step_dx;
            prev_y = step_dy;
        }
    }

    VisibleCells { cells }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldEdgeMode;
    use crate::kernel::WorldState;

    #[test]
    fn visibility_table_has_correct_ray_count() {
        let table = get_visibility_table(1);
        // Radius 1: 3×3 - 1 = 8 rays
        assert_eq!(table.rays.len(), 8);

        let table = get_visibility_table(2);
        // Radius 2: 5×5 - 1 = 24 rays
        assert_eq!(table.rays.len(), 24);
    }

    #[test]
    fn visibility_table_radius_5_default() {
        let table = get_visibility_table(5);
        // Radius 5: 11×11 - 1 = 120 rays
        assert_eq!(table.rays.len(), 120);
        assert_eq!(table.radius, 5);
    }

    #[test]
    fn self_cell_always_visible() {
        let world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        let table = get_visibility_table(3);
        let origin = Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);
        assert!(visible.iter().any(|c| c.dx == 0 && c.dy == 0));
    }

    #[test]
    fn open_world_all_cells_visible() {
        let world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        let table = get_visibility_table(2);
        let origin = Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);
        // All 5×5 = 25 cells should be visible
        assert_eq!(visible.len(), 25);
    }

    #[test]
    fn barrier_blocks_cells_behind() {
        let mut world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        // Place barrier directly north at (10, 9)
        world.set_barrier(Position::new(10, 9), true);
        let table = get_visibility_table(3);
        let origin = Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);

        // The barrier cell itself should be visible
        assert!(
            visible.iter().any(|c| c.dx == 0 && c.dy == -1),
            "barrier cell should be visible"
        );

        // Cells directly behind the barrier should NOT be visible
        // (0, -2) and (0, -3) should be blocked
        assert!(
            !visible.iter().any(|c| c.dx == 0 && c.dy == -2),
            "(0,-2) should be blocked by barrier at (0,-1)"
        );
        assert!(
            !visible.iter().any(|c| c.dx == 0 && c.dy == -3),
            "(0,-3) should be blocked by barrier at (0,-1)"
        );
    }

    #[test]
    fn strict_corner_blocking_both_sides() {
        let mut world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        // Place barriers to create a strict-corner: both sides of diagonal NE
        // From origin (10,10), NE means dx=1, dy=-1
        // Side cells: (11, 10) and (10, 9)
        world.set_barrier(Position::new(11, 10), true); // east
        world.set_barrier(Position::new(10, 9), true); // north

        let table = get_visibility_table(2);
        let origin = Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);

        // The diagonal NE cell (1, -1) should be blocked because both
        // orthogonal side cells are barriers
        assert!(
            !visible.iter().any(|c| c.dx == 1 && c.dy == -1),
            "diagonal (1,-1) should be blocked by strict-corner rule"
        );
    }

    #[test]
    fn strict_corner_one_side_open_allows_diagonal() {
        let mut world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        // Only one side of diagonal NE is blocked
        world.set_barrier(Position::new(11, 10), true); // east blocked
                                                        // north (10, 9) is open

        let table = get_visibility_table(2);
        let origin = Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);

        // Diagonal NE (1, -1) should be visible — only one side blocked
        assert!(
            visible.iter().any(|c| c.dx == 1 && c.dy == -1),
            "diagonal (1,-1) should be visible when only one side is blocked"
        );
    }

    #[test]
    fn bounded_edge_hides_out_of_bounds() {
        let world = WorldState::new(5, 5, WorldEdgeMode::Bounded);
        let table = get_visibility_table(3);
        let origin = Position::new(0, 0);
        let visible = compute_visible_cells(origin, &world, table);

        // Cells at negative offsets should not be visible
        assert!(
            !visible.iter().any(|c| c.dx < 0 || c.dy < 0),
            "no negative-offset cells visible in bounded mode at (0,0)"
        );
    }

    #[test]
    fn ray_steps_end_at_target() {
        let ray = build_ray(3, 0);
        assert_eq!(ray.steps.last(), Some(&(3, 0)));

        let ray = build_ray(0, -2);
        assert_eq!(ray.steps.last(), Some(&(0, -2)));

        let ray = build_ray(2, 2);
        assert_eq!(ray.steps.last(), Some(&(2, 2)));
    }

    #[test]
    fn no_duplicate_visible_cells() {
        let world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        let table = get_visibility_table(5);
        let origin = Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);

        let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for cell in visible.iter() {
            assert!(
                seen.insert((cell.dx, cell.dy)),
                "duplicate visible cell ({}, {})",
                cell.dx,
                cell.dy
            );
        }
    }
}
