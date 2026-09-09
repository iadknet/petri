//! Pattern generation algorithms for barrier painting.
//!
//! Generates barrier cell positions within a bounding rectangle using
//! procedural pattern algorithms (maze, spiral, noise, parallel lines, star).
//! Output is `Vec<PaintPoint>` suitable for feeding directly into
//! `Simulation::apply_paint()`.

mod lines;
mod maze;
mod noise;
mod spiral;
mod star;

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
    FbmThreshold {
        octaves: u32,
        frequency: f32,
        lacunarity: f32,
        persistence: f32,
        threshold: f32,
    },
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
    /// Normalize thresholded fBm parameters at config and runtime boundaries.
    pub fn normalize(&mut self) {
        if let Self::FbmThreshold {
            octaves,
            frequency,
            lacunarity,
            persistence,
            threshold,
        } = self
        {
            *octaves = (*octaves).clamp(1, 32);
            for (value, min, max, default) in [
                (frequency, 0.000001, 1.0, 0.02),
                (lacunarity, 1.0, 4.0, 2.0),
                (persistence, 0.0, 1.0, 0.5),
                (threshold, -1.0, 1.0, 0.0),
            ] {
                *value = if value.is_finite() {
                    value.clamp(min, max)
                } else {
                    default
                };
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
    rng: &mut impl Rng,
) -> Vec<PaintPoint> {
    if bounds.width == 0 || bounds.height == 0 {
        return Vec::new();
    }

    match params {
        PatternParams::FbmThreshold { .. } => {
            let mut normalized = params.clone();
            normalized.normalize();
            let PatternParams::FbmThreshold {
                octaves,
                frequency,
                lacunarity,
                persistence,
                threshold,
            } = normalized
            else {
                unreachable!()
            };
            let width = u32::from(bounds.width).min(65536 - u32::from(bounds.x)) as u16;
            let height = u32::from(bounds.height).min(65536 - u32::from(bounds.y)) as u16;
            let grid = crate::kernel::fertility::generate_fbm(
                width,
                height,
                octaves,
                frequency,
                lacunarity,
                persistence,
                rng.gen(),
            );
            grid.iter()
                .filter_map(|(x, y, value)| {
                    (*value > threshold).then_some(PaintPoint {
                        x: bounds.x + x,
                        y: bounds.y + y,
                    })
                })
                .collect()
        }
        PatternParams::Maze {
            corridor_width,
            wall_thickness,
            open_center_radius,
        } => maze::generate(
            bounds,
            *corridor_width,
            *wall_thickness,
            *open_center_radius,
            rng,
        ),
        PatternParams::Spiral {
            arm_count,
            arm_thickness,
            gap_width,
            clockwise,
            open_center_radius,
        } => spiral::generate(
            bounds,
            *arm_count,
            *arm_thickness,
            *gap_width,
            *clockwise,
            *open_center_radius,
            rng,
        ),
        PatternParams::Noise {
            density,
            cluster_size,
        } => noise::generate(bounds, *density, *cluster_size, rng),
        PatternParams::ParallelLines {
            spacing,
            thickness,
            jaggedness,
            angle_degrees,
        } => lines::generate(
            bounds,
            *spacing,
            *thickness,
            *jaggedness,
            *angle_degrees,
            rng,
        ),
        PatternParams::Star {
            point_count,
            ray_count,
            ray_length,
            ray_thickness,
        } => star::generate(
            bounds,
            *point_count,
            *ray_count,
            *ray_length,
            *ray_thickness,
            rng,
        ),
    }
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
        assert_eq!(result1.len(), result2.len());
        let s1: std::collections::HashSet<(u16, u16)> =
            result1.iter().map(|p| (p.x, p.y)).collect();
        let s2: std::collections::HashSet<(u16, u16)> =
            result2.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(s1, s2);
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
}
