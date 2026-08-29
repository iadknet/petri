use std::collections::HashSet;

use rand::Rng;

use crate::kernel::paint::PaintPoint;

use super::PatternBounds;

/// Generate a maze pattern using recursive backtracker (depth-first).
pub(super) fn generate(
    bounds: PatternBounds,
    corridor_width: u8,
    wall_thickness: u8,
    open_center_radius: u8,
    rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    let cw = corridor_width.max(1) as u16;
    let wt = wall_thickness.max(1) as u16;
    let cell_size = cw + wt;

    // Grid dimensions (minimum 2x2 for a meaningful maze).
    let cols = ((bounds.width.saturating_sub(wt)) / cell_size).max(2) as usize;
    let rows = ((bounds.height.saturating_sub(wt)) / cell_size).max(2) as usize;

    // Carve maze: visited[row][col]
    let mut visited = vec![vec![false; cols]; rows];
    let mut walls = vec![vec![[true; 4]; cols]; rows]; // [N, E, S, W]

    // Recursive backtracker using an explicit stack to avoid stack overflow.
    let start_row = rng.gen_range(0..rows);
    let start_col = rng.gen_range(0..cols);
    let mut stack = vec![(start_row, start_col)];
    visited[start_row][start_col] = true;

    while let Some(&(row, col)) = stack.last() {
        // Fixed-size buffer avoids heap allocation per iteration (max 4 neighbors).
        let mut neighbors = [(0usize, 0usize, 0usize, 0usize); 4];
        let mut count = 0;
        // N
        if row > 0 && !visited[row - 1][col] {
            neighbors[count] = (row - 1, col, 0, 2); // dir=N, opposite=S
            count += 1;
        }
        // E
        if col + 1 < cols && !visited[row][col + 1] {
            neighbors[count] = (row, col + 1, 1, 3);
            count += 1;
        }
        // S
        if row + 1 < rows && !visited[row + 1][col] {
            neighbors[count] = (row + 1, col, 2, 0);
            count += 1;
        }
        // W
        if col > 0 && !visited[row][col - 1] {
            neighbors[count] = (row, col - 1, 3, 1);
            count += 1;
        }

        if count == 0 {
            stack.pop();
        } else {
            let idx = rng.gen_range(0..count);
            let (nr, nc, dir, opp) = neighbors[idx];
            walls[row][col][dir] = false;
            walls[nr][nc][opp] = false;
            visited[nr][nc] = true;
            stack.push((nr, nc));
        }
    }

    // Render walls into cell positions.
    let total_grid_w = cols as u16 * cell_size + wt;
    let total_grid_h = rows as u16 * cell_size + wt;
    // Estimate: roughly half the grid area will be walls.
    let estimated_walls =
        (total_grid_w.min(bounds.width) as usize * total_grid_h.min(bounds.height) as usize) / 2;
    let mut cells = HashSet::with_capacity(estimated_walls);

    // Add all wall cells within the grid area.
    for gy in 0..total_grid_h.min(bounds.height) {
        for gx in 0..total_grid_w.min(bounds.width) {
            // Compute grid cell indices and local offsets.
            // When gx < wt or gy < wt, we're in the border wall region;
            // local_x/y are set to cell_size to force the wall case.
            let (in_col, local_x) = if gx >= wt {
                ((gx - wt) / cell_size, (gx - wt) % cell_size)
            } else {
                (0, cell_size) // border wall: force wall, col index unused
            };
            let (in_row, local_y) = if gy >= wt {
                ((gy - wt) / cell_size, (gy - wt) % cell_size)
            } else {
                (0, cell_size) // border wall: force wall, row index unused
            };

            let is_corridor_x = local_x < cw;
            let is_corridor_y = local_y < cw;

            let is_wall = if is_corridor_x && is_corridor_y {
                // Inside a cell: always corridor (not wall)
                false
            } else if is_corridor_x && !is_corridor_y {
                // Horizontal wall strip between row and row+1
                // This is the south wall of in_row
                let r = in_row as usize;
                let c = in_col as usize;
                r < rows && c < cols && walls[r][c][2] // south wall
            } else if !is_corridor_x && is_corridor_y {
                // Vertical wall strip between col and col+1
                let r = in_row as usize;
                let c = in_col as usize;
                r < rows && c < cols && walls[r][c][1] // east wall
            } else {
                // Corner/intersection: wall if any adjacent wall is present
                true
            };

            if is_wall {
                let wx = bounds.x + gx;
                let wy = bounds.y + gy;
                cells.insert((wx, wy));
            }
        }
    }

    // Clear open center.
    if open_center_radius > 0 {
        let cx = bounds.x as f64 + bounds.width as f64 / 2.0;
        let cy = bounds.y as f64 + bounds.height as f64 / 2.0;
        let r2 = (open_center_radius as f64) * (open_center_radius as f64);
        cells.retain(|&(x, y)| {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            dx * dx + dy * dy > r2
        });
    }

    // Add edge openings (one corridor-width gap per edge).
    add_edge_openings(bounds, cw, &mut cells, rng);

    cells
        .into_iter()
        .map(|(x, y)| PaintPoint { x, y })
        .collect()
}

