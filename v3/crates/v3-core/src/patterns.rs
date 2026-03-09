//! Pattern generation algorithms for barrier painting.
//!
//! Generates barrier cell positions within a bounding rectangle using
//! procedural pattern algorithms (maze, spiral, noise, parallel lines, star).
//! Output is `Vec<PaintPoint>` suitable for feeding directly into
//! `Simulation::apply_paint()`.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::kernel::paint::PaintPoint;

/// Axis-aligned bounding rectangle in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Pattern configuration. The enum discriminant serves as the pattern type tag.
/// No separate PatternType enum — the variant IS the type, avoiding mismatched
/// type+params pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "pattern_type")]
pub enum PatternParams {
    Maze {
        corridor_width: u8,
        wall_thickness: u8,
        open_center_radius: u8,
    },
    Spiral {
        arm_count: u8,
        arm_thickness: u8,
        gap_width: u8,
        clockwise: bool,
        open_center_radius: u8,
    },
    Noise {
        density: f32,
        cluster_size: u8,
    },
    ParallelLines {
        spacing: u8,
        thickness: u8,
        jaggedness: f32,
        angle_degrees: f32,
    },
    Star {
        point_count: u8,
        ray_count: u8,
        ray_length: u8,
        ray_thickness: u8,
    },
}

impl PatternParams {
    /// Estimated output cell count for `Vec::with_capacity` pre-allocation.
    fn estimated_cells(&self, area: u32) -> usize {
        match self {
            PatternParams::Maze {
                corridor_width,
                wall_thickness,
                ..
            } => {
                let cell = (*corridor_width as u32 + *wall_thickness as u32).max(1);
                // Walls occupy roughly wall_thickness / cell_size fraction of area
                (area as f64 * *wall_thickness as f64 / cell as f64) as usize
            }
            PatternParams::Spiral {
                arm_thickness,
                gap_width,
                ..
            } => {
                let cycle = (*arm_thickness as u32 + *gap_width as u32).max(1);
                (area as f64 * *arm_thickness as f64 / cycle as f64) as usize
            }
            PatternParams::Noise { density, .. } => {
                (area as f64 * (*density as f64).clamp(0.0, 1.0)) as usize
            }
            PatternParams::ParallelLines {
                spacing, thickness, ..
            } => {
                let cycle = (*spacing as u32).max(1);
                (area as f64 * *thickness as f64 / cycle as f64) as usize
            }
            PatternParams::Star {
                point_count,
                ray_count,
                ray_length,
                ray_thickness,
                ..
            } => {
                *point_count as usize
                    * *ray_count as usize
                    * *ray_length as usize
                    * *ray_thickness as usize
            }
        }
    }
}

/// Generate barrier cell positions for a pattern within the given bounds.
///
/// Returns deduplicated `PaintPoint`s, all guaranteed within bounds.
pub fn generate_pattern(
    bounds: PatternBounds,
    params: &PatternParams,
    _rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    if bounds.width == 0 || bounds.height == 0 {
        return Vec::new();
    }

    let area = bounds.width as u32 * bounds.height as u32;
    let capacity = params.estimated_cells(area);
    let _cells = std::collections::HashSet::<(u16, u16)>::with_capacity(capacity);

    // Stub: pattern algorithm implementations will be added in Step 2.
    // For now, return empty to establish the API contract.
    Vec::new()
}

