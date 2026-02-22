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
    food_density: Grid<u8>,
    barriers: Grid<bool>,
    creature_at: Grid<Option<CreatureId>>,
}

impl WorldState {
    /// Create an empty world with no food, no barriers, no creatures.
    pub fn new(width: u16, height: u16, edge_mode: WorldEdgeMode) -> Self {
        Self {
            width,
            height,
            edge_mode,
            food_density: Grid::new(width, height, 0),
            barriers: Grid::new(width, height, false),
            creature_at: Grid::new(width, height, None),
        }
    }

    // ── Food ─────────────────────────────────────────────────────────────────

    /// Seed initial food distribution.
    /// Each non-barrier cell independently rolls against `initial_coverage`;
    /// on success, food is set to `initial_density`.
    pub fn seed_food(&mut self, rng: &mut impl Rng, config: &SimulationConfig) {
        let coverage = config.world.food.initial_coverage;
        let density = config.world.food.initial_density;
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.barriers.get(x, y) {
                    continue;
                }
                if rng.gen::<f32>() < coverage {
                    self.food_density.set(x, y, density);
                }
            }
        }
    }

    /// Grow food phase: each non-barrier cell independently rolls against `growth_rate`.
    /// On success, food increments by 1 (saturating at 255).
    pub fn grow_food(&mut self, rng: &mut impl Rng, config: &SimulationConfig) {
        let rate = config.world.food.growth_rate;
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.barriers.get(x, y) {
                    continue;
                }
                if rng.gen::<f32>() < rate {
                    let current = *self.food_density.get(x, y);
                    self.food_density.set(x, y, current.saturating_add(1));
                }
            }
        }
    }

    /// Consume all food on a cell. Returns the amount consumed (0 if empty).
    pub fn consume_food(&mut self, pos: Position) -> u8 {
        let amount = *self.food_density.get(pos.x, pos.y);
        self.food_density.set(pos.x, pos.y, 0);
        amount
    }

    /// Get food density at a position.
    pub fn food_at(&self, pos: Position) -> u8 {
        *self.food_density.get(pos.x, pos.y)
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
    pub fn total_food(&self) -> u64 {
        let mut sum = 0u64;
        for y in 0..self.height {
            for x in 0..self.width {
                sum += *self.food_density.get(x, y) as u64;
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
        assert_eq!(w.total_food(), 0);
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
        assert!(w.total_food() > 0);
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
                assert!(f == 0 || f == cfg.world.food.initial_density);
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
        assert_eq!(w.food_at(barrier_pos), 0);
    }

    #[test]
    fn grow_food_increases_total_with_rate_one() {
        let mut w = WorldState::new(4, 4, WorldEdgeMode::Wrap);
        for y in 0..4u16 {
            for x in 0..4u16 {
                w.food_density.set(x, y, 10);
            }
        }
        let before = w.total_food();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 1.0;
        w.grow_food(&mut rng, &cfg);
        assert!(w.total_food() > before);
    }

    #[test]
    fn grow_food_saturates_at_255() {
        let mut w = WorldState::new(1, 1, WorldEdgeMode::Wrap);
        w.food_density.set(0, 0, 255);
        let mut rng = SmallRng::seed_from_u64(0);
        let mut cfg = default_config();
        cfg.world.food.growth_rate = 1.0;
        w.grow_food(&mut rng, &cfg);
        assert_eq!(w.food_at(Position::new(0, 0)), 255);
    }

    #[test]
    fn consume_food_returns_amount_and_clears_cell() {
        let mut w = small_wrap_world();
        let pos = Position::new(3, 3);
        w.food_density.set(3, 3, 100);
        let consumed = w.consume_food(pos);
        assert_eq!(consumed, 100);
        assert_eq!(w.food_at(pos), 0);
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
