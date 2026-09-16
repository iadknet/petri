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
    /// Mean grazing modifier over passable cells after this pass's recovery
    /// step; 1.0 while grazing is disabled.
    pub mean_grazing_modifier: f32,
    /// Passable cells whose grazing modifier is below 1.0 after recovery.
    pub grazed_cells: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FoodGrowthSummary {
    pub mean_occupancy_depletion: f32,
    pub occupied_cells_with_depletion: u32,
    pub growth_suppressed_by_occupancy_depletion: f32,
    pub cells_with_type_inhibition: u32,
    pub growth_suppressed_by_type_inhibition: f32,
    /// Non-barrier cells: the denominator of each type's grazed-cell share.
    pub passable_cells: u32,
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
    let reset_grazing = current_shared.grazing.enabled != next.shared.grazing.enabled;
    *current_shared = next.shared.clone();
    if reset_depletion || catalog_changed {
        occupancy_depletion.reset();
    }
    if reset_grazing {
        state.grazing_mut().reset();
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
    config: &FoodConfig,
    occupancy_depletion: &mut OccupancyDepletionLayer,
    _claim_scratch: &mut Vec<Vec<(OrdinaryFoodTypeId, f32)>>,
    barriers: &Grid<bool>,
    rng: &mut impl Rng,
) {
    use rand::seq::SliceRandom;
    let shared = &config.shared;

    state.clear_density();
    occupancy_depletion.reset();
    state.grazing_mut().reset();

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
        // Shuffle independently for each food type. Reusing one candidate
        // ordering would silently co-seed every type and erase the spatial
        // choice pressure that typed-food cognition is meant to solve.
        let mut shuffled_candidates = candidates.clone();
        if entry.config.initial_fertility_only && config.fertility.enabled {
            let (min, max) = fertility::effective_fertility_range(
                &config.annealing,
                config.fertility.min_fertility,
                config.fertility.max_fertility,
                0,
            );
            shuffled_candidates.retain(|pos| {
                state.fertility_grid(entry.id).is_some_and(|grid| {
                    fertility::map_fertility(*grid.get(pos.x, pos.y), min, max) > 0.0
                })
            });
        }
        shuffled_candidates.shuffle(rng);
        let coverage = entry.config.initial_coverage.clamp(0.0, 1.0);
        let target = ((coverage * shuffled_candidates.len() as f32).round() as usize)
            .min(shuffled_candidates.len());
        if target == 0 {
            continue;
        }
        let density = entry
            .config
            .initial_density
            .clamp(0.0, shared.max_density.max(0.0));
        for pos in shuffled_candidates.iter().copied().take(target) {
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

/// The fertility multiplier growth applies at one cell: the annealed habitat
/// fertility (1.0 when fertility is disabled or the type has no grid) times
/// the cell's grazing modifier.
#[inline]
fn grazed_habitat_fertility(
    state: &OrdinaryFoodState,
    full_config: &FoodConfig,
    type_idx: OrdinaryFoodTypeId,
    x: u16,
    y: u16,
    (eff_min, eff_max): (f32, f32),
) -> f32 {
    let habitat = if full_config.fertility.enabled {
        state
            .fertility_grid(type_idx)
            .map(|grid| fertility::map_fertility(*grid.get(x, y), eff_min, eff_max))
            .unwrap_or(1.0)
    } else {
        1.0
    };
    habitat
        * state
            .grazing()
            .multiplier_at(x, y, type_idx, full_config.shared.grazing.enabled)
}

#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::too_many_lines,
    reason = "one food growth pass over the grid; the per-cell claim, spread, \
              recovery, and inhibition stages share scratch buffers that would \
              have to be threaded through helpers"
)]
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
    // Grazing heals before any fertility read, on every passable cell: the
    // grazed cell is empty by construction and the growth loop below skips
    // empty cells.
    let (grazing_summaries, passable_cells) =
        state.grazing_mut().recover(barriers, &shared.grazing);

    let max_density = shared.max_density.max(0.0);
    if max_density <= 0.0 || catalog.is_empty() {
        return FoodGrowthSummary::default();
    }

    let spread_threshold = max_density * shared.spread_threshold_ratio.clamp(0.0, 1.0);
    let spread_density_ratio = shared.spread_density_ratio.clamp(0.0, 1.0);
    let recovery_floor = shared.recovery_floor_ratio.clamp(0.0, 1.0);
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
            mean_grazing_modifier: 1.0,
            grazed_cells: 0,
        })
        .collect();

    for entry in catalog.entries() {
        let type_idx = entry.id;
        let growth_rate = entry
            .config
            .growth_rate
            .unwrap_or(shared.growth_rate)
            .max(0.0);
        let recovery_spawn_rate = entry
            .config
            .recovery_spawn_rate
            .unwrap_or(shared.recovery_spawn_rate)
            .clamp(0.0, 1.0);
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

                let base_cell_fertility = grazed_habitat_fertility(
                    state,
                    full_config,
                    type_idx,
                    x,
                    y,
                    (eff_min, eff_max),
                );
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
                let base_neighbor_fertility = grazed_habitat_fertility(
                    state,
                    full_config,
                    type_idx,
                    target.x,
                    target.y,
                    (eff_min, eff_max),
                );
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

                let base_cell_fertility = grazed_habitat_fertility(
                    state,
                    full_config,
                    type_idx,
                    x,
                    y,
                    (eff_min, eff_max),
                );
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

        let grazing_summary = grazing_summaries
            .get(usize::from(type_idx.get()))
            .copied()
            .unwrap_or_default();
        let mut telemetry = FoodTypeTelemetry {
            type_idx,
            occupied_cells: 0,
            total_density: 0.0,
            mean_grazing_modifier: grazing_summary.mean_modifier,
            grazed_cells: grazing_summary.grazed_cells,
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
        passable_cells,
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
    fn legacy_placement_keeps_candidate_order_and_rng_even_for_zero_coverage() {
        use rand::{seq::SliceRandom, RngCore};
        for coverage in [0.0, 0.54, 1.0] {
            let mut config = FoodConfig::default();
            config.types[0].initial_coverage = coverage;
            config.types.push(crate::config::FoodTypeConfig {
                initial_coverage: coverage,
                ..Default::default()
            });
            let catalog = OrdinaryFoodCatalog::new(&config);
            let mut state = OrdinaryFoodState::new(7, 6, catalog.len());
            let mut barriers = barrier_grid(7, 6);
            barriers.set(2, 3, true);
            let mut actual_rng = StdRng::seed_from_u64(31);
            let mut expected_rng = actual_rng.clone();
            seed_density(
                &mut state,
                &catalog,
                &config,
                &mut OccupancyDepletionLayer::new(7, 6),
                &mut Vec::new(),
                &barriers,
                &mut actual_rng,
            );
            let candidates: Vec<_> = (0..6)
                .flat_map(|y| (0..7).map(move |x| Position::new(x, y)))
                .filter(|p| !*barriers.get(p.x, p.y))
                .collect();
            for entry in catalog.entries() {
                let mut shuffled = candidates.clone();
                shuffled.shuffle(&mut expected_rng);
                let target = (coverage * candidates.len() as f32).round() as usize;
                for pos in &candidates {
                    assert_eq!(
                        state.food_at_type(*pos, entry.id),
                        if shuffled[..target].contains(pos) {
                            1.0
                        } else {
                            0.0
                        }
                    );
                }
            }
            assert_eq!(actual_rng.next_u64(), expected_rng.next_u64());
        }
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
            &config,
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
    fn seed_density_shuffles_each_food_type_independently() {
        let mut config = FoodConfig::default();
        config.types.push(crate::config::FoodTypeConfig::default());
        config.types[0].initial_coverage = 0.5;
        config.types[1].initial_coverage = 0.5;

        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(8, 1, catalog.len());
        let mut occupancy_depletion = OccupancyDepletionLayer::new(8, 1);
        let mut claim_scratch = Vec::new();
        let barriers = barrier_grid(8, 1);
        let mut rng = StdRng::seed_from_u64(0xC0FFEE);

        seed_density(
            &mut state,
            &catalog,
            &config,
            &mut occupancy_depletion,
            &mut claim_scratch,
            &barriers,
            &mut rng,
        );

        let maintenance: std::collections::BTreeSet<_> = (0..8)
            .filter(|&x| state.food_at_type(Position::new(x, 0), OrdinaryFoodTypeId::new(0)) > 0.0)
            .collect();
        let reproductive: std::collections::BTreeSet<_> = (0..8)
            .filter(|&x| state.food_at_type(Position::new(x, 0), OrdinaryFoodTypeId::new(1)) > 0.0)
            .collect();

        assert_eq!(maintenance.len(), 4);
        assert_eq!(reproductive.len(), 4);
        assert_ne!(maintenance, reproductive);
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

    /// One growth pass on a `Bounded` 2x1 world with a dense source at (0,0):
    /// the only passable neighbor is (1,0), so the spread target is fixed and
    /// the delta landing there is measurable.
    fn spread_target_delta_with_grazing(
        grazing: crate::config::GrazingConfig,
        bites_on_target: u32,
    ) -> (f32, FoodGrowthSummary) {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 1.0;
        config.shared.spread_threshold_ratio = 0.0;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.recovery_floor_ratio = 0.0;
        config.shared.max_density = 10.0;
        config.shared.occupancy_depletion.enabled = false;
        config.shared.grazing = grazing;
        config.types = vec![crate::config::FoodTypeConfig {
            growth_inhibitor: 0.0,
            ..crate::config::FoodTypeConfig::default()
        }];
        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(2, 1, catalog.len());
        let source = Position::new(0, 0);
        let target = Position::new(1, 0);
        let type_idx = OrdinaryFoodTypeId::new(0);
        state.set_food_type_density(source, type_idx, 1.0);
        for _ in 0..bites_on_target {
            state.set_food_type_density(target, type_idx, 1.0);
            let _ = state.consume_type(target, type_idx, &config.shared.grazing);
        }
        let mut occupancy_depletion = OccupancyDepletionLayer::new(2, 1);
        let barriers = barrier_grid(2, 1);
        let occupancy = occupancy_grid::<()>(2, 1);
        let mut rng = StdRng::seed_from_u64(5);
        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Bounded,
            0,
            &mut rng,
        );
        (state.food_at_type(target, type_idx), summary)
    }

    #[test]
    fn bitten_cell_recolonizes_at_factor_of_the_unbitten_rate() {
        // recovery_ticks large enough that the one recovery step is negligible
        // against the bite while the cell still reads as grazed.
        let grazing = crate::config::GrazingConfig {
            recovery_ticks: 1_000_000,
            ..crate::config::GrazingConfig::default()
        };
        let (unbitten, _) = spread_target_delta_with_grazing(grazing.clone(), 0);
        let (bitten, summary) = spread_target_delta_with_grazing(grazing, 1);
        assert!(unbitten > 0.0);
        assert!(
            (bitten / unbitten - 0.5).abs() < 1e-4,
            "{bitten} vs {unbitten}"
        );
        assert_eq!(summary.per_type[0].grazed_cells, 1);
        assert!(summary.per_type[0].mean_grazing_modifier < 1.0);
        assert_eq!(summary.passable_cells, 2);
    }

    #[test]
    fn floored_cell_recolonizes_at_floor() {
        let grazing = crate::config::GrazingConfig {
            recovery_ticks: 1_000_000,
            ..crate::config::GrazingConfig::default()
        };
        let (unbitten, _) = spread_target_delta_with_grazing(grazing.clone(), 0);
        let (floored, _) = spread_target_delta_with_grazing(grazing, 9);
        assert!(
            (floored / unbitten - 0.05).abs() < 1e-4,
            "{floored} vs {unbitten}"
        );
    }

    #[test]
    fn empty_grazed_cell_recovers_each_growth_pass() {
        let grazing = crate::config::GrazingConfig {
            recovery_ticks: 4,
            ..crate::config::GrazingConfig::default()
        };
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 1.0;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.occupancy_depletion.enabled = false;
        config.shared.grazing = grazing;
        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        let type_idx = OrdinaryFoodTypeId::new(0);
        state.set_food_type_density(pos, type_idx, 1.0);
        assert_eq!(
            state.consume_type(pos, type_idx, &config.shared.grazing),
            1.0
        );
        assert_eq!(state.grazing().modifier_at(0, 0, type_idx), 0.5);
        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let mut rng = StdRng::seed_from_u64(1);
        let mut readings = Vec::new();
        for tick in 0..3 {
            let summary = grow(
                &mut state,
                &catalog,
                &config.shared,
                &config,
                &mut occupancy_depletion,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &barriers,
                &occupancy,
                WorldEdgeMode::Wrap,
                tick,
                &mut rng,
            );
            readings.push((
                state.grazing().modifier_at(0, 0, type_idx),
                summary.per_type[0].mean_grazing_modifier,
                summary.per_type[0].grazed_cells,
                summary.passable_cells,
            ));
        }
        // The cell stayed empty (no growth source) yet recovered by 0.25 per pass.
        assert_eq!(state.food_at_type(pos, type_idx), 0.0);
        assert_eq!(
            readings,
            vec![(0.75, 0.75, 1, 1), (1.0, 1.0, 0, 1), (1.0, 1.0, 0, 1)]
        );
    }

    #[test]
    fn disabled_grazing_reads_one_and_re_enabling_starts_from_one() {
        let disabled = crate::config::GrazingConfig {
            enabled: false,
            ..crate::config::GrazingConfig::default()
        };
        let (unbitten, _) = spread_target_delta_with_grazing(disabled.clone(), 0);
        let (bitten, summary) = spread_target_delta_with_grazing(disabled, 3);
        assert_eq!(bitten, unbitten);
        assert_eq!(summary.per_type[0].mean_grazing_modifier, 1.0);
        assert_eq!(summary.per_type[0].grazed_cells, 0);

        let mut current = FoodConfig::default();
        let catalog = OrdinaryFoodCatalog::new(&current);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        let type_idx = OrdinaryFoodTypeId::new(0);
        state.set_food_type_density(pos, type_idx, 1.0);
        let _ = state.consume_type(pos, type_idx, &current.shared.grazing);
        assert_eq!(state.grazing().modifier_at(0, 0, type_idx), 0.5);
        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);

        let mut next = current.clone();
        next.shared.grazing.enabled = false;
        apply_config_transition(
            &mut current.shared,
            &next,
            &mut state,
            &mut occupancy_depletion,
            false,
        );
        assert_eq!(state.grazing().modifier_at(0, 0, type_idx), 1.0);

        state.set_food_type_density(pos, type_idx, 1.0);
        next.shared.grazing.enabled = true;
        apply_config_transition(
            &mut current.shared,
            &next,
            &mut state,
            &mut occupancy_depletion,
            false,
        );
        let _ = state.consume_type(pos, type_idx, &current.shared.grazing);
        assert_eq!(state.grazing().modifier_at(0, 0, type_idx), 0.5);

        // Live edits to factor, floor, and recovery_ticks leave the grid alone.
        next.shared.grazing.factor = 0.9;
        next.shared.grazing.floor = 0.7;
        next.shared.grazing.recovery_ticks = 7;
        apply_config_transition(
            &mut current.shared,
            &next,
            &mut state,
            &mut occupancy_depletion,
            false,
        );
        assert_eq!(state.grazing().modifier_at(0, 0, type_idx), 0.5);
        assert_eq!(current.shared.grazing.recovery_ticks, 7);
    }

    #[test]
    fn occupancy_depletion_multiplies_beside_grazing() {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 1.0;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.recovery_floor_ratio = 0.0;
        config.shared.max_density = 10.0;
        config.shared.occupancy_depletion.enabled = true;
        config.shared.occupancy_depletion.deposit_per_occupied_tick = 1.0;
        config.shared.grazing.recovery_ticks = 1_000_000;
        config.types = vec![crate::config::FoodTypeConfig {
            growth_inhibitor: 0.0,
            ..crate::config::FoodTypeConfig::default()
        }];
        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        let pos = Position::new(0, 0);
        let type_idx = OrdinaryFoodTypeId::new(0);
        state.set_food_type_density(pos, type_idx, 1.0);
        let _ = state.consume_type(pos, type_idx, &config.shared.grazing);
        state.set_food_type_density(pos, type_idx, 1.0);
        let mut occupancy_depletion = OccupancyDepletionLayer::new(1, 1);
        let barriers = barrier_grid(1, 1);
        let occupancy = Grid::new(1, 1, Some(()));
        let mut rng = StdRng::seed_from_u64(1);
        grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            &mut rng,
        );
        // local delta = source * rate * grazing (0.5) * occupancy multiplier
        // (MIN_GROWTH_MULTIPLIER at full depletion).
        let expected = 1.0 + 1.0 * 0.5 * 0.5 * MIN_GROWTH_MULTIPLIER;
        assert!((state.food_at_type(pos, type_idx) - expected).abs() < 1e-5);
    }
}
