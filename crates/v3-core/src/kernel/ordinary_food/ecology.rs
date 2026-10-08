use rand::Rng;
use rayon::prelude::*;

use crate::config::{
    FertilityConfig, FertilityLayerTarget, FoodConfig, FoodResourceConfig,
    OccupancyDepletionConfig, OrdinaryFoodTypeId, WorldEdgeMode,
};
use crate::contracts::{Direction, Position};
use crate::kernel::fertility;
use crate::kernel::Grid;

use super::catalog::OrdinaryFoodCatalog;
use super::pass_clock::{FoodPass, PassClock};
use super::state::OrdinaryFoodState;
use super::stream::{DrawPurpose, FoodStream, FoodStreamKey};

const RECOVERY_PER_TICK: f32 = 0.03;
const MIN_GROWTH_MULTIPLIER: f32 = 0.35;
/// The candidate-list order of a spreading cell and the gather order of a
/// target's sources.
const SPREAD_DIRECTIONS: [Direction; 4] = [Direction::N, Direction::E, Direction::S, Direction::W];

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
    /// The grid passes this call ran, in execution order (T24.F03).
    #[cfg(feature = "telemetry-seams")]
    pub passes: super::pass_clock::FoodPasses,
}

impl FoodGrowthSummary {
    /// Moves the pass timings out for the tick-trace seam.
    #[cfg(feature = "telemetry-seams")]
    pub fn take_passes(&mut self) -> super::pass_clock::FoodPasses {
        std::mem::take(&mut self.passes)
    }
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
        self.grid.as_mut_slice().fill(0.0);
    }

    #[must_use]
    pub(super) fn grid(&self) -> &Grid<f32> {
        &self.grid
    }

    /// Decay, deposit and the mean in one row-parallel pass: each cell's
    /// value depends only on itself, and the per-row sums are reduced in row
    /// order, so the result does not depend on the pool.
    pub(super) fn recover_and_deposit<T: Clone + Sync>(
        &mut self,
        barriers: &Grid<bool>,
        occupancy: &Grid<Option<T>>,
        config: &OccupancyDepletionConfig,
    ) -> DepletionTickSummary {
        if !config.enabled {
            return DepletionTickSummary::default();
        }

        let width = usize::from(self.grid.width());
        let deposit_per_occupied_tick = config.deposit_per_occupied_tick.clamp(0.0, 1.0);
        let rows: Vec<(f32, u32, u32)> = self
            .grid
            .as_mut_slice()
            .par_chunks_mut(width)
            .zip(barriers.as_slice().par_chunks(width))
            .zip(occupancy.as_slice().par_chunks(width))
            .map(|((cells, barrier_row), occupancy_row)| {
                let (mut sum, mut passable, mut occupied) = (0.0f32, 0u32, 0u32);
                for ((cell, &barrier), occupant) in
                    cells.iter_mut().zip(barrier_row).zip(occupancy_row)
                {
                    if barrier {
                        *cell = 0.0;
                        continue;
                    }
                    let mut value = (*cell - RECOVERY_PER_TICK).max(0.0);
                    if occupant.is_some() && deposit_per_occupied_tick > 0.0 {
                        occupied += 1;
                        value = (value + deposit_per_occupied_tick).clamp(0.0, 1.0);
                    }
                    *cell = value;
                    passable += 1;
                    sum += value;
                }
                (sum, passable, occupied)
            })
            .collect();

        let (mut sum, mut passable_cells, mut occupied_cells) = (0.0f32, 0u32, 0u32);
        for (row_sum, row_passable, row_occupied) in rows {
            sum += row_sum;
            passable_cells += row_passable;
            occupied_cells += row_occupied;
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

/// `ClaimCell::pick` of a cell that sends no spread.
const NO_PICK: u32 = u32::MAX;

/// One cell's claim for the type being grown (T24.F02): its own term, the
/// two local growth terms a target needs to finish a spread term, and the
/// cell its one spread contribution goes to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ClaimCell {
    /// `source + local_delta`; 0 for a barrier or an empty cell.
    own: f32,
    local_base: f32,
    local_after_inhibition: f32,
    /// The drawn target's cell index, or [`NO_PICK`].
    pick: u32,
}

impl ClaimCell {
    const EMPTY: Self = Self {
        own: 0.0,
        local_base: 0.0,
        local_after_inhibition: 0.0,
        pick: NO_PICK,
    };
}

/// The per-cell buffers of `grow`, kept between ticks so the timed path
/// allocates nothing grid-sized (spec invariant 5). Every pass that reads a
/// buffer is preceded in the same call by a pass that writes all of it.
#[derive(Default)]
pub(super) struct GrowthScratch {
    claims: Vec<ClaimCell>,
    next_density: Vec<f32>,
    inhibited: Vec<bool>,
    inhibition_sums: Vec<f32>,
}

/// Prints the buffer size, not millions of scratch values, when a world is
/// debug-printed.
impl std::fmt::Debug for GrowthScratch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrowthScratch")
            .field("cells", &self.claims.len())
            .finish_non_exhaustive()
    }
}

impl GrowthScratch {
    /// Sizes every buffer to `cells`; a no-op unless the grid changed.
    fn fit(&mut self, cells: usize) {
        if self.claims.len() != cells {
            self.claims = vec![ClaimCell::EMPTY; cells];
            self.next_density = vec![0.0; cells];
            self.inhibited = vec![false; cells];
            self.inhibition_sums = vec![0.0; cells];
        }
    }

    /// The claims the last growth pass left, for tests.
    #[cfg(test)]
    fn picks(&self) -> impl Iterator<Item = Option<usize>> + '_ {
        self.claims
            .iter()
            .map(|claim| (claim.pick != NO_PICK).then_some(claim.pick as usize))
    }
}

/// The growth suppressed by occupancy depletion and by type inhibition, as
/// one row's or one pass's partial sum.
#[derive(Debug, Clone, Copy, Default)]
struct Suppression {
    occupancy: f32,
    inhibition: f32,
}

impl Suppression {
    /// Add `partials` in their order (row order).
    fn absorb(&mut self, partials: impl IntoIterator<Item = Self>) {
        for partial in partials {
            self.occupancy += partial.occupancy;
            self.inhibition += partial.inhibition;
        }
    }
}

#[inline]
fn inhibition_penalty(weighted_sum: f32, own_density: f32, own_inhibitor: f32) -> f32 {
    (weighted_sum - own_density * own_inhibitor).max(0.0)
}

/// Read-only per-cell inputs of one food type's growth, as flat slices.
struct CellTerms<'a> {
    width: u16,
    height: u16,
    edge_mode: WorldEdgeMode,
    barriers: &'a [bool],
    density: &'a [f32],
    /// The raw fertility grid and the annealed range, when fertility applies.
    habitat: Option<(&'a [f32], f32, f32)>,
    /// The grazing modifiers, when grazing is enabled.
    grazing: Option<&'a [f32]>,
    /// The occupancy-depletion grid, when depletion is enabled.
    depletion: Option<&'a [f32]>,
    inhibition_sums: &'a [f32],
    max_density: f32,
    type_inhibitor: f32,
}

