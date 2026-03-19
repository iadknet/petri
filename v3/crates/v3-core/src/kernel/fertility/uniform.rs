use crate::kernel::Grid;

/// Generate a uniform fertility grid where every cell has the same value.
/// The value is clamped to [-1, 1].
pub fn generate_uniform(width: u16, height: u16, value: f32) -> Grid<f32> {
    Grid::new(width, height, value.clamp(-1.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_grid_with_clamped_value() {
        let grid = generate_uniform(8, 8, 0.5);
        for (_, _, v) in grid.iter() {
            assert!((*v - 0.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn clamps_values_outside_range() {
        let grid_high = generate_uniform(4, 4, 2.5);
        for (_, _, v) in grid_high.iter() {
            assert!((*v - 1.0).abs() < f32::EPSILON, "expected 1.0, got {v}");
        }

        let grid_low = generate_uniform(4, 4, -3.0);
        for (_, _, v) in grid_low.iter() {
            assert!((*v - (-1.0)).abs() < f32::EPSILON, "expected -1.0, got {v}");
        }
    }
}
