//! Query-side simulation caches.

use crate::state::FramePayload;
use crate::transport::session::ViewSubscription;
use v3_core::config::OrdinaryFoodTypeId;
use v3_core::kernel::paint::{PaintPoint, PaintStats, PaintTool};
use v3_core::kernel::FoodResource;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct DirtyRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl DirtyRect {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

#[must_use]
pub fn paint_dirty_rect(
    world_width: u16,
    world_height: u16,
    brush_half_extent: u8,
    points: &[PaintPoint],
) -> Option<DirtyRect> {
    if points.is_empty() || world_width == 0 || world_height == 0 {
        return None;
    }

    let radius = brush_half_extent as u16;
    let max_world_x = world_width - 1;
    let max_world_y = world_height - 1;
    let min_x = points
        .iter()
        .map(|point| point.x.saturating_sub(radius))
        .min()
        .unwrap_or(0);
    let min_y = points
        .iter()
        .map(|point| point.y.saturating_sub(radius))
        .min()
        .unwrap_or(0);
    let max_x = points
        .iter()
        .map(|point| point.x.saturating_add(radius).min(max_world_x))
        .max()
        .unwrap_or(0);
    let max_y = points
        .iter()
        .map(|point| point.y.saturating_add(radius).min(max_world_y))
        .max()
        .unwrap_or(0);

    Some(DirtyRect {
        x: min_x,
        y: min_y,
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
    })
}

#[must_use]
pub fn view_intersects_dirty_rect(subscription: &ViewSubscription, rect: DirtyRect) -> bool {
    if rect.width == 0 || rect.height == 0 || subscription.width == 0 || subscription.height == 0 {
        return false;
    }

    let rect_right = rect.x as u32 + rect.width as u32;
    let rect_bottom = rect.y as u32 + rect.height as u32;
    let view_right = subscription.x as u32 + subscription.width as u32;
    let view_bottom = subscription.y as u32 + subscription.height as u32;

    (u32::from(rect.x)) < view_right
        && rect_right > u32::from(subscription.x)
        && (u32::from(rect.y)) < view_bottom
        && rect_bottom > u32::from(subscription.y)
}

#[must_use]
pub fn world_static_changed(tool: PaintTool, stats: &PaintStats) -> bool {
    matches!(tool, PaintTool::Barrier | PaintTool::EraseBarrier)
        && (stats.barrier_set_cells > 0 || stats.barrier_cleared_cells > 0)
}

#[must_use]
pub fn build_food_density_planes(frame: &FramePayload) -> Box<[f32]> {
    let cell_count = frame.width as usize * frame.height as usize;
    let type_count = frame.food_types.len();
    let mut dense = vec![0.0f32; type_count * cell_count];

    if cell_count == 0 || type_count == 0 {
        return dense.into_boxed_slice();
    }

    for cell in &frame.food {
        let type_idx = usize::from(cell.type_idx);
        if type_idx >= type_count {
            continue;
        }
        if cell.density <= 0.0 {
            continue;
        }
        let cell_index = cell.y as usize * frame.width as usize + cell.x as usize;
        let plane_offset = type_idx * cell_count;
        dense[plane_offset + cell_index] = cell.density;
    }

    dense.into_boxed_slice()
}

#[must_use]
pub fn build_barrier_mask(frame: &FramePayload) -> Box<[u8]> {
    let cell_count = frame.width as usize * frame.height as usize;
    let mut mask = vec![0u8; cell_count.div_ceil(8)];
    for barrier in &frame.barriers {
        let index = barrier.y as usize * frame.width as usize + barrier.x as usize;
        let byte = index / 8;
        let bit = index % 8;
        mask[byte] |= 1 << bit;
    }
    mask.into_boxed_slice()
}

/// Quantize fertility for primary food type (`type_idx = 0`): raw [-1,1] ->
/// effective `[min, max]` range to the `u8` range `[0, 255]`.
///
/// Edge case: when min == max the entire grid gets 128 (mid-point).
///
/// Returns an `Arc<[u8]>` so the result can be shared across frames without
/// copying the grid on every projection publish.
#[must_use]
pub fn build_primary_food_fertility_u8(food: &FoodResource) -> std::sync::Arc<[u8]> {
    let width = usize::from(food.width());
    let height = usize::from(food.height());
    let Some(fertility_grid) = food.fertility_for_type(OrdinaryFoodTypeId::default()) else {
        return vec![128u8; width * height].into();
    };
    let fertility_config = &food.full_config().fertility;
    let min = fertility_config.min_fertility;
    let max = fertility_config.max_fertility;

    if (max - min).abs() < f32::EPSILON {
        return vec![128u8; width * height].into();
    }

    let mut result = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let raw = *fertility_grid.get(x as u16, y as u16);
            // Algebraically equivalent to: raw → effective [min,max] → u8. The intermediate effective value cancels out.
            let t = (raw + 1.0) / 2.0;
            let u8val = (t * 255.0).round().clamp(0.0, 255.0) as u8;
            result.push(u8val);
        }
    }
    result.into()
}

