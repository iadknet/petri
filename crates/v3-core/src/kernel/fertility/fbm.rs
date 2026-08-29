use crate::kernel::Grid;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

/// Generate a fertility grid using fractional Brownian motion (Fbm) noise.
///
/// Each cell's value is the Fbm noise evaluated at `(x * frequency, y * frequency)`,
/// clamped to [-1, 1]. Deterministic for a given seed.
pub fn generate_fbm(
    width: u16,
    height: u16,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    seed: u64,
) -> Grid<f32> {
    // XOR-fold the u64 seed so both the upper and lower 32 bits influence the
    // Perlin builder's u32 seed parameter.
    let fbm = Fbm::<Perlin>::new((seed >> 32) as u32 ^ seed as u32)
        .set_octaves(octaves as usize)
        .set_frequency(frequency as f64)
        .set_lacunarity(lacunarity as f64)
        .set_persistence(persistence as f64);

    let mut grid = Grid::new(width, height, 0.0f32);
    for y in 0..height {
        for x in 0..width {
            // Frequency scaling is handled internally by the noise crate's Fbm builder.
            let val = fbm.get([x as f64, y as f64]) as f32;
            grid.set(x, y, val.clamp(-1.0, 1.0));
        }
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_in_valid_range() {
        let grid = generate_fbm(32, 32, 4, 0.05, 2.0, 0.5, 42);
        for (_, _, v) in grid.iter() {
            assert!(*v >= -1.0 && *v <= 1.0, "value {v} out of [-1, 1] range");
        }
    }

    #[test]
    fn deterministic_same_seed() {
        let g1 = generate_fbm(16, 16, 4, 0.05, 2.0, 0.5, 123);
        let g2 = generate_fbm(16, 16, 4, 0.05, 2.0, 0.5, 123);
        for y in 0..16u16 {
            for x in 0..16u16 {
                assert!(
                    (*g1.get(x, y) - *g2.get(x, y)).abs() < f32::EPSILON,
                    "mismatch at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn different_seeds_produce_different_output() {
        let g1 = generate_fbm(16, 16, 4, 0.05, 2.0, 0.5, 100);
        let g2 = generate_fbm(16, 16, 4, 0.05, 2.0, 0.5, 999);
        let mut differ = false;
        for y in 0..16u16 {
            for x in 0..16u16 {
                if (*g1.get(x, y) - *g2.get(x, y)).abs() > f32::EPSILON {
                    differ = true;
                    break;
                }
            }
            if differ {
                break;
            }
        }
        assert!(differ, "different seeds should produce different output");
    }
}
