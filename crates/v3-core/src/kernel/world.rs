use rand::Rng;

use crate::config::{FoodConfig, FoodResourceConfig, OrdinaryFoodTypeId, WorldEdgeMode};
use crate::contracts::{CreatureId, Direction, Position};
use crate::kernel::food_resource::{FoodGrowthSummary, FoodResource};
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
    food: FoodResource,
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
            // Config is replaced by reconfigure_food() before seeding.
            food: FoodResource::new(
                width,
                height,
                FoodConfig::single_type(FoodResourceConfig::default()),
                edge_mode,
            ),
            barriers: Grid::new(width, height, false),
            creature_at: Grid::new(width, height, None),
        }
    }

    // ── Food (delegates to FoodResource) ──────────────────────────────────────

    /// Read-only access to the `FoodResource`.
    pub fn food(&self) -> &FoodResource {
        &self.food
    }

    /// Seed the fertility map from the configured layers and world seed.
    ///
    /// Must be called before `seed_food` during world startup so that the
    /// fertility grid is ready before food growth begins.
    pub fn seed_fertility(&mut self, world_seed: u64) {
        self.food.seed_fertility(world_seed);
    }

    /// Seed initial food distribution (transitional delegate).
    pub fn seed_food(&mut self, rng: &mut impl Rng) {
        self.food.seed_density(&self.barriers, rng);
    }

    /// Grow food phase (transitional delegate).
    ///
    /// Reads barriers internally so callers do not need to pass them.
    pub fn grow_food(&mut self, tick: u64, rng: &mut impl Rng) -> FoodGrowthSummary {
        self.food.grow(&self.barriers, &self.creature_at, tick, rng)
    }

    /// Consume all food on a cell. Returns the amount consumed (0 if empty).
    /// Transitional delegate to `FoodResource::consume`.
    pub fn consume_food(&mut self, pos: Position) -> f32 {
        self.food.consume(pos)
    }

    /// Consume food only if the requested type owns the cell.
    pub fn consume_food_type(&mut self, pos: Position, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.food.consume_type(pos, type_idx)
    }

    /// Get food density at a position.
    /// Transitional delegate to `FoodResource::food_at`.
    pub fn food_at(&self, pos: Position) -> f32 {
        self.food.food_at(pos)
    }

    /// Get typed food density at a position, returning 0.0 for invalid types or non-owner cells.
    pub fn food_at_type(&self, pos: Position, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.food.food_at_type(pos, type_idx)
    }

    /// Get the dominant food type (highest per-type density), if any, for a cell.
    ///
    /// Compatibility helper for transitional single-type consumers. Cells may
    /// contain multiple food types simultaneously.
    pub fn dominant_food_type_at(&self, pos: Position) -> Option<OrdinaryFoodTypeId> {
        self.food.dominant_food_type_at(pos)
    }

    /// Set food density at a position directly.
    /// Transitional delegate to `FoodResource::set_food`.
    pub fn set_food(&mut self, pos: Position, value: f32) {
        self.food.set_food(pos, value);
    }

    /// Set typed food density directly for tests and tooling.
    pub fn set_food_type(&mut self, pos: Position, type_idx: OrdinaryFoodTypeId, value: f32) {
        self.food.set_food_type(pos, type_idx, value);
    }

    /// Replace the active food config through the food-kernel transition boundary.
    pub fn reconfigure_food(&mut self, config: FoodConfig) {
        self.food.apply_config_transition(config);
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

    /// Resolve an arbitrary signed local offset from an origin position.
    ///
    /// Per v3-world-grid-spec.md: the only geometry primitive that converts
    /// local signed offsets into world positions for perception consumers.
    pub fn resolve_offset(&self, origin: Position, dx: i32, dy: i32) -> Option<Position> {
        match self.edge_mode {
            WorldEdgeMode::Wrap => origin.neighbor_wrap(dx, dy, self.width, self.height),
            WorldEdgeMode::Bounded => origin.neighbor_bounded(dx, dy, self.width, self.height),
        }
    }

    // ── Diagnostics ──────────────────────────────────────────────────────────

    /// Total food across all cells (for testing and diagnostics).
    /// Transitional delegate to `FoodResource::total_food`.
    pub fn total_food(&self) -> f32 {
        self.food.total_food()
    }

    /// Total food owned by a specific type.
    pub fn total_food_by_type(&self, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.food.total_food_by_type(type_idx)
    }

    /// Occupancy depletion at a position (for testing and diagnostics).
    #[must_use]
    pub fn occupancy_depletion_at(&self, pos: Position) -> f32 {
        *self.food.occupancy_depletion().get(pos.x, pos.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FoodResourceConfig, FoodTypeConfig, SimulationConfig};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;

    fn default_config() -> SimulationConfig {
        SimulationConfig::default()
    }

    fn small_wrap_world() -> WorldState {
        WorldState::new(10, 10, WorldEdgeMode::Wrap)
    }

    /// Helper: create a WorldState with a custom food config applied.
    fn world_with_food_config(
        width: u16,
        height: u16,
        edge_mode: WorldEdgeMode,
        food_cfg: FoodResourceConfig,
    ) -> WorldState {
        let mut w = WorldState::new(width, height, edge_mode);
        w.reconfigure_food(FoodConfig::single_type(food_cfg));
        w
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
        w.reconfigure_food(default_config().world.food);
        let mut rng = SmallRng::seed_from_u64(42);
        w.seed_food(&mut rng);
        assert!(w.total_food() > 0.0);
    }

    #[test]
    fn seed_food_cells_have_density_or_zero() {
        let mut w = small_wrap_world();
        let cfg = default_config();
        w.reconfigure_food(cfg.world.food.clone());
        let mut rng = SmallRng::seed_from_u64(42);
        w.seed_food(&mut rng);
        for y in 0..10u16 {
            for x in 0..10u16 {
                let f = w.food_at(Position::new(x, y));
                assert!(
                    f >= 0.0
                        && f <= cfg.world.food.shared.max_density
                            * cfg.world.food.types.len() as f32
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
        let mut food_cfg = default_config().world.food;
        food_cfg.initial_coverage = 1.0; // guarantee all non-barrier cells are seeded
        w.reconfigure_food(food_cfg);
        w.seed_food(&mut rng);
        assert!((w.food_at(barrier_pos) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn grow_food_increases_total_with_rate_one() {
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.5,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(4, 4, WorldEdgeMode::Wrap, food_cfg);
        for y in 0..4u16 {
            for x in 0..4u16 {
                w.set_food(Position::new(x, y), 0.5);
            }
        }
        let before = w.total_food();
        let mut rng = SmallRng::seed_from_u64(42);
        let _ = w.grow_food(0, &mut rng);
        assert!(w.total_food() > before);
    }

    #[test]
    fn grow_food_clamps_at_max_density() {
        let food_cfg = FoodResourceConfig {
            growth_rate: 1.0,
            max_density: 1.0,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(1, 1, WorldEdgeMode::Wrap, food_cfg);
        w.set_food(Position::new(0, 0), 0.9);
        let mut rng = SmallRng::seed_from_u64(0);
        let _ = w.grow_food(0, &mut rng);
        assert!((w.food_at(Position::new(0, 0)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn grow_food_includes_occupied_cells_for_local_growth() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.5,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(1, 1, WorldEdgeMode::Wrap, food_cfg);
        let pos = Position::new(0, 0);
        w.set_food(pos, 0.8);
        w.place_creature(pos, id);

        let mut rng = SmallRng::seed_from_u64(0);
        let _ = w.grow_food(0, &mut rng);

        assert!((w.food_at(pos) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn consume_food_returns_amount_and_clears_cell() {
        let mut w = small_wrap_world();
        let pos = Position::new(3, 3);
        w.set_food(pos, 0.42);
        let consumed = w.consume_food(pos);
        assert!((consumed - 0.42).abs() < 1e-6);
        assert!((w.food_at(pos) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn seed_food_uses_exact_coverage_count() {
        let mut w = WorldState::new(10, 1, WorldEdgeMode::Wrap);
        let mut food_cfg = default_config().world.food;
        food_cfg.initial_coverage = 0.4;
        food_cfg.initial_density = 1.0;
        food_cfg.types[0].initial_coverage = 0.4;
        food_cfg.types[0].initial_density = 1.0;
        w.reconfigure_food(food_cfg);
        let mut rng = SmallRng::seed_from_u64(7);
        w.seed_food(&mut rng);
        let seeded = (0..10u16)
            .filter(|&x| w.food_at(Position::new(x, 0)) > 0.0)
            .count();
        assert_eq!(seeded, 4);
    }

    #[test]
    fn growth_is_density_proportional() {
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.5,
            spread_threshold_ratio: 1.0,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(2, 1, WorldEdgeMode::Bounded, food_cfg);
        w.set_food(Position::new(0, 0), 0.2);
        w.set_food(Position::new(1, 0), 0.4);
        let mut rng = SmallRng::seed_from_u64(0);
        let _ = w.grow_food(0, &mut rng);
        assert!((w.food_at(Position::new(0, 0)) - 0.3).abs() < 1e-6);
        assert!((w.food_at(Position::new(1, 0)) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn spread_activates_at_threshold() {
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.2,
            spread_threshold_ratio: 0.75,
            spread_density_ratio: 1.0,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(2, 1, WorldEdgeMode::Bounded, food_cfg);
        w.set_food(Position::new(0, 0), 0.75);
        let mut rng = SmallRng::seed_from_u64(1);
        let _ = w.grow_food(0, &mut rng);
        assert!((w.food_at(Position::new(0, 0)) - 0.9).abs() < 1e-6);
        assert!((w.food_at(Position::new(1, 0)) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn spread_can_target_occupied_neighbors() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.2,
            spread_threshold_ratio: 0.75,
            spread_density_ratio: 1.0,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(2, 1, WorldEdgeMode::Bounded, food_cfg);
        let occupied = Position::new(1, 0);
        w.set_food(Position::new(0, 0), 0.9);
        w.place_creature(occupied, id);

        let mut rng = SmallRng::seed_from_u64(3);
        let _ = w.grow_food(0, &mut rng);

        assert!(w.food_at(occupied) > 0.0);
    }

    #[test]
    fn spread_amount_reduced_by_density_ratio() {
        // source=0.75, growth_rate=0.2, spread_density_ratio=0.25
        // delta = 0.75 * 0.2 = 0.15
        // spread deposit = 0.15 * 0.25 = 0.0375
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.2,
            spread_threshold_ratio: 0.75,
            spread_density_ratio: 0.25,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(2, 1, WorldEdgeMode::Bounded, food_cfg);
        w.set_food(Position::new(0, 0), 0.75);
        let mut rng = SmallRng::seed_from_u64(1);
        let _ = w.grow_food(0, &mut rng);
        // Source gets local growth: 0.75 + 0.15 = 0.9
        assert!((w.food_at(Position::new(0, 0)) - 0.9).abs() < 1e-6);
        // Neighbor gets spread deposit: 0.15 * 0.25 = 0.0375
        assert!((w.food_at(Position::new(1, 0)) - 0.0375).abs() < 1e-6);
    }

    #[test]
    fn spread_density_ratio_zero_deposits_nothing() {
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.2,
            spread_threshold_ratio: 0.75,
            spread_density_ratio: 0.0,
            recovery_floor_ratio: 0.0,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(2, 1, WorldEdgeMode::Bounded, food_cfg);
        w.set_food(Position::new(0, 0), 0.9);
        let mut rng = SmallRng::seed_from_u64(1);
        let _ = w.grow_food(0, &mut rng);
        // Source gets local growth
        assert!(w.food_at(Position::new(0, 0)) > 0.9);
        // Neighbor gets nothing because ratio is 0
        assert!((w.food_at(Position::new(1, 0)) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn recovery_spawn_runs_when_density_below_floor() {
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.2,
            recovery_spawn_rate: 1.0,
            recovery_floor_ratio: 0.5,
            spread_threshold_ratio: 1.1,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(4, 1, WorldEdgeMode::Wrap, food_cfg);
        w.set_food(Position::new(0, 0), 0.01);
        let before = w.total_food();
        let mut rng = SmallRng::seed_from_u64(2);
        let _ = w.grow_food(0, &mut rng);
        assert!(w.total_food() > before);
    }

    #[test]
    fn recovery_spawn_can_fill_occupied_cells() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.5,
            recovery_spawn_rate: 1.0,
            recovery_floor_ratio: 1.0,
            spread_threshold_ratio: 1.1,
            ..FoodResourceConfig::default()
        };
        let mut w = world_with_food_config(1, 1, WorldEdgeMode::Wrap, food_cfg);
        let pos = Position::new(0, 0);
        w.place_creature(pos, id);

        let mut rng = SmallRng::seed_from_u64(4);
        let _ = w.grow_food(0, &mut rng);

        assert!(w.food_at(pos) > 0.0);
    }

    #[test]
    fn grow_food_reports_occupancy_depletion_summary() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w =
            world_with_food_config(2, 1, WorldEdgeMode::Bounded, FoodResourceConfig::default());
        let occupied = Position::new(0, 0);
        w.place_creature(occupied, id);

        let mut rng = SmallRng::seed_from_u64(9);
        let summary = w.grow_food(0, &mut rng);

        assert!(summary.mean_occupancy_depletion > 0.0);
        assert_eq!(summary.occupied_cells_with_depletion, 1);
        assert!(summary.growth_suppressed_by_occupancy_depletion >= 0.0);
        assert!(w.occupancy_depletion_at(occupied) > 0.0);
    }

    #[test]
    fn reconfigure_food_resets_depletion_when_enabled_toggles() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut config = FoodResourceConfig::default();
        config.occupancy_depletion.enabled = true;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.2;
        let mut w = world_with_food_config(1, 1, WorldEdgeMode::Wrap, config.clone());
        let pos = Position::new(0, 0);
        w.place_creature(pos, id);

        let mut rng = SmallRng::seed_from_u64(13);
        let _ = w.grow_food(0, &mut rng);
        assert!(w.occupancy_depletion_at(pos) > 0.0);

        config.occupancy_depletion.enabled = false;
        w.reconfigure_food(FoodConfig::single_type(config));

        assert_eq!(w.occupancy_depletion_at(pos), 0.0);
    }

    #[test]
    fn reconfigure_food_preserves_depletion_when_only_rate_changes() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut config = FoodResourceConfig::default();
        config.occupancy_depletion.enabled = true;
        config.occupancy_depletion.deposit_per_occupied_tick = 0.05;
        let mut w = world_with_food_config(1, 1, WorldEdgeMode::Wrap, config.clone());
        let pos = Position::new(0, 0);
        w.place_creature(pos, id);

        let mut rng = SmallRng::seed_from_u64(21);
        let _ = w.grow_food(0, &mut rng);
        let before = w.occupancy_depletion_at(pos);

        config.occupancy_depletion.deposit_per_occupied_tick = 0.25;
        w.reconfigure_food(FoodConfig::single_type(config));

        assert_eq!(w.occupancy_depletion_at(pos), before);
    }

    #[test]
    fn grow_food_keeps_barrier_cells_free_of_food_and_depletion() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let mut w =
            world_with_food_config(1, 1, WorldEdgeMode::Wrap, FoodResourceConfig::default());
        let pos = Position::new(0, 0);
        w.set_barrier(pos, true);
        w.set_food(pos, 0.5);
        w.place_creature(pos, id);

        let mut rng = SmallRng::seed_from_u64(34);
        let _ = w.grow_food(0, &mut rng);

        assert_eq!(w.food_at(pos), 0.0);
        assert_eq!(w.occupancy_depletion_at(pos), 0.0);
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
    fn resolve_offset_wrap_arbitrary() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Wrap);
        let origin = Position::new(2, 3);
        // Normal in-bounds
        let p = w.resolve_offset(origin, 3, 4).unwrap();
        assert_eq!(p, Position::new(5, 7));
        // Wrapping
        let p = w.resolve_offset(origin, -5, -5).unwrap();
        assert_eq!(p, Position::new(7, 8));
    }

    #[test]
    fn resolve_offset_bounded_out_of_bounds() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Bounded);
        assert!(w.resolve_offset(Position::new(2, 3), -5, 0).is_none());
        assert!(w.resolve_offset(Position::new(2, 3), 0, -5).is_none());
        // In-bounds
        assert_eq!(
            w.resolve_offset(Position::new(5, 5), -3, -3),
            Some(Position::new(2, 2))
        );
    }

    #[test]
    fn resolve_offset_self_returns_self() {
        let w = WorldState::new(10, 10, WorldEdgeMode::Wrap);
        let origin = Position::new(4, 6);
        assert_eq!(w.resolve_offset(origin, 0, 0), Some(origin));
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
        let food_cfg = default_config().world.food;
        let mut w1 = small_wrap_world();
        let mut w2 = small_wrap_world();
        w1.reconfigure_food(food_cfg.clone());
        w2.reconfigure_food(food_cfg);
        w1.seed_food(&mut SmallRng::seed_from_u64(99));
        w2.seed_food(&mut SmallRng::seed_from_u64(99));
        assert_eq!(w1.total_food(), w2.total_food());
        for y in 0..10u16 {
            for x in 0..10u16 {
                let pos = Position::new(x, y);
                assert_eq!(w1.food_at(pos), w2.food_at(pos));
            }
        }
    }

    #[test]
    fn reconfigure_food_updates_config() {
        let mut w = small_wrap_world();
        let food_cfg = FoodResourceConfig {
            growth_rate: 0.99,
            ..FoodResourceConfig::default()
        };
        w.reconfigure_food(FoodConfig::single_type(food_cfg));
        assert!((w.food().config().growth_rate - 0.99).abs() < f32::EPSILON);
    }

    #[test]
    fn reconfigure_food_resets_density_when_type_order_changes_with_same_count() {
        let mut w = WorldState::new(2, 1, WorldEdgeMode::Wrap);
        let mut initial = default_config().world.food;
        initial.types = vec![
            FoodTypeConfig {
                name: "Type A".to_string(),
                color: "#22c55e".to_string(),
                initial_density: 0.5,
                initial_coverage: 0.5,
                growth_inhibitor: 0.2,
            },
            FoodTypeConfig {
                name: "Type B".to_string(),
                color: "#0ea5e9".to_string(),
                initial_density: 0.5,
                initial_coverage: 0.5,
                growth_inhibitor: 0.2,
            },
        ];
        w.reconfigure_food(initial.clone());

        let pos = Position::new(0, 0);
        w.set_food_type(pos, OrdinaryFoodTypeId::new(0), 0.7);
        w.set_food_type(pos, OrdinaryFoodTypeId::new(1), 0.4);
        assert!(w.food_at(pos) > 0.0);

        let mut reordered = initial;
        reordered.types.swap(0, 1);
        w.reconfigure_food(reordered);

        assert_eq!(w.food_at(pos), 0.0);
        assert_eq!(w.food_at_type(pos, OrdinaryFoodTypeId::new(0)), 0.0);
        assert_eq!(w.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 0.0);
    }

    #[test]
    fn reconfigure_food_clamps_existing_density_to_lower_max_density() {
        let mut w = small_wrap_world();
        let pos = Position::new(0, 0);
        w.set_food(pos, 0.9);

        let mut next = default_config().world.food.shared;
        next.max_density = 0.3;
        w.reconfigure_food(FoodConfig::single_type(next));

        assert_eq!(w.food_at(pos), 0.3);
    }
}
