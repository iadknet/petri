use rand::Rng;

use crate::config::FoodResourceConfig;
use crate::contracts::{Direction, Position};
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
    /// `tick` is accepted for future annealing support (currently unused).
    /// `edge_mode` and world dimensions are read from the density grid itself.
    ///
    /// When `self.config.fertility.enabled` is false, all fertility multipliers
    /// are 1.0 (identity — no behavior change from pre-fertility code).
    pub fn grow(
        &mut self,
        barriers: &Grid<bool>,
        _tick: u64,
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

            let delta = source * growth_rate;
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
            self.add_food_clamped(target, delta * spread_density_ratio, max_density);
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
            self.add_food_clamped(pos, spawn_delta, max_density);
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
    use crate::config::FoodResourceConfig;

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
}
