use crate::kernel::Grid;
use rand::Rng;

/// Generate a fertility grid using Poisson-distributed blobs with Gaussian falloff.
///
/// Background is -1.0 (barren). Each blob has center value 1.0 decaying
/// toward -1.0 via Gaussian falloff. Overlapping blobs take the max value.
///
/// Blob centers are placed using random-with-minimum-distance-rejection:
/// each candidate is rejected if it falls within `min_radius * 2` of an
/// already-placed center. Up to `blob_count * 3` total attempts are made.
///
/// Deterministic for a given `rng` state.
pub fn generate_poisson_blobs(
    width: u16,
    height: u16,
    blob_count: u32,
    min_radius: f32,
    max_radius: f32,
    falloff: f32,
    rng: &mut impl Rng,
) -> Grid<f32> {
    let mut grid = Grid::new(width, height, -1.0f32);
    if width == 0 || height == 0 || blob_count == 0 {
        return grid;
    }

    let (min_r, max_r) = (min_radius.min(max_radius), min_radius.max(max_radius));
    let min_distance = min_r * 2.0;
    let min_distance_sq = min_distance * min_distance;
    let max_attempts = blob_count as usize * 3;

    // Place blob centers with minimum-distance rejection.
    let mut centers: Vec<(f32, f32, f32)> = Vec::new(); // (cx, cy, radius)
    for _ in 0..max_attempts {
        if centers.len() >= blob_count as usize {
            break;
        }
        let cx = rng.gen_range(0.0..width as f32);
        let cy = rng.gen_range(0.0..height as f32);

        let too_close = centers.iter().any(|(ox, oy, _)| {
            let dx = cx - ox;
            let dy = cy - oy;
            dx * dx + dy * dy < min_distance_sq
        });

        if !too_close {
            let radius = rng.gen_range(min_r..=max_r);
            centers.push((cx, cy, radius));
        }
    }

    // Paint each blob with Gaussian falloff.
    for &(cx, cy, radius) in &centers {
        // Determine bounding box for this blob (radius covers the area where
        // the Gaussian contribution is non-negligible).
        let x_min = ((cx - radius).floor().max(0.0)) as u16;
        let x_max = ((cx + radius).ceil().min(width as f32 - 1.0)) as u16;
        let y_min = ((cy - radius).floor().max(0.0)) as u16;
        let y_max = ((cy + radius).ceil().min(height as f32 - 1.0)) as u16;

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist_sq = dx * dx + dy * dy;
                let radius_sq = radius * radius;

                if dist_sq > radius_sq {
                    continue;
                }

                // Gaussian falloff: 1.0 at center, decays toward 0.0 at radius.
                // Map to [-1, 1]: value = 2 * gaussian - 1.
                let normalized_dist = dist_sq / radius_sq;
                let gaussian = (-falloff * normalized_dist).exp();
                let value = 2.0 * gaussian - 1.0;

                let current = *grid.get(x, y);
                if value > current {
                    grid.set(x, y, value);
                }
            }
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn make_rng(seed: u64) -> rand::rngs::SmallRng {
        rand::rngs::SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn output_in_valid_range() {
        let mut rng = make_rng(42);
        let grid = generate_poisson_blobs(64, 64, 10, 5.0, 15.0, 2.0, &mut rng);
        for (_, _, v) in grid.iter() {
            assert!(*v >= -1.0 && *v <= 1.0, "value {v} out of [-1, 1] range");
        }
    }

    #[test]
    fn deterministic_same_rng_seed() {
        let g1 = {
            let mut rng = make_rng(77);
            generate_poisson_blobs(32, 32, 8, 3.0, 10.0, 2.0, &mut rng)
        };
        let g2 = {
            let mut rng = make_rng(77);
            generate_poisson_blobs(32, 32, 8, 3.0, 10.0, 2.0, &mut rng)
        };
        for y in 0..32u16 {
            for x in 0..32u16 {
                assert!(
                    (*g1.get(x, y) - *g2.get(x, y)).abs() < f32::EPSILON,
                    "mismatch at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn creates_variation() {
        let mut rng = make_rng(42);
        let grid = generate_poisson_blobs(64, 64, 15, 5.0, 15.0, 2.0, &mut rng);
        let mut has_high = false;
        let mut has_low = false;
        for (_, _, v) in grid.iter() {
            if *v > 0.5 {
                has_high = true;
            }
            if *v < -0.5 {
                has_low = true;
            }
        }
        assert!(has_high, "expected some values > 0.5");
        assert!(has_low, "expected some values < -0.5");
    }
}
