//! Grazing recovery and overuse (T02.F04): one fertility modifier grid per
//! food type, pulled down by consuming bites and healed by rest.

use crate::config::{GrazingConfig, OrdinaryFoodTypeId};
use crate::kernel::Grid;

/// The modifier after one consuming bite: multiplied by `factor`, never below
/// `floor`.
#[inline]
#[must_use]
pub(super) fn bite_modifier(modifier: f32, factor: f32, floor: f32) -> f32 {
    (modifier * factor).max(floor)
}

/// The modifier after one tick of rest: raised by `step`, never above 1.0.
#[inline]
#[must_use]
pub(super) fn recover_modifier(modifier: f32, step: f32) -> f32 {
    (modifier + step).min(1.0)
}

/// Per-tick recovery step for `recovery_ticks` (a floored cell heals from 0.0
/// to 1.0 in exactly that many ticks). Zero is normalized away upstream; it is
/// treated as one tick here so the step is always finite.
#[inline]
#[must_use]
pub(super) fn recovery_step(recovery_ticks: u32) -> f32 {
    1.0 / recovery_ticks.max(1) as f32
}

/// Final-tick grazing reading for one food type, gathered in the recovery pass.
/// The default is the ungrazed reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct GrazingTypeSummary {
    /// Mean modifier over passable cells (1.0 when nothing is grazed).
    pub mean_modifier: f32,
    /// Passable cells whose modifier is below 1.0.
    pub grazed_cells: u32,
}

impl Default for GrazingTypeSummary {
    fn default() -> Self {
        Self {
            mean_modifier: 1.0,
            grazed_cells: 0,
        }
    }
}

#[derive(Debug)]
pub(super) struct GrazingLayer {
    width: u16,
    height: u16,
    by_type: Vec<Grid<f32>>,
}

impl GrazingLayer {
    #[must_use]
    pub(super) fn new(width: u16, height: u16, type_count: usize) -> Self {
        Self {
            width,
            height,
            by_type: (0..type_count)
                .map(|_| Grid::new(width, height, 1.0))
                .collect(),
        }
    }

    /// Every modifier back to 1.0, keeping the type count.
    pub(super) fn reset(&mut self) {
        self.resize_type_storage(self.by_type.len());
    }

    /// Fresh 1.0 grids for a changed catalog.
    pub(super) fn resize_type_storage(&mut self, type_count: usize) {
        *self = Self::new(self.width, self.height, type_count);
    }

    #[must_use]
    pub(super) fn modifier_at(&self, x: u16, y: u16, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.by_type
            .get(usize::from(type_idx.get()))
            .map_or(1.0, |grid| *grid.get(x, y))
    }

    /// The multiplier growth applies: the stored modifier, or 1.0 while
    /// grazing is disabled.
    #[inline]
    #[must_use]
    pub(super) fn multiplier_at(
        &self,
        x: u16,
        y: u16,
        type_idx: OrdinaryFoodTypeId,
        enabled: bool,
    ) -> f32 {
        if enabled {
            self.modifier_at(x, y, type_idx)
        } else {
            1.0
        }
    }

    /// Record one consuming bite of `type_idx` at the cell. Disabled grazing
    /// records nothing.
    pub(super) fn bite(
        &mut self,
        x: u16,
        y: u16,
        type_idx: OrdinaryFoodTypeId,
        config: &GrazingConfig,
    ) {
        if !config.enabled {
            return;
        }
        if let Some(grid) = self.by_type.get_mut(usize::from(type_idx.get())) {
            let cell = grid.get_mut(x, y);
            *cell = bite_modifier(*cell, config.factor, config.floor);
        }
    }