impl CellTerms<'_> {
    #[inline]
    fn source(&self, idx: usize) -> f32 {
        self.density[idx].clamp(0.0, self.max_density)
    }

    /// The annealed habitat fertility (1.0 when fertility is disabled or the
    /// type has no grid) times the cell's grazing modifier.
    #[inline]
    fn base_fertility(&self, idx: usize) -> f32 {
        let habitat = self.habitat.map_or(1.0, |(raw, min, max)| {
            fertility::map_fertility(raw[idx], min, max)
        });
        habitat * self.grazing.map_or(1.0, |modifiers| modifiers[idx])
    }

    #[inline]
    fn penalty(&self, idx: usize, source: f32) -> f32 {
        inhibition_penalty(self.inhibition_sums[idx], source, self.type_inhibitor)
    }

    #[inline]
    fn multiplier(&self, idx: usize) -> f32 {
        self.depletion.map_or(1.0, |depletion| {
            1.0 - depletion[idx] * (1.0 - MIN_GROWTH_MULTIPLIER)
        })
    }

    /// The cell index of `idx`'s neighbour in `dir` under the edge mode.
    #[inline]
    fn neighbor(&self, idx: usize, dir: Direction) -> Option<usize> {
        let width = usize::from(self.width);
        let pos = Position::new((idx % width) as u16, (idx / width) as u16);
        resolve_neighbor_static(pos, dir, self.width, self.height, self.edge_mode)
            .map(|target| usize::from(target.y) * width + usize::from(target.x))
    }
}

/// Per-type growth parameters shared by the claim, gather and recovery passes.
#[derive(Clone, Copy)]
struct TypeRates {
    growth_rate: f32,
    spread_threshold: f32,
    spread_density_ratio: f32,
}

/// The claim of cell `idx`: own term, local terms, own inhibited flag, and
/// the drawn spread target (one uniformly chosen entry of the N, E, S, W
/// candidates that resolve and are passable, repeats included).
#[inline]
fn claim_cell(
    terms: &CellTerms<'_>,
    rates: TypeRates,
    stream: FoodStream,
    idx: usize,
    inhibited: &mut bool,
    suppression: &mut Suppression,
) -> ClaimCell {
    if terms.barriers[idx] {
        return ClaimCell::EMPTY;
    }
    let source = terms.source(idx);
    if source <= 0.0 {
        return ClaimCell::EMPTY;
    }
    let base_cell_fertility = terms.base_fertility(idx);
    let cell_inhibition_penalty = terms.penalty(idx, source);
    let cell_fertility = (base_cell_fertility - cell_inhibition_penalty).max(0.0);
    let local_base = source * rates.growth_rate * base_cell_fertility;
    let local_after_inhibition = source * rates.growth_rate * cell_fertility;
    let local_delta = local_after_inhibition * terms.multiplier(idx);
    if local_base > local_after_inhibition && cell_inhibition_penalty > 0.0 {
        *inhibited = true;
    }
    suppression.inhibition += (local_base - local_after_inhibition).max(0.0);
    suppression.occupancy += (local_after_inhibition - local_delta).max(0.0);

    let mut pick = NO_PICK;
    if source >= rates.spread_threshold && local_after_inhibition > 0.0 {
        let mut candidates = [0usize; 4];
        let mut count = 0;
        for dir in SPREAD_DIRECTIONS {
            if let Some(target) = terms.neighbor(idx, dir) {
                if !terms.barriers[target] {
                    candidates[count] = target;
                    count += 1;
                }
            }
        }
        if count > 0 {
            pick = candidates[stream.below(idx as u64, count)] as u32;
        }
    }
    ClaimCell {
        own: source + local_delta,
        local_base,
        local_after_inhibition,
        pick,
    }
}

/// The density written for cell `t`: `clamp(own(t) + Σ spread_delta(s, t))`
/// over the distinct N, E, S, W neighbours `s` (first occurrence) whose pick
/// is `t`, each counted once; barrier cells get 0. The target-side terms are
/// computed only when some source delivers.
#[inline]
fn gather_cell(
    terms: &CellTerms<'_>,
    rates: TypeRates,
    claims: &[ClaimCell],
    t: usize,
    inhibited: &mut bool,
    suppression: &mut Suppression,
) -> f32 {
    if terms.barriers[t] {
        return 0.0;
    }
    let mut next = claims[t].own.clamp(0.0, terms.max_density);
    let mut sources = [usize::MAX; 4];
    let mut target_terms: Option<(f32, f32, f32)> = None;
    for (slot, dir) in SPREAD_DIRECTIONS.into_iter().enumerate() {
        let Some(s) = terms.neighbor(t, dir) else {
            continue;
        };
        if sources[..slot].contains(&s) {
            continue;
        }
        sources[slot] = s;
        let claim = claims[s];
        if claim.pick as usize != t {
            continue;
        }
        let (base_neighbor_fertility, neighbor_penalty, target_multiplier) = *target_terms
            .get_or_insert_with(|| {
                (
                    terms.base_fertility(t),
                    terms.penalty(t, terms.source(t)),
                    terms.multiplier(t),
                )
            });
        let neighbor_fertility = (base_neighbor_fertility - neighbor_penalty).max(0.0);
        let spread_base = claim.local_base * rates.spread_density_ratio * base_neighbor_fertility;
        let spread_after_inhibition =
            claim.local_after_inhibition * rates.spread_density_ratio * neighbor_fertility;
        let spread_delta = spread_after_inhibition * target_multiplier;
        if spread_base > spread_after_inhibition && neighbor_penalty > 0.0 {
            *inhibited = true;
        }
        suppression.inhibition += (spread_base - spread_after_inhibition).max(0.0);
        suppression.occupancy += (spread_after_inhibition - spread_delta).max(0.0);
        next = (next + spread_delta).clamp(0.0, terms.max_density);
    }
    next
}

/// The weighted density of every inhibiting type per cell, summed in catalog
/// order, into `sums`; one row-parallel pass.
fn build_type_inhibition_weighted_sums(
    sums: &mut [f32],
    state: &OrdinaryFoodState,
    catalog: &OrdinaryFoodCatalog,
    max_density: f32,
) {
    let inhibitors: Vec<(&[f32], f32)> = catalog
        .entries()
        .iter()
        .filter_map(|entry| {
            let inhibitor = entry.config.growth_inhibitor.clamp(0.0, 1.0);
            (inhibitor > 0.0)
                .then(|| state.density_slice(entry.id).map(|grid| (grid, inhibitor)))
                .flatten()
        })
        .collect();
    let width = usize::from(state.width());
    sums.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        for (x, cell) in row.iter_mut().enumerate() {
            let idx = y * width + x;
            let mut sum = 0.0;
            for &(density, inhibitor) in &inhibitors {
                sum += density[idx].clamp(0.0, max_density) * inhibitor;
            }
            *cell = sum;
        }
    });
}

/// The type's total density: per-row sums reduced in row order.
fn row_ordered_total(density: &[f32], width: usize) -> f32 {
    density
        .par_chunks(width)
        .map(|row| row.iter().sum::<f32>())
        .collect::<Vec<f32>>()
        .into_iter()
        .sum()
}

