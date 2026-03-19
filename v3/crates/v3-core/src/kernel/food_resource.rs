use rand::Rng;

use crate::config::FoodResourceConfig;
use crate::contracts::{Direction, Position};
use crate::kernel::fertility;
use crate::kernel::Grid;

/// Self-contained food substrate: density grid, fertility map, and growth logic.
///
/// Owns the `FoodResourceConfig` so that growth parameters are read from `self.config`
/// rather than threaded through every call site. Barrier queries are passed in by
/// the owning `WorldState` because barriers remain a world-level concern.
pub struct FoodResource {
    density: Grid<f32>,
    fertility: Grid<f32>,
    config: FoodResourceConfig,
    /// Reusable scratch buffer for `grow` to avoid per-tick allocation.
    growth_scratch: Vec<f32>,
}

impl FoodResource {
    /// Create a new food resource with zero density and zero fertility everywhere.
    pub fn new(width: u16, height: u16, config: FoodResourceConfig) -> Self {
        Self {
            density: Grid::new(width, height, 0.0),
            fertility: Grid::new(width, height, 0.0),
            config,
            growth_scratch: Vec::new(),
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Get food density at a position.
    pub fn food_at(&self, pos: Position) -> f32 {
        *self.density.get(pos.x, pos.y)
    }

    /// Consume all food on a cell. Returns the amount consumed (0 if empty).
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
    pub fn total_food(&self) -> f32 {
        self.density.as_slice().iter().sum()
    }

    /// Read-only access to the fertility grid.
    pub fn fertility(&self) -> &Grid<f32> {
        &self.fertility
    }

    /// Read-only access to the active food config.
    pub fn config(&self) -> &FoodResourceConfig {
        &self.config
    }

    /// Read-only access to the density grid.
    pub fn density_grid(&self) -> &Grid<f32> {
        &self.density
    }

    /// Replace the food config (used for live-tuning).
    pub fn update_config(&mut self, config: FoodResourceConfig) {
        self.config = config;
    }

    pub fn width(&self) -> u16 {
        self.density.width()
    }

    pub fn height(&self) -> u16 {
        self.density.height()
    }

    // ── Fertility ─────────────────────────────────────────────────────────────

    /// Generate and store a fertility map from the configured layers and world seed.
    ///
    /// Must be called before `seed_density` during world startup so that the
    /// fertility grid is ready before food growth begins.
    pub fn seed_fertility(&mut self, rng: &mut impl Rng, world_seed: u64) {
        self.fertility = fertility::generate_fertility(
            self.width(),
            self.height(),
            &self.config.fertility,
            rng,
            world_seed,
        );
    }

    // ── Growth ────────────────────────────────────────────────────────────────

    /// Add food clamped to max density.
    fn add_food_clamped(&mut self, pos: Position, delta: f32, max_density: f32) {
        if delta <= 0.0 {
            return;
        }
        let current = *self.density.get(pos.x, pos.y);
        self.density
            .set(pos.x, pos.y, (current + delta).min(max_density));
    }

    /// Grow food using v1-aligned mechanics: proportional growth, threshold spread,
    /// and low-density recovery spawn.
    ///
    /// `barriers` is provided by the owning `WorldState`.
    /// `tick` drives fertility annealing (ramping effective fertility over time).
    /// `edge_mode` and world dimensions are read from the density grid itself.
    ///
    /// When `self.config.fertility.enabled` is false, all fertility multipliers
    /// are 1.0 (identity — no behavior change from pre-fertility code).
    pub fn grow(
        &mut self,
        barriers: &Grid<bool>,
        tick: u64,
        rng: &mut impl Rng,
        width: u16,
        height: u16,
        edge_mode: crate::config::WorldEdgeMode,
    ) {
        let total_cells = width as usize * height as usize;
        if total_cells == 0 {
            return;
        }

        let growth_rate = self.config.growth_rate.max(0.0);
        let max_density = self.config.max_density.max(0.0);
        if max_density <= 0.0 {
            return;
        }

        let spread_threshold = max_density * self.config.spread_threshold_ratio.clamp(0.0, 1.0);
        let spread_density_ratio = self.config.spread_density_ratio.clamp(0.0, 1.0);
        let recovery_floor = self.config.recovery_floor_ratio.clamp(0.0, 1.0);
        let recovery_spawn_rate = self.config.recovery_spawn_rate.clamp(0.0, 1.0);

        // Compute effective fertility range once per tick.
        let fertility_enabled = self.config.fertility.enabled;
        let (eff_min, eff_max) = if fertility_enabled {
            fertility::effective_fertility_range(
                &self.config.annealing,
                self.config.fertility.min_fertility,
                self.config.fertility.max_fertility,
                tick,
            )
        } else {
            (1.0, 1.0)
        };

        // Snapshot food densities into a reusable buffer (avoids per-tick allocation).
        self.growth_scratch.clear();
        self.growth_scratch.extend(
            self.density
                .as_slice()
                .iter()
                .map(|v| v.clamp(0.0, max_density)),
        );
        let total_food: f32 = self.growth_scratch.iter().sum();
        let average_density_ratio =
            (total_food / (total_cells as f32 * max_density)).clamp(0.0, 1.0);

        for idx in 0..total_cells {
            let source = self.growth_scratch[idx];
            let x = (idx % width as usize) as u16;
            let y = (idx / width as usize) as u16;
            let pos = Position::new(x, y);

            if *barriers.get(x, y) {
                continue;
            }

            // Proportional growth with fertility multiplier.
            let cell_fertility = if fertility_enabled {
                fertility::map_fertility(*self.fertility.get(x, y), eff_min, eff_max)
            } else {
                1.0
            };
            let delta = source * growth_rate * cell_fertility;
            self.add_food_clamped(pos, delta, max_density);

            if source < spread_threshold || delta <= 0.0 {
                continue;
            }

            let mut neighbors = [Position::new(0, 0); 4];
            let mut ncount = 0;
            for dir in [Direction::N, Direction::E, Direction::S, Direction::W] {
                let npos = resolve_neighbor_static(pos, dir, width, height, edge_mode);
                let Some(npos) = npos else {
                    continue;
                };
                if *barriers.get(npos.x, npos.y) {
                    continue;
                }
                neighbors[ncount] = npos;
                ncount += 1;
            }
            if ncount == 0 {
                continue;
            }
            let target = neighbors[rng.gen_range(0..ncount)];
            // Spread deposit with neighbor's fertility multiplier.
            let neighbor_fertility = if fertility_enabled {
                fertility::map_fertility(*self.fertility.get(target.x, target.y), eff_min, eff_max)
            } else {
                1.0
            };
            self.add_food_clamped(
                target,
                delta * spread_density_ratio * neighbor_fertility,
                max_density,
            );
        }

        if average_density_ratio >= recovery_floor {
            return;
        }

        let spawn_attempts = (total_cells as f32 * recovery_spawn_rate).round() as usize;
        let spawn_delta = max_density * growth_rate;
        if spawn_attempts == 0 || spawn_delta <= 0.0 {
            return;
        }

        for _ in 0..spawn_attempts {
            let idx = rng.gen_range(0..total_cells);
            let x = (idx % width as usize) as u16;
            let y = (idx / width as usize) as u16;
            let pos = Position::new(x, y);
            if *barriers.get(x, y) {
                continue;
            }
            // Recovery spawn with fertility multiplier.
            let cell_fertility = if fertility_enabled {
                fertility::map_fertility(*self.fertility.get(x, y), eff_min, eff_max)
            } else {
                1.0
            };
            self.add_food_clamped(pos, spawn_delta * cell_fertility, max_density);
        }
    }

    // ── Seeding ───────────────────────────────────────────────────────────────

    /// Seed initial food distribution.
    /// Clears prior food and samples exact target coverage over non-barrier cells.
    pub fn seed_density(
        &mut self,
        barriers: &Grid<bool>,
        rng: &mut impl Rng,
        config: &FoodResourceConfig,
    ) {
        use rand::seq::SliceRandom;

        let width = self.density.width();
        let height = self.density.height();

        for y in 0..height {
            for x in 0..width {
                self.density.set(x, y, 0.0);
            }
        }

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

        let coverage = config.initial_coverage.clamp(0.0, 1.0);
        let target = ((coverage * candidates.len() as f32).round() as usize).min(candidates.len());
        if target == 0 {
            return;
        }

        let density = config.initial_density.clamp(0.0, config.max_density);

        candidates.shuffle(rng);
        for pos in candidates.into_iter().take(target) {
            self.density.set(pos.x, pos.y, density);
        }
    }
}

/// Static neighbor resolution (does not require WorldState).
/// Mirrors `WorldState::resolve_neighbor` but operates on raw dimensions and edge mode.
fn resolve_neighbor_static(
    pos: Position,
    dir: Direction,
    width: u16,
    height: u16,
    edge_mode: crate::config::WorldEdgeMode,
) -> Option<Position> {
    let (dx, dy) = dir.delta();
    match edge_mode {
        crate::config::WorldEdgeMode::Wrap => pos.neighbor_wrap(dx, dy, width, height),
        crate::config::WorldEdgeMode::Bounded => pos.neighbor_bounded(dx, dy, width, height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FertilityConfig, FoodResourceConfig, WorldEdgeMode};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn default_food() -> FoodResource {
        FoodResource::new(4, 4, FoodResourceConfig::default())
    }

    #[test]
    fn food_at_returns_zero_initially() {
        let food = default_food();
        assert!((food.food_at(Position::new(0, 0))).abs() < f32::EPSILON);
        assert!((food.food_at(Position::new(3, 3))).abs() < f32::EPSILON);
    }

    #[test]
    fn set_food_and_food_at_roundtrip() {
        let mut food = default_food();
        let pos = Position::new(2, 1);
        food.set_food(pos, 0.75);
        assert!((food.food_at(pos) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn consume_returns_amount_and_clears() {
        let mut food = default_food();
        let pos = Position::new(1, 2);
        food.set_food(pos, 0.42);
        let consumed = food.consume(pos);
        assert!((consumed - 0.42).abs() < f32::EPSILON);
        assert!((food.food_at(pos)).abs() < f32::EPSILON);
    }

    #[test]
    fn consume_returns_zero_on_empty_cell() {
        let mut food = default_food();
        let consumed = food.consume(Position::new(0, 0));
        assert!((consumed).abs() < f32::EPSILON);
    }

    #[test]
    fn total_food_sums_all_cells() {
        let mut food = default_food();
        food.set_food(Position::new(0, 0), 0.5);
        food.set_food(Position::new(1, 0), 0.3);
        food.set_food(Position::new(3, 3), 0.2);
        assert!((food.total_food() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn update_config_changes_active_config() {
        let mut food = default_food();
        let original_rate = food.config().growth_rate;
        let mut new_config = food.config().clone();
        new_config.growth_rate = original_rate + 1.0;
        food.update_config(new_config);
        assert!((food.config().growth_rate - (original_rate + 1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn width_and_height_match_construction() {
        let food = FoodResource::new(10, 20, FoodResourceConfig::default());
        assert_eq!(food.width(), 10);
        assert_eq!(food.height(), 20);
    }

    #[test]
    fn fertility_grid_initialized_to_zero() {
        let food = default_food();
        for (_, _, v) in food.fertility().iter() {
            assert!((v).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn density_grid_accessor_matches_food_at() {
        let mut food = default_food();
        food.set_food(Position::new(2, 3), 0.6);
        assert!(
            (food.density_grid().get(2, 3) - food.food_at(Position::new(2, 3))).abs()
                < f32::EPSILON
        );
    }

    // ── Fertility integration tests ─────────────────────────────────────────

    /// Helper: create a FoodResource with uniform fertility (all cells get the
    /// same raw value mapped to the given min/max range).
    fn food_with_uniform_fertility(
        width: u16,
        height: u16,
        min_fert: f32,
        max_fert: f32,
    ) -> FoodResource {
        let config = FoodResourceConfig {
            growth_rate: 0.05,
            max_density: 1.0,
            spread_threshold_ratio: 1.1, // disable spread
            recovery_floor_ratio: 0.0,   // disable recovery
            recovery_spawn_rate: 0.0,
            fertility: FertilityConfig {
                enabled: true,
                min_fertility: min_fert,
                max_fertility: max_fert,
                layers: vec![crate::config::FertilityLayer {
                    algorithm: crate::config::FertilityAlgorithm::Uniform { value: 1.0 },
                    weight: 1.0,
                }],
            },
            ..FoodResourceConfig::default()
        };
        // Uniform { value: 1.0 } fills fertility grid with 1.0 (raw).
        // map_fertility(1.0, min, max) = max (since (1+1)/2 = 1.0, so min + 1.0*(max-min) = max).
        let mut food = FoodResource::new(width, height, config);
        let mut rng = SmallRng::seed_from_u64(42);
        food.seed_fertility(&mut rng, 42);
        food
    }

    #[test]
    fn fertility_zero_stops_proportional_growth() {
        // Uniform raw=1.0, min=0, max=0 → effective fertility = 0.0 for all cells.
        let mut food = food_with_uniform_fertility(4, 4, 0.0, 0.0);
        let barriers = Grid::new(4, 4, false);
        food.set_food(Position::new(1, 1), 0.5);
        let before = food.food_at(Position::new(1, 1));
        let mut rng = SmallRng::seed_from_u64(1);
        food.grow(&barriers, 0, &mut rng, 4, 4, WorldEdgeMode::Wrap);
        let after = food.food_at(Position::new(1, 1));
        assert!(
            (after - before).abs() < f32::EPSILON,
            "expected no growth with zero fertility, before={before}, after={after}"
        );
    }

    #[test]
    fn fertility_doubles_growth() {
        // Uniform raw=1.0, min=2, max=2 → effective fertility = 2.0 for all cells.
        let mut food = food_with_uniform_fertility(4, 4, 2.0, 2.0);
        let barriers = Grid::new(4, 4, false);
        food.set_food(Position::new(1, 1), 0.5);
        let mut rng = SmallRng::seed_from_u64(1);
        food.grow(&barriers, 0, &mut rng, 4, 4, WorldEdgeMode::Wrap);
        // Expected: 0.5 + 0.5 * 0.05 * 2.0 = 0.55
        let after = food.food_at(Position::new(1, 1));
        assert!(
            (after - 0.55).abs() < 1e-6,
            "expected 0.55 with 2x fertility, got {after}"
        );
    }

    #[test]
    fn fertility_disabled_matches_baseline() {
        // Two FoodResources: one with fertility disabled, one enabled at 1.0/1.0.
        // Both should produce identical results after grow.
        let disabled_config = FoodResourceConfig {
            growth_rate: 0.05,
            max_density: 1.0,
            spread_threshold_ratio: 1.1, // disable spread
            recovery_floor_ratio: 0.0,
            recovery_spawn_rate: 0.0,
            fertility: FertilityConfig {
                enabled: false,
                ..FertilityConfig::default()
            },
            ..FoodResourceConfig::default()
        };
        // Uniform raw=1.0, min=1, max=1 → effective fertility = 1.0 for all cells.
        // This is identity — should match disabled behavior.
        let mut food_disabled = FoodResource::new(4, 4, disabled_config);
        let mut food_enabled = food_with_uniform_fertility(4, 4, 1.0, 1.0);

        let barriers = Grid::new(4, 4, false);

        // Set same food pattern on both.
        for y in 0..4u16 {
            for x in 0..4u16 {
                let val = (x as f32 + y as f32 * 4.0) / 16.0;
                food_disabled.set_food(Position::new(x, y), val);
                food_enabled.set_food(Position::new(x, y), val);
            }
        }

        let mut rng_d = SmallRng::seed_from_u64(99);
        let mut rng_e = SmallRng::seed_from_u64(99);
        food_disabled.grow(&barriers, 0, &mut rng_d, 4, 4, WorldEdgeMode::Wrap);
        food_enabled.grow(&barriers, 0, &mut rng_e, 4, 4, WorldEdgeMode::Wrap);

        for y in 0..4u16 {
            for x in 0..4u16 {
                let d = food_disabled.food_at(Position::new(x, y));
                let e = food_enabled.food_at(Position::new(x, y));
                assert!(
                    (d - e).abs() < 1e-6,
                    "mismatch at ({x},{y}): disabled={d}, enabled-at-1.0={e}"
                );
            }
        }
    }

    #[test]
    fn recovery_spawn_respects_fertility() {
        // Left half (x < 2): raw = -1.0 (barren), Right half (x >= 2): raw = 1.0 (fertile).
        // With min=0.0, max=2.0:
        //   map_fertility(-1.0, 0, 2) = 0.0 (barren)
        //   map_fertility(1.0, 0, 2) = 2.0 (fertile)
        let config = FoodResourceConfig {
            growth_rate: 0.05,
            max_density: 1.0,
            spread_threshold_ratio: 1.1, // disable spread
            recovery_floor_ratio: 1.0,   // always trigger recovery
            recovery_spawn_rate: 1.0,    // max spawn attempts
            fertility: FertilityConfig {
                enabled: true,
                min_fertility: 0.0,
                max_fertility: 2.0,
                layers: vec![], // won't matter, we'll set fertility grid manually
            },
            ..FoodResourceConfig::default()
        };
        let mut food = FoodResource::new(4, 4, config);
        // Manually set fertility grid: left half = -1.0 (barren), right half = 1.0 (fertile).
        for y in 0..4u16 {
            for x in 0..4u16 {
                if x < 2 {
                    food.fertility.set(x, y, -1.0);
                } else {
                    food.fertility.set(x, y, 1.0);
                }
            }
        }

        let barriers = Grid::new(4, 4, false);
        // Run many ticks to get recovery spawns.
        let mut rng = SmallRng::seed_from_u64(42);
        for tick in 0..50 {
            food.grow(&barriers, tick, &mut rng, 4, 4, WorldEdgeMode::Wrap);
        }

        // Barren cells (left half) should have no food.
        for y in 0..4u16 {
            for x in 0..2u16 {
                assert!(
                    food.food_at(Position::new(x, y)) < f32::EPSILON,
                    "barren cell ({x},{y}) has food: {}",
                    food.food_at(Position::new(x, y))
                );
            }
        }
        // Fertile cells should have some food after 50 recovery ticks.
        let fertile_food: f32 = (0..4u16)
            .flat_map(|y| (2..4u16).map(move |x| Position::new(x, y)))
            .map(|p| food.food_at(p))
            .sum();
        assert!(
            fertile_food > 0.0,
            "expected some food in fertile cells after recovery"
        );
    }

    #[test]
    fn seed_fertility_populates_grid() {
        let config = FoodResourceConfig {
            fertility: FertilityConfig {
                enabled: true,
                ..FertilityConfig::default()
            },
            ..FoodResourceConfig::default()
        };
        let mut food = FoodResource::new(32, 32, config);
        let mut rng = SmallRng::seed_from_u64(42);
        food.seed_fertility(&mut rng, 42);
        // Verify the grid has variation (not all zeros).
        let has_nonzero = food.fertility().iter().any(|(_, _, v)| v.abs() > 0.01);
        assert!(
            has_nonzero,
            "expected fertility grid to have non-zero values after seeding"
        );
    }
}
