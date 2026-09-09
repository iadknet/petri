mod catalog;
mod ecology;
mod state;

use rand::Rng;

use crate::config::{
    FoodConfig, FoodResourceConfig, FoodTypeConfig, OrdinaryFoodTypeId, WorldEdgeMode,
};
use crate::contracts::Position;
use crate::kernel::Grid;

pub use self::catalog::{OrdinaryFoodCatalog, OrdinaryFoodTypeEntry};
use self::ecology::{
    apply_config_transition, grow, seed_density, seed_fertility_maps, OccupancyDepletionLayer,
};
pub use self::ecology::{FoodGrowthSummary, FoodTypeTelemetry};
use self::state::OrdinaryFoodState;

#[derive(Debug)]
pub struct FoodResource {
    config: FoodConfig,
    catalog: OrdinaryFoodCatalog,
    state: OrdinaryFoodState,
    occupancy_depletion: OccupancyDepletionLayer,
    edge_mode: WorldEdgeMode,
    claim_scratch: Vec<Vec<(OrdinaryFoodTypeId, f32)>>,
    owner_snapshot: Vec<Option<OrdinaryFoodTypeId>>,
    density_snapshot: Vec<f32>,
    fertility_seed: Option<u64>,
}

impl FoodResource {
    #[must_use]
    pub fn new(width: u16, height: u16, config: FoodConfig, edge_mode: WorldEdgeMode) -> Self {
        let catalog = OrdinaryFoodCatalog::new(&config);
        let type_count = catalog.len();
        let total_cells = width as usize * height as usize;

        Self {
            config,
            catalog,
            state: OrdinaryFoodState::new(width, height, type_count),
            occupancy_depletion: OccupancyDepletionLayer::new(width, height),
            edge_mode,
            claim_scratch: Vec::with_capacity(total_cells),
            owner_snapshot: Vec::with_capacity(total_cells),
            density_snapshot: Vec::with_capacity(total_cells),
            fertility_seed: None,
        }
    }

    #[must_use]
    pub fn food_at(&self, pos: Position) -> f32 {
        self.state.density_at(pos)
    }

    #[must_use]
    pub fn food_at_type(&self, pos: Position, type_idx: OrdinaryFoodTypeId) -> f32 {
        if !self.catalog.is_valid(type_idx) {
            return 0.0;
        }
        self.state.food_at_type(pos, type_idx)
    }

    #[must_use]
    pub fn dominant_food_type_at(&self, pos: Position) -> Option<OrdinaryFoodTypeId> {
        let mut best: Option<(OrdinaryFoodTypeId, f32)> = None;
        for entry in self.catalog.entries() {
            let density = self.state.food_at_type(pos, entry.id);
            if density <= 0.0 {
                continue;
            }
            match best {
                Some((_, best_density)) if best_density >= density => {}
                _ => best = Some((entry.id, density)),
            }
        }
        best.map(|(type_idx, _)| type_idx)
    }

    #[must_use]
    pub fn consume(&mut self, pos: Position) -> f32 {
        self.state.consume_any(pos)
    }

    #[must_use]
    pub fn consume_type(&mut self, pos: Position, type_idx: OrdinaryFoodTypeId) -> f32 {
        if !self.catalog.is_valid(type_idx) {
            return 0.0;
        }
        self.state.consume_type(pos, type_idx)
    }

    pub fn set_food(&mut self, pos: Position, value: f32) {
        self.set_food_type(pos, OrdinaryFoodTypeId::default(), value);
    }

    pub fn set_food_type(&mut self, pos: Position, type_idx: OrdinaryFoodTypeId, value: f32) {
        if !self.catalog.is_valid(type_idx) {
            return;
        }
        let density = value.clamp(0.0, self.config.shared.max_density.max(0.0));
        self.state.set_food_type_density(pos, type_idx, density);
    }

    #[must_use]
    pub fn total_food(&self) -> f32 {
        self.state.total_food()
    }

    #[must_use]
    pub fn total_food_by_type(&self, type_idx: OrdinaryFoodTypeId) -> f32 {
        if !self.catalog.is_valid(type_idx) {
            return 0.0;
        }
        self.state.total_food_by_type(type_idx)
    }

    #[must_use]
    pub fn fertility(&self) -> &Grid<f32> {
        self.state
            .fertility_grid(OrdinaryFoodTypeId::default())
            .expect("primary food type fertility grid must exist")
    }

    #[must_use]
    pub fn fertility_for_type(&self, type_idx: OrdinaryFoodTypeId) -> Option<&Grid<f32>> {
        if !self.catalog.is_valid(type_idx) {
            return None;
        }
        self.state.fertility_grid(type_idx)
    }

    #[must_use]
    pub fn occupancy_depletion(&self) -> &Grid<f32> {
        self.occupancy_depletion.grid()
    }

    #[must_use]
    pub fn config(&self) -> &FoodResourceConfig {
        &self.config.shared
    }

    #[must_use]
    pub fn full_config(&self) -> &FoodConfig {
        &self.config
    }

    #[must_use]
    pub fn food_types(&self) -> &[OrdinaryFoodTypeEntry] {
        self.catalog.entries()
    }

    #[must_use]
    pub fn food_type_configs(&self) -> &[FoodTypeConfig] {
        &self.config.types
    }

    #[must_use]
    pub fn food_type_count(&self) -> usize {
        self.catalog.len()
    }

    pub fn for_each_food_cell(&self, visitor: impl FnMut(u16, u16, OrdinaryFoodTypeId, f32)) {
        self.state.for_each_food_cell(visitor);
    }

    pub fn apply_config_transition(&mut self, next: FoodConfig) {
        let next_catalog = OrdinaryFoodCatalog::new(&next);
        let catalog_shape_changed = self.catalog != next_catalog;
        apply_config_transition(
            &mut self.config.shared,
            &next,
            &mut self.state,
            &mut self.occupancy_depletion,
            catalog_shape_changed,
        );
        self.config.types = next.types;
        self.config.fertility = next.fertility;
        self.config.annealing = next.annealing;
        self.catalog = next_catalog;
        if let Some(world_seed) = self.fertility_seed {
            seed_fertility_maps(&mut self.state, &self.catalog, &self.config, world_seed);
        }
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.state.width()
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.state.height()
    }

    pub fn seed_fertility(&mut self, world_seed: u64) {
        self.fertility_seed = Some(world_seed);
        seed_fertility_maps(&mut self.state, &self.catalog, &self.config, world_seed);
    }

    pub fn grow<T: Clone>(
        &mut self,
        barriers: &Grid<bool>,
        occupancy: &Grid<Option<T>>,
        tick: u64,
        rng: &mut impl Rng,
    ) -> FoodGrowthSummary {
        grow(
            &mut self.state,
            &self.catalog,
            &self.config.shared,
            &self.config,
            &mut self.occupancy_depletion,
            &mut self.claim_scratch,
            &mut self.owner_snapshot,
            &mut self.density_snapshot,
            barriers,
            occupancy,
            self.edge_mode,
            tick,
            rng,
        )
    }

    pub fn seed_density(&mut self, barriers: &Grid<bool>, rng: &mut impl Rng) {
        seed_density(
            &mut self.state,
            &self.catalog,
            &self.config,
            &mut self.occupancy_depletion,
            &mut self.claim_scratch,
            barriers,
            rng,
        );
    }
}
