use std::collections::HashSet;
use std::f64::consts::PI;

use rand::Rng;

use crate::kernel::paint::PaintPoint;

use super::PatternBounds;

/// Generate parallel lines with jagged displacement.
pub(super) fn generate(
    bounds: PatternBounds,
    spacing: u8,
    thickness: u8,
    jaggedness: f32,
    angle_degrees: f32,
    rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    let spacing = spacing.max(2) as f64;
    let thickness = thickness.max(1) as i32;
    let jaggedness = jaggedness.max(0.0) as f64;
    let angle = (angle_degrees.clamp(0.0, 360.0) as f64) * PI / 180.0;

    // Direction along the line and perpendicular (normal).
    let dx = angle.cos();
    let dy = angle.sin();
    let nx = -angle.sin();
    let ny = angle.cos();

    // Compute how many lines fit: project bounds diagonal onto normal direction.
    let max_extent = (bounds.width as f64 * nx.abs()) + (bounds.height as f64 * ny.abs());
    let line_count = (max_extent / spacing).ceil() as usize;

    // For each step along the line, compute how far we can trace.
    let max_trace = (bounds.width as f64 * dx.abs()) + (bounds.height as f64 * dy.abs());
    let trace_steps = max_trace.ceil() as i32;

    let amplitude = jaggedness * spacing * 0.8;
    let step_size = amplitude * 0.15;
    let half_t = thickness / 2;

    let estimated = line_count * trace_steps as usize * thickness as usize;
    let mut cells = HashSet::with_capacity(estimated);

    for i in 0..line_count {
        let line_offset = (i as f64 + 0.5) * spacing;
        let mut displacement = 0.0_f64;

        // Origin offset: position the line grid centered on the bounds.
        let origin_x = bounds.x as f64 + bounds.width as f64 / 2.0;
        let origin_y = bounds.y as f64 + bounds.height as f64 / 2.0;

        // Base point on the normal axis.
        let base_x = origin_x + (line_offset - max_extent / 2.0) * nx;
        let base_y = origin_y + (line_offset - max_extent / 2.0) * ny;

        for t in (-trace_steps / 2)..=(trace_steps / 2) {
            // Apply jaggedness: random walk displacement.
            if amplitude > 0.0 {
                let delta = rng.gen_range(-step_size..=step_size);
                displacement = (displacement + delta).clamp(-amplitude, amplitude);
            }

            let px = base_x + t as f64 * dx + displacement * nx;
            let py = base_y + t as f64 * dy + displacement * ny;

            // Place thickness band perpendicular to line direction.
            for w in -half_t..=(half_t + thickness % 2 - 1) {
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
            width: 80,
            height: 80,
        }
    }

    #[test]
    fn lines_produces_non_empty_output() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 8, 1, 0.5, 0.0, &mut rng);
        assert!(!result.is_empty(), "lines should produce barrier cells");
    }

    #[test]
    fn lines_all_cells_within_bounds() {
        let bounds = PatternBounds {
            x: 5,
            y: 10,
            width: 70,
            height: 60,
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(bounds, 8, 1, 0.5, 0.0, &mut rng);
        for p in &result {
            assert!(p.x >= bounds.x && p.x < bounds.x + bounds.width);
            assert!(p.y >= bounds.y && p.y < bounds.y + bounds.height);
        }
    }

    #[test]
    fn lines_no_duplicate_cells() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 8, 1, 0.5, 0.0, &mut rng);
        let set: HashSet<(u16, u16)> = result.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(set.len(), result.len());
    }

    #[test]
    fn lines_horizontal_produces_horizontal_pattern() {
        let bounds = test_bounds();
        let mut rng = SmallRng::seed_from_u64(42);
        // angle=0, no jaggedness: lines should be approximately horizontal
        let result = generate(bounds, 10, 1, 0.0, 0.0, &mut rng);
        assert!(!result.is_empty());

        // Check that multiple distinct y values are present (multiple lines).
        let y_values: HashSet<u16> = result.iter().map(|p| p.y).collect();
        assert!(
            y_values.len() >= 2,
            "should have multiple horizontal lines, got {} distinct y values",
            y_values.len()
        );
    }

    #[test]
    fn lines_angled_produces_cells() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = generate(test_bounds(), 8, 1, 0.5, 45.0, &mut rng);
        assert!(!result.is_empty(), "45-degree lines should produce cells");
    }

    #[test]
    fn lines_line_count_matches_spacing() {
        let bounds = test_bounds(); // 80x80
        let spacing = 10u8;
        let mut rng = SmallRng::seed_from_u64(42);
        // Horizontal (angle=0), no jaggedness, thickness=1.
        let result = generate(bounds, spacing, 1, 0.0, 0.0, &mut rng);

        // Each line should appear at a distinct y value.
        let y_values: HashSet<u16> = result.iter().map(|p| p.y).collect();
        let expected_lines = (bounds.height as f64 / spacing as f64).ceil() as usize;
        // Allow +/-1 tolerance for boundary effects.
        assert!(
            y_values.len() >= expected_lines.saturating_sub(1)
                && y_values.len() <= expected_lines + 1,
            "expected ~{} lines, got {} distinct y values",
            expected_lines,
            y_values.len()
        );
    }

    #[test]
    fn lines_deterministic() {
        let bounds = test_bounds();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let mut rng2 = SmallRng::seed_from_u64(42);
        let r1 = generate(bounds, 8, 1, 0.5, 0.0, &mut rng1);
        let r2 = generate(bounds, 8, 1, 0.5, 0.0, &mut rng2);
        assert_eq!(r1.len(), r2.len());
        let s1: HashSet<(u16, u16)> = r1.iter().map(|p| (p.x, p.y)).collect();
        let s2: HashSet<(u16, u16)> = r2.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(s1, s2);
    }
}
