use rand::seq::SliceRandom;
use rand::Rng;

use crate::config::{SimulationConfig, WorldEdgeMode};
use crate::contracts::{CreatureId, Direction, Position};
use crate::kernel::Grid;

/// Central world state: food density, barriers, and creature occupancy.
///
/// Invariants maintained by callers:
/// - Single occupancy per cell.
/// - Validity checks (barrier-free, unoccupied) must be applied before placing/moving creatures.
pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub edge_mode: WorldEdgeMode,
    food_density: Grid<f32>,
    barriers: Grid<bool>,
    creature_at: Grid<Option<CreatureId>>,
    /// Reusable scratch buffer for `grow_food` to avoid per-tick allocation.
    food_snapshot: Vec<f32>,
}

impl WorldState {
    /// Create an empty world with no food, no barriers, no creatures.
    pub fn new(width: u16, height: u16, edge_mode: WorldEdgeMode) -> Self {
        Self {
            width,
            height,
            edge_mode,
            food_density: Grid::new(width, height, 0.0),
            barriers: Grid::new(width, height, false),
            creature_at: Grid::new(width, height, None),
            food_snapshot: Vec::new(),
        }
    }

    // ── Food ─────────────────────────────────────────────────────────────────

    /// Seed initial food distribution.
    /// Clears prior food and samples exact target coverage over non-barrier cells.
    pub fn seed_food(&mut self, rng: &mut impl Rng, config: &SimulationConfig) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.food_density.set(x, y, 0.0);
            }
        }

        let mut candidates = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.barriers.get(x, y) {
                    continue;
                }
                candidates.push(Position::new(x, y));
            }
        }
        if candidates.is_empty() {
            return;
        }

        let coverage = config.world.food.initial_coverage.clamp(0.0, 1.0);
        let target = ((coverage * candidates.len() as f32).round() as usize).min(candidates.len());
        if target == 0 {
            return;
        }

        let density = config
            .world
            .food
            .initial_density
            .clamp(0.0, config.world.food.max_density);

        candidates.shuffle(rng);
        for pos in candidates.into_iter().take(target) {
            self.food_density.set(pos.x, pos.y, density);
        }
    }

    fn add_food_clamped(&mut self, pos: Position, delta: f32, max_density: f32) {
        if delta <= 0.0 {
            return;
        }
        let current = *self.food_density.get(pos.x, pos.y);
        self.food_density
            .set(pos.x, pos.y, (current + delta).min(max_density));
    }

    /// Grow food phase using v1-aligned mechanics:
    /// proportional growth, threshold spread, and low-density recovery spawn.
    pub fn grow_food(&mut self, rng: &mut impl Rng, config: &SimulationConfig) {
        let total_cells = self.width as usize * self.height as usize;
        if total_cells == 0 {
            return;
        }

        let growth_rate = config.world.food.growth_rate.max(0.0);
        let max_density = config.world.food.max_density.max(0.0);
        if max_density <= 0.0 {
            return;
        }

        let spread_threshold =
            max_density * config.world.food.spread_threshold_ratio.clamp(0.0, 1.0);
        let spread_density_ratio = config.world.food.spread_density_ratio.clamp(0.0, 1.0);
        let recovery_floor = config.world.food.recovery_floor_ratio.clamp(0.0, 1.0);
        let recovery_spawn_rate = config.world.food.recovery_spawn_rate.clamp(0.0, 1.0);
        // Snapshot food densities into a reusable buffer (avoids per-tick allocation).
        self.food_snapshot.clear();
        self.food_snapshot.extend(
            self.food_density
                .as_slice()
                .iter()
                .map(|v| v.clamp(0.0, max_density)),
        );
        let total_food: f32 = self.food_snapshot.iter().sum();
        let average_density_ratio =
            (total_food / (total_cells as f32 * max_density)).clamp(0.0, 1.0);

        for idx in 0..total_cells {
            let source = self.food_snapshot[idx];
            let x = (idx % self.width as usize) as u16;
            let y = (idx / self.width as usize) as u16;
            let pos = Position::new(x, y);

            if self.is_barrier(pos) {
                continue;
            }

            let delta = source * growth_rate;
            self.add_food_clamped(pos, delta, max_density);

            if source < spread_threshold || delta <= 0.0 {
                continue;
            }

            let mut neighbors = Vec::with_capacity(4);
            for dir in [Direction::N, Direction::E, Direction::S, Direction::W] {
                let Some(npos) = self.resolve_neighbor(pos, dir) else {
                    continue;
                };
                if self.is_barrier(npos) {
                    continue;
                }
                neighbors.push(npos);
            }
            if neighbors.is_empty() {
                continue;
            }
            let target = neighbors[rng.gen_range(0..neighbors.len())];
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
            let x = (idx % self.width as usize) as u16;
            let y = (idx / self.width as usize) as u16;
            let pos = Position::new(x, y);
            if self.is_barrier(pos) {
                continue;
            }
            self.add_food_clamped(pos, spawn_delta, max_density);
        }
    }

    /// Consume all food on a cell. Returns the amount consumed (0 if empty).
    pub fn consume_food(&mut self, pos: Position) -> f32 {
        let amount = *self.food_density.get(pos.x, pos.y);
        self.food_density.set(pos.x, pos.y, 0.0);
        amount
    }

    /// Get food density at a position.
    pub fn food_at(&self, pos: Position) -> f32 {
        *self.food_density.get(pos.x, pos.y)
    }

    /// Set food density at a position directly.
    pub fn set_food(&mut self, pos: Position, value: f32) {
        self.food_density.set(pos.x, pos.y, value);
    }

    // ── Barriers ─────────────────────────────────────────────────────────────

    pub fn is_barrier(&self, pos: Position) -> bool {
        *self.barriers.get(pos.x, pos.y)
    }

    pub fn set_barrier(&mut self, pos: Position, value: bool) {
        self.barriers.set(pos.x, pos.y, value);
    }

    // ── Occupancy ────────────────────────────────────────────────────────────

    /// Get creature occupying a cell.
    pub fn creature_at(&self, pos: Position) -> Option<CreatureId> {
        *self.creature_at.get(pos.x, pos.y)
    }

    /// Place a creature on a cell. Caller must ensure the cell is valid.
    pub fn place_creature(&mut self, pos: Position, id: CreatureId) {
        self.creature_at.set(pos.x, pos.y, Some(id));
    }

    /// Remove creature occupancy from a cell.
    pub fn remove_creature(&mut self, pos: Position) {
        self.creature_at.set(pos.x, pos.y, None);
    }

    // ── Spatial primitives ───────────────────────────────────────────────────

    /// Resolve a neighbor position applying the world's edge mode.
    /// Returns None for bounded mode neighbors that fall outside bounds.
    pub fn resolve_neighbor(&self, pos: Position, dir: Direction) -> Option<Position> {
        let (dx, dy) = dir.delta();
        match self.edge_mode {
            WorldEdgeMode::Wrap => pos.neighbor_wrap(dx, dy, self.width, self.height),
            WorldEdgeMode::Bounded => pos.neighbor_bounded(dx, dy, self.width, self.height),
        }
    }

    /// Whether a resolved, in-bounds cell is valid as a move/spawn target.
    /// Requires: no barrier, no current occupant.
    pub fn is_valid_target_cell(&self, pos: Position) -> bool {
        !self.is_barrier(pos) && self.creature_at(pos).is_none()
    }

    // ── Diagnostics ──────────────────────────────────────────────────────────

    /// Total food across all cells (for testing and diagnostics).
    pub fn total_food(&self) -> f32 {
        let mut sum = 0.0f32;
        for y in 0..self.height {
            for x in 0..self.width {
                sum += *self.food_density.get(x, y);
            }
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    fn default_config() -> SimulationConfig {
        SimulationConfig::default()
    }

    fn small_wrap_world() -> WorldState {
        WorldState::new(10, 10, WorldEdgeMode::Wrap)
    }

    #[test]
    fn new_world_empty() {
        let w = small_wrap_world();
        assert!((w.total_food() - 0.0).abs() < 1e-6);
        for y in 0..10u16 {
            for x in 0..10u16 {
                let pos = Position::new(x, y);
                assert!(!w.is_barrier(pos));
                assert!(w.creature_at(pos).is_none());
            }
        }
    }

    #[test]
    fn seed_food_places_some_food() {
        let mut w = small_wrap_world();
        let mut rng = SmallRng::seed_from_u64(42);
        let cfg = default_config();
        w.seed_food(&mut rng, &cfg);
        assert!(w.total_food() > 0.0);
    }

    #[test]
    fn seed_food_cells_have_density_or_zero() {
        let mut w = small_wrap_world();
        let mut rng = SmallRng::seed_from_u64(42);
        let cfg = default_config();
        w.seed_food(&mut rng, &cfg);
        for y in 0..10u16 {
            for x in 0..10u16 {
                let f = w.food_at(Position::new(x, y));
                assert!(
                    (f - 0.0).abs() < 1e-6 || (f - cfg.world.food.initial_density).abs() < 1e-6
                );
            }
        }
    }

    #[test]
    fn seed_food_skips_barrier_cells() {
        let mut w = small_wrap_world();
        let barrier_pos = Position::new(5, 5);
        w.set_barrier(barrier_pos, true);
        let mut rng = SmallRng::seed_from_u64(0);
        let mut cfg = default_config();
        cfg.world.food.initial_coverage = 1.0; // guarantee all non-barrier cells are seeded
        w.seed_food(&mut rng, &cfg);
        assert!((w.food_at(barrier_pos) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn grow_food_increases_total_with_rate_one() {
        let mut w = WorldState::new(4, 4, WorldEdgeMode::Wrap);
        for y in 0..4u16 {
            for x in 0..4u16 {
                w.food_density.set(x, y, 0.5);
            }
        }
        let before = w.total_food();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.5;
        cfg.world.food.recovery_floor_ratio = 0.0;
        w.grow_food(&mut rng, &cfg);
        assert!(w.total_food() > before);
    }

    #[test]
    fn grow_food_clamps_at_max_density() {
        let mut w = WorldState::new(1, 1, WorldEdgeMode::Wrap);
        w.food_density.set(0, 0, 0.9);
        let mut rng = SmallRng::seed_from_u64(0);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 1.0;
        cfg.world.food.max_density = 1.0;
        cfg.world.food.recovery_floor_ratio = 0.0;
        w.grow_food(&mut rng, &cfg);
        assert!((w.food_at(Position::new(0, 0)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn grow_food_includes_occupied_cells_for_local_growth() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w = WorldState::new(1, 1, WorldEdgeMode::Wrap);
        let pos = Position::new(0, 0);
        w.food_density.set(0, 0, 0.8);
        w.place_creature(pos, id);

        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.5;
        cfg.world.food.recovery_floor_ratio = 0.0;
        let mut rng = SmallRng::seed_from_u64(0);
        w.grow_food(&mut rng, &cfg);

        assert!((w.food_at(pos) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn consume_food_returns_amount_and_clears_cell() {
        let mut w = small_wrap_world();
        let pos = Position::new(3, 3);
        w.food_density.set(3, 3, 0.42);
        let consumed = w.consume_food(pos);
        assert!((consumed - 0.42).abs() < 1e-6);
        assert!((w.food_at(pos) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn seed_food_uses_exact_coverage_count() {
        let mut w = WorldState::new(10, 1, WorldEdgeMode::Wrap);
        let mut cfg = default_config();
        cfg.world.food.initial_coverage = 0.4;
        cfg.world.food.initial_density = 1.0;
        let mut rng = SmallRng::seed_from_u64(7);
        w.seed_food(&mut rng, &cfg);
        let seeded = (0..10u16)
            .filter(|&x| w.food_at(Position::new(x, 0)) > 0.0)
            .count();
        assert_eq!(seeded, 4);
    }

    #[test]
    fn growth_is_density_proportional() {
        let mut w = WorldState::new(2, 1, WorldEdgeMode::Bounded);
        w.food_density.set(0, 0, 0.2);
        w.food_density.set(1, 0, 0.4);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.5;
        cfg.world.food.spread_threshold_ratio = 1.0;
        cfg.world.food.recovery_floor_ratio = 0.0;
        let mut rng = SmallRng::seed_from_u64(0);
        w.grow_food(&mut rng, &cfg);
        assert!((w.food_at(Position::new(0, 0)) - 0.3).abs() < 1e-6);
        assert!((w.food_at(Position::new(1, 0)) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn spread_activates_at_threshold() {
        let mut w = WorldState::new(2, 1, WorldEdgeMode::Bounded);
        w.food_density.set(0, 0, 0.75);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.2;
        cfg.world.food.spread_threshold_ratio = 0.75;
        cfg.world.food.spread_density_ratio = 1.0;
        cfg.world.food.recovery_floor_ratio = 0.0;
        let mut rng = SmallRng::seed_from_u64(1);
        w.grow_food(&mut rng, &cfg);
        assert!((w.food_at(Position::new(0, 0)) - 0.9).abs() < 1e-6);
        assert!((w.food_at(Position::new(1, 0)) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn spread_can_target_occupied_neighbors() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w = WorldState::new(2, 1, WorldEdgeMode::Bounded);
        let occupied = Position::new(1, 0);
        w.food_density.set(0, 0, 0.9);
        w.place_creature(occupied, id);

        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.2;
        cfg.world.food.spread_threshold_ratio = 0.75;
        cfg.world.food.spread_density_ratio = 1.0;
        cfg.world.food.recovery_floor_ratio = 0.0;
        let mut rng = SmallRng::seed_from_u64(3);
        w.grow_food(&mut rng, &cfg);

        assert!(w.food_at(occupied) > 0.0);
    }

    #[test]
    fn spread_amount_reduced_by_density_ratio() {
        // source=0.75, growth_rate=0.2, spread_density_ratio=0.25
        // delta = 0.75 * 0.2 = 0.15
        // spread deposit = 0.15 * 0.25 = 0.0375
        let mut w = WorldState::new(2, 1, WorldEdgeMode::Bounded);
        w.food_density.set(0, 0, 0.75);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.2;
        cfg.world.food.spread_threshold_ratio = 0.75;
        cfg.world.food.spread_density_ratio = 0.25;
        cfg.world.food.recovery_floor_ratio = 0.0;
        let mut rng = SmallRng::seed_from_u64(1);
        w.grow_food(&mut rng, &cfg);
        // Source gets local growth: 0.75 + 0.15 = 0.9
        assert!((w.food_at(Position::new(0, 0)) - 0.9).abs() < 1e-6);
        // Neighbor gets spread deposit: 0.15 * 0.25 = 0.0375
        assert!((w.food_at(Position::new(1, 0)) - 0.0375).abs() < 1e-6);
    }

    #[test]
    fn spread_density_ratio_zero_deposits_nothing() {
        let mut w = WorldState::new(2, 1, WorldEdgeMode::Bounded);
        w.food_density.set(0, 0, 0.9);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.2;
        cfg.world.food.spread_threshold_ratio = 0.75;
        cfg.world.food.spread_density_ratio = 0.0;
        cfg.world.food.recovery_floor_ratio = 0.0;
        let mut rng = SmallRng::seed_from_u64(1);
        w.grow_food(&mut rng, &cfg);
        // Source gets local growth
        assert!(w.food_at(Position::new(0, 0)) > 0.9);
        // Neighbor gets nothing because ratio is 0
        assert!((w.food_at(Position::new(1, 0)) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn recovery_spawn_runs_when_density_below_floor() {
        let mut w = WorldState::new(4, 1, WorldEdgeMode::Wrap);
        w.food_density.set(0, 0, 0.01);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.2;
        cfg.world.food.recovery_spawn_rate = 1.0;
        cfg.world.food.recovery_floor_ratio = 0.5;
        cfg.world.food.spread_threshold_ratio = 1.1;
        let before = w.total_food();
        let mut rng = SmallRng::seed_from_u64(2);
        w.grow_food(&mut rng, &cfg);
        assert!(w.total_food() > before);
    }

    #[test]
    fn recovery_spawn_can_fill_occupied_cells() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w = WorldState::new(1, 1, WorldEdgeMode::Wrap);
        let pos = Position::new(0, 0);
        w.place_creature(pos, id);

        let mut cfg = default_config();
        cfg.world.food.growth_rate = 0.5;
        cfg.world.food.recovery_spawn_rate = 1.0;
        cfg.world.food.recovery_floor_ratio = 1.0;
        cfg.world.food.spread_threshold_ratio = 1.1;
        let mut rng = SmallRng::seed_from_u64(4);
        w.grow_food(&mut rng, &cfg);

        assert!(w.food_at(pos) > 0.0);
    }

    #[test]
    fn resolve_neighbor_wrap_crosses_top_edge() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Wrap);
        let pos = Position::new(5, 0);
        let n = w.resolve_neighbor(pos, Direction::N).unwrap();
        assert_eq!(n, Position::new(5, 9));
    }

    #[test]
    fn resolve_neighbor_wrap_nw_corner() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Wrap);
        let n = w
            .resolve_neighbor(Position::new(0, 0), Direction::NW)
            .unwrap();
        assert_eq!(n, Position::new(9, 9));
    }

    #[test]
    fn resolve_neighbor_bounded_off_edge_none() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Bounded);
        assert!(w
            .resolve_neighbor(Position::new(0, 0), Direction::N)
            .is_none());
        assert!(w
            .resolve_neighbor(Position::new(0, 0), Direction::W)
            .is_none());
        assert!(w
            .resolve_neighbor(Position::new(9, 9), Direction::SE)
            .is_none());
    }

    #[test]
    fn is_valid_target_cell_open_cell() {
        let w = small_wrap_world();
        assert!(w.is_valid_target_cell(Position::new(3, 3)));
    }

    #[test]
    fn is_valid_target_cell_barrier_invalid() {
        let mut w = small_wrap_world();
        let pos = Position::new(4, 4);
        w.set_barrier(pos, true);
        assert!(!w.is_valid_target_cell(pos));
    }

    #[test]
    fn is_valid_target_cell_occupied_invalid() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w = small_wrap_world();
        let pos = Position::new(2, 2);
        w.place_creature(pos, id);
        assert!(!w.is_valid_target_cell(pos));
    }

    #[test]
    fn remove_creature_clears_occupancy() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w = small_wrap_world();
        let pos = Position::new(1, 1);
        w.place_creature(pos, id);
        assert_eq!(w.creature_at(pos), Some(id));
        w.remove_creature(pos);
        assert_eq!(w.creature_at(pos), None);
    }

    #[test]
    fn seed_food_deterministic() {
        let cfg = default_config();
        let mut w1 = small_wrap_world();
        let mut w2 = small_wrap_world();
        w1.seed_food(&mut SmallRng::seed_from_u64(99), &cfg);
        w2.seed_food(&mut SmallRng::seed_from_u64(99), &cfg);
        assert_eq!(w1.total_food(), w2.total_food());
        for y in 0..10u16 {
            for x in 0..10u16 {
                let pos = Position::new(x, y);
                assert_eq!(w1.food_at(pos), w2.food_at(pos));
            }
        }
    }
}
