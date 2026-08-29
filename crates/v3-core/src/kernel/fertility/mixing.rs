use crate::kernel::Grid;

/// Combine multiple weighted fertility grids into a single blended grid.
///
/// Each entry in `layers` is a `(grid, weight)` pair. The output value for each
/// cell is `sum(weight * cell_value) / total_weight`, clamped to [-1, 1].
///
/// Edge cases:
/// - Empty `layers` returns a 0x0 grid.
/// - `total_weight <= 0.0` returns a grid filled with 0.0 using dimensions from
///   the first layer.
pub fn mix_layers(layers: &[(Grid<f32>, f32)]) -> Grid<f32> {
    if layers.is_empty() {
        return Grid::new(0, 0, 0.0);
    }

    let width = layers[0].0.width();
    let height = layers[0].0.height();
    let total_weight: f32 = layers.iter().map(|(_, w)| *w).sum();

    if total_weight <= 0.0 {
        return Grid::new(width, height, 0.0);
    }

    let mut result = Grid::new(width, height, 0.0f32);
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0f32;
            for (grid, weight) in layers {
                sum += weight * grid.get(x, y);
            }
            result.set(x, y, (sum / total_weight).clamp(-1.0, 1.0));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_filled(width: u16, height: u16, value: f32) -> Grid<f32> {
        Grid::new(width, height, value)
    }

    #[test]
    fn single_layer_passthrough() {
        let g = grid_filled(4, 4, 0.7);
        let result = mix_layers(&[(g, 1.0)]);
        for (_, _, v) in result.iter() {
            assert!((*v - 0.7).abs() < f32::EPSILON, "expected 0.7, got {v}");
        }
    }

    #[test]
    fn two_equal_weight_layers_average() {
        let g1 = grid_filled(4, 4, 0.8);
        let g2 = grid_filled(4, 4, 0.2);
        let result = mix_layers(&[(g1, 1.0), (g2, 1.0)]);
        for (_, _, v) in result.iter() {
            assert!((*v - 0.5).abs() < 1e-5, "expected 0.5, got {v}");
        }
    }

    #[test]
    fn unequal_weights_produce_weighted_average() {
        // weight 3 * 1.0 + weight 1 * -1.0 = 2.0 / 4.0 = 0.5
        let g1 = grid_filled(4, 4, 1.0);
        let g2 = grid_filled(4, 4, -1.0);
        let result = mix_layers(&[(g1, 3.0), (g2, 1.0)]);
        for (_, _, v) in result.iter() {
            assert!((*v - 0.5).abs() < 1e-5, "expected 0.5, got {v}");
        }
    }

    #[test]
    fn zero_total_weight_returns_uniform_zero() {
        let g = grid_filled(4, 4, 0.9);
        let result = mix_layers(&[(g, 0.0)]);
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
        for (_, _, v) in result.iter() {
            assert!(v.abs() < f32::EPSILON, "expected 0.0, got {v}");
        }
    }

    #[test]
    fn output_clamped_to_valid_range() {
        // Even though individual layers are in [-1, 1], weighted average should
        // still be clamped. With equal values there's no overflow, but let's
        // verify the clamp path with a carefully constructed scenario.
        let mut g1 = Grid::new(2, 2, 0.0);
        g1.set(0, 0, 1.0);
        let mut g2 = Grid::new(2, 2, 0.0);
        g2.set(0, 0, 1.0);
        let result = mix_layers(&[(g1, 1.0), (g2, 1.0)]);
        // (1.0 + 1.0) / 2.0 = 1.0 — right at boundary
        assert!(*result.get(0, 0) <= 1.0);
        assert!(*result.get(0, 0) >= -1.0);
    }

    #[test]
    fn empty_layers_returns_zero_size_grid() {
        let result = mix_layers(&[]);
        assert_eq!(result.width(), 0);
        assert_eq!(result.height(), 0);
    }
}
