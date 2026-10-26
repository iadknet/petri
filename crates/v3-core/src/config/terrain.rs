//! Terrain schema: startup barrier layers and the pattern parameters they
//! carry. The generation algorithms live in `crate::patterns`.

use serde::{Deserialize, Serialize};

/// Axis-aligned bounding rectangle in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl PatternBounds {
    /// These bounds intersected with a `width` × `height` world: each extent
    /// is cut to the world minus the origin, saturating at zero, and the
    /// origin is unchanged. Computed without adding `u16` endpoints, so it
    /// cannot overflow or wrap.
    #[must_use]
    pub fn clipped_to(self, width: u16, height: u16) -> Self {
        Self {
            width: self.width.min(width.saturating_sub(self.x)),
            height: self.height.min(height.saturating_sub(self.y)),
            ..self
        }
    }
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

/// One additive startup barrier layer, using the existing pattern schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainLayer {
    pub params: PatternParams,
    pub bounds: Option<PatternBounds>,
    pub seed: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// The clip keeps the origin and exactly the part of the input that
        /// lies inside the world.
        #[test]
        fn clipped_to_lies_inside_world_and_input(
            x in any::<u16>(),
            y in any::<u16>(),
            width in any::<u16>(),
            height in any::<u16>(),
            world_width in any::<u16>(),
            world_height in any::<u16>(),
        ) {
            let input = PatternBounds { x, y, width, height };
            let clipped = input.clipped_to(world_width, world_height);
            prop_assert_eq!((clipped.x, clipped.y), (x, y));
            let fit_width = (u32::from(x) + u32::from(width)).min(u32::from(world_width)).saturating_sub(u32::from(x));
            let fit_height = (u32::from(y) + u32::from(height)).min(u32::from(world_height)).saturating_sub(u32::from(y));
            prop_assert_eq!((u32::from(clipped.width), u32::from(clipped.height)), (fit_width, fit_height));
        }
    }
}