#[must_use]
pub fn build_food_fertility_u8(food: &FoodResource) -> std::sync::Arc<[u8]> {
    build_primary_food_fertility_u8(food)
}

#[cfg(test)]
mod tests {
    use v3_core::config::{
        FertilityConfig, FertilityLayer, FoodConfig, FoodResourceConfig, WorldEdgeMode,
    };
    use v3_core::kernel::paint::{PaintPoint, PaintStats, PaintTool};
    use v3_core::kernel::FoodResource;

    use crate::state::{BarrierCell, CreatureSnapshot, FoodCell, FoodTypeSnapshot, FramePayload};
    use crate::transport::protocol::ZoomTier;
    use crate::transport::session::ViewSubscription;

    use super::{
        build_food_density_planes, build_food_fertility_u8, paint_dirty_rect,
        view_intersects_dirty_rect, world_static_changed, DirtyRect,
    };

    #[test]
    fn paint_dirty_rect_expands_and_clamps_to_world() {
        let rect = paint_dirty_rect(
            16,
            12,
            1,
            &[PaintPoint { x: 0, y: 0 }, PaintPoint { x: 15, y: 11 }],
        )
        .expect("dirty rect should be produced");

        assert_eq!(
            rect,
            DirtyRect {
                x: 0,
                y: 0,
                width: 16,
                height: 12,
            }
        );
    }

