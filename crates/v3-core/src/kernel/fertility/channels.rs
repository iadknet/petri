use super::generate_fbm;
use crate::kernel::Grid;

/// Shape of a channel layer: the fBm field whose zero contour the channels
/// follow, and the strip `width` (cells) and raw in-channel `value`.
pub struct ChannelParams {
    pub octaves: u32,
    pub frequency: f32,
    pub lacunarity: f32,
    pub persistence: f32,
    pub width: f32,
    pub value: f32,
}

/// Generate a fertility grid of narrow channels along the zero contour of an
/// fBm field (the ridges of ridged fBm, `1 - |f|`).
///
/// A cell is on a channel when its first-order distance to the contour,
/// `|f| / |grad f|`, is at most `width / 2`, so the strip is about `width`
/// cells across whatever the field's frequency or amplitude. Channel cells take
/// `value`; every other cell is -1. The gradient comes from central
/// differences of neighbouring cells (one-sided on the grid edge). A cell with
/// a vanishing gradient counts as off-channel unless it sits exactly on the
/// contour. Deterministic for a given seed.
pub fn generate_channels(width: u16, height: u16, params: &ChannelParams, seed: u64) -> Grid<f32> {
    let noise = generate_fbm(
        width,
        height,
        params.octaves,
        params.frequency,
        params.lacunarity,
        params.persistence,
        seed,
    );

    let half_width = params.width / 2.0;
    let mut grid = Grid::new(width, height, -1.0f32);
    for y in 0..height {
        for x in 0..width {
            let f = *noise.get(x, y);
            let gx = central_difference(width, x, |i| *noise.get(i, y));
            let gy = central_difference(height, y, |j| *noise.get(x, j));
            let gradient = gx.hypot(gy);
            // |f| <= half_width * |grad f| is the distance test without a division.
            if f.abs() <= half_width * gradient {
                grid.set(x, y, params.value);
            }
        }
    }
    grid
}

/// Derivative along one axis at index `i` of a run of `len` samples.
fn central_difference(len: u16, i: u16, at: impl Fn(u16) -> f32) -> f32 {
    let lo = i.saturating_sub(1);
    let hi = (i + 1).min(len.saturating_sub(1));
    if hi == lo {
        return 0.0;
    }
    (at(hi) - at(lo)) / f32::from(hi - lo)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: u16 = 256;

    fn params(width: f32) -> ChannelParams {
        ChannelParams {
            octaves: 4,
            frequency: 0.01,
            lacunarity: 2.0,
            persistence: 0.5,
            width,
            value: 0.5,
        }
    }

    fn channel_fraction(width: f32) -> f32 {
        let grid = generate_channels(SIZE, SIZE, &params(width), 42);
        let on = grid.as_slice().iter().filter(|v| **v > -1.0).count();
        on as f32 / grid.as_slice().len() as f32
    }

    #[test]
    fn cells_are_value_on_channels_and_barren_elsewhere() {
        let grid = generate_channels(SIZE, SIZE, &params(3.0), 42);
        assert!(grid.as_slice().iter().all(|v| *v == 0.5 || *v == -1.0));
    }

    #[test]
    fn channels_are_a_minority_at_width_three() {
        let fraction = channel_fraction(3.0);
        assert!(fraction > 0.0 && fraction < 0.5, "fraction {fraction}");
    }

    #[test]
    fn channel_fraction_rises_with_width_and_roughly_doubles_from_two_to_four() {
        let widths = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0];
        let fractions: Vec<f32> = widths.iter().map(|w| channel_fraction(*w)).collect();
        assert!(
            fractions.windows(2).all(|pair| pair[1] > pair[0]),
            "fractions not strictly rising: {fractions:?}"
        );
        let factor = fractions[3] / fractions[1];
        assert!(
            (1.5..=2.5).contains(&factor),
            "width 2 -> 4 factor {factor} (fractions {fractions:?})"
        );
    }

    #[test]
    fn same_seed_is_bit_identical_and_different_seed_differs() {
        let a = generate_channels(64, 64, &params(3.0), 9);
        let b = generate_channels(64, 64, &params(3.0), 9);
        let c = generate_channels(64, 64, &params(3.0), 10);
        let bits = |g: &Grid<f32>| g.as_slice().iter().map(|v| v.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&a), bits(&b));
        assert_ne!(bits(&a), bits(&c));
    }
}
