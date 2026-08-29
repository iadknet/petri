use crate::config::OccupancyDepletionConfig;
use crate::kernel::Grid;

const RECOVERY_PER_TICK: f32 = 0.03;
const MIN_GROWTH_MULTIPLIER: f32 = 0.35;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct DepletionTickSummary {
    pub mean_depletion: f32,
    pub occupied_cells: u32,
}

/// Dynamic occupancy-driven suppression memory for food regrowth.
#[derive(Debug)]
pub(super) struct OccupancyDepletionLayer {
    grid: Grid<f32>,
}

impl OccupancyDepletionLayer {
    pub(super) fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height, 0.0),
        }
    }

    pub(super) fn reset(&mut self) {
        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                self.grid.set(x, y, 0.0);
            }
        }
    }

    #[must_use]
    pub(super) fn grid(&self) -> &Grid<f32> {
        &self.grid
    }

    pub(super) fn recover_and_deposit<T: Clone>(
        &mut self,
        barriers: &Grid<bool>,
        occupancy: &Grid<Option<T>>,
        config: &OccupancyDepletionConfig,
    ) -> DepletionTickSummary {
        if !config.enabled {
            return DepletionTickSummary::default();
        }

        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                if *barriers.get(x, y) {
                    self.grid.set(x, y, 0.0);
                    continue;
                }
                let current = *self.grid.get(x, y);
                self.grid.set(x, y, (current - RECOVERY_PER_TICK).max(0.0));
            }
        }

        let deposit_per_occupied_tick = config.deposit_per_occupied_tick.clamp(0.0, 1.0);
        let mut occupied_cells = 0u32;
        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                if *barriers.get(x, y) || occupancy.get(x, y).is_none() {
                    continue;
                }
                if deposit_per_occupied_tick <= 0.0 {
                    continue;
                }
                occupied_cells += 1;
                let current = *self.grid.get(x, y);
                self.grid
                    .set(x, y, (current + deposit_per_occupied_tick).clamp(0.0, 1.0));
            }
        }

        let mut sum = 0.0;
        let mut passable_cells = 0u32;
        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                if *barriers.get(x, y) {
                    continue;
                }
                passable_cells += 1;
                sum += *self.grid.get(x, y);
            }
        }

        let mean_depletion = if passable_cells == 0 {
            0.0
        } else {
            sum / passable_cells as f32
        };

        DepletionTickSummary {
            mean_depletion,
            occupied_cells,
        }
    }

    #[must_use]
    pub(super) fn multiplier_at(&self, x: u16, y: u16, enabled: bool) -> f32 {
        if !enabled {
            return 1.0;
        }
        let depletion = *self.grid.get(x, y);
        1.0 - depletion * (1.0 - MIN_GROWTH_MULTIPLIER)
    }
}