/// Clear a corridor-width gap on each edge of the bounds.
fn add_edge_openings(
    bounds: PatternBounds,
    corridor_width: u16,
    cells: &mut HashSet<(u16, u16)>,
    rng: &mut impl Rng,
) {
    let cw = corridor_width.min(bounds.width).min(bounds.height);
    if cw == 0 {
        return;
    }

    // North edge: pick a random x position for the opening
    let max_x = bounds.width.saturating_sub(cw);
    let open_x = if max_x > 0 {
        bounds.x + rng.gen_range(0..=max_x)
    } else {
        bounds.x
    };
    for dx in 0..cw {
        cells.remove(&(open_x + dx, bounds.y));
    }

    // South edge
    let south_y = bounds.y + bounds.height - 1;
    let open_x = if max_x > 0 {
        bounds.x + rng.gen_range(0..=max_x)
    } else {
        bounds.x
    };
    for dx in 0..cw {
        cells.remove(&(open_x + dx, south_y));
    }

    // West edge
    let max_y = bounds.height.saturating_sub(cw);
    let open_y = if max_y > 0 {
        bounds.y + rng.gen_range(0..=max_y)
    } else {
        bounds.y
    };
    for dy in 0..cw {
        cells.remove(&(bounds.x, open_y + dy));
    }

    // East edge
    let east_x = bounds.x + bounds.width - 1;
    let open_y = if max_y > 0 {
        bounds.y + rng.gen_range(0..=max_y)
    } else {
        bounds.y
    };
    for dy in 0..cw {
        cells.remove(&(east_x, open_y + dy));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn test_bounds() -> PatternBounds {
        PatternBounds {
            x: 0,
            y: 0,
            width: 40,
            height: 40,
        }
    }

    #[test]
    fn maze_produces_non_empty_output() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 2, 1, 0, &mut rng);
        assert!(!result.is_empty(), "maze should produce wall cells");
    }

    #[test]
    fn maze_all_cells_within_bounds() {
        let bounds = PatternBounds {
            x: 5,
            y: 10,
            width: 30,
            height: 25,
        };
        let mut rng = SmallRng::seed_from_u64(99);
        let result = generate(bounds, 2, 1, 0, &mut rng);
        for p in &result {
            assert!(
                p.x >= bounds.x && p.x < bounds.x + bounds.width,
                "x={} out of bounds [{}, {})",
                p.x,
                bounds.x,
                bounds.x + bounds.width
            );
            assert!(
                p.y >= bounds.y && p.y < bounds.y + bounds.height,
                "y={} out of bounds [{}, {})",
                p.y,
                bounds.y,
                bounds.y + bounds.height
            );
        }
    }

    #[test]
    fn maze_no_duplicate_cells() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 2, 1, 0, &mut rng);
        let set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(set.len(), result.len(), "output should have no duplicates");
    }

    #[test]
    fn maze_corridors_are_connected() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 2, 1, 0, &mut rng);
        let wall_set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();

        // Find all non-wall cells within the grid area.
        let mut open_cells = Vec::new();
        for y in bounds.y..bounds.y + bounds.height {
            for x in bounds.x..bounds.x + bounds.width {
                if !wall_set.contains(&(x, y)) {
                    open_cells.push((x, y));
                }
            }
        }

        if open_cells.is_empty() {
            return;
        }

        // BFS from first open cell.
        let mut visited_bfs = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(open_cells[0]);
        visited_bfs.insert(open_cells[0]);

        while let Some((cx, cy)) = queue.pop_front() {
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= bounds.x as i32
                    && nx < (bounds.x + bounds.width) as i32
                    && ny >= bounds.y as i32
                    && ny < (bounds.y + bounds.height) as i32
                {
                    let np = (nx as u16, ny as u16);
                    if !wall_set.contains(&np) && visited_bfs.insert(np) {
                        queue.push_back(np);
                    }
                }
            }
        }

        assert_eq!(
            visited_bfs.len(),
            open_cells.len(),
            "all corridor cells should be reachable (connected)"
        );
    }

    #[test]
    fn maze_has_edge_openings() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 2, 1, 0, &mut rng);
        let wall_set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();

        // Each edge should have at least one non-wall cell.
        let north_open =
            (bounds.x..bounds.x + bounds.width).any(|x| !wall_set.contains(&(x, bounds.y)));
        let south_open = (bounds.x..bounds.x + bounds.width)
            .any(|x| !wall_set.contains(&(x, bounds.y + bounds.height - 1)));
        let west_open =
            (bounds.y..bounds.y + bounds.height).any(|y| !wall_set.contains(&(bounds.x, y)));
        let east_open = (bounds.y..bounds.y + bounds.height)
            .any(|y| !wall_set.contains(&(bounds.x + bounds.width - 1, y)));

        assert!(north_open, "north edge should have an opening");
        assert!(south_open, "south edge should have an opening");
        assert!(west_open, "west edge should have an opening");
        assert!(east_open, "east edge should have an opening");
    }

    #[test]
    fn maze_open_center_clears_center_cells() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 2, 1, 5, &mut rng);
        let wall_set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();

        let cx = bounds.x as f64 + bounds.width as f64 / 2.0;
        let cy = bounds.y as f64 + bounds.height as f64 / 2.0;
        let r2 = 5.0 * 5.0;

        for &(wx, wy) in &wall_set {
            let dx = wx as f64 + 0.5 - cx;
            let dy = wy as f64 + 0.5 - cy;
            assert!(
                dx * dx + dy * dy > r2,
                "wall at ({},{}) is within open center radius",
                wx,
                wy
            );
        }
    }

    #[test]
    fn maze_deterministic_with_same_seed() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(123);
        let mut rng2 = SmallRng::seed_from_u64(123);
        let r1 = generate(bounds, 2, 1, 0, &mut rng1);
        let r2 = generate(bounds, 2, 1, 0, &mut rng2);
        assert_eq!(r1.len(), r2.len());
        let s1: HashSet<(u16, u16)> = r1.iter().map(|p| (p.x, p.y)).collect();
        let s2: HashSet<(u16, u16)> = r2.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(s1, s2);
    }
}