#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::too_many_lines,
    reason = "one food growth pass over the grid: the eight timed regions of \
              T24.F03 share the per-type terms and the retained buffers"
)]
pub(super) fn grow<T: Clone + Sync>(
    state: &mut OrdinaryFoodState,
    catalog: &OrdinaryFoodCatalog,
    shared: &FoodResourceConfig,
    full_config: &FoodConfig,
    occupancy_depletion: &mut OccupancyDepletionLayer,
    scratch: &mut GrowthScratch,
    barriers: &Grid<bool>,
    occupancy: &Grid<Option<T>>,
    edge_mode: WorldEdgeMode,
    tick: u64,
    key: FoodStreamKey,
) -> FoodGrowthSummary {
    let total_cells = state.width() as usize * state.height() as usize;
    if total_cells == 0 {
        return FoodGrowthSummary::default();
    }
    let width = usize::from(state.width());

    let mut clock = PassClock::new(catalog.len());
    let start = PassClock::start();
    let depletion_summary =
        occupancy_depletion.recover_and_deposit(barriers, occupancy, &shared.occupancy_depletion);
    clock.stop(start, FoodPass::DepletionRecovery, None);
    // Grazing heals before any fertility read, on every passable cell: the
    // grazed cell is empty by construction and the growth passes below skip
    // empty cells.
    let start = PassClock::start();
    let (grazing_summaries, passable_cells) =
        state.grazing_mut().recover(barriers, &shared.grazing);
    clock.stop(start, FoodPass::GrazingRecovery, None);

    let max_density = shared.max_density.max(0.0);
    if max_density <= 0.0 || catalog.is_empty() {
        return clock.finish(FoodGrowthSummary::default());
    }

    let spread_threshold = max_density * shared.spread_threshold_ratio.clamp(0.0, 1.0);
    let spread_density_ratio = shared.spread_density_ratio.clamp(0.0, 1.0);
    let recovery_floor = shared.recovery_floor_ratio.clamp(0.0, 1.0);

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

    scratch.fit(total_cells);
    let GrowthScratch {
        claims,
        next_density,
        inhibited,
        inhibition_sums,
    } = scratch;

    let start = PassClock::start();
    build_type_inhibition_weighted_sums(inhibition_sums, state, catalog, max_density);
    inhibited.par_iter_mut().for_each(|flag| *flag = false);
    clock.stop(start, FoodPass::InhibitionSums, None);
    let mut suppressed = Suppression::default();
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
        let rates = TypeRates {
            growth_rate: entry
                .config
                .growth_rate
                .unwrap_or(shared.growth_rate)
                .max(0.0),
            spread_threshold,
            spread_density_ratio,
        };
        let recovery_spawn_rate = entry
            .config
            .recovery_spawn_rate
            .unwrap_or(shared.recovery_spawn_rate)
            .clamp(0.0, 1.0);
        let Some(density) = state.density_slice(type_idx) else {
            continue;
        };
        let terms = CellTerms {
            width: state.width(),
            height: state.height(),
            edge_mode,
            barriers: barriers.as_slice(),
            density,
            habitat: full_config
                .fertility
                .enabled
                .then(|| state.fertility_grid(type_idx))
                .flatten()
                .map(|grid| (grid.as_slice(), eff_min, eff_max)),
            grazing: shared
                .grazing
                .enabled
                .then(|| state.grazing().type_slice(type_idx))
                .flatten(),
            depletion: shared
                .occupancy_depletion
                .enabled
                .then(|| occupancy_depletion.grid().as_slice()),
            inhibition_sums: inhibition_sums.as_slice(),
            max_density,
            type_inhibitor: entry.config.growth_inhibitor.clamp(0.0, 1.0),
        };

        let type_density_ratio = {
            let start = PassClock::start();
            let type_total = row_ordered_total(density, width);
            clock.stop(start, FoodPass::DensityTotal, Some(type_idx));
            let type_capacity = total_cells as f32 * max_density;
            if type_capacity <= 0.0 {
                0.0
            } else {
                (type_total / type_capacity).clamp(0.0, 1.0)
            }
        };

        let start = PassClock::start();
        let spread_stream = FoodStream::new(key, tick, type_idx, DrawPurpose::SpreadTarget);
        let claim_rows: Vec<Suppression> = claims
            .par_chunks_mut(width)
            .zip(inhibited.par_chunks_mut(width))
            .enumerate()
            .map(|(y, (claim_row, flag_row))| {
                let mut row = Suppression::default();
                for (x, (claim, flag)) in claim_row.iter_mut().zip(flag_row).enumerate() {
                    *claim =
                        claim_cell(&terms, rates, spread_stream, y * width + x, flag, &mut row);
                }
                row
            })
            .collect();
        suppressed.absorb(claim_rows);
        let claims_read: &[ClaimCell] = claims;
        let gather_rows: Vec<Suppression> = next_density
            .par_chunks_mut(width)
            .zip(inhibited.par_chunks_mut(width))
            .enumerate()
            .map(|(y, (next_row, flag_row))| {
                let mut row = Suppression::default();
                for (x, (next, flag)) in next_row.iter_mut().zip(flag_row).enumerate() {
                    *next = gather_cell(&terms, rates, claims_read, y * width + x, flag, &mut row);
                }
                row
            })
            .collect();
        suppressed.absorb(gather_rows);
        clock.stop(start, FoodPass::GrowthSpread, Some(type_idx));

        if type_density_ratio < recovery_floor && recovery_spawn_rate > 0.0 {
            let start = PassClock::start();
            let spawn_attempts = (total_cells as f32 * recovery_spawn_rate).round() as usize;
            let spawn_delta = max_density * rates.growth_rate;
            let recovery_stream = FoodStream::new(key, tick, type_idx, DrawPurpose::RecoveryCell);

            for attempt in 0..spawn_attempts {
                if spawn_delta <= 0.0 {
                    break;
                }
                let idx = recovery_stream.below(attempt as u64, total_cells);
                if terms.barriers[idx] {
                    continue;
                }
                let base_cell_fertility = terms.base_fertility(idx);
                let recovery_inhibition_penalty = terms.penalty(idx, terms.source(idx));
                let cell_fertility = (base_cell_fertility - recovery_inhibition_penalty).max(0.0);
                let recovery_base = spawn_delta * base_cell_fertility;
                let recovery_after_inhibition = spawn_delta * cell_fertility;
                let recovery_delta = recovery_after_inhibition * terms.multiplier(idx);
                if recovery_base > recovery_after_inhibition && recovery_inhibition_penalty > 0.0 {
                    inhibited[idx] = true;
                }
                suppressed.inhibition += (recovery_base - recovery_after_inhibition).max(0.0);
                suppressed.occupancy += (recovery_after_inhibition - recovery_delta).max(0.0);
                next_density[idx] = (next_density[idx] + recovery_delta).clamp(0.0, max_density);
            }
            clock.stop(start, FoodPass::RecoverySpawns, Some(type_idx));
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
        let start = PassClock::start();
        let written = state
            .density_slice_mut(type_idx)
            .expect("the type's density grid was read above");
        let write_rows: Vec<(u32, f32)> = written
            .par_chunks_mut(width)
            .zip(next_density.par_chunks(width))
            .map(|(row, next_row)| {
                row.copy_from_slice(next_row);
                let (mut occupied, mut total) = (0u32, 0.0f32);
                for &density in next_row {
                    if density > 0.0 {
                        occupied += 1;
                        total += density;
                    }
                }
                (occupied, total)
            })
            .collect();
        for (occupied, total) in write_rows {
            telemetry.occupied_cells += occupied;
            telemetry.total_density += total;
        }
        per_type[usize::from(type_idx.get())] = telemetry;
        clock.stop(start, FoodPass::WriteBack, Some(type_idx));
    }

    let start = PassClock::start();
    let inhibited_cells = inhibited.par_iter().filter(|flag| **flag).count() as u32;
    clock.stop(start, FoodPass::InhibitedCount, None);

    clock.finish(FoodGrowthSummary {
        mean_occupancy_depletion: depletion_summary.mean_depletion,
        occupied_cells_with_depletion: depletion_summary.occupied_cells,
        growth_suppressed_by_occupancy_depletion: suppressed.occupancy,
        cells_with_type_inhibition: inhibited_cells,
        growth_suppressed_by_type_inhibition: suppressed.inhibition,
        passable_cells,
        per_type,
        #[cfg(feature = "telemetry-seams")]
        passes: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::stream::{food_draw, in_range};
    use super::*;

    use rand::{rngs::StdRng, SeedableRng};

    fn barrier_grid(width: u16, height: u16) -> Grid<bool> {
        Grid::new(width, height, false)
    }

    fn occupancy_grid<T: Clone>(width: u16, height: u16) -> Grid<Option<T>> {
        Grid::new(width, height, None)
    }

    #[test]
    fn row_ordered_total_bits_match_row_by_row_sum_in_every_thread_pool() {
        let (width, height) = (7_usize, 37_usize);
        let mut rng = StdRng::seed_from_u64(0x7240_f002);
        let density: Vec<f32> = (0..width * height)
            .map(|_| {
                if rng.gen_bool(0.3) {
                    rng.gen_range(1.0e6_f32..1.0e7)
                } else {
                    rng.gen_range(0.1_f32..1.0)
                }
            })
            .collect();
        let reference = density
            .chunks(width)
            .map(|row| row.iter().sum::<f32>())
            .sum::<f32>();
        assert_ne!(
            density.iter().sum::<f32>().to_bits(),
            reference.to_bits(),
            "fixture must be sensitive to summation order"
        );

        for threads in [1, 4] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("thread pool");
            let total = pool.install(|| row_ordered_total(&density, width));
            assert_eq!(
                total.to_bits(),
                reference.to_bits(),
                "{threads}-thread pool total {total} != row-by-row {reference}"
            );
        }
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
        let barriers = barrier_grid(1, 1);
        let mut rng = StdRng::seed_from_u64(7);

        seed_density(
            &mut state,
            &catalog,
            &config,
            &mut occupancy_depletion,
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
        let barriers = barrier_grid(8, 1);
        let mut rng = StdRng::seed_from_u64(0xC0FFEE);

        seed_density(
            &mut state,
            &catalog,
            &config,
            &mut occupancy_depletion,
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
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let key = FoodStreamKey::from_run_seed(11);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            key,
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
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let key = FoodStreamKey::from_run_seed(23);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            key,
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
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let key = FoodStreamKey::from_run_seed(37);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            key,
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
        let barriers = barrier_grid(1, 1);
        let occupancy = occupancy_grid::<()>(1, 1);
        let key = FoodStreamKey::from_run_seed(31);

        grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            key,
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
        let key = FoodStreamKey::from_run_seed(5);
        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Bounded,
            0,
            key,
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
        let key = FoodStreamKey::from_run_seed(1);
        let mut readings = Vec::new();
        for tick in 0..3 {
            let summary = grow(
                &mut state,
                &catalog,
                &config.shared,
                &config,
                &mut occupancy_depletion,
                &mut GrowthScratch::default(),
                &barriers,
                &occupancy,
                WorldEdgeMode::Wrap,
                tick,
                key,
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
        let key = FoodStreamKey::from_run_seed(1);
        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut occupancy_depletion,
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            key,
        );
        // local delta = source * rate * grazing (0.5) * occupancy multiplier
        // (MIN_GROWTH_MULTIPLIER at full depletion).
        let expected = 1.0 + 1.0 * 0.5 * 0.5 * MIN_GROWTH_MULTIPLIER;
        assert!((state.food_at_type(pos, type_idx) - expected).abs() < 1e-5);
        // The occupancy-suppressed share of the local growth is reported
        // (T24.F01 mutation gate).
        let suppressed = 1.0 * 0.5 * 0.5 * (1.0 - MIN_GROWTH_MULTIPLIER);
        assert!(
            (summary.growth_suppressed_by_occupancy_depletion - suppressed).abs() < 1e-6,
            "suppressed by occupancy: {}",
            summary.growth_suppressed_by_occupancy_depletion
        );
    }

    /// Without density headroom `grow` stops after the two recovery calls:
    /// no type is grown or written back, so no per-type telemetry and no
    /// passable-cell count leave the call (T24.F03 mutation gate).
    #[test]
    fn grow_without_max_density_returns_before_the_per_type_passes() {
        let mut config = FoodConfig::default();
        config.shared.max_density = 0.0;
        config.types.push(crate::config::FoodTypeConfig::default());
        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(4, 3, catalog.len());
        let barriers = barrier_grid(4, 3);
        let occupancy = occupancy_grid::<()>(4, 3);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut OccupancyDepletionLayer::new(4, 3),
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            WorldEdgeMode::Wrap,
            0,
            FoodStreamKey::from_run_seed(5),
        );

        assert!(summary.per_type.is_empty(), "{:?}", summary.per_type);
        assert_eq!(summary.passable_cells, 0);
    }

    /// Food config with every growth modifier off (fertility, inhibition,
    /// occupancy depletion, grazing), so local growth is `source × g`, a
    /// spread is `source × g × spread_density_ratio`, and a recovery spawn is
    /// `max_density × g` (T24.F01).
    fn plain_growth_config(type_count: usize) -> FoodConfig {
        let mut config = FoodConfig::default();
        config.fertility.enabled = false;
        config.shared.occupancy_depletion.enabled = false;
        config.shared.grazing.enabled = false;
        config.shared.recovery_spawn_rate = 0.0;
        config.shared.recovery_floor_ratio = 0.0;
        config.types = vec![
            crate::config::FoodTypeConfig {
                growth_inhibitor: 0.0,
                ..crate::config::FoodTypeConfig::default()
            };
            type_count
        ];
        config
    }

    /// One growth pass with no occupancy and no grazing ledgers, the
    /// world size taken from `barriers`.
    fn grow_once(
        state: &mut OrdinaryFoodState,
        catalog: &OrdinaryFoodCatalog,
        config: &FoodConfig,
        barriers: &Grid<bool>,
        edge_mode: WorldEdgeMode,
        seed: u64,
    ) {
        let (width, height) = (barriers.width(), barriers.height());
        grow(
            state,
            catalog,
            &config.shared,
            config,
            &mut OccupancyDepletionLayer::new(width, height),
            &mut GrowthScratch::default(),
            barriers,
            &occupancy_grid::<()>(width, height),
            edge_mode,
            0,
            FoodStreamKey::from_run_seed(seed),
        );
    }

    /// One growth pass on a `Bounded` 2x1 world with a dense cell spreading
    /// into an occupied cell below the spread threshold; returns the
    /// occupied cell's density.
    fn two_cell_spread_into_occupied(dense: Position, occupied: Position) -> f32 {
        let mut config = plain_growth_config(1);
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 1.0;
        config.shared.spread_threshold_ratio = 0.08;
        config.shared.max_density = 10.0;
        let catalog = OrdinaryFoodCatalog::new(&config);
        let type_idx = OrdinaryFoodTypeId::new(0);
        let mut state = OrdinaryFoodState::new(2, 1, catalog.len());
        state.set_food_type_density(dense, type_idx, 1.0);
        state.set_food_type_density(occupied, type_idx, 0.5);

        grow_once(
            &mut state,
            &catalog,
            &config,
            &barrier_grid(2, 1),
            WorldEdgeMode::Bounded,
            5,
        );
        state.food_at_type(occupied, type_idx)
    }

    #[test]
    fn spread_into_an_occupied_cell_adds_to_its_growth_in_both_orientations() {
        // target × (1 + g) + source × g × spread_density_ratio
        let expected = 0.5 * 1.5 + 1.0 * 0.5 * 1.0;
        let west_dense = two_cell_spread_into_occupied(Position::new(0, 0), Position::new(1, 0));
        let east_dense = two_cell_spread_into_occupied(Position::new(1, 0), Position::new(0, 0));
        assert!(
            (west_dense - expected).abs() < 1e-6,
            "west dense: {west_dense}"
        );
        assert!(
            (east_dense - expected).abs() < 1e-6,
            "east dense: {east_dense}"
        );
    }

    /// A growth-pass fixture for the per-cell oracle: the pre-growth state a
    /// test builds, run once through `grow` and once through the oracle.
    struct OracleFixture {
        width: u16,
        height: u16,
        edge_mode: WorldEdgeMode,
        config: FoodConfig,
        /// Pre-growth density per type, row-major.
        densities: Vec<Vec<f32>>,
        barriers: Vec<usize>,
        occupied: Vec<usize>,
        /// `(cell, type, bites)` applied to the grazing layer before the pass.
        bites: Vec<(usize, u16, u32)>,
        seed: u64,
        tick: u64,
    }

    impl OracleFixture {
        fn new(width: u16, height: u16, edge_mode: WorldEdgeMode, config: FoodConfig) -> Self {
            let cells = usize::from(width) * usize::from(height);
            Self {
                width,
                height,
                edge_mode,
                densities: vec![vec![0.0; cells]; config.types.len()],
                config,
                barriers: Vec::new(),
                occupied: Vec::new(),
                bites: Vec::new(),
                seed: 1,
                tick: 0,
            }
        }

        fn cells(&self) -> usize {
            usize::from(self.width) * usize::from(self.height)
        }

        fn position(&self, idx: usize) -> Position {
            let width = usize::from(self.width);
            Position::new((idx % width) as u16, (idx / width) as u16)
        }
    }

    /// What the oracle computed and saw for one growth pass.
    #[derive(Debug, Default)]
    struct OracleReport {
        /// Expected density per type, per cell.
        expected: Vec<Vec<f32>>,
        inhibited_cells: u32,
        suppressed_by_inhibition: f32,
        suppressed_by_occupancy: f32,
        /// A delivery crossed a wrapped edge.
        wrapped_delivery: bool,
        /// Some cell's unclamped sum exceeded `max_density`.
        clamp_bound: bool,
        /// Two distinct sources delivered to one target.
        shared_target: bool,
        /// A source delivered to a cell that is two of its neighbours, or to itself.
        repeated_candidate_delivery: bool,
        /// A delivery was reduced by the target's grazing, inhibition or depletion.
        modified_delivery: bool,
        /// A spreading cell had an empty candidate list.
        empty_candidate_list: bool,
        /// A recovery spawn landed on a passable cell.
        recovery_spawned: bool,
        /// A spread landed on an occupied, higher-index target with headroom
        /// after its own growth (T24.F01's lost delivery).
        delivery_to_later_occupied_cell: bool,
    }

    /// Test-side scatter of spec invariant 1 with grazing, occupancy
    /// depletion and type inhibition applied (fertility off): each spreading
    /// cell sends one contribution to the candidate-list entry `food_draw`
    /// picks; each cell's own growth, received spread and recovery spawns are
    /// summed before one clamp. Grazing and depletion recover first, as in `grow`.
    #[allow(
        clippy::too_many_lines,
        reason = "one independent scatter of the whole growth rule, kept in one place to read against \
                  the spec"
    )]
    fn per_cell_oracle(fixture: &OracleFixture) -> OracleReport {
        let shared = &fixture.config.shared;
        let cells = fixture.cells();
        let max = shared.max_density;
        let threshold = max * shared.spread_threshold_ratio;
        let key = FoodStreamKey::from_run_seed(fixture.seed);
        let is_barrier = |idx: usize| fixture.barriers.contains(&idx);
        let mut report = OracleReport::default();

        let depletion: Vec<f32> = (0..cells)
            .map(|idx| {
                let deposit = shared.occupancy_depletion.deposit_per_occupied_tick;
                if is_barrier(idx) || !fixture.occupied.contains(&idx) {
                    0.0
                } else {
                    deposit.clamp(0.0, 1.0)
                }
            })
            .collect();
        let multiplier = |idx: usize| {
            if shared.occupancy_depletion.enabled {
                1.0 - depletion[idx] * (1.0 - MIN_GROWTH_MULTIPLIER)
            } else {
                1.0
            }
        };
        let grazing = |idx: usize, type_number: u16| {
            if !shared.grazing.enabled || is_barrier(idx) {
                return 1.0;
            }
            let mut modifier = 1.0f32;
            for &(cell, bitten_type, bites) in &fixture.bites {
                if cell == idx && bitten_type == type_number {
                    for _ in 0..bites {
                        modifier = (modifier * shared.grazing.factor).max(shared.grazing.floor);
                    }
                }
            }
            (modifier + 1.0 / shared.grazing.recovery_ticks.max(1) as f32).min(1.0)
        };
        let inhibitors: Vec<f32> = fixture
            .config
            .types
            .iter()
            .map(|food_type| food_type.growth_inhibitor.clamp(0.0, 1.0))
            .collect();
        let weighted_sum = |idx: usize| {
            let mut sum = 0.0f32;
            for (density, &inhibitor) in fixture.densities.iter().zip(&inhibitors) {
                if inhibitor > 0.0 {
                    sum += density[idx].clamp(0.0, max) * inhibitor;
                }
            }
            sum
        };
        let neighbors = |idx: usize| -> Vec<usize> {
            let pos = fixture.position(idx);
            SPREAD_DIRECTIONS
                .into_iter()
                .filter_map(|dir| {
                    resolve_neighbor_static(
                        pos,
                        dir,
                        fixture.width,
                        fixture.height,
                        fixture.edge_mode,
                    )
                })
                .map(|p| usize::from(p.y) * usize::from(fixture.width) + usize::from(p.x))
                .filter(|&target| !is_barrier(target))
                .collect()
        };
        let mut inhibited = vec![false; cells];

        for (type_number, density) in fixture.densities.iter().enumerate() {
            let type_raw = type_number as u16;
            let type_idx = OrdinaryFoodTypeId::new(type_raw);
            let food_type = &fixture.config.types[type_number];
            let g = food_type.growth_rate.unwrap_or(shared.growth_rate);
            let inhibitor = inhibitors[type_number];
            // (base fertility, penalty) at a cell for this type.
            let fertility_terms = |idx: usize| {
                let base = grazing(idx, type_raw);
                let penalty =
                    (weighted_sum(idx) - density[idx].clamp(0.0, max) * inhibitor).max(0.0);
                (base, penalty)
            };
            let mut own = vec![0.0f32; cells];
            let mut sum = vec![0.0f32; cells];
            let mut senders: Vec<Vec<usize>> = vec![Vec::new(); cells];
            for idx in 0..cells {
                let source = density[idx].clamp(0.0, max);
                if is_barrier(idx) || source <= 0.0 {
                    continue;
                }
                let (base, penalty) = fertility_terms(idx);
                let fertility = (base - penalty).max(0.0);
                let local_base = source * g * base;
                let local_after = source * g * fertility;
                let local_delta = local_after * multiplier(idx);
                if local_base > local_after && penalty > 0.0 {
                    inhibited[idx] = true;
                }
                report.suppressed_by_inhibition += (local_base - local_after).max(0.0);
                report.suppressed_by_occupancy += (local_after - local_delta).max(0.0);
                own[idx] = source + local_delta;
                sum[idx] += own[idx];
                if source < threshold || local_after <= 0.0 {
                    continue;
                }
                let candidates = neighbors(idx);
                if candidates.is_empty() {
                    report.empty_candidate_list = true;
                    continue;
                }
                let draw = food_draw(
                    key,
                    fixture.tick,
                    type_idx,
                    DrawPurpose::SpreadTarget,
                    idx as u64,
                );
                let target = candidates[in_range(draw, candidates.len())];
                let (target_base, target_penalty) = fertility_terms(target);
                let target_fertility = (target_base - target_penalty).max(0.0);
                let spread_base = local_base * shared.spread_density_ratio * target_base;
                let spread_after = local_after * shared.spread_density_ratio * target_fertility;
                let spread_delta = spread_after * multiplier(target);
                if spread_base > spread_after && target_penalty > 0.0 {
                    inhibited[target] = true;
                }
                report.suppressed_by_inhibition += (spread_base - spread_after).max(0.0);
                report.suppressed_by_occupancy += (spread_after - spread_delta).max(0.0);
                report.modified_delivery |=
                    spread_delta < local_after * shared.spread_density_ratio;
                sum[target] += spread_delta;
                senders[target].push(idx);
                report.repeated_candidate_delivery |= target == idx
                    || candidates.iter().filter(|&&entry| entry == target).count() > 1;
                let (source_pos, target_pos) = (fixture.position(idx), fixture.position(target));
                if source_pos.x.abs_diff(target_pos.x) > 1
                    || source_pos.y.abs_diff(target_pos.y) > 1
                {
                    report.wrapped_delivery = true;
                }
                report.delivery_to_later_occupied_cell |=
                    target > idx && density[target] > 0.0 && own[target] < max;
            }
            let type_total: f32 = density.iter().sum();
            let spawn_rate = food_type
                .recovery_spawn_rate
                .unwrap_or(shared.recovery_spawn_rate);
            if type_total / (cells as f32 * max) < shared.recovery_floor_ratio && spawn_rate > 0.0 {
                let attempts = (cells as f32 * spawn_rate).round() as usize;
                for attempt in 0..attempts {
                    let draw = food_draw(
                        key,
                        fixture.tick,
                        type_idx,
                        DrawPurpose::RecoveryCell,
                        attempt as u64,
                    );
                    let idx = in_range(draw, cells);
                    if is_barrier(idx) {
                        continue;
                    }
                    let (base, penalty) = fertility_terms(idx);
                    let recovery_base = max * g * base;
                    let recovery_after = max * g * (base - penalty).max(0.0);
                    let recovery_delta = recovery_after * multiplier(idx);
                    if recovery_base > recovery_after && penalty > 0.0 {
                        inhibited[idx] = true;
                    }
                    report.suppressed_by_inhibition += (recovery_base - recovery_after).max(0.0);
                    report.suppressed_by_occupancy += (recovery_after - recovery_delta).max(0.0);
                    report.recovery_spawned = true;
                    sum[idx] += recovery_delta;
                }
            }
            report.shared_target |= senders.iter().any(|from| from.len() > 1);
            report.clamp_bound |= sum.iter().any(|&value| value > max);
            report
                .expected
                .push(sum.into_iter().map(|value| value.clamp(0.0, max)).collect());
        }
        report.inhibited_cells = inhibited.iter().filter(|flag| **flag).count() as u32;
        report
    }

    /// Runs `fixture` through `grow` and the oracle; asserts every cell's
    /// density, the inhibited-cell count and both suppression totals agree.
    fn assert_grow_matches_oracle(fixture: &OracleFixture) -> OracleReport {
        let catalog = OrdinaryFoodCatalog::new(&fixture.config);
        let mut state = OrdinaryFoodState::new(fixture.width, fixture.height, catalog.len());
        let mut barriers = barrier_grid(fixture.width, fixture.height);
        for &idx in &fixture.barriers {
            let pos = fixture.position(idx);
            barriers.set(pos.x, pos.y, true);
        }
        let mut occupancy = occupancy_grid::<()>(fixture.width, fixture.height);
        for &idx in &fixture.occupied {
            let pos = fixture.position(idx);
            occupancy.set(pos.x, pos.y, Some(()));
        }
        for &(idx, type_raw, bites) in &fixture.bites {
            let pos = fixture.position(idx);
            for _ in 0..bites {
                state.grazing_mut().bite(
                    pos.x,
                    pos.y,
                    OrdinaryFoodTypeId::new(type_raw),
                    &fixture.config.shared.grazing,
                );
            }
        }
        for (type_number, density) in fixture.densities.iter().enumerate() {
            for (idx, &value) in density.iter().enumerate() {
                state.set_food_type_density(
                    fixture.position(idx),
                    OrdinaryFoodTypeId::new(type_number as u16),
                    value,
                );
            }
        }
        let report = per_cell_oracle(fixture);

        let summary = grow(
            &mut state,
            &catalog,
            &fixture.config.shared,
            &fixture.config,
            &mut OccupancyDepletionLayer::new(fixture.width, fixture.height),
            &mut GrowthScratch::default(),
            &barriers,
            &occupancy,
            fixture.edge_mode,
            fixture.tick,
            FoodStreamKey::from_run_seed(fixture.seed),
        );

        for (type_number, expected) in report.expected.iter().enumerate() {
            let mut written_occupied = 0u32;
            for (idx, &want) in expected.iter().enumerate() {
                let pos = fixture.position(idx);
                let got = state.food_at_type(pos, OrdinaryFoodTypeId::new(type_number as u16));
                assert!(
                    (got - want).abs() <= 1e-5 * want.max(1.0),
                    "type {type_number} cell ({}, {}): got {got}, want {want}",
                    pos.x,
                    pos.y
                );
                written_occupied += u32::from(got > 0.0);
            }
            assert_eq!(
                summary.per_type[type_number].occupied_cells, written_occupied,
                "type {type_number}: occupied cells counts only cells with food"
            );
        }
        assert_eq!(summary.cells_with_type_inhibition, report.inhibited_cells);
        for (got, want, what) in [
            (
                summary.growth_suppressed_by_type_inhibition,
                report.suppressed_by_inhibition,
                "inhibition",
            ),
            (
                summary.growth_suppressed_by_occupancy_depletion,
                report.suppressed_by_occupancy,
                "occupancy",
            ),
        ] {
            assert!(
                (got - want).abs() <= 1e-5 * want.max(1.0),
                "suppressed by {what}: got {got}, want {want}"
            );
        }
        report
    }

    #[test]
    fn growth_pass_writes_the_per_cell_sum_of_own_growth_spread_and_recovery() {
        const WIDTH: u16 = 4;
        const HEIGHT: u16 = 3;
        let mut config = plain_growth_config(2);
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 0.5;
        config.shared.spread_threshold_ratio = 0.5;
        config.shared.max_density = 1.0;
        config.shared.recovery_spawn_rate = 0.25;
        config.shared.recovery_floor_ratio = 1.0;
        let mut fixture = OracleFixture::new(WIDTH, HEIGHT, WorldEdgeMode::Wrap, config);
        fixture.barriers = vec![5];
        #[rustfmt::skip]
        let densities = vec![
            vec![
                0.9, 0.3, 0.0, 0.9,
                0.3, 0.0, 0.9, 0.3,
                0.9, 0.3, 0.3, 0.6,
            ],
            vec![
                0.3, 0.9, 0.3, 0.0,
                0.6, 0.0, 0.3, 0.9,
                0.0, 0.3, 0.9, 0.3,
            ],
        ];
        fixture.densities = densities;
        fixture.seed = 7;

        let report = assert_grow_matches_oracle(&fixture);

        assert!(
            report.wrapped_delivery,
            "fixture must deliver across a wrapped edge"
        );
        assert!(
            report.clamp_bound,
            "fixture must bind the max_density clamp"
        );
        assert!(report.recovery_spawned, "fixture must spawn recovery");
        assert!(
            report.delivery_to_later_occupied_cell,
            "fixture must spread onto an occupied later-visited cell with headroom"
        );
    }

    /// Every mechanism the gather reads on: grazing (bitten cells), occupancy
    /// depletion (occupied cells), two inhibiting types, a barrier, a type
    /// below its recovery floor, on a wrapped grid.
    #[test]
    fn growth_pass_matches_the_per_cell_oracle_with_every_mechanism_enabled() {
        let mut config = plain_growth_config(2);
        config.shared.grazing = crate::config::GrazingConfig {
            recovery_ticks: 50,
            ..crate::config::GrazingConfig::default()
        };
        config.shared.occupancy_depletion.enabled = true;
        config.shared.occupancy_depletion.deposit_per_occupied_tick = 0.6;
        config.shared.growth_rate = 0.4;
        config.shared.spread_density_ratio = 0.8;
        config.shared.spread_threshold_ratio = 0.5;
        config.shared.max_density = 1.0;
        config.shared.recovery_floor_ratio = 0.2;
        config.types[0].growth_inhibitor = 0.3;
        config.types[1].growth_inhibitor = 0.6;
        config.types[1].recovery_spawn_rate = Some(0.3);
        let mut fixture = OracleFixture::new(6, 5, WorldEdgeMode::Wrap, config);
        let cells = fixture.cells();
        fixture.densities[0] = (0..cells)
            .map(|idx| [0.95, 0.7, 0.0, 0.55, 0.2][idx % 5])
            .collect();
        // Type 1 sits below its recovery floor but still inhibits type 0.
        fixture.densities[1] = (0..cells)
            .map(|idx| if idx % 7 == 0 { 0.6 } else { 0.0 })
            .collect();
        fixture.barriers = vec![8, 21];
        fixture.occupied = (0..cells).filter(|idx| idx % 3 == 1).collect();
        fixture.bites = (0..cells)
            .filter(|idx| idx % 4 == 2)
            .map(|idx| (idx, 0, 1 + (idx % 3) as u32))
            .collect();

        let mut seen = OracleReport::default();
        for (seed, tick) in [(3, 0), (5, 1), (8, 17)] {
            fixture.seed = seed;
            fixture.tick = tick;
            let report = assert_grow_matches_oracle(&fixture);
            seen.modified_delivery |= report.modified_delivery;
            seen.recovery_spawned |= report.recovery_spawned;
            seen.shared_target |= report.shared_target;
            seen.wrapped_delivery |= report.wrapped_delivery;
            seen.inhibited_cells += report.inhibited_cells;
            seen.suppressed_by_occupancy += report.suppressed_by_occupancy;
        }
        assert!(
            seen.modified_delivery,
            "a delivery must be reduced at its target"
        );
        assert!(seen.recovery_spawned, "recovery must spawn");
        assert!(seen.shared_target, "two sources must deliver to one target");
        assert!(
            seen.wrapped_delivery,
            "a delivery must cross a wrapped edge"
        );
        assert!(seen.inhibited_cells > 0, "inhibition must bind");
        assert!(seen.suppressed_by_occupancy > 0.0, "depletion must bind");
    }

    /// Two neighbours that can only pick the middle cell, whose own growth
    /// plus both contributions exceeds `max_density`.
    #[test]
    fn two_sources_overflowing_one_target_clamp_once() {
        let mut config = plain_growth_config(1);
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 1.0;
        config.shared.spread_threshold_ratio = 0.5;
        config.shared.max_density = 1.0;
        let mut fixture = OracleFixture::new(3, 1, WorldEdgeMode::Bounded, config);
        fixture.densities[0] = vec![0.8, 0.4, 0.8];
        // Cell 1 is below the threshold, so it spreads nowhere.
        let report = assert_grow_matches_oracle(&fixture);
        assert!(report.shared_target && report.clamp_bound);
        assert_eq!(report.expected[0][1], 1.0);
    }

    /// Narrow wrapped grids where a cell is two of its neighbour's
    /// neighbours, or its own neighbour: the source's candidate list keeps
    /// the repeats and the target counts the one contribution once.
    #[test]
    fn narrow_wrapped_grids_count_a_repeated_source_once() {
        for (width, height) in [(1, 1), (1, 2), (2, 1), (2, 2), (1, 5), (5, 1)] {
            let mut config = plain_growth_config(1);
            config.shared.growth_rate = 0.25;
            config.shared.spread_density_ratio = 0.5;
            config.shared.spread_threshold_ratio = 0.1;
            config.shared.max_density = 10.0;
            let mut fixture = OracleFixture::new(width, height, WorldEdgeMode::Wrap, config);
            let cells = fixture.cells();
            fixture.densities[0] = (0..cells).map(|idx| 1.0 + idx as f32 * 0.5).collect();
            let mut repeated = false;
            for seed in 0..16 {
                fixture.seed = seed;
                repeated |= assert_grow_matches_oracle(&fixture).repeated_candidate_delivery;
            }
            assert!(
                repeated,
                "{width}x{height}: no delivery to a repeated candidate"
            );
        }
    }

    /// A spreading cell with no passable neighbour sends nothing and keeps
    /// its own growth: a bounded 1x1 world and a wrapped cell walled in by barriers.
    #[test]
    fn a_cell_with_an_empty_candidate_list_keeps_only_its_own_growth() {
        let mut config = plain_growth_config(1);
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 1.0;
        config.shared.spread_threshold_ratio = 0.1;
        config.shared.max_density = 10.0;
        let mut single = OracleFixture::new(1, 1, WorldEdgeMode::Bounded, config.clone());
        single.densities[0] = vec![2.0];
        let report = assert_grow_matches_oracle(&single);
        assert!(report.empty_candidate_list);
        assert_eq!(report.expected[0][0], 3.0);

        let mut walled = OracleFixture::new(3, 3, WorldEdgeMode::Wrap, config);
        walled.densities[0][4] = 2.0;
        walled.barriers = vec![1, 3, 5, 7];
        let report = assert_grow_matches_oracle(&walled);
        assert!(report.empty_candidate_list);
        assert_eq!(report.expected[0][4], 3.0);
        assert_eq!(report.expected[0].iter().sum::<f32>(), 3.0);
    }

    /// A type that cannot grow loses no growth to inhibition, so the other
    /// type's penalty on its cells marks none of them inhibited.
    #[test]
    fn a_type_with_zero_growth_rate_marks_no_cell_inhibited() {
        let mut config = plain_growth_config(2);
        config.shared.max_density = 1.0;
        config.shared.spread_threshold_ratio = 0.5;
        config.types[0].growth_rate = Some(0.0);
        config.types[1].growth_inhibitor = 0.5;
        let mut fixture = OracleFixture::new(3, 1, WorldEdgeMode::Bounded, config);
        // Type 1 puts a positive penalty on both type-0 cells.
        fixture.densities[0] = vec![0.8, 0.4, 0.0];
        fixture.densities[1] = vec![0.6, 0.6, 0.0];

        let report = assert_grow_matches_oracle(&fixture);

        assert_eq!(report.inhibited_cells, 0);
    }

    /// A dense cell whose inhibition penalty consumes all its fertility has
    /// no growth to share: it spreads nothing, so its neighbour records no
    /// suppressed spread.
    #[test]
    fn a_fully_inhibited_dense_cell_spreads_nothing() {
        let mut config = plain_growth_config(2);
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 0.5;
        config.shared.spread_threshold_ratio = 0.5;
        config.shared.max_density = 1.0;
        config.types[1].growth_inhibitor = 1.0;
        let mut fixture = OracleFixture::new(2, 1, WorldEdgeMode::Bounded, config);
        // Type 1 at full density makes type 0's penalty at cell 0 equal its fertility.
        fixture.densities[0] = vec![0.9, 0.0];
        fixture.densities[1] = vec![1.0, 0.0];

        let report = assert_grow_matches_oracle(&fixture);

        assert_eq!(report.expected[0], vec![0.9, 0.0]);
        assert!((report.suppressed_by_inhibition - 0.45).abs() < 1e-6);
    }

    /// With a zero spread ratio a delivery carries no growth, so the
    /// target's inhibition penalty suppresses nothing and marks no cell.
    #[test]
    fn a_zero_spread_ratio_delivery_marks_no_target_inhibited() {
        let mut config = plain_growth_config(2);
        config.shared.growth_rate = 0.5;
        config.shared.spread_density_ratio = 0.0;
        config.shared.spread_threshold_ratio = 0.5;
        config.shared.max_density = 1.0;
        config.types[1].growth_inhibitor = 0.5;
        let mut fixture = OracleFixture::new(2, 1, WorldEdgeMode::Bounded, config);
        // Type 0 spreads from cell 0 onto cell 1, where type 1 puts a penalty.
        fixture.densities[0] = vec![0.9, 0.0];
        fixture.densities[1] = vec![0.0, 0.6];

        let report = assert_grow_matches_oracle(&fixture);

        assert_eq!(report.inhibited_cells, 0);
    }

    /// A recovery spawn on a cell whose habitat fertility is zero adds
    /// nothing, and the other type's penalty there suppresses nothing and
    /// marks no cell, because there was no growth to lose.
    #[test]
    fn a_recovery_spawn_on_zero_fertility_records_no_inhibition() {
        let mut config = plain_growth_config(2);
        config.fertility.enabled = true;
        config.fertility.min_fertility = 0.0;
        config.fertility.max_fertility = 1.0;
        config.annealing.enabled = false;
        config.shared.growth_rate = 0.5;
        config.shared.max_density = 1.0;
        config.shared.recovery_floor_ratio = 1.0;
        config.types[0].recovery_spawn_rate = Some(1.0);
        config.types[1].growth_inhibitor = 1.0;
        let catalog = OrdinaryFoodCatalog::new(&config);
        let (barren, inhibitor) = (OrdinaryFoodTypeId::new(0), OrdinaryFoodTypeId::new(1));
        let cell = Position::new(0, 0);
        let mut state = OrdinaryFoodState::new(1, 1, catalog.len());
        // Raw -1 maps to min_fertility 0 for the empty type 0.
        state.set_fertility_grid(barren, Grid::new(1, 1, -1.0));
        state.set_fertility_grid(inhibitor, Grid::new(1, 1, 1.0));
        state.set_food_type_density(cell, inhibitor, 1.0);

        let summary = grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut OccupancyDepletionLayer::new(1, 1),
            &mut GrowthScratch::default(),
            &barrier_grid(1, 1),
            &occupancy_grid::<()>(1, 1),
            WorldEdgeMode::Bounded,
            0,
            FoodStreamKey::from_run_seed(3),
        );

        assert_eq!(state.food_at_type(cell, barren), 0.0);
        assert_eq!(summary.cells_with_type_inhibition, 0);
        assert_eq!(summary.growth_suppressed_by_type_inhibition, 0.0);
    }

    /// With a zero deposit, occupancy adds no depletion and no cell counts
    /// as an occupied depletion cell.
    #[test]
    fn a_zero_deposit_counts_no_occupied_depletion_cell() {
        let config = OccupancyDepletionConfig {
            enabled: true,
            deposit_per_occupied_tick: 0.0,
        };
        let mut layer = OccupancyDepletionLayer::new(2, 1);
        let mut occupancy = occupancy_grid::<()>(2, 1);
        occupancy.set(0, 0, Some(()));

        let summary = layer.recover_and_deposit(&barrier_grid(2, 1), &occupancy, &config);

        assert_eq!(
            summary,
            DepletionTickSummary {
                mean_depletion: 0.0,
                occupied_cells: 0,
            }
        );
    }

    #[test]
    fn growth_scratch_debug_prints_its_cell_count_not_its_buffers() {
        let mut scratch = GrowthScratch::default();
        scratch.fit(6);

        assert_eq!(format!("{scratch:?}"), "GrowthScratch { cells: 6, .. }");
    }

    /// On a uniform saturated wrapped patch every cell spreads; the keyed
    /// picks share the four directions (fixed seed, a count bound, not a
    /// distribution proof).
    #[test]
    fn spread_targets_on_a_saturated_patch_share_the_four_directions() {
        const SIDE: u16 = 64;
        let mut config = plain_growth_config(1);
        config.shared.max_density = 1.0;
        config.shared.spread_threshold_ratio = 0.5;
        config.shared.growth_rate = 0.1;
        let catalog = OrdinaryFoodCatalog::new(&config);
        let mut state = OrdinaryFoodState::new(SIDE, SIDE, catalog.len());
        for y in 0..SIDE {
            for x in 0..SIDE {
                state.set_food_type_density(Position::new(x, y), OrdinaryFoodTypeId::new(0), 1.0);
            }
        }
        let mut scratch = GrowthScratch::default();
        grow(
            &mut state,
            &catalog,
            &config.shared,
            &config,
            &mut OccupancyDepletionLayer::new(SIDE, SIDE),
            &mut scratch,
            &barrier_grid(SIDE, SIDE),
            &occupancy_grid::<()>(SIDE, SIDE),
            WorldEdgeMode::Wrap,
            3,
            FoodStreamKey::from_run_seed(11),
        );
        let mut by_direction = [0u32; 4];
        for (idx, pick) in scratch.picks().enumerate() {
            let target = pick.expect("every saturated cell spreads");
            let source = Position::new(idx as u16 % SIDE, idx as u16 / SIDE);
            let direction = SPREAD_DIRECTIONS
                .into_iter()
                .position(|dir| {
                    resolve_neighbor_static(source, dir, SIDE, SIDE, WorldEdgeMode::Wrap)
                        .is_some_and(|p| {
                            usize::from(p.y) * usize::from(SIDE) + usize::from(p.x) == target
                        })
                })
                .expect("the pick is a neighbour");
            by_direction[direction] += 1;
        }
        let cells = u32::from(SIDE) * u32::from(SIDE);
        for (direction, count) in by_direction.into_iter().enumerate() {
            assert!(
                (cells * 22 / 100..=cells * 28 / 100).contains(&count),
                "direction {direction}: {count} of {cells} ({by_direction:?})"
            );
        }
    }

    proptest::proptest! {
        /// With no clamp binding and no modifier, one growth pass conserves
        /// `Σ source × (1 + g) + Σ_{source ≥ threshold} source × g × ratio`
        /// whatever targets the spreading cells draw.
        #[test]
        fn growth_pass_total_keeps_every_spread_contribution(
            width in 2u16..=6,
            height in 2u16..=6,
            densities in proptest::collection::vec(0.0f32..=10.0, 36),
            growth_rate in 0.01f32..=1.0,
            spread_density_ratio in 0.0f32..=1.0,
            bounded in proptest::bool::ANY,
            seed in proptest::num::u64::ANY,
        ) {
            let mut config = plain_growth_config(1);
            config.shared.growth_rate = growth_rate;
            config.shared.spread_density_ratio = spread_density_ratio;
            config.shared.max_density = 1000.0;
            config.shared.spread_threshold_ratio = 0.005;
            let threshold = f64::from(config.shared.max_density * config.shared.spread_threshold_ratio);
            let catalog = OrdinaryFoodCatalog::new(&config);
            let type_idx = OrdinaryFoodTypeId::new(0);
            let mut state = OrdinaryFoodState::new(width, height, catalog.len());
            let mut expected = 0.0f64;
            for y in 0..height {
                for x in 0..width {
                    let source = densities[usize::from(y * width + x)];
                    state.set_food_type_density(Position::new(x, y), type_idx, source);
                    let (source, g) = (f64::from(source), f64::from(growth_rate));
                    expected += source * (1.0 + g);
                    if source >= threshold {
                        expected += source * g * f64::from(spread_density_ratio);
                    }
                }
            }
            let edge_mode = if bounded { WorldEdgeMode::Bounded } else { WorldEdgeMode::Wrap };

            grow_once(
                &mut state,
                &catalog,
                &config,
                &barrier_grid(width, height),
                edge_mode,
                seed,
            );

            let total: f64 = (0..height)
                .flat_map(|y| (0..width).map(move |x| Position::new(x, y)))
                .map(|pos| f64::from(state.food_at_type(pos, type_idx)))
                .sum();
            proptest::prop_assert!(
                (total - expected).abs() <= 1e-5 * expected.max(1.0),
                "total {total} vs expected {expected}"
            );
        }
    }
}
