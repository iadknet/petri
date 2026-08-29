use rand::Rng;

use crate::config::{FoodResourceConfig, WorldEdgeMode};
use crate::contracts::{Direction, Position};
use crate::kernel::fertility;
use crate::kernel::Grid;

use super::depletion::OccupancyDepletionLayer;
use super::resolve_neighbor_static;

pub(super) struct GrowthContext<'a> {
    pub(super) density: &'a mut Grid<f32>,
    pub(super) fertility: &'a Grid<f32>,
    pub(super) occupancy_depletion: &'a OccupancyDepletionLayer,
    pub(super) config: &'a FoodResourceConfig,
    pub(super) barriers: &'a Grid<bool>,
    pub(super) edge_mode: WorldEdgeMode,
    pub(super) tick: u64,
    pub(super) growth_scratch: &'a mut Vec<f32>,
}

pub(super) fn apply_growth(context: GrowthContext<'_>, rng: &mut impl Rng) -> f32 {
    let width = context.density.width();
    let height = context.density.height();
    let total_cells = width as usize * height as usize;
    if total_cells == 0 {
        return 0.0;
    }

    let growth_rate = context.config.growth_rate.max(0.0);
    let max_density = context.config.max_density.max(0.0);
    if max_density <= 0.0 {
        return 0.0;
    }

    let spread_threshold = max_density * context.config.spread_threshold_ratio.clamp(0.0, 1.0);
    let spread_density_ratio = context.config.spread_density_ratio.clamp(0.0, 1.0);
    let recovery_floor = context.config.recovery_floor_ratio.clamp(0.0, 1.0);
    let recovery_spawn_rate = context.config.recovery_spawn_rate.clamp(0.0, 1.0);
    let occupancy_enabled = context.config.occupancy_depletion.enabled;

    let fertility_enabled = context.config.fertility.enabled;
    let (eff_min, eff_max) = if fertility_enabled {
        fertility::effective_fertility_range(
            &context.config.annealing,
            context.config.fertility.min_fertility,
            context.config.fertility.max_fertility,
            context.tick,
        )
    } else {
        (1.0, 1.0)
    };

    context.growth_scratch.clear();
    context.growth_scratch.extend(
        context
            .density
            .as_slice()
            .iter()
            .map(|v| v.clamp(0.0, max_density)),
    );
    let total_food: f32 = context.growth_scratch.iter().sum();
    let average_density_ratio = (total_food / (total_cells as f32 * max_density)).clamp(0.0, 1.0);

    let mut suppressed_total = 0.0;

    for idx in 0..total_cells {
        let source = context.growth_scratch[idx];
        let x = (idx % width as usize) as u16;
        let y = (idx / width as usize) as u16;
        let pos = Position::new(x, y);

        if *context.barriers.get(x, y) {
            continue;
        }

        let cell_fertility = if fertility_enabled {
            fertility::map_fertility(*context.fertility.get(x, y), eff_min, eff_max)
        } else {
            1.0
        };
        let cell_multiplier = context
            .occupancy_depletion
            .multiplier_at(x, y, occupancy_enabled);
        let local_unsuppressed = source * growth_rate * cell_fertility;
        let local_delta = local_unsuppressed * cell_multiplier;
        suppressed_total += add_food_clamped_with_suppression(
            context.density,
            pos,
            local_unsuppressed,
            local_delta,
            max_density,
        );

        if source < spread_threshold || local_unsuppressed <= 0.0 {
            continue;
        }

        let mut neighbors = [Position::new(0, 0); 4];
        let mut ncount = 0;
        for dir in [Direction::N, Direction::E, Direction::S, Direction::W] {
            let Some(npos) = resolve_neighbor_static(pos, dir, width, height, context.edge_mode)
            else {
                continue;
            };
            if *context.barriers.get(npos.x, npos.y) {
                continue;
            }
            neighbors[ncount] = npos;
            ncount += 1;
        }
        if ncount == 0 {
            continue;
        }

        let target = neighbors[rng.gen_range(0..ncount)];
        let neighbor_fertility = if fertility_enabled {
            fertility::map_fertility(*context.fertility.get(target.x, target.y), eff_min, eff_max)
        } else {
            1.0
        };
        let target_multiplier =
            context
                .occupancy_depletion
                .multiplier_at(target.x, target.y, occupancy_enabled);
        let spread_unsuppressed = local_unsuppressed * spread_density_ratio * neighbor_fertility;
        let spread_delta = spread_unsuppressed * target_multiplier;
        suppressed_total += add_food_clamped_with_suppression(
            context.density,
            target,
            spread_unsuppressed,
            spread_delta,
            max_density,
        );
    }

    if average_density_ratio >= recovery_floor {
        return suppressed_total;
    }

    let spawn_attempts = (total_cells as f32 * recovery_spawn_rate).round() as usize;
    let spawn_delta = max_density * growth_rate;
    if spawn_attempts == 0 || spawn_delta <= 0.0 {
        return suppressed_total;
    }

    for _ in 0..spawn_attempts {
        let idx = rng.gen_range(0..total_cells);
        let x = (idx % width as usize) as u16;
        let y = (idx / width as usize) as u16;
        let pos = Position::new(x, y);
        if *context.barriers.get(x, y) {
            continue;
        }

        let cell_fertility = if fertility_enabled {
            fertility::map_fertility(*context.fertility.get(x, y), eff_min, eff_max)
        } else {
            1.0
        };
        let cell_multiplier = context
            .occupancy_depletion
            .multiplier_at(x, y, occupancy_enabled);
        let recovery_unsuppressed = spawn_delta * cell_fertility;
        let recovery_delta = recovery_unsuppressed * cell_multiplier;
        suppressed_total += add_food_clamped_with_suppression(
            context.density,
            pos,
            recovery_unsuppressed,
            recovery_delta,
            max_density,
        );
    }

    suppressed_total
}

fn add_food_clamped_with_suppression(
    density: &mut Grid<f32>,
    pos: Position,
    unsuppressed_delta: f32,
    actual_delta: f32,
    max_density: f32,
) -> f32 {
    if unsuppressed_delta <= 0.0 && actual_delta <= 0.0 {
        return 0.0;
    }
    let current = *density.get(pos.x, pos.y);
    let applied_unsuppressed = clamped_added_amount(current, unsuppressed_delta, max_density);
    let applied_actual = clamped_added_amount(current, actual_delta, max_density);
    if applied_actual > 0.0 {
        density.set(pos.x, pos.y, current + applied_actual);
    }
    applied_unsuppressed - applied_actual
}

fn clamped_added_amount(current: f32, delta: f32, max_density: f32) -> f32 {
    if delta <= 0.0 || current >= max_density {
        return 0.0;
    }
    (current + delta).min(max_density) - current
}
