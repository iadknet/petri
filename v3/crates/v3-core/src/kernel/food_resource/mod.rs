mod depletion;
mod growth;
mod reconfigure;

use rand::Rng;

use crate::config::{FoodResourceConfig, WorldEdgeMode};
use crate::contracts::{Direction, Position};
use crate::kernel::fertility;
use crate::kernel::Grid;

use self::depletion::OccupancyDepletionLayer;
use self::growth::{apply_growth, GrowthContext};

/// Aggregate summary emitted by one food-growth pass.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FoodGrowthSummary {
    pub mean_occupancy_depletion: f32,
    pub occupied_cells_with_depletion: u32,
    pub growth_suppressed_by_occupancy_depletion: f32,
}

/// Self-contained food substrate: density grid, fertility map, depletion layer, and growth logic.
pub struct FoodResource {
    density: Grid<f32>,
    fertility: Grid<f32>,
    occupancy_depletion: OccupancyDepletionLayer,
    config: FoodResourceConfig,
    edge_mode: WorldEdgeMode,
    /// Reusable scratch buffer for `grow` to avoid per-tick allocation.
    growth_scratch: Vec<f32>,
}

impl FoodResource {
    /// Create a new food resource with zero density, zero fertility, and neutral depletion.
    pub fn new(
        width: u16,
        height: u16,
        config: FoodResourceConfig,
        edge_mode: WorldEdgeMode,
    ) -> Self {
        let total_cells = width as usize * height as usize;
        Self {
            density: Grid::new(width, height, 0.0),
            fertility: Grid::new(width, height, 0.0),
            occupancy_depletion: OccupancyDepletionLayer::new(width, height),
            config,
            edge_mode,
            growth_scratch: Vec::with_capacity(total_cells),
        }
    }

    /// Get food density at a position.
    #[must_use]
    pub fn food_at(&self, pos: Position) -> f32 {
        *self.density.get(pos.x, pos.y)
    }

    /// Consume all food on a cell. Returns the amount consumed (0 if empty).
    #[must_use]
    pub fn consume(&mut self, pos: Position) -> f32 {
        let amount = *self.density.get(pos.x, pos.y);
        self.density.set(pos.x, pos.y, 0.0);
        amount
    }

    /// Set food density at a position directly.
    pub fn set_food(&mut self, pos: Position, value: f32) {
        self.density.set(pos.x, pos.y, value);
    }

    /// Total food across all cells (for testing and diagnostics).
    #[must_use]
    pub fn total_food(&self) -> f32 {
        self.density.as_slice().iter().sum()
    }

    /// Read-only access to the fertility grid.
    #[must_use]
    pub fn fertility(&self) -> &Grid<f32> {
        &self.fertility
    }

    /// Read-only access to the occupancy depletion grid.
    #[must_use]
    pub fn occupancy_depletion(&self) -> &Grid<f32> {
        self.occupancy_depletion.grid()
    }

    /// Read-only access to the active food config.
    #[must_use]
    pub fn config(&self) -> &FoodResourceConfig {
        &self.config
    }

    /// Read-only access to the density grid.
    #[must_use]
    pub fn density_grid(&self) -> &Grid<f32> {
        &self.density
    }

    /// Replace the food config, preserving or resetting dynamic food state per transition rules.
    pub fn apply_config_transition(&mut self, config: FoodResourceConfig) {
        reconfigure::apply_config_transition(
            &mut self.config,
            config,
            &mut self.occupancy_depletion,
        );
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.density.width()
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.density.height()
    }

    /// Generate and store a fertility map from the configured layers and world seed.
    pub fn seed_fertility(&mut self, world_seed: u64) {
        self.fertility = fertility::generate_fertility(
            self.width(),
            self.height(),
            &self.config.fertility,
            world_seed,
        );
    }

    /// Grow food with optional occupancy-driven suppression.
    pub fn grow<T: Clone>(
        &mut self,
        barriers: &Grid<bool>,
        occupancy: &Grid<Option<T>>,
        tick: u64,
        rng: &mut impl Rng,
    ) -> FoodGrowthSummary {
        let total_cells = self.width() as usize * self.height() as usize;
        if total_cells == 0 {
            return FoodGrowthSummary::default();
        }

        let depletion_summary = self.occupancy_depletion.recover_and_deposit(
            barriers,
            occupancy,
            &self.config.occupancy_depletion,
        );

        let suppressed = apply_growth(
            GrowthContext {
                density: &mut self.density,
                fertility: &self.fertility,
                occupancy_depletion: &self.occupancy_depletion,
                config: &self.config,
                barriers,
                edge_mode: self.edge_mode,
                tick,
                growth_scratch: &mut self.growth_scratch,
            },
            rng,
        );

        FoodGrowthSummary {
            mean_occupancy_depletion: depletion_summary.mean_depletion,
            occupied_cells_with_depletion: depletion_summary.occupied_cells,
            growth_suppressed_by_occupancy_depletion: suppressed,
        }
    }

