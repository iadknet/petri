use rand::Rng;

use crate::config::{
    FertilityConfig, FertilityLayerTarget, FoodConfig, FoodResourceConfig,
    OccupancyDepletionConfig, OrdinaryFoodTypeId, WorldEdgeMode,
};
use crate::contracts::{Direction, Position};
use crate::kernel::fertility;
use crate::kernel::Grid;

use super::catalog::OrdinaryFoodCatalog;
use super::state::OrdinaryFoodState;

const RECOVERY_PER_TICK: f32 = 0.03;
const MIN_GROWTH_MULTIPLIER: f32 = 0.35;

#[derive(Debug, Clone, PartialEq)]
pub struct FoodTypeTelemetry {
    pub type_idx: OrdinaryFoodTypeId,
    pub occupied_cells: u32,
    pub total_density: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FoodGrowthSummary {
    pub mean_occupancy_depletion: f32,
    pub occupied_cells_with_depletion: u32,
    pub growth_suppressed_by_occupancy_depletion: f32,
    pub cells_with_type_inhibition: u32,
    pub growth_suppressed_by_type_inhibition: f32,
    pub per_type: Vec<FoodTypeTelemetry>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct DepletionTickSummary {
    pub mean_depletion: f32,
    pub occupied_cells: u32,
}

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
                if *barriers.get(x, y)
                    || occupancy.get(x, y).is_none()
                    || deposit_per_occupied_tick <= 0.0
                {
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

        DepletionTickSummary {
            mean_depletion: if passable_cells == 0 {
                0.0
            } else {
                sum / passable_cells as f32
            },
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

#[must_use]
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

pub(super) fn seed_fertility_maps(
    state: &mut OrdinaryFoodState,
    catalog: &OrdinaryFoodCatalog,
    config: &FoodConfig,
    world_seed: u64,
) {
    for entry in catalog.entries() {
        let targeted_layers: Vec<_> = config
            .fertility
            .layers
            .iter()
            .filter(|layer| match &layer.target {
                FertilityLayerTarget::AllFoods => true,
                FertilityLayerTarget::SingleType { type_idx } => *type_idx == entry.id,
            })
            .cloned()
            .collect();

        let grid = if !config.fertility.enabled {
            Grid::new(state.width(), state.height(), 0.0)
        } else {
            fertility::generate_fertility(
                state.width(),
                state.height(),
                &FertilityConfig {
                    enabled: config.fertility.enabled,
                    min_fertility: config.fertility.min_fertility,
                    max_fertility: config.fertility.max_fertility,
                    layers: targeted_layers,
                },
                world_seed.wrapping_add(u64::from(entry.id.get())),
            )
        };

        state.set_fertility_grid(entry.id, grid);
    }
}

pub(super) fn apply_config_transition(
    current_shared: &mut FoodResourceConfig,
    next: &FoodConfig,
    state: &mut OrdinaryFoodState,
    occupancy_depletion: &mut OccupancyDepletionLayer,
    catalog_changed: bool,
) {
    let previous_max_density = current_shared.max_density.max(0.0);
    let reset_depletion =
        current_shared.occupancy_depletion.enabled != next.shared.occupancy_depletion.enabled;
    *current_shared = next.shared.clone();
    if reset_depletion || catalog_changed {
        occupancy_depletion.reset();
    }
    if catalog_changed {
        state.clear_density();
        state.resize_type_storage(next.types.len().max(1));
    } else if previous_max_density > current_shared.max_density.max(0.0) {
        state.clamp_density_to_max(current_shared.max_density);
    }
}

pub(super) fn seed_density(
    state: &mut OrdinaryFoodState,
    catalog: &OrdinaryFoodCatalog,
    shared: &FoodResourceConfig,
    occupancy_depletion: &mut OccupancyDepletionLayer,
    _claim_scratch: &mut Vec<Vec<(OrdinaryFoodTypeId, f32)>>,
    barriers: &Grid<bool>,
    rng: &mut impl Rng,
) {
    use rand::seq::SliceRandom;

    state.clear_density();
    occupancy_depletion.reset();

    let width = state.width();
    let height = state.height();
    let mut candidates = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if !*barriers.get(x, y) {
                candidates.push(Position::new(x, y));
            }
        }
    }
    if candidates.is_empty() {
        return;
    }

    for entry in catalog.entries() {
        let coverage = entry.config.initial_coverage.clamp(0.0, 1.0);
        let target = ((coverage * candidates.len() as f32).round() as usize).min(candidates.len());
        if target == 0 {
            continue;
        }
        let density = entry
            .config
            .initial_density
            .clamp(0.0, shared.max_density.max(0.0));
        let mut shuffled = candidates.clone();
        shuffled.shuffle(rng);
        for pos in shuffled.into_iter().take(target) {
            state.set_food_type_density(pos, entry.id, density);
        }
    }
}

fn build_type_inhibition_weighted_sums(
    state: &OrdinaryFoodState,
    catalog: &OrdinaryFoodCatalog,
    max_density: f32,
) -> Vec<f32> {
    let total_cells = state.width() as usize * state.height() as usize;
    let mut weighted_sums = vec![0.0; total_cells];

    for entry in catalog.entries() {
        let inhibitor = entry.config.growth_inhibitor.clamp(0.0, 1.0);
        if inhibitor <= 0.0 {
            continue;
        }
        for y in 0..state.height() {
            for x in 0..state.width() {
                let idx = y as usize * state.width() as usize + x as usize;
                let pos = Position::new(x, y);
                let density = state.food_at_type(pos, entry.id).clamp(0.0, max_density);
                weighted_sums[idx] += density * inhibitor;
            }
        }
    }

    weighted_sums
}

#[inline]
fn inhibition_penalty(weighted_sum: f32, own_density: f32, own_inhibitor: f32) -> f32 {
    (weighted_sum - own_density * own_inhibitor).max(0.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn grow<T: Clone>(
    state: &mut OrdinaryFoodState,
    catalog: &OrdinaryFoodCatalog,
    shared: &FoodResourceConfig,
    full_config: &FoodConfig,
    occupancy_depletion: &mut OccupancyDepletionLayer,
    _claim_scratch: &mut Vec<Vec<(OrdinaryFoodTypeId, f32)>>,
    _owner_snapshot: &mut Vec<Option<OrdinaryFoodTypeId>>,
    _density_snapshot: &mut Vec<f32>,
    barriers: &Grid<bool>,
    occupancy: &Grid<Option<T>>,
    edge_mode: WorldEdgeMode,
    tick: u64,
    rng: &mut impl Rng,
) -> FoodGrowthSummary {
    let total_cells = state.width() as usize * state.height() as usize;
    if total_cells == 0 {
        return FoodGrowthSummary::default();
    }

    let depletion_summary =
        occupancy_depletion.recover_and_deposit(barriers, occupancy, &shared.occupancy_depletion);

    let max_density = shared.max_density.max(0.0);
    if max_density <= 0.0 || catalog.is_empty() {
        return FoodGrowthSummary::default();
    }

    let growth_rate = shared.growth_rate.max(0.0);
    let spread_threshold = max_density * shared.spread_threshold_ratio.clamp(0.0, 1.0);
    let spread_density_ratio = shared.spread_density_ratio.clamp(0.0, 1.0);
    let recovery_floor = shared.recovery_floor_ratio.clamp(0.0, 1.0);
    let recovery_spawn_rate = shared.recovery_spawn_rate.clamp(0.0, 1.0);
    let occupancy_enabled = shared.occupancy_depletion.enabled;

    let (eff_min, eff_max) = if full_config.fertility.enabled {
        fertility::effective_fertility_range(
            &full_config.annealing,
            full_config.fertility.min_fertility,
            full_config.fertility.max_fertility,
            tick,
        )
    } else {
        (1.0, 1.0)
    };

    let inhibition_weighted_sums = build_type_inhibition_weighted_sums(state, catalog, max_density);
    let mut inhibited_cell_flags = vec![false; total_cells];
    let mut suppressed_by_occupancy_total = 0.0;
    let mut suppressed_by_inhibition_total = 0.0;
    let mut per_type: Vec<FoodTypeTelemetry> = catalog
        .entries()
        .iter()
        .map(|entry| FoodTypeTelemetry {
            type_idx: entry.id,
            occupied_cells: 0,
            total_density: 0.0,
        })
        .collect();

    for entry in catalog.entries() {
        let type_idx = entry.id;
        let type_inhibitor = entry.config.growth_inhibitor.clamp(0.0, 1.0);
        let type_density_ratio = {
            let type_total = state.total_food_by_type(type_idx);
            let type_capacity = total_cells as f32 * max_density;
            if type_capacity <= 0.0 {
                0.0
            } else {
                (type_total / type_capacity).clamp(0.0, 1.0)
            }
        };
        let mut next_density = vec![0.0; total_cells];

        for y in 0..state.height() {
            for x in 0..state.width() {
                let pos = Position::new(x, y);
                let idx = y as usize * state.width() as usize + x as usize;
                if *barriers.get(x, y) {
                    state.set_food_type_density(pos, type_idx, 0.0);
                    continue;
                }

                let source = state.food_at_type(pos, type_idx).clamp(0.0, max_density);
                if source <= 0.0 {
                    continue;
                }

                let base_cell_fertility = if full_config.fertility.enabled {
                    state
                        .fertility_grid(type_idx)
                        .map(|grid| fertility::map_fertility(*grid.get(x, y), eff_min, eff_max))
                        .unwrap_or(1.0)
                } else {
                    1.0
                };
                let cell_inhibition_penalty =
                    inhibition_penalty(inhibition_weighted_sums[idx], source, type_inhibitor);
                let cell_fertility = (base_cell_fertility - cell_inhibition_penalty).max(0.0);
                let cell_multiplier = occupancy_depletion.multiplier_at(x, y, occupancy_enabled);
                let local_base = source * growth_rate * base_cell_fertility;
                let local_after_inhibition = source * growth_rate * cell_fertility;
                let local_delta = local_after_inhibition * cell_multiplier;
                if local_base > local_after_inhibition && cell_inhibition_penalty > 0.0 {
                    inhibited_cell_flags[idx] = true;
                }
                suppressed_by_inhibition_total += (local_base - local_after_inhibition).max(0.0);
                suppressed_by_occupancy_total += (local_after_inhibition - local_delta).max(0.0);
                next_density[idx] = (source + local_delta).clamp(0.0, max_density);

                if source < spread_threshold || local_after_inhibition <= 0.0 {
                    continue;
                }

                let mut neighbors = [Position::new(0, 0); 4];
                let mut count = 0;
                for dir in [Direction::N, Direction::E, Direction::S, Direction::W] {
                    let Some(target) =
                        resolve_neighbor_static(pos, dir, state.width(), state.height(), edge_mode)
                    else {
                        continue;
                    };
                    if *barriers.get(target.x, target.y) {
                        continue;
                    }
                    neighbors[count] = target;
                    count += 1;
                }
                if count == 0 {
                    continue;
                }

                let target = neighbors[rng.gen_range(0..count)];
                let target_multiplier =
                    occupancy_depletion.multiplier_at(target.x, target.y, occupancy_enabled);
                let target_idx = target.y as usize * state.width() as usize + target.x as usize;
                let base_neighbor_fertility = if full_config.fertility.enabled {
                    state
                        .fertility_grid(type_idx)
                        .map(|grid| {
                            fertility::map_fertility(
                                *grid.get(target.x, target.y),
                                eff_min,
                                eff_max,
                            )
                        })
                        .unwrap_or(1.0)
                } else {
                    1.0
                };
                let target_source = state.food_at_type(target, type_idx).clamp(0.0, max_density);
                let neighbor_penalty = inhibition_penalty(
                    inhibition_weighted_sums[target_idx],
                    target_source,
                    type_inhibitor,
                );
                let neighbor_fertility = (base_neighbor_fertility - neighbor_penalty).max(0.0);
                let spread_base = local_base * spread_density_ratio * base_neighbor_fertility;
                let spread_after_inhibition =
                    local_after_inhibition * spread_density_ratio * neighbor_fertility;
                let spread_delta = spread_after_inhibition * target_multiplier;
                if spread_base > spread_after_inhibition && neighbor_penalty > 0.0 {
                    inhibited_cell_flags[target_idx] = true;
                }
                suppressed_by_inhibition_total += (spread_base - spread_after_inhibition).max(0.0);
                suppressed_by_occupancy_total += (spread_after_inhibition - spread_delta).max(0.0);
                next_density[target_idx] =
                    (next_density[target_idx] + spread_delta).clamp(0.0, max_density);
            }
        }

        if type_density_ratio < recovery_floor && recovery_spawn_rate > 0.0 {
            let spawn_attempts = (total_cells as f32 * recovery_spawn_rate).round() as usize;
            let spawn_delta = max_density * growth_rate;

            for _ in 0..spawn_attempts {
                if spawn_delta <= 0.0 {
                    break;
                }
                let idx = rng.gen_range(0..total_cells);
                let x = (idx % state.width() as usize) as u16;
                let y = (idx / state.width() as usize) as u16;
                if *barriers.get(x, y) {
                    continue;
                }

                let base_cell_fertility = if full_config.fertility.enabled {
                    state
                        .fertility_grid(type_idx)
                        .map(|grid| fertility::map_fertility(*grid.get(x, y), eff_min, eff_max))
                        .unwrap_or(1.0)
                } else {
                    1.0
                };
                let recovery_source = state
                    .food_at_type(Position::new(x, y), type_idx)
                    .clamp(0.0, max_density);
                let recovery_inhibition_penalty = inhibition_penalty(
                    inhibition_weighted_sums[idx],
                    recovery_source,
                    type_inhibitor,
                );
                let cell_fertility = (base_cell_fertility - recovery_inhibition_penalty).max(0.0);
                let cell_multiplier = occupancy_depletion.multiplier_at(x, y, occupancy_enabled);
                let recovery_base = spawn_delta * base_cell_fertility;
                let recovery_after_inhibition = spawn_delta * cell_fertility;
                let recovery_delta = recovery_after_inhibition * cell_multiplier;
                if recovery_base > recovery_after_inhibition && recovery_inhibition_penalty > 0.0 {
                    inhibited_cell_flags[idx] = true;
                }
                suppressed_by_inhibition_total +=
                    (recovery_base - recovery_after_inhibition).max(0.0);
                suppressed_by_occupancy_total +=
                    (recovery_after_inhibition - recovery_delta).max(0.0);
                next_density[idx] = (next_density[idx] + recovery_delta).clamp(0.0, max_density);
            }
        }

        let mut telemetry = FoodTypeTelemetry {
            type_idx,
            occupied_cells: 0,
            total_density: 0.0,
        };
        for y in 0..state.height() {
            for x in 0..state.width() {
                let idx = y as usize * state.width() as usize + x as usize;
                let density = next_density[idx];
                state.set_food_type_density(Position::new(x, y), type_idx, density);
                if density > 0.0 {
                    telemetry.occupied_cells += 1;
                    telemetry.total_density += density;
                }
            }
        }
        per_type[usize::from(type_idx.get())] = telemetry;
    }

    let inhibited_cells = inhibited_cell_flags.iter().filter(|flag| **flag).count() as u32;

    FoodGrowthSummary {
        mean_occupancy_depletion: depletion_summary.mean_depletion,
        occupied_cells_with_depletion: depletion_summary.occupied_cells,
        growth_suppressed_by_occupancy_depletion: suppressed_by_occupancy_total,
        cells_with_type_inhibition: inhibited_cells,
        growth_suppressed_by_type_inhibition: suppressed_by_inhibition_total,
        per_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::{rngs::StdRng, SeedableRng};

    fn barrier_grid(width: u16, height: u16) -> Grid<bool> {
        Grid::new(width, height, false)
    }

    fn occupancy_grid<T: Clone>(width: u16, height: u16) -> Grid<Option<T>> {
        Grid::new(width, height, None)
    }

    #[test]
    fn seed_density_allows_multiple_types_on_one_cell() {
        let mut config = FoodConfig::default();
        config.types = vec![
            crate::config::FoodTypeConfig {
                initial_density: 0.25,
                initial_coverage: 1.0,
                ..crate::config::FoodTypeConfig::default()
            },
            crate::config::FoodTypeConfig {
                initial_density: 0.75,
                initial_coverage: 1.0,
                ..crate::config::FoodTypeConfig::default()
            },
        ];

        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let mut claim_scratch = Vec::new();
        let barriers = barrier_grid(1, 1);
        let mut rng = StdRng::seed_from_u64(7);

        seed_density(
            &mut state,
            &catalog,
            &config.shared,
            &mut occupancy_depletion,
            &mut claim_scratch,
            &barriers,
            &mut rng,
        );

        let pos = Position::new(0, 0);
        assert_eq!(state.food_at_type(pos, OrdinaryFoodTypeId::new(0)), 0.25);
        assert_eq!(state.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 0.75);
        assert_eq!(state.density_at(pos), 1.0);
    }

    #[test]
    fn grow_updates_each_type_from_its_own_source_density() {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 1.0;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.recovery_floor_ratio = 0.0;
        config.shared.max_density = 10.0;
        config.shared.occupancy_depletion.enabled = false;
        config.types = vec![
            crate::config::FoodTypeConfig {
                growth_inhibitor: 0.0,
                ..crate::config::FoodTypeConfig::default()
            },
            crate::config::FoodTypeConfig {
                growth_inhibitor: 0.0,
                ..crate::config::FoodTypeConfig::default()
            },
        ];

        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(0), 2.0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(1), 4.0);

        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let mut claim_scratch = Vec::new();
        let mut owner_snapshot = Vec::new();
        let mut density_snapshot = Vec::new();
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let mut rng = StdRng::seed_from_u64(11);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut claim_scratch,
            &mut owner_snapshot,
            &mut density_snapshot,
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            &mut rng,
        );

        assert_eq!(state.food_at_type(pos, OrdinaryFoodTypeId::new(0)), 3.0);
        assert_eq!(state.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 6.0);
        assert_eq!(summary.per_type.len(), 2);
        assert_eq!(summary.per_type[0].occupied_cells, 1);
        assert_eq!(summary.per_type[1].occupied_cells, 1);
    }

    #[test]
    fn grow_applies_cross_type_growth_inhibition() {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.growth_rate = 1.0;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 1.0;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.recovery_floor_ratio = 0.0;
        config.shared.max_density = 10.0;
        config.shared.occupancy_depletion.enabled = false;
        config.types = vec![
            crate::config::FoodTypeConfig {
                growth_inhibitor: 0.0,
                ..crate::config::FoodTypeConfig::default()
            },
            crate::config::FoodTypeConfig {
                growth_inhibitor: 1.0,
                ..crate::config::FoodTypeConfig::default()
            },
        ];

        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(0), 1.0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(1), 1.0);

        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let mut claim_scratch = Vec::new();
        let mut owner_snapshot = Vec::new();
        let mut density_snapshot = Vec::new();
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let mut rng = StdRng::seed_from_u64(23);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut claim_scratch,
            &mut owner_snapshot,
            &mut density_snapshot,
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            &mut rng,
        );

        assert_eq!(state.food_at_type(pos, OrdinaryFoodTypeId::new(0)), 1.0);
        assert_eq!(state.food_at_type(pos, OrdinaryFoodTypeId::new(1)), 2.0);
        assert_eq!(summary.cells_with_type_inhibition, 1);
        assert!(summary.growth_suppressed_by_type_inhibition > 0.0);
    }

    #[test]
    fn grow_does_not_count_self_inhibition_as_type_inhibition() {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.growth_rate = 1.0;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 1.0;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.recovery_floor_ratio = 0.0;
        config.shared.max_density = 10.0;
        config.shared.occupancy_depletion.enabled = false;
        config.types = vec![crate::config::FoodTypeConfig {
            growth_inhibitor: 1.0,
            ..crate::config::FoodTypeConfig::default()
        }];

        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(0), 1.0);

        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let mut claim_scratch = Vec::new();
        let mut owner_snapshot = Vec::new();
        let mut density_snapshot = Vec::new();
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let mut rng = StdRng::seed_from_u64(37);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut claim_scratch,
            &mut owner_snapshot,
            &mut density_snapshot,
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            &mut rng,
        );

