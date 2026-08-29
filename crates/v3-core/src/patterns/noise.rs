use std::collections::HashSet;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::kernel::paint::PaintPoint;

use super::PatternBounds;

/// Generate a clustered random noise pattern.
pub(super) fn generate(
    bounds: PatternBounds,
    density: f32,
    cluster_size: u8,
    rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    let density = density.clamp(0.0, 1.0);
    let total = bounds.width as u32 * bounds.height as u32;
    let target = (total as f64 * density as f64).round() as usize;

    if target == 0 {
        return Vec::new();
    }

    let cluster_size = cluster_size.max(1);
    let mut cells = HashSet::with_capacity(target);

    if cluster_size <= 1 {
        // Point noise: randomly select unique positions.
        while cells.len() < target && cells.len() < total as usize {
            let x = bounds.x + rng.gen_range(0..bounds.width);
            let y = bounds.y + rng.gen_range(0..bounds.height);
            cells.insert((x, y));
        }
    } else {
        // Clustered noise: seed points with quadratic falloff.
        let cluster_area =
            std::f64::consts::PI / 4.0 * (cluster_size as f64 * 2.0) * (cluster_size as f64 * 2.0);
        let seed_count = (target as f64 / cluster_area.max(1.0)).ceil() as usize;

        for _ in 0..seed_count.max(1) {
            let sx = bounds.x + rng.gen_range(0..bounds.width);
            let sy = bounds.y + rng.gen_range(0..bounds.height);
            let r = cluster_size as i32;

            for dy in -r..=r {
                for dx in -r..=r {
                    let cx = sx as i32 + dx;
                    let cy = sy as i32 + dy;
                    if cx < bounds.x as i32
                        || cx >= (bounds.x + bounds.width) as i32
                        || cy < bounds.y as i32
                        || cy >= (bounds.y + bounds.height) as i32
                    {
                        continue;
                    }

                    let dist2 = (dx * dx + dy * dy) as f64;
                    let max_dist2 = (r * r) as f64;
                    if dist2 > max_dist2 {
                        continue;
                    }

                    // Quadratic falloff probability.
                    let prob = 1.0 - dist2 / max_dist2;
                    if rng.gen::<f64>() < prob {
                        cells.insert((cx as u16, cy as u16));
                    }
                }
            }
        }

        // Adjust to target: add random cells if under, remove if over.
        while cells.len() < target && cells.len() < total as usize {
            let x = bounds.x + rng.gen_range(0..bounds.width);
            let y = bounds.y + rng.gen_range(0..bounds.height);
            cells.insert((x, y));
        }

        if cells.len() > target {
            let mut vec: Vec<(u16, u16)> = cells.into_iter().collect();
            vec.shuffle(rng);
            vec.truncate(target);
            cells = vec.into_iter().collect();
        }
    }

    cells
        .into_iter()
        .map(|(x, y)| PaintPoint { x, y })
        .collect()
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
            width: 100,
            height: 100,
        }
    }

    #[test]
    fn noise_produces_non_empty_output() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 0.15, 3, &mut rng);
        assert!(!result.is_empty(), "noise should produce barrier cells");
    }

    #[test]
    fn noise_all_cells_within_bounds() {
        let bounds = PatternBounds {
            x: 10,
            y: 20,
            width: 80,
            height: 60,
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 0.15, 3, &mut rng);
        for p in &result {
            assert!(p.x >= bounds.x && p.x < bounds.x + bounds.width);
            assert!(p.y >= bounds.y && p.y < bounds.y + bounds.height);
        }
    }

    #[test]
    fn noise_density_within_tolerance() {
        let bounds = test_bounds();
        let area = bounds.width as f64 * bounds.height as f64;
        let density = 0.15;
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, density, 3, &mut rng);
        let actual_density = result.len() as f64 / area;
        let tolerance = 0.10; // within 10% of target
        assert!(
            (actual_density - density as f64).abs() < tolerance,
            "density {:.3} should be within {:.3} of {:.3}",
            actual_density,
            tolerance,
            density
        );
    }

    #[test]
    fn noise_no_duplicate_cells() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 0.15, 3, &mut rng);
        let set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(set.len(), result.len());
    }

    #[test]
    fn noise_zero_density_returns_empty() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 0.0, 3, &mut rng);
        assert!(result.is_empty());
    }

    #[test]
    fn noise_point_mode_cluster_size_1() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 0.1, 1, &mut rng);
        assert!(!result.is_empty());
        let expected = (100.0 * 100.0 * 0.1) as usize;
        assert_eq!(result.len(), expected);
    }

    #[test]
    fn noise_deterministic() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let mut rng2 = SmallRng::seed_from_u64(42);
        let r1 = generate(bounds, 0.15, 3, &mut rng1);
        let r2 = generate(bounds, 0.15, 3, &mut rng2);
        assert_eq!(r1.len(), r2.len());
        let s1: HashSet<(u16, u16)> = r1.iter().map(|p| (p.x, p.y)).collect();
        let s2: HashSet<(u16, u16)> = r2.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(s1, s2);
    }
}