    /// Seed initial food distribution and clear dynamic depletion memory.
    pub fn seed_density(&mut self, barriers: &Grid<bool>, rng: &mut impl Rng) {
        use rand::seq::SliceRandom;

        let width = self.density.width();
        let height = self.density.height();

        for y in 0..height {
            for x in 0..width {
                self.density.set(x, y, 0.0);
            }
        }
        self.occupancy_depletion.reset();

        let mut candidates = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if *barriers.get(x, y) {
                    continue;
                }
                candidates.push(Position::new(x, y));
            }
        }
        if candidates.is_empty() {
            return;
        }

        let coverage = self.config.initial_coverage.clamp(0.0, 1.0);
        let target = ((coverage * candidates.len() as f32).round() as usize).min(candidates.len());
        if target == 0 {
            return;
        }

        let density = self
            .config
            .initial_density
            .clamp(0.0, self.config.max_density);

        candidates.shuffle(rng);
        for pos in candidates.into_iter().take(target) {
            self.density.set(pos.x, pos.y, density);
        }
    }
}

/// Static neighbor resolution (does not require `WorldState`).
pub(super) fn resolve_neighbor_static(
    pos: Position,
    dir: Direction,
    width: u16,
    height: u16,
    edge_mode: WorldEdgeMode,
) -> Option<Position> {
    let (dx, dy) = dir.delta();
    match edge_mode {
        WorldEdgeMode::Wrap => pos.neighbor_wrap(dx, dy, width, height),
        WorldEdgeMode::Bounded => pos.neighbor_bounded(dx, dy, width, height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    use crate::config::OccupancyDepletionConfig;
    use crate::contracts::CreatureId;

    fn food_config() -> FoodResourceConfig {
        FoodResourceConfig {
            fertility: crate::config::FertilityConfig {
                enabled: false,
                ..crate::config::FertilityConfig::default()
            },
            ..FoodResourceConfig::default()
        }
    }

    fn barriers(width: u16, height: u16) -> Grid<bool> {
        Grid::new(width, height, false)
    }

    fn empty_occupancy(width: u16, height: u16) -> Grid<Option<CreatureId>> {
        Grid::new(width, height, None)
    }

    fn single_occupancy(width: u16, height: u16, pos: Position) -> Grid<Option<CreatureId>> {
        let mut occupancy = empty_occupancy(width, height);
        let mut ids: SlotMap<CreatureId, ()> = SlotMap::with_key();
        occupancy.set(pos.x, pos.y, Some(ids.insert(())));
        occupancy
    }

    #[test]
    fn grow_with_disabled_occupancy_depletion_is_identity_for_occupied_cells() {
        let mut config = food_config();
        config.growth_rate = 0.5;
        config.max_density = 2.0;
        config.occupancy_depletion = OccupancyDepletionConfig {
            enabled: false,
            deposit_per_occupied_tick: 0.4,
        };
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);
        food.set_food(Position::new(0, 0), 0.8);

        let mut rng = SmallRng::seed_from_u64(1);
        let summary = food.grow(&barriers, &occupied, 0, &mut rng);

        assert!((food.food_at(Position::new(0, 0)) - 1.2).abs() < 1e-6);
        assert_eq!(food.occupancy_depletion().as_slice(), &[0.0]);
        assert_eq!(summary, FoodGrowthSummary::default());
    }

    #[test]
    fn grow_deposits_to_occupied_cells_and_recovers_when_vacated() {
        let mut config = food_config();
        config.growth_rate = 0.0;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let barriers = barriers(1, 1);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let empty = empty_occupancy(1, 1);

        let mut rng = SmallRng::seed_from_u64(2);
        let _ = food.grow(&barriers, &occupied, 0, &mut rng);
        assert!((food.occupancy_depletion().as_slice()[0] - 0.2).abs() < 1e-6);

        let _ = food.grow(&barriers, &empty, 1, &mut rng);
        assert!((food.occupancy_depletion().as_slice()[0] - 0.17).abs() < 1e-6);
    }

    #[test]
    fn grow_suppresses_local_growth_in_occupied_cells() {
        let mut config = food_config();
        config.growth_rate = 0.5;
        config.max_density = 2.0;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);
        food.set_food(Position::new(0, 0), 0.8);

        let mut rng = SmallRng::seed_from_u64(3);
        let summary = food.grow(&barriers, &occupied, 0, &mut rng);

        assert!((food.food_at(Position::new(0, 0)) - 1.148).abs() < 1e-6);
        assert!((summary.growth_suppressed_by_occupancy_depletion - 0.052).abs() < 1e-6);
    }

    #[test]
    fn grow_suppresses_spread_into_occupied_target_cells() {
        let mut config = food_config();
        config.growth_rate = 0.2;
        config.max_density = 2.0;
        config.spread_threshold_ratio = 0.375;
        config.spread_density_ratio = 1.0;
        config.recovery_floor_ratio = 0.0;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut food = FoodResource::new(2, 1, config, WorldEdgeMode::Bounded);
        let occupied = single_occupancy(2, 1, Position::new(1, 0));
        let barriers = barriers(2, 1);
        food.set_food(Position::new(0, 0), 0.75);

        let mut rng = SmallRng::seed_from_u64(4);
        let summary = food.grow(&barriers, &occupied, 0, &mut rng);

        assert!((food.food_at(Position::new(0, 0)) - 0.9).abs() < 1e-6);
        assert!((food.food_at(Position::new(1, 0)) - 0.1305).abs() < 1e-6);
        assert!((summary.growth_suppressed_by_occupancy_depletion - 0.0195).abs() < 1e-6);
    }

    #[test]
    fn grow_suppresses_recovery_spawn_in_occupied_cells() {
        let mut config = food_config();
        config.growth_rate = 0.5;
        config.max_density = 2.0;
        config.recovery_spawn_rate = 1.0;
        config.recovery_floor_ratio = 1.0;
        config.spread_threshold_ratio = 1.1;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);

        let mut rng = SmallRng::seed_from_u64(5);
        let summary = food.grow(&barriers, &occupied, 0, &mut rng);

        assert!((food.food_at(Position::new(0, 0)) - 0.87).abs() < 1e-6);
        assert!((summary.growth_suppressed_by_occupancy_depletion - 0.13).abs() < 1e-6);
    }

    #[test]
    fn grow_respects_internal_minimum_growth_multiplier_floor() {
        let mut config = food_config();
        config.growth_rate = 1.0;
        config.max_density = 2.0;
        config.occupancy_depletion.deposit_per_occupied_tick = 1.0;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);
        food.set_food(Position::new(0, 0), 1.0);

        let mut rng = SmallRng::seed_from_u64(6);
        let _ = food.grow(&barriers, &occupied, 0, &mut rng);

        assert!((food.food_at(Position::new(0, 0)) - 1.35).abs() < 1e-6);
    }

    #[test]
    fn suppression_summary_tracks_applied_growth_when_density_is_clamped() {
        let mut config = food_config();
        config.growth_rate = 1.0;
        config.max_density = 1.0;
        config.occupancy_depletion.deposit_per_occupied_tick = 1.0;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);
        food.set_food(Position::new(0, 0), 0.9);

        let mut rng = SmallRng::seed_from_u64(10);
        let summary = food.grow(&barriers, &occupied, 0, &mut rng);

        assert!((food.food_at(Position::new(0, 0)) - 1.0).abs() < 1e-6);
        assert_eq!(summary.growth_suppressed_by_occupancy_depletion, 0.0);
    }

    #[test]
    fn apply_config_transition_resets_depletion_when_enabled_toggles() {
        let mut config = food_config();
        config.growth_rate = 0.0;
        config.occupancy_depletion.enabled = true;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut food = FoodResource::new(1, 1, config.clone(), WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);

        let mut rng = SmallRng::seed_from_u64(7);
        let _ = food.grow(&barriers, &occupied, 0, &mut rng);
        assert!(food.occupancy_depletion().as_slice()[0] > 0.0);

        config.occupancy_depletion.enabled = false;
        food.apply_config_transition(config);

        assert_eq!(food.occupancy_depletion().as_slice(), &[0.0]);
    }

    #[test]
    fn apply_config_transition_preserves_depletion_when_only_rate_changes() {
        let mut config = food_config();
        config.growth_rate = 0.0;
        config.occupancy_depletion.enabled = true;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.05;
        let mut food = FoodResource::new(1, 1, config.clone(), WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);

        let mut rng = SmallRng::seed_from_u64(8);
        let _ = food.grow(&barriers, &occupied, 0, &mut rng);
        let before = food.occupancy_depletion().as_slice()[0];

        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        food.apply_config_transition(config);

        assert_eq!(food.occupancy_depletion().as_slice(), &[before]);
    }

    #[test]
    fn occupied_cells_summary_stays_zero_when_deposit_rate_is_zero() {
        let mut config = food_config();
        config.growth_rate = 0.0;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.0;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);

        let mut rng = SmallRng::seed_from_u64(11);
        let summary = food.grow(&barriers, &occupied, 0, &mut rng);

        assert_eq!(summary.occupied_cells_with_depletion, 0);
        assert_eq!(food.occupancy_depletion().as_slice(), &[0.0]);
    }

    #[test]
    fn seed_density_clears_existing_depletion() {
        let mut config = food_config();
        config.growth_rate = 0.0;
        config.initial_coverage = 1.0;
        config.initial_density = 0.5;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut food = FoodResource::new(1, 1, config, WorldEdgeMode::Wrap);
        let occupied = single_occupancy(1, 1, Position::new(0, 0));
        let barriers = barriers(1, 1);

        let mut rng = SmallRng::seed_from_u64(9);
        let _ = food.grow(&barriers, &occupied, 0, &mut rng);
        assert!(food.occupancy_depletion().as_slice()[0] > 0.0);

        food.seed_density(&barriers, &mut rng);

        assert_eq!(food.occupancy_depletion().as_slice(), &[0.0]);
        assert!((food.food_at(Position::new(0, 0)) - 0.5).abs() < 1e-6);
    }
}