    /// Advance every passable cell of every type one tick toward 1.0 and read
    /// the per-type summary. Barrier cells are held at 1.0. Disabled grazing
    /// leaves the grids untouched and reports the ungrazed reading.
    pub(super) fn recover(
        &mut self,
        barriers: &Grid<bool>,
        config: &GrazingConfig,
    ) -> (Vec<GrazingTypeSummary>, u32) {
        let passable_cells = barriers.as_slice().iter().filter(|b| !**b).count() as u32;
        if !config.enabled {
            return (
                vec![GrazingTypeSummary::default(); self.by_type.len()],
                passable_cells,
            );
        }
        let step = recovery_step(config.recovery_ticks);
        let summaries = self
            .by_type
            .iter_mut()
            .map(|grid| {
                let mut sum = 0.0f32;
                let mut grazed_cells = 0u32;
                for y in 0..grid.height() {
                    for x in 0..grid.width() {
                        if *barriers.get(x, y) {
                            grid.set(x, y, 1.0);
                            continue;
                        }
                        let cell = grid.get_mut(x, y);
                        *cell = recover_modifier(*cell, step);
                        sum += *cell;
                        if *cell < 1.0 {
                            grazed_cells += 1;
                        }
                    }
                }
                GrazingTypeSummary {
                    mean_modifier: if passable_cells == 0 {
                        1.0
                    } else {
                        sum / passable_cells as f32
                    },
                    grazed_cells,
                }
            })
            .collect();
        (summaries, passable_cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MAX_GRAZING_RECOVERY_TICKS;
    use proptest::prelude::*;

    fn type_id(raw: u16) -> OrdinaryFoodTypeId {
        OrdinaryFoodTypeId::new(raw)
    }

    /// `floor` first, then a stored modifier already within `[floor, 1.0]`: the
    /// production case, since live `floor` edits never rewrite stored cells.
    fn floor_and_modifier() -> impl Strategy<Value = (f32, f32)> {
        (0.0f32..=1.0).prop_flat_map(|floor| (Just(floor), floor..=1.0f32))
    }

    proptest! {
        #[test]
        fn bite_stays_within_range_and_is_non_increasing(
            (floor, modifier) in floor_and_modifier(),
            factor in 0.0f32..=1.0,
        ) {
            let bitten = bite_modifier(modifier, factor, floor);
            prop_assert!(bitten >= floor && bitten <= 1.0);
            prop_assert!(bitten <= modifier);
        }

        #[test]
        fn recovery_stays_within_range_and_is_non_decreasing(
            (floor, modifier) in floor_and_modifier(),
            recovery_ticks in 1u32..=MAX_GRAZING_RECOVERY_TICKS,
        ) {
            let recovered = recover_modifier(modifier, recovery_step(recovery_ticks));
            prop_assert!(recovered >= floor && recovered <= 1.0);
            prop_assert!(recovered >= modifier);
        }

        /// Bounded to realistic `recovery_ticks`: past roughly 1e7 the f32 step
        /// falls under half an ulp near 1.0 and the sum stops moving. Within
        /// the domain, each of the up-to-`n` additions rounds by at most
        /// `EPSILON / 2`, so the sum can trail by `n * EPSILON / 2` and need
        /// that many more `1 / n` steps: at most `n^2 * EPSILON / 2` ticks.
        #[test]
        fn recovery_reaches_exactly_one_within_the_ceiling(
            modifier in 0.0f32..=1.0,
            recovery_ticks in 1u32..=MAX_GRAZING_RECOVERY_TICKS,
        ) {
            let step = recovery_step(recovery_ticks);
            let ticks_f64 = f64::from(recovery_ticks);
            let rounding_slack = (ticks_f64 * ticks_f64 * f64::from(f32::EPSILON)).ceil() as u32;
            let bound =
                ((1.0 - f64::from(modifier)) * ticks_f64).ceil() as u32 + 1 + rounding_slack;
            let mut current = modifier;
            let mut ticks = 0u32;
            while current < 1.0 && ticks <= bound {
                current = recover_modifier(current, step);
                ticks += 1;
            }
            prop_assert_eq!(current, 1.0);
            prop_assert!(ticks <= bound);
        }

        #[test]
        fn a_bite_of_one_type_leaves_the_other_bit_identical(
            bitten_type in 0u16..3,
            x in 0u16..4,
            y in 0u16..3,
            (floor, _) in floor_and_modifier(),
            factor in 0.0f32..=1.0,
            bites in 1u32..6,
        ) {
            let config = GrazingConfig { enabled: true, factor, floor, recovery_ticks: 1000 };
            let mut layer = GrazingLayer::new(4, 3, 3);
            let before: Vec<Vec<u32>> = layer
                .by_type
                .iter()
                .map(|grid| grid.as_slice().iter().map(|v| v.to_bits()).collect())
                .collect();
            for _ in 0..bites {
                layer.bite(x, y, type_id(bitten_type), &config);
            }
            for (idx, grid) in layer.by_type.iter().enumerate() {
                if idx == usize::from(bitten_type) {
                    continue;
                }
                let after: Vec<u32> = grid.as_slice().iter().map(|v| v.to_bits()).collect();
                prop_assert_eq!(&after, &before[idx]);
            }
        }
    }

    #[test]
    fn five_default_bites_reach_the_floor() {
        let config = GrazingConfig::default();
        let mut layer = GrazingLayer::new(1, 1, 1);
        let mut readings = Vec::new();
        for _ in 0..5 {
            layer.bite(0, 0, type_id(0), &config);
            readings.push(layer.modifier_at(0, 0, type_id(0)));
        }
        assert_eq!(readings, vec![0.5, 0.25, 0.125, 0.0625, 0.05]);
    }

    #[test]
    fn disabled_grazing_records_no_bite_and_reads_one() {
        let config = GrazingConfig {
            enabled: false,
            ..GrazingConfig::default()
        };
        let mut layer = GrazingLayer::new(2, 1, 1);
        layer.bite(1, 0, type_id(0), &config);
        assert_eq!(layer.modifier_at(1, 0, type_id(0)), 1.0);
        layer.bite(1, 0, type_id(0), &GrazingConfig::default());
        assert_eq!(layer.modifier_at(1, 0, type_id(0)), 0.5);
        assert_eq!(layer.multiplier_at(1, 0, type_id(0), false), 1.0);
        assert_eq!(layer.multiplier_at(1, 0, type_id(0), true), 0.5);
    }

    #[test]
    fn recover_holds_barriers_at_one_and_summarizes_passable_cells() {
        let config = GrazingConfig {
            recovery_ticks: 4,
            ..GrazingConfig::default()
        };
        let mut barriers = Grid::new(2, 2, false);
        barriers.set(1, 1, true);
        let mut layer = GrazingLayer::new(2, 2, 2);
        layer.bite(0, 0, type_id(0), &config);
        layer.bite(0, 0, type_id(0), &config);
        layer.by_type[1].set(1, 1, 0.1);

        let (summary, passable) = layer.recover(&barriers, &config);

        assert_eq!(passable, 3);
        assert_eq!(layer.modifier_at(0, 0, type_id(0)), 0.5);
        assert_eq!(layer.modifier_at(1, 1, type_id(1)), 1.0);
        assert_eq!(summary[0].grazed_cells, 1);
        assert!((summary[0].mean_modifier - 2.5 / 3.0).abs() < 1e-6);
        assert_eq!(summary[1], GrazingTypeSummary::default());
    }

    #[test]
    fn reset_and_resize_start_from_one() {
        let config = GrazingConfig::default();
        let mut layer = GrazingLayer::new(2, 1, 1);
        layer.bite(0, 0, type_id(0), &config);
        layer.reset();
        assert_eq!(layer.modifier_at(0, 0, type_id(0)), 1.0);
        layer.bite(0, 0, type_id(0), &config);
        layer.resize_type_storage(3);
        assert_eq!(layer.by_type.len(), 3);
        assert_eq!(layer.modifier_at(0, 0, type_id(0)), 1.0);
        assert_eq!(layer.modifier_at(0, 0, type_id(7)), 1.0);
    }
}
