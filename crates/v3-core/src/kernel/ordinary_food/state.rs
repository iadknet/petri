use crate::config::{GrazingConfig, OrdinaryFoodTypeId};
use crate::contracts::Position;
use crate::kernel::Grid;

use super::grazing::GrazingLayer;

#[derive(Debug)]
pub struct OrdinaryFoodState {
    width: u16,
    height: u16,
    density_by_type: Vec<Grid<f32>>,
    fertility_by_type: Vec<Grid<f32>>,
    grazing: GrazingLayer,
}

impl OrdinaryFoodState {
    #[must_use]
    pub fn new(width: u16, height: u16, type_count: usize) -> Self {
        Self {
            width,
            height,
            density_by_type: (0..type_count)
                .map(|_| Grid::new(width, height, 0.0))
                .collect(),
            fertility_by_type: (0..type_count)
                .map(|_| Grid::new(width, height, 0.0))
                .collect(),
            grazing: GrazingLayer::new(width, height, type_count),
        }
    }

    pub fn resize_type_storage(&mut self, type_count: usize) {
        let width = self.width;
        let height = self.height;
        self.density_by_type = (0..type_count)
            .map(|_| Grid::new(width, height, 0.0))
            .collect();
        self.fertility_by_type = (0..type_count)
            .map(|_| Grid::new(width, height, 0.0))
            .collect();
        self.grazing.resize_type_storage(type_count);
    }

    #[must_use]
    pub fn grazing(&self) -> &GrazingLayer {
        &self.grazing
    }

