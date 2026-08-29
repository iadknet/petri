use std::collections::HashSet;
use std::f64::consts::PI;

use rand::Rng;

use crate::kernel::paint::PaintPoint;

use super::PatternBounds;

/// Generate a star pattern with radiating rays from center points.
pub(super) fn generate(
    bounds: PatternBounds,
    point_count: u8,
    ray_count: u8,
    ray_length: u8,
    ray_thickness: u8,
    rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    let point_count = point_count.max(1) as usize;
    let ray_count = ray_count.max(2);
    let ray_length = ray_length.max(1) as f64;
    let ray_thickness = ray_thickness.max(1) as i32;
    let half_t = ray_thickness / 2;

    let centers = distribute_centers(bounds, point_count, rng);
    let angular_spacing = 2.0 * PI / ray_count as f64;
    let jitter_range = angular_spacing * 0.05;

    let estimated = point_count * ray_count as usize * ray_length as usize * ray_thickness as usize;
    let mut cells = HashSet::with_capacity(estimated);

    for (cx, cy) in &centers {
        let cx = *cx as f64 + 0.5;
        let cy = *cy as f64 + 0.5;

        for j in 0..ray_count {
            let base_angle = j as f64 * angular_spacing;
            let jitter = rng.gen_range(-jitter_range..=jitter_range);
            let theta = base_angle + jitter;

            let dir_x = theta.cos();
            let dir_y = theta.sin();
            // Normal perpendicular to ray direction.
            let nx = -theta.sin();
            let ny = theta.cos();

            for t in 0..=(ray_length as i32) {
                let px = cx + t as f64 * dir_x;
                let py = cy + t as f64 * dir_y;

                // Place thickness band.
                for w in -half_t..=(half_t + ray_thickness % 2 - 1) {
                    let cell_x = (px + w as f64 * nx).floor() as i32;
                    let cell_y = (py + w as f64 * ny).floor() as i32;

                    if cell_x >= bounds.x as i32
                        && cell_x < (bounds.x + bounds.width) as i32
                        && cell_y >= bounds.y as i32
                        && cell_y < (bounds.y + bounds.height) as i32
                    {
                        cells.insert((cell_x as u16, cell_y as u16));
                    }
                }
            }
        }
    }

    cells
        .into_iter()
        .map(|(x, y)| PaintPoint { x, y })
        .collect()
}

/// Distribute center points within bounds.
fn distribute_centers(bounds: PatternBounds, count: usize, rng: &mut impl Rng) -> Vec<(u16, u16)> {
    if count == 1 {
        return vec![(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)];
    }

    // Poisson-like distribution: try to place points with minimum distance.
    let min_dist = (bounds.width.min(bounds.height) as f64) / (count as f64 + 1.0);
    let min_dist2 = min_dist * min_dist;
    let max_attempts = count * 30;

    let mut centers = Vec::with_capacity(count);
    let mut attempts = 0;

    while centers.len() < count && attempts < max_attempts {
        let x = bounds.x + rng.gen_range(0..bounds.width);
        let y = bounds.y + rng.gen_range(0..bounds.height);
        attempts += 1;

        let too_close = centers.iter().any(|&(cx, cy): &(u16, u16)| {
            let dx = x as f64 - cx as f64;
            let dy = y as f64 - cy as f64;
            dx * dx + dy * dy < min_dist2
        });

        if !too_close {
            centers.push((x, y));
        }
    }

    // Fallback: fill remaining with random placement.
    while centers.len() < count {
        let x = bounds.x + rng.gen_range(0..bounds.width);
        let y = bounds.y + rng.gen_range(0..bounds.height);
        centers.push((x, y));
    }

    centers
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
            width: 60,
            height: 60,
        }
    }

    #[test]
    fn star_produces_non_empty_output() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 1, 8, 20, 1, &mut rng);
        assert!(!result.is_empty(), "star should produce barrier cells");
    }

    #[test]
    fn star_all_cells_within_bounds() {
        let bounds = PatternBounds {
            x: 10,
            y: 5,
            width: 50,
            height: 50,
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 1, 8, 20, 1, &mut rng);
        for p in &result {
            assert!(p.x >= bounds.x && p.x < bounds.x + bounds.width);
            assert!(p.y >= bounds.y && p.y < bounds.y + bounds.height);
        }
    }

    #[test]
    fn star_no_duplicate_cells() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 1, 8, 20, 1, &mut rng);
        let set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(set.len(), result.len());
    }

    #[test]
    fn star_multiple_centers_produce_more_cells() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let r1 = generate(bounds, 1, 8, 15, 1, &mut rng1);
        let mut rng3 = SmallRng::seed_from_u64(42);
        let r3 = generate(bounds, 3, 8, 15, 1, &mut rng3);
        assert!(
            r3.len() > r1.len(),
            "3 centers ({}) should produce more cells than 1 center ({})",
            r3.len(),
            r1.len()
        );
    }

    #[test]
    fn star_single_center_at_bounds_center() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        let centers = distribute_centers(bounds, 1, &mut rng);
        assert_eq!(centers.len(), 1);
        assert_eq!(
            centers[0],
            (bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
        );
    }

    #[test]
    fn star_multiple_centers_distributed() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        let centers = distribute_centers(bounds, 4, &mut rng);
        assert_eq!(centers.len(), 4);
        // All centers should be within bounds.
        for &(x, y) in &centers {
            assert!(x >= bounds.x && x < bounds.x + bounds.width);
            assert!(y >= bounds.y && y < bounds.y + bounds.height);
        }
    }

    #[test]
    fn star_deterministic() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let mut rng2 = SmallRng::seed_from_u64(42);
        let r1 = generate(bounds, 2, 8, 20, 1, &mut rng1);
        let r2 = generate(bounds, 2, 8, 20, 1, &mut rng2);
        assert_eq!(r1.len(), r2.len());
        let s1: HashSet<(u16, u16)> = r1.iter().map(|p| (p.x, p.y)).collect();
        let s2: HashSet<(u16, u16)> = r2.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(s1, s2);
    }
}
