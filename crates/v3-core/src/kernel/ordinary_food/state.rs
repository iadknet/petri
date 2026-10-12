use crate::config::{GrazingConfig, OrdinaryFoodTypeId};
use crate::contracts::Position;
use crate::kernel::Grid;

use super::grazing::{GrazingLayer, FULL_BITE};

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

    /// One type's density grid as a row-major slice.
    #[must_use]
    pub fn density_slice(&self, type_idx: OrdinaryFoodTypeId) -> Option<&[f32]> {
        self.density_by_type
            .get(usize::from(type_idx.get()))
            .map(Grid::as_slice)
    }

    /// One type's density grid as a mutable row-major slice.
    pub fn density_slice_mut(&mut self, type_idx: OrdinaryFoodTypeId) -> Option<&mut [f32]> {
        self.density_by_type
            .get_mut(usize::from(type_idx.get()))
            .map(Grid::as_mut_slice)
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
            self.grazing.bite(
                pos.x,
                pos.y,
                OrdinaryFoodTypeId::new(type_raw),
                FULL_BITE,
                grazing,
            );
        }
        amount
    }

    /// Eat one type's density at the cell, leaving the share `leave` (in
    /// `[0, 1]`) in place (T02.F08), and return the amount eaten,
    /// `amount × (1 − leave)`. A positive eat is one bite of share
    /// `1 − leave`; an empty cell, `leave` 1.0, or an eat that rounds to zero
    /// changes nothing and returns 0.0. `leave` 0.0 clears the cell with a
    /// full bite.
    #[must_use]
    pub fn consume_type(
        &mut self,
        pos: Position,
        type_idx: OrdinaryFoodTypeId,
        leave: f32,
        grazing: &GrazingConfig,
    ) -> f32 {
        let Some(grid) = self.density_by_type.get_mut(usize::from(type_idx.get())) else {
            return 0.0;
        };
        let amount = *grid.get(pos.x, pos.y);
        if amount <= 0.0 {
            return 0.0;
        }
        let share = 1.0 - leave;
        let eaten = amount * share;
        if eaten <= 0.0 {
            return 0.0;
        }
        grid.set(pos.x, pos.y, amount - eaten);
        self.grazing.bite(pos.x, pos.y, type_idx, share, grazing);
        eaten
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
    use super::super::grazing::bite_modifier;
    use super::*;
    use proptest::strategy::{Just, Strategy};

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
                0.0,
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
            state.consume_type(pos(1, 1), OrdinaryFoodTypeId::new(0), 0.0, &grazing),
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
            state.consume_type(pos(1, 1), OrdinaryFoodTypeId::new(0), 0.0, &grazing),
            0.0
        );
        assert_eq!(
            state
                .grazing()
                .modifier_at(1, 1, OrdinaryFoodTypeId::new(0)),
            0.5
        );
        assert_eq!(
            state.consume_type(pos(1, 1), OrdinaryFoodTypeId::new(1), 0.0, &grazing),
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
        let _ = state.consume_type(pos(0, 0), OrdinaryFoodTypeId::new(0), 0.0, &grazing);
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

    fn live_grazing(factor: f32, floor: f32) -> GrazingConfig {
        GrazingConfig {
            enabled: true,
            factor,
            floor,
            recovery_ticks: 1000,
        }
    }

    /// One cell of type 0 holding `amount`, its modifier pre-set to
    /// `modifier` by a bite of factor `modifier` from 1.0.
    fn cell_with(amount: f32, modifier: f32) -> OrdinaryFoodState {
        let mut state = OrdinaryFoodState::new(1, 1, 1);
        state.set_food_type_density(pos(0, 0), OrdinaryFoodTypeId::new(0), amount);
        state.grazing_mut().bite(
            0,
            0,
            OrdinaryFoodTypeId::new(0),
            FULL_BITE,
            &live_grazing(modifier, 0.0),
        );
        state
    }

    fn food(state: &OrdinaryFoodState) -> f32 {
        state.food_at_type(pos(0, 0), OrdinaryFoodTypeId::new(0))
    }

    fn modifier(state: &OrdinaryFoodState) -> f32 {
        state
            .grazing()
            .modifier_at(0, 0, OrdinaryFoodTypeId::new(0))
    }

    fn eat(state: &mut OrdinaryFoodState, leave: f32, grazing: &GrazingConfig) -> f32 {
        state.consume_type(pos(0, 0), OrdinaryFoodTypeId::new(0), leave, grazing)
    }

    /// T02.F08: `leave` 0.0 is the pre-feature consume bit for bit: the whole
    /// amount eaten, the cell cleared, one full bite.
    #[test]
    fn consume_type_leave_zero_is_the_full_bite() {
        let grazing = live_grazing(0.5, 0.05);
        let mut state = cell_with(1.3, 0.7);
        assert_eq!(eat(&mut state, 0.0, &grazing).to_bits(), 1.3f32.to_bits());
        assert_eq!(food(&state).to_bits(), 0.0f32.to_bits());
        assert_eq!(
            modifier(&state).to_bits(),
            (0.7f32 * 0.5).max(0.05).to_bits()
        );
    }

    /// T02.F08: `leave` 1.0 eats nothing: cell and modifier untouched.
    #[test]
    fn consume_type_leave_one_is_a_failed_eat() {
        let mut state = cell_with(1.3, 0.7);
        assert_eq!(eat(&mut state, 1.0, &live_grazing(0.5, 0.05)), 0.0);
        assert_eq!(food(&state), 1.3);
        assert_eq!(modifier(&state), 0.7);
    }

    /// T02.F08: a mid `leave` eats `amount × (1 − leave)`, keeps the rest
    /// and bites by `factor^(1 − leave)`.
    #[test]
    fn consume_type_mid_leave_splits_the_cell_and_bites_by_the_share() {
        let mut state = cell_with(2.0, 1.0);
        let eaten = eat(&mut state, 0.75, &live_grazing(0.25, 0.0));
        assert_eq!(eaten, 0.5);
        assert_eq!(food(&state), 1.5);
        assert!((modifier(&state) - 0.25f32.powf(0.25)).abs() < 1e-6);
    }

    /// T02.F08: a positive subnormal density whose eaten share rounds to zero
    /// is a failed eat with the cell untouched.
    #[test]
    fn consume_type_eat_rounding_to_zero_is_a_failed_eat() {
        let tiny = f32::from_bits(1);
        let mut state = cell_with(tiny, 1.0);
        assert_eq!(eat(&mut state, 0.75, &live_grazing(0.5, 0.0)), 0.0);
        assert_eq!(food(&state).to_bits(), tiny.to_bits());
        assert_eq!(modifier(&state), 1.0);
    }

    /// T02.F08: a positive `leave` whose complement rounds to 1.0 is a full
    /// bite.
    #[test]
    fn consume_type_tiny_leave_is_a_full_bite() {
        let leave = f32::EPSILON / 4.0;
        assert_eq!(1.0 - leave, 1.0);
        let mut state = cell_with(1.3, 0.7);
        assert_eq!(eat(&mut state, leave, &live_grazing(0.5, 0.05)), 1.3);
        assert_eq!(food(&state), 0.0);
        assert_eq!(modifier(&state).to_bits(), (0.7f32 * 0.5).to_bits());
    }

    /// T02.F08: with grazing disabled the split still applies, with no bite.
    #[test]
    fn consume_type_disabled_grazing_splits_without_a_bite() {
        let disabled = GrazingConfig {
            enabled: false,
            ..live_grazing(0.5, 0.05)
        };
        let mut state = cell_with(2.0, 0.7);
        assert_eq!(eat(&mut state, 0.5, &disabled), 1.0);
        assert_eq!(food(&state), 1.0);
        assert_eq!(modifier(&state), 0.7);
    }

    /// T02.F08: a stored modifier below a live-edited floor clamps up to it.
    #[test]
    fn consume_type_partial_bite_below_the_floor_clamps_to_the_floor() {
        let mut state = cell_with(2.0, 0.1);
        let _ = eat(&mut state, 0.5, &live_grazing(0.9, 0.2));
        assert_eq!(modifier(&state), 0.2);
    }

    proptest::proptest! {
        /// T02.F08 consume split: the eaten and left parts are non-negative,
        /// sum to the amount within an ulp-scale bound, and the modifier is the
        /// bite of share `1 − leave` (none when nothing is eaten), within
        /// `[floor, m]` and never below the full bite's.
        #[test]
        fn consume_type_splits_the_cell_and_bites_by_the_eaten_share(
            amount in proptest::num::f32::POSITIVE | proptest::num::f32::NORMAL | proptest::num::f32::SUBNORMAL,
            leave in 0.0f32..=1.0,
            factor in 0.0f32..=1.0,
            (floor, m) in (0.0f32..=1.0).prop_flat_map(|floor| (Just(floor), floor..=1.0f32)),
        ) {
            let grazing = live_grazing(factor, floor);
            let mut state = cell_with(amount, m);
            let eaten = eat(&mut state, leave, &grazing);
            let left = food(&state);
            proptest::prop_assert!((0.0..=amount).contains(&eaten));
            proptest::prop_assert!(left >= 0.0);
            proptest::prop_assert!(((eaten + left) - amount).abs() <= f32::EPSILON * amount);
            let bitten = modifier(&state);
            if eaten > 0.0 {
                proptest::prop_assert_eq!(
                    bitten.to_bits(),
                    bite_modifier(m, factor, floor, 1.0 - leave).to_bits()
                );
            } else {
                proptest::prop_assert_eq!(left.to_bits(), amount.to_bits());
                proptest::prop_assert_eq!(bitten.to_bits(), m.to_bits());
            }
            proptest::prop_assert!(bitten >= floor && bitten <= m);
            proptest::prop_assert!(
                bitten >= bite_modifier(m, factor, floor, FULL_BITE)
            );
        }
    }
}