    #[test]
    fn view_intersection_excludes_distant_subscription() {
        let rect = DirtyRect {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let subscription = ViewSubscription {
            request_id: 9,
            x: 10,
            y: 10,
            width: 4,
            height: 4,
            canvas_width: 100,
            canvas_height: 100,
            zoom_tier: ZoomTier::Detail,
        };

        assert!(!view_intersects_dirty_rect(&subscription, rect));
    }

    #[test]
    fn world_static_change_detects_barrier_edits_only() {
        let stats = PaintStats {
            barrier_set_cells: 2,
            ..PaintStats::default()
        };
        assert!(world_static_changed(PaintTool::Barrier, &stats));
        assert!(world_static_changed(PaintTool::EraseBarrier, &stats));
        assert!(!world_static_changed(PaintTool::Food, &stats));
        assert!(!world_static_changed(PaintTool::EraseFood, &stats));
    }

    // ── Fertility quantization tests ────────────────────────────────────────

    /// Create a FoodResource with uniform fertility for quantization tests.
    fn food_with_fertility(
        width: u16,
        height: u16,
        min_fert: f32,
        max_fert: f32,
        uniform_value: f32,
    ) -> FoodResource {
        let config = FoodResourceConfig {
            fertility: FertilityConfig {
                enabled: true,
                min_fertility: min_fert,
                max_fertility: max_fert,
                layers: vec![FertilityLayer {
                    algorithm: v3_core::config::FertilityAlgorithm::Uniform {
                        value: uniform_value,
                    },
                    weight: 1.0,
                    target: v3_core::config::FertilityLayerTarget::default(),
                }],
            },
            ..FoodResourceConfig::default()
        };
        let mut food = FoodResource::new(
            width,
            height,
            FoodConfig::single_type(config),
            WorldEdgeMode::Wrap,
        );
        food.seed_fertility(42);
        food
    }

    #[test]
    fn quantize_fertility_maps_range_to_u8() {
        // Uniform value 0.0 → raw fertility = 0.0.
        // With min=0, max=2: t = (0+1)/2 = 0.5, u8 = (0.5*255).round() = 128.
        let food = food_with_fertility(2, 2, 0.0, 2.0, 0.0);
        let result = build_food_fertility_u8(&food);
        assert_eq!(result.len(), 4);
        for &v in result.iter() {
            assert_eq!(v, 128, "expected 128 for midpoint fertility, got {v}");
        }
    }

    #[test]
    fn quantize_fertility_extreme_values() {
        // Uniform value 1.0 → raw = 1.0.
        // t = (1+1)/2 = 1.0, u8 = (1.0*255).round() = 255.
        let food = food_with_fertility(2, 2, 0.0, 2.0, 1.0);
        let result = build_food_fertility_u8(&food);
        for &v in result.iter() {
            assert_eq!(v, 255, "expected 255 for max fertility, got {v}");
        }

        // Uniform value -1.0 → raw = -1.0.
        // t = (-1+1)/2 = 0.0, effective = min → u8 = 0.
        let food = food_with_fertility(2, 2, 0.0, 2.0, -1.0);
        let result = build_food_fertility_u8(&food);
        for &v in result.iter() {
            assert_eq!(v, 0, "expected 0 for min fertility, got {v}");
        }
    }

    #[test]
    fn quantize_fertility_min_equals_max_returns_128() {
        let food = food_with_fertility(3, 3, 1.5, 1.5, 0.5);
        let result = build_food_fertility_u8(&food);
        assert_eq!(result.len(), 9);
        for &v in result.iter() {
            assert_eq!(v, 128, "expected 128 when min==max, got {v}");
        }
    }

    #[test]
    fn build_food_density_planes_is_plane_major_and_zero_filled() {
        let frame = FramePayload {
            width: 2,
            height: 2,
            creatures: vec![CreatureSnapshot {
                id: 1,
                x: 0,
                y: 0,
                energy: 1.0,
                generation: 1,
                phenotype_rgb: [1, 2, 3],
            }],
            food_types: vec![
                FoodTypeSnapshot {
                    type_idx: 0,
                    name: "A".to_string(),
                    color: "#111111".to_string(),
                    growth_inhibitor: 0.2,
                },
                FoodTypeSnapshot {
                    type_idx: 1,
                    name: "B".to_string(),
                    color: "#222222".to_string(),
                    growth_inhibitor: 0.3,
                },
            ],
            food: vec![
                FoodCell {
                    x: 0,
                    y: 0,
                    type_idx: 0,
                    density: 0.25,
                },
                FoodCell {
                    x: 1,
                    y: 1,
                    type_idx: 1,
                    density: 0.75,
                },
                FoodCell {
                    x: 1,
                    y: 0,
                    type_idx: 9,
                    density: 1.0,
                },
            ],
            barriers: vec![BarrierCell { x: 1, y: 0 }],
            food_fertility_u8: vec![0u8; 4].into(),
        };

        let result = build_food_density_planes(&frame);

        assert_eq!(result.len(), 8);
        assert_eq!(&result[..4], &[0.25, 0.0, 0.0, 0.0]);
        assert_eq!(&result[4..], &[0.0, 0.0, 0.0, 0.75]);
    }
}