        assert_eq!(summary.cells_with_type_inhibition, 0);
        assert_eq!(summary.growth_suppressed_by_type_inhibition, 0.0);
    }

    #[test]
    fn recovery_is_evaluated_per_type_not_global_density() {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 1.0;
        config.shared.recovery_spawn_rate = 1.0;
        config.shared.recovery_floor_ratio = 0.5;
        config.shared.max_density = 1.0;
        config.shared.occupancy_depletion.enabled = false;
        config.types = vec![
            crate::config::FoodTypeConfig {
                growth_inhibitor: 0.0,
                ..crate::config::FoodTypeConfig::default()
            },
            crate::config::FoodTypeConfig {
                growth_inhibitor: 0.0,
                ..crate::config::FoodTypeConfig::default()
            },
        ];

        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(0), 0.0);
        state.set_food_type_density(pos, OrdinaryFoodTypeId::new(1), 1.0);

        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let mut claim_scratch = Vec::new();
        let mut owner_snapshot = Vec::new();
        let mut density_snapshot = Vec::new();
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let mut rng = StdRng::seed_from_u64(31);

        grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut claim_scratch,
            &mut owner_snapshot,
            &mut density_snapshot,
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            &mut rng,
        );

        assert!(state.food_at_type(pos, OrdinaryFoodTypeId::new(0)) > 0.0);
    }
}