    pub fn grazing_mut(&mut self) -> &mut GrazingLayer {
        &mut self.grazing
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn clear_density(&mut self) {
        for grid in &mut self.density_by_type {
            for y in 0..self.height {
                for x in 0..self.width {
                    grid.set(x, y, 0.0);
                }
            }
        }
    }

    pub fn clamp_density_to_max(&mut self, max_density: f32) {
        let capped_max = max_density.max(0.0);
        for grid in &mut self.density_by_type {
            for y in 0..self.height {
                for x in 0..self.width {
                    let density = *grid.get(x, y);
                    if density > capped_max {
                        grid.set(x, y, capped_max);
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn density_at(&self, pos: Position) -> f32 {
        self.density_by_type
            .iter()
            .map(|grid| *grid.get(pos.x, pos.y))
            .sum()
    }

    #[must_use]
    pub fn food_at_type(&self, pos: Position, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.density_by_type
            .get(usize::from(type_idx.get()))
            .map(|grid| *grid.get(pos.x, pos.y))
            .unwrap_or(0.0)
    }

    pub fn set_food_type_density(
        &mut self,
        pos: Position,
        type_idx: OrdinaryFoodTypeId,
        density: f32,
    ) {
        if let Some(grid) = self.density_by_type.get_mut(usize::from(type_idx.get())) {
            grid.set(pos.x, pos.y, density);
        }
    }

    /// Remove every type's density at the cell; each type that had a positive
    /// amount is bitten so no caller bypasses the grazing rule.
    #[must_use]
    pub fn consume_any(&mut self, pos: Position, grazing: &GrazingConfig) -> f32 {
        let mut amount = 0.0;
        for (type_index, grid) in self.density_by_type.iter_mut().enumerate() {
            let removed = *grid.get(pos.x, pos.y);
            if removed <= 0.0 {
                continue;
            }
            amount += removed;
            grid.set(pos.x, pos.y, 0.0);
            let Ok(type_raw) = u16::try_from(type_index) else {
                continue;
            };
            self.grazing
                .bite(pos.x, pos.y, OrdinaryFoodTypeId::new(type_raw), grazing);
        }
        amount
    }

    /// Remove one type's density at the cell. A positive removal is one bite;
    /// an empty cell is not.
    #[must_use]
    pub fn consume_type(
        &mut self,
        pos: Position,
        type_idx: OrdinaryFoodTypeId,
        grazing: &GrazingConfig,
    ) -> f32 {
        let Some(grid) = self.density_by_type.get_mut(usize::from(type_idx.get())) else {
            return 0.0;
        };
        let amount = *grid.get(pos.x, pos.y);
        if amount > 0.0 {
            grid.set(pos.x, pos.y, 0.0);
            self.grazing.bite(pos.x, pos.y, type_idx, grazing);
        }
        amount
    }

    #[must_use]
    pub fn total_food(&self) -> f32 {
        self.density_by_type
            .iter()
            .map(|grid| grid.as_slice().iter().sum::<f32>())
            .sum()
    }

    #[must_use]
    pub fn total_food_by_type(&self, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.density_by_type
            .get(usize::from(type_idx.get()))
            .map(|grid| grid.as_slice().iter().sum())
            .unwrap_or(0.0)
    }

    pub fn set_fertility_grid(&mut self, type_idx: OrdinaryFoodTypeId, grid: Grid<f32>) {
        let idx = usize::from(type_idx.get());
        if let Some(slot) = self.fertility_by_type.get_mut(idx) {
            *slot = grid;
        }
    }

    #[must_use]
    pub fn fertility_grid(&self, type_idx: OrdinaryFoodTypeId) -> Option<&Grid<f32>> {
        self.fertility_by_type.get(usize::from(type_idx.get()))
    }

    pub fn for_each_food_cell(&self, mut visitor: impl FnMut(u16, u16, OrdinaryFoodTypeId, f32)) {
        let width = usize::from(self.width);
        for (type_index, grid) in self.density_by_type.iter().enumerate() {
            let Ok(type_raw) = u16::try_from(type_index) else {
                break;
            };
            let type_idx = OrdinaryFoodTypeId::new(type_raw);
            for (flat_idx, density) in grid.as_slice().iter().copied().enumerate() {
                if density <= 0.0 {
                    continue;
                }
                let x = (flat_idx % width) as u16;
                let y = (flat_idx / width) as u16;
                visitor(x, y, type_idx, density);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16) -> Position {
        Position::new(x, y)
    }

    #[test]
    fn density_is_summed_across_types() {
        let mut state = OrdinaryFoodState::new(4, 3, 3);
        state.set_food_type_density(pos(1, 2), OrdinaryFoodTypeId::new(0), 1.25);
        state.set_food_type_density(pos(1, 2), OrdinaryFoodTypeId::new(1), 2.5);

        assert_eq!(state.width(), 4);
        assert_eq!(state.height(), 3);
        assert_eq!(state.density_at(pos(1, 2)), 3.75);
        assert_eq!(
            state.food_at_type(pos(1, 2), OrdinaryFoodTypeId::new(0)),
            1.25
        );
        assert_eq!(
            state.food_at_type(pos(1, 2), OrdinaryFoodTypeId::new(1)),
            2.5
        );
        assert_eq!(
            state.food_at_type(pos(1, 2), OrdinaryFoodTypeId::new(9)),
            0.0
        );
        assert_eq!(state.total_food(), 3.75);
        assert_eq!(state.total_food_by_type(OrdinaryFoodTypeId::new(0)), 1.25);
        assert_eq!(state.total_food_by_type(OrdinaryFoodTypeId::new(1)), 2.5);
        assert_eq!(state.total_food_by_type(OrdinaryFoodTypeId::new(9)), 0.0);
    }

    #[test]
    fn consume_any_clears_every_type_plane() {
        let mut state = OrdinaryFoodState::new(2, 2, 2);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), 1.0);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(1), 2.0);

        assert_eq!(state.consume_any(pos(0, 0), &GrazingConfig::default()), 3.0);
        assert_eq!(state.density_at(pos(0, 0)), 0.0);
        assert_eq!(
            state.food_at_type(pos(0, 0), OrdinaryFoodTypeId::new(0)),
            0.0
        );
        assert_eq!(
            state.food_at_type(pos(0, 0), OrdinaryFoodTypeId::new(1)),
            0.0
        );
    }

    #[test]
    fn consume_type_only_clears_matching_plane() {
        let mut state = OrdinaryFoodState::new(2, 2, 2);
        state.set_food_type_density(pos(1, 1), OrdinaryFoodTypeId::new(0), 1.5);
        state.set_food_type_density(pos(1, 1), OrdinaryFoodTypeId::new(1), 2.5);

        assert_eq!(
            state.consume_type(
                pos(1, 1),
                OrdinaryFoodTypeId::new(0),
                &GrazingConfig::default()
            ),
            1.5
        );
        assert_eq!(
            state.food_at_type(pos(1, 1), OrdinaryFoodTypeId::new(0)),
            0.0
        );
        assert_eq!(
            state.food_at_type(pos(1, 1), OrdinaryFoodTypeId::new(1)),
            2.5
        );
        assert_eq!(state.density_at(pos(1, 1)), 2.5);
    }

    #[test]
    fn consume_type_bites_only_the_removed_type_and_only_when_positive() {
        let grazing = GrazingConfig::default();
        let mut state = OrdinaryFoodState::new(2, 2, 2);
        state.set_food_type_density(pos(1, 1), OrdinaryFoodTypeId::new(0), 1.5);

        assert_eq!(
            state.consume_type(pos(1, 1), OrdinaryFoodTypeId::new(0), &grazing),
            1.5
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(1, 1, OrdinaryFoodTypeId::new(0)),
            0.5
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(1, 1, OrdinaryFoodTypeId::new(1)),
            1.0
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(0, 0, OrdinaryFoodTypeId::new(0)),
            1.0
        );

        // The cell is empty now: no bite.
        assert_eq!(
            state.consume_type(pos(1, 1), OrdinaryFoodTypeId::new(0), &grazing),
            0.0
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(1, 1, OrdinaryFoodTypeId::new(0)),
            0.5
        );
        assert_eq!(
            state.consume_type(pos(1, 1), OrdinaryFoodTypeId::new(1), &grazing),
            0.0
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(1, 1, OrdinaryFoodTypeId::new(1)),
            1.0
        );
    }

    #[test]
    fn consume_any_bites_each_type_it_removed() {
        let grazing = GrazingConfig::default();
        let mut state = OrdinaryFoodState::new(2, 2, 3);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), 1.0);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(2), 2.0);

        assert_eq!(state.consume_any(pos(0, 0), &grazing), 3.0);
        assert_eq!(
            state
                .grazing()
                .modifier_at(0, 0, OrdinaryFoodTypeId::new(0)),
            0.5
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(0, 0, OrdinaryFoodTypeId::new(1)),
            1.0
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(0, 0, OrdinaryFoodTypeId::new(2)),
            0.5
        );
    }

    #[test]
    fn resize_type_storage_resets_grazing_to_one() {
        let grazing = GrazingConfig::default();
        let mut state = OrdinaryFoodState::new(2, 2, 1);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), 1.0);
        let _ = state.consume_type(pos(0, 0), OrdinaryFoodTypeId::new(0), &grazing);
        state.resize_type_storage(2);
        assert_eq!(
            state
                .grazing()
                .modifier_at(0, 0, OrdinaryFoodTypeId::new(0)),
            1.0
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(0, 0, OrdinaryFoodTypeId::new(1)),
            1.0
        );
    }