/// Convenience wrapper that creates a `SmallRng` from a seed and calls
/// `generate_pattern`.
pub fn generate_pattern_seeded(
    bounds: PatternBounds,
    params: &PatternParams,
    seed: u64,
) -> Vec<PaintPoint> {
    let mut rng = SmallRng::seed_from_u64(seed);
    generate_pattern(bounds, params, &mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_maze() -> PatternParams {
        PatternParams::Maze {
            corridor_width: 2,
            wall_thickness: 1,
            open_center_radius: 0,
        }
    }

    fn default_spiral() -> PatternParams {
        PatternParams::Spiral {
            arm_count: 3,
            arm_thickness: 2,
            gap_width: 4,
            clockwise: true,
            open_center_radius: 0,
        }
    }

    fn default_noise() -> PatternParams {
        PatternParams::Noise {
            density: 0.15,
            cluster_size: 3,
        }
    }

    fn default_parallel_lines() -> PatternParams {
        PatternParams::ParallelLines {
            spacing: 8,
            thickness: 1,
            jaggedness: 0.5,
            angle_degrees: 0.0,
        }
    }

    fn default_star() -> PatternParams {
        PatternParams::Star {
            point_count: 1,
            ray_count: 8,
            ray_length: 20,
            ray_thickness: 1,
        }
    }

    // --- Bounds validation ---

    #[test]
    fn zero_width_returns_empty() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 0,
            height: 10,
        };
        let result = generate_pattern_seeded(bounds, &default_maze(), 42);
        assert!(result.is_empty());
    }

    #[test]
    fn zero_height_returns_empty() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 10,
            height: 0,
        };
        let result = generate_pattern_seeded(bounds, &default_maze(), 42);
        assert!(result.is_empty());
    }

    #[test]
    fn zero_area_returns_empty() {
        let bounds = PatternBounds {
            x: 5,
            y: 5,
            width: 0,
            height: 0,
        };
        let result = generate_pattern_seeded(bounds, &default_noise(), 42);
        assert!(result.is_empty());
    }

    // --- Serde round-trip ---

    #[test]
    fn serde_roundtrip_maze() {
        let params = default_maze();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"pattern_type\":\"Maze\""));
        let decoded: PatternParams = serde_json::from_str(&json).unwrap();
        match decoded {
            PatternParams::Maze {
                corridor_width,
                wall_thickness,
                open_center_radius,
            } => {
                assert_eq!(corridor_width, 2);
                assert_eq!(wall_thickness, 1);
                assert_eq!(open_center_radius, 0);
            }
            _ => panic!("expected Maze variant"),
        }
    }

    #[test]
    fn serde_roundtrip_spiral() {
        let params = default_spiral();
        let json = serde_json::to_string(&params).unwrap();
        let decoded: PatternParams = serde_json::from_str(&json).unwrap();
        match decoded {
            PatternParams::Spiral {
                arm_count,
                arm_thickness,
                gap_width,
                clockwise,
                open_center_radius,
            } => {
                assert_eq!(arm_count, 3);
                assert_eq!(arm_thickness, 2);
                assert_eq!(gap_width, 4);
                assert!(clockwise);
                assert_eq!(open_center_radius, 0);
            }
            _ => panic!("expected Spiral variant"),
        }
    }

    #[test]
    fn serde_roundtrip_noise() {
        let params = default_noise();
        let json = serde_json::to_string(&params).unwrap();
        let decoded: PatternParams = serde_json::from_str(&json).unwrap();
        match decoded {
            PatternParams::Noise {
                density,
                cluster_size,
            } => {
                assert!((density - 0.15).abs() < f32::EPSILON);
                assert_eq!(cluster_size, 3);
            }
            _ => panic!("expected Noise variant"),
        }
    }

    #[test]
    fn serde_roundtrip_parallel_lines() {
        let params = default_parallel_lines();
        let json = serde_json::to_string(&params).unwrap();
        let decoded: PatternParams = serde_json::from_str(&json).unwrap();
        match decoded {
            PatternParams::ParallelLines {
                spacing,
                thickness,
                jaggedness,
                angle_degrees,
            } => {
                assert_eq!(spacing, 8);
                assert_eq!(thickness, 1);
                assert!((jaggedness - 0.5).abs() < f32::EPSILON);
                assert!((angle_degrees).abs() < f32::EPSILON);
            }
            _ => panic!("expected ParallelLines variant"),
        }
    }

    #[test]
    fn serde_roundtrip_star() {
        let params = default_star();
        let json = serde_json::to_string(&params).unwrap();
        let decoded: PatternParams = serde_json::from_str(&json).unwrap();
        match decoded {
            PatternParams::Star {
                point_count,
                ray_count,
                ray_length,
                ray_thickness,
            } => {
                assert_eq!(point_count, 1);
                assert_eq!(ray_count, 8);
                assert_eq!(ray_length, 20);
                assert_eq!(ray_thickness, 1);
            }
            _ => panic!("expected Star variant"),
        }
    }

    #[test]
    fn serde_roundtrip_bounds() {
        let bounds = PatternBounds {
            x: 10,
            y: 20,
            width: 100,
            height: 50,
        };
        let json = serde_json::to_string(&bounds).unwrap();
        let decoded: PatternBounds = serde_json::from_str(&json).unwrap();
        assert_eq!(bounds, decoded);
    }

    // --- Generate function contract ---

    #[test]
    fn generate_pattern_seeded_is_deterministic() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 50,
            height: 50,
        };
        let params = default_noise();
        let result1 = generate_pattern_seeded(bounds, &params, 12345);
        let result2 = generate_pattern_seeded(bounds, &params, 12345);
        assert_eq!(result1, result2);
    }

    #[test]
    fn generate_pattern_returns_paint_points() {
        let bounds = PatternBounds {
            x: 0,
            y: 0,
            width: 20,
            height: 20,
        };
        // This test verifies the function compiles and returns Vec<PaintPoint>.
        // Actual non-empty output tests will be added in Step 2 with algorithms.
        let result: Vec<PaintPoint> = generate_pattern_seeded(bounds, &default_maze(), 42);
        // All returned points must be within bounds (vacuously true for empty stub)
        for p in &result {
            assert!(p.x >= bounds.x && p.x < bounds.x + bounds.width);
            assert!(p.y >= bounds.y && p.y < bounds.y + bounds.height);
        }
    }

    #[test]
    fn estimated_cells_reasonable_for_noise() {
        let params = PatternParams::Noise {
            density: 0.5,
            cluster_size: 3,
        };
        // 100x100 at 50% density = ~5000 cells
        let estimate = params.estimated_cells(10_000);
        assert!(estimate > 0);
        assert!(estimate <= 10_000);
    }
}
