use crate::config::FertilityBlend;
use crate::kernel::Grid;

/// Combine generated layers by their blend mode.
///
/// The `WeightedAverage` layers mix exactly as [`mix_layers`] does (in their
/// given order); with none, the base is barren (-1). Each `Max` layer then
/// raises a cell to `max(cell, layer value)`, its weight unused. The result is
/// clamped to [-1, 1] and does not depend on where the `Max` layers sit in
/// the list. Empty `layers` returns a 0x0 grid.
pub fn blend_layers(layers: Vec<(Grid<f32>, f32, FertilityBlend)>) -> Grid<f32> {
    let Some((first, _, _)) = layers.first() else {
        return Grid::new(0, 0, 0.0);
    };
    let (width, height) = (first.width(), first.height());
    let mut weighted = Vec::new();
    let mut max = Vec::new();
    for (grid, weight, blend) in layers {
        match blend {
            FertilityBlend::WeightedAverage => weighted.push((grid, weight)),
            FertilityBlend::Max => max.push(grid),
        }
    }

    let mut result = if weighted.is_empty() {
        Grid::new(width, height, -1.0)
    } else {
        mix_layers(&weighted)
    };
    for grid in &max {
        for (cell, layer) in result.as_mut_slice().iter_mut().zip(grid.as_slice()) {
            *cell = cell.max(*layer).clamp(-1.0, 1.0);
        }
    }
    result
}

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
    use proptest::prelude::*;

    fn bits(grid: &Grid<f32>) -> Vec<u32> {
        grid.as_slice().iter().map(|v| v.to_bits()).collect()
    }

    fn grid_from(values: &[f32]) -> Grid<f32> {
        let mut grid = Grid::new(values.len() as u16, 1, 0.0);
        grid.as_mut_slice().copy_from_slice(values);
        grid
    }

    #[test]
    fn weighted_only_blend_is_bit_identical_to_mix_layers() {
        let a = grid_from(&[0.3, -0.7, 1.0, -1.0]);
        let b = grid_from(&[-0.2, 0.9, 0.1, 0.4]);
        let legacy = mix_layers(&[(a.clone(), 1.3), (b.clone(), 0.7)]);
        let blended = blend_layers(vec![
            (a, 1.3, FertilityBlend::WeightedAverage),
            (b, 0.7, FertilityBlend::WeightedAverage),
        ]);
        assert_eq!(bits(&blended), bits(&legacy));
    }

    #[test]
    fn max_layer_keeps_blob_centre_instead_of_diluting_it() {
        let blob = grid_from(&[1.0, -1.0]);
        let channel = grid_from(&[-1.0, 0.25]);
        let blended = blend_layers(vec![
            (blob.clone(), 1.0, FertilityBlend::WeightedAverage),
            (blob, 1.0, FertilityBlend::WeightedAverage),
            (channel, 1.0, FertilityBlend::Max),
        ]);
        assert_eq!(blended.as_slice(), &[1.0, 0.25]);
    }

    #[test]
    fn max_only_blend_starts_from_barren() {
        let channel = grid_from(&[-1.0, 0.25]);
        let blended = blend_layers(vec![(channel, 0.0, FertilityBlend::Max)]);
        assert_eq!(blended.as_slice(), &[-1.0, 0.25]);
    }

    #[test]
    fn zero_total_weight_base_is_kept_under_max_layers() {
        let base = grid_from(&[0.9, 0.9]);
        let channel = grid_from(&[-1.0, 0.25]);
        let blended = blend_layers(vec![
            (base, 0.0, FertilityBlend::WeightedAverage),
            (channel, 1.0, FertilityBlend::Max),
        ]);
        assert_eq!(blended.as_slice(), &[0.0, 0.25]);
    }

    const CELLS: usize = 6;

    fn layer_strategy() -> impl Strategy<Value = (Vec<f32>, f32, bool)> {
        (
            prop::collection::vec(-1.5f32..=1.5, CELLS),
            0.0f32..=4.0,
            any::<bool>(),
        )
    }

    fn build(layers: &[(Vec<f32>, f32, bool)]) -> Vec<(Grid<f32>, f32, FertilityBlend)> {
        layers
            .iter()
            .map(|(values, weight, is_max)| {
                let blend = if *is_max {
                    FertilityBlend::Max
                } else {
                    FertilityBlend::WeightedAverage
                };
                (grid_from(values), *weight, blend)
            })
            .collect()
    }

    proptest! {
        /// A `Max` layer never lowers a cell below the mix without it, the
        /// result stays in [-1, 1], and moving the `Max` layers anywhere in the
        /// list (weighted layers keep their relative order) or reversing the
        /// `Max` layers among themselves leaves the grid bit-identical.
        #[test]
        fn max_layers_only_raise_and_ignore_position(
            layers in prop::collection::vec(layer_strategy(), 1..6),
            rotate in 0usize..6,
        ) {
            let built = build(&layers);
            let blended = blend_layers(built.clone());
            prop_assert!(blended.as_slice().iter().all(|v| (-1.0..=1.0).contains(v)));

            let without_max: Vec<_> = built
                .iter()
                .filter(|(_, _, b)| *b == FertilityBlend::WeightedAverage)
                .cloned()
                .collect();
            if !without_max.is_empty() {
                let base = blend_layers(without_max);
                for (with, without) in blended.as_slice().iter().zip(base.as_slice()) {
                    prop_assert!(with >= without, "{with} < {without}");
                }
            }

            let (weighted, mut max): (Vec<_>, Vec<_>) = built
                .into_iter()
                .partition(|(_, _, b)| *b == FertilityBlend::WeightedAverage);
            max.reverse();
            let split = rotate % (weighted.len() + 1);
            let mut reordered: Vec<_> = weighted[..split].to_vec();
            reordered.extend(max);
            reordered.extend_from_slice(&weighted[split..]);
            prop_assert_eq!(bits(&blend_layers(reordered)), bits(&blended));
        }
    }

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