    #[test]
    fn clear_density_resets_all_type_planes() {
        let mut state = OrdinaryFoodState::new(3, 2, 2);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), 1.0);
        state.set_food_type_density(pos(2, 1), OrdinaryFoodTypeId::new(1), 4.0);

        state.clear_density();

        assert_eq!(state.total_food(), 0.0);
        assert_eq!(state.density_at(pos(0, 0)), 0.0);
        assert_eq!(state.density_at(pos(2, 1)), 0.0);
    }

    #[test]
    fn for_each_food_cell_visits_only_non_zero_cells() {
        let mut state = OrdinaryFoodState::new(2, 2, 2);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), 0.25);
        state.set_food_type_density(pos(1, 1), OrdinaryFoodTypeId::new(1), 0.75);

        let mut cells = Vec::new();
        state.for_each_food_cell(|x, y, type_idx, density| {
            cells.push((x, y, type_idx.get(), density));
        });
        cells.sort_by_key(|(x, y, type_idx, _)| (*type_idx, *x, *y));

        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0], (0, 0, 0, 0.25));
        assert_eq!(cells[1], (1, 1, 1, 0.75));
    }

    #[test]
    fn clamp_density_to_max_caps_all_type_planes() {
        let mut state = OrdinaryFoodState::new(2, 1, 2);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), 0.9);
        state.set_food_type_density(pos(1, 0), OrdinaryFoodTypeId::new(1), 0.6);

        state.clamp_density_to_max(0.5);

        assert_eq!(
            state.food_at_type(pos(0, 0), OrdinaryFoodTypeId::new(0)),
            0.5
        );
        assert_eq!(
            state.food_at_type(pos(1, 0), OrdinaryFoodTypeId::new(1)),
            0.5
        );
    }
}
