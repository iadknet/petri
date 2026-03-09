use std::collections::HashSet;
use std::f64::consts::PI;

use rand::Rng;

use crate::kernel::paint::PaintPoint;

use super::PatternBounds;

/// Generate an Archimedean spiral pattern.
pub(super) fn generate(
    bounds: PatternBounds,
    arm_count: u8,
    arm_thickness: u8,
    gap_width: u8,
    clockwise: bool,
    open_center_radius: u8,
    _rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    let arm_count = arm_count.max(1);
    let arm_thickness = arm_thickness.max(1);
    let gap_width = gap_width.max(1);

    let cx = bounds.x as f64 + bounds.width as f64 / 2.0;
    let cy = bounds.y as f64 + bounds.height as f64 / 2.0;
    let max_r = (bounds.width.min(bounds.height) as f64) / 2.0;
    let thickness_r = arm_thickness as f64 / 2.0;
    let dir: f64 = if clockwise { 1.0 } else { -1.0 };
    let cycle = (gap_width as f64 + arm_thickness as f64).max(1.0);

    // Estimate: each arm traces ~max_r steps, placing ~arm_thickness cells each.
    let estimated = arm_count as usize * max_r as usize * arm_thickness as usize;
    let mut cells = HashSet::with_capacity(estimated);

    for i in 0..arm_count {
        let theta_0 = i as f64 * (2.0 * PI / arm_count as f64);
        let mut t = 0.0_f64;

        loop {
            let r = t;
            if r > max_r {
                break;
            }

            if r >= open_center_radius as f64 {
                let theta = theta_0 + dir * t * (2.0 * PI / cycle);
                let px = cx + r * theta.cos();
                let py = cy + r * theta.sin();

                // Place disk of arm_thickness/2 radius.
                let ir = thickness_r.ceil() as i32;
                for dy in -ir..=ir {
                    for dx in -ir..=ir {
                        let dist2 = (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64);
                        if dist2 <= thickness_r * thickness_r {
                            let cell_x = (px + dx as f64).floor() as i32;
                            let cell_y = (py + dy as f64).floor() as i32;
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

            // Adaptive step to avoid gaps near center.
            let step = 0.5_f64.min(0.5 / r.max(1.0));
            t += step;
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
            width: 60,
            height: 60,
        }
    }

    #[test]
    fn spiral_produces_non_empty_output() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 3, 2, 4, true, 0, &mut rng);
        assert!(!result.is_empty(), "spiral should produce barrier cells");
    }

    #[test]
    fn spiral_all_cells_within_bounds() {
        let bounds = PatternBounds {
            x: 10,
            y: 5,
            width: 50,
            height: 50,
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 3, 2, 4, true, 0, &mut rng);
        for p in &result {
            assert!(p.x >= bounds.x && p.x < bounds.x + bounds.width);
            assert!(p.y >= bounds.y && p.y < bounds.y + bounds.height);
        }
    }

    #[test]
    fn spiral_no_duplicate_cells() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 3, 2, 4, true, 0, &mut rng);
        let set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(set.len(), result.len());
    }

    #[test]
    fn spiral_open_center_clears_center() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 3, 2, 4, true, 10, &mut rng);
        let cx = bounds.x as f64 + bounds.width as f64 / 2.0;
        let cy = bounds.y as f64 + bounds.height as f64 / 2.0;
        // With open_center_radius=10, arm_thickness=2 (radius 1.0), and
        // floating-point rounding, cells can be ~thickness_r + 1.0 closer.
        let min_r = 10.0 - 2.0; // thickness_r + rounding tolerance
        for p in &result {
            let dx = p.x as f64 + 0.5 - cx;
            let dy = p.y as f64 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(
                dist >= min_r,
                "cell ({},{}) at dist {:.1} is within open center",
                p.x,
                p.y,
                dist
            );
        }
    }

    #[test]
    fn spiral_multiple_arms_produce_more_cells() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let r1 = generate(bounds, 1, 2, 4, true, 0, &mut rng1);
        let mut rng3 = SmallRng::seed_from_u64(42);
        let r3 = generate(bounds, 3, 2, 4, true, 0, &mut rng3);
        assert!(
            r3.len() > r1.len(),
            "3 arms ({}) should produce more cells than 1 arm ({})",
            r3.len(),
            r1.len()
        );
    }

    #[test]
    fn spiral_deterministic() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let mut rng2 = SmallRng::seed_from_u64(42);
        let r1 = generate(bounds, 3, 2, 4, true, 0, &mut rng1);
        let r2 = generate(bounds, 3, 2, 4, true, 0, &mut rng2);
        let s1: HashSet<(u16, u16)> = r1.iter().map(|p| (p.x, p.y)).collect();
        let s2: HashSet<(u16, u16)> = r2.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(s1, s2);
    }
}
